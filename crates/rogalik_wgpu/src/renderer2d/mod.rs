use rogalik_common::{Color, EngineError, ResourceId, SpriteParams};
use rogalik_math::vectors::Vector2f;

use crate::assets::WgpuAssets;
use crate::structs::BindParams;

mod globals;
mod postprocess_pass;
mod sprite_pass;

const MAX_LIGHTS: u32 = 16;

pub struct Renderer2d {
    global: globals::GlobalUniform,
    sprite_pass: sprite_pass::SpritePass,
    post_process_passes: Vec<postprocess_pass::PostProcessPass>,
}
impl Renderer2d {
    pub fn new() -> Self {
        let sprite_pass = sprite_pass::SpritePass::new(wgpu::Color::BLACK);
        Self {
            global: globals::GlobalUniform::default(),
            sprite_pass,
            post_process_passes: Vec::new(),
        }
    }
    pub fn set_clear_color(&mut self, color: wgpu::Color) {
        self.sprite_pass.clear_color = color;
    }
    pub fn add_post_process(
        &mut self,
        shader_id: ResourceId,
        filtering: rogalik_common::TextureFiltering,
    ) {
        let filter_mode = match filtering {
            rogalik_common::TextureFiltering::Nearest => wgpu::FilterMode::Nearest,
            rogalik_common::TextureFiltering::Linear => wgpu::FilterMode::Linear,
        };
        self.post_process_passes
            .push(postprocess_pass::PostProcessPass::new(
                shader_id,
                filter_mode,
            ));
    }
    pub fn create_wgpu_data(
        &mut self,
        assets: &WgpuAssets,
        width: u32,
        height: u32,
        device: &wgpu::Device,
        texture_format: wgpu::TextureFormat,
    ) {
        for pass in self.post_process_passes.iter_mut() {
            let _ = pass.create_wgpu_data(assets, width, height, device, texture_format);
        }
    }
    pub fn draw_atlas_sprite(
        &mut self,
        assets: &WgpuAssets,
        index: usize,
        material_name: &str,
        camera_id: ResourceId,
        position: Vector2f,
        z_index: i32,
        size: Vector2f,
        params: SpriteParams,
    ) -> Result<(), EngineError> {
        let &material_id = assets
            .get_material_id(material_name)
            .ok_or(EngineError::ResourceNotFound)?;
        let material = assets
            .get_material(material_id)
            .ok_or(EngineError::ResourceNotFound)?;

        let bind_params = BindParams {
            camera_id,
            material_id,
            shader_id: material.shader_id,
        };

        if let Some(_) = params.slice {
            let s = material
                .atlas
                .ok_or(EngineError::InvalidResource)?
                .get_sliced_sprite(index, position, size, params);
            self.sprite_pass
                .add_to_queue(&s.0, &s.1, z_index, bind_params);
        } else {
            let s = material
                .atlas
                .ok_or(EngineError::InvalidResource)?
                .get_sprite(index, position, size, params);
            self.sprite_pass
                .add_to_queue(&s.0, &s.1, z_index, bind_params);
        };
        Ok(())
    }
    pub fn draw_text(
        &mut self,
        assets: &WgpuAssets,
        font: &str,
        text: &str,
        camera_id: ResourceId,
        position: Vector2f,
        z_index: i32,
        size: f32,
        params: SpriteParams,
    ) -> Result<(), EngineError> {
        // let font = assets.get_font(font).ok_or(EngineError::ResourceNotFound)?;
        // let bind_params = BindParams {
        //     camera_id,
        //     texture_id: font.atlas.texture_id,
        // };
        // for s in font.get_sprites(text, position, size, params) {
        //     self.add_to_queue(&s.0, &s.1, z_index, bind_params);
        // }
        Ok(())
    }
    pub fn set_ambient(&mut self, color: Color) {
        self.global.set_ambient(color);
    }
    pub fn add_light(
        &mut self,
        intensity: f32,
        color: Color,
        position: Vector2f,
    ) -> Result<(), EngineError> {
        self.global.add_light(intensity, color, position)
    }
    pub fn render(
        &mut self,
        assets: &WgpuAssets,
        time: f32,
        surface: &wgpu::Surface,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<(), EngineError> {
        self.global.set_time(time);
        let global_bind_group = self.global.get_bind_group(
            device,
            assets
                .bind_group_layouts
                .get(&crate::assets::bind_groups::BindGroupKind::Global)
                .ok_or(EngineError::GraphicsInternalError)?,
        );
        let output = surface
            .get_current_texture()
            .map_err(|_| EngineError::GraphicsNotReady)?;

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Renderer2D Encoder"),
        });

        let mut current_view = if let Some(pass) = self.post_process_passes.get(0) {
            pass.get_view().ok_or(EngineError::GraphicsNotReady)?
        } else {
            &view
        };

        self.sprite_pass.render(
            assets,
            &mut encoder,
            device,
            queue,
            &global_bind_group,
            current_view,
        )?;

        let mut post_processes = self.post_process_passes.iter().peekable();
        while let Some(pass) = post_processes.next() {
            current_view = if let Some(next_pass) = post_processes.peek() {
                next_pass.get_view().ok_or(EngineError::GraphicsNotReady)?
            } else {
                &view
            };
            pass.render(assets, &mut encoder, &current_view, &global_bind_group)?;
        }

        queue.submit(std::iter::once(encoder.finish()));
        output.present();
        self.global.frame_end();
        Ok(())
    }
}
