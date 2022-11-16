use bytemuck::{Pod, Zeroable};
use wgpu::{
    include_wgsl, BindGroup, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, Buffer, ColorTargetState, Device, FragmentState,
    ShaderModule, ShaderStages, TextureFormat, VertexBufferLayout, VertexState,
};

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array![0 => Float32x3];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct CellInfo {
    pub pos: [f32; 2],
    pub living: u32,
}

impl CellInfo {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![1 => Float32x2, 2 => Uint32];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}

pub struct Shader {
    module: ShaderModule,
    vertex_buffer_layout: Vec<VertexBufferLayout<'static>>,
    color_target_states: Vec<Option<ColorTargetState>>,
}

impl Shader {
    pub fn new(device: &Device, texture_format: TextureFormat) -> Self {
        let module = device.create_shader_module(include_wgsl!("../shaders/shader.wgsl"));

        let vertex_buffer_layout = vec![Vertex::desc(), CellInfo::desc()];
        let color_target_states = vec![Some(ColorTargetState {
            format: texture_format,
            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
            write_mask: wgpu::ColorWrites::ALL,
        })];

        Self {
            module,
            vertex_buffer_layout,
            color_target_states,
        }
    }

    pub fn vertex_state(&self) -> VertexState {
        VertexState {
            module: &self.module,
            entry_point: "vs_main",
            buffers: &self.vertex_buffer_layout,
        }
    }

    pub fn frag_state(&self) -> FragmentState {
        FragmentState {
            module: &self.module,
            entry_point: "fs_main",
            targets: &self.color_target_states,
        }
    }

    pub fn create_camera_bind_group(
        &self,
        device: &Device,
        camera_buffer: &Buffer,
    ) -> (BindGroupLayout, BindGroup) {
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
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        (bind_group_layout, bind_group)
    }
}
