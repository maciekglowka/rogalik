use std::collections::HashMap;

use rogalik_assets::{AssetStore, AssetStoreTrait};
use rogalik_common::{EngineError, ResourceId, ShaderKind};

use super::bind_groups::BindGroupKind;
use crate::structs::Vertex;

pub fn get_pipeline_layouts(
    bind_group_layous: &HashMap<BindGroupKind, wgpu::BindGroupLayout>,
    device: &wgpu::Device,
) -> Result<HashMap<ShaderKind, wgpu::PipelineLayout>, EngineError> {
    Ok(HashMap::from_iter([
        (
            ShaderKind::Sprite,
            get_sprite_shader_pipeline_layout(bind_group_layous, device)?,
        ),
        (
            ShaderKind::PostProcess,
            get_post_process_pipeline_layout(bind_group_layous, device)?,
        ),
    ]))
}

pub struct Shader {
    asset_id: ResourceId,
    pub kind: ShaderKind,
    pub pipeline: Option<wgpu::RenderPipeline>,
}
impl Shader {
    pub fn new(kind: ShaderKind, asset_id: ResourceId) -> Self {
        Self {
            asset_id,
            kind,
            pipeline: None,
        }
    }
    pub fn create_wgpu_data(
        &mut self,
        asset_store: &mut AssetStore,
        device: &wgpu::Device,
        texture_format: &wgpu::TextureFormat,
        pipeline_layouts: &HashMap<ShaderKind, wgpu::PipelineLayout>,
    ) -> Result<(), EngineError> {
        let asset = asset_store
            .get(self.asset_id)
            .ok_or(EngineError::ResourceNotFound)?;

        let layout = pipeline_layouts
            .get(&self.kind)
            .ok_or(EngineError::GraphicsInternalError)?;

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(&format!("Shader {:?}", self.asset_id)),
            source: wgpu::ShaderSource::Wgsl(
                std::str::from_utf8(&asset.data)
                    .map_err(|_| EngineError::InvalidResource)?
                    .into(),
            ),
        });

        self.pipeline = match self.kind {
            ShaderKind::Sprite => Some(get_sprite_shader_pipeline(
                &shader,
                layout,
                texture_format,
                device,
            )),
            ShaderKind::PostProcess => Some(get_post_process_shader_pipeline(
                &shader,
                layout,
                texture_format,
                device,
            )),
        };

        Ok(())
    }
}

fn get_sprite_shader_pipeline(
    shader: &wgpu::ShaderModule,
    layout: &wgpu::PipelineLayout,
    texture_format: &wgpu::TextureFormat,
    device: &wgpu::Device,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Sprite pipeline"),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some("vs_main"),
            buffers: &[Vertex::layout()],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: *texture_format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            unclipped_depth: false,
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
        cache: None,
    })
}

fn get_post_process_shader_pipeline(
    shader: &wgpu::ShaderModule,
    layout: &wgpu::PipelineLayout,
    texture_format: &wgpu::TextureFormat,
    device: &wgpu::Device,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("PostProcess pipeline"),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some("vs_main"),
            buffers: &[],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: *texture_format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            unclipped_depth: false,
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
        cache: None,
    })
}

fn get_sprite_shader_pipeline_layout(
    bind_group_layous: &HashMap<BindGroupKind, wgpu::BindGroupLayout>,
    device: &wgpu::Device,
) -> Result<wgpu::PipelineLayout, EngineError> {
    Ok(
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Sprite Pipeline Layout"),
            bind_group_layouts: &[
                bind_group_layous
                    .get(&BindGroupKind::Sprite)
                    .ok_or(EngineError::GraphicsInternalError)?,
                bind_group_layous
                    .get(&BindGroupKind::Camera)
                    .ok_or(EngineError::GraphicsInternalError)?,
                bind_group_layous
                    .get(&BindGroupKind::Time)
                    .ok_or(EngineError::GraphicsInternalError)?,
            ],
            push_constant_ranges: &[],
        }),
    )
}

fn get_post_process_pipeline_layout(
    bind_group_layous: &HashMap<BindGroupKind, wgpu::BindGroupLayout>,
    device: &wgpu::Device,
) -> Result<wgpu::PipelineLayout, EngineError> {
    Ok(
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Postprocess Pipeline Layout"),
            bind_group_layouts: &[bind_group_layous
                .get(&BindGroupKind::PostProcess)
                .ok_or(EngineError::GraphicsInternalError)?],

            push_constant_ranges: &[],
        }),
    )
}
