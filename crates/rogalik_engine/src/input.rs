use std::collections::HashSet;
use winit::event::{
    ElementState,
    KeyboardInput
};

pub use winit::event::{MouseButton, VirtualKeyCode};

pub struct InputContext {
    keys_pressed: HashSet<VirtualKeyCode>,
    mouse_buttons_pressed: HashSet<MouseButton>
}
impl InputContext {
    pub fn new() -> Self {
        Self {
            keys_pressed: HashSet::new(),
            mouse_buttons_pressed: HashSet::new(),
        }
    }
    pub fn handle_keyboard(&mut self, input: &KeyboardInput) {
        if let Some(key) = input.virtual_keycode {
            match input.state {
                ElementState::Pressed => self.keys_pressed.insert(key),
                ElementState::Released => self.keys_pressed.remove(&key),
            };
        }
    }
    pub fn handle_mouse_button(&mut self, button: &MouseButton, state: &ElementState) {
        match state {
            ElementState::Pressed => self.mouse_buttons_pressed.insert(*button),
            ElementState::Released => self.mouse_buttons_pressed.remove(button),
        };
    }
    pub fn is_key_down(&self, code: VirtualKeyCode) -> bool {
        self.keys_pressed.contains(&code)
    }
    pub fn is_mouse_button_down(&self, button: MouseButton) -> bool {
        self.mouse_buttons_pressed.contains(&button)
    }
}