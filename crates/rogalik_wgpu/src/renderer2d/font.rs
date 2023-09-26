use rogalik_engine::traits::ResourceId;
use rogalik_math::vectors::Vector2F;

use crate::structs::Vertex;
use super::BindParams;
use super::atlas::SpriteAtlas;

pub struct Font {
    atlas: SpriteAtlas
}
impl Font {
    pub fn new(
        texture_id: ResourceId,
        rows: usize,
        cols: usize,
    ) -> Self {
        let atlas = SpriteAtlas::new(texture_id, rows, cols);
        Self {
            atlas
        }
    }
    pub fn get_sprites(
        &self,
        text: &str,
        camera_id: ResourceId,
        position: Vector2F,
        size: Vector2F
    ) -> Vec<([Vertex; 4], [u16; 6], BindParams)> {
        let mut offset = Vector2F::new(0., 0.);
        let mut sprites = Vec::new();
        for c in text.chars() {
            sprites.push(
                self.atlas.get_sprite(c as usize, camera_id, position + offset, size)
            );
            offset += Vector2F::new(size.x, 0.);
        }
        sprites
    }
}