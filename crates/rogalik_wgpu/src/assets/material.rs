use rogalik_arena::{Arena, Id};
use wgpu::util::DeviceExt;

use super::{
    atlas::{AtlasParams, SpriteAtlas},
    shader::Shader,
    texture::{TextureData, TextureFiltering, TextureRepeat},
};
use crate::{
    assets::bind_groups::BindGroupLayoutKind,
    renderer2d::uniforms::ShaderUniform,
    utils::{get_wgpu_address_mode, get_wgpu_filter_mode},
    GraphicsError, ShaderKind,
};

#[derive(Clone, Default)]
pub struct MaterialParams {
    pub atlas: Option<AtlasParams>,
    pub textures: Vec<Id<TextureData>>,
    pub shader: Option<Id<Shader>>,
    pub repeat: TextureRepeat,
    pub filtering: TextureFiltering,
    pub uniform: Option<ShaderUniform>,
}

#[derive(Debug)]
pub struct Material {
    address_mode: wgpu::AddressMode,
    pub(crate) atlas: Option<SpriteAtlas>,
    atlas_params: Option<AtlasParams>,
    pub(crate) bind_group_layout_kind: BindGroupLayoutKind,
    pub(crate) bind_group: Option<wgpu::BindGroup>,
    pub(crate) textures: Vec<Id<TextureData>>,
    filter_mode: wgpu::FilterMode,
    pub(crate) shader_id: Id<Shader>,
    uniform_data: Option<ShaderUniform>,
    uniform_buffer: Option<wgpu::Buffer>,
}
impl Material {
    pub(crate) fn new(params: MaterialParams) -> Self {
        let address_mode = get_wgpu_address_mode(params.repeat);
        let filter_mode = get_wgpu_filter_mode(params.filtering);

        let bind_group_layout_kind = BindGroupLayoutKind::Shader {
            kind: ShaderKind::Sprite,
            uniform: params.uniform.is_some(),
            textures: params.textures.len(),
        };

        Self {
            atlas: None,
            atlas_params: params.atlas,
            bind_group_layout_kind,
            bind_group: None,
            textures: params.textures,
            shader_id: params.shader.unwrap(),
            address_mode,
            filter_mode,
            uniform_data: params.uniform,
            uniform_buffer: None,
        }
    }
    pub(crate) fn create_wgpu_data(
        &mut self,
        textures: &Arena<TextureData>,
        device: &wgpu::Device,
        bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Result<(), GraphicsError> {
        // TODO avoid vec by using iterator.
        let material_textures = self
            .textures
            .iter()
            .map(|t| textures.get(t))
            .collect::<Option<Vec<&TextureData>>>()
            .ok_or(GraphicsError::InternalError)?;

        let (bind_group, uniform_buffer) = get_material_bind_group(
            &material_textures,
            &self.uniform_data,
            device,
            bind_group_layout,
            self.address_mode,
            self.filter_mode,
        )?;
        self.bind_group = Some(bind_group);
        self.uniform_buffer = uniform_buffer;

        match (material_textures.get(0), &self.atlas_params) {
            (None, _) => (),
            (Some(t), None) => {
                // Create 1x1 atlas for compatibility.
                self.atlas = Some(SpriteAtlas::from_grid(t.dim, 1, 1, None));
            }
            (
                Some(t),
                Some(AtlasParams::Grid {
                    cols,
                    rows,
                    padding,
                }),
            ) => self.atlas = Some(SpriteAtlas::from_grid(t.dim, *rows, *cols, *padding)),
            (Some(t), Some(AtlasParams::Free(entries))) => {
                self.atlas = Some(SpriteAtlas::from_entries(entries, t.dim));
            }
        }

        Ok(())
    }
    pub(crate) fn update_uniform<T>(
        &mut self,
        data: T,
        queue: &wgpu::Queue,
    ) -> Result<(), GraphicsError>
    where
        T: bytemuck::NoUninit,
    {
        let Some(uniform) = self.uniform_data.as_mut() else {
            return Err(GraphicsError::MaterialError(
                "material does not support uniform buffers".to_string(),
            ));
        };

        uniform.update(data)?;

        // Update buffer if it has already been created.
        if let Some(buffer) = self.uniform_buffer.as_ref() {
            queue.write_buffer(buffer, 0, uniform.data());
        }

        Ok(())
    }
}

fn get_material_bind_group(
    textures: &[&TextureData],
    uniform_data: &Option<ShaderUniform>,
    device: &wgpu::Device,
    bind_group_layout: &wgpu::BindGroupLayout,
    address_mode: wgpu::AddressMode,
    filter_mode: wgpu::FilterMode,
) -> Result<(wgpu::BindGroup, Option<wgpu::Buffer>), GraphicsError> {
    let mut entries = vec![];

    // TODO reuse samplers between materials.
    let sampler = match textures.len() {
        0 => None,
        _ => Some(device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: address_mode,
            address_mode_v: address_mode,
            address_mode_w: address_mode,
            mag_filter: filter_mode,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        })),
    };

    // The order here has to match bind group layout for sprites.
    if let Some(sampler) = &sampler {
        entries.push(wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::Sampler(sampler),
        });
        for i in 0..textures.len() {
            let view = textures[i].view.as_ref().ok_or(GraphicsError::NotReady)?;
            entries.push(wgpu::BindGroupEntry {
                binding: i as u32 + 1,
                resource: wgpu::BindingResource::TextureView(view),
            });
        }
    }

    let uniform_buffer = uniform_data.as_ref().map(|u| {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Material Uniform Buffer"),
            contents: u.data(),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        })
    });

    if let Some(buffer) = &uniform_buffer {
        entries.push(wgpu::BindGroupEntry {
            binding: entries.len() as u32,
            resource: buffer.as_entire_binding(),
        })
    }

    Ok((
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: bind_group_layout,
            entries: &entries,
            label: Some("Sprite Diffuse Bind Group"),
        }),
        uniform_buffer,
    ))
}
