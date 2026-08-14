use std::collections::HashMap;

use crate::assets::{
    font::{get_text_layout, get_textbox_layout, text_key_size, TextLayout},
    WgpuAssets,
};

/// (font, text, 100 * size, max_width)
type Key = (String, String, u32, u32);

#[derive(Default)]
pub(crate) struct TextCache {
    entries: HashMap<Key, (TextLayout, bool)>,
}
impl TextCache {
    /// Retrieves cached entry and sets its `accessed` flag.
    ///
    /// If entry does not exists, it is created.
    /// If `max_width` is provided a textbox is assumed. Otherwise layouts a
    /// single line text.
    ///
    /// It is expected that font exists (and has prerendered glyphs at requested
    /// size).
    pub(crate) fn get(
        &mut self,
        assets: &WgpuAssets,
        font_name: &str,
        text: &str,
        size: f32,
        max_width: Option<f32>,
    ) -> &TextLayout {
        let key = (
            font_name.to_string(),
            text.to_string(),
            text_key_size(size),
            max_width.unwrap_or(0.) as u32,
        );

        let (layout, accessed) = self.entries.entry(key).or_insert_with(|| {
            let font = assets.get_font(font_name).unwrap();
            let layout = if let Some(max_width) = max_width {
                get_textbox_layout(assets, text, font, size, max_width)
            } else {
                get_text_layout(assets, text, font, size)
            };
            (layout, true)
        });
        *accessed = true;

        layout
    }
}
