use crate::{
    assets::{
        bind_groups::BindGroupLayoutKind,
        postprocess::{get_texture_view, get_view_sampler, PostProcessEffect},
        WgpuAssets,
    },
    GraphicsError, PassParams, PostProcessParams, PostProcessStage,
};

#[derive(Default)]
pub(crate) struct PostProcessRenderer {
    /// Views used by viewport resolution shaders.
    upscaled_views: Option<[wgpu::TextureView; 2]>,
    /// Bind groups used by viewport resolution shaders.
    upscaled_bind_groups: Option<[wgpu::BindGroup; 2]>,
    /// Views used by render resolution shaders (if pixel perfect is enabled).
    downscaled_views: Option<[wgpu::TextureView; 2]>,
    /// Bind groups used by render resolution shaders (if pixel perfect is
    /// enabled).
    downscaled_bind_groups: Option<[wgpu::BindGroup; 2]>,
    /// Postprocessing effect used for upscaling when pixel perfect is enabled.
    upscale_effect: Option<PostProcessEffect>,
}
impl PostProcessRenderer {
    pub(crate) fn queue<'a>(&'a self, assets: &'a WgpuAssets) -> Vec<&'a PostProcessEffect> {
        let mut queue = vec![];

        if let Some(upscale) = &self.upscale_effect {
            queue.extend(
                assets
                    .postprocess
                    .values()
                    .filter(|p| p.enabled && p.stage == PostProcessStage::Downscaled),
            );
            queue.push(upscale);
        }

        queue.extend(
            assets
                .postprocess
                .values()
                .filter(|p| p.enabled && p.stage == PostProcessStage::Upscaled),
        );
        queue
    }

    /// Returns first view in the sequence for sprite pass
    /// to render on.
    pub(crate) fn view(&self) -> Result<&wgpu::TextureView, GraphicsError> {
        match self.upscale_effect {
            None => &self.upscaled_views,
            Some(_) => &self.downscaled_views,
        }
        .as_ref()
        .ok_or(GraphicsError::NotReady)
        .map(|v| &v[0])
    }

    /// This should only be called when:
    /// - there is at least one pass active
    /// - sprite pass has rendered to the initial view.
    pub(crate) fn render(
        &self,
        assets: &WgpuAssets,
        encoder: &mut wgpu::CommandEncoder,
        queue: &Vec<&PostProcessEffect>,
        output: &wgpu::TextureView,
        uniform_bind_group: &wgpu::BindGroup,
    ) -> Result<(), GraphicsError> {
        let mut effects = queue.iter().peekable();

        let mut input_idx = 0;
        let mut output_idx = 1;

        while let Some(effect) = effects.next() {
            let target = match effects.peek() {
                None => output,
                Some(PostProcessEffect {
                    stage: PostProcessStage::Downscaled,
                    ..
                }) => &self
                    .downscaled_views
                    .as_ref()
                    .ok_or(GraphicsError::NotReady)?[output_idx],
                Some(PostProcessEffect {
                    stage: PostProcessStage::Upscaled,
                    ..
                }) => &self
                    .upscaled_views
                    .as_ref()
                    .ok_or(GraphicsError::NotReady)?[output_idx],
            };

            let source = &match effect.stage {
                PostProcessStage::Downscaled => &self.downscaled_bind_groups,
                PostProcessStage::Upscaled => &self.upscaled_bind_groups,
            }
            .as_ref()
            .ok_or(GraphicsError::NotReady)?[input_idx];

            effect.render(assets, encoder, source, target, uniform_bind_group)?;

            std::mem::swap(&mut input_idx, &mut output_idx);
        }

        Ok(())
    }
    #[allow(clippy::too_many_arguments)]
    pub fn create_wgpu_data(
        &mut self,
        input_bind_group_layout: &wgpu::BindGroupLayout,
        w: u32,
        h: u32,
        device: &wgpu::Device,
        texture_format: &wgpu::TextureFormat,
    ) -> Result<(), GraphicsError> {
        let (views, bind_groups) =
            get_views_and_bind_groups(input_bind_group_layout, w, h, device, texture_format)?;
        self.upscaled_views = Some(views);
        self.upscaled_bind_groups = Some(bind_groups);

        Ok(())
    }

    pub(crate) fn create_downscale_wgpu_data(
        &mut self,
        assets: &WgpuAssets,
        w: u32,
        h: u32,
        device: &wgpu::Device,
        texture_format: &wgpu::TextureFormat,
    ) -> Result<(), GraphicsError> {
        // Create upscale pass
        let upscale_shader = assets
            .builtin_shaders
            .get(&crate::assets::shader::BuiltInShader::Upscale)
            .ok_or(GraphicsError::InternalError)?;

        let mut upscale_effect = PostProcessEffect::new(PostProcessParams {
            // TODO confirm
            stage: PostProcessStage::Downscaled,
            enabled: true,
            passes: vec![PassParams {
                shader: *upscale_shader,
                texture: None,
                uniform: None,
            }],
            repeat: crate::TextureRepeat::Clamp,
            filtering: crate::TextureFiltering::Nearest,
        });

        upscale_effect.create_wgpu_data(
            &assets.textures,
            assets.default_diffuse.ok_or(GraphicsError::InternalError)?,
            assets
                .bind_group_layouts
                .get(&BindGroupLayoutKind::PostProcessPass)
                .ok_or(GraphicsError::InternalError)?,
            w,
            h,
            device,
            texture_format,
        )?;

        self.upscale_effect = Some(upscale_effect);

        let input_bind_group_layout = assets
            .bind_group_layouts
            .get(&BindGroupLayoutKind::PostProcessInput)
            .ok_or(GraphicsError::InternalError)?;

        let (views, bind_groups) =
            get_views_and_bind_groups(input_bind_group_layout, w, h, device, texture_format)?;
        self.downscaled_views = Some(views);
        self.downscaled_bind_groups = Some(bind_groups);

        Ok(())
    }
}

fn get_input_bind_group(
    bind_group_layout: &wgpu::BindGroupLayout,
    scene_view: &wgpu::TextureView,
    filter_mode: wgpu::FilterMode,
    device: &wgpu::Device,
) -> Result<wgpu::BindGroup, GraphicsError> {
    // FIXME Reuse sampler

    let view_sampler = get_view_sampler(filter_mode, device);

    Ok(device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("PostProcess Input Bind Group"),
        layout: bind_group_layout,
        entries: &[
            // Scene Sampler
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Sampler(&view_sampler),
            },
            // Scene texture
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&scene_view),
            },
        ],
    }))
}

/// Creates sharde view + bind groups sets.
fn get_views_and_bind_groups(
    input_bind_group_layout: &wgpu::BindGroupLayout,
    w: u32,
    h: u32,
    device: &wgpu::Device,
    texture_format: &wgpu::TextureFormat,
) -> Result<([wgpu::TextureView; 2], [wgpu::BindGroup; 2]), GraphicsError> {
    let mut views = vec![];
    let mut bind_groups = vec![];

    for _ in 0..2 {
        views.push(get_texture_view(w, h, device, texture_format));
        bind_groups.push(get_input_bind_group(
            input_bind_group_layout,
            views.last().unwrap(),
            wgpu::FilterMode::Nearest,
            device,
        )?);
    }

    Ok((views.try_into().unwrap(), bind_groups.try_into().unwrap()))
}
