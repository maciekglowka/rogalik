use std::collections::HashMap;

use rogalik_common::{
    structs::{AssetId, AtlasPosition, ShaderId},
    AtlasParams, EngineError, FontParams, ResourceId, SpriteParams, TextureFiltering,
};
use rogalik_math::vectors::Vector2f;

use crate::{
    assets::WgpuAssets,
    structs::{MaterialId, Quad},
};

pub(crate) struct Font {
    pub(crate) charset: Vec<char>,
    charmap: CharMap,
    pub(crate) kind: FontKind,
    character_spacing: Option<f32>,
    line_spacing: Option<f32>,
    pub(crate) filtering: TextureFiltering,
    pub(crate) shader: Option<ResourceId<ShaderId>>,
}
impl Font {
    pub(crate) fn new_from_atlas(params: &FontParams, material_id: ResourceId<MaterialId>) -> Self {
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
            line_spacing: params.line_spacing,
            filtering: params.filtering,
            shader: params.shader,
        }
    }

    pub(crate) fn new_from_ttf(params: &FontParams, asset_id: ResourceId<AssetId>) -> Self {
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
                sizes: HashMap::new(),
            },
            character_spacing: params.character_spacing,
            line_spacing: params.line_spacing,
            filtering: params.filtering,
            shader: params.shader,
        }
    }
}

pub(crate) struct CharMetric {
    gap: f32,
}

pub(crate) struct LineMetrics {
    gap: f32,
}

pub(crate) struct FontSize {
    pub(crate) material_id: ResourceId<MaterialId>,
    pub(crate) char_metrics: Vec<CharMetric>,
    pub(crate) line_metrics: LineMetrics,
}

#[inline(always)]
pub(crate) fn text_key_size(size: f32) -> u32 {
    (10. * size) as u32
}

pub(crate) enum FontKind {
    /// Single atlas. Stores material id.
    Bitmap(ResourceId<MaterialId>),
    Ttf {
        /// Ttf source file.
        asset_id: ResourceId<AssetId>,
        /// Material ids by font size.
        sizes: HashMap<u32, FontSize>,
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
    pub(crate) material_id: ResourceId<MaterialId>,
}

pub(crate) struct TtfGlyphs {
    pub(crate) texture_data: Vec<u8>,
    pub(crate) texture_size: (u32, u32),
    pub(crate) atlas_params: AtlasParams,
    pub(crate) char_metrics: Vec<CharMetric>,
    pub(crate) line_metrics: LineMetrics,
}

/// Get base text layout from an existing atlas.
///
/// Does not check if an atlas for requested size exists.
pub(crate) fn get_text_layout(
    assets: &WgpuAssets,
    text: &str,
    font: &Font,
    size: f32,
) -> Result<TextLayout, EngineError> {
    calculate_layout(assets, [[text]], font, size, None)
}
/// Get base wrapped text box layout from an existing atlas.
///
/// Does not check if an atlas for requested size exists.
pub(crate) fn get_textbox_layout(
    assets: &WgpuAssets,
    text: &str,
    font: &Font,
    size: f32,
    max_width: f32,
) -> Result<TextLayout, EngineError> {
    let lines = text.split('\n').map(|s| s.split_inclusive(' '));
    calculate_layout(assets, lines, font, size, Some(max_width))
}

fn calculate_layout<T, U, S>(
    assets: &WgpuAssets,
    text: T,
    font: &Font,
    size: f32,
    max_width: Option<f32>,
) -> Result<TextLayout, EngineError>
where
    T: IntoIterator<Item = U>,
    U: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let (scale, material_id, char_metrics, line_metrics) = match &font.kind {
        FontKind::Bitmap(id) => (true, *id, None, None),
        FontKind::Ttf { sizes, .. } => {
            let size_entry = &sizes[&text_key_size(size)];
            (
                false,
                size_entry.material_id,
                Some(&size_entry.char_metrics),
                Some(&size_entry.line_metrics),
            )
        }
    };
    let material = assets
        .get_material(material_id)
        .ok_or(EngineError::ResourceNotFound)?;
    let atlas = material
        .atlas
        .as_ref()
        .ok_or(EngineError::GraphicsNotReady)?;

    // Text is anchored top-left (unlike regular sprites).
    let mut offset = Vector2f::new(0., -(size));
    let mut chars = Vec::new();

    let char_gap = font
        .character_spacing
        .map(|s| s * size)
        .unwrap_or(0.)
        .round();

    let mut line_spacing = (font.line_spacing.unwrap_or(1.) * size).round();
    if let Some(metrics) = line_metrics {
        line_spacing += metrics.gap;
    }

    // Keep the last gap for width calculation.
    let mut h_gap = char_gap;

    let base_h = size;

    let mut text_w: f32 = 0.;

    for line in text {
        let mut first_word = true;

        for word in line {
            let mut word_w = 0.;
            let word_idx = chars.len();

            for c in word.as_ref().chars() {
                let sprite_index = font.charmap.0.get(c as usize).copied().unwrap_or(0) as usize;
                let Some(entry) = atlas.get_entry(sprite_index) else {
                    continue;
                };

                let (w, h) = if scale {
                    let mut w = ((entry.w as f32 / entry.h as f32) * base_h).round();
                    if w.is_nan() {
                        w = 0.;
                    };
                    (w, base_h)
                } else {
                    (entry.w as f32, entry.h as f32)
                };

                chars.push(LayoutChar {
                    w,
                    h,
                    offset: Vector2f::new(offset.x + word_w, offset.y),
                    sprite_index,
                });

                if let Some(metrics) = char_metrics {
                    h_gap = char_gap + metrics[sprite_index].gap;
                }

                word_w += w + h_gap;
            }

            let mut shift = false;
            if let Some(max_width) = max_width {
                if !first_word && offset.x + word_w > max_width {
                    // Shift to a next line.
                    for c in chars[word_idx..].iter_mut() {
                        c.offset.x -= offset.x;
                        c.offset.y -= line_spacing;
                    }
                    shift = true;
                }
            }

            if shift {
                offset.x = word_w;
                offset.y -= line_spacing;
            } else {
                offset.x += word_w;
                text_w = text_w.max(offset.x);
            }
            first_word = false;
        }

        offset.y -= line_spacing;
        offset.x = 0.;
    }

    let line_gap = line_spacing - size;

    Ok(TextLayout {
        chars,
        width: text_w,
        height: -offset.y - size - line_gap,
        material_id,
    })
}

/// Get text sprites from an existing atlas.
///
/// Does not check if an atlas for requested size exists.
pub(crate) fn get_text_sprites(
    assets: &WgpuAssets,
    layout: &TextLayout,
    position: Vector2f,
    params: SpriteParams,
) -> Result<Vec<Quad>, EngineError> {
    let material = assets
        .get_material(layout.material_id)
        .ok_or(EngineError::ResourceNotFound)?;
    let atlas = material
        .atlas
        .as_ref()
        .ok_or(EngineError::GraphicsNotReady)?;

    Ok(layout
        .chars
        .iter()
        .map(|char| {
            atlas.get_sprite(
                char.sprite_index,
                (position + char.offset).round(),
                Vector2f::new(char.w, char.h),
                params,
            )
        })
        .collect())
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

/// Returns (buffer, atlas_params, (width, height)).
pub(crate) fn render_ttf_glyphs(
    charset: &[char],
    font_data: &[u8],
    size: f32,
) -> Result<TtfGlyphs, EngineError> {
    let ttf = fontdue::Font::from_bytes(font_data, fontdue::FontSettings::default())
        .inspect_err(|e| log::error!("Error while loading TTF: {e}"))
        .map_err(|_| EngineError::InvalidResource)?;

    let line_metrics = ttf
        .horizontal_line_metrics(size)
        .ok_or(EngineError::InvalidResource)?;

    // Use fixed height for simplicity (bit wasteful).
    let h = (line_metrics.ascent - line_metrics.descent) as usize;
    let h_step = h + 1;
    let baseline_offset = line_metrics.ascent as usize;

    let data = charset
        .iter()
        .map(|c| ttf.rasterize(*c, size))
        .collect::<Vec<_>>();

    // Make texture more-less square.
    let col_no = charset.len().isqrt();
    let row_no = charset.len() / col_no + 1;

    let texture_w = data
        .chunks(col_no)
        // Sum row characters + 1px gap.
        .map(|row| row.iter().map(|(m, _)| m.width).sum::<usize>() + row.len())
        .max()
        .ok_or(EngineError::InvalidResource)?;

    let texture_h = h_step * row_no;

    let mut texture_data = vec![0; 4 * texture_w * texture_h];
    let mut atlas_positions = vec![];
    let mut char_metrics = vec![];

    let mut col = 0;
    let mut offset_x = 0;
    let mut offset_y = 0;

    for (metrics, bitmap) in data {
        let top_offset = ((baseline_offset as i32) - metrics.ymin) as usize - metrics.height;

        // Blit
        if metrics.width > 0 {
            for (y, row) in bitmap.chunks(metrics.width).enumerate() {
                let base = 4 * ((y + offset_y + top_offset) * texture_w + offset_x);
                for (x, b) in row.iter().enumerate() {
                    let color = if *b == 0 { [0; 4] } else { [255, 255, 255, *b] };
                    texture_data[base + 4 * x..base + 4 * x + 4].copy_from_slice(&color);
                }
            }
        }

        atlas_positions.push(AtlasPosition::new(
            offset_x as u32,
            offset_y as u32,
            metrics.width as u32,
            h as u32,
        ));

        char_metrics.push(CharMetric {
            gap: metrics.advance_width - metrics.width as f32,
        });

        col += 1;
        offset_x += metrics.width + 1;

        if col == col_no {
            col = 0;
            offset_x = 0;
            offset_y += h_step;
        }
    }

    log::debug!("created font glyphs at size: {size}, texture ({texture_w} x {texture_h})");

    let atlas_params = AtlasParams::Free(atlas_positions);
    let line_metrics = LineMetrics {
        gap: line_metrics.line_gap,
    };

    Ok(TtfGlyphs {
        texture_data,
        texture_size: (texture_w as u32, texture_h as u32),
        atlas_params,
        char_metrics,
        line_metrics,
    })
}
