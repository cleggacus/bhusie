pub mod vertex;
pub mod quad;
pub mod pipelines;
pub mod array_buffer;
pub mod model;
pub mod material;
pub mod texture;
pub mod triangle;

use wgpu::{util::DeviceExt, PresentMode};
use winit::window::Window;

use crate::{renderer::pipelines::{bloom_pipline::{BloomDirection, BloomDownPipelineDescriptor}, fxaa_pipline::{EdgeThresholdMax, EdgeThresholdMin, FXAAPipelineDescriptor}, hdr_pipeline::HDRPipelineDescriptor, mix_pipeline::{MixDetails, MixPipelineDescriptor}, sky_pipeline::SkyPipelineDescriptor}, scene::{blackhole::BlackHoleUniform, camera::CameraUniform, Scene}, ui::UI};

use self::pipelines::{bloom_pipline::BloomPipeline, fxaa_pipline::{FXAADetails, FXAADetailsUniform, FXAAPipeline}, hdr_pipeline::HDRPipeline, mix_pipeline::MixPipeline, ray_pipeline::{RayDetails, RayPipeline, RayPipelineDescriptor}, screen_pipeline::{ScreenPassDescriptor, ScreenPipeline, ScreenPipelineDescriptor}, sky_pipeline::SkyPipeline};

pub struct Renderer<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface_config: wgpu::SurfaceConfiguration,
    bloom_pipelines: Vec<BloomPipeline>,
    sky_pipeline: SkyPipeline,
    hdr_pipeline: HDRPipeline,
    screen_pipeline: ScreenPipeline,

    pub present_mode: PresentMode,

    pub fxaa_details: FXAADetails,
    pub fxaa_details_uniform: FXAADetailsUniform,
    fxaa_details_buffer: wgpu::Buffer,
    fxaa_pipeline: FXAAPipeline,

    pub mix_details: MixDetails,
    mix_details_buffer: wgpu::Buffer,
    mix_pipeline: MixPipeline,

    pub ray_details: RayDetails,
    ray_details_buffer: wgpu::Buffer,
    ray_pipelines: Vec<RayPipeline>,

    black_hole_uniform: BlackHoleUniform,
    black_hole_buffer: wgpu::Buffer,

    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,

    material_buffer: wgpu::Buffer,
    model_buffer: wgpu::Buffer,
}

impl<'a> Renderer<'a> {
    pub async fn new(window: &'a Window, scene: &Scene) -> Self {
        let mut size = window.inner_size();
        size.width = size.width.max(1);
        size.height = size.height.max(1);

        let instance = wgpu::Instance::default();

        let surface = instance.create_surface(window)
            .expect("Failed to create surface");

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .expect("Failed to find an appropriate adapter");

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits {
                    ..wgpu::Limits::downlevel_defaults()
                        .using_resolution(adapter.limits())
                }
            },
            None,
        )
        .await
        .expect("Failed to create device");

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &surface_config);

        let material_buffer = scene.materials.create_buffer(&device);
        let model_buffer = scene.models.create_buffer(&device);

        let ray_details = RayDetails {
            angle_division_threshold: 0.02,
            step_size: 0.1,
            max_iterations: 2000,
            ..RayDetails::default()
        };

        let ray_details_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Ray Details Buffer"),
                contents: bytemuck::cast_slice(&[ray_details]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let camera_uniform = CameraUniform::new();

        let camera_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[camera_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let black_hole_uniform = BlackHoleUniform::new();

        let black_hole_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Black Hole Buffer"),
                contents: bytemuck::cast_slice(&[black_hole_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let base_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
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

        let base_texture_view = base_texture.create_view(&Default::default());

        let mut ray_pipelines: Vec<RayPipeline> = Vec::new();

        // let mut current_res = (9.0, 5.0); // 720p 64
        // let mut current_res = (19.0, 10.0); // 720p 45
        // let mut current_res = (39.0, 22.0); // 720p 45
        // let mut current_res = (79.0, 44.0); // 720p 45
        // let mut current_res = (159.0, 89.0); // 720p 45
        let mut current_res = (8.0, 4.0); // 1080p
        let ray_multiplier = 4.0;
        let iters = 5;

        for i in 0..iters {
            log::info!("Loading ray pipeline ({}): {}, {}", i, current_res.0 as u32, current_res.1 as u32);

            let prev_texture_view = match ray_pipelines.last() {
                Some(ray_pipeline) => ray_pipeline.output_view(),
                None => &base_texture_view
            };

            let ray_pipeline = RayPipeline::new(RayPipelineDescriptor {
                device: &device, 
                queue: &queue,
                resolution: (current_res.0 as u32, current_res.1 as u32),
                camera_buffer: &camera_buffer,
                black_hole_buffer: &black_hole_buffer,
                material_buffer: &material_buffer,
                model_buffer: &model_buffer,
                ray_details_buffer: &ray_details_buffer,
                prev_texture_view,
            });

            ray_pipelines.push(ray_pipeline);

            if i < iters-1 {
                current_res.0 = current_res.0 * ray_multiplier - (ray_multiplier - 1.0);
                current_res.1 = current_res.1 * ray_multiplier - (ray_multiplier - 1.0);
            }
        }

        log::info!("Loading sky pipeline");

        let sky_pipeline = SkyPipeline::new(SkyPipelineDescriptor {
            device: &device,
            queue: &queue,
            resolution: (current_res.0 as u32, current_res.1 as u32),
            prev_texture_view: ray_pipelines.last().unwrap().output_view(),
        });


        let bloom_pipeline_count = 6;
        let bloom_multiplier = 2.0;
        let mut bloom_pipelines: Vec<BloomPipeline> = Vec::new();

        for i in 0..bloom_pipeline_count*2 {
            let is_down = i < bloom_pipeline_count;

            current_res = if is_down { 
                (current_res.0 / bloom_multiplier, current_res.1 / bloom_multiplier)
            } else {
                (current_res.0 * bloom_multiplier, current_res.1 * bloom_multiplier)
            };

            log::info!("Loading bloom pipeline ({} {}): {}, {}", 
                if i < bloom_pipeline_count {
                    "Down"
                } else {
                    "Up"
                }, i%bloom_pipeline_count, current_res.0 as u32, current_res.1 as u32);

            let prev_texture_view = match bloom_pipelines.last() {
                Some(bloom_pipeline) => bloom_pipeline.output_view(),
                None => sky_pipeline.output_view(),
            };

            bloom_pipelines.push(
                BloomPipeline::new(BloomDownPipelineDescriptor {
                    device: &device,
                    resolution: (current_res.0 as u32, current_res.1 as u32),
                    texture_view: prev_texture_view,
                    direction: if is_down {
                        BloomDirection::Down
                    } else {
                        BloomDirection::Up
                    },
                })
            )
        }

        let mix_details = MixDetails {
            mix_ratio: 0.7
        };

        let mix_details_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Mix Details Buffer"),
                contents: bytemuck::cast_slice(&[mix_details]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );
        
        log::info!("Loading mix pipeline");

        let mix_pipeline = MixPipeline::new(MixPipelineDescriptor {
            device: &device,
            resolution: (current_res.0 as u32, current_res.1 as u32),
            texture_view_1: sky_pipeline.output_view(),
            texture_view_2: bloom_pipelines.last().unwrap().output_view(),
            mix_buffer: &mix_details_buffer,
        });

        log::info!("Loading hdr pipeline");

        let hdr_pipeline = HDRPipeline::new(HDRPipelineDescriptor {
            device: &device,
            resolution: (current_res.0 as u32, current_res.1 as u32),
            texture_view: mix_pipeline.output_view(),
        });

        log::info!("Loading fxaa pipeline");

        let fxaa_details = FXAADetails {
            edge_threshold_min: EdgeThresholdMin::Ultra,
            edge_threshold_max: EdgeThresholdMax::Ultra,
            iterations: 12,
            subpixel_quality: 0.75,
        };

        let fxaa_details_uniform = FXAADetailsUniform::default();

        let fxaa_details_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("FXAA Details Buffer"),
                contents: bytemuck::cast_slice(&[fxaa_details_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let fxaa_pipeline = FXAAPipeline::new(FXAAPipelineDescriptor {
            device: &device,
            resolution: (current_res.0 as u32, current_res.1 as u32),
            texture_view: hdr_pipeline.output_view(),
            fxaa_buffer: &fxaa_details_buffer 
        });

        log::info!("Loading screen pipeline");

        let screen_pipeline = ScreenPipeline::new(ScreenPipelineDescriptor { 
            device: &device, 
            input_view: fxaa_pipeline.output_view(),
            format: surface_format,
            resolution: (current_res.0 as u32, current_res.1 as u32),
        });

        let present_mode = surface_config.present_mode;


        Self {
            surface,
            device,
            queue,
            surface_config,
            screen_pipeline,
            sky_pipeline,
            hdr_pipeline,
            bloom_pipelines,
            present_mode,

            ray_pipelines,
            ray_details,
            ray_details_buffer,

            fxaa_pipeline,
            fxaa_details,
            fxaa_details_uniform,
            fxaa_details_buffer,

            mix_pipeline,
            mix_details,
            mix_details_buffer,

            camera_uniform,
            camera_buffer,

            black_hole_uniform,
            black_hole_buffer,

            material_buffer,
            model_buffer,
        }
    }

    pub fn update_present_mode(&mut self) {
        self.surface_config.present_mode = self.present_mode;
        self.surface.configure(&self.device, &self.surface_config);
    }

    pub fn render(&mut self, ui: &mut UI, scene: &mut Scene, dt: f32) -> Result<(), wgpu::SurfaceError> {
        if self.present_mode != self.surface_config.present_mode {
            self.update_present_mode()
        }

        self.camera_uniform.update(&scene.camera);
        self.black_hole_uniform.update(&scene.black_hole);

        self.ray_details.time += dt;
        self.ray_details.material_count = scene.materials.size() as i32;
        self.ray_details.model_count = scene.models.size() as i32;

        self.fxaa_details_uniform.update(&self.fxaa_details);

        self.queue.write_buffer(&self.black_hole_buffer, 0, bytemuck::cast_slice(&[self.black_hole_uniform]));
        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[self.camera_uniform]));
        self.queue.write_buffer(&self.ray_details_buffer, 0, bytemuck::cast_slice(&[self.ray_details]));
        self.queue.write_buffer(&self.mix_details_buffer, 0, bytemuck::cast_slice(&[self.mix_details]));
        self.queue.write_buffer(&self.fxaa_details_buffer, 0, bytemuck::cast_slice(&[self.fxaa_details_uniform]));

        scene.models.update_buffer(&self.queue, &self.model_buffer);
        scene.materials.update_buffer(&self.queue, &self.material_buffer);

        let output = self.surface.get_current_texture()?;

        let output_view = output.texture.create_view(&wgpu::TextureViewDescriptor { 
            ..wgpu::TextureViewDescriptor::default()
        });

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });


        // compute passes 
        
        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Compute Pass"),
                timestamp_writes: None,
            });

            for rp in &mut self.ray_pipelines {
                rp.pass(&mut compute_pass);
            } 

            self.sky_pipeline.pass(&mut compute_pass);
        }

        // render passes 

        for bp in &mut self.bloom_pipelines {
            bp.pass(&mut encoder);
        } 

        self.mix_pipeline.pass(&mut encoder);
        self.hdr_pipeline.pass(&mut encoder);
        self.fxaa_pipeline.pass(&mut encoder);

        self.screen_pipeline.pass(ScreenPassDescriptor {
            surface_config: &self.surface_config,
            encoder: &mut encoder,
            device: &self.device,
            queue: &self.queue,
            output_view: &output_view,
            ui,
        });

        // cleanup and present
        
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.surface_config.width = width;
            self.surface_config.height = height;
            self.surface.configure(&self.device, &self.surface_config);
        }
    }
}

