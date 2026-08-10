use std::error::Error;
use std::mem;

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;
use winit::{dpi::PhysicalSize, window::Window};

use crate::camera::Camera;
use crate::gpu_buffers::{BirdBuffers, BirdInstanceRaw};
use sim_core::{Bird, FlockParams};

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct BirdVertex {
    position: [f32; 2],
}

struct DepthResources {
    view: wgpu::TextureView,
}

pub struct Renderer<'window> {
    surface: wgpu::Surface<'window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: PhysicalSize<u32>,
    depth: DepthResources,
    sky_pipeline: wgpu::RenderPipeline,
    render_pipeline: wgpu::RenderPipeline,
    bird_vertex_buffer: wgpu::Buffer,
    bird_vertex_count: u32,
    bird_buffers: BirdBuffers,
    camera: Camera,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    frame_index: u64,
}

impl<'window> Renderer<'window> {
    pub async fn new(
        window: &'window Window,
        bird_capacity: usize,
    ) -> Result<Self, Box<dyn Error>> {
        let size = window.inner_size();
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(window)?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await?;

        #[cfg(target_arch = "wasm32")]
        let required_limits =
            wgpu::Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits());
        #[cfg(not(target_arch = "wasm32"))]
        let required_limits = wgpu::Limits::default();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("murmuration-device"),
                required_features: wgpu::Features::empty(),
                required_limits,
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
            })
            .await?;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(wgpu::TextureFormat::is_srgb)
            .unwrap_or(surface_caps.formats[0]);
        let present_mode = surface_caps
            .present_modes
            .iter()
            .copied()
            .find(|mode| *mode == wgpu::PresentMode::Mailbox)
            .unwrap_or(wgpu::PresentMode::Fifo);
        let alpha_mode = surface_caps.alpha_modes[0];

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode,
            alpha_mode,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);
        let depth = create_depth_resources(&device, &config);

        let camera = Camera::default();
        let bootstrap_params = FlockParams::default();
        let camera_uniform = camera.build_uniform(
            config.width as f32 / config.height as f32,
            0.0,
            &bootstrap_params,
        );
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("camera-buffer"),
            contents: bytemuck::bytes_of(&camera_uniform),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("camera-bind-group-layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("camera-bind-group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("render-birds-shader"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("../../../assets/shaders/render_birds.wgsl").into(),
            ),
        });
        let sky_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("sky-shader"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("../../../assets/shaders/sky.wgsl").into(),
            ),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("render-pipeline-layout"),
                bind_group_layouts: &[&camera_bind_group_layout],
                push_constant_ranges: &[],
            });
        let sky_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("sky-pipeline-layout"),
            bind_group_layouts: &[&camera_bind_group_layout],
            push_constant_ranges: &[],
        });
        let sky_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("sky-render-pipeline"),
            layout: Some(&sky_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &sky_shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &sky_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Always,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("bird-render-pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: mem::size_of::<BirdVertex>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: wgpu::VertexFormat::Float32x2,
                        }],
                    },
                    wgpu::VertexBufferLayout {
                        array_stride: mem::size_of::<BirdInstanceRaw>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &[
                            wgpu::VertexAttribute {
                                offset: 0,
                                shader_location: 1,
                                format: wgpu::VertexFormat::Float32x4,
                            },
                            wgpu::VertexAttribute {
                                offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                                shader_location: 2,
                                format: wgpu::VertexFormat::Float32x4,
                            },
                            wgpu::VertexAttribute {
                                offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                                shader_location: 3,
                                format: wgpu::VertexFormat::Float32x4,
                            },
                        ],
                    },
                ],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Always,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let bird_vertices = [
            BirdVertex {
                position: [-1.0, -1.0],
            },
            BirdVertex {
                position: [1.0, -1.0],
            },
            BirdVertex {
                position: [1.0, 1.0],
            },
            BirdVertex {
                position: [-1.0, -1.0],
            },
            BirdVertex {
                position: [1.0, 1.0],
            },
            BirdVertex {
                position: [-1.0, 1.0],
            },
        ];
        let bird_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("bird-vertex-buffer"),
            contents: bytemuck::cast_slice(&bird_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let empty_instances = vec![
            BirdInstanceRaw {
                position: [0.0; 4],
                velocity: [0.0; 4],
                aux: [0.0; 4],
            };
            bird_capacity.max(1)
        ];
        let bird_instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("bird-instance-buffer"),
            contents: bytemuck::cast_slice(&empty_instances),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        Ok(Self {
            surface,
            device,
            queue,
            config,
            size,
            depth,
            sky_pipeline,
            render_pipeline,
            bird_vertex_buffer,
            bird_vertex_count: bird_vertices.len() as u32,
            bird_buffers: BirdBuffers::new(bird_instance_buffer, bird_capacity.max(1)),
            camera,
            camera_buffer,
            camera_bind_group,
            frame_index: 0,
        })
    }

    pub fn frame_index(&self) -> u64 {
        self.frame_index
    }

    pub fn size(&self) -> PhysicalSize<u32> {
        self.size
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width == 0 || new_size.height == 0 {
            self.size = new_size;
            return;
        }

        self.size = new_size;
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.surface.configure(&self.device, &self.config);
        self.depth = create_depth_resources(&self.device, &self.config);
    }

    pub fn draw(
        &mut self,
        birds: &[Bird],
        _params: &FlockParams,
        time_seconds: f32,
    ) -> Result<(), wgpu::SurfaceError> {
        if birds.len() > self.bird_buffers.bird_capacity {
            self.grow_instance_buffer(birds.len());
        }

        let camera_uniform = self.camera.build_uniform(
            self.config.width as f32 / self.config.height as f32,
            time_seconds,
            _params,
        );
        self.queue
            .write_buffer(&self.camera_buffer, 0, bytemuck::bytes_of(&camera_uniform));

        let instance_data: Vec<_> = birds.iter().map(BirdInstanceRaw::from_bird).collect();
        self.queue.write_buffer(
            &self.bird_buffers.buffer,
            0,
            bytemuck::cast_slice(&instance_data),
        );

        let frame = self.surface.get_current_texture()?;
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("bird-render-encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("bird-render-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.sky_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.draw(0..3, 0..1);

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.bird_vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.bird_buffers.buffer.slice(..));
            render_pass.draw(0..self.bird_vertex_count, 0..birds.len() as u32);
        }

        self.queue.submit(Some(encoder.finish()));
        frame.present();
        self.frame_index = self.frame_index.saturating_add(1);
        Ok(())
    }

    fn grow_instance_buffer(&mut self, bird_count: usize) {
        let new_capacity = bird_count.next_power_of_two().max(1);
        let empty_instances = vec![
            BirdInstanceRaw {
                position: [0.0; 4],
                velocity: [0.0; 4],
                aux: [0.0; 4],
            };
            new_capacity
        ];
        let buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("bird-instance-buffer"),
                contents: bytemuck::cast_slice(&empty_instances),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            });
        self.bird_buffers = BirdBuffers::new(buffer, new_capacity);
    }
}

fn create_depth_resources(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
) -> DepthResources {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("depth-texture"),
        size: wgpu::Extent3d {
            width: config.width.max(1),
            height: config.height.max(1),
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });

    DepthResources {
        view: texture.create_view(&wgpu::TextureViewDescriptor::default()),
    }
}
