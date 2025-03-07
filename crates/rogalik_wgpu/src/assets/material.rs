use image::{GenericImageView, ImageBuffer, Rgba};

use rogalik_assets::{AssetContext, AssetStore};
use rogalik_common::{
    AtlasParams, EngineError, MaterialParams, ResourceId, TextureFiltering, TextureRepeat,
};

use super::atlas::SpriteAtlas;

#[derive(Debug)]
pub struct Material {
    address_mode: wgpu::AddressMode,
    pub atlas: Option<SpriteAtlas>,
    atlas_params: Option<AtlasParams>,
    pub bind_group: Option<wgpu::BindGroup>,
    pub diffuse_asset_id: ResourceId,
    pub normal_asset_id: ResourceId,
    filter_mode: wgpu::FilterMode,
    pub shader_id: ResourceId,
}
impl Material {
    pub fn new(
        diffuse_asset_id: ResourceId,
        normal_asset_id: ResourceId,
        shader_id: ResourceId,
        material_params: MaterialParams,
    ) -> Self {
        let address_mode = match material_params.repeat {
            TextureRepeat::Clamp => wgpu::AddressMode::ClampToEdge,
            TextureRepeat::Repeat => wgpu::AddressMode::Repeat,
            TextureRepeat::MirrorRepeat => wgpu::AddressMode::MirrorRepeat,
        };
        let filter_mode = match material_params.filtering {
            TextureFiltering::Nearest => wgpu::FilterMode::Nearest,
            TextureFiltering::Linear => wgpu::FilterMode::Linear,
        };
        Self {
            atlas: None,
            atlas_params: material_params.atlas,
            bind_group: None,
            diffuse_asset_id,
            normal_asset_id,
            shader_id,
            address_mode,
            filter_mode,
        }
    }
    pub fn create_wgpu_data(
        &mut self,
        asset_store: &mut AssetStore,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Result<(), EngineError> {
        let diffuse_asset = asset_store
            .get(self.diffuse_asset_id)
            .ok_or(EngineError::ResourceNotFound)?;
        let diffuse_texture = TextureData::from_bytes(diffuse_asset.data.get());

        let normal_asset = asset_store
            .get(self.normal_asset_id)
            .ok_or(EngineError::ResourceNotFound)?;
        let normal_texture = TextureData::from_bytes(normal_asset.data.get());

        self.bind_group = Some(get_material_bind_group(
            &diffuse_texture,
            &normal_texture,
            device,
            queue,
            bind_group_layout,
            self.address_mode,
            self.filter_mode,
        ));

        if let Some(atlas_params) = self.atlas_params {
            self.atlas = Some(SpriteAtlas::new(
                diffuse_texture.dim,
                atlas_params.rows,
                atlas_params.cols,
                atlas_params.padding,
            ))
        }

        Ok(())
    }
}

struct TextureData {
    pub buffer: ImageBuffer<Rgba<u8>, Vec<u8>>,
    pub dim: (u32, u32),
}
impl TextureData {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let img = image::load_from_memory(bytes).expect("Failed to load texture!");
        let rgba = img.to_rgba8();
        let dim = img.dimensions();

        Self { dim, buffer: rgba }
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
}

fn get_material_bind_group(
    diffuse_data: &TextureData,
    normal_data: &TextureData,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    bind_group_layout: &wgpu::BindGroupLayout,
    address_mode: wgpu::AddressMode,
    filter_mode: wgpu::FilterMode,
) -> wgpu::BindGroup {
    let diffuse_texture = diffuse_data.to_wgpu_texture(device, queue, false);
    let diff_tex_view = diffuse_texture.create_view(&wgpu::TextureViewDescriptor::default());
    let diff_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: address_mode,
        address_mode_v: address_mode,
        address_mode_w: address_mode,
        mag_filter: filter_mode,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });

    let normal_texture = normal_data.to_wgpu_texture(device, queue, true);
    let normal_tex_view = normal_texture.create_view(&wgpu::TextureViewDescriptor::default());
    let normal_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: address_mode,
        address_mode_v: address_mode,
        address_mode_w: address_mode,
        mag_filter: filter_mode,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });

    device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&diff_tex_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&diff_sampler),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(&normal_tex_view),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::Sampler(&normal_sampler),
            },
        ],
        label: Some("Sprite Diffuse Bind Group"),
    })
}
