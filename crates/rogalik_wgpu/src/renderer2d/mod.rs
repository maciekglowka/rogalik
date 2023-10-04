// use std::cell::RefCell;

use rogalik_engine::{ResourceId, Params2d};
use rogalik_math::vectors::Vector2f;

use crate::camera;
use crate::structs::Vertex;

mod atlas;
mod font;
mod sprite_pass;
mod texture;

pub struct Renderer2d {
    atlases: Vec<atlas::SpriteAtlas>,
    fonts: Vec<font::Font>,
    render_pass: sprite_pass::SpritePass,
    vertex_queue: Vec<Vertex>,
    triangle_queue: Vec<Triangle>,
    textures: Vec<texture::Texture2d>
}
impl Renderer2d {
    pub fn new(
        device: &wgpu::Device,
        texture_format: &wgpu::TextureFormat
    ) -> Self {
        let render_pass = sprite_pass::SpritePass::new(
            wgpu::Color::BLACK,
            device,
            texture_format
        );
        Self {
            render_pass,
            atlases: Vec::new(),
            fonts: Vec::new(),
            vertex_queue: Vec::new(),
            triangle_queue: Vec::new(),
            textures: Vec::new()
        }
    }
    pub fn set_clear_color(&mut self, color: wgpu::Color) {
        self.render_pass.clear_color = color;
    }
    fn load_texture(
        &mut self,
        bytes: &[u8],
        device: &wgpu::Device,
        queue: &wgpu::Queue
    ) -> ResourceId {
        let id = ResourceId(self.textures.len());
        let texture = texture::Texture2d::from_bytes(
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
        let id = ResourceId(self.atlases.len());
        let texture_id = self.load_texture(bytes, device, queue);
        let atlas = atlas::SpriteAtlas::new(
            texture_id,
            rows,
            cols,
        );
        self.atlases.push(atlas);
        id
    }
    pub fn load_font(
        &mut self,
        bytes: &[u8],
        rows: usize,
        cols: usize,
        device: &wgpu::Device,
        queue: &wgpu::Queue
    ) -> ResourceId {
        let id = ResourceId(self.fonts.len());
        let texture_id = self.load_texture(bytes, device, queue);
        let font = font::Font::new(
            texture_id,
            self.textures[texture_id.0].size(),
            rows,
            cols,
        );
        self.fonts.push(font);
        id
    }
    fn add_to_queue(&mut self, vertices: &[Vertex], indices: &[u16], params: BindParams) {
        // TODO add error if indices are not divisible by 3
        let offset = self.vertex_queue.len() as u16;
        self.vertex_queue.extend(vertices);
        self.triangle_queue.extend(
            indices.chunks(3)
                .map(|v| Triangle {
                    indices: [v[0] + offset, v[1] + offset, v[2] + offset],
                    params
                })
        )
    }
    pub fn draw_atlas_sprite(
        &mut self,
        index: usize,
        atlas_id: ResourceId,
        camera_id: ResourceId,
        position: Vector2f,
        size: Vector2f,
        params: Params2d
    ) {
        // TODO handle errors
        let s = self.atlases[atlas_id.0].get_sprite(index, camera_id, position, size, params);
        self.add_to_queue(&s.0, &s.1, s.2);
    }
    pub fn draw_text(
        &mut self,
        text: &str,
        font_id: ResourceId,
        camera_id: ResourceId,
        position: Vector2f,
        size: f32,
        params: Params2d
    ) {
        for s in self.fonts[font_id.0].get_sprites(text, camera_id, position, size, params) {
            self.add_to_queue(&s.0, &s.1, s.2);
        }
    }
    pub fn text_dimensions(
        &self,
        text: &str,
        font_id: ResourceId,
        size: f32
    ) -> Vector2f {
        let dim = self.fonts[font_id.0].get_character_size();
        let ratio = dim.x / dim.y;
        let l = text.chars().count();
        size * Vector2f::new(
            ratio * l as f32,
            1.
        )
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
            &self.textures,
            &self.vertex_queue,
            &self.triangle_queue,
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
    params: BindParams
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct BindParams {
    texture_id: ResourceId,
    camera_id: ResourceId
}