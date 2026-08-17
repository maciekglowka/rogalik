use std::f32;

use rogalik::prelude::*;

const SPRITE_SIZE: f32 = 8.;
const PIXEL_SCALE: u32 = 4;

// Main game object.
struct GameState;
impl Game for GameState {
    fn setup(&mut self, context: &mut Context) {
        // Load texture
        let diffuse_texture = Some(
            context
                .graphics
                .load_texture("examples/colored_tilemap.png"),
        );

        // Create sprite material
        context.graphics.load_material(
            "sprites",
            MaterialParams {
                atlas: Some(AtlasParams::Grid {
                    cols: 16,
                    rows: 10,
                    padding: Some((1, 1)),
                }),
                diffuse_texture,
                ..Default::default()
            },
        );

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
        _game: &mut Self::Game,
        context: &mut Context,
        _scenes: &mut SceneController<Self::Game>,
    ) {
        // Simple
        let _ = context.graphics.draw_atlas_sprite(
            "sprites",
            20,
            Vector2f::ZERO,
            0,
            Vector2f::splat(SPRITE_SIZE),
            SpriteParams::default(),
        );
        // 9-slice.
        let _ = context.graphics.draw_atlas_sprite(
            "sprites",
            103,
            Vector2f::new(2. * SPRITE_SIZE, 0.),
            0,
            Vector2f::splat(2. * SPRITE_SIZE),
            SpriteParams {
                slice: Some(2),
                ..Default::default()
            },
        );
        // Rotation (sprite is rotated around it's center).
        let _ = context.graphics.draw_atlas_sprite(
            "sprites",
            4,
            Vector2f::new(0., 2. * SPRITE_SIZE),
            0,
            Vector2f::splat(SPRITE_SIZE),
            SpriteParams {
                rotate: 0.5 * f32::consts::PI,
                ..Default::default()
            },
        );
    }
}

fn main() {
    let engine = EngineBuilder::new()
        .with_title("RGLK".to_string())
        .build(GameState, Box::new(MainScene));
    engine.run();
}
