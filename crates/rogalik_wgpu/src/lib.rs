use std::sync::Arc;
use winit::window::Window;

use rogalik_assets::AssetError;

mod assets;
mod context;
mod data;
mod renderer2d;
mod structs;
mod tools;
mod utils;

pub use assets::atlas::{AtlasParams, AtlasPosition, SpriteParams};
pub use assets::camera::Camera2d;
pub use assets::font::FontParams;
pub use assets::material::{Material, MaterialParams};
pub use assets::postprocess::PostProcessParams;
pub use assets::shader::{BuiltInShader, Shader, ShaderKind};
pub use assets::texture::{TextureData, TextureFiltering, TextureRepeat};
pub use context::WgpuContext;
pub use structs::Color;

pub trait GraphicsSetup {
    /// Creates and initializes the graphics context and surface.
    /// This method should be called once the application window is available.
    /// (called by the engine internally)
    fn create_context(&mut self, window: Arc<Window>);
    /// Checks if the graphics context has been successfully created and
    /// initialized.
    fn has_context(&self) -> bool;
    /// Updates the internal time counter, typically used for
    /// shader uniforms. `delta`: The time elapsed since the last frame, in
    /// seconds.
    /// (called by the engine internally)
    fn update_time(&mut self, delta: f32);
    /// Updates and reloads any assets (e.g., textures, shaders) that have
    /// changed.
    /// (called by the engine internally)
    fn update_assets(&mut self);
    /// Resizes the rendering surface and adjusts internal rendering
    /// resolutions. This should be called when the window size changes.
    /// `w`: The new width of the rendering surface.
    /// `h`: The new height of the rendering surface.
    /// (called by the engine internally)
    fn resize(&mut self, w: u32, h: u32);
    /// Renders the current frame, processing all queued draw calls and
    /// post-processing effects.
    /// (called by the engine internally)
    fn render(&mut self);
}

pub trait GraphicsDevTools {
    fn toggle_recording(&mut self);
    fn request_screenshot(&mut self);
    fn take_screenshot(&mut self) -> Option<Vec<u8>>;
}

#[derive(Debug)]
pub enum GraphicsError {
    AssetError(String),
    FontError(String),
    InternalError,
    NotReady,
    MaterialError(String),
    ResourceNotFound(String),
    ShaderError(String),
    TextureError(String),
}
impl std::fmt::Display for GraphicsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AssetError(inner) => {
                write!(f, "audio asset error: {inner}")
            }
            Self::FontError(inner) => {
                write!(f, "font error: {inner}")
            }
            Self::InternalError => {
                write!(f, "internal error")
            }
            Self::MaterialError(inner) => {
                write!(f, "material error: {inner}")
            }
            Self::NotReady => {
                write!(f, "graphics not ready")
            }
            Self::ResourceNotFound(inner) => {
                write!(f, "resource not found: {inner}")
            }
            Self::ShaderError(inner) => {
                write!(f, "shader error: {inner}")
            }
            Self::TextureError(inner) => {
                write!(f, "texture error: {inner}")
            }
        }
    }
}
impl std::error::Error for GraphicsError {}

impl From<AssetError> for GraphicsError {
    fn from(value: AssetError) -> Self {
        Self::AssetError(value.to_string())
    }
}
