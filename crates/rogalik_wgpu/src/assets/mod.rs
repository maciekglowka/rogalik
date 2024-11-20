use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use rogalik_assets::{AssetContext, AssetState, AssetStore};
use rogalik_common::{EngineError, MaterialParams, ResourceId, ShaderKind};
use rogalik_math::vectors::Vector2f;

pub mod atlas;
pub mod bind_groups;
pub mod camera;
pub mod font;
pub mod material;
pub mod shader;

pub struct WgpuAssets {
    asset_store: Arc<Mutex<AssetStore>>,
    pub bind_group_layouts: HashMap<bind_groups::BindGroupKind, wgpu::BindGroupLayout>,
    cameras: Vec<camera::Camera2D>,
    default_shader: ResourceId,
    // fonts: HashMap<String, font::Font>,
    pub pipeline_layouts: HashMap<ShaderKind, wgpu::PipelineLayout>,
    material_names: HashMap<String, ResourceId>, // lookup
    materials: Vec<material::Material>,
    shaders: Vec<shader::Shader>, // pub materials: HashMap<ResourceId, material::Material>, // key == diffuse_asset_id
                                  // pub shaders: HashMap<ResourceId, shader::Shader>, // key == asset_id
}
impl WgpuAssets {
    pub fn new(asset_store: Arc<Mutex<AssetStore>>) -> Self {
        let default_shader = ResourceId(0);
        let shader_asset_id = asset_store
            .lock()
            .expect("Can't acquire the asset store!")
            .from_bytes(include_bytes!("sprite_shader.wgsl"));
        let shader = shader::Shader::new(ShaderKind::Sprite, shader_asset_id);

        Self {
            asset_store,
            bind_group_layouts: HashMap::new(),
            cameras: Vec::new(),
            default_shader,
            material_names: HashMap::new(),
            materials: Vec::new(),
            pipeline_layouts: HashMap::new(),
            shaders: vec![shader],
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
        let mut store = self
            .asset_store
            .lock()
            .expect("Can't acquire the asset store!");

        let material_layout = self
            .bind_group_layouts
            .get(&bind_groups::BindGroupKind::Sprite)
            .ok_or(EngineError::GraphicsInternalError)?;

        for material in self.materials.iter_mut() {
            material.create_wgpu_data(&mut store, device, queue, material_layout)?;
        }

        for shader in self.shaders.iter_mut() {
            shader.create_wgpu_data(&mut store, device, texture_format, &self.pipeline_layouts)?;
        }
        Ok(())
    }
    pub fn update_assets(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_format: &wgpu::TextureFormat,
    ) -> Result<(), EngineError> {
        let mut store = self
            .asset_store
            .lock()
            .expect("Can't acquire the asset store!");

        let material_layout = self
            .bind_group_layouts
            .get(&bind_groups::BindGroupKind::Sprite)
            .ok_or(EngineError::GraphicsInternalError)?;

        for material in self.materials.iter_mut() {
            if let Some(asset) = store.get(material.diffuse_asset_id) {
                if asset.state == AssetState::Updated {
                    let _ = material.create_wgpu_data(&mut store, device, queue, material_layout);
                }
            }
            #[cfg(debug_assertions)]
            store.mark_read(material.diffuse_asset_id);
        }

        for shader in self.shaders.iter_mut() {
            shader.create_wgpu_data(&mut store, device, texture_format, &self.pipeline_layouts)?;
        }

        Ok(())
    }
    fn create_bind_group_layouts(&mut self, device: &wgpu::Device) {
        self.bind_group_layouts = bind_groups::get_bind_group_layouts(device);
    }
    fn create_pipeline_layouts(&mut self, device: &wgpu::Device) -> Result<(), EngineError> {
        self.pipeline_layouts = shader::get_pipeline_layouts(&self.bind_group_layouts, device)?;
        Ok(())
    }
    pub fn create_shader(&mut self, kind: ShaderKind, path: &str) -> ResourceId {
        let asset_id = self.load_asset(path);
        let shader = shader::Shader::new(kind, asset_id);
        let shader_id = self.get_next_shader_id();
        self.shaders.push(shader);
        shader_id
    }
    pub fn create_material(&mut self, name: &str, params: MaterialParams) {
        let diffuse_id = self.load_asset(params.diffuse_path);

        let shader_id = if let Some(id) = params.shader {
            id
        } else {
            self.default_shader
        };

        let material = material::Material::new(diffuse_id, shader_id, params);
        let material_id = self.get_next_material_id();
        self.material_names.insert(name.to_string(), material_id);
        self.materials.push(material);
    }
    pub fn create_camera(&mut self, w: f32, h: f32, scale: f32, target: Vector2f) -> ResourceId {
        let id = self.get_next_camera_id();
        let camera = camera::Camera2D::new(w as f32, h as f32, scale, target);
        self.cameras.push(camera);
        id
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
    pub fn get_material_id(&self, name: &str) -> Option<&ResourceId> {
        self.material_names.get(name)
    }
    pub fn get_material(&self, id: ResourceId) -> Option<&material::Material> {
        self.materials.get(id.0)
    }
    pub fn get_shader(&self, id: ResourceId) -> Option<&shader::Shader> {
        self.shaders.get(id.0)
    }
    pub fn get_camera(&self, id: ResourceId) -> Option<&camera::Camera2D> {
        self.cameras.get(id.0)
    }
    pub fn get_camera_mut(&mut self, id: ResourceId) -> Option<&mut camera::Camera2D> {
        self.cameras.get_mut(id.0)
    }
    pub fn iter_cameras(&self) -> impl Iterator<Item = &camera::Camera2D> {
        self.cameras.iter()
    }
    pub fn iter_cameras_mut(&mut self) -> impl Iterator<Item = &mut camera::Camera2D> {
        self.cameras.iter_mut()
    }
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
    fn get_next_shader_id(&self) -> ResourceId {
        ResourceId(self.shaders.len())
    }
    fn get_next_material_id(&self) -> ResourceId {
        ResourceId(self.materials.len())
    }
    fn get_next_camera_id(&self) -> ResourceId {
        ResourceId(self.cameras.len())
    }
}
