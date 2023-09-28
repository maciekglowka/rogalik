
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct ResourceId(pub usize);

#[derive(Clone, Copy, Debug)]
pub struct Color(pub u8, pub u8, pub u8, pub u8);
impl Color {
    pub fn as_f32(&self) -> [f32; 4] {
        [
            self.0 as f32 / 255.,
            self.1 as f32 / 255.,
            self.2 as f32 / 255.,
            self.3 as f32 / 255.,
        ]
    }
}
impl Default for Color {
    fn default() -> Self {
        Self (255, 255, 255, 255)
    }
}

#[derive(Clone, Copy, Default)]
pub struct Params2d {
    pub color: Color,
    pub flip_x: bool,
    pub flip_y: bool
}
