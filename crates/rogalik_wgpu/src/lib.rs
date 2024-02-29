use winit::window::Window;

use rogalik_engine::{GraphicsContext, ResourceId, Params2d, EngineError};
use rogalik_math::vectors::Vector2f;

mod assets;
mod camera;
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
    assets: assets::AssetStore,
    current_camera_id: ResourceId,
    cameras: Vec<camera::Camera2D>,
    clear_color: wgpu::Color,
    surface_state: Option<SurfaceState>
}
impl GraphicsContext for WgpuContext {
    fn new() -> Self {
        Self {
            assets: assets::AssetStore::new(),
            current_camera_id: ResourceId::default(),
            cameras: Vec::new(),
            clear_color: wgpu::Color::BLACK,
            surface_state: None
        }
    }
    fn has_context(&self) -> bool {
        self.surface_state.is_some()
    }
    fn create_context(&mut self, window: &Window) {
        self.surface_state = create_surface_state(
            window,
            &self.assets,
            self.clear_color
        );
        if self.surface_state.is_some() {
            for camera in self.cameras.iter_mut() {
                camera.resize_viewport(
                    self.surface_state.as_ref().unwrap().config.width as f32,
                    self.surface_state.as_ref().unwrap().config.height as f32,
                );
            }
        }
    }
    fn set_clear_color(&mut self, color: rogalik_engine::Color) {
        let col = color.as_srgb();
        self.clear_color = wgpu::Color { 
            r: col[0] as f64, g: col[1] as f64, b: col[2] as f64, a: col[3] as f64 
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

            for camera in self.cameras.iter_mut() {
                camera.resize_viewport(width as f32, height as f32);
            }
        }
    }
    fn render(&mut self) {
        if let Some(state) = &mut self.surface_state {
            state.renderer2d.render(
                &state.surface,
                &state.device,
                &state.queue,
                &self.cameras
            );
        }
    }
    fn load_sprite_atlas(&mut self, name: &str, bytes: &[u8], rows: usize, cols: usize, padding: Option<(f32, f32)>) {
        self.assets.load_atlas(name, bytes, rows, cols, padding);
    }
    fn load_font(&mut self, name: &str, bytes: &[u8], rows: usize, cols: usize, padding: Option<(f32, f32)>) {
        self.assets.load_font(name, bytes, rows, cols, padding);
    }
    fn draw_atlas_sprite(
            &mut self,
            atlas: &str,
            index: usize,
            position: rogalik_math::vectors::Vector2f,
            z_index: i32,
            size: rogalik_math::vectors::Vector2f,
            params: Params2d
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
                params
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
            params: Params2d
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
                params
            )
        } else {
            Err(EngineError::GraphicsNotReady)
        }
    }
    fn text_dimensions(&self, font: &str, text: &str, size: f32) -> Vector2f {
        if let Some(font) = self.assets.get_font(font) {
            font.text_dimensions(text, size)
        } else {
            Vector2f::ZERO
        }
    }
    fn create_camera(&mut self, scale: f32, target: Vector2f) -> ResourceId {
        let id = ResourceId(self.cameras.len());
        let (w, h) = match &self.surface_state {
            Some(s) => (s.config.width, s.config.height),
            None => (0, 0)
        };
        let camera = camera::Camera2D::new(w as f32, h as f32, scale, target);
        self.cameras.push(camera);
        id
    }
    fn set_camera(&mut self, id: ResourceId) {
        self.current_camera_id = id;
    }
    fn get_current_camera(&self) -> &dyn rogalik_engine::traits::Camera {
        &self.cameras[self.current_camera_id.0]
    }
    fn get_camera(&self, id: ResourceId) -> Option<&dyn rogalik_engine::traits::Camera>{
        Some(self.cameras.get(id.0)?)
    }
    fn get_camera_mut(&mut self, id: ResourceId) -> Option<&mut dyn rogalik_engine::traits::Camera> {
        Some(self.cameras.get_mut(id.0)?)
    }
}

fn create_surface_state(
    window: &Window,
    assets: &assets::AssetStore,
    clear_color: wgpu::Color
) -> Option<SurfaceState> {
    let size = window.inner_size();

    if size.width == 0 || size.height == 0 {
        return None;
    }

    log::debug!("Creating WGPU instance");
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        dx12_shader_compiler: Default::default()
    });
    log::debug!("Creating WGPU surface");
    let surface = unsafe { instance.create_surface(window) }.unwrap();
    log::debug!("Creating WGPU adapter");
    let adapter = pollster::block_on(instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            }
        )).expect("Request for adapter failed!");
    log::debug!("Creating WGPU device");
    let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
                label: None
            },
            None
        )).expect("Could not create the device!");

    log::debug!("Config WGPU surface");
    let surface_caps = surface.get_capabilities(&adapter);
    let surface_format = surface_caps.formats.iter()
        .copied()
        .find(|f| f.is_srgb())
        .unwrap_or(surface_caps.formats[0]);

    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: size.width,
        height: size.height,
        present_mode: surface_caps.present_modes[0],
        alpha_mode: surface_caps.alpha_modes[0],
        view_formats: vec![]
    };
    surface.configure(&device, &config);

    log::debug!("Creating Renderer2d");
    let renderer2d = renderer2d::Renderer2d::new(
        assets,
        &device,
        &queue,
        &surface_format,
        clear_color
    );

    Some(SurfaceState { 
        surface,
        device,
        queue,
        config,
        renderer2d
    })
}