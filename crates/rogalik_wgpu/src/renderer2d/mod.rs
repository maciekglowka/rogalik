use rogalik_common::{Color, EngineError, PostProcessParams, ResourceId, SpriteParams};
use rogalik_math::vectors::Vector2f;

use crate::assets::{material::Material, postprocess::PostProcessPass, WgpuAssets};
use crate::structs::BindParams;

mod sprite_pass;
pub(crate) mod uniforms;

const MAX_LIGHTS: u32 = 16;

pub struct Renderer2d {
    sprite_pass: sprite_pass::SpritePass,
    rendering_resolution: Option<(u32, u32)>, // for pixel perfect renders
    upscale_pass: Option<PostProcessPass>,    // for pixel perfect renders
    uniforms: uniforms::Uniforms,
}
impl Renderer2d {
    pub fn new() -> Self {
        let sprite_pass = sprite_pass::SpritePass::new(wgpu::Color::BLACK);
        Self {
            sprite_pass,
            rendering_resolution: None,
            upscale_pass: None,
            uniforms: uniforms::Uniforms::default(),
        }
    }
    pub fn set_clear_color(&mut self, color: wgpu::Color) {
        self.sprite_pass.clear_color = color;
    }
    pub fn resize(&mut self, w: u32, h: u32) {
        self.uniforms.globals.viewport_size = [w, h];
        if self.rendering_resolution.is_none() {
            self.uniforms.globals.render_size = [w, h];
        }
    }
    pub fn set_rendering_resolution(
        &mut self,
        assets: &mut WgpuAssets,
        w: u32,
        h: u32,
    ) -> Result<(), EngineError> {
        self.rendering_resolution = Some((w, h));
        let shader_id = assets
            .builtin_shaders
            .get(&crate::BuiltInShader::Upscale)
            .ok_or(EngineError::GraphicsInternalError)?;
        self.upscale_pass = Some(PostProcessPass::new(
            assets.default_diffuse,
            PostProcessParams {
                shader: *shader_id,
                filtering: rogalik_common::TextureFiltering::Nearest,
                ..Default::default()
            },
        ));
        self.uniforms.globals.render_size = [w, h];
        Ok(())
    }
    pub fn create_upscale_pass(
        &mut self,
        assets: &WgpuAssets,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_format: &wgpu::TextureFormat,
    ) -> Result<(), EngineError> {
        if let Some((w, h)) = self.rendering_resolution {
            log::debug!("Creating upscale pass with w:{}, h:{}", w, h);
            let postprocess_layout = assets
                .bind_group_layouts
                .get(&crate::assets::bind_groups::BindGroupLayoutKind::PostProcess)
                .ok_or(EngineError::GraphicsInternalError)?;
            return self
                .upscale_pass
                .as_mut()
                .ok_or(EngineError::GraphicsInternalError)?
                .create_wgpu_data(
                    &assets.textures,
                    &postprocess_layout,
                    w,
                    h,
                    device,
                    queue,
                    texture_format,
                );
        }
        Ok(())
    }
    pub fn set_ambient(&mut self, color: Color) {
        self.uniforms.lights.set_ambient(color);
    }
    pub fn add_light(
        &mut self,
        strength: f32,
        color: Color,
        position: Vector2f,
    ) -> Result<(), EngineError> {
        self.uniforms.lights.add_light(strength, color, position)
    }
    pub fn create_wgpu_data(
        &mut self,
        assets: &WgpuAssets,
        width: u32,
        height: u32,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_format: &wgpu::TextureFormat,
    ) -> Result<(), EngineError> {
        log::debug!("Creating Renderer2d data with w:{}, h:{}", width, height);
        self.create_upscale_pass(assets, device, queue, texture_format)?;
        self.uniforms.create_wgpu_data(
            assets
                .bind_group_layouts
                .get(&crate::assets::bind_groups::BindGroupLayoutKind::Uniform)
                .ok_or(EngineError::GraphicsInternalError)?,
            device,
        );
        Ok(())
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
        let (material_id, material) = get_material(material_name, assets)?;

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
        let (material_id, material) = get_material(font, assets)?;
        let atlas = material.atlas.ok_or(EngineError::InvalidResource)?;

        let bind_params = BindParams {
            camera_id,
            material_id,
            shader_id: material.shader_id,
        };

        for s in crate::assets::font::get_text_sprites(text, atlas, position, size, params) {
            self.sprite_pass
                .add_to_queue(&s.0, &s.1, z_index, bind_params);
        }
        Ok(())
    }
    pub fn draw_mesh(
        &mut self,
        assets: &WgpuAssets,
        material_name: &str,
        camera_id: ResourceId,
        vertices: &[crate::structs::Vertex],
        indices: &[u16],
        z_index: i32,
    ) -> Result<(), EngineError> {
        let (material_id, material) = get_material(material_name, assets)?;

        let bind_params = BindParams {
            camera_id,
            material_id,
            shader_id: material.shader_id,
        };
        self.sprite_pass
            .add_to_queue(vertices, indices, z_index, bind_params);

        Ok(())
    }

    pub fn render(
        &mut self,
        assets: &WgpuAssets,
        time: f32,
        surface: &wgpu::Surface,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<(), EngineError> {
        for camera in assets.cameras.iter() {
            camera.write_buffer(queue)?;
        }

        self.uniforms.globals.time = time;
        self.uniforms.write_buffers(queue)?;

        let output = surface
            .get_current_texture()
            .map_err(|_| EngineError::GraphicsNotReady)?;

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Renderer2D Encoder"),
        });

        // TODO avoid allocation here?
        let mut post_process_queue = Vec::new();
        if let Some(pass) = &self.upscale_pass {
            post_process_queue.push(pass);
        }
        post_process_queue.extend(
            assets
                .postprocess
                .iter()
                .filter(|p| p.get_strength() > 0.001),
        );

        let mut current_view = if let Some(pass) = post_process_queue.get(0) {
            pass.get_view().ok_or(EngineError::GraphicsNotReady)?
        } else {
            &view
        };

        self.sprite_pass.render(
            assets,
            &mut encoder,
            device,
            &self.uniforms.bind_groups,
            current_view,
        )?;

        let mut post_processes = post_process_queue.iter().peekable();
        while let Some(pass) = post_processes.next() {
            current_view = if let Some(next_pass) = post_processes.peek() {
                next_pass.get_view().ok_or(EngineError::GraphicsNotReady)?
            } else {
                &view
            };
            pass.render(
                assets,
                &mut encoder,
                &current_view,
                &self.uniforms.bind_groups,
            )?;
        }

        queue.submit(std::iter::once(encoder.finish()));
        output.present();
        self.uniforms.lights.frame_end();
        Ok(())
    }
}

fn get_material<'a>(
    name: &str,
    assets: &'a WgpuAssets,
) -> Result<(ResourceId, &'a Material), EngineError> {
    let &material_id = assets
        .get_material_id(name)
        .ok_or(EngineError::ResourceNotFound)?;
    let material = assets
        .get_material(material_id)
        .ok_or(EngineError::ResourceNotFound)?;
    Ok((material_id, material))
}
