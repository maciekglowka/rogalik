use rogalik_engine::{ResourceId, Params2d};
use rogalik_math::vectors::{Vector2i, Vector2f};

use crate::structs::Vertex;
use super::BindParams;
use super::atlas::SpriteAtlas;

pub struct Font {
    atlas: SpriteAtlas,
    character_size: Vector2f
}
impl Font {
    pub fn new(
        texture_id: ResourceId,
        texture_size: Vector2i,
        rows: usize,
        cols: usize,
        padding: Option<(f32, f32)>
    ) -> Self {
        let atlas = SpriteAtlas::new(
            texture_id,
            texture_size,
            rows,
            cols,
            padding
        );
        let (sp_w, sp_h) = super::atlas::sprite_pixel_size(texture_size, rows, cols, padding);
        let character_size = Vector2f::new(sp_w, sp_h);
        Self {
            atlas,
            character_size
        }
    }
    pub fn get_character_size(&self) -> Vector2f {
        self.character_size
    }
    pub fn get_sprites(
        &self,
        text: &str,
        camera_id: ResourceId,
        position: Vector2f,
        size: f32,
        params: Params2d
    ) -> Vec<([Vertex; 4], [u16; 6], BindParams)> {
        // TODO calculate font proportions
        // TODO take flip_h into account?
        let mut offset = Vector2f::new(0., 0.);
        let mut sprites = Vec::new();
        for c in text.chars() {
            sprites.push(
                self.atlas.get_sprite(c as usize, camera_id, position + offset, Vector2f::new(size, size), params)
            );
            offset += Vector2f::new(size, 0.);
        }
        sprites
    }
}