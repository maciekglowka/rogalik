use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use rogalik_assets::{AssetStore, AssetStoreTrait};
use rogalik_common::{MaterialParams, ResourceId, TextureFiltering, TextureRepeat};

pub mod atlas;
pub mod font;
pub mod texture;

#[derive(Default)]
pub struct WgpuAssets {
    asset_store: Arc<Mutex<AssetStore>>,
    atlases: HashMap<String, atlas::SpriteAtlas>,
    fonts: HashMap<String, font::Font>,
    // materials: HashMap<String, Material>,
    shader_pipelines: HashMap<ResourceId, wgpu::RenderPipeline>,
    texture_bind_groups: HashMap<ResourceId, wgpu::BindGroup>,
}
impl WgpuAssets {
    pub fn new(asset_store: Arc<Mutex<AssetStore>>) -> Self {
        Self {
            asset_store,
            ..Default::default()
        }
    }
    fn create_texture(
        &mut self,
        texture_id: ResourceId,
        filtering: TextureFiltering,
        repeat: TextureRepeat,
    ) {
        if self.texture_bind_groups.contains_key(&texture_id) {
            return;
        }
        self.set_texture_data(texture_id, filtering, repeat);
    }
    fn set_texture_data(
        &mut self,
        texture_id: ResourceId,
        filtering: TextureFiltering,
        repeat: TextureRepeat,
    ) {
        if let Ok(lock) = self.asset_store.lock() {
            let asset = lock.get(texture_id).expect("Invalid texture id!");
            // let texture = texture::TextureData::from_bytes(&asset.data, filtering, repeat);
            // self.textures.insert(texture_id, texture);
        }
    }
    // pub fn get_texture(&self, id: ResourceId) -> Option<&texture::TextureData> {
    //     self.texture_bind_groups.get(&id)
    // }
    // pub fn get_textures(&self) -> &HashMap<ResourceId, texture::TextureData> {
    //     &self.texture_bind_groups
    // }
    pub fn create_material(&mut self, name: &str, params: MaterialParams) {
        let diffuse = self
            .asset_store
            .lock()
            .expect("Can't aqcuire the asset store!")
            .load(params.diffuse)
            .expect(&format!("Can't load {}!", params.diffuse));
        // self.create_texture(material.color, material.filtering, material.repeat);
        // TODO create normal texture
        // TODO create shader

        // if let Some(atlas_data) = material.atlas {
        //     let atlas = atlas::SpriteAtlas::new(
        //         self.textures[&material.color].dim,
        //         atlas_data.rows,
        //         atlas_data.cols,
        //         atlas_data.padding,
        //     );
        //     self.atlases.insert(name.to_string(), atlas);
        // }
        // self.materials.insert(name.to_string(), material);
    }
    pub fn load_font(
        &mut self,
        name: &str,
        bytes: &[u8],
        rows: usize,
        cols: usize,
        padding: Option<(f32, f32)>,
    ) {
        // let texture_id = self.load_texture(bytes);
        // let font = font::Font::new(
        //     texture_id,
        //     self.textures[texture_id.0].dim,
        //     rows,
        //     cols,
        //     padding,
        // );
        // self.fonts.insert(name.to_string(), font);
    }
    // pub fn get_material(&self, name: &str) -> Option<&Material> {
    //     self.materials.get(name)
    // }
    pub fn get_atlas(&self, name: &str) -> Option<&atlas::SpriteAtlas> {
        self.atlases.get(name)
    }
    pub fn get_font(&self, name: &str) -> Option<&font::Font> {
        self.fonts.get(name)
    }
}
