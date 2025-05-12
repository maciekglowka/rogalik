use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

use rogalik_assets::{AssetContext, AssetState, AssetStore};
use rogalik_common::{
    AtlasParams, BuiltInShader, EngineError, MaterialParams, PostProcessParams, ResourceId,
    ShaderKind,
};
use rogalik_math::vectors::Vector2f;

pub mod atlas;
pub mod bind_groups;
pub mod camera;
pub mod font;
pub mod material;
pub mod postprocess;
pub mod shader;
mod texture;

pub struct WgpuAssets {
    asset_store: Arc<Mutex<AssetStore>>,
    pub bind_group_layouts: HashMap<bind_groups::BindGroupLayoutKind, wgpu::BindGroupLayout>,
    pub(crate) builtin_shaders: HashMap<BuiltInShader, ResourceId>,
    cameras: Vec<camera::Camera2D>,
    pub(crate) default_shader: ResourceId,
    pub(crate) default_normal: ResourceId,
    pub(crate) default_diffuse: ResourceId,
    pub pipeline_layouts: HashMap<ShaderKind, wgpu::PipelineLayout>,
    material_names: HashMap<String, ResourceId>, // lookup
    materials: Vec<material::Material>,
    pub(crate) postprocess: Vec<postprocess::PostProcessPass>,
    shaders: Vec<shader::Shader>,
    pub(crate) textures: Vec<texture::TextureData>,
}
impl WgpuAssets {
    pub fn new(asset_store: Arc<Mutex<AssetStore>>) -> Self {
        let mut assets = Self {
            // perhaps this clone could be avoided?
            asset_store: asset_store.clone(),
            bind_group_layouts: HashMap::new(),
            builtin_shaders: HashMap::new(),
            cameras: Vec::new(),
            default_shader: ResourceId::default(),
            default_normal: ResourceId::default(),
            default_diffuse: ResourceId::default(),
            material_names: HashMap::new(),
            materials: Vec::new(),
            pipeline_layouts: HashMap::new(),
            postprocess: Vec::new(),
            shaders: Vec::new(),
            textures: Vec::new(),
        };
        assets.load_builtins();
        assets
    }
    /// Only (and immediately) to be called in a constructor
    fn load_builtins(&mut self) {
        self.load_builtin_shader(
            include_bytes!("include/sprite_unlit.wgsl"),
            ShaderKind::Sprite,
            BuiltInShader::SpriteUnlit,
        );
        self.load_builtin_shader(
            include_bytes!("include/sprite_lit.wgsl"),
            ShaderKind::Sprite,
            BuiltInShader::SpriteLit,
        );
        self.load_builtin_shader(
            include_bytes!("include/sprite_pass_upscale.wgsl"),
            ShaderKind::PostProcess,
            BuiltInShader::Upscale,
        );
        self.load_builtin_shader(
            include_bytes!("include/lut.wgsl"),
            ShaderKind::PostProcess,
            BuiltInShader::Lut,
        );

        self.default_normal = self.texture_from_bytes(include_bytes!("include/default_normal.png"));
        self.default_diffuse = self.texture_from_bytes(include_bytes!("include/white.png"));
    }
    pub fn create_wgpu_data(
        &mut self,
        w: u32,
        h: u32,
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
            material.create_wgpu_data(&self.textures, device, queue, material_layout)?;
        }

        for shader in self.shaders.iter_mut() {
            log::debug!("Creating shader: {:?}", shader);
            shader.create_wgpu_data(&mut store, device, texture_format, &self.pipeline_layouts)?;
        }
        drop(store);
        self.update_postprocess_wgpu_data(w, h, device, queue, texture_format)?;

        Ok(())
    }
    pub fn update_postprocess_wgpu_data(
        &mut self,
        w: u32,
        h: u32,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_format: &wgpu::TextureFormat,
    ) -> Result<(), EngineError> {
        let postprocess_layout = self
            .bind_group_layouts
            .get(&bind_groups::BindGroupLayoutKind::PostProcess)
            .ok_or(EngineError::GraphicsInternalError)?;

        for pass in self.postprocess.iter_mut() {
            pass.create_wgpu_data(
                &self.textures,
                &postprocess_layout,
                w,
                h,
                device,
                queue,
                texture_format,
            )?;
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

        let mut updated_textures = HashSet::new();

        for (i, texture) in self.textures.iter_mut().enumerate() {
            if let Some(asset) = store.get(texture.asset_id) {
                if asset.state == AssetState::Updated {
                    log::debug!("Updating texture {}, Asset: {:?}", i, texture.asset_id);
                    texture.update_bytes(asset.data.get());
                    updated_textures.insert(i);

                    #[cfg(debug_assertions)]
                    store.mark_read(texture.asset_id);
                }
            }
        }

        for material in self.materials.iter_mut() {
            if updated_textures.contains(&material.diffuse_texture_id.0)
                || updated_textures.contains(&material.normal_texture_id.0)
            {
                log::debug!("Updating material {:?}", material);
                if let Err(_) =
                    material.create_wgpu_data(&self.textures, device, queue, material_layout)
                {
                    log::error!("Material reload failed!");
                }
            }
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
        let diffuse_id = params.diffuse_texture.unwrap_or(self.default_diffuse);
        let normal_id = params.normal_texture.unwrap_or(self.default_normal);
        let shader_id = params.shader.unwrap_or(self.default_shader);

        let material = material::Material::new(diffuse_id, normal_id, shader_id, params);
        let material_id = self.get_next_material_id();
        self.material_names.insert(name.to_string(), material_id);
        self.materials.push(material);
    }
    pub fn create_post_process(&mut self, params: PostProcessParams) {
        let texture_id = params.texture.unwrap_or(self.default_diffuse);
        let pass = postprocess::PostProcessPass::new(texture_id, params);
        self.postprocess.push(pass);
    }
    pub(crate) fn texture_from_path(&mut self, path: &str) -> ResourceId {
        let asset_id = self.load_asset(path);
        self.create_texture(asset_id)
    }
    fn texture_from_bytes(&mut self, bytes: &'static [u8]) -> ResourceId {
        let asset_id = {
            let mut store = self
                .asset_store
                .lock()
                .expect("Can't acquire the asset store!");
            store.from_bytes(bytes)
        };
        self.create_texture(asset_id)
    }
    fn create_texture(&mut self, asset_id: ResourceId) -> ResourceId {
        let texture = {
            let store = self
                .asset_store
                .lock()
                .expect("Can't acquire the asset store!");
            let asset = store
                .get(asset_id)
                .ok_or(EngineError::ResourceNotFound)
                .expect("Invalid texture asset!");
            texture::TextureData::from_bytes(asset_id, asset.data.get())
        };
        let texture_id = self.get_next_texture_id();
        self.textures.push(texture);
        texture_id
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
            diffuse_texture: Some(self.texture_from_path(path)),
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
    fn load_builtin_shader(
        &mut self,
        bytes: &'static [u8],
        kind: ShaderKind,
        builtin_id: BuiltInShader,
    ) {
        let mut store = self
            .asset_store
            .lock()
            .expect("Can't acquire the asset store!");
        let asset_id = store.from_bytes(bytes);
        let shader = shader::Shader::new(kind, asset_id);
        let id = self.get_next_shader_id();
        self.shaders.push(shader);
        self.builtin_shaders.insert(builtin_id, id);
    }
    fn get_next_shader_id(&self) -> ResourceId {
        ResourceId(self.shaders.len())
    }
    fn get_next_material_id(&self) -> ResourceId {
        ResourceId(self.materials.len())
    }
    fn get_next_texture_id(&self) -> ResourceId {
        ResourceId(self.textures.len())
    }
    fn get_next_camera_id(&self) -> ResourceId {
        ResourceId(self.cameras.len())
    }
}
