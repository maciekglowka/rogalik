use rogalik::prelude::*;

const SPRITE_W: u32 = 8;
const SPRITE_H: u32 = 16;
const SLICE: u32 = 2;

const PIXEL_SCALE: u32 = 4;
const BOARD_DIM: i32 = 4;

// Main game object.
struct GameState;
impl Game for GameState {
    fn setup(&mut self, context: &mut Context) {
        // Load texture
        let diffuse_texture = Some(context.graphics.load_texture("examples/slice-test.png"));

        // Create sprite material
        context.graphics.load_material(
            "sprites",
            MaterialParams {
                atlas: Some(AtlasParams {
                    cols: 2,
                    rows: 2,
                    padding: Some((2., 2.)),
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
        let _ = context.graphics.draw_atlas_sprite(
            "sprites",
            3,
            Vector2f::ZERO,
            0,
            Vector2f::new(3. * SPRITE_W as f32, 4. * SPRITE_H as f32),
            SpriteParams {
                slice: Some(2),
                rotate: Some(1.),
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
