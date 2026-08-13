use image::{GenericImageView, ImageBuffer, Rgba};
use rogalik_common::{EngineError, ResourceId};

pub(crate) struct TextureData {
    /// Asset handle used for hot reloading.
    pub asset_id: Option<ResourceId>,
    pub buffer: ImageBuffer<Rgba<u8>, Vec<u8>>,
    pub dim: (u32, u32),
}
impl TextureData {
    pub(crate) fn from_file_bytes(
        asset_id: Option<ResourceId>,
        bytes: &[u8],
    ) -> Result<Self, EngineError> {
        let (rgba, dim) = TextureData::get_buffer_from_file(bytes)?;
        Ok(Self {
            dim,
            buffer: rgba,
            asset_id,
        })
    }
    pub(crate) fn from_raw(bytes: &[u8], width: u32, height: u32) -> Result<Self, EngineError> {
        let (rgba, dim) = TextureData::get_buffer_from_raw(bytes, width, height)?;
        Ok(Self {
            dim,
            buffer: rgba,
            asset_id: None,
        })
    }
    pub(crate) fn update_bytes(&mut self, bytes: &[u8]) {
        let Ok((rgba, dim)) = TextureData::get_buffer_from_file(bytes) else {
            return;
        };
        self.buffer = rgba;
        self.dim = dim;
    }
    pub(crate) fn to_wgpu_texture(
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

    fn get_buffer_from_file(
        bytes: &[u8],
    ) -> Result<(ImageBuffer<Rgba<u8>, Vec<u8>>, (u32, u32)), EngineError> {
        let img = image::load_from_memory(bytes)
            .inspect_err(|e| log::error!("Failed to load texture: {e}"))
            .map_err(|_| EngineError::InvalidResource)?;
        let rgba = img.to_rgba8();
        let dim = img.dimensions();

        Ok((rgba, dim))
    }
    /// Expects rgba.
    fn get_buffer_from_raw(
        bytes: &[u8],
        width: u32,
        height: u32,
    ) -> Result<(ImageBuffer<Rgba<u8>, Vec<u8>>, (u32, u32)), EngineError> {
        let buf = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(width, height, bytes.to_vec())
            .ok_or(EngineError::InvalidResource)?;
        Ok((buf, (width, height)))
    }
}
