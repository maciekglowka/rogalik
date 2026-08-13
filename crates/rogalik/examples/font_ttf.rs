use rogalik::prelude::*;

// Main game object.
struct GameState;
impl Game for GameState {
    fn setup(&mut self, context: &mut Context) {
        // Load a default font
        context
            .graphics
            .load_font("pixel", "examples/m5x7.ttf", FontParams::default());

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
        let text = "Hello World";
        let font_size = 36;

        let width = context.graphics.text_dimensions("pixel", text, font_size).x;

        let _ = context.graphics.draw_text(
            "pixel",
            text,
            Vector2f::new(-0.5 * width, 0.),
            0,
            font_size,
            SpriteParams::default(),
        );
    }
}

fn main() {
    let engine = EngineBuilder::new()
        .with_title("RGLK Bitmap Font".to_string())
        .resizable(true)
        .build(GameState, Box::new(MainScene));
    engine.run();
}
