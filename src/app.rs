use std::sync::{Arc, Mutex};

use log::info;
use wgpu::{
    util::DeviceExt, BindGroup, Buffer, Device, Instance, PrimitiveState, Queue, RenderPipeline,
    Surface, SurfaceConfiguration,
};
use winit::{event::VirtualKeyCode, window::Window};

use crate::{
    camera::Camera,
    event_chain::{DrawHandlerSubscriber, KeyboardHandlerSubscriber},
    life::Life,
    model::{Model, Quad},
    shader::Shader,
};

pub struct App {
    surface: Surface,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,

    render_pipeline: RenderPipeline,

    camera: Camera,
    camera_buffer: Buffer,
    camera_bind_group: BindGroup,

    quad: Quad,

    instance_buffer: Buffer,

    life: Life,
    life_bind_group: BindGroup,
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

        let life = Life::new(1024, 1024, &device);

        let cells = life.generate_cell_info();

        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&cells),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        // Shader init
        let shader = Shader::new(&device, config.format);
        // Camera prepare
        let camera = Camera::new(size.width, size.height);
        let camera_raw = camera.build_raw();

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_raw]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let (camera_bind_group_layout, camera_bind_group) =
            shader.create_camera_bind_group(&device, &camera_buffer);

        let (life_bind_group_layout, life_bind_group) =
            shader.create_life_field_bind(&device, life.life_buffer());

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout, &life_bind_group_layout],
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

        Arc::new(Mutex::new(Self {
            surface,
            device,
            queue,
            config,

            render_pipeline,

            camera,
            camera_buffer,
            camera_bind_group,

            quad,

            instance_buffer,

            life,
            life_bind_group,
        }))
    }

    pub fn update(&mut self) {
        self.life.step(&self.queue, &self.device);

        let camera_raw = self.camera.build_raw();
        self.queue
            .write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[camera_raw]));
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
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_bind_group(1, &self.life_bind_group, &[]);
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));

            self.quad
                .draw(&mut render_pass, 0..self.life.cell_count() as _);
        }

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

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
