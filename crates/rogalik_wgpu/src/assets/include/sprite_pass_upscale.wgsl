struct GlobalsUniform {
    time: f32,
    _padding_0: u32,
    render_size: vec2<u32>,
    viewport_size: vec2<u32>,
    _padding_1: u32,
    _padding_2: u32,
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

    var scale_u = round(f32(globals.viewport_size.x) / f32(globals.render_size.x));
    var scaled_rw  = scale_u * f32(globals.render_size.x);
    var ru = (scaled_rw - f32(globals.viewport_size.x)) / scaled_rw;
    out.uv.x -= out.uv.x * ru;

    var scale_v = round(f32(globals.viewport_size.y) / f32(globals.render_size.y));
    var scaled_rh  = scale_v * f32(globals.render_size.y);
    var rv = (scaled_rh - f32(globals.viewport_size.y)) / scaled_rh;
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

