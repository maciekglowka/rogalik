use std::collections::HashSet;
use winit::event::{
    ElementState,
    KeyboardInput
};

pub use winit::event::{MouseButton, VirtualKeyCode};

pub struct InputContext {
    keys_down: HashSet<VirtualKeyCode>,
    keys_pressed: HashSet<VirtualKeyCode>,
    keys_released: HashSet<VirtualKeyCode>,
    mouse_buttons_down: HashSet<MouseButton>,
    mouse_buttons_pressed: HashSet<MouseButton>,
    mouse_buttons_released: HashSet<MouseButton>,
}
impl InputContext {
    pub fn new() -> Self {
        Self {
            keys_down: HashSet::new(),
            keys_pressed: HashSet::new(),
            keys_released: HashSet::new(),
            mouse_buttons_down: HashSet::new(),
            mouse_buttons_pressed: HashSet::new(),
            mouse_buttons_released: HashSet::new()
        }
    }
    pub fn clear(&mut self) {
        self.keys_pressed = HashSet::new();
        self.keys_released = HashSet::new();
        self.mouse_buttons_pressed = HashSet::new();
        self.mouse_buttons_released = HashSet::new();
    }
    pub fn handle_keyboard(&mut self, input: &KeyboardInput) {
        if let Some(key) = input.virtual_keycode {
            match input.state {
                ElementState::Pressed => {
                    if !self.keys_down.contains(&key) {
                        self.keys_pressed.insert(key);
                        self.keys_down.insert(key);
                    }
                },
                ElementState::Released => {
                    self.keys_down.remove(&key);
                    self.keys_released.insert(key);
                },
            };
        }
    }
    pub fn handle_mouse_button(&mut self, button: &MouseButton, state: &ElementState) {
        match state {
            ElementState::Pressed => {
                if !self.mouse_buttons_down.contains(button) {
                    self.mouse_buttons_down.insert(*button);
                    self.mouse_buttons_pressed.insert(*button);
                }
            },
            ElementState::Released => {
                self.mouse_buttons_down.remove(button);
                self.mouse_buttons_released.insert(*button);
            },
        };
    }
    pub fn is_key_down(&self, code: VirtualKeyCode) -> bool {
        self.keys_down.contains(&code)
    }
    pub fn is_key_pressed(&self, code: VirtualKeyCode) -> bool {
        self.keys_pressed.contains(&code)
    }
    pub fn is_key_released(&self, code: VirtualKeyCode) -> bool {
        self.keys_released.contains(&code)
    }
    pub fn is_mouse_button_down(&self, button: MouseButton) -> bool {
        self.mouse_buttons_down.contains(&button)
    }
    pub fn is_mouse_button_pressed(&self, button: MouseButton) -> bool {
        self.mouse_buttons_pressed.contains(&button)
    }
    pub fn is_mouse_button_released(&self, button: MouseButton) -> bool {
        self.mouse_buttons_released.contains(&button)
    }
}