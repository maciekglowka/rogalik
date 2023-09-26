use winit::{
    window::Window
};
use rogalik_math::vectors::Vector2F;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ResourceId(pub usize);

pub trait Game<G: GraphicsContext> {
    fn setup(&mut self, context: &mut super::Context<G>);
    fn update(&mut self, context: &mut super::Context<G>);
}

pub trait GraphicsContext {
    fn new(window: &Window) -> Self;
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
    fn draw_indexed_sprite(
        &mut self,
        atlas_id: ResourceId,
        index: usize,
        position: Vector2F,
        size: Vector2F
    );
    fn draw_text(
        &mut self,
        font_id: ResourceId,
        text: &str,
        position: Vector2F,
        size: Vector2F
    );
    fn create_camera(&mut self, scale: f32, target: Vector2F) -> ResourceId;
    fn set_camera(&mut self, id: ResourceId);
    fn get_camera(&self, id: ResourceId) -> Option<&dyn Camera>;
    fn get_camera_mut(&mut self, id: ResourceId) -> Option<&mut dyn Camera>;
    fn get_viewport_size(&self) -> Vector2F;
}

pub trait Camera {
    fn get_target(&self) -> Vector2F;
    fn get_scale(&self) -> f32;
    fn set_target(&mut self, target: Vector2F);
    fn set_scale(&mut self, scale: f32);
}