use rogalik::prelude::*;

struct GameState;
impl Game for GameState {
    fn setup(&mut self, context: &mut Context) {
        // Load a default font
        context
            .graphics
            .load_font("pixel", "examples/font.png", 16, 16, Some((11., 7.)), None);

        // Create camera
        context.graphics.create_camera(1., Vector2f::ZERO);
    }
}

struct MainScene;
impl Scene for MainScene {
    type Game = GameState;

    fn update(
        &mut self,
        _game: &mut Self::Game,
        context: &mut Context,
        _scenes: &mut SceneController<Self::Game>,
    ) {
        let text = "Hello World!";
        let font_size = 36.;

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
        .with_title("Hello Rogalik!".to_string())
        .build(GameState, Box::new(MainScene));
    engine.run();
}
