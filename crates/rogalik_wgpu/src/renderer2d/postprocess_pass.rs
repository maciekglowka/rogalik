use std::collections::HashMap;

use rogalik_common::{EngineError, ResourceId};

use super::uniforms::UniformKind;
use crate::assets::{bind_groups::BindGroupLayoutKind, WgpuAssets};

#[derive(Debug)]
pub struct PostProcessPass {
    pub shader_id: ResourceId,
    bind_group: Option<wgpu::BindGroup>,
    filter_mode: wgpu::FilterMode,
    view: Option<wgpu::TextureView>,
}
impl PostProcessPass {
    pub fn new(shader_id: ResourceId, filter_mode: wgpu::FilterMode) -> Self {
        Self {
            shader_id,
            bind_group: None,
            filter_mode,
            view: None,
        }
    }
    pub fn get_view(&self) -> Option<&wgpu::TextureView> {
        self.view.as_ref()
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
        assets: &WgpuAssets,
        w: u32,
        h: u32,
        device: &wgpu::Device,
        texture_format: wgpu::TextureFormat,
    ) -> Result<(), EngineError> {
        let view = Self::get_texture_view(w, h, device, texture_format);
        self.bind_group = Some(Self::get_bind_group(
            assets,
            &view,
            self.filter_mode,
            device,
        )?);
        self.view = Some(view);
        Ok(())
    }
    fn get_bind_group(
        assets: &WgpuAssets,
        view: &wgpu::TextureView,
        filter_mode: wgpu::FilterMode,
        device: &wgpu::Device,
    ) -> Result<wgpu::BindGroup, EngineError> {
        Ok(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("PostProcess Bind Group"),
            layout: assets
                .bind_group_layouts
                .get(&BindGroupLayoutKind::PostProcess)
                .ok_or(EngineError::GraphicsInternalError)?,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&Self::get_sampler(
                        filter_mode,
                        device,
                    )),
                },
            ],
        }))
    }
    fn get_texture_view(
        w: u32,
        h: u32,
        device: &wgpu::Device,
        texture_format: wgpu::TextureFormat,
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
            format: texture_format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        texture.create_view(&wgpu::TextureViewDescriptor::default())
    }
    fn get_sampler(filter_mode: wgpu::FilterMode, device: &wgpu::Device) -> wgpu::Sampler {
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
