use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::PhysicalKey;
use winit::window::{Window, WindowAttributes, WindowId};

#[cfg(feature = "remote")]
use crate::remote::{RemoteHandle, RemoteResponse};
use crate::{
    engine::Context,
    scenes::{update_scenes, SceneManager},
    Game, Scene,
};
use rogalik_common::traits::{AudioSetup, GraphicsContext, GraphicsSetup};

pub struct App<T> {
    pub context: Context,
    pub game: T,
    pub scene_manager: SceneManager<T>,
    window: Option<Arc<Window>>,
    window_attributes: WindowAttributes,
    #[cfg(feature = "remote")]
    pub(crate) remote_handle: Option<RemoteHandle>,
}
impl<T: Game> App<T> {
    pub fn new(
        game: T,
        context: Context,
        scene: Box<dyn Scene<Game = T>>,
        window_attributes: WindowAttributes,
    ) -> Self {
        Self {
            context,
            game,
            scene_manager: SceneManager::new(scene),
            window: None,
            window_attributes,
            #[cfg(feature = "remote")]
            remote_handle: None,
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
    fn can_accept_input(&self) -> bool {
        #[cfg(feature = "remote")]
        if let Some(remote) = &self.remote_handle {
            return !remote.is_connected();
        }
        true
    }
}

impl<T: Game> ApplicationHandler<ExternalEvent> for App<T> {
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
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::KeyboardInput { event, .. } => {
                // TODO check if can accept input when remote keyboard commands are impl.
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
                        if code == winit::keyboard::KeyCode::F8 {
                            self.context.graphics.toggle_recording();
                        }
                    }
                }
            }
            WindowEvent::MouseInput { button, state, .. } => {
                if self.can_accept_input() {
                    self.context.input.handle_mouse_button(&button, &state);
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                if self.can_accept_input() {
                    self.context
                        .input
                        .handle_mouse_move(position, &self.context.inner_size);
                }
            }
            WindowEvent::Touch(winit::event::Touch {
                phase,
                location,
                id,
                ..
            }) => {
                if self.can_accept_input() {
                    self.context
                        .input
                        .handle_touch(id, phase, location, &self.context.inner_size);
                }
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

                #[cfg(feature = "capture")]
                if let Some(handle) = &self.remote_handle {
                    if handle.is_expecting_screenshot() {
                        if let Some(buf) = self.context.graphics.take_screenshot() {
                            // TODO handle error.
                            let _ = handle.tx.send(RemoteResponse::ScreenShot(buf));
                        }
                    }
                }
            }
            _ => (),
        }
    }
    /// At the moment handles only requests from the remote controller.
    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: ExternalEvent) {
        match event {
            ExternalEvent::MouseButton(button, state) => {
                self.context.input.handle_mouse_button(&button, &state)
            }
            ExternalEvent::MouseMove(x, y) => {
                self.context.input.handle_mouse_move(
                    winit::dpi::PhysicalPosition {
                        x: x as f64,
                        y: y as f64,
                    },
                    &self.context.inner_size,
                );
            }
            ExternalEvent::ScreenShot => {
                self.context.graphics.request_screenshot();
            }
        }
    }
}

#[derive(Debug)]
pub(crate) enum ExternalEvent {
    MouseButton(MouseButton, ElementState),
    MouseMove(u32, u32),
    ScreenShot,
}

pub fn get_event_loop() -> EventLoop<ExternalEvent> {
    let event_loop = EventLoop::with_user_event()
        .build()
        .expect("Can't create the event loop!");
    #[cfg(not(target_arch = "wasm32"))]
    event_loop.set_control_flow(ControlFlow::Poll);
    #[cfg(target_arch = "wasm32")]
    event_loop.set_control_flow(ControlFlow::Wait);
    event_loop
}
