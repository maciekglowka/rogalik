// use std::cell::RefCell;

use rogalik_engine::traits::ResourceId;
use rogalik_math::vectors::Vector2F;

use crate::camera;
use crate::structs::Vertex;

mod atlas;
mod pass;
mod texture;

const QUAD_VERTICES: usize = 6;

pub struct SpriteManager {
    atlases: Vec<atlas::SpriteAtlas>,
    render_pass: pass::SpritePass,
    render_queue: Vec<RenderQuad>,
    textures: Vec<texture::Texture2D>
}
impl SpriteManager {
    pub fn new(
        device: &wgpu::Device,
        texture_format: &wgpu::TextureFormat
    ) -> Self {
        let render_pass = pass::SpritePass::new(
            wgpu::Color::BLUE,
            device,
            texture_format
        );
        Self {
            render_pass,
            atlases: Vec::new(),
            render_queue: Vec::new(),
            textures: Vec::new()
        }
    }
    fn load_texture(
        &mut self,
        bytes: &[u8],
        device: &wgpu::Device,
        queue: &wgpu::Queue
    ) -> usize {
        let id = self.textures.len();
        let texture = texture::Texture2D::from_bytes(
            bytes,
            device,
            queue,
            self.render_pass.get_bind_group_layout()
        );
        self.textures.push(texture);
        id
    }
    pub fn load_atlas(
        &mut self,
        bytes: &[u8],
        rows: usize,
        cols: usize,
        device: &wgpu::Device,
        queue: &wgpu::Queue
    ) -> ResourceId {
        let id = self.atlases.len();
        let texture_id = self.load_texture(bytes, device, queue);
        let atlas = atlas::SpriteAtlas::new(
            texture_id,
            rows,
            cols,
        );
        self.atlases.push(atlas);
        id
    }
    pub fn draw_indexed_sprite(
        &mut self,
        index: usize,
        atlas_id: ResourceId,
        camera_id: ResourceId,
        position: Vector2F,
        size: Vector2F
    ) {
        // TODO handle errors
        let quad = self.atlases[atlas_id].get_quad(index, camera_id, position, size);
        self.render_queue.push(quad);
    }
    pub fn render(
        &mut self,
        surface: &wgpu::Surface,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        cameras: &Vec<camera::Camera>
    ) {
        // self.draw_indexed_sprite(0, 0, 0, Vector2F::new(0., 0.), Vector2F::new(1.0, 1.0));
        self.draw_indexed_sprite(0, 0, 0, Vector2F::new(0., 0.), Vector2F::new(400.0, 300.0));
        // self.draw_indexed_sprite(0, 0, 0, Vector2F::new(-50., -50.), Vector2F::new(100.0, 100.0));
        // self.draw_indexed_sprite(0, 0, 0, Vector2F::new(50., 50.), Vector2F::new(100.0, 100.0));
        let _ = self.render_pass.render(
            cameras,
            &self.textures,
            &self.render_queue,
            surface,
            device,
            queue
        );
        self.render_queue.clear();
    }
}

pub struct RenderQuad {
    vertices: [Vertex; QUAD_VERTICES],
    texture_id: usize,
    camera_id: usize
}
