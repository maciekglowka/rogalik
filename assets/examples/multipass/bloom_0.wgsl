struct VertexOutput {
    @location(0) uv: vec2<f32>,
    @builtin(position) clip_position: vec4<f32>,
}

@vertex
fn vs_main(
    @builtin(vertex_index) vi: u32
) -> VertexOutput {
    var out: VertexOutput;
    out.uv = vec2<f32>(
        f32((vi << 1u) & 2u),
        f32(vi & 2u),
    );
    out.clip_position = vec4<f32>(out.uv * 2.0 - 1.0, 0.0, 1.0);
    out.uv.y = 1.0 - out.uv.y;
    return out;
}

// Fragment shader
 
struct GlobalsUniform {
    time: f32,
    _padding_0: f32,
    _padding_1: f32,
    _padding_2: f32,
    render_size: vec2<f32>,
    viewport_size: vec2<f32>,
}

@group(0) @binding(0)
var<uniform> globals: GlobalsUniform;

@group(1) @binding(0)
var s_scene: sampler;
@group(1) @binding(1)
var t_scene: texture_2d<f32>;

const LUMA = vec4(.299, .587, .114, 0.);

@fragment
fn fs_main(vs: VertexOutput) -> @location(0) vec4<f32> {
    let step_x = 1. / globals.viewport_size.x;

    var blur = vec4(0.);

    for (var x=-5; x<=5; x++) {
        let pixel = textureSample(t_scene, s_scene, vs.uv + vec2(f32(x) * step_x, 0.));
        let luma = dot(pixel, LUMA);
        blur += step(0.5, luma) * pixel;
    }

    blur /= 11.0;

    return blur;
}

