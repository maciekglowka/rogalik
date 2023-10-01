use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
    dpi::PhysicalSize
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
    window_size: PhysicalSize<u32>
}
impl<G: GraphicsContext> Context<G> {
    pub fn get_viewport_size(&self) -> rogalik_math::vectors::Vector2f {
        rogalik_math::vectors::vector2::Vector2f::new(
            self.window_size.width as f32, self.window_size.height as f32
        )
    }
}

pub struct Engine<G, T>
where
    G: GraphicsContext + 'static,
    T: Game<G> + 'static
{
    window: Window,
    event_loop: EventLoop<()>,
    context: Context<G>,
    game: T
}
impl<G, T> Engine<G, T>
where
    G: GraphicsContext + 'static,
    T: Game<G> + 'static
{
    pub fn new(game: T, width: u32, height: u32, title: &str) -> Self {
        // set logging
        cfg_if::cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                wasm::configure_handlers()
            } else {
                env_logger::init();
            }
        }

        // set window
        let window_size = PhysicalSize::new(width, height);
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title(title)
            .with_inner_size(window_size)
            .build(&event_loop)
            .expect("Can't create window!");

        // set canvas
        #[cfg(target_arch = "wasm32")]
        wasm::set_canvas(&window, width, height);
        
        let graphics = GraphicsContext::new(&window);
        let context = Context {
            graphics,
            input: input::InputContext::new(),
            time: time::Time::new(),
            window_size
        };
        Self {
            window, event_loop, game, context
        }
    }
    pub fn run(self) {
        pollster::block_on(run::<G, T>(self.window, self.event_loop, self.game, self.context));
    }
}

async fn run<G, T> (
    window: Window,
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
            } if window_id == window.id() => {
                match event {
                    WindowEvent::KeyboardInput { input, .. } => {
                        context.input.handle_keyboard(input);
                    },
                    WindowEvent::MouseInput { button, state, .. } => {
                        context.input.handle_mouse_button(button, state);
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        context.input.handle_mouse_move(*position, window.inner_size());
                    },
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(physical_size) => {
                        context.graphics.resize(physical_size.width, physical_size.height);
                        context.window_size = *physical_size;
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, ..} => {
                        context.graphics.resize(new_inner_size.width, new_inner_size.height);
                        context.window_size = **new_inner_size;
                    }
                    _ => {}
                }
            },
            Event::RedrawRequested(window_id) if window_id == window.id() => {
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
                window.request_redraw();
            },
            _ => {}
        }
    });
}
