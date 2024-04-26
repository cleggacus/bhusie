use wgpu::util::DeviceExt;

use crate::{renderer::{quad::Quad, vertex::Vertex}, ui::UI};

pub struct ScreenPipelineDescriptor<'a> {
    pub device: &'a wgpu::Device,
    pub input_view: &'a wgpu::TextureView,
    pub bloom_view: &'a wgpu::TextureView,
    pub format: wgpu::TextureFormat,
    pub resolution: (u32, u32)
}

pub struct ScreenPipeline {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    egui_renderer: egui_wgpu::Renderer,
    resolution: (u32, u32)
}

impl ScreenPipeline {
    pub fn new(descriptor: ScreenPipelineDescriptor) -> Self {
        let shader = descriptor.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Screen Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/screen.wgsl").into()),
        });

        let bind_group_layout =
            descriptor.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("screen bind group layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                ],
            });

        let pipeline_layout =
            descriptor.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

        let pipeline = descriptor.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Screen Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: descriptor.format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                ..Default::default()
            },
            multiview: None,
        });

        let sampler = descriptor.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });


        let bind_group = descriptor.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(
                        descriptor.input_view
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(
                        descriptor.bloom_view
                    ),
                },
            ],
        });

        let vertex_buffer = descriptor.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: &[],
            usage: wgpu::BufferUsages::VERTEX,
        });

        let egui_renderer = egui_wgpu::Renderer::new(descriptor.device, descriptor.format, None, 1);

        Self {
            pipeline,
            vertex_buffer,
            bind_group,
            egui_renderer,
            resolution: descriptor.resolution, 
        }
    }

    fn get_screen_quad(&self, surface_width: u32, surface_height: u32) -> Quad {
        let resolution = self.resolution;

        let texture_ratio = resolution.0 as f32 / resolution.1 as f32;
        let screen_ratio = surface_width as f32 / surface_height as f32;

        let (w, h) = if texture_ratio > screen_ratio {
            (surface_width as f32, surface_width as f32 / texture_ratio)
        } else {
            (surface_height as f32 * texture_ratio, surface_height as f32)
        };

        let x = (surface_width as f32 - w) / 2.0;
        let y = (surface_height as f32 - h) / 2.0;

        Quad::new(
            x / surface_width as f32,
            y / surface_height as f32,
            w / surface_width as f32,
            h / surface_height as f32,
        )
    }

    pub fn pass(&mut self, descriptor: ScreenPassDescriptor) {
        let egui_full_output = descriptor.ui.take_full_output();

        let egui_ctx = descriptor.ui.ctx();

        if let Some(egui_full_output) = &egui_full_output {
            for (id, image_delta) in &egui_full_output.textures_delta.set {
                self.egui_renderer.update_texture(descriptor.device, descriptor.queue, *id, image_delta)
            }
        }

        let egui_data = egui_full_output.map(|egui_full_output| (
            egui_ctx.tessellate(egui_full_output.shapes, egui_full_output.pixels_per_point),
            egui_wgpu::ScreenDescriptor { 
                size_in_pixels: [
                    descriptor.surface_config.width,
                    descriptor.surface_config.height
                ],
                pixels_per_point: egui_full_output.pixels_per_point
            }
        ));

        if let Some((egui_primitives, screen_descriptor)) = &egui_data {
            self.egui_renderer.update_buffers(
                descriptor.device,
                descriptor.queue,
                descriptor.encoder,
                egui_primitives,
                screen_descriptor,
            );
        }

        let mut render_pass = descriptor.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: descriptor.output_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        self.vertex_buffer = descriptor.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(self.get_screen_quad(
                descriptor.surface_config.width,
                descriptor.surface_config.height,
            ).get_vertices()),
            usage: wgpu::BufferUsages::VERTEX,
        });

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw(0..6, 0..1);

        if let Some((egui_primitives, screen_descriptor)) = &egui_data {
            self.egui_renderer.render(&mut render_pass, egui_primitives, screen_descriptor);
        }
    }
}


pub struct ScreenPassDescriptor<'a> {
    pub surface_config: &'a wgpu::SurfaceConfiguration,
    pub encoder: &'a mut wgpu::CommandEncoder,
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub output_view: &'a wgpu::TextureView,
    pub ui: &'a mut UI,
}
