use rogalik::prelude::*;

// Main game object.
struct GameState;
impl Game for GameState {
    fn setup(&mut self, context: &mut Context) {
        // Load a default font
        context
            .graphics
            .load_font_atlas(
                "pixel",
                "examples/font.png",
                AtlasParams::Grid {
                    cols: 16,
                    rows: 6,
                    padding: Some((1, 1)),
                },
                FontParams {
                    // Typically manual spacing is needed for bitmap fonts.
                    character_spacing: Some(0.125),
                    line_spacing: Some(1.125),
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
        let text = "Hello World";
        let paragraph =
            "This is a small\nmultiline paragraph. A long line should be wrapped.\n\nEnd.";

        let font_size = 36.;

        let width = context.graphics.text_dimensions("pixel", text, font_size).x;

        context
            .graphics
            .draw_text(
                "pixel",
                text,
                Vector2f::new(-0.5 * width, 2. * font_size),
                0,
                font_size,
                SpriteParams::default(),
            )
            .unwrap();

        context
            .graphics
            .draw_textbox(
                "pixel",
                paragraph,
                Vector2f::new(20., 0.),
                0,
                0.5 * font_size,
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
