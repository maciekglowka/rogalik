use std::sync::{Arc, Mutex};
use winit::window::Window;

use rogalik_common::{EngineError, GraphicsContext, ResourceId, SpriteParams};
use rogalik_math::vectors::Vector2f;

mod assets;
mod renderer2d;
mod structs;

struct SurfaceState {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    renderer2d: renderer2d::Renderer2d,
}

pub struct WgpuContext {
    assets: assets::WgpuAssets,
    current_camera_id: ResourceId,
    clear_color: wgpu::Color,
    surface_state: Option<SurfaceState>,
}
impl WgpuContext {
    pub fn new(asset_store: Arc<Mutex<rogalik_assets::AssetStore>>) -> Self {
        Self {
            assets: assets::WgpuAssets::new(asset_store),
            current_camera_id: ResourceId::default(),
            clear_color: wgpu::Color::BLACK,
            surface_state: None,
        }
    }
}
impl GraphicsContext for WgpuContext {
    fn has_context(&self) -> bool {
        self.surface_state.is_some()
    }
    fn create_context(&mut self, window: &Window) {
        self.surface_state = create_surface_state(window, &mut self.assets, self.clear_color);
        if self.surface_state.is_some() {
            for camera in self.assets.iter_cameras_mut() {
                camera.resize_viewport(
                    self.surface_state.as_ref().unwrap().config.width as f32,
                    self.surface_state.as_ref().unwrap().config.height as f32,
                );
            }
        }
    }
    fn set_clear_color(&mut self, color: rogalik_common::Color) {
        let col = color.as_srgb();
        self.clear_color = wgpu::Color {
            r: col[0] as f64,
            g: col[1] as f64,
            b: col[2] as f64,
            a: col[3] as f64,
        };
        if let Some(state) = &mut self.surface_state {
            state.renderer2d.set_clear_color(self.clear_color);
        };
    }
    fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            if let Some(state) = &mut self.surface_state {
                state.config.width = width;
                state.config.height = height;
                state.surface.configure(&state.device, &state.config);
            }

            for camera in self.assets.iter_cameras_mut() {
                camera.resize_viewport(width as f32, height as f32);
            }
        }
    }
    fn render(&mut self) {
        if let Some(state) = &mut self.surface_state {
            state
                .renderer2d
                .render(&self.assets, &state.surface, &state.device, &state.queue);
        }
    }
    fn load_material(&mut self, name: &str, params: rogalik_common::MaterialParams) {
        self.assets.create_material(name, params);
        // TODO id self.surface_state build bind_group
    }
    fn load_shader(&mut self, kind: rogalik_common::ShaderKind, path: &str) -> ResourceId {
        // TODO id self.surface_state build pipeline
        self.assets.create_shader(kind, path)
    }
    fn load_font(
        &mut self,
        name: &str,
        bytes: &[u8],
        rows: usize,
        cols: usize,
        padding: Option<(f32, f32)>,
    ) {
        self.assets.load_font(name, bytes, rows, cols, padding);
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
        if let Some(state) = &mut self.surface_state {
            state.renderer2d.draw_atlas_sprite(
                &self.assets,
                index,
                atlas,
                self.current_camera_id,
                position,
                z_index,
                size,
                params,
            )
        } else {
            Err(EngineError::GraphicsNotReady)
        }
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
        if let Some(state) = &mut self.surface_state {
            state.renderer2d.draw_text(
                &self.assets,
                font,
                text,
                self.current_camera_id,
                position,
                z_index,
                size,
                params,
            )
        } else {
            Err(EngineError::GraphicsNotReady)
        }
    }
    fn text_dimensions(&self, font: &str, text: &str, size: f32) -> Vector2f {
        // if let Some(font) = self.assets.get_font(font) {
        //     font.text_dimensions(text, size)
        // } else {
        //     Vector2f::ZERO
        // }
        Vector2f::ZERO
    }
    fn create_camera(&mut self, scale: f32, target: Vector2f) -> ResourceId {
        let (w, h) = match &self.surface_state {
            Some(s) => (s.config.width, s.config.height),
            None => (0, 0),
        };
        self.assets.create_camera(w as f32, h as f32, scale, target)
    }
    fn set_camera(&mut self, id: ResourceId) {
        self.current_camera_id = id;
    }
    fn get_current_camera(&self) -> &dyn rogalik_common::Camera {
        self.assets.get_camera(self.current_camera_id).unwrap()
    }
    fn get_camera(&self, id: ResourceId) -> Option<&dyn rogalik_common::Camera> {
        Some(self.assets.get_camera(id)?)
    }
    fn get_camera_mut(&mut self, id: ResourceId) -> Option<&mut dyn rogalik_common::Camera> {
        Some(self.assets.get_camera_mut(id)?)
    }
}

fn create_surface_state(
    window: &Window,
    assets: &mut assets::WgpuAssets,
    clear_color: wgpu::Color,
) -> Option<SurfaceState> {
    let size = window.inner_size();

    if size.width == 0 || size.height == 0 {
        return None;
    }

    log::debug!("Creating WGPU instance");
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        dx12_shader_compiler: Default::default(),
    });
    log::debug!("Creating WGPU surface");
    let surface = unsafe { instance.create_surface(window) }.unwrap();
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
            features: wgpu::Features::empty(),
            limits: if cfg!(target_arch = "wasm32") {
                wgpu::Limits::downlevel_webgl2_defaults()
            } else {
                wgpu::Limits::default()
            },
            label: None,
        },
        None,
    ))
    .expect("Could not create the device!");

    log::debug!("Config WGPU surface");
    let surface_caps = surface.get_capabilities(&adapter);
    let surface_format = surface_caps
        .formats
        .iter()
        .copied()
        .find(|f| f.is_srgb())
        .unwrap_or(surface_caps.formats[0]);

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

    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: size.width,
        height: size.height,
        present_mode,
        alpha_mode: surface_caps.alpha_modes[0],
        view_formats: vec![],
    };
    surface.configure(&device, &config);

    assets.create_wgpu_data(&device, &queue, &surface_format);

    log::debug!("Creating Renderer2d");
    let renderer2d =
        renderer2d::Renderer2d::new(assets, &device, &queue, &surface_format, clear_color);

    Some(SurfaceState {
        surface,
        device,
        queue,
        config,
        renderer2d,
    })
}
