use std::f32;

use rogalik::prelude::*;

const SPRITE_SIZE: f32 = 8.;
const PIXEL_SCALE: u32 = 16;

// Main game object.
struct GameState;
impl Game for GameState {
    fn setup(&mut self, context: &mut SetupContext) {
        // Load texture
        let diffuse_texture = context
            .graphics
            .load_texture("examples/colored_tilemap.png", false)
            .unwrap();

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
                    ..Default::default()
                },
            )
            .unwrap();

        // Create post processing pass

        let shader = context
            .graphics
            .load_shader(ShaderKind::PostProcess, "examples/pixel_blur.wgsl");

        context
            .graphics
            .add_post_process(
                "blur",
                PostProcessParams {
                    stage: PostProcessStage::Downscaled,
                    passes: vec![PassParams {
                        shader,
                        texture: None,
                        uniform: None,
                    }],
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
        _game: &mut Self::Game,
        context: &mut Context,
        _scenes: &mut SceneController<Self::Game>,
    ) {
        let _ = context.graphics.draw_atlas_sprite(
            "sprites",
            20,
            Vector2f::ZERO,
            0,
            Vector2f::splat(SPRITE_SIZE),
            SpriteParams::default(),
        );
    }
}

fn main() {
    let engine = EngineBuilder::new()
        .with_title("RGLK".to_string())
        .build(GameState, Box::new(MainScene));
    engine.run();
}
