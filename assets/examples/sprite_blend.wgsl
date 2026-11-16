// This shader blends between two colors, based on value from uniform buffer.
 
// Vertex shader
struct CameraUniform {
    view_proj: mat4x4<f32>
};
@group(1) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
    @location(2) secondary_color: vec4<f32>,
    @location(3) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) secondary_color: vec4<f32>,
    @location(2) tex_coords: vec2<f32>
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.color = model.color;
    out.secondary_color = model.secondary_color;
    out.tex_coords = model.tex_coords;
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);
    return out;
}

// Fragment shader
 
struct BlendUniform {
    blend: f32,
    _padding_0: f32,
    _padding_1: f32,
    _padding_2: f32,
}

@group(2) @binding(0)
var s_diffuse: sampler;
@group(2) @binding(1)
var t_diffuse: texture_2d<f32>;

@group(2) @binding(2)
var<uniform> blend_uniform: BlendUniform;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return mix(in.color, in.secondary_color, blend_uniform.blend) * textureSample(t_diffuse, s_diffuse, in.tex_coords);
}

