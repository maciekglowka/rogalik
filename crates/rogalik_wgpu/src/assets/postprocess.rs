use std::collections::HashMap;
use wgpu::util::DeviceExt;

use rogalik_common::{EngineError, PostProcessParams, ResourceId};

use crate::assets::{texture::TextureData, WgpuAssets};
use crate::renderer2d::uniforms::UniformKind;
use crate::utils::{get_wgpu_address_mode, get_wgpu_filter_mode};

#[derive(Debug)]
pub struct PostProcessPass {
    pub shader_id: ResourceId,
    texture_id: ResourceId,
    bind_group: Option<wgpu::BindGroup>,
    uniform_buffer: Option<wgpu::Buffer>,
    uniform_data: PostProcessUniform,
    filter_mode: wgpu::FilterMode,
    address_mode: wgpu::AddressMode,
    view: Option<wgpu::TextureView>,
}
impl PostProcessPass {
    pub fn new(texture_id: ResourceId, params: PostProcessParams) -> Self {
        let address_mode = get_wgpu_address_mode(params.repeat);
        let filter_mode = get_wgpu_filter_mode(params.filtering);
        Self {
            shader_id: params.shader,
            bind_group: None,
            uniform_buffer: None,
            uniform_data: PostProcessUniform::new(),
            filter_mode,
            address_mode,
            texture_id,
            view: None,
        }
    }
    pub fn get_view(&self) -> Option<&wgpu::TextureView> {
        self.view.as_ref()
    }
    pub fn set_strength(&mut self, value: f32) {
        self.uniform_data.strength = value;
    }
    pub fn write_buffer(&self, queue: &wgpu::Queue) -> Result<(), EngineError> {
        queue.write_buffer(
            self.uniform_buffer
                .as_ref()
                .ok_or(EngineError::GraphicsNotReady)?,
            0,
            bytemuck::cast_slice(&[self.uniform_data]),
        );
        Ok(())
    }
    pub fn render(
        &self,
        assets: &WgpuAssets,
        encoder: &mut wgpu::CommandEncoder,
        output: &wgpu::TextureView,
        uniform_bind_groups: &HashMap<UniformKind, wgpu::BindGroup>,
    ) -> Result<(), EngineError> {
        let shader = assets
            .get_shader(self.shader_id)
            .ok_or(EngineError::GraphicsInternalError)?;
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("PostProcess"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &output,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            ..Default::default()
        });
        pass.set_pipeline(
            shader
                .pipeline
                .as_ref()
                .ok_or(EngineError::GraphicsNotReady)?,
        );
        pass.set_bind_group(
            0,
            self.bind_group
                .as_ref()
                .ok_or(EngineError::GraphicsNotReady)?,
            &[],
        );
        pass.set_bind_group(1, uniform_bind_groups.get(&UniformKind::Globals), &[]);
        pass.draw(0..3, 0..1);
        Ok(())
    }
    pub fn create_wgpu_data(
        &mut self,
        textures: &Vec<TextureData>,
        bind_group_layout: &wgpu::BindGroupLayout,
        w: u32,
        h: u32,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_format: &wgpu::TextureFormat,
    ) -> Result<(), EngineError> {
        let view = Self::get_texture_view(w, h, device, texture_format);
        let (bind_group, uniform_buffer) = Self::get_bind_group(
            textures,
            &self.uniform_data,
            bind_group_layout,
            &view,
            self.filter_mode,
            self.address_mode,
            self.texture_id,
            device,
            queue,
        )?;
        self.bind_group = Some(bind_group);
        self.uniform_buffer = Some(uniform_buffer);
        self.view = Some(view);
        Ok(())
    }
    fn get_bind_group(
        textures: &Vec<TextureData>,
        uniform_data: &PostProcessUniform,
        bind_group_layout: &wgpu::BindGroupLayout,
        view: &wgpu::TextureView,
        filter_mode: wgpu::FilterMode,
        address_mode: wgpu::AddressMode,
        texture_id: ResourceId,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<(wgpu::BindGroup, wgpu::Buffer), EngineError> {
        let texture = textures
            .get(texture_id.0)
            .ok_or(EngineError::ResourceNotFound)?;

        let texture_view = texture
            .to_wgpu_texture(device, queue, true)
            .create_view(&wgpu::TextureViewDescriptor::default());

        let texture_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: address_mode,
            address_mode_v: address_mode,
            address_mode_w: address_mode,
            mag_filter: filter_mode,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("PostProcess Uniform Buffer"),
            contents: bytemuck::cast_slice(&[*uniform_data]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        Ok((
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("PostProcess Bind Group"),
                layout: bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&Self::get_view_sampler(
                            filter_mode,
                            device,
                        )),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(&texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::Sampler(&texture_sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: uniform_buffer.as_entire_binding(),
                    },
                ],
            }),
            uniform_buffer,
        ))
    }
    fn get_texture_view(
        w: u32,
        h: u32,
        device: &wgpu::Device,
        texture_format: &wgpu::TextureFormat,
    ) -> wgpu::TextureView {
        let size = wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("PostProcess Texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: texture_format.clone(),
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        texture.create_view(&wgpu::TextureViewDescriptor::default())
    }
    fn get_view_sampler(filter_mode: wgpu::FilterMode, device: &wgpu::Device) -> wgpu::Sampler {
        device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: filter_mode,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        })
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::NoUninit, bytemuck::Zeroable)]
struct PostProcessUniform {
    pub strength: f32,
    _padding: [f32; 3], // for WASM
}
impl PostProcessUniform {
    pub fn new() -> Self {
        Self {
            strength: 1.0,
            ..Default::default()
        }
    }
}
