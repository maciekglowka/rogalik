use std::collections::HashMap;

use crate::{assets::WgpuAssets, structs::Quad};
use rogalik_common::{EngineError, FontParams, ResourceId, SpriteParams};
use rogalik_math::vectors::Vector2f;

pub(crate) struct Font {
    charset: Vec<char>,
    charmap: CharMap,
    kind: FontKind,
    character_spacing: Option<f32>,
}
impl Font {
    pub(crate) fn new_from_atlas(params: &FontParams, material_id: ResourceId) -> Self {
        let charset = params
            .charset
            .map(|c| c.to_vec())
            .unwrap_or(get_default_charset());
        let charmap = get_charmap(&charset);

        Self {
            charset,
            charmap,
            kind: FontKind::Bitmap(material_id),
            character_spacing: params.character_spacing,
        }
    }

    pub(crate) fn new_from_ttf(params: &FontParams, asset_id: ResourceId) -> Self {
        let charset = params
            .charset
            .map(|c| c.to_vec())
            .unwrap_or(get_default_charset());
        let charmap = get_charmap(&charset);

        Self {
            charset,
            charmap,
            kind: FontKind::Ttf {
                asset_id,
                material_ids: HashMap::new(),
            },
            character_spacing: params.character_spacing,
        }
    }

    pub(crate) fn ensure_size(&mut self, assets: &mut WgpuAssets, size: u32) {
        match &mut self.kind {
            FontKind::Bitmap(_) => (),
            FontKind::Ttf {
                asset_id,
                material_ids,
            } => {
                if material_ids.contains_key(&size) {
                    return;
                }
            }
        }
    }
}

pub(crate) enum FontKind {
    /// Single atlas. Stores material id.
    Bitmap(ResourceId),
    Ttf {
        /// Ttf source file.
        asset_id: ResourceId,
        /// Material ids by font size.
        material_ids: HashMap<u32, ResourceId>,
    },
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
        FontKind::Ttf { material_ids, .. } => material_ids[&size],
    };
    let material = assets.get_material(material_id).unwrap();
    let atlas = material.atlas.as_ref().unwrap();

    // Text is anchored top-left (unlike regular sprites).
    let mut offset = Vector2f::new(0., -(size as f32));
    let mut chars = Vec::new();

    let gap = font
        .character_spacing
        .map(|s| s * size as f32)
        .unwrap_or(0.)
        .round();

    for c in text.chars() {
        let sprite_index = font.charmap.0.get(c as usize).copied().unwrap_or(0) as usize;
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

        offset += Vector2f::new(w + gap, 0.);
    }

    TextLayout {
        chars,
        width: offset.x - gap,
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

/// Returns standard ascii charser
/// ' ' (space) maps to 0.
fn get_default_charset() -> Vec<char> {
    (0x20..0x7f).map(|i| char::from_u32(i).unwrap()).collect()
}

/// Generate charmap from a charset.
///
/// Characters are expected to be placed on the atlas following the charset
/// order.
fn get_charmap(charset: &[char]) -> CharMap {
    let max = charset.iter().map(|c| *c as usize).max().unwrap_or(0);
    let mut mapping = vec![0; max + 1];
    for (i, c) in charset.iter().enumerate() {
        mapping[*c as usize] = i as u16
    }
    CharMap(mapping)
}

fn render_glyphs(charset: &[char], font_data: &[u8], size: f32) -> Result<(), EngineError> {
    let ttf = fontdue::Font::from_bytes(font_data, fontdue::FontSettings::default())
        .inspect_err(|e| log::error!("Error while loading TTF: {e}"))
        .map_err(|_| EngineError::InvalidResource)?;

    let line_metrics = ttf
        .horizontal_line_metrics(size)
        .ok_or(EngineError::InvalidResource)?;

    // Use fixed height for simplicity (bit wasteful).
    let h = line_metrics.ascent + line_metrics.descent;
    let baseline_offset = line_metrics.ascent;

    let data = charset
        .iter()
        .map(|c| ttf.rasterize(*c, size))
        .collect::<Vec<_>>();

    // Make texture more-less square.
    let col_no = charset.len().isqrt();
    let row_no = charset.len() / col_no + 1;

    let texture_w = data
        .chunks(col_no)
        .map(|row| row.iter().map(|(m, _)| m.width).sum::<usize>())
        .max()
        .ok_or(EngineError::InvalidResource)?;

    let texture_h = h as usize * row_no;
    let texture_data = vec![[0; 0; 0; 0;]; texture_w * texture_h];

    let rows = vec![];
    let mut col = 0;

    for chr in charset {
        let (metrics, bitmap) = ttf.rasterize(*chr, size);
        col += 1;
    }

    // println!("{metrics:?}");

    // // println!("{bitmap:?}");

    // for line in bitmap.chunks(metrics.width) {
    //     // println!("{line:?}");
    //     for c in line {
    //         let cc = if *c > 0 { "@" } else { " " };
    //         print!("{cc}");
    //     }
    //     println!("");
    // }

    Ok(())
}
