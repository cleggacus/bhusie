use crate::renderer::texture;

pub struct SkyPipelineDescriptor<'a> {
    pub device: &'a wgpu::Device, 
    pub queue: &'a wgpu::Queue, 
    pub resolution: (u32, u32),
    pub prev_texture_view: &'a wgpu::TextureView
}

pub struct SkyPipeline {
    pipeline: wgpu::ComputePipeline,
    bind_group: wgpu::BindGroup,
    resolution: (u32, u32),
    texture_view: wgpu::TextureView,
}

impl SkyPipeline {
    pub fn new(descriptor: SkyPipelineDescriptor) -> Self {
        let shader = descriptor.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader_sky"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/sky.wgsl").into()),
        });

        let texture = descriptor.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("texture thingy"),
            size: wgpu::Extent3d {
                width: descriptor.resolution.0,
                height: descriptor.resolution.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::STORAGE_BINDING,
            view_formats: Default::default(),
        });

        let texture_view = texture.create_view(&Default::default());

        let sky_texture = texture::Texture::from_bytes(
            descriptor.device, descriptor.queue, include_bytes!("../textures/sky.jpg"));

        let bind_group_layout =
            descriptor.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("compute_bind_group_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::StorageTexture {
                            access: wgpu::StorageTextureAccess::WriteOnly,
                            format: wgpu::TextureFormat::Rgba8Unorm,
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        },
                        count: None,
                    },
                ],
            });

        let bind_group = descriptor.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sky_texture.sampler()),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(sky_texture.view()),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(descriptor.prev_texture_view),
                },
            ],
        });

        let pipeline_layout =
            descriptor.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[
                    &bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        let pipeline = descriptor.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: "main",
        });

        Self {
            pipeline,
            bind_group,
            resolution: descriptor.resolution,
            texture_view,
        }
    }

    pub fn output_view(&self) -> &wgpu::TextureView {
        &self.texture_view
    }

    pub fn pass(&mut self,encoder: &mut wgpu::CommandEncoder) {
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Compute Pass"),
            timestamp_writes: None,
        });

        compute_pass.set_pipeline(&self.pipeline);
        compute_pass.set_bind_group(0, &self.bind_group, &[]);
        compute_pass.dispatch_workgroups(
            (self.resolution.0 + 7) / 8,
            (self.resolution.1 + 7) / 8,
            1,
        );
    }
}
