use std::sync::{Arc, Mutex};
use winit::window::Window;

use rogalik_common::{EngineError, GraphicsContext, ResourceId, SpriteParams};
use rogalik_math::vectors::Vector2f;

pub use assets::shader::BuiltInShader;

mod assets;
mod renderer2d;
mod structs;
mod utils;

const MAX_TIME: f32 = 3600.;

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
    surface_state: Option<SurfaceState>,
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
            surface_state: None,
            time: 0.,
        }
    }
    /// Returns (vw, vh, rw, rh)
    fn get_current_resolutions(&self) -> (u32, u32, u32, u32) {
        let (w, h) = match &self.surface_state {
            Some(s) => (s.config.width, s.config.height),
            None => (0, 0),
        };
        let (rw, rh) = match self.rendering_resolution {
            Some((rw, rh)) => (rw, rh),
            None => (w, h),
        };
        (w, h, rw, rh)
    }
    fn resize_renderer(&mut self) {
        if let Some(state) = &self.surface_state {
            self.renderer2d
                .resize(state.config.width, state.config.height);
        }
    }
    fn resize_cameras(&mut self) {
        let (vw, vh, rw, rh) = self.get_current_resolutions();
        for camera in self.assets.iter_cameras_mut() {
            camera.resize_viewport(vw as f32, vh as f32, rw as f32, rh as f32);
        }
    }
}
impl GraphicsContext for WgpuContext {
    fn has_context(&self) -> bool {
        self.surface_state.is_some()
    }
    fn create_context(&mut self, window: Arc<Window>) {
        self.surface_state = create_surface_state(window);
        if let Some(state) = &self.surface_state {
            let w = state.config.width;
            let h = state.config.height;
            log::debug!("State config dim: {}, {}", w, h);

            let _ = self
                .assets
                .create_wgpu_data(&state.device, &state.queue, &state.config.format);
            log::debug!("Asset data created");
            let _ = self.renderer2d.create_wgpu_data(
                &self.assets,
                w,
                h,
                &state.device,
                state.config.format,
            );
            log::debug!("Renderer2d data created");
            self.resize_cameras();
            self.resize_renderer();
        }
    }
    fn update_time(&mut self, delta: f32) {
        self.time += delta;
        self.time = self.time % MAX_TIME;
    }
    fn update_assets(&mut self) {
        if let Some(state) = &self.surface_state {
            let _ = self
                .assets
                .update_assets(&state.device, &state.queue, &state.config.format);
        }
    }
    fn set_clear_color(&mut self, color: rogalik_common::Color) {
        self.clear_color = utils::color_to_wgpu(color);
        self.renderer2d.set_clear_color(self.clear_color);
    }
    fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            if let Some(state) = &mut self.surface_state {
                state.config.width = width;
                state.config.height = height;
                state.surface.configure(&state.device, &state.config);
                let _ = self.renderer2d.create_wgpu_data(
                    &self.assets,
                    width,
                    height,
                    &state.device,
                    state.config.format,
                );
            }
            self.resize_cameras();
            self.resize_renderer();
        }
    }
    fn render(&mut self) {
        if let Some(state) = &mut self.surface_state {
            let _ = self.renderer2d.render(
                &self.assets,
                self.time,
                &state.surface,
                &state.device,
                &state.queue,
            );
        }
    }
    fn set_rendering_resolution(&mut self, w: u32, h: u32) {
        log::debug!("Setting rendering resolution at: {}x{}", w, h);
        self.rendering_resolution = Some((w, h));
        if self
            .renderer2d
            .set_rendering_resolution(&self.assets, w, h)
            .is_ok()
        {
            if let Some(state) = &self.surface_state {
                let _ = self.renderer2d.create_upscale_pass(
                    &self.assets,
                    &state.device,
                    state.config.format,
                );
            }
        }
        self.resize_cameras();
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
    ) {
        self.assets.load_font(name, path, rows, cols, padding);
    }
    fn add_post_process(
        &mut self,
        shader_id: ResourceId,
        filtering: rogalik_common::TextureFiltering,
    ) {
        self.renderer2d.add_post_process(shader_id, filtering);
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
    fn add_light(
        &mut self,
        strength: f32,
        color: rogalik_common::Color,
        position: Vector2f,
    ) -> Result<(), EngineError> {
        self.renderer2d.add_light(strength, color, position)
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
}

fn create_surface_state(window: Arc<Window>) -> Option<SurfaceState> {
    log::debug!("Creating WGPU instance");
    #[cfg(not(target_arch = "wasm32"))]
    let size = (window.inner_size().width, window.inner_size().height);
    #[cfg(target_arch = "wasm32")]
    let size = (
        (window.inner_size().width as f64 / window.scale_factor()) as u32,
        (window.inner_size().height as f64 / window.scale_factor()) as u32,
    );
    log::debug!("Size: {:?}", size);

    if size.0 == 0 || size.1 == 0 {
        return None;
    }

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });
    log::debug!("Creating WGPU surface");
    let surface = instance.create_surface(window).unwrap();
    log::debug!("Creating WGPU adapter");
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::default(),
        compatible_surface: Some(&surface),
        force_fallback_adapter: false,
    }))
    .expect("Request for adapter failed!");
    log::debug!("Creating WGPU device");
    let (device, queue) = pollster::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            required_features: wgpu::Features::empty(),
            required_limits: if cfg!(target_arch = "wasm32") {
                wgpu::Limits::downlevel_webgl2_defaults()
            } else {
                wgpu::Limits::default()
            },
            label: None,
            memory_hints: Default::default(),
        },
        None,
    ))
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

    Some(SurfaceState {
        surface,
        device,
        queue,
        config,
    })
}
