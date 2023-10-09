use std::collections::HashMap;

use rogalik_engine::ResourceId;

pub mod atlas;
pub mod font;
pub mod texture;

#[derive(Default)]
pub struct AssetStore {
    atlases: HashMap<String, atlas::SpriteAtlas>,
    fonts: HashMap<String, font::Font>,
    textures: Vec<texture::TextureData>
}
impl AssetStore {
    pub fn new() -> Self {
        Self::default()
    }
    fn load_texture(&mut self, bytes: &[u8]) -> ResourceId  {
        let id = ResourceId(self.textures.len());
        let texture = texture::TextureData::from_bytes(bytes);
        self.textures.push(texture);
        id
    }
    pub fn get_texture(&self, id: ResourceId) -> Option<&texture::TextureData> {
        self.textures.get(id.0)
    }
    pub fn get_textures(&self) -> &Vec<texture::TextureData> {
        &self.textures
    }
    pub fn load_atlas(
        &mut self,
        name: &str,
        bytes: &[u8],
        rows: usize,
        cols: usize,
        padding: Option<(f32, f32)>
    ) {
        let texture_id = self.load_texture(bytes);
        let atlas = atlas::SpriteAtlas::new(
            texture_id,
            self.textures[texture_id.0].dim,
            rows,
            cols,
            padding
        );
        self.atlases.insert(name.to_string(), atlas);
    }
    pub fn load_font(
        &mut self,
        name: &str,
        bytes: &[u8],
        rows: usize,
        cols: usize,
        padding: Option<(f32, f32)>
    ) {
        let texture_id = self.load_texture(bytes);
        let font = font::Font::new(
            texture_id,
            self.textures[texture_id.0].dim,
            rows,
            cols,
            padding
        );
        self.fonts.insert(name.to_string(), font);
    }
    pub fn get_atlas(&self, name: &str) -> Option<&atlas::SpriteAtlas> {
        self.atlases.get(name)
    }
    pub fn get_font(&self, name: &str) -> Option<&font::Font> {
        self.fonts.get(name)
    }
}
