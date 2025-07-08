use rogalik_audio::AudioEngine;
use rogalik_common::AudioDeviceParams;
use rogalik_wgpu::WgpuContext;
use std::sync::{Arc, Mutex};
#[cfg(target_os = "android")]
pub use winit::platform::android::activity::AndroidApp;
#[cfg(target_arch = "wasm32")]
use winit::platform::web::WindowAttributesExtWebSys;
use winit::{
    dpi::{LogicalSize, PhysicalSize},
    event_loop::EventLoop,
    window::WindowAttributes,
};

#[cfg(target_os = "android")]
mod android;
mod app;
pub mod input;
mod scenes;
mod time;
pub mod traits;

#[cfg(target_arch = "wasm32")]
mod wasm;

pub use log;
pub use time::Instant;
pub use traits::{Game, Scene, SceneChange};

use rogalik_assets::AssetStore;

pub struct Context {
    pub assets: Arc<Mutex<rogalik_assets::AssetStore>>,
    pub audio: AudioEngine,
    pub graphics: WgpuContext,
    pub input: input::InputContext,
    pub time: time::Time,
    inner_size: PhysicalSize<u32>,
    scale_factor: f64,
    pub os_path: Option<String>,
}
impl Context {
    pub fn get_physical_size(&self) -> rogalik_math::vectors::Vector2f {
        rogalik_math::vectors::vector2::Vector2f::new(
            self.inner_size.width as f32,
            self.inner_size.height as f32,
        )
    }
    pub fn get_logical_size(&self) -> rogalik_math::vectors::Vector2f {
        let size: LogicalSize<f32> = self.inner_size.to_logical(self.scale_factor);
        rogalik_math::vectors::vector2::Vector2f::new(size.width, size.height)
    }
}

#[derive(Default)]
pub struct EngineBuilder {
    title: Option<String>,
    physical_size: Option<(u32, u32)>,
    logical_size: Option<(f32, f32)>,
    resizable: bool,
    fullscreen: bool,
    audio_params: Option<AudioDeviceParams>,
}
impl EngineBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn with_title(mut self, title: String) -> Self {
        self.title = Some(title);
        self
    }
    pub fn with_physical_size(mut self, w: u32, h: u32) -> Self {
        self.physical_size = Some((w, h));
        self
    }
    pub fn with_logical_size(mut self, w: f32, h: f32) -> Self {
        self.logical_size = Some((w, h));
        self
    }
    pub fn resizable(mut self, val: bool) -> Self {
        self.resizable = val;
        self
    }
    pub fn fullscreen(mut self, val: bool) -> Self {
        self.fullscreen = val;
        self
    }
    pub fn with_audio(mut self, params: AudioDeviceParams) -> Self {
        self.audio_params = Some(params);
        self
    }
    pub fn build<T>(&self, game: T, scene: Box<dyn traits::Scene<Game = T>>) -> Engine<T>
    where
        T: Game + 'static,
    {
        // set logging
        // #[cfg(not(target_os = "android"))]
        // env_logger::init();

        // set window
        let event_loop = app::get_event_loop();
        let mut window_attributes = WindowAttributes::default();

        if let Some(title) = &self.title {
            window_attributes = window_attributes.with_title(title);
        }

        window_attributes.resizable = self.resizable;

        if let Some(size) = self.physical_size {
            let window_size = PhysicalSize::new(size.0, size.1);
            window_attributes = window_attributes.with_inner_size(window_size);
        } else if let Some(size) = self.logical_size {
            let window_size = LogicalSize::new(size.0, size.1);
            window_attributes = window_attributes.with_inner_size(window_size);
        }

        if self.fullscreen {
            window_attributes = window_attributes
                .with_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
        }

        let assets = Arc::new(Mutex::new(AssetStore::default()));
        let graphics = WgpuContext::new(assets.clone());
        let audio = AudioEngine::new(assets.clone(), self.audio_params);

        let context = Context {
            assets,
            audio,
            graphics,
            input: input::InputContext::new(),
            time: time::Time::new(),
            inner_size: PhysicalSize::default(),
            scale_factor: 1.,
            os_path: None,
        };
        let mut app = app::App::new(game, context, window_attributes);
        app.scene_manager.push(scene);
        Engine { event_loop, app }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn build_wasm<T>(&self, game: T, scene: Box<dyn traits::Scene<Game = T>>) -> Engine<T>
    where
        T: Game + 'static,
    {
        wasm::configure_handlers();
        log::info!("Logging configured");
        let event_loop = app::get_event_loop();
        log::info!("Created WASM window");

        let assets = Arc::new(Mutex::new(AssetStore::default()));
        let audio = AudioEngine::new(assets.clone(), self.audio_params);
        let graphics = WgpuContext::new(assets.clone());
        let context = Context {
            assets,
            audio,
            graphics,
            input: input::InputContext::new(),
            time: time::Time::new(),
            inner_size: PhysicalSize::default(),
            scale_factor: 1.,
            os_path: None,
        };

        let canvas = wasm::get_canvas();
        let window_attributes = WindowAttributes::default().with_canvas(Some(canvas));
        let mut app = app::App::new(game, context, window_attributes);
        app.scene_manager.push(scene);

        Engine { event_loop, app }
    }

    #[cfg(target_os = "android")]
    pub fn build_android<G, T>(&self, game: T, app: AndroidApp) -> Engine<G, T>
    where
        G: GraphicsContext + 'static,
        T: Game<G> + 'static,
    {
        use winit::event_loop::EventLoopBuilder;
        use winit::platform::android::EventLoopBuilderExtAndroid;

        android_logger::init_once(
            android_logger::Config::default()
                .with_max_level(log::LevelFilter::Info)
                .with_tag("Rogalik"),
        );

        let os_path = app
            .internal_data_path()
            .map_or(None, |a| a.to_str().map(|a| a.to_string()));

        // set window
        let event_loop = EventLoopBuilder::new()
            .with_android_app(app)
            .build()
            .expect("Can't create the event loop!");

        let mut window_builder = WindowBuilder::new();

        if let Some(title) = &self.title {
            window_builder = window_builder.with_title(title);
        }

        let window = window_builder
            .build(&event_loop)
            .expect("Can't create window!");

        log::info!("Creating graphics context");
        let graphics = GraphicsContext::new();
        log::info!("Graphics created");

        android::hide_ui();

        let context = Context {
            graphics,
            input: input::InputContext::new(),
            time: time::Time::new(),
            inner_size: window.inner_size(),
            scale_factor: window.scale_factor(),
            window,
            os_path,
        };
        log::info!("Creating Engine");
        Engine {
            event_loop,
            game,
            context,
        }
    }
}

pub struct Engine<T>
where
    T: Game + 'static,
{
    app: app::App<T>,
    event_loop: EventLoop<()>,
}
impl<T> Engine<T>
where
    T: Game + 'static,
{
    pub fn run(self) {
        run::<T>(self.event_loop, self.app);
    }
}

fn run<T>(event_loop: EventLoop<()>, mut app: app::App<T>)
where
    T: Game + 'static,
{
    app.game.setup(&mut app.context);
    let _ = event_loop.run_app(&mut app);
}
