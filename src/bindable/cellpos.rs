use bytemuck::{Pod, Zeroable};
use wgpu::{util::DeviceExt, Buffer, Device};

use super::{BindableToVertexBuffers, HaveBuffer};

/// Holds cell's position of field and array
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct CellPos {
    pub pos: [f32; 2],
    pub idx: u32,
}

impl CellPos {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![1 => Float32x2, 2 => Uint32];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}

pub struct CellPosInstances {
    buffer: Buffer,
}

impl CellPosInstances {
    pub fn new(positions: Vec<CellPos>, device: &Device) -> Self {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Cell positions instance buffer"),
            contents: bytemuck::cast_slice(&positions),
            usage: wgpu::BufferUsages::VERTEX,
        });

        Self { buffer }
    }
}

impl HaveBuffer for CellPosInstances {
    fn get_buffer(&self) -> &Buffer {
        &self.buffer
    }
}

impl BindableToVertexBuffers for CellPosInstances {}

impl Drop for CellPosInstances {
    fn drop(&mut self) {
        self.buffer.destroy()
    }
}
