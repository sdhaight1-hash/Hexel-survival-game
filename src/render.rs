//! Thin wgpu view of the simulation-owned chunk and first-person camera.
//! No terrain, movement or world-state logic lives inside this module.

use crate::{
    debug_menu::{self, MenuAction, MenuSnapshot, MenuVertex},
    seed_lod,
    materials_p0, materials_v2,
    mesh::{build_chunk_mesh_relative, resident_boundary_markers, torus_guides, Vertex},
    presentation::PresentationOptions,
    texture_pack::TexturePack,
};
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use hexel_survival_game::{
    player::Player,
    surface_layers::SurfacePreset,
    world::{ChunkPos, StreamDelta, World},
};
use std::{
    borrow::Cow,
    collections::{HashMap, VecDeque},
    sync::Arc,
    time::Instant,
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

struct TerrainGpuMesh {
    buffer: wgpu::Buffer,
    vertices: u32,
}

/// CPU-side mesh/build/upload submission metrics, not GPU execution time.
pub struct StreamGpuUpdate {
    pub meshed_buffers: usize,
    pub uploaded_bytes: u64,
    pub cpu_ms: f64,
}

/// Wall-clock CPU timings around rendering calls; these do not measure GPU execution.
#[derive(Clone, Copy, Debug, Default)]
pub struct DrawCpuTimings {
    pub total_ms: f64,
    pub prepare_ms: f64,
    pub acquire_ms: f64,
    pub encode_ms: f64,
    pub submit_ms: f64,
    pub present_ms: f64,
}

pub struct Renderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    presentation_report: String,
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
    surface_preset: SurfacePreset,
    metallic_glints: bool,
    terrain_meshes: HashMap<ChunkPos, TerrainGpuMesh>,
    seed_lod_worker: Option<seed_lod::Worker>,
    seed_lod_key: Option<seed_lod::Key>,
    seed_lod_buffers: Vec<(wgpu::Buffer, u32)>,
    boundary_markers_enabled: bool,
    boundary_marker_buffer: Option<wgpu::Buffer>,
    boundary_marker_vertices: u32,
    torus_guide_buffer: Option<wgpu::Buffer>,
    torus_guide_vertices: u32,
    torus_hud_buffer: Option<wgpu::Buffer>,
    torus_hud_vertices: u32,
    torus_hud_cache: Option<String>,
    /// Fixed axial origin shared by all opt-in distant-spawn GPU meshes.
    render_origin: Option<(i32, i32)>,
    crosshair_buffer: wgpu::Buffer,
    crosshair_vertices: u32,
    _depth_texture: wgpu::Texture,
    depth_view: wgpu::TextureView,
}

impl Renderer {
    pub async fn new(
        window: Arc<Window>,
        world: &World,
        presentation: PresentationOptions,
    ) -> Result<Self, String> {
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
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Hexel Survival device"),
                    required_features: if supports_bc7 {
                        wgpu::Features::TEXTURE_COMPRESSION_BC
                    } else {
                        wgpu::Features::empty()
                    },
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::Performance,
                },
                None,
            )
            .await
            .map_err(|e| format!("Could not initialize graphics device: {e}"))?;
        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(caps.formats[0]);
        let effective_mode = presentation.resolve_mode(&caps.present_modes);
        let adapter_info = adapter.get_info();
        let presentation_report = format!(
            "PRESENTATION: adapter {} ({:?}); requested {:?}, effective {:?}; supported {:?}; desired maximum frame latency {}; redraw {}; FPS cap {:?}.{}",
            adapter_info.name,
            adapter_info.backend,
            presentation.mode,
            effective_mode,
            caps.present_modes,
            presentation.max_frame_latency,
            if presentation.fps_cap.is_some() {
                "ControlFlow::WaitUntil"
            } else {
                "ControlFlow::Poll"
            },
            presentation.fps_cap,
            if effective_mode != presentation.mode {
                " Requested mode unsupported; fell back to FIFO."
            } else {
                ""
            }
        );
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: effective_mode,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: presentation.max_frame_latency,
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
        let v2_materials = match materials_v2::preview_layers(&packed) {
            Ok(Some(layers)) => {
                let v2_texture = device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("V2 candidate gallery preview (RGBA8)"),
                    size: wgpu::Extent3d {
                        width: packed.width,
                        height: packed.height,
                        depth_or_array_layers: packed.layers as u32,
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
                    "V2 candidate preview ready: F8 toggles rock/copper comparison against P0."
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

        let render_origin = presentation.spawn_cell;
        let mut terrain_meshes = HashMap::new();
        for pos in world.chunk_positions() {
            let mesh = build_chunk_mesh_relative(world, pos, render_origin);
            if mesh.is_empty() {
                continue;
            }
            let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Exposed hexel chunk faces"),
                contents: bytemuck::cast_slice(&mesh),
                usage: wgpu::BufferUsages::VERTEX,
            });
            terrain_meshes.insert(
                pos,
                TerrainGpuMesh {
                    buffer,
                    vertices: mesh.len() as u32,
                },
            );
        }
        if terrain_meshes.is_empty() {
            return Err("No terrain geometry was generated".to_string());
        }
        let guide_data = world.torus_frame().map(torus_guides).unwrap_or_default();
        let torus_guide_vertices = guide_data.len() as u32;
        let torus_guide_buffer = (!guide_data.is_empty()).then(|| {
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Global torus seam guides"),
                contents: bytemuck::cast_slice(&guide_data),
                usage: wgpu::BufferUsages::VERTEX,
            })
        });
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
            presentation_report,
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
            surface_preset: SurfacePreset::Dry,
            metallic_glints: true,
            terrain_meshes,
            seed_lod_worker: if presentation.distant_terrain {
                world.distant_surface_source().map(|(source, _)| seed_lod::Worker::spawn(source))
            } else {
                None
            },
            seed_lod_key: None,
            seed_lod_buffers: Vec::new(),
            boundary_markers_enabled: false,
            boundary_marker_buffer: None,
            boundary_marker_vertices: 0,
            torus_guide_buffer,
            torus_guide_vertices,
            torus_hud_buffer: None,
            torus_hud_vertices: 0,
            torus_hud_cache: None,
            render_origin,
            crosshair_buffer,
            crosshair_vertices: crosshair.len() as u32,
            _depth_texture: depth_texture,
            depth_view,
        })
    }

    /// Upload only newly resident sections, and drop evicted GPU buffers.
    /// Absent-neighbor face checks use the deterministic base, not air.
    /// An empty mesh still belongs to the resident CPU set, but needs no buffer.
    pub fn sync_stream(&mut self, world: &World, delta: &StreamDelta) -> StreamGpuUpdate {
        let start = Instant::now();
        debug_assert!(delta.removed.iter().all(|&pos| world.chunk(pos).is_none()));
        debug_assert!(delta.added.iter().all(|&pos| world.chunk(pos).is_some()));
        for pos in &delta.removed {
            self.terrain_meshes.remove(pos);
        }
        let mut meshed_buffers = 0usize;
        let mut uploaded_bytes = 0u64;
        for &pos in &delta.added {
            let mesh = build_chunk_mesh_relative(world, pos, self.render_origin);
            if mesh.is_empty() {
                continue;
            }
            uploaded_bytes += (mesh.len() * std::mem::size_of::<Vertex>()) as u64;
            let buffer = self
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Streamed hexel faces"),
                    contents: bytemuck::cast_slice(&mesh),
                    usage: wgpu::BufferUsages::VERTEX,
                });
            self.terrain_meshes.insert(
                pos,
                TerrainGpuMesh {
                    buffer,
                    vertices: mesh.len() as u32,
                },
            );
            meshed_buffers += 1;
        }
        debug_assert!(self
            .terrain_meshes
            .keys()
            .all(|&pos| world.chunk(pos).is_some()));
        StreamGpuUpdate {
            meshed_buffers,
            uploaded_bytes,
            cpu_ms: start.elapsed().as_secs_f64() * 1000.0,
        }
    }

    /// Rebase invalidates *all* local GPU vertex positions. Never try to
    /// preserve buffers by chunk key across frame changes.
    pub fn rebuild_torus_world(&mut self, world: &World) {
        self.terrain_meshes.clear();
        for pos in world.chunk_positions() {
            let vertices = build_chunk_mesh_relative(world, pos, self.render_origin);
            if vertices.is_empty() {
                continue;
            }
            let buffer = self
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Rebased periodic terrain mesh"),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });
            self.terrain_meshes.insert(
                pos,
                TerrainGpuMesh {
                    buffer,
                    vertices: vertices.len() as u32,
                },
            );
        }
        let guides = world.torus_frame().map(torus_guides).unwrap_or_default();
        self.torus_guide_vertices = guides.len() as u32;
        self.torus_guide_buffer = (!guides.is_empty()).then(|| {
            self.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Rebased global torus guides"),
                    contents: bytemuck::cast_slice(&guides),
                    usage: wgpu::BufferUsages::VERTEX,
                })
        });
    }

    /// F12: optional marker crosses at the perimeter of resident section columns.
    pub fn toggle_boundary_markers(&mut self, world: &World, eye_y: f32) -> bool {
        self.boundary_markers_enabled = !self.boundary_markers_enabled;
        self.refresh_boundary_markers(world, eye_y);
        self.boundary_markers_enabled
    }

    pub fn refresh_boundary_markers(&mut self, world: &World, eye_y: f32) {
        if !self.boundary_markers_enabled {
            self.boundary_marker_buffer = None;
            self.boundary_marker_vertices = 0;
            return;
        }
        let vertices = resident_boundary_markers(world, self.render_origin, eye_y);
        self.boundary_marker_vertices = vertices.len() as u32;
        self.boundary_marker_buffer = (!vertices.is_empty()).then(|| {
            self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("F12 resident boundary diagnostic markers"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            })
        });
    }

    /// Non-blocking mesh delivery: seed sampling and meshing run on a worker.
    pub fn refresh_distant_terrain(&mut self, world: &World, feet: Vec3) {
        let Some(worker) = self.seed_lod_worker.as_mut() else {
            return;
        };
        let Some((_, frame)) = world.distant_surface_source() else {
            return;
        };
        let desired = seed_lod::Key::at(feet, frame);
        if self.seed_lod_key == Some(desired) {
            return;
        }
        if let Some(completed) = worker.poll(desired) {
            self.seed_lod_buffers = completed.bands.into_iter().map(|vertices| {
                let buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Seed-first distant LOD band"),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });
                (buffer, vertices.len() as u32)
            }).collect();
            self.seed_lod_key = Some(desired);
        }
    }

    pub fn invalidate_distant_terrain(&mut self) {
        self.seed_lod_key = None;
        self.seed_lod_buffers.clear();
        if let Some(worker) = self.seed_lod_worker.as_mut() {
            worker.invalidate();
        }
    }

    pub fn presentation_report(&self) -> &str {
        &self.presentation_report
    }

    pub fn terrain_counts(&self) -> (usize, u64) {
        (
            self.terrain_meshes.len(),
            self.terrain_meshes
                .values()
                .map(|mesh| u64::from(mesh.vertices))
                .sum(),
        )
    }

    /// Toggle a validated candidate rock/ore comparison without changing P0.
    /// Returns None when no compatible candidate cache has been baked.
    pub fn toggle_v2_preview(&mut self) -> Option<bool> {
        self.v2_materials.as_ref()?;
        self.v2_preview_enabled = !self.v2_preview_enabled;
        Some(self.v2_preview_enabled)
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

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        if size.width == 0 || size.height == 0 {
            return;
        }
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);
        let (texture, view) = create_depth(&self.device, size.width, size.height);
        self._depth_texture = texture;
        self.depth_view = view;
        self.menu_cache = None;
        self.console_cache_generation = None;
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

    pub fn menu_snapshot(&self, frame_stats: bool) -> MenuSnapshot {
        let has_art = self.v2_materials.is_some();
        MenuSnapshot {
            console_open: false,
            frame_stats,
            chunk_culling: None,
            unlocked_present: None,
            gpu_timestamps: false,
            uv_checker: self.debug_checker,
            uv_density: self.texels_per_metre,
            v2_materials: has_art.then_some(self.v2_preview_enabled),
            surface: has_art.then_some(self.surface_preset.label()),
            metal_glints: has_art.then_some(self.metallic_glints),
            gray_copper: None,
        }
    }

    pub fn draw(
        &mut self,
        player: &Player,
        menu: Option<(MenuSnapshot, Option<MenuAction>)>,
        console: Option<(&VecDeque<String>, u64)>,
        torus_hud: Option<&str>,
    ) -> Result<DrawCpuTimings, wgpu::SurfaceError> {
        let draw_start = Instant::now();
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

        if self.torus_hud_cache.as_deref() != torus_hud {
            self.torus_hud_cache = torus_hud.map(str::to_owned);
            let vertices = torus_hud.map_or_else(Vec::new, |status| {
                debug_menu::torus_hud_geometry(self.config.width, self.config.height, status)
            });
            self.torus_hud_vertices = vertices.len() as u32;
            self.torus_hud_buffer = (!vertices.is_empty()).then(|| {
                self.device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Global torus HUD"),
                        contents: bytemuck::cast_slice(&vertices),
                        usage: wgpu::BufferUsages::VERTEX,
                    })
            });
        }
        let aspect = self.config.width as f32 / self.config.height as f32;
        let absolute_eye = player.eye_position();
        // Vertex buffers for opt-in distant spawns use integer-axial-relative
        // metre positions; the camera must use exactly the same origin.
        // Player position remains global f32 for now (movement precision is
        // a separate milestone), but mesh corner precision is restored.
        let eye = if let Some((q, r)) = self.render_origin {
            let x = f64::from(absolute_eye.x) - f64::from(q) - f64::from(r) * 0.5;
            let z = f64::from(absolute_eye.z)
                - f64::from(r) * f64::from(hexel_survival_game::world::HEX_ROW_SPACING);
            Vec3::new(x as f32, absolute_eye.y, z as f32)
        } else {
            absolute_eye
        };
        let view_proj = Mat4::perspective_rh(70.0_f32.to_radians(), aspect, 0.05, if self.seed_lod_worker.is_some() { 1000.0 } else { 200.0 })
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
                    0.0,
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
        let acquire_start = Instant::now();
        let frame = self.surface.get_current_texture()?;
        let encode_start = Instant::now();
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("World frame"),
            });
        {
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
                timestamp_writes: None,
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
            for (buffer, vertices) in &self.seed_lod_buffers {
                pass.set_vertex_buffer(0, buffer.slice(..));
                pass.draw(0..*vertices, 0..1);
            }
            for mesh in self.terrain_meshes.values() {
                pass.set_vertex_buffer(0, mesh.buffer.slice(..));
                pass.draw(0..mesh.vertices, 0..1);
            }
            if let Some(buffer) = &self.boundary_marker_buffer {
                pass.set_vertex_buffer(0, buffer.slice(..));
                pass.draw(0..self.boundary_marker_vertices, 0..1);
            }
            if let Some(buffer) = &self.torus_guide_buffer {
                pass.set_vertex_buffer(0, buffer.slice(..));
                pass.draw(0..self.torus_guide_vertices, 0..1);
            }
            if menu.is_none() && console.is_none() {
                if let Some(buffer) = &self.torus_hud_buffer {
                    pass.set_pipeline(&self.menu_pipeline);
                    pass.set_vertex_buffer(0, buffer.slice(..));
                    pass.draw(0..self.torus_hud_vertices, 0..1);
                }
                pass.set_pipeline(&self.crosshair_pipeline);
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
        let command_buffer = encoder.finish();
        let submit_start = Instant::now();
        self.queue.submit(std::iter::once(command_buffer));
        let present_start = Instant::now();
        frame.present();
        let finished = Instant::now();
        Ok(DrawCpuTimings {
            total_ms: finished.duration_since(draw_start).as_secs_f64() * 1000.0,
            prepare_ms: acquire_start.duration_since(draw_start).as_secs_f64() * 1000.0,
            acquire_ms: encode_start.duration_since(acquire_start).as_secs_f64() * 1000.0,
            encode_ms: submit_start.duration_since(encode_start).as_secs_f64() * 1000.0,
            submit_ms: present_start.duration_since(submit_start).as_secs_f64() * 1000.0,
            present_ms: finished.duration_since(present_start).as_secs_f64() * 1000.0,
        })
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
