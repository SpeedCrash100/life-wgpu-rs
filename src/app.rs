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
    shader::{CellInfo, Shader, Vertex},
};

const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-0.5, 0.5, 0.0],
    },
    Vertex {
        position: [-0.5, -0.5, 0.0],
    },
    Vertex {
        position: [0.5, -0.5, 0.0],
    },
    Vertex {
        position: [0.5, 0.5, 0.0],
    },
];

const INDICES: &[u16] = &[0, 1, 2, 0, 2, 3];

pub struct App {
    surface: Surface,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,

    render_pipeline: RenderPipeline,

    camera: Camera,
    camera_buffer: Buffer,
    camera_bind_group: BindGroup,

    vertex_buffer: Buffer,
    indices_buffer: Buffer,
    num_indices: u32,

    instance_buffer: Buffer,
    cells: Vec<CellInfo>,

    life: Life,
}

impl App {
    pub async fn new(window: &Window) -> Arc<Mutex<Self>> {
        let instance = Instance::new(wgpu::Backends::all());
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
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
        };
        surface.configure(&device, &config);

        // Init instances

        let life = Life::new(256, 256);

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

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout],
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

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let num_indices = INDICES.len() as u32;

        let indices_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        Arc::new(Mutex::new(Self {
            surface,
            device,
            queue,
            config,

            render_pipeline,

            camera,
            camera_buffer,
            camera_bind_group,

            vertex_buffer,
            indices_buffer,
            num_indices,

            instance_buffer,
            cells,

            life,
        }))
    }

    pub fn update(&mut self) {
        self.life.step();
        self.cells = self.life.generate_cell_info();
        self.queue
            .write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(&self.cells));

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
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            render_pass.set_index_buffer(self.indices_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..self.cells.len() as _);
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
