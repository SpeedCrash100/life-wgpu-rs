// Vertex shader

struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0) 
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var<storage, read> life_field: array<u32>;


struct VertexInput {
    @location(0) position: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

struct CellInfo {
    @location(1) pos: vec2<f32>,
    @location(2) idx: u32,
}

@vertex
fn vs_main(
    model: VertexInput,
    instance: CellInfo,
) -> VertexOutput {

    var shift = vec4<f32>(instance.pos, 0.0, 0.0);
    var position = vec4<f32>(model.position, 1.0) + shift;

    var out: VertexOutput;
    out.clip_position = camera.view_proj * position;
    if life_field[instance.idx] > 0u {
        out.color = vec4<f32>(1.0, 0.0, 0.0, 1.0);
    } else {
        out.color = vec4<f32>(0.5, 0.5, 0.5, 1.0);
    }
    
    return out;
}

// Fragment shader


@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}

