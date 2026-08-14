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
        let text = "Hello World!";
        let paragraph =
            "This is a small\nmultiline paragraph. A long line should be wrapped.\n\nEnd.";
        let font_size = 32.;

        let width = context.graphics.text_dimensions("pixel", text, font_size).x;

        // Single line text.
        context
            .graphics
            .draw_text(
                "pixel",
                text,
                Vector2f::new(-0.5 * width, 2. * font_size as f32),
                0,
                font_size,
                SpriteParams::default(),
            )
            .unwrap();

        // Multiline wrapped text.
        context
            .graphics
            .draw_textbox(
                "pixel",
                paragraph,
                Vector2f::new(-0.5 * width, 0.),
                0,
                font_size,
                200.,
                SpriteParams::default(),
            )
            .unwrap();
    }
}

fn main() {
    let engine = EngineBuilder::new()
        .with_title("RGLK Bitmap Font".to_string())
        .resizable(true)
        .build(GameState, Box::new(MainScene));
    engine.run();
}
