//! Thin wgpu view of the simulation-owned chunk and first-person camera.
//! No terrain, movement or world-state logic lives inside this module.

use crate::mesh::{build_chunk_mesh, Vertex};
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use hexel_survival_game::{
    player::Player,
    world::{ChunkPos, World},
};
use std::sync::Arc;
use wgpu::util::DeviceExt;
use winit::{dpi::PhysicalSize, window::Window};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],
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

pub struct Renderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
    crosshair_pipeline: wgpu::RenderPipeline,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    terrain_buffer: wgpu::Buffer,
    terrain_vertices: u32,
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
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Hexel Survival device"),
                    required_features: wgpu::Features::empty(),
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
            }),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let camera_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Camera binding layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
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
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("World shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("world_shader.wgsl").into()),
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("World pipeline layout"),
            bind_group_layouts: &[&camera_layout],
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

        let mesh = build_chunk_mesh(world, ChunkPos::default());
        if mesh.is_empty() {
            return Err("No terrain geometry was generated".to_string());
        }
        let terrain_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Exposed hexel faces"),
            contents: bytemuck::cast_slice(&mesh),
            usage: wgpu::BufferUsages::VERTEX,
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
            pipeline,
            crosshair_pipeline,
            camera_buffer,
            camera_bind_group,
            terrain_buffer,
            terrain_vertices: mesh.len() as u32,
            crosshair_buffer,
            crosshair_vertices: crosshair.len() as u32,
            _depth_texture: depth_texture,
            depth_view,
        })
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

    pub fn draw(&mut self, player: &Player) -> Result<(), wgpu::SurfaceError> {
        let aspect = self.config.width as f32 / self.config.height as f32;
        let view_proj = Mat4::perspective_rh(70.0_f32.to_radians(), aspect, 0.05, 200.0)
            * Mat4::look_to_rh(player.eye_position(), player.forward(), Vec3::Y);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::bytes_of(&CameraUniform {
                view_proj: view_proj.to_cols_array_2d(),
            }),
        );
        let frame = self.surface.get_current_texture()?;
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
            pass.set_vertex_buffer(0, self.terrain_buffer.slice(..));
            pass.draw(0..self.terrain_vertices, 0..1);
            pass.set_pipeline(&self.crosshair_pipeline);
            pass.set_vertex_buffer(0, self.crosshair_buffer.slice(..));
            pass.draw(0..self.crosshair_vertices, 0..1);
        }
        self.queue.submit(std::iter::once(encoder.finish()));
        frame.present();
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
