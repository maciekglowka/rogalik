use rogalik::prelude::*;

const SPRITE_SIZE: f32 = 16.;
const PIXEL_SCALE: u32 = 4;
const BOARD_DIM: i32 = 4;

// Main game object.
struct GameState;
impl Game for GameState {
    fn setup(&mut self, context: &mut Context) {
        // Load diffuse texture
        let diffuse_texture = Some(
            context
                .graphics
                .load_texture("examples/lighting/diffuse.png"),
        );

        // Load normal texture
        let normal_texture = Some(
            context
                .graphics
                .load_texture("examples/lighting/normal.png"),
        );

        // Create sprite material
        context.graphics.load_material(
            "sprites",
            MaterialParams {
                atlas: Some(AtlasParams {
                    cols: 1,
                    rows: 1,
                    padding: None,
                }),
                diffuse_texture,
                normal_texture,
                shader: context
                    .graphics
                    .get_builtin_shader(BuiltInShader::SpriteLit),
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
        for x in -BOARD_DIM..=BOARD_DIM {
            for y in -BOARD_DIM..=BOARD_DIM {
                let _ = context.graphics.draw_atlas_sprite(
                    "sprites",
                    0,
                    SPRITE_SIZE * Vector2i::new(x, y).as_f32(),
                    0,
                    Vector2f::splat(SPRITE_SIZE),
                    SpriteParams::default(),
                );
            }
        }

        let mouse = context.input.get_mouse_physical_position();
        let mouse_world = context.graphics.get_current_camera().camera_to_world(mouse);

        // Dynamic light
        let _ = context
            .graphics
            .add_light(mouse_world, 16., Color(255, 128, 0, 255), 0.5);

        // Static light
        let _ = context.graphics.add_light(
            SPRITE_SIZE * Vector2i::splat(-BOARD_DIM).as_f32(),
            64.,
            Color(128, 0, 255, 255),
            0.5,
        );
    }
}

fn main() {
    let engine = EngineBuilder::new()
        .with_title("RGLK".to_string())
        .build(GameState, Box::new(MainScene));
    engine.run();
}
