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

@group(1)
@binding(0)
var input_sampler: sampler;

@group(1)
@binding(1)
var input_image: texture_2d<f32>;


@fragment
fn fs_main(vs: VertexOutput) -> @location(0) vec4<f32> {
    // Pixel sampled twice ;(
    let col = textureSample(input_image, input_sampler, vs.uv);

    let dim = textureDimensions(input_image);
    let step_x = 1. / f32(dim.x);
    let step_y = 1. / f32(dim.y);

    var blur = vec4(0.);

    for (var x=-2; x<=2; x++) {
        for (var y=-2; y<=2; y++) {
            let pixel = textureSample(input_image, input_sampler, vs.uv + vec2(f32(x) * step_x, f32(y) * step_y));
            blur += pixel;
        }
    }

    blur /= 25.0;

    return mix(col, blur, 0.2);
}

