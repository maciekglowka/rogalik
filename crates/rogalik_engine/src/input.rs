use std::collections::{HashSet, HashMap};
use winit::{
    dpi::{PhysicalPosition, PhysicalSize, LogicalSize},
    event::{ElementState, KeyboardInput},
};

pub use winit::event::{MouseButton, VirtualKeyCode, TouchPhase};
use rogalik_math::vectors::Vector2f;

#[derive(Clone, Copy, Debug)]
pub struct Touch {
    pub position: Vector2f,
    pub phase: TouchPhase
}

pub struct InputContext {
    keys_down: HashSet<VirtualKeyCode>,
    keys_pressed: HashSet<VirtualKeyCode>,
    keys_released: HashSet<VirtualKeyCode>,
    mouse_buttons_down: HashSet<MouseButton>,
    mouse_buttons_pressed: HashSet<MouseButton>,
    mouse_buttons_released: HashSet<MouseButton>,
    mouse_physical_position: Vector2f,
    // mouse_logical_position: Vector2f,
    touches: HashMap<u64, Touch>,
    touch_click: bool
}
impl InputContext {
    pub fn new() -> Self {
        Self {
            keys_down: HashSet::new(),
            keys_pressed: HashSet::new(),
            keys_released: HashSet::new(),
            mouse_buttons_down: HashSet::new(),
            mouse_buttons_pressed: HashSet::new(),
            mouse_buttons_released: HashSet::new(),
            mouse_physical_position: Vector2f::ZERO,
            // mouse_logical_position: Vector2f::ZERO,
            touches: HashMap::new(),
            touch_click: true
        }
    }
    pub fn clear(&mut self) {
        self.keys_pressed = HashSet::new();
        self.keys_released = HashSet::new();
        self.mouse_buttons_pressed = HashSet::new();
        self.mouse_buttons_released = HashSet::new();
        self.touches.retain(|_, t| t.phase != TouchPhase::Ended && t.phase != TouchPhase::Cancelled);
    }
    fn calculate_position(&self, position: PhysicalPosition<f64>, window_size: &PhysicalSize<u32>) -> Vector2f {
        Vector2f::new(
            position.x as f32,
            window_size.height as f32 - position.y as f32,
        )
    }
    pub fn handle_mouse_move(&mut self, position: PhysicalPosition<f64>, window_size: &PhysicalSize<u32>) {
        // let window_logical: LogicalSize<f32> = window_size.to_logical(scale);
        // let mouse_logical = position.to_logical(scale);
        self.mouse_physical_position = self.calculate_position(position, window_size);
        // self.mouse_logical_position = Vector2f::new(
        //     mouse_logical.x,
        //     window_logical.height - mouse_logical.y
        // );
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
    pub fn handle_touch(
        &mut self,
        id: u64,
        phase: TouchPhase,
        position: PhysicalPosition<f64>,
        window_size: &PhysicalSize<u32>,
    ) {
        self.touches.insert(id, Touch { phase, position: self.calculate_position(position, window_size) });

        if self.touch_click {
            match phase {
                TouchPhase::Started => {
                    self.handle_mouse_button(&MouseButton::Left, &ElementState::Pressed);
                    self.handle_mouse_move(position, window_size);
                },
                TouchPhase::Ended => self.handle_mouse_button(&MouseButton::Left, &ElementState::Released),
                _ => ()
            };
        }
    }
    pub fn get_mouse_physical_position(&self) -> Vector2f {
        self.mouse_physical_position
    }
    // pub fn get_mouse_logical_position(&self) -> Vector2f {
    //     self.mouse_logical_position
    // }
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
    pub fn get_touches(&self) -> &HashMap<u64, Touch> {
        &self.touches
    }
}