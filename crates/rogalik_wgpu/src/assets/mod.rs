use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use rogalik_assets::{AssetStore, AssetStoreTrait};
use rogalik_common::{
    EngineError, MaterialParams, ResourceId, ShaderKind, TextureFiltering, TextureRepeat,
};

pub mod atlas;
mod bind_groups;
pub mod camera;
pub mod font;
pub mod material;
pub mod shader;

pub struct WgpuAssets {
    asset_store: Arc<Mutex<AssetStore>>,
    bind_group_layouts: HashMap<bind_groups::BindGroupKind, wgpu::BindGroupLayout>,
    pub cameras: HashMap<ResourceId, camera::Camera2D>,
    default_shader: ResourceId,
    // fonts: HashMap<String, font::Font>,
    pipeline_layouts: HashMap<ShaderKind, wgpu::PipelineLayout>,
    materials: HashMap<String, material::Material>,
    shaders: HashMap<ResourceId, shader::Shader>, // key == asset_id
}
impl WgpuAssets {
    pub fn new(asset_store: Arc<Mutex<AssetStore>>) -> Self {
        let default_shader = asset_store
            .lock()
            .expect("Can't acquire the asset store!")
            .from_bytes(include_bytes!("sprite_shader.wgsl"));
        Self {
            asset_store,
            bind_group_layouts: HashMap::new(),
            cameras: HashMap::new(),
            default_shader,
            materials: HashMap::new(),
            pipeline_layouts: HashMap::new(),
            shaders: HashMap::new(),
        }
    }
    pub fn create_wgpu_data(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_format: &wgpu::TextureFormat,
    ) -> Result<(), EngineError> {
        self.create_bind_group_layouts(device);
        self.create_pipeline_layouts(device)?;
        Ok(())
    }
    fn create_bind_group_layouts(&mut self, device: &wgpu::Device) {
        self.bind_group_layouts = bind_groups::get_bind_group_layouts(device);
    }
    fn create_pipeline_layouts(&mut self, device: &wgpu::Device) -> Result<(), EngineError> {
        self.pipeline_layouts = shader::get_pipeline_layouts(&self.bind_group_layouts, device)?;
        Ok(())
    }
    // pub fn build_wgpu_data(
    //     &mut self,
    //     device: &wgpu::Device,
    //     queue: &wgpu::Queue,
    //     bind_group_layout: &wgpu::BindGroupLayout,
    // ) {
    //     let mut store = self
    //         .asset_store
    //         .lock()
    //         .expect("Can't aqcuire the asset store!");
    //     for (name, material) in self.materials.iter_mut() {
    //         if material
    //             .build_wgpu_data(&mut store, device, queue, bind_group_layout)
    //             .is_err()
    //         {
    //             log::error!("Can't create material data for {}!", name);
    //         }
    //     }
    // }
    // pub fn get_texture(&self, id: ResourceId) -> Option<&texture::TextureData> {
    //     self.texture_bind_groups.get(&id)
    // }
    // pub fn get_textures(&self) -> &HashMap<ResourceId, texture::TextureData> {
    //     &self.texture_bind_groups
    // }
    pub fn create_shader(&mut self, kind: ShaderKind, path: &str) -> ResourceId {
        let shader_id = self.load_asset(path);
        let shader = shader::Shader::new(kind, shader_id);
        self.shaders.insert(shader_id, shader);
        shader_id
    }
    pub fn create_material(&mut self, name: &str, params: MaterialParams) {
        // let mut store = self
        //     .asset_store
        //     .lock()
        //     .expect("Can't aqcuire the asset store!");

        // let diffuse_id = store
        //     .load(params.diffuse_path)
        //     .expect(&format!("Can't load {}!", params.diffuse_path));
        let diffuse_id = self.load_asset(params.diffuse_path);

        let shader_id = if let Some(id) = params.shader {
            id
        } else {
            self.default_shader
        };

        let material = material::Material::new(diffuse_id, shader_id, params);
        self.materials.insert(name.to_string(), material);
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
    // pub fn get_atlas(&self, name: &str) -> Option<&atlas::SpriteAtlas> {
    //     self.atlases.get(name)
    // }
    // pub fn get_font(&self, name: &str) -> Option<&font::Font> {
    //     self.fonts.get(name)
    // }
    fn load_asset(&self, path: &str) -> ResourceId {
        let mut store = self
            .asset_store
            .lock()
            .expect("Can't acquire the asset store!");
        store.load(path).expect(&format!("Can't load {}!", path))
    }
}
