use std::time::{Duration, Instant};
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder}
};

pub mod traits;

#[cfg(target_arch = "wasm32")]
mod wasm;

pub use traits::ResourceId;
pub use traits::GraphicsContext;

pub struct Context<G: GraphicsContext> {
    pub graphics: G
}

pub struct Engine {
    window: Window,
    event_loop: EventLoop<()>
}
impl Engine {
    pub fn new<G: GraphicsContext + 'static>() -> (Self, Context<G>) {
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
        let engine = Engine {
            window, event_loop
        };
        let context = Context {
            graphics
        };
        (engine, context)
    }
    pub fn run<G: GraphicsContext + 'static>(self, context: Context<G>) {
        pollster::block_on(run::<G>(self.window, self.event_loop, context));
    }
}

// #[cfg_attr(target_arch="wasm32", wasm_bindgen(start))]
async fn run<G: GraphicsContext + 'static>(
    window: Window,
    event_loop: EventLoop<()>,
    mut context: Context<G>
) {
    let _ = event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                window_id,
                ref event
            } if window_id == window.id() => {
                match event {
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
                // let now = Instant::now();
                context.graphics.render();
                // println!("{}", now.elapsed().as_millis());
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
