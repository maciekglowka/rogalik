use winit::window::Window;
use rogalik_math::vectors::Vector2f;

use crate::EngineError;
use crate::structs::{Color, Params2d, ResourceId};

pub trait Game<G: GraphicsContext> {
    fn setup(&mut self, context: &mut super::Context<G>);
    fn update(&mut self, context: &mut super::Context<G>);
    fn resize(&mut self, _context: &mut super::Context<G>) {}
}

pub trait GraphicsContext {
    fn new() -> Self;
    fn create_context(&mut self, window: &Window);
    fn set_clear_color(&mut self, color: Color);
    fn resize(&mut self, w: u32, h: u32);
    fn render(&mut self);
    fn load_sprite_atlas(
        &mut self,
        name: &str,
        bytes: &[u8],
        rows: usize,
        cols: usize,
        padding: Option<(f32, f32)>
    );
    fn load_font(
        &mut self,
        name: &str,
        bytes: &[u8],
        rows: usize,
        cols: usize,
        padding: Option<(f32, f32)>
    );
    fn draw_atlas_sprite(
        &mut self,
        atlas: &str,
        index: usize,
        position: Vector2f,
        size: Vector2f,
        params: Params2d
    ) -> Result<(), EngineError>;
    fn draw_text(
        &mut self,
        font: &str,
        text: &str,
        position: Vector2f,
        size: f32,
        params: Params2d
    ) -> Result<(), EngineError>;
    fn text_dimensions(&self, font: &str, text: &str, size: f32) -> Vector2f;
    fn create_camera(&mut self, scale: f32, target: Vector2f) -> ResourceId;
    fn set_camera(&mut self, id: ResourceId);
    fn get_camera(&self, id: ResourceId) -> Option<&dyn Camera>;
    fn get_current_camera(&self) -> &dyn Camera;
    fn get_camera_mut(&mut self, id: ResourceId) -> Option<&mut dyn Camera>;
    // fn get_viewport_size(&self) -> Vector2f;
}

pub trait Camera {
    fn get_target(&self) -> Vector2f;
    fn get_scale(&self) -> f32;
    fn set_target(&mut self, target: Vector2f);
    fn set_scale(&mut self, scale: f32);
    fn camera_to_world(&self, v: Vector2f) -> Vector2f;
}