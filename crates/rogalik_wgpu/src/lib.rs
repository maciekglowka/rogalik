use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use winit::window::Window;

use rogalik_arena::ResourceId;
use rogalik_common::{
    structs::{CameraId, ShaderId, TextureId},
    traits::{GraphicsDevTools, GraphicsSetup},
    AtlasParams, BuiltInShader, EngineError, FontParams, GraphicsContext, SpriteParams,
};
use rogalik_math::vectors::Vector2f;

mod assets;
mod context;
mod renderer2d;
mod structs;
mod tools;
mod utils;

pub use context::WgpuContext;

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
