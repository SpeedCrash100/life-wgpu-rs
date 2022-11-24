use glam::{Mat4, Vec2, Vec3, Vec4Swizzles};
use wgpu::{util::DeviceExt, BindGroup, BindGroupLayout, Buffer, Device, Queue};

use super::{BinableToRenderPass, HaveBindGroup};

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: Mat4 = Mat4::from_cols_array(
    &[
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
    ]
);

pub struct Camera {
    position: Vec2,
    scale: f32,
    update_required: bool,

    view: Mat4,
    ortho: Mat4,
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

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[Mat4::IDENTITY]),
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

        let mut camera = Self {
            position: Vec2::ZERO,
            scale: 1.0,
            update_required: false,

            view: Mat4::IDENTITY,
            ortho: Mat4::IDENTITY,
            speed: 5.0,

            buffer,
            bind_group_layout,
            bind_group,
        };

        camera.resize(width, height);
        camera.rebuild_view();

        camera
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.update_required = true;

        let w = width as f32 / 2.0;
        let h = height as f32 / 2.0;
        self.ortho = Mat4::orthographic_rh(-w, w, -h, h, 0.1, 100.0);
    }

    pub fn update(&mut self, queue: &Queue) -> bool {
        if self.update_required {
            self.update_required = false;
            let raw = Self::build_raw(&self.ortho, &self.view);
            queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[raw]));

            return true;
        }

        false
    }

    fn build_raw(ortho: &Mat4, view: &Mat4) -> Mat4 {
        OPENGL_TO_WGPU_MATRIX.mul_mat4(ortho).mul_mat4(view)
    }

    pub fn up(&mut self) {
        self.translate(Vec2::NEG_Y * self.speed)
    }

    pub fn down(&mut self) {
        self.translate(Vec2::Y * self.speed)
    }

    pub fn left(&mut self) {
        self.translate(Vec2::X * self.speed)
    }

    pub fn right(&mut self) {
        self.translate(Vec2::NEG_X * self.speed)
    }

    pub fn zoom_in(&mut self) {
        self.scale(1.1);
    }

    pub fn zoom_out(&mut self) {
        self.scale(0.9);
    }

    pub fn scale(&mut self, factor: f32) {
        self.update_required = true;

        self.scale *= factor;
        self.view = Mat4::from_scale(Vec3::splat(factor)).mul_mat4(&self.view);
    }

    pub fn translate(&mut self, translate: Vec2) {
        self.update_required = true;

        self.position = self.position.mul_add(Vec2::ONE, translate);
        self.view = self
            .view
            .mul_mat4(&Mat4::from_translation(translate.extend(0.0)))
    }

    pub fn set_position(&mut self, position: Vec2) {
        self.position = position;
        self.rebuild_view();
    }

    pub fn rebuild_view(&mut self) {
        self.update_required = true;

        let eye: Vec3 = self.position.extend(1.0);
        let center: Vec3 = self.position.extend(0.0);
        let up: Vec3 = Vec3::Y;

        self.view =
            Mat4::from_scale(Vec3::splat(self.scale)).mul_mat4(&Mat4::look_at_rh(eye, center, up));
    }

    pub fn from_clip_space_to_local(&self, clip_coord: Vec2) -> Vec2 {
        let clip4 = clip_coord.extend(0.0).extend(1.0);
        let raw = Self::build_raw(&self.ortho, &self.view);
        raw.inverse().mul_vec4(clip4).xy()
    }

    pub fn view_box(&self) -> (Vec2, Vec2) {
        let min = self.from_clip_space_to_local([-1.0, -1.0].into());
        let max = self.from_clip_space_to_local([1.0, 1.0].into());

        (min, max)
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
