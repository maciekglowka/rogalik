struct GlobalsUniform {
    time: f32,
    _padding_0: u32,
    _padding_1: u32,
    _padding_2: u32,
    render_size: vec2<f32>,
    viewport_size: vec2<f32>,
    clear_color: vec4<f32>,
}

struct VertexOutput {
    @location(0) uv: vec2<f32>,
    @builtin(position) clip_position: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> globals: GlobalsUniform;

@vertex
fn vs_main(
    @builtin(vertex_index) vi: u32
) -> VertexOutput {
    var out: VertexOutput;
    let pos = vec2<f32>(
        f32((vi << 1u) & 2u),
        f32(vi & 2u),
    );
    out.clip_position = vec4<f32>(pos * 2.0 - 1.0, 0.0, 1.0);
    let screen_uv = vec2<f32>(pos.x, 1.0 - pos.y);

    // Round is used instead of floor as OSes tend to e.g. 1px from the requested inner_size
    // and it seems better to crop slightly in such a situation.
    //
    // An option to pass custom upscale shader should be considered in the future.
    let scale = min(
        round(globals.viewport_size.x / globals.render_size.x),
        round(globals.viewport_size.y / globals.render_size.y),
    );
    let scaled_size = scale * globals.render_size;

    let origin = floor((globals.viewport_size - scaled_size) * 0.5);
    out.uv = (screen_uv * globals.viewport_size - origin) / scaled_size;
    
    return out;
}

@group(1)
@binding(0)
var post_sampler: sampler;

@group(1)
@binding(1)
var post_image: texture_2d<f32>;

@fragment
fn fs_main(vs: VertexOutput) -> @location(0) vec4<f32> {
    let col = textureSample(post_image, post_sampler, vs.uv);
    let in_bounds = all(vs.uv >= vec2<f32>(0.0)) && all(vs.uv <= vec2<f32>(1.0));
    return select(globals.clear_color, col, in_bounds);
}

