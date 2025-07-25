struct GlobalsUniform {
    time: f32,
    _padding_0: u32,
    render_size: vec2<f32>,
    viewport_size: vec2<f32>,
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

    var scale_u = round(globals.viewport_size.x / globals.render_size.x);
    var scaled_rw = scale_u * globals.render_size.x;
    var ru = (scaled_rw - globals.viewport_size.x) / scaled_rw;
    out.uv.x -= out.uv.x * ru;

    var scale_v = round(globals.viewport_size.y / globals.render_size.y);
    var scaled_rh = scale_v * globals.render_size.y;
    var rv = (scaled_rh - globals.viewport_size.y) / scaled_rh;
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

