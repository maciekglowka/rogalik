use image::{GenericImageView, ImageBuffer, Rgba};

use rogalik_arena::Id;
use rogalik_assets::{Asset, AssetContext, AssetStore};

use crate::GraphicsError;

type BufferOutput = ImageBuffer<Rgba<u8>, Vec<u8>>;

#[derive(Clone, Copy, Default)]
pub enum TextureRepeat {
    #[default]
    Clamp,
    Repeat,
    MirrorRepeat,
}

#[derive(Clone, Copy, Debug, Default)]
pub enum TextureFiltering {
    #[default]
    Nearest,
    Linear,
}

enum TextureSource {
    Asset(Id<Asset>),
    Raw(Vec<u8>),
}

pub struct TextureData {
    pub(crate) view: Option<wgpu::TextureView>,
    pub(crate) dim: (u32, u32),
    source: TextureSource,
    is_data: bool,
}
impl TextureData {
    pub(crate) fn from_asset(asset_id: Id<Asset>, is_data: bool) -> Result<Self, GraphicsError> {
        Ok(Self {
            source: TextureSource::Asset(asset_id),
            // Dim will be written on buffer read.
            dim: (0, 0),
            view: None,
            is_data,
        })
    }
    pub(crate) fn from_raw(
        bytes: &[u8],
        width: u32,
        height: u32,
        is_data: bool,
    ) -> Result<Self, GraphicsError> {
        Ok(Self {
            source: TextureSource::Raw(bytes.to_vec()),
            dim: (width, height),
            view: None,
            is_data,
        })
    }
    pub(crate) fn create_wgpu_data(
        &mut self,
        store: &mut AssetStore,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<(), GraphicsError> {
        let buffer = match &self.source {
            TextureSource::Asset(asset_id) => {
                let asset = store.get(*asset_id).ok_or_else(|| {
                    GraphicsError::ResourceNotFound(format!(
                        "texture asset not found: {asset_id:?}"
                    ))
                })?;
                let (buffer, dim) = Self::get_buffer_from_file(asset.data.get())?;
                self.dim = dim;
                buffer
            }
            TextureSource::Raw(bytes) => Self::get_buffer_from_raw(bytes, self.dim.0, self.dim.1)?,
        };

        let size = wgpu::Extent3d {
            width: self.dim.0,
            height: self.dim.1,
            depth_or_array_layers: 1,
        };
        let format = if self.is_data {
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
            &buffer,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * self.dim.0),
                rows_per_image: Some(self.dim.1),
            },
            size,
        );
        self.view = Some(texture.create_view(&wgpu::TextureViewDescriptor::default()));

        Ok(())
    }
    // pub(crate) fn to_wgpu_texture(
    //     &self,
    //     device: &wgpu::Device,
    //     queue: &wgpu::Queue,
    //     is_data: bool,
    // ) -> wgpu::Texture {
    //     let size = wgpu::Extent3d {
    //         width: self.dim.0,
    //         height: self.dim.1,
    //         depth_or_array_layers: 1,
    //     };
    //     let format = if is_data {
    //         wgpu::TextureFormat::Rgba8Unorm
    //     } else {
    //         wgpu::TextureFormat::Rgba8UnormSrgb
    //     };
    //     let texture = device.create_texture(&wgpu::TextureDescriptor {
    //         size,
    //         mip_level_count: 1,
    //         sample_count: 1,
    //         dimension: wgpu::TextureDimension::D2,
    //         format,
    //         usage: wgpu::TextureUsages::TEXTURE_BINDING |
    // wgpu::TextureUsages::COPY_DST,         label: Some("Texture"),
    //         view_formats: &[],
    //     });
    //     queue.write_texture(
    //         wgpu::ImageCopyTexture {
    //             texture: &texture,
    //             mip_level: 0,
    //             origin: wgpu::Origin3d::ZERO,
    //             aspect: wgpu::TextureAspect::All,
    //         },
    //         &self.buffer,
    //         wgpu::ImageDataLayout {
    //             offset: 0,
    //             bytes_per_row: Some(4 * self.dim.0),
    //             rows_per_image: Some(self.dim.1),
    //         },
    //         size,
    //     );
    //     texture
    // }
    //
    pub(crate) fn asset_id(&self) -> Option<Id<Asset>> {
        match &self.source {
            TextureSource::Asset(asset_id) => Some(*asset_id),
            TextureSource::Raw { .. } => None,
        }
    }

    fn get_buffer_from_file(bytes: &[u8]) -> Result<(BufferOutput, (u32, u32)), GraphicsError> {
        let img = image::load_from_memory(bytes)
            .inspect_err(|e| log::error!("Failed to load texture: {e}"))
            .map_err(|e| GraphicsError::TextureError(e.to_string()))?;
        let rgba = img.to_rgba8();
        let dim = img.dimensions();

        Ok((rgba, dim))
    }
    /// Expects rgba.
    fn get_buffer_from_raw(
        bytes: &[u8],
        width: u32,
        height: u32,
    ) -> Result<BufferOutput, GraphicsError> {
        let buf = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(width, height, bytes.to_vec())
            .ok_or_else(|| GraphicsError::TextureError("can't create image buffer".to_string()))?;
        Ok(buf)
    }
}
