use rogalik_common::{AtlasParams, EngineError, MaterialParams, ResourceId};

use super::{atlas::SpriteAtlas, texture::TextureData};
use crate::utils::{get_wgpu_address_mode, get_wgpu_filter_mode};

#[derive(Debug)]
pub struct Material {
    address_mode: wgpu::AddressMode,
    pub atlas: Option<SpriteAtlas>,
    atlas_params: Option<AtlasParams>,
    pub bind_group: Option<wgpu::BindGroup>,
    pub diffuse_texture_id: ResourceId,
    pub normal_texture_id: ResourceId,
    filter_mode: wgpu::FilterMode,
    pub shader_id: ResourceId,
}
impl Material {
    pub fn new(
        diffuse_texture_id: ResourceId,
        normal_texture_id: ResourceId,
        shader_id: ResourceId,
        material_params: MaterialParams,
    ) -> Self {
        let address_mode = get_wgpu_address_mode(material_params.repeat);
        let filter_mode = get_wgpu_filter_mode(material_params.filtering);
        Self {
            atlas: None,
            atlas_params: material_params.atlas,
            bind_group: None,
            diffuse_texture_id,
            normal_texture_id,
            shader_id,
            address_mode,
            filter_mode,
        }
    }
    pub fn create_wgpu_data(
        &mut self,
        textures: &Vec<TextureData>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Result<(), EngineError> {
        let diffuse_texture = textures
            .get(self.diffuse_texture_id.0)
            .ok_or(EngineError::ResourceNotFound)?;
        let normal_texture = textures
            .get(self.normal_texture_id.0)
            .ok_or(EngineError::ResourceNotFound)?;

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
        } else {
            // Create 1x1 atlas for compatibility.
            self.atlas = Some(SpriteAtlas::new(diffuse_texture.dim, 1, 1, None));
        }

        Ok(())
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
