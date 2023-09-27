use winit::window::Window;

use rogalik_engine::{GraphicsContext, ResourceId, Params2d};
use rogalik_math::vectors::Vector2f;

mod camera;
mod renderer2d;
mod structs;

pub struct WgpuContext {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    renderer2d: renderer2d::Renderer2d,
    current_camera_id: ResourceId,
    cameras: Vec<camera::Camera2D>,
    // clear_color: 
}
impl GraphicsContext for WgpuContext {
    fn new(window: &Window) -> Self {
        create_context(window)
    }
    fn set_clear_color(&mut self, color: rogalik_engine::Color) {
        let col = color.as_f32();
        self.renderer2d.set_clear_color(wgpu::Color { 
            r: col[0] as f64, g: col[1] as f64, b: col[2] as f64, a: col[3] as f64 
        });
    }
    fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);

            for camera in self.cameras.iter_mut() {
                camera.resize_viewport(width as f32, height as f32);
            }
        }
    }
    fn render(&mut self) {
        self.renderer2d.render(
            &self.surface,
            &self.device,
            &self.queue,
            &self.cameras
        );
    }
    fn load_sprite_atlas(&mut self, bytes: &[u8], rows: usize, cols: usize) -> ResourceId {
        self.renderer2d.load_atlas(bytes, rows, cols, &self.device, &self.queue)
    }
    fn load_font(&mut self, bytes: &[u8], rows: usize, cols: usize) -> ResourceId {
        self.renderer2d.load_font(bytes, rows, cols, &self.device, &self.queue)
    }
    fn draw_atlas_sprite(
            &mut self,
            atlas_id: ResourceId,
            index: usize,
            position: rogalik_math::vectors::Vector2f,
            size: rogalik_math::vectors::Vector2f,
            params: Params2d
        ) {
        self.renderer2d.draw_atlas_sprite(
            index,
            atlas_id,
            self.current_camera_id,
            position,
            size,
            params
        );
    }
    fn draw_text(
            &mut self,
            font_id: ResourceId,
            text: &str,
            position: Vector2f,
            size: f32,
            params: Params2d
        ) {
        self.renderer2d.draw_text(
            text,
            font_id,
            self.current_camera_id,
            position,
            size,
            params
        );
    }
    fn create_camera(&mut self, scale: f32, target: Vector2f) -> ResourceId {
        let id = ResourceId(self.cameras.len());
        let camera = camera::Camera2D::new(self.config.width as f32, self.config.height as f32, scale, target);
        self.cameras.push(camera);
        id
    }
    fn set_camera(&mut self, id: ResourceId) {
        self.current_camera_id = id;
    }
    fn get_camera(&self, id: ResourceId) -> Option<&dyn rogalik_engine::traits::Camera>{
        Some(self.cameras.get(id.0)?)
    }
    fn get_camera_mut(&mut self, id: ResourceId) -> Option<&mut dyn rogalik_engine::traits::Camera> {
        Some(self.cameras.get_mut(id.0)?)
    }
    fn get_viewport_size(&self) -> Vector2f {
        Vector2f::new(
            self.config.width as f32,
            self.config.height as f32,
        )
    }
}

fn create_context(window: &Window) -> WgpuContext {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        dx12_shader_compiler: Default::default()
    });

    let surface = unsafe { instance.create_surface(&window) }.unwrap();

    let adapter = pollster::block_on(instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            }
        )).expect("Request for adapter failed!");

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

    let surface_caps = surface.get_capabilities(&adapter);
    let surface_format = surface_caps.formats.iter()
        .copied()
        .find(|f| f.is_srgb())
        .unwrap_or(surface_caps.formats[0]);

    let size = window.inner_size();
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

    let renderer2d = renderer2d::Renderer2d::new(&device, &surface_format);

    WgpuContext {
        surface,
        device,
        queue,
        config,
        renderer2d,
        current_camera_id: ResourceId(0),
        cameras: Vec::new()
    }
}