use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};

use crate::{
    scenes::{update_scenes, SceneManager},
    Context, Game,
};
use rogalik_common::GraphicsContext;

pub struct App<T> {
    pub context: Context,
    pub game: T,
    pub scene_manager: SceneManager<T>,
    window: Option<Window>,
    window_attributes: WindowAttributes,
}
impl<T: Game> App<T> {
    pub fn new(game: T, context: Context, window_attributes: WindowAttributes) -> Self {
        Self {
            context,
            game,
            scene_manager: SceneManager::new(),
            window: None,
            window_attributes,
        }
    }
}

impl<T: Game> ApplicationHandler for App<T> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.window = Some(
            event_loop
                .create_window(self.window_attributes.clone())
                .expect("Can't create window!"),
        );
        self.context.scale_factor = self
            .window
            .as_ref()
            .expect("No valid window!")
            .scale_factor();
        self.context.inner_size = self.window.as_ref().expect("No valid window!").inner_size();
        self.context
            .graphics
            .create_context(self.window.as_ref().expect("No valid window!"));
        self.game.resume(&mut self.context);
        self.game.resize(&mut self.context);
    }
    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        self.window
            .as_ref()
            .expect("No window found!")
            .request_redraw();
    }
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::KeyboardInput { event, .. } => {
                self.context.input.handle_keyboard(&event);
            }
            WindowEvent::MouseInput { button, state, .. } => {
                self.context.input.handle_mouse_button(&button, &state);
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.context
                    .input
                    .handle_mouse_move(position, &self.context.inner_size);
            }
            WindowEvent::Touch(winit::event::Touch {
                phase,
                location,
                id,
                ..
            }) => {
                log::info!("Engine touch: {}, {:?}", id, phase);
                self.context
                    .input
                    .handle_touch(id, phase, location, &self.context.inner_size);
            }
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(physical_size) => {
                if !self.context.graphics.has_context() {
                    if let Some(window) = &self.window {
                        self.context.graphics.create_context(window);
                    }
                }
                log::info!("Resized: {:?}", physical_size);
                self.context.inner_size = physical_size;
                self.context
                    .graphics
                    .resize(physical_size.width, physical_size.height);
                self.game.resize(&mut self.context);
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                log::info!("Scale factor changed: {:?}", scale_factor);
                self.context.scale_factor = scale_factor;
            }
            WindowEvent::RedrawRequested => {
                // let start = std::time::Instant::now();
                self.context.time.update();
                self.context
                    .graphics
                    .update_time(self.context.time.get_delta());
                update_scenes(&mut self.scene_manager, &mut self.game, &mut self.context);

                self.context.graphics.render();
                self.context.input.clear();
                // println!(
                //     "{} {}",
                //     1. / start.elapsed().as_secs_f32(),
                //     start.elapsed().as_secs_f32()
                // );
            }
            _ => (),
        }
    }
}

pub fn get_event_loop() -> EventLoop<()> {
    let event_loop = EventLoop::new().expect("Can't create the event loop!");
    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop
}
