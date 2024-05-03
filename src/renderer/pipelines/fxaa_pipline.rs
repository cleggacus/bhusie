use wgpu::TextureView;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EdgeThresholdMax {
    Low,
    Medium,
    High,
    Ultra,
    Extreme,
}

impl From<EdgeThresholdMax> for String {
    fn from(value: EdgeThresholdMax) -> Self {
        match value {
            EdgeThresholdMax::Low => "Low".into(),
            EdgeThresholdMax::Medium => "Medium".into(),
            EdgeThresholdMax::High => "High".into(),
            EdgeThresholdMax::Ultra => "Ultra".into(),
            EdgeThresholdMax::Extreme => "Extreme".into(),
        }
    }
}


impl From<EdgeThresholdMax> for f32 {
    fn from(value: EdgeThresholdMax) -> Self {
        match value {
            EdgeThresholdMax::Low => 0.250,
            EdgeThresholdMax::Medium => 0.166,
            EdgeThresholdMax::High => 0.125,
            EdgeThresholdMax::Ultra => 0.063,
            EdgeThresholdMax::Extreme => 0.031,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EdgeThresholdMin {
    Low,
    Medium,
    High,
    Ultra,
    Extreme,
}
impl From<EdgeThresholdMin> for String {
    fn from(value: EdgeThresholdMin) -> Self {
        match value {
            EdgeThresholdMin::Low => "Low".into(),
            EdgeThresholdMin::Medium => "Medium".into(),
            EdgeThresholdMin::High => "High".into(),
            EdgeThresholdMin::Ultra => "Ultra".into(),
            EdgeThresholdMin::Extreme => "Extreme".into(),
        }
    }
}

impl From<EdgeThresholdMin> for f32 {
    fn from(value: EdgeThresholdMin) -> Self {
        match value {
            EdgeThresholdMin::Low => 0.0833,
            EdgeThresholdMin::Medium => 0.0625,
            EdgeThresholdMin::High => 0.0312,
            EdgeThresholdMin::Ultra => 0.0156,
            EdgeThresholdMin::Extreme => 0.0078,
        }
    }
}

pub struct FXAADetails {
    pub edge_threshold_min: EdgeThresholdMin,
    pub edge_threshold_max: EdgeThresholdMax,
    pub iterations: i32,
    pub subpixel_quality: f32,
}

#[repr(C)]
#[derive(Default, Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct FXAADetailsUniform {
    pub edge_threshold_min: f32,
    pub edge_threshold_max: f32,
    pub iterations: i32,
    pub subpixel_quality: f32,
}

impl FXAADetailsUniform {
    pub fn update(&mut self, value: &FXAADetails) {
        self.edge_threshold_min = value.edge_threshold_min.into();
        self.edge_threshold_max = value.edge_threshold_max.into();
        self.iterations = value.iterations;
        self.subpixel_quality = value.subpixel_quality;
    }
}

pub struct FXAAPipelineDescriptor<'a> {
    pub device: &'a wgpu::Device, 
    pub resolution: (u32, u32),
    pub texture_view: &'a TextureView,
    pub fxaa_buffer: &'a wgpu::Buffer,
}

pub struct FXAAPipeline {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    texture_view_out: wgpu::TextureView,
    texture_out: wgpu::Texture,
}

impl FXAAPipeline {
    pub fn new(descriptor: FXAAPipelineDescriptor) -> Self {
        let texture = descriptor.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: descriptor.resolution.0,
                height: descriptor.resolution.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: Default::default(),
        });

        let texture_view_out = texture.create_view(&Default::default());

        let shader = descriptor.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(
                include_str!("../shaders/fxaa.wgsl").into(),
            ),
        });

        let bind_group_layout =
            descriptor.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
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
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: texture.format(),
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
            mag_filter: wgpu::FilterMode::Linear,
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
                    resource: wgpu::BindingResource::TextureView(descriptor.texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: descriptor.fxaa_buffer.as_entire_binding(),
                },
            ],
        });

        Self {
            pipeline,
            bind_group,
            texture_view_out,
            texture_out: texture
        }
    }

    pub fn output_view(&self) -> &wgpu::TextureView {
        &self.texture_view_out
    }

    pub fn output_texture(&self) -> &wgpu::Texture {
        &self.texture_out
    }

    pub fn pass(&mut self, encoder: &mut wgpu::CommandEncoder) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.texture_view_out,
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

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.draw(0..3, 0..1);
    }
}
