// use std::cell::RefCell;
use rogalik_engine::{ResourceId, Params2d, EngineError};
use rogalik_math::vectors::Vector2f;

use crate::assets::AssetStore;
use crate::camera;
use crate::structs::Vertex;

mod sprite_pass;
mod texture;

pub struct Renderer2d {
    render_pass: sprite_pass::SpritePass,
    vertex_queue: Vec<Vertex>,
    triangle_queue: Vec<Triangle>,
    texture_bind_groups: Vec<wgpu::BindGroup>,
}
impl Renderer2d {
    pub fn new(
        assets: &AssetStore,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_format: &wgpu::TextureFormat,
        clear_color: wgpu::Color
    ) -> Self {
        let render_pass = sprite_pass::SpritePass::new(
            clear_color,
            device,
            texture_format
        );

        let texture_bind_groups = assets.get_textures()
            .iter()
            .map(|t| texture::get_texture_bind_group(
                    t,
                    device,
                    queue,
                    &render_pass.bind_group_layout
                )
            )
            .collect();

        Self {
            render_pass,
            vertex_queue: Vec::new(),
            triangle_queue: Vec::new(),
            texture_bind_groups,
        }
    }
    pub fn set_clear_color(&mut self, color: wgpu::Color) {
        self.render_pass.clear_color = color;
    }
    fn add_to_queue(
        &mut self,
        vertices: &[Vertex],
        indices: &[u16],
        z_index: i32,
        params: BindParams
    ) {
        // TODO add error if indices are not divisible by 3
        let offset = self.vertex_queue.len() as u16;
        self.vertex_queue.extend(vertices);
        self.triangle_queue.extend(
            indices.chunks(3)
                .map(|v| Triangle {
                    indices: [v[0] + offset, v[1] + offset, v[2] + offset],
                    z_index,
                    params
                })
        )
    }
    pub fn draw_atlas_sprite(
        &mut self,
        assets: &AssetStore,
        index: usize,
        atlas: &str,
        camera_id: ResourceId,
        position: Vector2f,
        z_index: i32,
        size: Vector2f,
        params: Params2d
    ) -> Result<(), EngineError> {
        // TODO handle errors
        let atlas = assets.get_atlas(atlas).ok_or(EngineError::ResourceNotFound)?;
        let bind_params = BindParams { camera_id, texture_id: atlas.texture_id };

        if let Some(_) = params.slice {
            let s = atlas.get_sliced_sprite(index, position, size, params);
            self.add_to_queue(&s.0, &s.1, z_index, bind_params);
        } else {
            let s = atlas.get_sprite(index, camera_id, position, size, params);
            self.add_to_queue(&s.0, &s.1, z_index, bind_params);
        };
        Ok(())
    }
    pub fn draw_text(
        &mut self,
        assets: &AssetStore,
        font: &str,
        text: &str,
        camera_id: ResourceId,
        position: Vector2f,
        z_index: i32,
        size: f32,
        params: Params2d
    ) -> Result<(), EngineError> {
        let font = assets.get_font(font).ok_or(EngineError::ResourceNotFound)?;
        let bind_params = BindParams { camera_id, texture_id: font.atlas.texture_id };
        for s in font.get_sprites(text, camera_id, position, size, params) {
            self.add_to_queue(&s.0, &s.1, z_index, bind_params);
        };
        Ok(())
    }
    pub fn render(
        &mut self,
        surface: &wgpu::Surface,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        cameras: &Vec<camera::Camera2D>
    ) {
        let _ = self.render_pass.render(
            cameras,
            &self.texture_bind_groups,
            &self.vertex_queue,
            &mut self.triangle_queue,
            surface,
            device,
            queue
        );
        self.vertex_queue.clear();
        self.triangle_queue.clear();
    }
}

#[derive(Clone, Copy)]
pub struct Triangle {
    indices: [u16; 3],
    z_index: i32,
    params: BindParams
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct BindParams {
    texture_id: ResourceId,
    camera_id: ResourceId
}