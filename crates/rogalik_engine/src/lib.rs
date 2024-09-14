#[cfg(target_os = "android")]
pub use winit::platform::android::activity::AndroidApp;
use winit::{
    dpi::{LogicalSize, PhysicalSize},
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

#[cfg(target_os = "android")]
mod android;

pub mod input;
mod time;
pub mod traits;

#[cfg(target_arch = "wasm32")]
mod wasm;

pub use log;
pub use rogalik_common::{Color, GraphicsContext, Params2d, ResourceId};
pub use time::Instant;
pub use traits::Game;

pub struct Context {
    pub graphics: rogalik_wgpu::WgpuContext,
    pub input: input::InputContext,
    pub time: time::Time,
    pub window: Window,
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
    pub fn build<T>(&self, game: T) -> Engine<T>
    where
        T: Game + 'static,
    {
        // set logging
        #[cfg(not(target_os = "android"))]
        env_logger::init();

        // set window
        let event_loop = EventLoop::new().expect("Can't create the event loop!");
        let mut window_builder = WindowBuilder::new();

        if let Some(title) = &self.title {
            window_builder = window_builder.with_title(title);
        }
        if let Some(size) = self.physical_size {
            let window_size = PhysicalSize::new(size.0, size.1);
            window_builder = window_builder.with_inner_size(window_size);
        } else if let Some(size) = self.logical_size {
            let window_size = LogicalSize::new(size.0, size.1);
            window_builder = window_builder.with_inner_size(window_size);
        }

        let window = window_builder
            .build(&event_loop)
            .expect("Can't create window!");

        let graphics = GraphicsContext::new();
        let context = Context {
            graphics,
            input: input::InputContext::new(),
            time: time::Time::new(),
            inner_size: window.inner_size(),
            scale_factor: window.scale_factor(),
            window,
            os_path: None,
        };
        Engine {
            event_loop,
            game,
            context,
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn build_wasm<G, T>(&self, game: T) -> Engine<G, T>
    where
        G: GraphicsContext + 'static,
        T: Game<G> + 'static,
    {
        wasm::configure_handlers();
        log::info!("Logging configured");
        let event_loop = EventLoop::new().expect("Can't create the event loop!");
        let window = wasm::get_window(&event_loop);
        log::info!("Created WASM window");
        log::info!("{:?}", window.inner_size());
        let graphics = GraphicsContext::new();
        let context = Context {
            graphics,
            input: input::InputContext::new(),
            time: time::Time::new(),
            inner_size: window.inner_size(),
            scale_factor: window.scale_factor(),
            window,
            os_path: None,
        };
        Engine {
            event_loop,
            game,
            context,
        }
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
    event_loop: EventLoop<()>,
    context: Context,
    game: T,
}
impl<T> Engine<T>
where
    T: Game + 'static,
{
    pub fn run(self) {
        pollster::block_on(run::<T>(self.event_loop, self.game, self.context));
    }
}

async fn run<T>(event_loop: EventLoop<()>, mut game: T, mut context: Context)
where
    T: Game + 'static,
{
    game.setup(&mut context);
    let mut close_requested = false;

    let _ = event_loop.run(move |event, event_loop| {
        match event {
            Event::WindowEvent {
                window_id,
                ref event,
            } if window_id == context.window.id() => {
                match event {
                    WindowEvent::KeyboardInput { event, .. } => {
                        context.input.handle_keyboard(event);
                    }
                    WindowEvent::MouseInput { button, state, .. } => {
                        context.input.handle_mouse_button(button, state);
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        context
                            .input
                            .handle_mouse_move(*position, &context.inner_size);
                    }
                    WindowEvent::Touch(winit::event::Touch {
                        phase,
                        location,
                        id,
                        ..
                    }) => {
                        log::info!("Engine touch: {}, {:?}", id, phase);
                        context
                            .input
                            .handle_touch(*id, *phase, *location, &context.inner_size);
                    }
                    WindowEvent::CloseRequested => close_requested = true,
                    WindowEvent::Resized(physical_size) => {
                        if !context.graphics.has_context() {
                            context.graphics.create_context(&context.window);
                        }
                        log::info!("Resized: {:?}", physical_size);
                        context.inner_size = *physical_size;
                        // context.scale_factor = context.window.scale_factor();
                        context
                            .graphics
                            .resize(physical_size.width, physical_size.height);
                        game.resize(&mut context);
                    }
                    WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                        context.scale_factor = *scale_factor;
                    }
                    WindowEvent::RedrawRequested => {
                        // state.update();
                        // let start = std::time::Instant::now();
                        context.time.update();
                        game.update(&mut context);
                        context.graphics.render();
                        context.input.clear();
                        // println!("{} {}", 1. / start.elapsed().as_secs_f32(), start.elapsed().as_secs_f32());
                        // match gpu_state.render(&pass) {
                        //     Ok(_) => {},
                        //     Err(wgpu::SurfaceError::Lost) => gpu_state.resize(window.inner_size()),
                        //     Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                        //     Err(e) => eprintln!("{:?}", e)
                        // }
                    }
                    _ => {}
                }
            }
            Event::Resumed => {
                context.graphics.create_context(&context.window);
                game.resume(&mut context);
                game.resize(&mut context);
            }
            Event::AboutToWait => {
                if !close_requested {
                    context.window.request_redraw();
                } else {
                    event_loop.exit();
                }
            }
            _ => {}
        }
    });
}
