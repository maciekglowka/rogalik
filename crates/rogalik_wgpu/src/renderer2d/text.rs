use std::collections::HashMap;

use crate::assets::{
    font::{get_text_layout, TextLayout},
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
    ///
    /// It is expected that font exists (and has prerendered glyphs at requested
    /// size).
    pub(crate) fn get(
        &mut self,
        assets: &WgpuAssets,
        font_name: &str,
        text: &str,
        size: u32,
    ) -> &TextLayout {
        let key = (font_name.to_string(), text.to_string(), size);

        let (layout, accessed) = self.entries.entry(key).or_insert_with(|| {
            let font = assets.get_font(font_name).unwrap();
            let layout = get_text_layout(assets, text, font, size);
            (layout, true)
        });
        *accessed = true;

        layout
    }
}
