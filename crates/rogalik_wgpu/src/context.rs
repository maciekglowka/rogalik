use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use winit::window::Window;

use rogalik_arena::ResourceId;
use rogalik_math::vectors::Vector2f;

use crate::{
    AtlasParams, BuiltInShader, Camera2d, Color, FontParams, GraphicsDevTools, GraphicsError,
    GraphicsSetup, MaterialParams, PostProcessParams, Shader, ShaderKind, SpriteParams,
    TextureData,
};

const MAX_TIME: f32 = 3600.;

// because of WASM
// static SURFACE_STATE: Arc<Mutex<Option<SurfaceState>>> =
// Arc::new(Mutex::new(None)); static mut SURFACE_STATE:
// Option<Arc<Mutex<SurfaceState>>> = None;
static SURFACE_REFRESH: AtomicBool = AtomicBool::new(false);

struct SurfaceState {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
}

pub struct WgpuContext {
    assets: crate::assets::WgpuAssets,
    current_camera_id: Option<ResourceId<Camera2d>>,
    clear_color: wgpu::Color,
    renderer2d: crate::renderer2d::Renderer2d,
    rendering_resolution: Option<(u32, u32)>,
    surface_state: Arc<Mutex<Option<SurfaceState>>>, // because of WASM
    time: f32,
}

/// Public API
impl WgpuContext {
    /// Sets the color used to clear the rendering target before each frame.
    pub fn set_clear_color(&mut self, color: Color) {
        self.clear_color = crate::utils::color_to_wgpu(color);
        self.renderer2d.set_clear_color(self.clear_color);
    }
    /// Sets a custom rendering resolution, enabling pixel-perfect rendering and
    /// upscaling. If not set, the rendering resolution defaults to the
    /// viewport size.
    pub fn set_rendering_resolution(&mut self, w: u32, h: u32) {
        log::debug!("Setting rendering resolution at: {}x{}", w, h);
        if w == 0 || h == 0 {
            log::warn!("Invalid rendering resolution. Aborting");
            return;
        }

        self.handle_surface_refresh();

        self.rendering_resolution = Some((w, h));
        if self
            .renderer2d
            .set_rendering_resolution(&mut self.assets, w, h)
            .is_ok()
        {
            if let Ok(state) = self.surface_state.lock() {
                if let Some(state) = state.as_ref() {
                    let _ = self.renderer2d.create_upscale_pass(
                        &self.assets,
                        &state.device,
                        &state.queue,
                        &state.config.format,
                    );
                }
            }
        }
        self.resize_cameras();
    }
    /// Loads a texture from the given file path and returns its `ResourceId`.
    pub fn load_texture(&mut self, path: &str) -> ResourceId<TextureData> {
        self.assets.texture_from_path(path)
    }
    /// Loads a material with the given name and parameters.
    /// Materials define how objects are rendered, including their textures and
    /// shaders.
    pub fn create_material(
        &mut self,
        name: &str,
        params: MaterialParams,
    ) -> Result<(), GraphicsError> {
        self.assets.create_material(name, params).map(|_| ())
        // TODO if self.surface_state build bind_group
    }
    /// Loads a shader from the given file path and returns its `ResourceId`.
    pub fn load_shader(&mut self, kind: ShaderKind, path: &str) -> ResourceId<Shader> {
        // TODO if self.surface_state build pipeline
        self.assets.create_shader(kind, path)
    }
    /// Loads a font from a TTF file.
    pub fn load_font(
        &mut self,
        name: &str,
        path: &str,
        params: FontParams,
    ) -> Result<(), GraphicsError> {
        self.assets.load_font(name, path, params)
    }
    /// Loads a font from a texture atlas, allowing text rendering.
    pub fn load_font_atlas(
        &mut self,
        name: &str,
        path: &str,
        atlas: AtlasParams,
        params: FontParams,
    ) -> Result<(), GraphicsError> {
        self.assets.load_font_atlas(name, path, atlas, params)
    }
    /// Adds a post-processing effect to be applied after scene rendering.
    pub fn add_post_process(&mut self, name: &str, params: PostProcessParams) {
        self.assets.create_post_process(name, params);
    }
    /// Queues a standard sprite for drawing in the next render pass.
    pub fn draw_sprite(
        &mut self,
        material: &str,
        position: Vector2f,
        z_index: i32,
        size: Vector2f,
        params: SpriteParams,
    ) -> Result<(), GraphicsError> {
        self.renderer2d.draw_atlas_sprite(
            &self.assets,
            0,
            material,
            self.current_camera_id,
            position,
            z_index,
            size,
            params,
        )
    }
    /// Queues a specific sprite from a texture atlas for drawing.
    pub fn draw_atlas_sprite(
        &mut self,
        atlas: &str,
        index: usize,
        position: rogalik_math::vectors::Vector2f,
        z_index: i32,
        size: rogalik_math::vectors::Vector2f,
        params: SpriteParams,
    ) -> Result<(), GraphicsError> {
        self.renderer2d.draw_atlas_sprite(
            &self.assets,
            index,
            atlas,
            self.current_camera_id,
            position,
            z_index,
            size,
            params,
        )
    }
    /// Queues a custom mesh for drawing.
    pub fn draw_mesh(
        &mut self,
        material: &str,
        vertices: &[Vector2f],
        uvs: &[Vector2f],
        indices: &[u16],
        z_index: i32,
    ) -> Result<(), GraphicsError> {
        let vs = vertices
            .iter()
            .zip(uvs)
            .map(|(v, uv)| crate::structs::Vertex {
                position: [v.x, v.y, 0.],
                color: [1., 1., 1., 1.],
                tex_coords: [uv.x, uv.y],
            })
            // TODO allocation can be avoided here
            .collect::<Vec<_>>();
        self.renderer2d.draw_mesh(
            &self.assets,
            material,
            self.current_camera_id,
            &vs,
            indices,
            z_index,
        )
    }
    /// Queues singleline text for drawing using a specified font.
    ///
    /// On success text dimensions are returned.
    pub fn draw_text(
        &mut self,
        font: &str,
        text: &str,
        position: Vector2f,
        z_index: i32,
        size: f32,
        params: SpriteParams,
    ) -> Result<Vector2f, GraphicsError> {
        self.ensure_font_size(font, size)?;
        self.renderer2d.draw_text(
            &mut self.assets,
            font,
            text,
            self.current_camera_id,
            position,
            z_index,
            size,
            None,
            params,
        )
    }
    /// Queues wrapped multiline text for drawing using a specified font.
    ///
    /// On success textbox dimensions are returned.
    pub fn draw_textbox(
        &mut self,
        font: &str,
        text: &str,
        position: Vector2f,
        z_index: i32,
        size: f32,
        max_width: f32,
        params: SpriteParams,
    ) -> Result<Vector2f, GraphicsError> {
        self.ensure_font_size(font, size)?;
        self.renderer2d.draw_text(
            &mut self.assets,
            font,
            text,
            self.current_camera_id,
            position,
            z_index,
            size,
            Some(max_width),
            params,
        )
    }
    /// Sets the global ambient light color for the scene.
    pub fn set_ambient(&mut self, color: Color) {
        self.renderer2d.set_ambient(color);
    }
    /// Adds a point light source to the scene for the current frame.
    /// Lights are reset at the end of each frame.
    pub fn draw_light(
        &mut self,
        position: Vector2f,
        radius: f32,
        color: Color,
        falloff: f32,
    ) -> Result<(), GraphicsError> {
        self.renderer2d.add_light(position, radius, color, falloff)
    }
    pub fn set_postprocess_strength(
        &mut self,
        name: &str,
        value: f32,
    ) -> Result<(), GraphicsError> {
        let id = *self
            .assets
            .get_postprocess_id(name)
            .ok_or(GraphicsError::ResourceNotFound(format!(
                "post process {name}"
            )))?;
        let pass = self
            .assets
            .get_postprocess_mut(id)
            .ok_or(GraphicsError::ResourceNotFound(format!(
                "post process {name}"
            )))?;

        pass.set_strength(value);

        if let Ok(state) = self.surface_state.lock() {
            if let Some(state) = state.as_ref() {
                pass.write_buffer(&state.queue)?;
            };
        }
        Ok(())
    }
    /// Calculates the dimensions (width and height) a singleline text would
    /// occupy when rendered with a specific font and size.
    pub fn text_dimensions(&mut self, font: &str, text: &str, size: f32) -> Vector2f {
        if self.ensure_font_size(font, size).is_err() {
            return Vector2f::ZERO;
        }

        self.renderer2d
            .get_text_dimensions(&mut self.assets, font, text, size, None)
            .unwrap_or(Vector2f::ZERO)
    }
    /// Calculates the dimensions (width and height) a multiline wrapped textbox
    /// would occupy when rendered with a specific font and size.
    pub fn textbox_dimensions(
        &mut self,
        font: &str,
        text: &str,
        size: f32,
        max_width: f32,
    ) -> Vector2f {
        if self.ensure_font_size(font, size).is_err() {
            return Vector2f::ZERO;
        }

        self.renderer2d
            .get_text_dimensions(&mut self.assets, font, text, size, Some(max_width))
            .unwrap_or(Vector2f::ZERO)
    }
    /// Creates a new 2D camera with a specified scale and target position.
    pub fn create_camera(&mut self, scale: f32, target: Vector2f) -> ResourceId<Camera2d> {
        let (vw, vh, rw, rh) = self.get_current_resolutions();
        let id = self
            .assets
            .create_camera(vw as f32, vh as f32, rw as f32, rh as f32, scale, target);

        if self.current_camera_id.is_none() {
            self.current_camera_id = Some(id)
        }

        id
    }
    /// Sets the currently active camera by its `ResourceId`.
    /// All subsequent draw calls will use this camera's view.
    pub fn set_camera(&mut self, id: &ResourceId<Camera2d>) {
        self.current_camera_id = Some(*id);
    }
    /// Retrieves an immutable reference to the currently active camera.
    pub fn get_current_camera(&self) -> &Camera2d {
        self.assets.get_camera(self.current_camera_id).unwrap()
    }
    /// Retrieves a mutable reference to the currently active camera.
    pub fn get_current_camera_mut(&mut self) -> &mut Camera2d {
        self.assets.get_camera_mut(self.current_camera_id).unwrap()
    }
    /// Retrieves an immutable reference to a camera by its `ResourceId`.
    pub fn get_camera(&self, id: &ResourceId<Camera2d>) -> Option<&Camera2d> {
        self.assets.get_camera(*id)
    }
    /// Retrieves a mutable reference to a camera by its `ResourceId`.
    pub fn get_camera_mut(&mut self, id: &ResourceId<Camera2d>) -> Option<&mut Camera2d> {
        self.assets.get_camera_mut(*id)
    }
    /// Retrieves the `ResourceId` of a built-in shader.
    /// Returns `None` if the shader is not found.
    pub fn get_builtin_shader(&self, shader: BuiltInShader) -> Option<ResourceId<Shader>> {
        self.assets.builtin_shaders.get(&shader).copied()
    }
}

/// DevTools hidden by a trait.
impl GraphicsDevTools for WgpuContext {
    fn toggle_recording(&mut self) {
        #[cfg(all(dev_tools, feature = "capture"))]
        self.renderer2d.recorder.toggle_recording();
    }
    fn request_screenshot(&mut self) {
        #[cfg(all(dev_tools, feature = "capture"))]
        self.renderer2d.recorder.request_screenshot();
    }
    fn take_screenshot(&mut self) -> Option<Vec<u8>> {
        cfg_select! {
            all(dev_tools, feature = "capture") => self.renderer2d.recorder.take_screenshot(),
            _ => None,
        }
    }
}

/// Wgpu internals.
impl WgpuContext {
    pub fn new(asset_store: Arc<Mutex<rogalik_assets::AssetStore>>) -> Self {
        Self {
            assets: crate::assets::WgpuAssets::new(asset_store),
            current_camera_id: None,
            clear_color: wgpu::Color::BLACK,
            renderer2d: crate::renderer2d::Renderer2d::new(),
            rendering_resolution: None,
            // This is a WASM only `allow` - to keep code the same across targets.
            #[allow(clippy::arc_with_non_send_sync)]
            surface_state: Arc::new(Mutex::new(None)),
            time: 0.,
        }
    }
    /// Returns (vw, vh, rw, rh)
    fn get_current_resolutions(&self) -> (u32, u32, u32, u32) {
        let (w, h) = match self.surface_state.lock() {
            Ok(s) => match s.as_ref() {
                Some(s) => (s.config.width, s.config.height),
                _ => (0, 0),
            },
            _ => (0, 0),
        };
        let (rw, rh) = match self.rendering_resolution {
            Some((rw, rh)) => (rw, rh),
            None => (w, h),
        };
        (w, h, rw, rh)
    }
    fn resize_renderer(&mut self) {
        if let Ok(state) = self.surface_state.lock() {
            if let Some(state) = state.as_ref() {
                self.renderer2d
                    .resize(state.config.width, state.config.height);
            }
        }
    }
    fn resize_cameras(&mut self) {
        let (vw, vh, rw, rh) = self.get_current_resolutions();
        for camera in self.assets.cameras.iter_mut() {
            camera.resize_viewport(vw as f32, vh as f32, rw as f32, rh as f32);
        }
    }
    fn post_surface_state(&mut self) {
        log::debug!("Executing post surface state creation actions");
        if let Ok(state) = self.surface_state.lock() {
            if let Some(state) = state.as_ref() {
                let w = state.config.width;
                let h = state.config.height;
                log::debug!("State config dim: {}, {}", w, h);

                let _ = self.assets.create_wgpu_data(
                    w,
                    h,
                    &state.device,
                    &state.queue,
                    &state.config.format,
                );
                log::debug!("Asset data created");
                let _ = self.renderer2d.create_wgpu_data(
                    &self.assets,
                    w,
                    h,
                    &state.device,
                    &state.queue,
                    &state.config.format,
                );
                log::debug!("Renderer2d data created");
            }
            SURFACE_REFRESH.store(false, Ordering::Relaxed);
        }
        self.resize_cameras();
        self.resize_renderer();
    }
    fn handle_surface_refresh(&mut self) {
        if SURFACE_REFRESH.load(Ordering::Relaxed) {
            self.post_surface_state();
        }
    }
    fn ensure_font_size(&mut self, font: &str, size: f32) -> Result<(), GraphicsError> {
        match self.assets.has_font_size(font, size) {
            Ok(true) => Ok(()),
            Ok(false) => {
                let state = self.surface_state.lock().unwrap();
                let state = state.as_ref().ok_or(GraphicsError::NotReady)?;

                self.assets
                    .create_font_size(font, size, &state.device, &state.queue)
            }
            Err(e) => Err(e),
        }
    }
}

// Public setup methods, hidden by a trait.
impl GraphicsSetup for WgpuContext {
    fn has_context(&self) -> bool {
        match self.surface_state.lock() {
            Ok(s) => s.is_some(),
            _ => false,
        }
    }
    fn create_context(&mut self, window: Arc<Window>) {
        #[cfg(not(target_arch = "wasm32"))]
        pollster::block_on(create_surface_state(self.surface_state.clone(), window));
        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(create_surface_state(self.surface_state.clone(), window));
    }
    fn update_time(&mut self, delta: f32) {
        self.time += delta;
        self.time %= MAX_TIME;
    }
    fn update_assets(&mut self) {
        if let Ok(state) = self.surface_state.lock() {
            if let Some(state) = state.as_ref() {
                let _ =
                    self.assets
                        .update_assets(&state.device, &state.queue, &state.config.format);
            }
        }
    }
    fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            if let Ok(mut state) = self.surface_state.lock() {
                if let Some(state) = state.as_mut() {
                    state.config.width = width;
                    state.config.height = height;
                    state.surface.configure(&state.device, &state.config);
                    let _ = self.renderer2d.create_wgpu_data(
                        &self.assets,
                        width,
                        height,
                        &state.device,
                        &state.queue,
                        &state.config.format,
                    );
                    let _ = self.assets.update_postprocess_wgpu_data(
                        width,
                        height,
                        &state.device,
                        &state.queue,
                        &state.config.format,
                    );
                }
            }
            self.resize_cameras();
            self.resize_renderer();
        }
    }
    fn render(&mut self) {
        self.handle_surface_refresh();
        if let Ok(state) = self.surface_state.lock() {
            if let Some(state) = state.as_ref() {
                let _ = self.renderer2d.render(
                    &self.assets,
                    self.time,
                    &state.surface,
                    &state.device,
                    &state.queue,
                );
            }
        }
    }
}

async fn create_surface_state(
    surface_state: Arc<Mutex<Option<SurfaceState>>>,
    window: Arc<Window>,
) {
    log::debug!("Creating WGPU instance");
    let size = (window.inner_size().width, window.inner_size().height);

    if size.0 == 0 || size.1 == 0 {
        return;
    }

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: get_backends(),
        ..Default::default()
    });
    log::debug!("Creating WGPU surface");
    let surface = instance.create_surface(window).unwrap();
    log::debug!("Creating WGPU adapter");

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })
        .await
        .expect("Request for adapter failed!");

    log::debug!("Creating WGPU device");

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits: get_limits(),
                label: None,
                memory_hints: Default::default(),
            },
            None,
        )
        .await
        .expect("Could not create the device!");

    log::debug!("Config WGPU surface");
    let surface_caps = surface.get_capabilities(&adapter);
    log::debug!("WGPU surface capabilities: {:?}", surface_caps);
    let surface_format = surface_caps
        .formats
        .iter()
        .copied()
        .find(|f| f.is_srgb())
        .unwrap_or(surface_caps.formats[0]);
    log::debug!("WGPU surface format: {:?}", surface_format);

    let present_mode = if surface_caps
        .present_modes
        .contains(&wgpu::PresentMode::Fifo)
        || surface_caps
            .present_modes
            .contains(&wgpu::PresentMode::FifoRelaxed)
    {
        wgpu::PresentMode::AutoVsync
    } else {
        surface_caps.present_modes[0]
    };
    log::debug!("WGPU present mode: {:?}", present_mode);

    // COPY_SRC needed only for recordings
    #[cfg(dev_tools)]
    let usage = wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC;
    #[cfg(not(dev_tools))]
    let usage = wgpu::TextureUsages::RENDER_ATTACHMENT;

    let config = wgpu::SurfaceConfiguration {
        usage,
        format: surface_format,
        width: size.0,
        height: size.1,
        present_mode,
        alpha_mode: surface_caps.alpha_modes[0],
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    };
    log::debug!("WGPU surface config: {:?}", config);
    surface.configure(&device, &config);
    log::debug!("WGPU surface configured");

    if let Ok(mut state) = surface_state.lock() {
        *state = Some(SurfaceState {
            surface,
            device,
            queue,
            config,
        });
    };
    log::debug!("Surface refresh set to true");
    SURFACE_REFRESH.store(true, Ordering::Relaxed);
}

#[cfg(not(target_arch = "wasm32"))]
fn get_backends() -> wgpu::Backends {
    wgpu::Backends::all()
}
#[cfg(target_arch = "wasm32")]
fn get_backends() -> wgpu::Backends {
    wgpu::Backends::GL
}

#[cfg(not(target_arch = "wasm32"))]
fn get_limits() -> wgpu::Limits {
    wgpu::Limits::default()
}
#[cfg(target_arch = "wasm32")]
fn get_limits() -> wgpu::Limits {
    let mut limits = wgpu::Limits::downlevel_webgl2_defaults();
    limits.max_color_attachments = 4;
    limits.max_texture_dimension_1d = 4096;
    limits.max_texture_dimension_2d = 4096;
    limits
}
