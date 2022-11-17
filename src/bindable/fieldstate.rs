use wgpu::{
    util::DeviceExt, BindGroup, BindGroupLayout, Buffer, BufferUsages, CommandEncoder, Device,
};

use super::{BinableToRenderPass, BindableToComputePass, HaveBindGroup};

pub struct FieldState {
    buffer: Buffer,
    bind_group: BindGroup,
    bind_group_layout: BindGroupLayout,
}

impl FieldState {
    pub fn new(state: &Vec<u32>, device: &Device, read_only: bool) -> Self {
        use wgpu::util::BufferInitDescriptor;
        use wgpu::{
            BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType,
            ShaderStages,
        };

        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("life buffer"),
            contents: bytemuck::cast_slice(&state),
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
        });

        let visibility = if read_only {
            ShaderStages::COMPUTE | ShaderStages::VERTEX
        } else {
            ShaderStages::COMPUTE
        };

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Life's field bind group layout"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility,
                ty: BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Life's field bind group"),
            layout: &bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        Self {
            buffer,
            bind_group,
            bind_group_layout,
        }
    }

    pub fn copy_from(&self, other: &FieldState, encoder: &mut CommandEncoder) {
        assert!(self.buffer.size() == other.buffer.size());

        encoder.copy_buffer_to_buffer(&other.buffer, 0, &self.buffer, 0, self.buffer.size())
    }
}

impl HaveBindGroup for FieldState {
    fn get_bind_layout(&self) -> &BindGroupLayout {
        &self.bind_group_layout
    }

    fn get_bind(&self) -> &BindGroup {
        &self.bind_group
    }
}

impl BindableToComputePass for FieldState {}
impl BinableToRenderPass for FieldState {}

impl Drop for FieldState {
    fn drop(&mut self) {
        self.buffer.destroy()
    }
}
