use std::f32;

use rogalik::prelude::*;

const SPRITE_SIZE: f32 = 200.;
const PIXEL_SCALE: u32 = 4;

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
                    textures: vec![diffuse_texture],
                    ..Default::default()
                },
            )
            .unwrap();

        // Create post processing pass

        let shader = context
            .graphics
            .load_shader(ShaderKind::PostProcess, "examples/lut/lut.wgsl");
        let lut_texture = context
            .graphics
            .load_texture("examples/lut/lut.png", false)
            .unwrap();

        context
            .graphics
            .add_post_process(
                "lut",
                shader,
                PostProcessParams {
                    texture: Some(lut_texture),
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
        // Draw entire atlas.
        let _ = context.graphics.draw_sprite(
            "sprites",
            Vector2f::splat(-0.5 * SPRITE_SIZE),
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
