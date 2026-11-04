//! Thin wgpu view of the simulation-owned chunk and first-person camera.
//! No terrain, movement or world-state logic lives inside this module.

use crate::{
    materials_p0, materials_v2,
    mesh::{build_chunk_mesh, Vertex},
    texture_pack::TexturePack,
};
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use hexel_survival_game::{
    player::Player,
    surface_layers::SurfacePreset,
    world::{CellPos, World, CHUNK_SIDE},
};
use std::{borrow::Cow, sync::Arc, time::Instant};
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
}

#[derive(Default, Clone, Copy)]
pub struct DrawStats {
    pub visible_chunks: usize,
    pub visible_vertices: u64,
    /// CPU wall time for the draw path, including surface acquisition/present.
    /// This is not a GPU timestamp or a shader-only benchmark.
    pub draw_cpu_ms: f32,
}

pub struct Renderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
    crosshair_pipeline: wgpu::RenderPipeline,
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

        let mesh_start = Instant::now();
        let mut terrain_chunks = Vec::new();
        let mut total_vertices = 0u64;
        for pos in world.chunk_positions() {
            let mesh = build_chunk_mesh(world, pos);
            if mesh.is_empty() {
                continue;
            }
            let count = mesh.len() as u32;
            total_vertices += mesh.len() as u64;
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
            pipeline,
            crosshair_pipeline,
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

    pub fn draw(&mut self, player: &Player) -> Result<(), wgpu::SurfaceError> {
        let draw_start = Instant::now();
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
        let mut visible_chunks = 0usize;
        let mut visible_vertices = 0u64;
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
                visible_chunks += 1;
                visible_vertices += u64::from(chunk.vertices);
            }
            pass.set_pipeline(&self.crosshair_pipeline);
            pass.set_vertex_buffer(0, self.crosshair_buffer.slice(..));
            pass.draw(0..self.crosshair_vertices, 0..1);
        }
        self.queue.submit(std::iter::once(encoder.finish()));
        frame.present();
        self.last_stats = DrawStats {
            visible_chunks,
            visible_vertices,
            draw_cpu_ms: draw_start.elapsed().as_secs_f32() * 1000.0,
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
