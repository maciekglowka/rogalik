use std::f32;

use rogalik::prelude::*;

const SPRITE_SIZE: f32 = 8.;
const RENDER_W: u32 = 240;
const RENDER_H: u32 = 160;

// Main game object.
struct GameState;
impl Game for GameState {
    fn setup(&mut self, context: &mut SetupContext) {
        context
            .graphics
            .set_rendering_resolution(RENDER_W, RENDER_H);

        // Load textures
        let color_texture = context
            .graphics
            .load_texture("examples/colored_tilemap.png", false)
            .unwrap();
        let mono_texture = context
            .graphics
            .load_texture("examples/monochrome_tilemap.png", false)
            .unwrap();

        // Create sprite materials
        context
            .graphics
            .create_material(
                "color",
                MaterialParams {
                    atlas: Some(AtlasParams::Grid {
                        cols: 16,
                        rows: 10,
                        padding: Some((1, 1)),
                    }),
                    textures: vec![color_texture],
                    ..Default::default()
                },
            )
            .unwrap();
        context
            .graphics
            .create_material(
                "mono",
                MaterialParams {
                    atlas: Some(AtlasParams::Grid {
                        cols: 16,
                        rows: 10,
                        padding: Some((1, 1)),
                    }),
                    textures: vec![mono_texture],
                    ..Default::default()
                },
            )
            .unwrap();

        // Create camera
        context.graphics.create_camera(1., Vector2f::ZERO);
    }

    fn update(&mut self, context: &mut Context) {
        // Simple
        let _ = context.graphics.draw_atlas_sprite(
            "color",
            20,
            Vector2f::ZERO,
            0,
            Vector2f::splat(SPRITE_SIZE),
            SpriteParams::default(),
        );
        // 9-slice.
        let _ = context.graphics.draw_atlas_sprite(
            "color",
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
            "mono",
            4,
            Vector2f::new(0., 2. * SPRITE_SIZE),
            0,
            Vector2f::splat(SPRITE_SIZE),
            SpriteParams {
                rotate: 0.5 * f32::consts::PI,
                ..Default::default()
            },
        );

        // Stretched BG.
        context
            .graphics
            .draw_atlas_sprite(
                "color",
                6 * 16,
                -0.5 * Vector2f::new(RENDER_W as f32, RENDER_H as f32),
                -1,
                Vector2f::new(RENDER_W as f32, RENDER_H as f32),
                SpriteParams {
                    slice: Some(2),
                    ..Default::default()
                },
            )
            .unwrap();
    }
}

fn main() {
    let engine = EngineBuilder::new()
        .with_title("RGLK".to_string())
        .with_physical_size(960, 640)
        .resizable(true)
        .build(GameState);
    engine.run();
}
