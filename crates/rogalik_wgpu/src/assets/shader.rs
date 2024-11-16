use std::collections::HashMap;

use rogalik_assets::{AssetStore, AssetStoreTrait};
use rogalik_common::{EngineError, ResourceId, ShaderKind};

use super::bind_groups::BindGroupKind;
use crate::structs::Vertex;

pub fn get_pipeline_layouts(
    bind_group_layous: &HashMap<BindGroupKind, wgpu::BindGroupLayout>,
    device: &wgpu::Device,
) -> Result<HashMap<ShaderKind, wgpu::PipelineLayout>, EngineError> {
    Ok(HashMap::from_iter([(
        ShaderKind::Sprite,
        get_sprite_shader_pipeline_layout(bind_group_layous, device)?,
    )]))
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
        pipeline_layout: &wgpu::PipelineLayout,
    ) -> Result<(), EngineError> {
        let asset = asset_store
            .get(self.asset_id)
            .ok_or(EngineError::ResourceNotFound)?;

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(&format!("Shader {:?}", self.asset_id)),
            source: wgpu::ShaderSource::Wgsl(
                std::str::from_utf8(&asset.data)
                    .map_err(|_| EngineError::InvalidResource)?
                    .into(),
            ),
        });

        self.pipeline = Some(
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Sprite pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[Vertex::layout()],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: *texture_format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
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
            }),
        );
        Ok(())
    }
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
            ],
            push_constant_ranges: &[],
        }),
    )
}
