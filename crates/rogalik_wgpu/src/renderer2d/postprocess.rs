use crate::{
    assets::{
        postprocess::{get_texture_view, get_view_sampler, PostProcessEffect},
        WgpuAssets,
    },
    GraphicsError, PostProcessStage,
};

#[derive(Default)]
pub(crate) struct PostProcessRenderer {
    upscaled_views: Option<[wgpu::TextureView; 2]>,
    upscaled_bind_groups: Option<[wgpu::BindGroup; 2]>,
}
impl PostProcessRenderer {
    pub(crate) fn queue<'a>(&'a self, assets: &'a WgpuAssets) -> Vec<&'a PostProcessEffect> {
        // TODO add upscale pass and downscale effects
        assets
            .postprocess
            .values()
            .filter(|p| p.enabled && p.stage == PostProcessStage::Upscaled)
            .collect()
    }

    /// Returns first view in the sequence for sprite pass
    /// to render on.
    pub(crate) fn view(&self) -> Result<&wgpu::TextureView, GraphicsError> {
        // TODO return downscale view if pixel perfect
        Ok(&self
            .upscaled_views
            .as_ref()
            .ok_or(GraphicsError::NotReady)?[0])
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
            // FIXME downscale views.
            let target = match effects.peek() {
                None => output,
                Some(_) => &self
                    .upscaled_views
                    .as_ref()
                    .ok_or(GraphicsError::NotReady)?[output_idx],
            };

            let source = &self
                .upscaled_bind_groups
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
        let mut upscaled_views = vec![];
        let mut upscaled_bind_groups = vec![];

        for _ in 0..2 {
            upscaled_views.push(get_texture_view(w, h, device, texture_format));
            upscaled_bind_groups.push(Self::get_input_bind_group(
                input_bind_group_layout,
                upscaled_views.last().unwrap(),
                wgpu::FilterMode::Nearest,
                device,
            )?);
        }

        self.upscaled_views = Some(upscaled_views.try_into().unwrap());
        self.upscaled_bind_groups = Some(upscaled_bind_groups.try_into().unwrap());

        Ok(())
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
}
