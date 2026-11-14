use wgpu::util::DeviceExt;

use rogalik_arena::{Arena, Id};

use crate::utils::{get_wgpu_address_mode, get_wgpu_filter_mode};
use crate::ShaderUniform;
use crate::{
    assets::{
        shader::Shader,
        texture::{TextureData, TextureFiltering, TextureRepeat},
        WgpuAssets,
    },
    GraphicsError,
};

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub enum PostProcessStage {
    Downscaled,
    #[default]
    Upscaled,
}

#[derive(Clone)]
pub struct PassParams {
    pub shader: Id<Shader>,
    pub texture: Option<Id<TextureData>>,
    pub uniform: Option<ShaderUniform>,
}

#[derive(Clone)]
pub struct PostProcessParams {
    pub stage: PostProcessStage,
    pub enabled: bool,
    pub passes: Vec<PassParams>,
    pub repeat: TextureRepeat,
    pub filtering: TextureFiltering,
}
impl Default for PostProcessParams {
    fn default() -> Self {
        Self {
            stage: PostProcessStage::default(),
            enabled: true,
            passes: vec![],
            repeat: TextureRepeat::Clamp,
            filtering: TextureFiltering::Nearest,
        }
    }
}

#[derive(Debug)]
pub(crate) struct PostProcessPass {
    pub(crate) shader_id: Id<Shader>,
    bind_group: Option<wgpu::BindGroup>,
    uniform_data: Option<ShaderUniform>,
    uniform_buffer: Option<wgpu::Buffer>,
    texture: Option<Id<TextureData>>,
    prev_pass_view: Option<wgpu::TextureView>,
}
impl PostProcessPass {
    #[allow(clippy::too_many_arguments)]
    fn create_wgpu_data(
        &mut self,
        textures: &Arena<TextureData>,
        default_texture: Id<TextureData>,
        bind_group_layout: &wgpu::BindGroupLayout,
        has_prev_pass: bool,
        w: u32,
        h: u32,
        filter_mode: wgpu::FilterMode,
        address_mode: wgpu::AddressMode,
        device: &wgpu::Device,
        texture_format: &wgpu::TextureFormat,
    ) -> Result<(), GraphicsError> {
        let default_texture_view = &textures
            .get(&default_texture)
            .ok_or(GraphicsError::InternalError)?
            .view
            .as_ref()
            .ok_or(GraphicsError::NotReady("default texture"))?;

        let prev_pass_view = if has_prev_pass {
            Some(get_texture_view(w, h, device, texture_format))
        } else {
            None
        };

        let texture_view = self
            .texture
            .as_ref()
            .and_then(|t| textures.get(t))
            .and_then(|t| t.view.as_ref());

        let (bind_group, uniform_buffer) = Self::get_bind_group(
            bind_group_layout,
            prev_pass_view.as_ref().unwrap_or(default_texture_view),
            texture_view.unwrap_or(default_texture_view),
            filter_mode,
            address_mode,
            &self.uniform_data,
            device,
        )?;
        self.bind_group = Some(bind_group);
        self.uniform_buffer = uniform_buffer;
        self.prev_pass_view = prev_pass_view;
        Ok(())
    }
    #[allow(clippy::too_many_arguments)]
    fn get_bind_group(
        bind_group_layout: &wgpu::BindGroupLayout,
        prev_pass_view: &wgpu::TextureView,
        texture_view: &wgpu::TextureView,
        filter_mode: wgpu::FilterMode,
        address_mode: wgpu::AddressMode,
        uniform_data: &Option<ShaderUniform>,
        device: &wgpu::Device,
    ) -> Result<(wgpu::BindGroup, Option<wgpu::Buffer>), GraphicsError> {
        // FIXME Reuse sampler
        let view_sampler = get_view_sampler(filter_mode, device);
        let texture_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: address_mode,
            address_mode_v: address_mode,
            address_mode_w: address_mode,
            mag_filter: filter_mode,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let default_uniform = DefaultUniform::default();
        let default_buffer = bytemuck::bytes_of(&default_uniform);
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("PostProcess Pass Uniform Buffer"),
            contents: match uniform_data {
                Some(u) => u.data(),
                None => default_buffer,
            },
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        Ok((
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("PostProcess Pass Bind Group"),
                layout: bind_group_layout,
                entries: &[
                    // Prev pass
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Sampler(&view_sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(prev_pass_view),
                    },
                    // Pass custom texture
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(&texture_sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::TextureView(texture_view),
                    },
                    // Uniform
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: uniform_buffer.as_entire_binding(),
                    },
                ],
            }),
            // Only output `Some` if pass contains actual buffer.
            uniform_data.as_ref().map(|_| uniform_buffer),
        ))
    }
}

#[derive(Debug)]
pub(crate) struct PostProcessEffect {
    pub(crate) enabled: bool,
    pub(crate) stage: PostProcessStage,
    passes: Vec<PostProcessPass>,
    filter_mode: wgpu::FilterMode,
    address_mode: wgpu::AddressMode,
}
impl PostProcessEffect {
    pub(crate) fn new(params: PostProcessParams) -> Self {
        let address_mode = get_wgpu_address_mode(params.repeat);
        let filter_mode = get_wgpu_filter_mode(params.filtering);

        let passes = params
            .passes
            .into_iter()
            .map(|p| PostProcessPass {
                shader_id: p.shader,
                bind_group: None,
                uniform_data: p.uniform,
                uniform_buffer: None,
                texture: p.texture,
                prev_pass_view: None,
            })
            .collect();

        Self {
            enabled: params.enabled,
            stage: params.stage,
            passes,
            filter_mode,
            address_mode,
        }
    }
    /// Returns view of the first pass.
    // pub fn get_view(&self) -> Option<&wgpu::TextureView> {
    //     self.view.as_ref()
    // }
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
    pub(crate) fn render(
        &self,
        assets: &WgpuAssets,
        encoder: &mut wgpu::CommandEncoder,
        input: &wgpu::BindGroup,
        output: &wgpu::TextureView,
        uniform_bind_group: &wgpu::BindGroup,
    ) -> Result<(), GraphicsError> {
        let mut passes = self.passes.iter().peekable();

        while let Some(pass) = passes.next() {
            let target = match passes.peek() {
                None => output,
                Some(p) => p
                    .prev_pass_view
                    .as_ref()
                    .ok_or(GraphicsError::NotReady("previous postprocess pass view"))?,
            };

            let shader = assets.get_shader(&pass.shader_id).ok_or_else(|| {
                GraphicsError::ResourceNotFound(format!("shader: {:?}", pass.shader_id))
            })?;

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("PostProcess Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                ..Default::default()
            });

            render_pass.set_pipeline(
                shader
                    .pipeline
                    .as_ref()
                    .ok_or(GraphicsError::NotReady("postprocess shader pipeline"))?,
            );

            render_pass.set_bind_group(0, uniform_bind_group, &[]);
            render_pass.set_bind_group(1, input, &[]);
            render_pass.set_bind_group(
                2,
                pass.bind_group
                    .as_ref()
                    .ok_or(GraphicsError::NotReady("postprocess pass bind group"))?,
                &[],
            );
            render_pass.draw(0..3, 0..1);
        }

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn create_wgpu_data(
        &mut self,
        textures: &Arena<TextureData>,
        default_texture: Id<TextureData>,
        bind_group_layout: &wgpu::BindGroupLayout,
        w: u32,
        h: u32,
        device: &wgpu::Device,
        texture_format: &wgpu::TextureFormat,
    ) -> Result<(), GraphicsError> {
        for (i, pass) in self.passes.iter_mut().enumerate() {
            pass.create_wgpu_data(
                textures,
                default_texture,
                bind_group_layout,
                i > 0,
                w,
                h,
                self.filter_mode,
                self.address_mode,
                device,
                texture_format,
            )?;
        }
        Ok(())
    }
}

pub(crate) fn get_texture_view(
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
pub(crate) fn get_view_sampler(
    filter_mode: wgpu::FilterMode,
    device: &wgpu::Device,
) -> wgpu::Sampler {
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

/// Dummy uniform in case pass doesn't provide a custom type.
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::NoUninit, bytemuck::Zeroable)]
struct DefaultUniform {
    // FIXME verify if can be smaller
    _padding: [f32; 4],
}
