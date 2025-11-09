use std::sync::{Arc, Mutex};
#[cfg(target_os = "android")]
use winit::platform::android::activity::AndroidApp;
#[cfg(target_arch = "wasm32")]
use winit::platform::web::WindowAttributesExtWebSys;
use winit::{
    dpi::{LogicalSize, PhysicalSize},
    event_loop::EventLoop,
    window::WindowAttributes,
};

use rogalik_assets::AssetStore;
use rogalik_audio::AudioEngine;
use rogalik_common::AudioDeviceParams;
use rogalik_wgpu::WgpuContext;

use crate::{
    app::{get_event_loop, App},
    input::InputContext,
    time::Time,
    traits::Scene,
    Game,
};

pub struct Context {
    pub assets: Arc<Mutex<rogalik_assets::AssetStore>>,
    pub audio: AudioEngine,
    pub graphics: WgpuContext,
    pub(crate) inner_size: PhysicalSize<u32>,
    pub input: InputContext,
    pub os_path: Option<String>,
    pub(crate) scale_factor: f64,
    pub time: Time,
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
    pub fn build<T>(&self, game: T, scene: Box<dyn Scene<Game = T>>) -> Engine<T>
    where
        T: Game + 'static,
    {
        // set window
        let event_loop = get_event_loop();
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
            input: InputContext::new(),
            time: Time::new(),
            inner_size: PhysicalSize::default(),
            scale_factor: 1.,
            os_path: None,
        };
        let app = App::new(game, context, scene, window_attributes);
        Engine { event_loop, app }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn build_wasm<T>(&self, game: T, scene: Box<dyn Scene<Game = T>>) -> Engine<T>
    where
        T: Game + 'static,
    {
        crate::wasm::configure_handlers();
        log::info!("Logging configured");
        let event_loop = get_event_loop();
        log::info!("Created WASM window");

        let assets = Arc::new(Mutex::new(AssetStore::default()));
        let audio = AudioEngine::new(assets.clone(), self.audio_params);
        let graphics = WgpuContext::new(assets.clone());
        let context = Context {
            assets,
            audio,
            graphics,
            input: InputContext::new(),
            time: Time::new(),
            inner_size: PhysicalSize::default(),
            scale_factor: 1.,
            os_path: None,
        };

        let canvas = crate::wasm::get_canvas();
        let window_attributes = WindowAttributes::default().with_canvas(Some(canvas));
        let app = App::new(game, context, scene, window_attributes);

        Engine { event_loop, app }
    }

    #[cfg(target_os = "android")]
    pub fn build_android<T>(
        &self,
        game: T,
        scene: Box<dyn Scene<Game = T>>,
        app: AndroidApp,
    ) -> Engine<T>
    where
        T: Game + 'static,
    {
        use winit::event_loop::EventLoop;
        use winit::platform::android::EventLoopBuilderExtAndroid;

        let os_path = app
            .internal_data_path()
            .map_or(None, |a| a.to_str().map(|a| a.to_string()));

        let event_loop = EventLoop::builder()
            .with_android_app(app)
            .build()
            .expect("Can't create the event loop!");

        // event_loop.set_control_flow(ControlFlow::Poll);

        let mut window_attributes = WindowAttributes::default();
        if let Some(title) = &self.title {
            window_attributes = window_attributes.with_title(title);
        }

        log::info!("Creating asset store");
        let assets = Arc::new(Mutex::new(AssetStore::default()));
        log::info!("Creating graphics context");
        let graphics = WgpuContext::new(assets.clone());
        log::info!("Creating audio context");
        let audio = AudioEngine::new(assets.clone(), self.audio_params);

        // crate::android::hide_ui();

        let context = Context {
            assets,
            audio,
            graphics,
            input: InputContext::new(),
            time: Time::new(),
            inner_size: PhysicalSize::default(),
            scale_factor: 1.,
            os_path,
        };

        let app = App::new(game, context, scene, window_attributes);
        log::info!("Creating Engine");
        Engine { event_loop, app }
    }
}

pub struct Engine<T>
where
    T: Game + 'static,
{
    app: App<T>,
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

fn run<T>(event_loop: EventLoop<()>, mut app: App<T>)
where
    T: Game + 'static,
{
    app.game.setup(&mut app.context);
    let _ = event_loop.run_app(&mut app);
}
