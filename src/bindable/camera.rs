use bytemuck::{Pod, Zeroable};
use cgmath::{ortho, Matrix4, Point3, Vector3};
use wgpu::{util::DeviceExt, BindGroup, BindGroupLayout, Buffer, Device, Queue};

use super::{BinableToRenderPass, HaveBindGroup};

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

pub struct Camera {
    view: Matrix4<f32>,
    ortho: Matrix4<f32>,
    speed: f32,

    buffer: Buffer,
    bind_group: BindGroup,
    bind_group_layout: BindGroupLayout,
}

impl Camera {
    pub fn new(width: u32, height: u32, device: &Device) -> Self {
        use wgpu::{
            BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType,
            ShaderStages,
        };

        debug_assert!(width > 0 && height > 0);

        let eye: Point3<f32> = (0.0, 0.0, 1.0).into();
        let center: Point3<f32> = (0.0, 0.0, 0.0).into();
        let up: Vector3<f32> = cgmath::Vector3::unit_y();

        let mut view = cgmath::Matrix4::look_at_rh(eye, center, up);
        view = view * cgmath::Matrix4::from_scale(1.0);
        let ortho = ortho(0.0, width as f32, 0.0, height as f32, 0.1, 100.0);

        let camera_raw = Self::build_raw(&ortho, &view);
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_raw]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Camera bind group layout"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX,
                ty: BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera bind group"),
            layout: &bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        Self {
            view,
            ortho,
            speed: 5.0,

            buffer,
            bind_group_layout,
            bind_group,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.ortho = ortho(0.0, width as f32, 0.0, height as f32, 0.1, 100.0);
    }

    pub fn update(&self, queue: &Queue) {
        let raw = Self::build_raw(&self.ortho, &self.view);
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[raw]));
    }

    fn build_raw(ortho: &Matrix4<f32>, view: &Matrix4<f32>) -> CameraRaw {
        let mat = OPENGL_TO_WGPU_MATRIX * ortho * view;
        CameraRaw {
            view_proj: mat.into(),
        }
    }

    pub fn up(&mut self) {
        self.view = self.view
            * cgmath::Matrix4::from_translation(Vector3 {
                x: 0.0,
                y: -self.speed,
                z: 0.0,
            })
    }

    pub fn down(&mut self) {
        self.view = self.view
            * cgmath::Matrix4::from_translation(Vector3 {
                x: 0.0,
                y: self.speed,
                z: 0.0,
            })
    }

    pub fn left(&mut self) {
        self.view = self.view
            * cgmath::Matrix4::from_translation(Vector3 {
                x: self.speed,
                y: 0.0,
                z: 0.0,
            })
    }

    pub fn right(&mut self) {
        self.view = self.view
            * cgmath::Matrix4::from_translation(Vector3 {
                x: -self.speed,
                y: 0.0,
                z: 0.0,
            })
    }

    pub fn zoom_in(&mut self) {
        self.view = self.view * cgmath::Matrix4::from_scale(1.1);
    }

    pub fn zoom_out(&mut self) {
        self.view = self.view * cgmath::Matrix4::from_scale(0.9);
    }
}

impl HaveBindGroup for Camera {
    fn get_bind_layout(&self) -> &BindGroupLayout {
        &self.bind_group_layout
    }

    fn get_bind(&self) -> &BindGroup {
        &self.bind_group
    }
}

impl BinableToRenderPass for Camera {}

impl Drop for Camera {
    fn drop(&mut self) {
        self.buffer.destroy()
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct CameraRaw {
    view_proj: [[f32; 4]; 4],
}
