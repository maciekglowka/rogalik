use rogalik_engine::{ResourceId, Params2d};
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
        size: f32,
        params: Params2d
    ) -> Vec<([Vertex; 4], [u16; 6], BindParams)> {
        // TODO calculate font proportions
        // TODO take flip_h into account?
        let mut offset = Vector2F::new(0., 0.);
        let mut sprites = Vec::new();
        for c in text.chars() {
            sprites.push(
                self.atlas.get_sprite(c as usize, camera_id, position + offset, Vector2F::new(size, size), params)
            );
            offset += Vector2F::new(size, 0.);
        }
        sprites
    }
}