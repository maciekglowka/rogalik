use std::collections::HashMap;

use crate::{assets::WgpuAssets, structs::Quad};
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

#[derive(Debug)]
pub(crate) struct LayoutChar {
    w: f32,
    h: f32,
    offset: Vector2f,
    sprite_index: usize,
}
pub(crate) struct TextLayout {
    chars: Vec<LayoutChar>,
    pub(crate) width: f32,
    pub(crate) height: f32,
    pub(crate) material_id: ResourceId,
}
// impl TextSprites {
//     pub(crate) fn translated(&self, origin: Vector2f) -> impl Iterator<Item =
// Quad> + use<'_> {         let vert = move |mut v: Vertex| {
//             v.position[0] += origin.x;
//             v.position[1] += origin.y;
//             v
//         };

//         self.sprites
//             .iter()
//             .map(move |(v, t)| ([vert(v[0]), vert(v[1]), vert(v[2]),
// vert(v[3])], *t))     }
// }

/// Get base text layout from an existing atlas.
///
/// Does not check if an atlas for requested size exists.
pub(crate) fn get_text_layout(
    assets: &WgpuAssets,
    text: &str,
    font: &Font,
    size: u32,
) -> TextLayout {
    let material_id = match &font.kind {
        FontKind::Bitmap(id) => *id,
        FontKind::Ttf(map) => map[&size],
    };
    let material = assets.get_material(material_id).unwrap();
    let atlas = material.atlas.as_ref().unwrap();

    let mut offset = Vector2f::new(0., 0.);
    let mut chars = Vec::new();

    for c in text.chars() {
        let sprite_index = font.char_map.0.get(c as usize).copied().unwrap_or(0) as usize;
        let Some(entry) = atlas.get_entry(sprite_index) else {
            continue;
        };
        let h = size as f32;
        let w = (entry.w as f32 / entry.h as f32) * h;

        chars.push(LayoutChar {
            w,
            h,
            offset,
            sprite_index,
        });

        offset += Vector2f::new(w, 0.);
    }
    println!("{chars:?}");
    TextLayout {
        chars,
        width: offset.x,
        height: size as f32,
        material_id,
    }
}
/// Get text sprites from an existing atlas.
///
/// Does not check if an atlas for requested size exists.
pub(crate) fn get_text_sprites(
    assets: &WgpuAssets,
    layout: &TextLayout,
    position: Vector2f,
    params: SpriteParams,
) -> Vec<Quad> {
    let material = assets.get_material(layout.material_id).unwrap();
    let atlas = material.atlas.as_ref().unwrap();

    layout
        .chars
        .iter()
        .map(|char| {
            atlas.get_sprite(
                char.sprite_index,
                position + char.offset,
                Vector2f::new(char.w, char.h),
                params,
            )
        })
        .collect()
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
