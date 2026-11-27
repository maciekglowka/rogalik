use std::f32;

use rogalik::{
    input::{MouseButton, Scroll},
    prelude::*,
};

const SPRITE_SIZE: f32 = 8.;
const RENDER_W: u32 = 240;
const RENDER_H: u32 = 160;

const WORLD_W: usize = 35;
const WORLD_H: usize = 35;

// Main game object.
#[derive(Default)]
struct GameState {
    drag_start: Option<Vector2f>,
    main_camera: Option<Id<Camera2d>>,
    ui_camera: Option<Id<Camera2d>>,
}
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

        // Create sprite materials
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
                    textures: vec![color_texture],
                    ..Default::default()
                },
            )
            .unwrap();

        // Create cameras
        self.main_camera = Some(context.graphics.create_camera(
            4.,
            0.5 * SPRITE_SIZE * Vector2f::new(WORLD_W as f32, WORLD_H as f32),
        ));
        // Make (0, 0) at bottom-left for the UI.
        self.ui_camera = Some(context.graphics.create_camera(
            1.,
            0.5 * Vector2f::new(RENDER_W as f32, RENDER_H as f32),
        ));
    }

    fn update(
        &mut self,
        context: &mut Context,
    ) {
        context.graphics.set_camera(&self.main_camera.unwrap());

        if context.input.is_mouse_button_pressed(MouseButton::Left) {
            self.drag_start = Some(context.input.get_mouse_physical_position());
        }
        if context.input.is_mouse_button_down(MouseButton::Left) && let Some(start) = self.drag_start {
            let end = context.input.get_mouse_physical_position();
            let camera = context.graphics.get_current_camera_mut();
            let d = (start - end) / camera.get_scale();
            camera.set_target(camera.get_target() + d);
            self.drag_start = Some(end);
        }
        if context.input.is_mouse_button_released(MouseButton::Left) {
            self.drag_start = None;
        }

        let scroll = match context.input.get_mouse_scroll() {
            Some(Scroll::Pixel(_, y)) => Some(y),
            Some(Scroll::Line(_, y)) => Some(y),
            None => None,
        };
        if let Some(y) = scroll {
            let camera = context.graphics.get_current_camera_mut();
            let scale = camera.get_scale();
            camera.set_scale((scale + y.signum()).max(1.));
        }

        // Draw world.
        for x in 0..WORLD_W {
            for y in 0..WORLD_H {
                context
                    .graphics
                    .draw_atlas_sprite(
                        "sprites",
                        140,
                        SPRITE_SIZE * Vector2f::new(x as f32, y as f32),
                        0,
                        Vector2f::splat(SPRITE_SIZE),
                        SpriteParams::default(),
                    )
                    .unwrap();
            }
        }

        // Draw UI
        context.graphics.set_camera(&self.ui_camera.unwrap());

        context
            .graphics
            .draw_atlas_sprite(
                "sprites",
                6 * 16,
                Vector2f::splat(4.),
                100,
                Vector2f::new(RENDER_W as f32 - 8., 2. * SPRITE_SIZE),
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
        .build(GameState::default());
    engine.run();
}
