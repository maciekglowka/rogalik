use winit::{
    window::Window
};
use rogalik_math::vectors::Vector2f;

use crate::structs::{Color, Params2d, ResourceId};

pub trait Game<G: GraphicsContext> {
    fn setup(&mut self, context: &mut super::Context<G>);
    fn update(&mut self, context: &mut super::Context<G>);
}

pub trait GraphicsContext {
    fn new(window: &Window) -> Self;
    fn set_clear_color(&mut self, color: Color);
    fn resize(&mut self, w: u32, h: u32);
    fn render(&mut self);
    fn load_sprite_atlas(
        &mut self,
        bytes: &[u8],
        rows: usize,
        cols: usize
    ) -> ResourceId;
    fn load_font(
        &mut self,
        bytes: &[u8],
        rows: usize,
        cols: usize
    ) -> ResourceId;
    fn draw_atlas_sprite(
        &mut self,
        atlas_id: ResourceId,
        index: usize,
        position: Vector2f,
        size: Vector2f,
        params: Params2d
    );
    fn draw_text(
        &mut self,
        font_id: ResourceId,
        text: &str,
        position: Vector2f,
        size: f32,
        params: Params2d
    );
    fn text_dimensions(&self, font_id: ResourceId, text: &str, size: f32) -> Vector2f;
    fn create_camera(&mut self, scale: f32, target: Vector2f) -> ResourceId;
    fn set_camera(&mut self, id: ResourceId);
    fn get_camera(&self, id: ResourceId) -> Option<&dyn Camera>;
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