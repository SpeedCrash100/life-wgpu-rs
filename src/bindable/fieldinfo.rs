use bytemuck::{Pod, Zeroable};
use wgpu::{util::DeviceExt, BindGroup, BindGroupLayout, Buffer, Device};

use super::{BindableToComputePass, HaveBindGroup};

/// Hold size information about field
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct FieldInfoRaw {
    width: u32,
    height: u32,
}

pub struct FieldInfo {
    field_info: FieldInfoRaw,

    buffer: Buffer,
    bind_group: BindGroup,
    bind_group_layout: BindGroupLayout,
}

impl FieldInfo {
    pub fn new(width: u32, height: u32, device: &Device) -> Self {
        let field_info = FieldInfoRaw { width, height };
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Field info buffer"),
            contents: bytemuck::cast_slice(&[field_info]),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        use wgpu::{
            BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType,
            ShaderStages,
        };

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Life's field bind group layout"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
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
            field_info,

            buffer,
            bind_group,
            bind_group_layout,
        }
    }

    pub fn width(&self) -> u32 {
        self.field_info.width
    }

    pub fn height(&self) -> u32 {
        self.field_info.height
    }
}

impl HaveBindGroup for FieldInfo {
    fn get_bind_layout(&self) -> &BindGroupLayout {
        &self.bind_group_layout
    }

    fn get_bind(&self) -> &BindGroup {
        &self.bind_group
    }
}

impl BindableToComputePass for FieldInfo {}

impl Drop for FieldInfo {
    fn drop(&mut self) {
        self.buffer.destroy()
    }
}
