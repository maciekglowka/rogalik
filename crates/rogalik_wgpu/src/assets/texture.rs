use image::{GenericImageView, ImageBuffer, Rgba};

use rogalik_common::{TextureFiltering, TextureRepeat};

pub struct TextureData {
    pub buffer: ImageBuffer<Rgba<u8>, Vec<u8>>,
    pub dim: (u32, u32),
    pub address_mode: wgpu::AddressMode,
    pub filter_mode: wgpu::FilterMode,
}
impl TextureData {
    pub fn from_bytes(bytes: &[u8], filtering: TextureFiltering, repeat: TextureRepeat) -> Self {
        let img = image::load_from_memory(bytes).expect("Failed to load texture!");
        let rgba = img.to_rgba8();
        let dim = img.dimensions();
        let address_mode = match repeat {
            TextureRepeat::Clamp => wgpu::AddressMode::ClampToEdge,
            TextureRepeat::Repeat => wgpu::AddressMode::Repeat,
            TextureRepeat::MirrorRepeat => wgpu::AddressMode::MirrorRepeat,
        };
        let filter_mode = match filtering {
            TextureFiltering::Nearest => wgpu::FilterMode::Nearest,
            TextureFiltering::Linear => wgpu::FilterMode::Linear,
        };

        Self {
            dim,
            buffer: rgba,
            address_mode,
            filter_mode,
        }
    }
}
