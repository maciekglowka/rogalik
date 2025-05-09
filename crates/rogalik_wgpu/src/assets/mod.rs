use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use rogalik_assets::{AssetContext, AssetState, AssetStore};
use rogalik_common::{AtlasParams, EngineError, MaterialParams, ResourceId, ShaderKind};
use rogalik_math::vectors::Vector2f;

pub mod atlas;
pub mod bind_groups;
pub mod camera;
pub mod font;
pub mod material;
pub mod shader;

use shader::BuiltInShader;

pub struct WgpuAssets {
    asset_store: Arc<Mutex<AssetStore>>,
    pub bind_group_layouts: HashMap<bind_groups::BindGroupLayoutKind, wgpu::BindGroupLayout>,
    pub(crate) built_in_shaders: HashMap<BuiltInShader, ResourceId>,
    cameras: Vec<camera::Camera2D>,
    default_shader: ResourceId,
    default_normal: ResourceId,
    pub pipeline_layouts: HashMap<ShaderKind, wgpu::PipelineLayout>,
    material_names: HashMap<String, ResourceId>, // lookup
    materials: Vec<material::Material>,
    shaders: Vec<shader::Shader>,
}
impl WgpuAssets {
    pub fn new(asset_store: Arc<Mutex<AssetStore>>) -> Self {
        let mut assets = Self {
            // perhaps this clone could be avoided?
            asset_store: asset_store.clone(),
            bind_group_layouts: HashMap::new(),
            built_in_shaders: HashMap::new(),
            cameras: Vec::new(),
            default_shader: ResourceId::default(),
            default_normal: ResourceId::default(),
            material_names: HashMap::new(),
            materials: Vec::new(),
            pipeline_layouts: HashMap::new(),
            shaders: Vec::new(),
        };
        assets.load_built_ins(asset_store);
        assets
    }
    /// Only (and immediately) to be called in a constructor
    fn load_built_ins(&mut self, asset_store: Arc<Mutex<AssetStore>>) {
        let mut store = asset_store.lock().expect("Can't acquire the asset store!");

        let unlit_asset_id = store.from_bytes(include_bytes!("include/sprite_unlit.wgsl"));
        let unlit_shader = shader::Shader::new(ShaderKind::Sprite, unlit_asset_id);
        self.default_shader = ResourceId(self.shaders.len());
        self.shaders.push(unlit_shader);
        self.built_in_shaders
            .insert(BuiltInShader::SpriteUnlit, self.default_shader);

        let upscale_asset_id = store.from_bytes(include_bytes!("include/sprite_pass_upscale.wgsl"));
        let upscale_shader = shader::Shader::new(ShaderKind::PostProcess, upscale_asset_id);
        let upscale_id = ResourceId(self.shaders.len());
        self.shaders.push(upscale_shader);
        self.built_in_shaders
            .insert(BuiltInShader::Upscale, upscale_id);

        self.default_normal = store.from_bytes(include_bytes!("include/default_normal.png"));
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
            .get(&bind_groups::BindGroupLayoutKind::Sprite)
            .ok_or(EngineError::GraphicsInternalError)?;

        for material in self.materials.iter_mut() {
            log::debug!("Creating material: {:?}", material);
            material.create_wgpu_data(&mut store, device, queue, material_layout)?;
        }

        for shader in self.shaders.iter_mut() {
            log::debug!("Creating shader: {:?}", shader);
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
            .get(&bind_groups::BindGroupLayoutKind::Sprite)
            .ok_or(EngineError::GraphicsInternalError)?;

        for material in self.materials.iter_mut() {
            let mut update = false;
            if let Some(asset) = store.get(material.diffuse_asset_id) {
                if asset.state == AssetState::Updated {
                    update = true;
                }
            }
            if let Some(asset) = store.get(material.normal_asset_id) {
                if asset.state == AssetState::Updated {
                    update = true;
                }
            }
            if update {
                if let Err(_) =
                    material.create_wgpu_data(&mut store, device, queue, material_layout)
                {
                    log::debug!("Material reload failed!");
                }
            }
            #[cfg(debug_assertions)]
            store.mark_read(material.diffuse_asset_id);
        }

        for shader in self.shaders.iter_mut() {
            if let Err(_) =
                shader.create_wgpu_data(&mut store, device, texture_format, &self.pipeline_layouts)
            {
                log::debug!("Shader reload failed!");
            }
            #[cfg(debug_assertions)]
            store.mark_read(shader.asset_id);
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
        let normal_id = if let Some(normal_path) = params.normal_path {
            self.load_asset(normal_path)
        } else {
            self.default_normal
        };

        let shader_id = if let Some(id) = params.shader {
            id
        } else {
            self.default_shader
        };

        let material = material::Material::new(diffuse_id, normal_id, shader_id, params);
        let material_id = self.get_next_material_id();
        self.material_names.insert(name.to_string(), material_id);
        self.materials.push(material);
    }
    pub fn create_camera(
        &mut self,
        vw: f32,
        vh: f32,
        rw: f32,
        rh: f32,
        scale: f32,
        target: Vector2f,
    ) -> ResourceId {
        let id = self.get_next_camera_id();
        let camera = camera::Camera2D::new(vw, vh, rw, rh, scale, target);
        self.cameras.push(camera);
        id
    }
    pub fn load_font(
        &mut self,
        name: &str,
        path: &str,
        rows: usize,
        cols: usize,
        padding: Option<(f32, f32)>,
    ) {
        let atlas = Some(AtlasParams {
            rows,
            cols,
            padding,
        });

        let params = MaterialParams {
            atlas,
            diffuse_path: path,
            ..Default::default()
        };
        self.create_material(name, params);
    }
    pub fn get_text_dimensions(&self, font: &str, text: &str, size: f32) -> Option<Vector2f> {
        let material = self.get_material(*self.get_material_id(font)?)?;
        let (w, h) = material.atlas?.get_sprite_size();
        let ratio = w / h;
        let l = text.chars().count();
        Some(size * Vector2f::new(ratio * l as f32, 1.))
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
