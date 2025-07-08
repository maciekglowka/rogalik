use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::PhysicalKey;
use winit::window::{Window, WindowAttributes, WindowId};

use crate::{
    scenes::{update_scenes, SceneManager},
    Context, Game,
};
use rogalik_common::{AudioContext, GraphicsContext};

pub struct App<T> {
    pub context: Context,
    pub game: T,
    pub scene_manager: SceneManager<T>,
    window: Option<Arc<Window>>,
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
    fn set_inner_size_on_resume(&mut self) {
        self.context.inner_size = self.window.as_ref().expect("No valid window!").inner_size();
    }
    fn resize(&mut self, physical_size: PhysicalSize<u32>) {
        self.context.inner_size = physical_size;
        self.context
            .graphics
            .resize(physical_size.width, physical_size.height);
    }
}

impl<T: Game> ApplicationHandler for App<T> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        log::info!("App resume");
        self.window = Some(Arc::new(
            event_loop
                .create_window(self.window_attributes.clone())
                .expect("Can't create window!"),
        ));
        self.context.scale_factor = self
            .window
            .as_ref()
            .expect("No valid window!")
            .scale_factor();
        log::info!("Scale factor set to: {:?}", self.context.scale_factor);

        self.set_inner_size_on_resume();
        log::info!("Inner size set to: {:?}", self.context.inner_size);

        self.context
            .graphics
            .create_context(self.window.as_ref().expect("No valid window!").clone());

        self.context.audio.create_context();

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

                // reload assets
                #[cfg(debug_assertions)]
                if let PhysicalKey::Code(code) = event.physical_key {
                    if let ElementState::Pressed = event.state {
                        // TODO add CTRL or SHIFT modifier
                        if code == winit::keyboard::KeyCode::F5 {
                            if let Ok(mut store) = self.context.assets.lock() {
                                store.reload_modified();
                            }
                            self.context.graphics.update_assets();
                            self.context.audio.update_assets();
                            self.game.reload_assets(&mut self.context);
                        }
                    }
                }
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
                        self.context.graphics.create_context(window.clone());
                    }
                }

                log::info!("Resized: {:?}", physical_size);
                self.resize(physical_size);
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
    #[cfg(not(target_arch = "wasm32"))]
    event_loop.set_control_flow(ControlFlow::Poll);
    #[cfg(target_arch = "wasm32")]
    event_loop.set_control_flow(ControlFlow::Wait);
    event_loop
}
