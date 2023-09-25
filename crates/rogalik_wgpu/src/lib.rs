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
    current_camera_id: usize,
    cameras: Vec<camera::Camera>
}
impl GraphicsContext for WgpuContext {
    fn new(window: &Window) -> Self {
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
        let cameras = vec![
            camera::Camera::new(&device)
        ];

        WgpuContext {
            surface, device, queue, config, sprite_manager, current_camera_id: 0, cameras
        }
    }
    fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
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
}