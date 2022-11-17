use wgpu::{util::DeviceExt, Buffer, BufferUsages, Device};

use super::{Model, Vertex};

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

pub struct Quad {
    vertex_buffer: Buffer,
    indices_buffer: Buffer,
    num_indices: u32,
}

impl Quad {
    pub fn new(device: &Device) -> Self {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Quad vertex buffer"),
            contents: bytemuck::cast_slice(&VERTICES),
            usage: BufferUsages::VERTEX,
        });

        let indices_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Quad indices buffer"),
            contents: bytemuck::cast_slice(&INDICES),
            usage: BufferUsages::INDEX,
        });

        let num_indices = INDICES.len() as u32;

        Self {
            vertex_buffer,
            indices_buffer,
            num_indices,
        }
    }
}

impl Model for Quad {
    fn draw<'pass, 'my: 'pass>(
        &'my self,
        render_pass: &mut wgpu::RenderPass<'pass>,
        instances: std::ops::Range<u32>,
    ) {
        use wgpu::IndexFormat;

        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.indices_buffer.slice(..), IndexFormat::Uint16);

        render_pass.draw_indexed(0..self.num_indices, 0, instances)
    }
}
