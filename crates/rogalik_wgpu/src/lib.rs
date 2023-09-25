use winit::window::Window;

use rogalik_engine::traits::{GraphicsContext, ResourceId};

mod camera;
mod sprites;
mod structs;

pub struct WgpuContext {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    sprite_manager: sprites::SpriteManager,
    current_camera_id: ResourceId,
    cameras: Vec<camera::Camera2D>
}
impl GraphicsContext for WgpuContext {
    fn new(window: &Window) -> Self {
        create_context(window)
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
        self.sprite_manager.render(
            &self.surface,
            &self.device,
            &self.queue,
            &self.cameras
        );
    }
    fn load_sprite_atlas(&mut self, bytes: &[u8], rows: usize, cols: usize) -> ResourceId {
        self.sprite_manager.load_atlas(bytes, rows, cols, &self.device, &self.queue)
    }
    fn draw_indexed_sprite(
            &mut self,
            atlas_id: ResourceId,
            index: usize,
            position: rogalik_math::vectors::Vector2F,
            size: rogalik_math::vectors::Vector2F
        ) {
        self.sprite_manager.draw_indexed_sprite(
            index,
            atlas_id,
            self.current_camera_id,
            position,
            size
        );
    }
    fn create_camera(&mut self) -> ResourceId {
        let id = ResourceId(self.cameras.len());
        let camera = camera::Camera2D::new(self.config.width as f32, self.config.height as f32);
        self.cameras.push(camera);
        id
    }
    fn get_camera(&self, id: ResourceId) -> Option<&dyn rogalik_engine::traits::Camera>{
        Some(self.cameras.get(id.0)?)
    }
    fn get_camera_mut(&mut self, id: ResourceId) -> Option<&mut dyn rogalik_engine::traits::Camera> {
        Some(self.cameras.get_mut(id.0)?)
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

    let sprite_manager = sprites::SpriteManager::new(&device, &surface_format);

    WgpuContext {
        surface,
        device,
        queue,
        config,
        sprite_manager,
        current_camera_id: ResourceId(0),
        cameras: Vec::new()
    }
}