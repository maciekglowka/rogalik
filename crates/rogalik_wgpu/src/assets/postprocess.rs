use wgpu::util::DeviceExt;

use rogalik_arena::{Arena, Id};

use crate::assets::bind_groups::BindGroupLayoutKind;
use crate::utils::{get_wgpu_address_mode, get_wgpu_filter_mode};
use crate::{
    assets::{
        shader::Shader,
        texture::{TextureData, TextureFiltering, TextureRepeat},
        WgpuAssets,
    },
    GraphicsError,
};
use crate::{ShaderKind, ShaderUniform};

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub enum PostProcessStage {
    Downscaled,
    #[default]
    Upscaled,
}

#[derive(Clone)]
pub struct PostProcessParams {
    pub stage: PostProcessStage,
    pub enabled: bool,
    pub texture: Option<Id<TextureData>>,
    pub uniform: Option<ShaderUniform>,
    pub repeat: TextureRepeat,
    pub filtering: TextureFiltering,
}
impl Default for PostProcessParams {
    fn default() -> Self {
        Self {
            stage: PostProcessStage::default(),
            enabled: true,
            texture: None,
            uniform: None,
            repeat: TextureRepeat::Clamp,
            filtering: TextureFiltering::Nearest,
        }
    }
}

#[derive(Debug)]
pub(crate) struct PostProcessEffect {
    pub(crate) enabled: bool,
    pub(crate) stage: PostProcessStage,
    pub(crate) shader_id: Id<Shader>,
    pub(crate) bind_group_layout_kind: BindGroupLayoutKind,
    bind_group: Option<wgpu::BindGroup>,
    uniform_data: Option<ShaderUniform>,
    uniform_buffer: Option<wgpu::Buffer>,
    texture: Option<Id<TextureData>>,
    // It is a bit wasteful not to share texture views between
    // postprocess passes in case of many effects,
    // but makes implementation much simpler.
    view: Option<wgpu::TextureView>,
    filter_mode: wgpu::FilterMode,
    address_mode: wgpu::AddressMode,
}
impl PostProcessEffect {
    pub fn new(shader_id: Id<Shader>, params: PostProcessParams) -> Self {
        let address_mode = get_wgpu_address_mode(params.repeat);
        let filter_mode = get_wgpu_filter_mode(params.filtering);

        let bind_group_layout_kind = BindGroupLayoutKind::Shader {
            kind: ShaderKind::PostProcess,
            uniform: params.uniform.is_some(),
            textures: if params.texture.is_some() { 1 } else { 0 },
        };

        Self {
            enabled: params.enabled,
            shader_id,
            stage: params.stage,
            bind_group: None,
            bind_group_layout_kind,
            texture: params.texture,
            uniform_data: params.uniform,
            uniform_buffer: None,
            view: None,
            filter_mode,
            address_mode,
        }
    }
    /// Returns view of the first pass.
    pub fn get_view(&self) -> Option<&wgpu::TextureView> {
        self.view.as_ref()
    }
    // FIXME
    // pub fn set_strength(&mut self, value: f32) {
    //     self.uniform_data.strength = value;
    // }
    // pub fn get_strength(&self) -> f32 {
    //     self.uniform_data.strength
    // }
    // pub fn write_buffer(&self, queue: &wgpu::Queue) -> Result<(),
    // GraphicsError> {     queue.write_buffer(
    //         self.uniform_buffer
    //             .as_ref()
    //             .ok_or(GraphicsError::NotReady)?,
    //         0,
    //         bytemuck::cast_slice(&[self.uniform_data]),
    //     );
    //     Ok(())
    // }
    pub fn render(
        &self,
        assets: &WgpuAssets,
        encoder: &mut wgpu::CommandEncoder,
        output: &wgpu::TextureView,
        uniform_bind_group: &wgpu::BindGroup,
    ) -> Result<(), GraphicsError> {
        let shader = assets.get_shader(&self.shader_id).ok_or_else(|| {
            GraphicsError::ResourceNotFound(format!("shader: {:?}", self.shader_id))
        })?;

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("PostProcess Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: output,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            ..Default::default()
        });

        render_pass.set_pipeline(shader.pipeline.as_ref().ok_or(GraphicsError::NotReady)?);

        render_pass.set_bind_group(0, uniform_bind_group, &[]);
        render_pass.set_bind_group(
            1,
            self.bind_group.as_ref().ok_or(GraphicsError::NotReady)?,
            &[],
        );
        render_pass.draw(0..3, 0..1);

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn create_wgpu_data(
        &mut self,
        textures: &Arena<TextureData>,
        bind_group_layout: &wgpu::BindGroupLayout,
        w: u32,
        h: u32,
        device: &wgpu::Device,
        texture_format: &wgpu::TextureFormat,
    ) -> Result<(), GraphicsError> {
        let view = get_texture_view(w, h, device, texture_format);

        let texture = self.texture.as_ref().and_then(|t| textures.get(t));

        let (bind_group, uniform_buffer) = Self::get_bind_group(
            bind_group_layout,
            &view,
            self.filter_mode,
            self.address_mode,
            &texture,
            &self.uniform_data,
            device,
        )?;
        self.bind_group = Some(bind_group);
        self.uniform_buffer = uniform_buffer;
        self.view = Some(view);
        Ok(())
    }
    #[allow(clippy::too_many_arguments)]
    fn get_bind_group(
        bind_group_layout: &wgpu::BindGroupLayout,
        view: &wgpu::TextureView,
        filter_mode: wgpu::FilterMode,
        address_mode: wgpu::AddressMode,
        texture: &Option<&TextureData>,
        uniform_data: &Option<ShaderUniform>,
        device: &wgpu::Device,
    ) -> Result<(wgpu::BindGroup, Option<wgpu::Buffer>), GraphicsError> {
        let view_sampler = get_view_sampler(filter_mode, device);

        let mut entries = vec![
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Sampler(&view_sampler),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(view),
            },
        ];

        let texture = texture.as_ref().map(|t| {
            (
                device.create_sampler(&wgpu::SamplerDescriptor {
                    address_mode_u: address_mode,
                    address_mode_v: address_mode,
                    address_mode_w: address_mode,
                    mag_filter: filter_mode,
                    min_filter: wgpu::FilterMode::Nearest,
                    mipmap_filter: wgpu::FilterMode::Nearest,
                    ..Default::default()
                }),
                &t.view,
            )
        });

        if let Some((sampler, view)) = &texture {
            entries.push(wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Sampler(sampler),
            });
            entries.push(wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::TextureView(
                    view.as_ref().ok_or(GraphicsError::NotReady)?,
                ),
            });
        };

        let uniform_buffer = uniform_data.as_ref().map(|u| {
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("PostProcess Uniform Buffer"),
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
                label: Some("PostProcess Bind Group"),
                layout: bind_group_layout,
                entries: &entries,
            }),
            uniform_buffer,
        ))
    }
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
        format: *texture_format,
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

// #[repr(C)]
// #[derive(Copy, Clone, Debug, Default, bytemuck::NoUninit,
// bytemuck::Zeroable)] struct PostProcessUniform {
//     pub strength: f32,
//     _padding: [f32; 3], // for WASM
// }
// impl PostProcessUniform {
//     pub fn new() -> Self {
//         Self {
//             strength: 1.0,
//             ..Default::default()
//         }
//     }
// }
