use std::collections::HashMap;

use rogalik_arena::Id;
use rogalik_assets::{Asset, AssetContext, AssetStore};

use super::bind_groups::BindGroupLayoutKind;
use crate::{structs::Vertex, GraphicsError};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum ShaderKind {
    Sprite,
    PostProcess,
}

#[derive(Hash, Eq, PartialEq, Debug)]
pub(crate) enum BuiltInShader {
    SpriteUnlit,
    Upscale,
}

pub fn get_pipeline_layouts(
    bind_group_layouts: &HashMap<BindGroupLayoutKind, wgpu::BindGroupLayout>,
    device: &wgpu::Device,
) -> Result<HashMap<BindGroupLayoutKind, wgpu::PipelineLayout>, GraphicsError> {
    let mut output = HashMap::new();

    for kind in bind_group_layouts.keys() {
        match kind {
            BindGroupLayoutKind::Shader {
                kind: ShaderKind::Sprite,
                ..
            } => {
                output.insert(
                    *kind,
                    get_sprite_shader_pipeline_layout(bind_group_layouts, kind, device)?,
                );
            }
            BindGroupLayoutKind::Shader {
                kind: ShaderKind::PostProcess,
                ..
            } => {
                output.insert(
                    *kind,
                    get_post_process_pipeline_layout(bind_group_layouts, kind, device)?,
                );
            }
            _ => (),
        }
    }
    Ok(output)
}

#[derive(Debug)]
pub struct Shader {
    pub(crate) asset_id: Id<Asset>,
    pub(crate) kind: ShaderKind,
    pub(crate) pipeline: Option<wgpu::RenderPipeline>,
    /// This field should be set by the first material that uses the shader.
    pub(crate) bind_group_layout_kind: Option<BindGroupLayoutKind>,
}
impl Shader {
    pub(crate) fn new(kind: ShaderKind, asset_id: Id<Asset>) -> Self {
        Self {
            asset_id,
            kind,
            pipeline: None,
            bind_group_layout_kind: None,
        }
    }
    pub(crate) fn create_wgpu_data(
        &mut self,
        asset_store: &mut AssetStore,
        device: &wgpu::Device,
        texture_format: &wgpu::TextureFormat,
        pipeline_layouts: &HashMap<BindGroupLayoutKind, wgpu::PipelineLayout>,
    ) -> Result<(), GraphicsError> {
        let asset = asset_store.get(self.asset_id).ok_or_else(|| {
            GraphicsError::ResourceNotFound(format!("shader: {:?}", self.asset_id))
        })?;

        let Some(bind_group_layout_kind) = self.bind_group_layout_kind else {
            log::warn!("unused shader, asset id: {:?}", self.asset_id);
            return Ok(());
        };

        let layout = pipeline_layouts
            .get(&bind_group_layout_kind)
            .ok_or(GraphicsError::InternalError)?;

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(&format!("Shader {:?}", self.asset_id)),
            source: wgpu::ShaderSource::Wgsl(
                std::str::from_utf8(asset.data.get())
                    .map_err(|e| GraphicsError::ShaderError(e.to_string()))?
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
        log::debug!("Updated shader WGPU data.");

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
    bind_group_layouts: &HashMap<BindGroupLayoutKind, wgpu::BindGroupLayout>,
    sprite_layout_kind: &BindGroupLayoutKind,
    device: &wgpu::Device,
) -> Result<wgpu::PipelineLayout, GraphicsError> {
    Ok(
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Sprite Pipeline Layout"),
            bind_group_layouts: &[
                bind_group_layouts
                    .get(&BindGroupLayoutKind::Globals)
                    .ok_or(GraphicsError::InternalError)?,
                bind_group_layouts
                    .get(&BindGroupLayoutKind::Camera)
                    .ok_or(GraphicsError::InternalError)?,
                bind_group_layouts
                    .get(sprite_layout_kind)
                    .ok_or(GraphicsError::InternalError)?,
            ],
            push_constant_ranges: &[],
        }),
    )
}

fn get_post_process_pipeline_layout(
    bind_group_layouts: &HashMap<BindGroupLayoutKind, wgpu::BindGroupLayout>,
    pass_layout_kind: &BindGroupLayoutKind,
    device: &wgpu::Device,
) -> Result<wgpu::PipelineLayout, GraphicsError> {
    Ok(
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Postprocess Pipeline Layout"),
            bind_group_layouts: &[
                bind_group_layouts
                    .get(&BindGroupLayoutKind::Globals)
                    .ok_or(GraphicsError::InternalError)?,
                bind_group_layouts
                    .get(pass_layout_kind)
                    .ok_or(GraphicsError::InternalError)?,
            ],
            push_constant_ranges: &[],
        }),
    )
}
