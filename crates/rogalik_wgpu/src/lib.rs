use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use winit::window::Window;

use rogalik_common::{BuiltInShader, EngineError, GraphicsContext, ResourceId, SpriteParams};
use rogalik_math::vectors::Vector2f;

mod assets;
mod renderer2d;
mod structs;
mod utils;

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
    assets: assets::WgpuAssets,
    current_camera_id: ResourceId,
    clear_color: wgpu::Color,
    renderer2d: renderer2d::Renderer2d,
    rendering_resolution: Option<(u32, u32)>,
    surface_state: Arc<Mutex<Option<SurfaceState>>>, // because of WASM
    time: f32,
}
impl WgpuContext {
    pub fn new(asset_store: Arc<Mutex<rogalik_assets::AssetStore>>) -> Self {
        Self {
            assets: assets::WgpuAssets::new(asset_store),
            current_camera_id: ResourceId::default(),
            clear_color: wgpu::Color::BLACK,
            renderer2d: renderer2d::Renderer2d::new(),
            rendering_resolution: None,
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
}
impl GraphicsContext for WgpuContext {
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
        self.time = self.time % MAX_TIME;
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
    fn set_clear_color(&mut self, color: rogalik_common::Color) {
        self.clear_color = utils::color_to_wgpu(color);
        self.renderer2d.set_clear_color(self.clear_color);
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
        if SURFACE_REFRESH.load(Ordering::Relaxed) {
            self.post_surface_state();
        }
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
    fn set_rendering_resolution(&mut self, w: u32, h: u32) {
        log::debug!("Setting rendering resolution at: {}x{}", w, h);
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
    fn load_texture(&mut self, path: &str) -> ResourceId {
        self.assets.texture_from_path(path)
    }
    fn load_material(&mut self, name: &str, params: rogalik_common::MaterialParams) {
        self.assets.create_material(name, params);
        // TODO if self.surface_state build bind_group
    }
    fn load_shader(&mut self, kind: rogalik_common::ShaderKind, path: &str) -> ResourceId {
        // TODO if self.surface_state build pipeline
        self.assets.create_shader(kind, path)
    }
    fn load_font(
        &mut self,
        name: &str,
        path: &str,
        rows: usize,
        cols: usize,
        padding: Option<(f32, f32)>,
        shader: Option<ResourceId>,
    ) {
        self.assets
            .load_font(name, path, rows, cols, padding, shader);
    }
    fn add_post_process(&mut self, name: &str, params: rogalik_common::PostProcessParams) {
        self.assets.create_post_process(name, params);
    }
    fn draw_sprite(
        &mut self,
        material: &str,
        position: Vector2f,
        z_index: i32,
        size: Vector2f,
        params: SpriteParams,
    ) -> Result<(), EngineError> {
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
    fn draw_atlas_sprite(
        &mut self,
        atlas: &str,
        index: usize,
        position: rogalik_math::vectors::Vector2f,
        z_index: i32,
        size: rogalik_math::vectors::Vector2f,
        params: SpriteParams,
    ) -> Result<(), EngineError> {
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
    fn draw_text(
        &mut self,
        font: &str,
        text: &str,
        position: Vector2f,
        z_index: i32,
        size: f32,
        params: SpriteParams,
    ) -> Result<(), EngineError> {
        self.renderer2d.draw_text(
            &self.assets,
            font,
            text,
            self.current_camera_id,
            position,
            z_index,
            size,
            params,
        )
    }
    fn draw_mesh(
        &mut self,
        material: &str,
        vertices: &[Vector2f],
        uvs: &[Vector2f],
        indices: &[u16],
        z_index: i32,
    ) -> Result<(), EngineError> {
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
    fn set_ambient(&mut self, color: rogalik_common::Color) {
        self.renderer2d.set_ambient(color);
    }
    fn set_postprocess_strength(&mut self, name: &str, value: f32) -> Result<(), EngineError> {
        let id = *self
            .assets
            .get_postprocess_id(name)
            .ok_or(EngineError::ResourceNotFound)?;
        let pass = self
            .assets
            .get_postprocess_mut(id)
            .ok_or(EngineError::ResourceNotFound)?;

        pass.set_strength(value);

        if let Ok(state) = self.surface_state.lock() {
            if let Some(state) = state.as_ref() {
                pass.write_buffer(&state.queue)?;
            };
        }
        Ok(())
    }
    fn add_light(
        &mut self,
        position: Vector2f,
        radius: f32,
        color: rogalik_common::Color,
        falloff: f32,
    ) -> Result<(), EngineError> {
        self.renderer2d.add_light(position, radius, color, falloff)
    }
    fn text_dimensions(&self, font: &str, text: &str, size: f32) -> Vector2f {
        self.assets
            .get_text_dimensions(font, text, size)
            .unwrap_or(Vector2f::ZERO)
    }
    fn create_camera(&mut self, scale: f32, target: Vector2f) -> ResourceId {
        let (vw, vh, rw, rh) = self.get_current_resolutions();
        self.assets
            .create_camera(vw as f32, vh as f32, rw as f32, rh as f32, scale, target)
    }
    fn set_camera(&mut self, id: ResourceId) {
        self.current_camera_id = id;
    }
    fn get_current_camera(&self) -> &dyn rogalik_common::Camera {
        self.assets.get_camera(self.current_camera_id).unwrap()
    }
    fn get_current_camera_mut(&mut self) -> &mut dyn rogalik_common::Camera {
        self.assets.get_camera_mut(self.current_camera_id).unwrap()
    }
    fn get_camera(&self, id: ResourceId) -> Option<&dyn rogalik_common::Camera> {
        Some(self.assets.get_camera(id)?)
    }
    fn get_camera_mut(&mut self, id: ResourceId) -> Option<&mut dyn rogalik_common::Camera> {
        Some(self.assets.get_camera_mut(id)?)
    }
    fn get_builtin_shader(&self, shader: BuiltInShader) -> Option<ResourceId> {
        self.assets.builtin_shaders.get(&shader).copied()
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
        // backends: wgpu::Backends::all(),
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

    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
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
    limits
}
