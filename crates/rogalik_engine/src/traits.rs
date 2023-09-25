use winit::{
    window::Window
};
use rogalik_math::vectors::Vector2F;

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct ResourceId(pub usize);

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
    fn draw_indexed_sprite(
        &mut self,
        atlas_id: ResourceId,
        index: usize,
        position: Vector2F,
        size: Vector2F
    );
    fn get_camera(&self) -> Option<&dyn Camera>;
    fn get_camera_mut(&mut self) -> Option<&mut dyn Camera>;
}

pub trait Camera {
    fn get_target(&self) -> Vector2F;
    fn get_scale(&self) -> f32;
    fn set_target(&mut self, target: Vector2F);
    fn set_scale(&mut self, scale: f32);
}