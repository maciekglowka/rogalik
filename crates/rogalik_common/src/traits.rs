use rogalik_math::vectors::Vector2f;
use winit::window::Window;

use crate::structs::{Color, EngineError, ResourceId, ShaderKind, SpriteParams};

pub trait GraphicsContext {
    fn create_context(&mut self, window: &Window);
    fn has_context(&self) -> bool;
    fn set_clear_color(&mut self, color: Color);
    fn resize(&mut self, w: u32, h: u32);
    fn render(&mut self);
    fn load_material(&mut self, name: &str, params: crate::MaterialParams);
    fn load_shader(&mut self, kind: ShaderKind, path: &str) -> ResourceId;
    fn load_font(
        &mut self,
        name: &str,
        bytes: &[u8],
        rows: usize,
        cols: usize,
        padding: Option<(f32, f32)>,
    );
    fn draw_atlas_sprite(
        &mut self,
        material: &str,
        index: usize,
        position: Vector2f,
        z_index: i32,
        size: Vector2f,
        params: SpriteParams,
    ) -> Result<(), EngineError>;
    fn draw_text(
        &mut self,
        font: &str,
        text: &str,
        position: Vector2f,
        z_index: i32,
        size: f32,
        params: SpriteParams,
    ) -> Result<(), EngineError>;
    fn text_dimensions(&self, font: &str, text: &str, size: f32) -> Vector2f;
    fn create_camera(&mut self, scale: f32, target: Vector2f) -> ResourceId;
    fn set_camera(&mut self, id: ResourceId);
    fn get_camera(&self, id: ResourceId) -> Option<&dyn Camera>;
    fn get_current_camera(&self) -> &dyn Camera;
    fn get_camera_mut(&mut self, id: ResourceId) -> Option<&mut dyn Camera>;
}

pub trait Camera {
    fn get_target(&self) -> Vector2f;
    fn get_scale(&self) -> f32;
    fn set_target(&mut self, target: Vector2f);
    fn set_scale(&mut self, scale: f32);
    fn camera_to_world(&self, v: Vector2f) -> Vector2f;
}
