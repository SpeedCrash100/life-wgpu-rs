use std::ops::Range;

use bytemuck::{Pod, Zeroable};
use wgpu::RenderPass;

mod quad;
pub use quad::Quad;

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array![0 => Float32x3];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

pub trait Model {
    /// Executes commnands for Render Pass to bind buffers and draw model
    fn draw<'pass, 'my: 'pass>(
        &'my self,
        render_pass: &mut RenderPass<'pass>,
        instances: Range<u32>,
    );
}
