use crate::structs::Vertex;
use rogalik_common::SpriteParams;
use rogalik_math::vectors::Vector2f;

pub fn get_text_sprites(
    text: &str,
    atlas: super::atlas::SpriteAtlas,
    position: Vector2f,
    size: f32,
    params: SpriteParams,
) -> Vec<([Vertex; 4], [u16; 6])> {
    // TODO take flip_h into account?
    let mut offset = Vector2f::new(0., 0.);
    let mut sprites = Vec::new();
    let (w, h) = atlas.get_sprite_size();
    let ratio = w / h;
    for c in text.chars() {
        sprites.push(atlas.get_sprite(
            c as usize,
            position + offset,
            Vector2f::new(ratio * size, size),
            params,
        ));
        offset += Vector2f::new(ratio * size, 0.);
    }
    sprites
}
