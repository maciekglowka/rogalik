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

@group(1)
@binding(0)
var s_scene: sampler;

@group(1)
@binding(1)
var t_scene: texture_2d<f32>;

@group(2)
@binding(2)
var s_lut: sampler;

@group(2)
@binding(3)
var t_lut: texture_2d<f32>;

const LUT_SIZE: f32 = 32.0;

@fragment
fn fs_main(vs: VertexOutput) -> @location(0) vec4<f32> {
    let col = textureSample(t_scene, s_scene, vs.uv);

    var u = floor((LUT_SIZE - 1.) * col.b) / LUT_SIZE;
    u += floor((LUT_SIZE - 1.) * col.r) / (LUT_SIZE * LUT_SIZE);
    var v = floor((LUT_SIZE - 1.) * col.g) / LUT_SIZE;

    let lut_col = textureSample(t_lut, s_lut, vec2(u, v));
    return lut_col;
}

