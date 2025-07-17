use rogalik_math::vectors::Vector2f;
use std::sync::Arc;
use winit::window::Window;

use crate::structs::{BuiltInShader, Color, EngineError, ResourceId, ShaderKind, SpriteParams};

/// Defines the interface for interacting with the engine's graphics rendering
/// capabilities.
pub trait GraphicsContext {
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
    /// Sets the color used to clear the rendering target before each frame.
    /// `color`: The `Color` to set as the clear color.
    fn set_clear_color(&mut self, color: Color);
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
    /// Sets a custom rendering resolution, enabling pixel-perfect rendering and
    /// upscaling. If not set, the rendering resolution defaults to the
    /// viewport size. `w`: The desired width for internal rendering.
    /// `h`: The desired height for internal rendering.
    fn set_rendering_resolution(&mut self, w: u32, h: u32);
    /// Loads a texture from the given file path and returns its `ResourceId`.
    /// `path`: The file path to the texture image.
    fn load_texture(&mut self, path: &str) -> ResourceId;
    /// Loads a material with the given name and parameters.
    /// Materials define how objects are rendered, including their textures and
    /// shaders. `name`: A unique identifier for the material.
    /// `params`: Parameters defining the material's properties (e.g., textures,
    /// atlas, filtering).
    fn load_material(&mut self, name: &str, params: crate::MaterialParams);
    /// Loads a shader from the given file path and returns its `ResourceId`.
    /// `kind`: The type of shader (e.g., `Sprite`, `PostProcess`).
    /// `path`: The file path to the shader source code (WGSL).
    fn load_shader(&mut self, kind: ShaderKind, path: &str) -> ResourceId;
    /// Loads a font from a texture atlas, allowing text rendering.
    /// `name`: A unique identifier for the font.
    /// `path`: The file path to the font texture atlas.
    /// `rows`: The number of rows in the font atlas.
    /// `cols`: The number of columns in the font atlas.
    /// `padding`: Optional padding around each character in the atlas (x, y).
    /// `shader`: Optional `ResourceId` of a custom shader to use for text
    /// rendering.
    fn load_font(
        &mut self,
        name: &str,
        path: &str,
        rows: usize,
        cols: usize,
        padding: Option<(f32, f32)>,
        shader: Option<ResourceId>,
    );
    /// Adds a post-processing effect to be applied after the main scene
    /// rendering. `name`: A unique identifier for the post-process effect.
    /// `params`: Parameters defining the post-process effect (e.g., shader,
    /// texture).
    fn add_post_process(&mut self, name: &str, params: crate::PostProcessParams);
    /// Queues a standard sprite for drawing in the next render pass.
    /// This method uses the material's default (1x1) atlas or the first sprite
    /// in a defined atlas. `material`: The name of the material to use for
    /// rendering the sprite. `position`: The world position of the sprite
    /// (bottom-left corner). `z_index`: The Z-order for rendering (higher
    /// values are rendered on top). `size`: The width and height of the
    /// sprite in world units. `params`: Additional sprite parameters (e.g.,
    /// color, flip, rotation, slicing).
    fn draw_sprite(
        &mut self,
        material: &str,
        position: Vector2f,
        z_index: i32,
        size: Vector2f,
        params: SpriteParams,
    ) -> Result<(), EngineError>;
    /// Queues a specific sprite from a texture atlas for drawing.
    /// `material`: The name of the material associated with the atlas.
    /// `index`: The index of the sprite within the atlas.
    /// `position`: The world position of the sprite (bottom-left corner).
    /// `z_index`: The Z-order for rendering (higher values are rendered on
    /// top). `size`: The width and height of the sprite in world units.
    /// `params`: Additional sprite parameters (e.g., color, flip, rotation,
    /// slicing).
    fn draw_atlas_sprite(
        &mut self,
        material: &str,
        index: usize,
        position: Vector2f,
        z_index: i32,
        size: Vector2f,
        params: SpriteParams,
    ) -> Result<(), EngineError>;
    /// Queues text for drawing using a specified font.
    /// `font`: The name of the loaded font (material) to use.
    /// `text`: The string to render.
    /// `position`: The world position of the first character (bottom-left
    /// corner). `z_index`: The Z-order for rendering (higher values are
    /// rendered on top). `size`: The desired height of the text in world
    /// units. `params`: Additional sprite parameters applied to each
    /// character (e.g., color, flip).
    fn draw_text(
        &mut self,
        font: &str,
        text: &str,
        position: Vector2f,
        z_index: i32,
        size: f32,
        params: SpriteParams,
    ) -> Result<(), EngineError>;
    /// Queues a custom mesh for drawing.
    /// `material`: The name of the material to use for rendering the mesh.
    /// `vertices`: A slice of `Vector2f` representing the positions of the mesh
    /// vertices. `uvs`: A slice of `Vector2f` representing the UV
    /// coordinates for each vertex. `indices`: A slice of `u16` defining
    /// the triangles of the mesh. `z_index`: The Z-order for rendering
    /// (higher values are rendered on top).
    fn draw_mesh(
        &mut self,
        material: &str,
        vertices: &[Vector2f],
        uvs: &[Vector2f],
        indices: &[u16],
        z_index: i32,
    ) -> Result<(), EngineError>;
    /// Adds a point light source to the scene for the current frame.
    /// Lights are reset at the end of each frame.
    /// `position`: The world position of the light source.
    /// `radius`: The size of the light.
    /// `color`: The color of the light. (alpha value is discarded)
    /// `falloff`: The hardness of the light.
    fn add_light(
        &mut self,
        position: Vector2f,
        radius: f32,
        color: Color,
        falloff: f32,
    ) -> Result<(), EngineError>;
    /// Sets the global ambient light color for the scene.
    /// `color`: The `Color` representing the ambient light.
    fn set_ambient(&mut self, color: Color);
    /// Sets the strength (intensity) of a previously added post-process effect.
    /// `name`: The name of the post-process effect.
    /// `value`: The new strength value (typically between 0.0 and 1.0).
    /// Note: if the strength is set to 0. (or almost 0. to tackle float
    /// imprecission) the pass is not processed at all in order to save
    /// hardware resources.
    fn set_postprocess_strength(&mut self, name: &str, value: f32) -> Result<(), EngineError>;
    /// Calculates the dimensions (width and height) a given text string would
    /// occupy when rendered with a specific font and size. `font`: The name
    /// of the font (material) to use for calculation. `text`: The string
    /// whose dimensions are to be measured. `size`: The desired height of
    /// the text. Returns a `Vector2f` representing the width and height.
    fn text_dimensions(&self, font: &str, text: &str, size: f32) -> Vector2f;
    /// Creates a new 2D camera with a specified scale and target position.
    /// Returns a `ResourceId` for the newly created camera.
    /// `scale`: The zoom level of the camera (e.g., 1.0 is no zoom).
    /// `target`: The initial world position that the camera will center on.
    fn create_camera(&mut self, scale: f32, target: Vector2f) -> ResourceId;
    /// Sets the currently active camera by its `ResourceId`.
    /// All subsequent draw calls will use this camera's view.
    /// `id`: The `ResourceId` of the camera to activate.
    fn set_camera(&mut self, id: ResourceId);
    /// Retrieves an immutable reference to a camera by its `ResourceId`.
    /// Returns `None` if the camera does not exist.
    /// `id`: The `ResourceId` of the camera to retrieve.
    fn get_camera(&self, id: ResourceId) -> Option<&dyn Camera>;
    /// Retrieves an immutable reference to the currently active camera.
    fn get_current_camera(&self) -> &dyn Camera;
    /// Retrieves a mutable reference to the currently active camera.
    fn get_current_camera_mut(&mut self) -> &mut dyn Camera;
    /// Retrieves a mutable reference to a camera by its `ResourceId`.
    /// Returns `None` if the camera does not exist.
    /// `id`: The `ResourceId` of the camera to retrieve.
    fn get_camera_mut(&mut self, id: ResourceId) -> Option<&mut dyn Camera>;
    /// Retrieves the `ResourceId` of a built-in shader.
    /// Returns `None` if the shader is not found.
    /// `shader`: The `BuiltInShader` enum variant identifying the desired
    /// shader.
    fn get_builtin_shader(&self, shader: BuiltInShader) -> Option<ResourceId>;
}

/// Provides an interface for audio operations within the engine.
pub trait AudioContext {
    /// Creates and initializes the audio context and device.
    fn create_context(&mut self);
    /// Checks if the audio context has been successfully created.
    fn has_context(&self) -> bool;
    /// Updates audio assets, checking for any changes or reloads.
    fn update_assets(&mut self);
    /// Sets the global master volume for all audio playback.
    /// `volume`: A value between 0.0 and 1.0.
    fn set_master_volume(&mut self, volume: f32);
    /// Loads an audio source from a given path and associates it with a `name`.
    fn load_source(&mut self, name: &str, path: &str) -> Result<(), EngineError>;
    /// Starts playing the audio source identified by `name`.
    /// If `looped` is true, the audio will loop indifinitely.
    fn play(&mut self, name: &str, looped: bool) -> Result<(), EngineError>;
    /// Stops the audio source identified by `name`.
    fn stop(&mut self, name: &str) -> Result<(), EngineError>;
    /// Resumes playing a previously stopped audio source identified by `name`.
    fn resume(&mut self, name: &str) -> Result<(), EngineError>;
    /// Sets the individual volume for the audio source identified by `name`.
    /// `volume`: A value between 0.0 and 1.0.
    fn set_volume(&mut self, name: &str, volume: f32) -> Result<(), EngineError>;
    /// Sets the pan (left-right balance) for the audio source identified by
    /// `name`. `pan` should be between -1.0 (full left) and 1.0 (full
    /// right).
    fn set_pan(&mut self, name: &str, pan: f32) -> Result<(), EngineError>;
}

/// Defines the interface for a camera in the engine, providing methods to
/// manage its position, zoom level, and transformations between camera and
/// world coordinates.
pub trait Camera {
    /// Returns the current target position of the camera in world coordinates.
    fn get_target(&self) -> Vector2f;
    /// Returns the current scale (zoom level) of the camera.
    fn get_scale(&self) -> f32;
    /// Sets the camera's target position in world coordinates.
    /// `target`: The new target position.
    fn set_target(&mut self, target: Vector2f);
    /// Sets the camera's scale (zoom level).
    /// `scale`: The new scale value.
    fn set_scale(&mut self, scale: f32);
    /// Converts a point from camera coordinates to world coordinates.
    /// `v`: The vector in camera coordinates.
    fn camera_to_world(&self, v: Vector2f) -> Vector2f;
    /// Returns the current rectangular bounds of the camera's view in world
    /// coordinates. The return value is a tuple `(min_vector, max_vector)`
    /// representing the bottom-left and top-right corners of the camera's
    /// view.
    fn get_bounds(&self) -> (Vector2f, Vector2f);
}
