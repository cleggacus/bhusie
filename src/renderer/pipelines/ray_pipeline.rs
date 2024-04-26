use crate::renderer::texture;

#[repr(C)]
#[derive(Default, Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RayDetails {
    pub material_count: i32,
    pub model_count: i32,
    pub time: f32,
    pub integration_method: i32,
    pub step_size: f32,
    pub max_iterations: i32,
    pub show_disk_texture: i32,
    pub show_red_shift: i32,
}

pub struct RayPipelineDescriptor<'a> {
    pub device: &'a wgpu::Device, 
    pub queue: &'a wgpu::Queue, 
    pub resolution: (u32, u32),
    pub camera_buffer: &'a wgpu::Buffer,
    pub black_hole_buffer: &'a wgpu::Buffer,
    pub material_buffer: &'a wgpu::Buffer,
    pub model_buffer: &'a wgpu::Buffer,
    pub ray_details_buffer: &'a wgpu::Buffer,
    pub prev_texture_view: &'a wgpu::TextureView,
}

pub struct RayPipeline {
    pipeline: wgpu::ComputePipeline,
    bind_group: wgpu::BindGroup,
    texture_view: wgpu::TextureView,
    resolution: (u32, u32),
}

impl RayPipeline {
    pub fn new(
        descriptor: RayPipelineDescriptor
    ) -> Self {
        let shader = descriptor.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader_ray"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/ray.wgsl").into()),
        });

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
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::STORAGE_BINDING,
            view_formats: Default::default(),
        });

        let texture_view = texture.create_view(&Default::default());

        let temp_texture = texture::Texture::from_bytes(
            descriptor.device, descriptor.queue, include_bytes!("../textures/colourtemp.jpg"));

        let disk_texture = texture::Texture::from_bytes(
            descriptor.device, descriptor.queue, include_bytes!("../textures/disk_spiral.png"));

        let bind_group_layout =
            descriptor.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("ray bind group layout"),
                entries: &[
                    // output texture
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::StorageTexture {
                            access: wgpu::StorageTextureAccess::WriteOnly,
                            format: texture.format(),
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    // camera
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // details
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // materials
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage {
                                read_only: true,
                            },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // models
                    wgpu::BindGroupLayoutEntry {
                        binding: 4,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage {
                                read_only: true,
                            },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // black hole
                    wgpu::BindGroupLayoutEntry {
                        binding: 5,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // temp sampler
                    wgpu::BindGroupLayoutEntry {
                        binding: 6,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    // temp texture
                    wgpu::BindGroupLayoutEntry {
                        binding: 7,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    // disk sampler
                    wgpu::BindGroupLayoutEntry {
                        binding: 8,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    // disk texture
                    wgpu::BindGroupLayoutEntry {
                        binding: 9,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    // previous iteration
                    wgpu::BindGroupLayoutEntry {
                        binding: 10,
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
                    resource: descriptor.camera_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: descriptor.ray_details_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: descriptor.material_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: descriptor.model_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: descriptor.black_hole_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: wgpu::BindingResource::Sampler(temp_texture.sampler()),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: wgpu::BindingResource::TextureView(temp_texture.view()),
                },
                wgpu::BindGroupEntry {
                    binding: 8,
                    resource: wgpu::BindingResource::Sampler(disk_texture.sampler()),
                },
                wgpu::BindGroupEntry {
                    binding: 9,
                    resource: wgpu::BindingResource::TextureView(disk_texture.view()),
                },
                wgpu::BindGroupEntry {
                    binding: 10,
                    resource: wgpu::BindingResource::TextureView(descriptor.prev_texture_view),
                },
            ],
        });

        let pipeline_layout =
            descriptor.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Compute Pipeline Layout"),
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
            texture_view,
            resolution: descriptor.resolution,
        }
    }

    pub fn output_view(&self) -> &wgpu::TextureView {
        &self.texture_view
    }

    pub fn pass(&mut self, encoder: &mut wgpu::CommandEncoder) {
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
