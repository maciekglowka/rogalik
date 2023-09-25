use winit::{
    dpi::PhysicalSize,
    window::Window
};
use rogalik_math::vectors::Vector2F;

pub type ResourceId = usize;

// pub typ

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
}
