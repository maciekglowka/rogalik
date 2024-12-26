struct GlobalsUniform {
    time: f32,
    rw: u32,
    rh: u32,
    vw: u32,
    vh: u32,
}

struct VertexOutput {
    @location(0) uv: vec2<f32>,
    @builtin(position) clip_position: vec4<f32>,
}

@group(1) @binding(0)
var<uniform> globals: GlobalsUniform;

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

    var scale_u = round(f32(globals.vw) / f32(globals.rw));
    var scaled_rw  = scale_u * f32(globals.rw);
    var ru = (scaled_rw - f32(globals.vw)) / scaled_rw;
    out.uv.x -= out.uv.x * ru;

    var scale_v = round(f32(globals.vh) / f32(globals.rh));
    var scaled_rh  = scale_v * f32(globals.rh);
    var rv = (scaled_rh - f32(globals.vh)) / scaled_rh;
    out.uv.y -= out.uv.y * rv;
    
    return out;
}

@group(0)
@binding(0)
var post_image: texture_2d<f32>;

@group(0)
@binding(1)
var post_sampler: sampler;

@fragment
fn fs_main(vs: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(post_image, post_sampler, vs.uv);
}

