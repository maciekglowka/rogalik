use std::collections::HashMap;

use crate::assets::{
    font::{get_text_layout, get_textbox_layout, TextLayout},
    WgpuAssets,
};

/// (font, text, size)
type Key = (String, String, u32);

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
        size: u32,
        max_width: Option<u32>,
    ) -> &TextLayout {
        let key = (font_name.to_string(), text.to_string(), size);

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
