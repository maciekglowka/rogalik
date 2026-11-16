use std::f32;

use rogalik::prelude::*;

const SPRITE_SIZE: f32 = 32.;
const PIXEL_SCALE: u32 = 4;

// Main game object.
#[derive(Default)]
struct GameState {
    blend: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::NoUninit, bytemuck::Zeroable)]
struct BlendUniform {
    blend: f32,
    _padding: [f32; 3], // for WASM
}

impl Game for GameState {
    fn setup(&mut self, context: &mut SetupContext) {
        // Load texture
        let diffuse_texture = context
            .graphics
            .load_texture("examples/colored_tilemap.png", false)
            .unwrap();

        let shader = context
            .graphics
            .load_shader(ShaderKind::Sprite, "examples/sprite_blend.wgsl");

        // Create sprite material
        context
            .graphics
            .create_material(
                "sprites",
                MaterialParams {
                    atlas: Some(AtlasParams::Grid {
                        cols: 16,
                        rows: 10,
                        padding: Some((1, 1)),
                    }),
                    textures: vec![diffuse_texture],
                    shader: Some(shader),
                    uniform: Some(ShaderUniform::init(BlendUniform::default())),
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
}

// At least one scene is needed.
struct MainScene;
impl Scene for MainScene {
    type Game = GameState;

    fn update(
        &mut self,
        game: &mut Self::Game,
        context: &mut Context,
        _scenes: &mut SceneController<Self::Game>,
    ) {
        if context.input.is_key_pressed(rogalik::input::KeyCode::Space) {
            game.blend = (game.blend + 0.1) % 1.0;
            context
                .graphics
                .update_material_uniform(
                    "sprites",
                    BlendUniform {
                        blend: game.blend,
                        ..Default::default()
                    },
                )
                .unwrap();
        }

        let _ = context.graphics.draw_atlas_sprite(
            "sprites",
            20,
            Vector2f::ZERO,
            0,
            Vector2f::splat(SPRITE_SIZE),
            SpriteParams {
                color: Color(255, 0, 0, 255),
                secondary_color: Color(0, 0, 255, 255),
                ..Default::default()
            },
        );
    }
}

fn main() {
    let engine = EngineBuilder::new()
        .with_title("RGLK".to_string())
        .build(GameState::default(), Box::new(MainScene));
    engine.run();
}
