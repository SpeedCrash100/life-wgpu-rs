use std::{
    sync::{Arc, Mutex},
    time::Instant,
};

use glam::Vec2;
use log::info;
use wgpu::{
    Device, Instance, PrimitiveState, Queue, RenderPipeline, Surface, SurfaceConfiguration,
};
use winit::{event::VirtualKeyCode, window::Window};

use crate::{
    bindable::{
        BinableToRenderPass, BindableToVertexBuffers, Camera, CellPosInstances, FieldState,
        HaveBindGroup,
    },
    event_chain::{DrawHandlerSubscriber, KeyboardHandlerSubscriber},
    life::Life,
    model::{Model, Quad},
    shader::Shader,
    text::FpsText,
};

pub struct App {
    surface: Surface,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,

    render_pipeline: RenderPipeline,

    camera: Camera,

    quad: Quad,

    instance_buffer: CellPosInstances,

    life: Life,
    life_buffer: Arc<FieldState>,

    fps: f32,
    previous_frame_time: Instant,
    fps_text: FpsText,
}

impl App {
    pub async fn new(window: &Window) -> Arc<Mutex<Self>> {
        let backends = match std::env::var("WGPU_DRIVER") {
            Ok(s) => {
                if s == "vulkan" {
                    wgpu::Backends::VULKAN
                } else if s == "dx12" {
                    wgpu::Backends::DX12
                } else if s == "dx11" {
                    wgpu::Backends::DX11
                } else if s == "gl" {
                    wgpu::Backends::GL
                } else {
                    wgpu::Backends::all()
                }
            }
            Err(_) => wgpu::Backends::all(),
        };

        let instance = Instance::new(backends);
        let surface = unsafe { instance.create_surface(window) };

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .expect("Cannot find adapter");

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .expect("Cannot create device");

        let size = window.inner_size();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoNoVsync,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
        };
        surface.configure(&device, &config);

        // Init instances

        let life_w = 1024;
        let life_h = 1024;
        let life = Life::new(life_w, life_h, &device);
        let life_buffer = life.life_buffer();

        let instance_buffer =
            life.generate_cell_info((Vec2::ZERO, [life_w as f32, life_h as f32].into()), &device);

        // Shader init
        let shader = Shader::new(&device, config.format);
        // Camera prepare
        let mut camera = Camera::new(size.width, size.height, &device);
        camera.set_position(Vec2::from_array([life_w as f32 / 2.0, life_h as f32 / 2.0]));

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &camera.get_bind_layout(),
                    &life.life_buffer().get_bind_layout(),
                ],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: shader.vertex_state(),
            fragment: Some(shader.frag_state()),
            primitive: PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: Default::default(),
            multiview: None,
        });

        let quad = Quad::new(&device);

        let fps = 0.0;
        let previous_frame_time = Instant::now();
        let fps_text = FpsText::new(&device, config.format);

        Arc::new(Mutex::new(Self {
            surface,
            device,
            queue,
            config,

            render_pipeline,

            camera,

            quad,

            instance_buffer,

            life,
            life_buffer,

            fps,
            previous_frame_time,
            fps_text,
        }))
    }

    pub fn update(&mut self) {
        let now = Instant::now();
        let frame_time = now - self.previous_frame_time;
        self.fps = 1.0 / (frame_time.as_secs_f32());
        self.previous_frame_time = now;

        self.life.step(&self.queue, &self.device);
        if self.camera.update(&self.queue) {
            // camera updated rebuild view box
            let view_box = self.camera.view_box();
            self.instance_buffer = self.life.generate_cell_info(view_box, &self.device);
        }
    }
}

impl DrawHandlerSubscriber for App {
    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.height > 0 && new_size.width > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.reconfigure();
        }
    }

    fn draw(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.update();

        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.3,
                            g: 0.3,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            self.camera.bind_to_render_pass(&mut render_pass, 0, &[]);
            self.life_buffer
                .bind_to_render_pass(&mut render_pass, 1, &[]);

            self.instance_buffer
                .bind_vertex_to_render_pass(&mut render_pass, 1);

            self.quad
                .draw(&mut render_pass, 0..self.instance_buffer.len() as _);
        }

        // draw fps
        self.fps_text
            .draw(self.fps, &self.device, &mut encoder, &view);

        // submit will accept anything that implements IntoIter
        self.fps_text.submit();
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        self.fps_text.recall();

        Ok(())
    }

    fn reconfigure(&mut self) {
        self.surface.configure(&self.device, &self.config);
        self.camera.resize(self.config.width, self.config.height);
    }
}

impl KeyboardHandlerSubscriber for App {
    fn key_pressed(&mut self, key_code: &winit::event::VirtualKeyCode) {
        info!("Pressed: {:?}", key_code);
        match key_code {
            VirtualKeyCode::W => self.camera.up(),
            VirtualKeyCode::S => self.camera.down(),
            VirtualKeyCode::A => self.camera.left(),
            VirtualKeyCode::D => self.camera.right(),
            VirtualKeyCode::Up => self.camera.zoom_in(),
            VirtualKeyCode::Down => self.camera.zoom_out(),
            _ => {}
        }
    }
}
