use wgpu::{
    include_wgsl, ColorTargetState, Device, FragmentState, ShaderModule, TextureFormat,
    VertexBufferLayout, VertexState,
};

use crate::{bindable::CellPos, model::Vertex};

pub struct Shader {
    module: ShaderModule,
    vertex_buffer_layout: Vec<VertexBufferLayout<'static>>,
    color_target_states: Vec<Option<ColorTargetState>>,
}

impl Shader {
    pub fn new(device: &Device, texture_format: TextureFormat) -> Self {
        let module = device.create_shader_module(include_wgsl!("../shaders/shader.wgsl"));

        let vertex_buffer_layout = vec![Vertex::desc(), CellPos::desc()];
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
}
