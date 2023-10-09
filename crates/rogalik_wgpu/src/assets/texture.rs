use image::{GenericImageView, ImageBuffer, Rgba};

pub struct TextureData {
    pub buffer: ImageBuffer<Rgba<u8>, Vec<u8>>,
    pub dim: (u32, u32)
}
impl TextureData {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let img = image::load_from_memory(bytes)
            .expect("Failed to load texture!");
        let rgba = img.to_rgba8();
        let dim = img.dimensions();
        Self {
            dim,
            buffer: rgba
        }
    }
}