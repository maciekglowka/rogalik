use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
    dpi::{LogicalSize, PhysicalSize}
};

pub mod input;
pub mod structs;
mod time;
pub mod traits;

#[cfg(target_arch = "wasm32")]
mod wasm;

pub use traits::{Game, GraphicsContext};
pub use structs::{ResourceId, Params2d, Color};

pub struct Context<G: GraphicsContext> {
    pub graphics: G,
    pub input: input::InputContext,
    pub time: time::Time,
    pub window: Window,
    inner_size: PhysicalSize<u32>,
    scale_factor: f64
}
impl<G: GraphicsContext> Context<G> {
    pub fn get_physical_size(&self) -> rogalik_math::vectors::Vector2f {
        rogalik_math::vectors::vector2::Vector2f::new(
            self.inner_size.width as f32, self.inner_size.height as f32
        )
    }
    pub fn get_logical_size(&self) -> rogalik_math::vectors::Vector2f {
        let size: LogicalSize<f32> = self.inner_size
            .to_logical(self.scale_factor);
        rogalik_math::vectors::vector2::Vector2f::new(
            size.width, size.height
        )
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
    pub fn build<G, T>(&self, game: T) -> Engine<G, T>
    where
        G: GraphicsContext + 'static,
        T: Game<G> + 'static
    {
        // set logging
        cfg_if::cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                wasm::configure_handlers()
            } else {
                env_logger::init();
            }
        }

        // set window
        let event_loop = EventLoop::new();
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
        
        let window = window_builder.build(&event_loop)
            .expect("Can't create window!");

            // set canvas
        #[cfg(target_arch = "wasm32")]
        wasm::set_canvas(&window);
        
        let graphics = GraphicsContext::new(&window);
        let context = Context {
            graphics,
            input: input::InputContext::new(),
            time: time::Time::new(),
            inner_size: window.inner_size(),
            scale_factor: window.scale_factor(),
            window,
        };
        Engine {
            event_loop, game, context
        }
    }
}

pub struct Engine<G, T>
where
    G: GraphicsContext + 'static,
    T: Game<G> + 'static
{
    event_loop: EventLoop<()>,
    context: Context<G>,
    game: T
}
impl<G, T> Engine<G, T>
where
    G: GraphicsContext + 'static,
    T: Game<G> + 'static
{
    pub fn run(self) {
        pollster::block_on(run::<G, T>(self.event_loop, self.game, self.context));
    }
}

async fn run<G, T> (
    event_loop: EventLoop<()>,
    mut game: T,
    mut context: Context<G>
) 
where
    G: GraphicsContext + 'static,
    T: Game<G> + 'static
{
    game.setup(&mut context);

    let _ = event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                window_id,
                ref event
            } if window_id == context.window.id() => {
                match event {
                    WindowEvent::KeyboardInput { input, .. } => {
                        context.input.handle_keyboard(input);
                    },
                    WindowEvent::MouseInput { button, state, .. } => {
                        context.input.handle_mouse_button(button, state);
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        context.input.handle_mouse_move(*position, &context.window);
                    },
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(physical_size) => {
                        context.inner_size = context.window.inner_size();
                        context.scale_factor = context.window.scale_factor();
                        context.graphics.resize(physical_size.width, physical_size.height);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, ..} => {
                        context.inner_size = context.window.inner_size();
                        context.scale_factor = context.window.scale_factor();
                        context.graphics.resize(new_inner_size.width, new_inner_size.height);
                    }
                    _ => {}
                }
            },
            Event::RedrawRequested(window_id) if window_id == context.window.id() => {
                // state.update();
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
            },
            Event::MainEventsCleared => {
                context.window.request_redraw();
            },
            _ => {}
        }
    });
}
