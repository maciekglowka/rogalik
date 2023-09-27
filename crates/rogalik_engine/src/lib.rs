use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder}
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
    pub time: time::Time
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
    pub fn new(game: T) -> Self {
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
        let window = WindowBuilder::new().build(&event_loop).unwrap();

        // set canvas
        #[cfg(target_arch = "wasm32")]
        wasm::set_canvas(&window);
        
        let graphics = GraphicsContext::new(&window);
        let context = Context {
            graphics,
            input: input::InputContext::new(),
            time: time::Time::new()
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
    let mut frame_start = std::time::Instant::now();

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
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(physical_size) =>
                        context.graphics.resize(physical_size.width, physical_size.height),
                    WindowEvent::ScaleFactorChanged { new_inner_size, ..} => 
                        context.graphics.resize(new_inner_size.width, new_inner_size.height),
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
