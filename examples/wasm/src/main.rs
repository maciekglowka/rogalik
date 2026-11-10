use rogalik::prelude::*;
use wasm_bindgen::prelude::*;

// Main game object.
struct GameState;
impl Game for GameState {
    fn setup(&mut self, context: &mut Context) {
        context
            .graphics
            .load_font(
                "monogram",
                "examples/monogram.ttf",
                FontParams {
                    // Make slightly tighter lines.
                    line_spacing: Some(0.75),
                    ..Default::default()
                },
            )
            .unwrap();

        // Create camera
        context.graphics.create_camera(1., Vector2f::ZERO);
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
        let text = "Hello Wasm!";
        let font_size = 32.;

        let width = context.graphics.text_dimensions("m5x7", text, font_size).x;

        // Single line text.
        context
            .graphics
            .draw_text(
                "m5x7",
                text,
                Vector2f::new(-0.5 * width, 2. * font_size),
                0,
                font_size,
                SpriteParams::default(),
            )
            .unwrap();
    }
}

#[wasm_bindgen(start)]
pub fn main() {
    let engine = EngineBuilder::new().build_wasm(GameState, Box::new(MainScene));
    engine.run();
}
