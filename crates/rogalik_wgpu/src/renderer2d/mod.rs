use rogalik_arena::Id;
use rogalik_math::vectors::Vector2f;

use crate::assets::{
    atlas::SpriteParams,
    camera::Camera2d,
    font::get_text_sprites,
    material::Material,
    postprocess::{PostProcessParams, PostProcessPass},
    texture::{TextureFiltering, TextureRepeat},
    WgpuAssets,
};
use crate::structs::{BindParams, Color};
use crate::GraphicsError;

mod sprite_pass;
mod text;
pub(crate) mod uniforms;

const MAX_LIGHTS: u32 = 16;

pub struct Renderer2d {
    sprite_pass: sprite_pass::SpritePass,
    #[cfg(all(dev_tools, feature = "capture"))]
    pub(crate) recorder: crate::tools::Recorder,
    rendering_resolution: Option<(u32, u32)>, // for pixel perfect renders
    upscale_pass: Option<PostProcessPass>,    // for pixel perfect renders
    uniforms: uniforms::Uniforms,
    text_cache: text::TextCache,
}
impl Renderer2d {
    pub fn new() -> Self {
        let sprite_pass = sprite_pass::SpritePass::new(wgpu::Color::BLACK);
        Self {
            sprite_pass,
            #[cfg(all(dev_tools, feature = "capture"))]
            recorder: crate::tools::Recorder::default(),
            rendering_resolution: None,
            upscale_pass: None,
            uniforms: uniforms::Uniforms::default(),
            text_cache: Default::default(),
        }
    }
    pub fn set_clear_color(&mut self, color: wgpu::Color) {
        self.sprite_pass.clear_color = color;
    }
    pub fn resize(&mut self, w: u32, h: u32) {
        self.uniforms.globals.viewport_size = [w as f32, h as f32];
        if self.rendering_resolution.is_none() {
            self.uniforms.globals.render_size = [w as f32, h as f32];
        }
    }
    pub fn set_rendering_resolution(
        &mut self,
        assets: &mut WgpuAssets,
        w: u32,
        h: u32,
    ) -> Result<(), GraphicsError> {
        self.rendering_resolution = Some((w, h));
        let shader_id = assets
            .builtin_shaders
            .get(&crate::BuiltInShader::Upscale)
            .ok_or(GraphicsError::InternalError)?;
        self.upscale_pass = Some(PostProcessPass::new(
            assets.default_diffuse.unwrap(),
            PostProcessParams {
                shader: *shader_id,
                filtering: TextureFiltering::Nearest,
                repeat: TextureRepeat::default(),
                texture: None,
            },
        ));
        self.uniforms.globals.render_size = [w as f32, h as f32];
        Ok(())
    }
    pub fn create_upscale_pass(
        &mut self,
        assets: &WgpuAssets,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_format: &wgpu::TextureFormat,
    ) -> Result<(), GraphicsError> {
        if let Some((w, h)) = self.rendering_resolution {
            log::debug!("Creating upscale pass with w:{}, h:{}", w, h);
            let postprocess_layout = assets
                .bind_group_layouts
                .get(&crate::assets::bind_groups::BindGroupLayoutKind::PostProcess)
                .ok_or(GraphicsError::InternalError)?;
            return self
                .upscale_pass
                .as_mut()
                .ok_or(GraphicsError::InternalError)?
                .create_wgpu_data(
                    &assets.textures,
                    postprocess_layout,
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
        position: Vector2f,
        radius: f32,
        color: Color,
        falloff: f32,
    ) -> Result<(), GraphicsError> {
        self.uniforms
            .lights
            .add_light(position, radius, color, falloff)
    }
    pub fn create_wgpu_data(
        &mut self,
        assets: &WgpuAssets,
        width: u32,
        height: u32,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_format: &wgpu::TextureFormat,
    ) -> Result<(), GraphicsError> {
        log::debug!("Creating Renderer2d data with w:{}, h:{}", width, height);
        self.create_upscale_pass(assets, device, queue, texture_format)?;
        self.uniforms.create_wgpu_data(
            assets
                .bind_group_layouts
                .get(&crate::assets::bind_groups::BindGroupLayoutKind::Uniform)
                .ok_or(GraphicsError::InternalError)?,
            device,
        );
        self.sprite_pass.create_wgpu_data();
        Ok(())
    }
    #[allow(clippy::too_many_arguments)]
    pub fn draw_atlas_sprite(
        &mut self,
        assets: &WgpuAssets,
        index: usize,
        material_name: &str,
        camera_id: Id<Camera2d>,
        position: Vector2f,
        z_index: i32,
        size: Vector2f,
        params: SpriteParams,
    ) -> Result<(), GraphicsError> {
        let (material_id, material) = get_material(material_name, assets)?;

        let bind_params = BindParams {
            camera_id,
            material_id,
            shader_id: material.shader_id,
        };

        if params.slice.is_some() {
            let s = material
                .atlas
                .as_ref()
                .ok_or(GraphicsError::MaterialError("atlas error".to_string()))?
                .get_sliced_sprite(index, position, size, params);
            self.sprite_pass
                .add_to_queue(&s.0, &s.1, z_index, bind_params);
        } else {
            let s = material
                .atlas
                .as_ref()
                .ok_or(GraphicsError::MaterialError("atlas error".to_string()))?
                .get_sprite(index, position, size, params);
            self.sprite_pass
                .add_to_queue(&s.0, &s.1, z_index, bind_params);
        };
        Ok(())
    }
    pub(crate) fn get_text_dimensions(
        &mut self,
        assets: &mut WgpuAssets,
        font: &str,
        text: &str,
        size: f32,
        max_width: Option<f32>,
    ) -> Option<Vector2f> {
        let layout = self
            .text_cache
            .get(assets, font, text, size, max_width)
            .ok()?;
        Some(Vector2f::new(layout.width, layout.height))
    }
    #[allow(clippy::too_many_arguments)]
    pub fn draw_text(
        &mut self,
        assets: &mut WgpuAssets,
        font_name: &str,
        text: &str,
        camera_id: Id<Camera2d>,
        position: Vector2f,
        z_index: i32,
        size: f32,
        max_width: Option<f32>,
        params: SpriteParams,
    ) -> Result<Vector2f, GraphicsError> {
        let layout = self
            .text_cache
            .get(assets, font_name, text, size, max_width)?;
        let sprites = get_text_sprites(assets, layout, position, params)?;

        let material =
            assets
                .get_material(layout.material_id)
                .ok_or(GraphicsError::ResourceNotFound(format!(
                    "material: {:?}",
                    layout.material_id
                )))?;

        let bind_params = BindParams {
            camera_id,
            material_id: layout.material_id,
            shader_id: material.shader_id,
        };

        for s in sprites {
            self.sprite_pass
                .add_to_queue(&s.0, &s.1, z_index, bind_params);
        }

        Ok(Vector2f::new(layout.width, layout.height))
    }
    pub fn draw_mesh(
        &mut self,
        assets: &WgpuAssets,
        material_name: &str,
        camera_id: Id<Camera2d>,
        vertices: &[crate::structs::Vertex],
        indices: &[u16],
        z_index: i32,
    ) -> Result<(), GraphicsError> {
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
    ) -> Result<(), GraphicsError> {
        for camera in assets.cameras.iter() {
            camera.write_buffer(queue)?;
        }

        self.uniforms.globals.time = time;
        self.uniforms.write_buffers(queue)?;

        let output = surface
            .get_current_texture()
            .map_err(|_| GraphicsError::NotReady)?;

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

        let mut current_view = if let Some(pass) = post_process_queue.first() {
            pass.get_view().ok_or(GraphicsError::NotReady)?
        } else {
            &view
        };

        self.sprite_pass.render(
            assets,
            &mut encoder,
            device,
            queue,
            &self.uniforms.bind_groups,
            current_view,
        )?;

        let mut post_processes = post_process_queue.iter().peekable();
        while let Some(pass) = post_processes.next() {
            current_view = if let Some(next_pass) = post_processes.peek() {
                next_pass.get_view().ok_or(GraphicsError::NotReady)?
            } else {
                &view
            };
            pass.render(
                assets,
                &mut encoder,
                current_view,
                &self.uniforms.bind_groups,
            )?;
        }

        queue.submit(std::iter::once(encoder.finish()));

        #[cfg(all(dev_tools, feature = "capture"))]
        {
            self.recorder.handle_queue(
                self.uniforms.globals.viewport_size[0] as u32,
                self.uniforms.globals.viewport_size[1] as u32,
                device,
                queue,
                &output,
            );
        }

        output.present();

        self.uniforms.lights.frame_end();
        self.text_cache.clean();

        Ok(())
    }
}

fn get_material<'a>(
    name: &str,
    assets: &'a WgpuAssets,
) -> Result<(Id<Material>, &'a Material), GraphicsError> {
    let material_id = assets
        .get_material_id(name)
        .ok_or(GraphicsError::ResourceNotFound(format!("material: {name}")))?;
    let material = assets
        .get_material(material_id)
        .ok_or(GraphicsError::ResourceNotFound(format!(
            "material: {material_id:?}"
        )))?;
    Ok((*material_id, material))
}
