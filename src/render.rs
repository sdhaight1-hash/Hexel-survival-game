//! Thin wgpu view of the simulation-owned chunk and first-person camera.
//! No terrain, movement or world-state logic lives inside this module.

use crate::{
    debug_menu::{self, MenuAction, MenuSnapshot, MenuVertex},
    materials_p0, materials_v2,
    mesh::{build_chunk_mesh_with_stats, ExposedMeshStats, Vertex},
    texture_pack::TexturePack,
};
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use hexel_survival_game::{
    player::Player,
    surface_layers::SurfacePreset,
    world::{CellPos, World, CHUNK_SIDE},
};
use std::{
    borrow::Cow,
    collections::VecDeque,
    sync::{mpsc, Arc},
    time::{Duration, Instant},
};
use wgpu::util::DeviceExt;
use winit::{dpi::PhysicalSize, window::Window};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],
    options: [f32; 4],
    eye_position: [f32; 4],
    surface: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct CrosshairVertex {
    position: [f32; 2],
}

impl CrosshairVertex {
    const ATTRIBUTES: [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array![0 => Float32x2];

    fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

struct ChunkMesh {
    buffer: wgpu::Buffer,
    vertices: u32,
    center: Vec3,
    exposed: ExposedMeshStats,
}

#[derive(Default, Clone, Copy)]
pub struct DrawStats {
    pub submitted_chunks: usize,
    pub submitted_vertices: u64,
    pub submitted_exposed_hexels: u64,
    pub submitted_cap_faces: u64,
    pub submitted_side_faces: u64,
    /// CPU wall time includes surface acquisition/present, and may block.
    pub draw_cpu_ms: f32,
    /// Asynchronously retrieved GPU world render-pass time for one sampled
    /// frame. Not a full-frame latency, CPU time or isolated shader time.
    pub gpu_pass_ms: Option<f32>,
}

/// One in-flight render-pass timestamp readback. Never blocks gameplay
/// waiting for a GPU result; unsupported adapters simply have no timer.
struct GpuTimer {
    queries: wgpu::QuerySet,
    resolved: wgpu::Buffer,
    readback: wgpu::Buffer,
    pending: Option<mpsc::Receiver<Result<(), wgpu::BufferAsyncError>>>,
    last_request: Instant,
    period_ns: f32,
}

impl GpuTimer {
    fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        Self {
            queries: device.create_query_set(&wgpu::QuerySetDescriptor {
                label: Some("World pass timestamps"),
                ty: wgpu::QueryType::Timestamp,
                count: 2,
            }),
            resolved: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("GPU timestamp resolve"),
                size: 16,
                usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            }),
            readback: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("GPU timestamp staging"),
                size: 16,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            }),
            pending: None,
            last_request: Instant::now(),
            period_ns: queue.get_timestamp_period(),
        }
    }

    fn sample_ready(&mut self, device: &wgpu::Device) -> Option<f32> {
        let pending = self.pending.as_ref()?;
        // Poll for callbacks without waiting on the GPU.
        device.poll(wgpu::Maintain::Poll);
        let completed = match pending.try_recv() {
            Ok(done) => done,
            Err(mpsc::TryRecvError::Empty) => return None,
            Err(mpsc::TryRecvError::Disconnected) => {
                self.pending = None;
                return None;
            }
        };
        self.pending = None;
        if completed.is_err() {
            return None;
        }
        let mapped = self.readback.slice(..).get_mapped_range();
        let start = u64::from_le_bytes(mapped[0..8].try_into().unwrap());
        let end = u64::from_le_bytes(mapped[8..16].try_into().unwrap());
        drop(mapped);
        self.readback.unmap();
        (end > start).then_some((end - start) as f32 * self.period_ns / 1_000_000.0)
    }

    fn should_sample(&self) -> bool {
        self.pending.is_none() && self.last_request.elapsed() >= Duration::from_millis(250)
    }

    fn begin_readback(&mut self) {
        let (send, receive) = mpsc::channel();
        self.readback
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |result| {
                let _ = send.send(result);
            });
        self.pending = Some(receive);
        self.last_request = Instant::now();
    }
}

pub struct Renderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    hardware_report: String,
    geometry_report: String,
    supports_unlocked_present: bool,
    gpu_timer: Option<GpuTimer>,
    gpu_profiling_enabled: bool,
    pipeline: wgpu::RenderPipeline,
    crosshair_pipeline: wgpu::RenderPipeline,
    menu_pipeline: wgpu::RenderPipeline,
    menu_buffer: Option<wgpu::Buffer>,
    menu_vertex_count: u32,
    menu_cache: Option<(MenuSnapshot, Option<MenuAction>)>,
    console_buffer: Option<wgpu::Buffer>,
    console_vertex_count: u32,
    console_cache_generation: Option<u64>,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    debug_checker: bool,
    texels_per_metre: u32,
    material_bind_group: wgpu::BindGroup,
    _material_texture: wgpu::Texture,
    v2_materials: Option<(wgpu::Texture, wgpu::BindGroup)>,
    v2_preview_enabled: bool,
    grayscale_copper_available: bool,
    grayscale_copper_enabled: bool,
    surface_preset: SurfacePreset,
    metallic_glints: bool,
    terrain_chunks: Vec<ChunkMesh>,
    chunk_culling: bool,
    last_stats: DrawStats,
    crosshair_buffer: wgpu::Buffer,
    crosshair_vertices: u32,
    _depth_texture: wgpu::Texture,
    depth_view: wgpu::TextureView,
}

impl Renderer {
    pub async fn new(window: Arc<Window>, world: &World) -> Result<Self, String> {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        let surface = instance
            .create_surface(window)
            .map_err(|e| format!("Could not create rendering surface: {e}"))?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| "No compatible graphics adapter detected".to_string())?;
        // Only request BC compression when the actual adapter exposes it.
        // BC7 is GPU-decoded in hardware, while unsupported GPUs get a
        // single CPU BC7 -> RGBA decode during initialization.
        let supports_bc7 = adapter
            .features()
            .contains(wgpu::Features::TEXTURE_COMPRESSION_BC);
        let supports_timestamps = adapter.features().contains(wgpu::Features::TIMESTAMP_QUERY);
        let mut required_features = wgpu::Features::empty();
        if supports_bc7 {
            required_features |= wgpu::Features::TEXTURE_COMPRESSION_BC;
        }
        if supports_timestamps {
            required_features |= wgpu::Features::TIMESTAMP_QUERY;
        }
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Hexel Survival device"),
                    required_features,
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::Performance,
                },
                None,
            )
            .await
            .map_err(|e| format!("Could not initialize graphics device: {e}"))?;
        let caps = surface.get_capabilities(&adapter);
        let supports_unlocked_present =
            world.is_benchmark() && caps.present_modes.contains(&wgpu::PresentMode::Immediate);
        let gpu_timer = supports_timestamps.then(|| GpuTimer::new(&device, &queue));
        println!(
            "Benchmark adapter: {} | F5 Immediate: {} | GPU timestamps: {} (sampled world render pass only)",
            adapter.get_info().name,
            if supports_unlocked_present { "available" } else { "unavailable" },
            if supports_timestamps { "available" } else { "unavailable" }
        );
        let info = adapter.get_info();
        let hardware_report = format!(
            "GPU: {} | type {:?} | vendor {:04x} device {:04x} | backend {:?} | driver {} {} | BC7 {} | timestamp {} | Immediate {}",
            info.name,
            info.device_type,
            info.vendor,
            info.device,
            info.backend,
            info.driver,
            info.driver_info.replace('\n', " "),
            supports_bc7,
            supports_timestamps,
            supports_unlocked_present,
        );
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera matrix"),
            contents: bytemuck::bytes_of(&CameraUniform {
                view_proj: Mat4::IDENTITY.to_cols_array_2d(),
                options: [0.0, 1.0, 0.0, 0.0],
                eye_position: [0.0, 0.0, 0.0, 0.0],
                surface: [0.0, 0.0, 0.0, 1.0],
            }),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let camera_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Camera binding layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera bind group"),
            layout: &camera_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });
        // Compression happened in build.rs: startup never encodes texture
        // blocks or resamples the original high-resolution source PNGs.
        let packed = TexturePack::embedded()
            .map_err(|e| format!("Invalid embedded material BC7 pack: {e}"))?;
        if (packed.width, packed.height, packed.layers)
            != (
                materials_p0::TEXTURE_SIZE,
                materials_p0::TEXTURE_SIZE,
                materials_p0::LAYER_COUNT,
            )
        {
            return Err("P0 material pack dimensions/layer count do not match the mesh".into());
        }
        let texture_format = if supports_bc7 {
            wgpu::TextureFormat::Bc7RgbaUnormSrgb
        } else {
            wgpu::TextureFormat::Rgba8UnormSrgb
        };
        println!(
            "Material textures: {} path; {:.2} MiB BC7 vs {:.2} MiB RGBA ({} GPU memory reduction)",
            if supports_bc7 {
                "hardware BC7"
            } else {
                "CPU BC7 decode -> RGBA fallback"
            },
            packed.bc7_bytes() as f64 / 1_048_576.0,
            packed.rgba_bytes() as f64 / 1_048_576.0,
            if supports_bc7 {
                "75%"
            } else {
                "0% on this adapter"
            },
        );
        let material_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(if supports_bc7 {
                "Hexel BC7 materials"
            } else {
                "Hexel RGBA fallback materials"
            }),
            size: wgpu::Extent3d {
                width: packed.width,
                height: packed.height,
                depth_or_array_layers: packed.layers as u32,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: texture_format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        for layer in 0..packed.layers {
            let pixels: Cow<'_, [u8]> = if supports_bc7 {
                Cow::Borrowed(packed.bc7_layer(layer))
            } else {
                Cow::Owned(
                    packed
                        .decode_layer(layer)
                        .map_err(|e| format!("BC7 fallback decode layer {layer}: {e}"))?,
                )
            };
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &material_texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: 0,
                        z: layer as u32,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                &pixels,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(if supports_bc7 {
                        (packed.width / 4) * 16 // 4x4 BC7 blocks, 16 bytes per block
                    } else {
                        packed.width * 4
                    }),
                    rows_per_image: Some(if supports_bc7 {
                        packed.height / 4
                    } else {
                        packed.height
                    }),
                },
                wgpu::Extent3d {
                    width: packed.width,
                    height: packed.height,
                    depth_or_array_layers: 1,
                },
            );
        }
        let material_view = material_texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });
        let material_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("P0 physically tiled nearest-neighbor materials"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        let material_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Hexel material array layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let material_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Hexel material array"),
            layout: &material_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&material_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&material_sampler),
                },
            ],
        });
        // Optional F8 candidate-art comparison. It never replaces or edits
        // the embedded, GPU-compressed P0 texture array.
        let mut grayscale_copper_available = false;
        let v2_materials = match materials_v2::preview_layers(&packed) {
            Ok(Some(layers)) => {
                grayscale_copper_available = layers.len() > packed.layers as usize;
                let v2_texture = device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("V2 candidate gallery preview (RGBA8)"),
                    size: wgpu::Extent3d {
                        width: packed.width,
                        height: packed.height,
                        depth_or_array_layers: layers.len() as u32,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                    view_formats: &[],
                });
                for (layer, pixels) in layers.iter().enumerate() {
                    queue.write_texture(
                        wgpu::TexelCopyTextureInfo {
                            texture: &v2_texture,
                            mip_level: 0,
                            origin: wgpu::Origin3d {
                                x: 0,
                                y: 0,
                                z: layer as u32,
                            },
                            aspect: wgpu::TextureAspect::All,
                        },
                        pixels,
                        wgpu::TexelCopyBufferLayout {
                            offset: 0,
                            bytes_per_row: Some(packed.width * 4),
                            rows_per_image: Some(packed.height),
                        },
                        wgpu::Extent3d {
                            width: packed.width,
                            height: packed.height,
                            depth_or_array_layers: 1,
                        },
                    );
                }
                let v2_view = v2_texture.create_view(&wgpu::TextureViewDescriptor {
                    dimension: Some(wgpu::TextureViewDimension::D2Array),
                    ..Default::default()
                });
                let v2_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("V2 candidate gallery bind group"),
                    layout: &material_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&v2_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&material_sampler),
                        },
                    ],
                });
                println!(
                    "V2 candidate preview ready: F8 toggles rock/copper against P0; F11 compares shader palette copper."
                );
                Some((v2_texture, v2_bind_group))
            }
            Ok(None) => None,
            Err(error) => {
                eprintln!("V2 preview unavailable ({error}); P0 gallery remains active.");
                None
            }
        };
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("World shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("world_shader.wgsl").into()),
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("World pipeline layout"),
            bind_group_layouts: &[&camera_layout, &material_layout],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Hexel terrain"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::layout()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth24Plus,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let crosshair_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Crosshair shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("crosshair.wgsl").into()),
        });
        let crosshair_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Crosshair pipeline layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });
        let crosshair_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Screen crosshair"),
            layout: Some(&crosshair_layout),
            vertex: wgpu::VertexState {
                module: &crosshair_shader,
                entry_point: Some("vs_main"),
                buffers: &[CrosshairVertex::layout()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &crosshair_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList,
                cull_mode: None,
                ..Default::default()
            },
            // The pass has a Depth24Plus attachment, so this pipeline must
            // declare the same format. Keep the HUD visible regardless of
            // terrain depth without modifying the depth buffer.
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth24Plus,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Always,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let menu_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Debug menu shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("debug_menu.wgsl").into()),
        });
        let menu_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Developer diagnostics overlay"),
            layout: Some(&crosshair_layout),
            vertex: wgpu::VertexState {
                module: &menu_shader,
                entry_point: Some("vs_main"),
                buffers: &[MenuVertex::layout()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &menu_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth24Plus,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Always,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let mesh_start = Instant::now();
        let mut terrain_chunks = Vec::new();
        let mut total_vertices = 0u64;
        let mut solid_hexels = 0u64;
        let mut exposed_hexels = 0u64;
        let mut cap_faces = 0u64;
        let mut side_faces = 0u64;
        for pos in world.chunk_positions() {
            let (mesh, exposed) = build_chunk_mesh_with_stats(world, pos);
            if mesh.is_empty() {
                continue;
            }
            let count = mesh.len() as u32;
            total_vertices += mesh.len() as u64;
            solid_hexels += u64::from(exposed.solid_hexels);
            exposed_hexels += u64::from(exposed.exposed_hexels);
            cap_faces += u64::from(exposed.cap_faces);
            side_faces += u64::from(exposed.side_faces);
            let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Exposed hexel chunk faces"),
                contents: bytemuck::cast_slice(&mesh),
                usage: wgpu::BufferUsages::VERTEX,
            });
            let cell = CellPos::new(
                pos.q * CHUNK_SIDE + CHUNK_SIDE / 2,
                pos.r * CHUNK_SIDE + CHUNK_SIDE / 2,
                pos.y * CHUNK_SIDE,
            );
            terrain_chunks.push(ChunkMesh {
                buffer,
                vertices: count,
                center: cell.center(),
                exposed,
            });
        }
        if terrain_chunks.is_empty() {
            return Err("No terrain geometry was generated".to_string());
        }
        println!(
            "Terrain meshes: {} loaded chunk(s), {} vertices, {:.1} MiB vertex buffers, {:.1} ms CPU build/upload (not GPU time).",
            terrain_chunks.len(),
            total_vertices,
            total_vertices as f64 * std::mem::size_of::<Vertex>() as f64 / (1024.0 * 1024.0),
            mesh_start.elapsed().as_secs_f64() * 1000.0
        );
        let mesh_ms = mesh_start.elapsed().as_secs_f64() * 1000.0;
        let geometry_report = format!(
            "SCENE resident {} chunks / {} cells | solid {} hexels | exposed {} hexels | exposed caps {} sides {} ({} faces) | triangles {} | vertices {} | vertex buffers {:.1} MiB | build/upload {:.1} ms CPU",
            world.chunk_count(),
            world.chunk_count() * 16 * 16 * 16,
            solid_hexels,
            exposed_hexels,
            cap_faces,
            side_faces,
            cap_faces + side_faces,
            total_vertices / 3,
            total_vertices,
            total_vertices as f64 * std::mem::size_of::<Vertex>() as f64 / (1024.0 * 1024.0),
            mesh_ms,
        );
        let crosshair = [
            CrosshairVertex {
                position: [-0.013, 0.0],
            },
            CrosshairVertex {
                position: [-0.003, 0.0],
            },
            CrosshairVertex {
                position: [0.003, 0.0],
            },
            CrosshairVertex {
                position: [0.013, 0.0],
            },
            CrosshairVertex {
                position: [0.0, -0.022],
            },
            CrosshairVertex {
                position: [0.0, -0.005],
            },
            CrosshairVertex {
                position: [0.0, 0.005],
            },
            CrosshairVertex {
                position: [0.0, 0.022],
            },
        ];
        let crosshair_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("First-person crosshair"),
            contents: bytemuck::cast_slice(&crosshair),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let (depth_texture, depth_view) = create_depth(&device, config.width, config.height);
        Ok(Self {
            surface,
            device,
            queue,
            config,
            hardware_report,
            geometry_report,
            supports_unlocked_present,
            gpu_timer,
            gpu_profiling_enabled: false,
            pipeline,
            crosshair_pipeline,
            menu_pipeline,
            menu_buffer: None,
            menu_vertex_count: 0,
            menu_cache: None,
            console_buffer: None,
            console_vertex_count: 0,
            console_cache_generation: None,
            camera_buffer,
            camera_bind_group,
            debug_checker: false,
            texels_per_metre: 128,
            material_bind_group,
            _material_texture: material_texture,
            v2_materials,
            v2_preview_enabled: false,
            grayscale_copper_available,
            grayscale_copper_enabled: false,
            surface_preset: SurfacePreset::Dry,
            metallic_glints: true,
            terrain_chunks,
            chunk_culling: true,
            last_stats: DrawStats::default(),
            crosshair_buffer,
            crosshair_vertices: crosshair.len() as u32,
            _depth_texture: depth_texture,
            depth_view,
        })
    }

    /// Toggle a validated candidate rock/ore comparison without changing P0.
    /// Returns None when no compatible candidate cache has been baked.
    pub fn toggle_v2_preview(&mut self) -> Option<bool> {
        self.v2_materials.as_ref()?;
        self.v2_preview_enabled = !self.v2_preview_enabled;
        if !self.v2_preview_enabled {
            self.grayscale_copper_enabled = false;
        }
        Some(self.v2_preview_enabled)
    }

    /// A/B copper: F8 original full-color composite vs F11 shared grayscale
    /// mask and shader palette. Only available with F8 and a copper source.
    pub fn toggle_grayscale_copper(&mut self) -> Option<bool> {
        if !self.v2_preview_enabled || !self.grayscale_copper_available {
            return None;
        }
        self.grayscale_copper_enabled = !self.grayscale_copper_enabled;
        Some(self.grayscale_copper_enabled)
    }

    /// Change only the F8 material gallery, not saved weather or ecology.
    pub fn cycle_surface_preset(&mut self) -> Option<&'static str> {
        self.v2_materials.as_ref()?;
        self.surface_preset = self.surface_preset.next();
        Some(self.surface_preset.label())
    }

    /// A/B compare ore specular response without changing ore color or grade.
    pub fn toggle_metallic_glints(&mut self) -> Option<bool> {
        self.v2_materials.as_ref()?;
        self.metallic_glints = !self.metallic_glints;
        Some(self.metallic_glints)
    }

    /// F4: draw all resident chunks or limit to nearby chunks.
    /// Both modes keep the same 200 loaded/generated meshes.
    pub fn toggle_chunk_culling(&mut self) -> Option<bool> {
        if self.terrain_chunks.len() <= 1 {
            return None;
        }
        self.chunk_culling = !self.chunk_culling;
        Some(self.chunk_culling)
    }

    /// F5 is a benchmark-only, explicitly supported Immediate swap mode.
    /// Never silently fall back to Mailbox/FIFO while claiming uncapped.
    pub fn toggle_present_mode(&mut self) -> Option<bool> {
        if !self.supports_unlocked_present {
            return None;
        }
        let unlocked = self.config.present_mode == wgpu::PresentMode::Fifo;
        self.config.present_mode = if unlocked {
            wgpu::PresentMode::Immediate
        } else {
            wgpu::PresentMode::Fifo
        };
        self.surface.configure(&self.device, &self.config);
        Some(unlocked)
    }

    pub fn presentation_label(&self) -> &'static str {
        if self.config.present_mode == wgpu::PresentMode::Immediate {
            "IMMEDIATE (unlocked)"
        } else {
            "FIFO (vsync)"
        }
    }

    pub fn set_gpu_profiling(&mut self, enabled: bool) {
        self.gpu_profiling_enabled = enabled;
    }

    pub fn gpu_timestamps_supported(&self) -> bool {
        self.gpu_timer.is_some()
    }

    pub fn menu_snapshot(&self, frame_stats: bool) -> MenuSnapshot {
        let has_art = self.v2_materials.is_some();
        MenuSnapshot {
            console_open: false,
            frame_stats,
            chunk_culling: (self.terrain_chunks.len() > 1).then_some(self.chunk_culling),
            unlocked_present: self
                .supports_unlocked_present
                .then_some(self.config.present_mode == wgpu::PresentMode::Immediate),
            gpu_timestamps: self.gpu_timer.is_some(),
            uv_checker: self.debug_checker,
            uv_density: self.texels_per_metre,
            v2_materials: has_art.then_some(self.v2_preview_enabled),
            surface: has_art.then_some(self.surface_preset.label()),
            metal_glints: has_art.then_some(self.metallic_glints),
            gray_copper: (self.v2_preview_enabled && self.grayscale_copper_available)
                .then_some(self.grayscale_copper_enabled),
        }
    }

    pub fn hardware_report(&self) -> &str {
        &self.hardware_report
    }

    pub fn geometry_report(&self) -> &str {
        &self.geometry_report
    }

    pub fn viewport_report(&self) -> String {
        format!(
            "DISPLAY render viewport {}x{} | swap format {:?} | present {} | GPU pass sampling {}",
            self.config.width,
            self.config.height,
            self.config.format,
            self.presentation_label(),
            self.gpu_timer.is_some(),
        )
    }

    pub fn stats(&self) -> DrawStats {
        self.last_stats
    }

    pub fn material_comparison_label(&self) -> &'static str {
        if !self.v2_preview_enabled {
            "P0"
        } else if self.grayscale_copper_enabled {
            "F11 grayscale"
        } else {
            "F8 full-color"
        }
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        if size.width == 0 || size.height == 0 {
            return;
        }
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);
        self.menu_cache = None; // pixel-space panels must be rebuilt after resize
        self.console_cache_generation = None;
        let (texture, view) = create_depth(&self.device, size.width, size.height);
        self._depth_texture = texture;
        self.depth_view = view;
    }

    /// Show a high-contrast 16px-grid on every cap and prism side.
    pub fn toggle_checker(&mut self) -> bool {
        self.debug_checker = !self.debug_checker;
        self.debug_checker
    }

    /// Compare physical texture scale of the diagnostic grid only.
    pub fn cycle_density(&mut self) -> u32 {
        self.texels_per_metre = match self.texels_per_metre {
            64 => 128,
            128 => 256,
            _ => 64,
        };
        self.texels_per_metre
    }

    pub fn draw(
        &mut self,
        player: &Player,
        menu: Option<(MenuSnapshot, Option<MenuAction>)>,
        console: Option<(&VecDeque<String>, u64)>,
    ) -> Result<(), wgpu::SurfaceError> {
        let draw_start = Instant::now();
        let gpu_sample_ms = self
            .gpu_timer
            .as_mut()
            .and_then(|timer| timer.sample_ready(&self.device));
        let timestamp_this_frame = self.gpu_profiling_enabled
            && self.gpu_timer.as_ref().is_some_and(GpuTimer::should_sample);
        if let Some((snapshot, hovered)) = menu {
            if self.menu_cache != Some((snapshot, hovered)) {
                let vertices =
                    debug_menu::geometry(self.config.width, self.config.height, snapshot, hovered);
                self.menu_vertex_count = vertices.len() as u32;
                self.menu_buffer = Some(self.device.create_buffer_init(
                    &wgpu::util::BufferInitDescriptor {
                        label: Some("Cached diagnostics menu triangles"),
                        contents: bytemuck::cast_slice(&vertices),
                        usage: wgpu::BufferUsages::VERTEX,
                    },
                ));
                self.menu_cache = Some((snapshot, hovered));
            }
        } else if self.menu_cache.is_some() {
            self.menu_cache = None;
            self.menu_buffer = None;
            self.menu_vertex_count = 0;
        }
        if let Some((lines, generation)) = console {
            if self.console_cache_generation != Some(generation) {
                let vertices =
                    debug_menu::console_geometry(self.config.width, self.config.height, lines);
                self.console_vertex_count = vertices.len() as u32;
                self.console_buffer = if vertices.is_empty() {
                    None
                } else {
                    Some(
                        self.device
                            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                label: Some("Cached bottom benchmark log"),
                                contents: bytemuck::cast_slice(&vertices),
                                usage: wgpu::BufferUsages::VERTEX,
                            }),
                    )
                };
                self.console_cache_generation = Some(generation);
            }
        } else if self.console_cache_generation.is_some() {
            self.console_cache_generation = None;
            self.console_buffer = None;
            self.console_vertex_count = 0;
        }
        let aspect = self.config.width as f32 / self.config.height as f32;
        let eye = player.eye_position();
        let view_proj = Mat4::perspective_rh(70.0_f32.to_radians(), aspect, 0.05, 200.0)
            * Mat4::look_to_rh(eye, player.forward(), Vec3::Y);
        let conditions = self.surface_preset.conditions();
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::bytes_of(&CameraUniform {
                view_proj: view_proj.to_cols_array_2d(),
                options: [
                    if self.debug_checker { 1.0 } else { 0.0 },
                    self.texels_per_metre as f32 / materials_p0::REFERENCE_TEXELS_PER_METRE,
                    if self.v2_preview_enabled { 1.0 } else { 0.0 },
                    if self.v2_preview_enabled && self.grayscale_copper_enabled {
                        1.0
                    } else {
                        0.0
                    },
                ],
                eye_position: [eye.x, eye.y, eye.z, 0.0],
                surface: [
                    conditions.wetness,
                    conditions.moss,
                    conditions.frost,
                    if self.metallic_glints { 1.0 } else { 0.0 },
                ],
            }),
        );
        let frame = self.surface.get_current_texture()?;
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut submitted_chunks = 0usize;
        let mut submitted_vertices = 0u64;
        let mut submitted_exposed_hexels = 0u64;
        let mut submitted_cap_faces = 0u64;
        let mut submitted_side_faces = 0u64;
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("World frame"),
            });
        {
            let timestamp_writes = if timestamp_this_frame {
                self.gpu_timer
                    .as_ref()
                    .map(|timer| wgpu::RenderPassTimestampWrites {
                        query_set: &timer.queries,
                        beginning_of_pass_write_index: Some(0),
                        end_of_pass_write_index: Some(1),
                    })
            } else {
                None
            };
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("First-person world pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.50,
                            g: 0.70,
                            b: 0.87,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.camera_bind_group, &[]);
            let material_group = if self.v2_preview_enabled {
                self.v2_materials
                    .as_ref()
                    .map(|(_, group)| group)
                    .unwrap_or(&self.material_bind_group)
            } else {
                &self.material_bind_group
            };
            pass.set_bind_group(1, material_group, &[]);
            for chunk in &self.terrain_chunks {
                let dx = chunk.center.x - eye.x;
                let dz = chunk.center.z - eye.z;
                // Conservative static distance culling, with extra room for
                // skewed axial chunks. Not a camera-frustum or occlusion system.
                if self.chunk_culling && dx * dx + dz * dz > 112.0 * 112.0 {
                    continue;
                }
                pass.set_vertex_buffer(0, chunk.buffer.slice(..));
                pass.draw(0..chunk.vertices, 0..1);
                submitted_chunks += 1;
                submitted_vertices += u64::from(chunk.vertices);
                submitted_exposed_hexels += u64::from(chunk.exposed.exposed_hexels);
                submitted_cap_faces += u64::from(chunk.exposed.cap_faces);
                submitted_side_faces += u64::from(chunk.exposed.side_faces);
            }
            pass.set_pipeline(&self.crosshair_pipeline);
            if menu.is_none() && console.is_none() {
                pass.set_vertex_buffer(0, self.crosshair_buffer.slice(..));
                pass.draw(0..self.crosshair_vertices, 0..1);
            }
            if let Some(buffer) = self.menu_buffer.as_ref().filter(|_| menu.is_some()) {
                pass.set_pipeline(&self.menu_pipeline);
                pass.set_vertex_buffer(0, buffer.slice(..));
                pass.draw(0..self.menu_vertex_count, 0..1);
            }
            if let Some(buffer) = self.console_buffer.as_ref().filter(|_| console.is_some()) {
                pass.set_pipeline(&self.menu_pipeline);
                pass.set_vertex_buffer(0, buffer.slice(..));
                pass.draw(0..self.console_vertex_count, 0..1);
            }
        }
        if timestamp_this_frame {
            if let Some(timer) = self.gpu_timer.as_ref() {
                encoder.resolve_query_set(&timer.queries, 0..2, &timer.resolved, 0);
                encoder.copy_buffer_to_buffer(&timer.resolved, 0, &timer.readback, 0, 16);
            }
        }
        self.queue.submit(std::iter::once(encoder.finish()));
        frame.present();
        if timestamp_this_frame {
            if let Some(timer) = self.gpu_timer.as_mut() {
                timer.begin_readback();
            }
        }
        self.last_stats = DrawStats {
            submitted_chunks,
            submitted_vertices,
            submitted_exposed_hexels,
            submitted_cap_faces,
            submitted_side_faces,
            draw_cpu_ms: draw_start.elapsed().as_secs_f32() * 1000.0,
            gpu_pass_ms: gpu_sample_ms,
        };
        Ok(())
    }
}

fn create_depth(
    device: &wgpu::Device,
    width: u32,
    height: u32,
) -> (wgpu::Texture, wgpu::TextureView) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Depth target"),
        size: wgpu::Extent3d {
            width: width.max(1),
            height: height.max(1),
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth24Plus,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    (texture, view)
}
