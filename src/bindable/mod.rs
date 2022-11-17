use wgpu::Buffer;
use wgpu::{BindGroup, BindGroupLayout, ComputePass, DynamicOffset, RenderPass};

mod cellpos;
pub use cellpos::CellPos;
pub use cellpos::CellPosInstances;

mod fieldinfo;
pub use fieldinfo::FieldInfo;

mod fieldstate;
pub use fieldstate::FieldState;

pub trait HaveBindGroup {
    /// Returns layout for binding
    fn get_bind_layout(&self) -> &BindGroupLayout;

    /// Return bind group descriptor
    fn get_bind(&self) -> &BindGroup;
}

pub trait BindableToComputePass: HaveBindGroup {
    fn bind_to_compute_pass<'my: 'pass, 'pass>(
        &'my self,
        cp: &mut ComputePass<'pass>,
        index: u32,
        offsets: &[DynamicOffset],
    ) {
        cp.set_bind_group(index, self.get_bind(), offsets);
    }
}

pub trait BinableToRenderPass: HaveBindGroup {
    fn bind_to_render_pass<'my: 'pass, 'pass>(
        &'my self,
        rp: &mut RenderPass<'pass>,
        index: u32,
        offsets: &[DynamicOffset],
    ) {
        rp.set_bind_group(index, self.get_bind(), offsets);
    }
}

pub trait HaveBuffer {
    fn get_buffer(&self) -> &Buffer;
}

pub trait BindableToVertexBuffers: HaveBuffer {
    fn bind_vertex_to_render_pass<'my: 'pass, 'pass>(
        &'my self,
        rp: &mut RenderPass<'pass>,
        slot: u32,
    ) {
        rp.set_vertex_buffer(slot, self.get_buffer().slice(..))
    }
}
