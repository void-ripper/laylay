use wgpu::{BindGroup, Buffer, IndexFormat, PipelineCompilationOptions, SurfaceTargetUnsafe};
use winit::{dpi::PhysicalSize, window::Window};

use crate::scene::{
    camera::{Camera, RawCamera},
    drawable::Drawable,
    light::{Light, RawLight},
    material::Material,
    model::Vertex,
    ScenePtr,
};

pub struct RenderContext<'w> {
    pub surface: wgpu::Surface<'w>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub window: Window,
    render_pipeline: wgpu::RenderPipeline,
    camera: RawCamera,
    camera_buffer: Buffer,
    camera_bind_group: BindGroup,
    lights: [RawLight; 10],
    light_buffer: Buffer,
    light_bind_group: BindGroup,
}

impl<'w> RenderContext<'w> {
    pub async fn new(win: Window) -> Self {
        let size = win.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = unsafe {
            instance
                .create_surface_unsafe(SurfaceTargetUnsafe::from_window(&win).unwrap())
                .unwrap()
        };

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptionsBase {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::default(),
                    label: None,
                },
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .filter(|f| f.is_srgb())
            .next()
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            desired_maximum_frame_latency: 2,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let (camera, camera_buffer, camera_bind_group_layout, camera_bind_group) =
            Camera::setup(&device);

        let (lights, light_buffer, light_bind_group_layout, light_bind_group) =
            Light::setup(&device);

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout, &light_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            cache: None,
            vertex: wgpu::VertexState {
                compilation_options: PipelineCompilationOptions::default(),
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[
                    Vertex::desc(),
                    Drawable::instace_desc(),
                    Material::instace_desc(),
                ],
            },
            fragment: Some(wgpu::FragmentState {
                // 3.
                module: &shader,
                compilation_options: PipelineCompilationOptions::default(),
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    // 4.
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, // 2.
                // cull_mode: Some(wgpu::Face::Back),
                cull_mode: None,
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None, // 1.
            multisample: wgpu::MultisampleState {
                count: 1,                         // 2.
                mask: !0,                         // 3.
                alpha_to_coverage_enabled: false, // 4.
            },
            multiview: None, // 5.
        });

        Self {
            window: win,
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            camera_bind_group,
            camera_buffer,
            camera,
            lights,
            light_buffer,
            light_bind_group,
        }
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub async fn render(&mut self, scene: ScenePtr) -> Result<(), wgpu::SurfaceError> {
        {
            let mut cam = scene.camera.write().await;
            cam.update().await;
            self.camera.view_proj = cam.transform;
        }
        {
            let mut lights = scene.lights.write().await;
            for light in lights.iter_mut() {
                light.update().await;
            }
        }
        self.queue
            .write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[self.camera]));
        self.queue
            .write_buffer(&self.light_buffer, 0, bytemuck::cast_slice(&self.lights));

        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let drawables = scene.drawables.read().await;

        for drw in drawables.values() {
            let instances = drw.instances.lock().await;
            self.queue.write_buffer(
                &instances.instance_buffer,
                0,
                bytemuck::cast_slice(&instances.instance_matrices),
            );
            self.queue.write_buffer(
                &instances.instance_material_buffer,
                0,
                bytemuck::cast_slice(&instances.instance_materials),
            );
        }

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.05,
                            g: 0.05,
                            b: 0.05,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_bind_group(1, &self.light_bind_group, &[]);

            for drw in drawables.values() {
                let mut instances = drw.instances.lock().await;
                let inst_count = instances.nodes.len();

                if inst_count > 0 {
                    // TODO: this is very ugly
                    for (id, n) in instances.nodes.clone().values().enumerate() {
                        instances.instance_matrices[id] = *n.world_transform.read().await;
                        instances.instance_materials[id] = n.material.lock().await.unwrap();
                    }
                    render_pass.set_vertex_buffer(0, drw.vertex_buffer.slice(..));
                    render_pass.set_vertex_buffer(1, instances.instance_buffer.slice(..));
                    render_pass.set_vertex_buffer(2, instances.instance_material_buffer.slice(..));
                    render_pass.set_index_buffer(drw.index_buffer.slice(..), IndexFormat::Uint32);
                    render_pass.draw_indexed(0..drw.index_count, 0, 0..inst_count as u32);
                }
            }
        };

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
