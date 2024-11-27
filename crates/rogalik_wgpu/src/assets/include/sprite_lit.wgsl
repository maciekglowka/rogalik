const MAX_LIGHTS: u32 = 16;

struct CameraUniform {
    view_proj: mat4x4<f32>
};
struct PointLight {
    position: vec3<f32>,
    strength: f32,
    color: vec4<f32>,
}
struct LightsUniform {
    light_count: u32,
    _padding_0: f32,
    _padding_1: f32,
    _padding_2: f32,
    ambient: vec4<f32>,
    lights: array<PointLight, MAX_LIGHTS>,
}

// Vertex shader
@group(1) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
    @location(2) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) world_position: vec3<f32>
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.color = model.color;
    out.tex_coords = model.tex_coords;
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);
    out.world_position = model.position;
    return out;
}

// Fragment shader
@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;
@group(0) @binding(2)
var t_normal: texture_2d<f32>;
@group(0) @binding(3)
var s_normal: sampler;
@group(3) @binding(0)
var<uniform> lights_uniform: LightsUniform;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var color = in.color * textureSample(t_diffuse, s_diffuse, in.tex_coords);

    var normal = normalize(textureSample(t_normal, s_normal, in.tex_coords).xyz * 2.0 - 1.0);
    let a = color[3];

    var light = lights_uniform.ambient;

    for (var i=0; i<i32(lights_uniform.light_count); i++) {
        let d = lights_uniform.lights[i].position - in.world_position;
        let dist = max(1., dot(d, d));
        let max_dist = pow(lights_uniform.lights[i].strength, 2.);
        let f = step(dist, max_dist);

        let dir = normalize(vec3(lights_uniform.lights[i].position.xy, 0.) - in.world_position);
        // UP vector should be netural - scale cos a so pi/2 == 1
        let n = dot(normal, dir) + 1.;

        let c = n * f * lights_uniform.lights[i].color;
        light += c;
    }
    
    color *= light;
    color[3] = a;
    return color;
}

