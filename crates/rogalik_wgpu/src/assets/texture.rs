use image::{GenericImageView, ImageBuffer, Rgba};
use rogalik_common::ResourceId;

pub(crate) struct TextureData {
    pub asset_id: ResourceId,
    pub buffer: ImageBuffer<Rgba<u8>, Vec<u8>>,
    pub dim: (u32, u32),
}
impl TextureData {
    pub fn from_bytes(asset_id: ResourceId, bytes: &[u8]) -> Self {
        let (rgba, dim) = TextureData::get_buffer(bytes);
        Self {
            dim,
            buffer: rgba,
            asset_id,
        }
    }
    pub fn update_bytes(&mut self, bytes: &[u8]) {
        let (rgba, dim) = TextureData::get_buffer(bytes);
        self.buffer = rgba;
        self.dim = dim;
    }
    pub fn to_wgpu_texture(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        is_data: bool,
    ) -> wgpu::Texture {
        let size = wgpu::Extent3d {
            width: self.dim.0,
            height: self.dim.1,
            depth_or_array_layers: 1,
        };
        let format = if is_data {
            wgpu::TextureFormat::Rgba8Unorm
        } else {
            wgpu::TextureFormat::Rgba8UnormSrgb
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("Texture"),
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &self.buffer,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * self.dim.0),
                rows_per_image: Some(self.dim.1),
            },
            size,
        );
        texture
    }

    fn get_buffer(bytes: &[u8]) -> (ImageBuffer<Rgba<u8>, Vec<u8>>, (u32, u32)) {
        let img = image::load_from_memory(bytes).expect("Failed to load texture!");
        let rgba = img.to_rgba8();
        let dim = img.dimensions();
        (rgba, dim)
    }
}
