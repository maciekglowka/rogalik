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

// #[derive(Clone, Copy, Debug)]
// pub struct Font {
//     character_size: Vector2f,
// }
// impl Font {
//     pub fn new(
//         material: &Material,
//         texture_size: (u32, u32),
//         rows: usize,
//         cols: usize,
//         padding: Option<(f32, f32)>,
//     ) -> Self {
//         let (sp_w, sp_h) =
//             super::atlas::sprite_pixel_size(texture_size.0, texture_size.1, rows, cols, padding);
//         let character_size = Vector2f::new(sp_w, sp_h);
//         Self {
//             atlas,
//             character_size,
//         }
//     }
//     pub fn get_character_size(&self) -> Vector2f {
//         self.character_size
//     }
//     pub fn get_sprites(
//         &self,
//         text: &str,
//         position: Vector2f,
//         size: f32,
//         params: SpriteParams,
//     ) -> Vec<([Vertex; 4], [u16; 6])> {
//         // TODO take flip_h into account?
//         let mut offset = Vector2f::new(0., 0.);
//         let mut sprites = Vec::new();
//         let ratio = self.character_size.x / self.character_size.y;
//         for c in text.chars() {
//             sprites.push(self.atlas.get_sprite(
//                 c as usize,
//                 position + offset,
//                 Vector2f::new(ratio * size, size),
//                 params,
//             ));
//             offset += Vector2f::new(ratio * size, 0.);
//         }
//         sprites
//     }
//     pub fn text_dimensions(&self, text: &str, size: f32) -> Vector2f {
//         let dim = self.character_size;
//         let ratio = dim.x / dim.y;
//         let l = text.chars().count();
//         size * Vector2f::new(ratio * l as f32, 1.)
//     }
// }
