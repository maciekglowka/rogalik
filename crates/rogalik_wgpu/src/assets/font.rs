use std::collections::HashMap;

use crate::{assets::WgpuAssets, structs::Vertex};
use rogalik_common::{FontParams, ResourceId, SpriteParams};
use rogalik_math::vectors::Vector2f;

pub(crate) struct Font {
    char_map: CharMap,
    kind: FontKind,
}
impl Font {
    pub(crate) fn new_from_atlas(params: &FontParams, material_id: ResourceId) -> Self {
        let char_map = if let Some(charset) = params.charset {
            get_char_map_from_charset(charset)
        } else {
            get_default_char_map()
        };

        Self {
            char_map,
            kind: FontKind::Bitmap(material_id),
        }
    }

    pub(crate) fn new_from_ttf(params: &FontParams) -> Self {
        let char_map = if let Some(charset) = params.charset {
            get_char_map_from_charset(charset)
        } else {
            get_default_char_map()
        };
        Self {
            char_map,
            kind: FontKind::Ttf(HashMap::new()),
        }
    }
}

pub(crate) enum FontKind {
    /// Single atlas. Stores material id.
    Bitmap(ResourceId),
    /// Material ids by font size.
    Ttf(HashMap<u32, ResourceId>),
}

/// Maps char (index) to sprite atlas index.
///
/// Invalid characters should be mapped to a default placeholder.
///
/// Uses u16 for mem optimization.
/// In case of high mem usage, consider hybrid vec / hashmap solution.
struct CharMap(Vec<u16>);

/// Get text sprites from an existing atlas.
///
/// Does not check if an atlas for requested size exists.
pub(crate) fn get_text_sprites(
    text: &str,
    font: &Font,
    size: u32,
    params: SpriteParams,
    assets: &WgpuAssets,
) {
    let material_id = match &font.kind {
        FontKind::Bitmap(id) => *id,
        FontKind::Ttf(map) => map[&size],
    };
    let material = assets.get_material(material_id).unwrap();
    let atlas = material.atlas.as_ref().unwrap();

    // let (w, h) = atlas.get_sprite_size();
    // let ratio = w / h;
}

pub fn get_text_sprites_(
    text: &str,
    atlas: super::atlas::SpriteAtlas,
    position: Vector2f,
    size: f32,
    params: SpriteParams,
) -> Vec<([Vertex; 4], [u16; 6])> {
    // TODO take flip_h into account?
    let mut offset = Vector2f::new(0., 0.);
    let mut sprites = Vec::new();
    // let (w, h) = atlas.get_sprite_size();
    // let ratio = w / h;
    let ratio = 1.;
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

/// Returns standard ascii mapping shifted
/// so ' ' (space) maps to 0.
///
/// Characters < 0x20 will map to space as a placeholder.
fn get_default_char_map() -> CharMap {
    let mapping = (0x00..0x7f).map(|c: u16| c.saturating_sub(0x20)).collect();
    CharMap(mapping)
}

/// Generate charmap from user defined charset.
///
/// Characters are expected to be placed on the atlas following the charset
/// order.
fn get_char_map_from_charset(charset: &[char]) -> CharMap {
    let max = charset.iter().map(|c| *c as usize).max().unwrap_or(0);
    let mut mapping = vec![0; max + 1];
    for (i, c) in charset.iter().enumerate() {
        mapping[*c as usize] = i as u16
    }
    CharMap(mapping)
}
