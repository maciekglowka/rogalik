use rogalik_math::vectors::Vector2f;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, PartialOrd, Ord)]
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
    pub fn as_srgb(&self) -> [f32; 4] {
        let f = self.as_f32();
        [
            srgb_single(f[0]),
            srgb_single(f[1]),
            srgb_single(f[2]),
            f[3]
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
    pub flip_y: bool,
    pub rotate: Option<f32>,
    // slice size in px, base sprite size
    pub slice: Option<(usize, Vector2f)>,
}

#[inline(always)]
fn srgb_single(v: f32) -> f32 {
    ((v + 0.055) / 1.055).powf(2.4)
}