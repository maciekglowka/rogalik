use rogalik_common::{EngineError, ResourceId};
use std::collections::HashMap;
use wgpu::util::DeviceExt;

use crate::assets::{bind_groups::BindGroupKind, WgpuAssets};
use crate::structs::{BindParams, Triangle, Vertex};

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
    pub fn create_wgpu_data(
        &mut self,
        assets: &WgpuAssets,
        w: u32,
        h: u32,
        device: &wgpu::Device,
    ) -> Result<(), EngineError> {
        let view = Self::get_texture_view(w, h, device);
        self.bind_group = Some(Self::get_bind_group(
            assets,
            &view,
            self.filter_mode,
            device,
        )?);
        self.view = Some(view);
        Ok(())
    }
    // pub fn render(

    //     view: &wgpu::TextureView,
    // )
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
                .get(&BindGroupKind::PostProcess)
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
    fn get_texture_view(w: u32, h: u32, device: &wgpu::Device) -> wgpu::TextureView {
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
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
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
