const MAX_LIGHTS: u32 = 16;

struct CameraUniform {
    view_proj: mat4x4<f32>
};
struct PointLight {
    position: vec3<f32>,
    radius: f32,
    color: vec3<f32>,
    falloff: f32,
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

    // Keep sprite's alpha
    let alpha = color[3];

    var total_light = lights_uniform.ambient;

    for (var i=0; i<i32(lights_uniform.light_count); i++) {
        let diff = lights_uniform.lights[i].position - in.world_position;
        let dist = max(0., dot(diff, diff));
        let max_dist = pow(lights_uniform.lights[i].radius, 2.);

        // Apply falloff.
        let t = max_dist - dist;
        let strength = smoothstep(0., max_dist * lights_uniform.lights[i].falloff, t);

        // Light elevation is equal to its radius.
        let elevated_light = vec3(
            lights_uniform.lights[i].position.xy,
            lights_uniform.lights[i].radius
        );
        // Normalized direction.
        let dir = normalize(elevated_light - in.world_position);
        let n = min(1., max(dot(normal, dir), 0.));

        // Apply the color
        let light = n * strength * vec4(lights_uniform.lights[i].color, 1.);
        total_light += light;
    }
    
    color *= total_light;
    color[3] = alpha;
    return color;
}

