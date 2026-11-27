use rogalik::prelude::*;

const SPRITE_SIZE: f32 = 16.;
const PIXEL_SCALE: u32 = 4;
const BOARD_DIM: i32 = 4;

const MAX_LIGHTS: usize = 4;

// Main game object.
struct GameState;
impl Game for GameState {
    fn setup(&mut self, context: &mut SetupContext) {
        // Set shared lights uniform
        context.graphics.set_user_uniform(LightUniform::default());

        // Load diffuse texture
        let diffuse_texture = context
            .graphics
            .load_texture("examples/lighting/diffuse.png", false)
            .unwrap();

        // Load normal texture
        let normal_texture = context
            .graphics
            .load_texture("examples/lighting/normal.png", true)
            .unwrap();

        let shader = context
            .graphics
            .load_shader(ShaderKind::Sprite, "examples/lighting/sprite_lit.wgsl");

        // Create sprite material
        context
            .graphics
            .create_material(
                "sprites",
                MaterialParams {
                    textures: vec![diffuse_texture, normal_texture],
                    shader: Some(shader),
                    ..Default::default()
                },
            )
            .unwrap();

        // Create camera
        context.graphics.create_camera(1., Vector2f::ZERO);
    }
    fn resize(&mut self, context: &mut Context) {
        // Set pixel perfect rendering
        let viewport = context.get_physical_size();
        context.graphics.set_rendering_resolution(
            viewport.x as u32 / PIXEL_SCALE,
            viewport.y as u32 / PIXEL_SCALE,
        );
    }

    fn update(&mut self, context: &mut Context) {
        for x in -BOARD_DIM..=BOARD_DIM {
            for y in -BOARD_DIM..=BOARD_DIM {
                let _ = context.graphics.draw_atlas_sprite(
                    "sprites",
                    0,
                    SPRITE_SIZE * Vector2i::new(x, y).as_f32(),
                    0,
                    Vector2f::splat(SPRITE_SIZE),
                    SpriteParams::default(),
                );
            }
        }

        let mouse = context.input.get_mouse_physical_position();
        let mouse_world = context.graphics.get_current_camera().camera_to_world(mouse);

        let static_position = SPRITE_SIZE * Vector2i::splat(-BOARD_DIM).as_f32();

        let lights = LightUniform {
            light_count: 2,
            lights: [
                // Dynamic light
                PointLight {
                    position: [mouse_world.x, mouse_world.y, 0.],
                    radius: 16.,
                    color: [1., 0.5, 0.],
                    falloff: 0.5,
                },
                // Static light
                PointLight {
                    position: [static_position.x, static_position.y, 0.],
                    radius: 64.,
                    color: [0.5, 0., 1.],
                    falloff: 0.5,
                },
                PointLight::default(),
                PointLight::default(),
            ],
            ..Default::default()
        };
        context.graphics.update_user_uniform(lights).unwrap();
    }
}

fn main() {
    let engine = EngineBuilder::new()
        .with_title("RGLK".to_string())
        .build(GameState);
    engine.run();
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::NoUninit, bytemuck::Zeroable)]
struct LightUniform {
    light_count: u32,
    _padding_0: f32,
    _padding_1: f32,
    _padding_2: f32,
    ambient: [f32; 4],
    lights: [PointLight; MAX_LIGHTS],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct PointLight {
    position: [f32; 3],
    radius: f32,
    color: [f32; 3],
    falloff: f32,
}
