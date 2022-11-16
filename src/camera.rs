use bytemuck::{Pod, Zeroable};
use cgmath::{ortho, Matrix4, Point3, Vector3};

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
}

impl Camera {
    pub fn new(width: u32, height: u32) -> Self {
        debug_assert!(width > 0 && height > 0);

        let eye: Point3<f32> = (0.0, 0.0, 1.0).into();
        let center: Point3<f32> = (0.0, 0.0, 0.0).into();
        let up: Vector3<f32> = cgmath::Vector3::unit_y();

        let view = cgmath::Matrix4::look_at_rh(eye, center, up);
        let ortho = ortho(0.0, width as f32, 0.0, height as f32, 0.1, 100.0);

        Self {
            view,
            ortho,
            speed: 0.1,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.ortho = ortho(0.0, width as f32, 0.0, height as f32, 0.1, 100.0);
    }

    pub fn build_raw(&self) -> CameraRaw {
        let mat = OPENGL_TO_WGPU_MATRIX * self.ortho * self.view;
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

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct CameraRaw {
    view_proj: [[f32; 4]; 4],
}
