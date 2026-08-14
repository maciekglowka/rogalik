use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

use rogalik_assets::{AssetContext, AssetState, AssetStore};
use rogalik_common::{
    AtlasParams, BuiltInShader, EngineError, FontParams, MaterialParams, PostProcessParams,
    ResourceId, ShaderKind,
};
use rogalik_math::vectors::Vector2f;

use crate::assets::font::{render_ttf_glyphs, Font, FontSize, TtfGlyphs};

pub mod atlas;
pub mod bind_groups;
pub mod camera;
pub mod font;
pub mod material;
pub mod postprocess;
pub mod shader;
mod texture;

pub struct WgpuAssets {
    pub(crate) asset_store: Arc<Mutex<AssetStore>>,
    pub(crate) bind_group_layouts: HashMap<bind_groups::BindGroupLayoutKind, wgpu::BindGroupLayout>,
    pub(crate) builtin_shaders: HashMap<BuiltInShader, ResourceId>,
    pub(crate) cameras: Vec<camera::Camera2D>,
    pub(crate) default_shader: ResourceId,
    pub(crate) default_normal: ResourceId,
    pub(crate) default_diffuse: ResourceId,
    pub(crate) fonts: HashMap<String, Font>,
    pub(crate) pipeline_layouts: HashMap<ShaderKind, wgpu::PipelineLayout>,
    material_names: HashMap<String, ResourceId>, // lookup
    materials: Vec<material::Material>,
    pub(crate) postprocess: Vec<postprocess::PostProcessPass>,
    postprocess_names: HashMap<String, ResourceId>, // lookup
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
            fonts: HashMap::new(),
            material_names: HashMap::new(),
            materials: Vec::new(),
            pipeline_layouts: HashMap::new(),
            postprocess: Vec::new(),
            postprocess_names: HashMap::new(),
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

        for camera in self.cameras.iter_mut() {
            camera.create_wgpu_data(
                device,
                self.bind_group_layouts
                    .get(&crate::assets::bind_groups::BindGroupLayoutKind::Uniform)
                    .ok_or(EngineError::GraphicsInternalError)?,
            );
        }

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
                postprocess_layout,
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
            if let Some(asset) = texture.asset_id.and_then(|id| store.get(id)) {
                if asset.state == AssetState::Updated {
                    log::debug!("Updating texture {}, Asset: {:?}", i, texture.asset_id);
                    texture.update_bytes(asset.data.get());
                    updated_textures.insert(i);

                    #[cfg(debug_assertions)]
                    store.mark_read(texture.asset_id.unwrap());
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
    pub fn create_material(
        &mut self,
        name: &str,
        params: MaterialParams,
    ) -> Result<ResourceId, EngineError> {
        if self.material_names.contains_key(name) {
            return Err(EngineError::NameConflict);
        }
        let diffuse_id = params.diffuse_texture.unwrap_or(self.default_diffuse);
        let normal_id = params.normal_texture.unwrap_or(self.default_normal);
        let shader_id = params.shader.unwrap_or(self.default_shader);

        let material = material::Material::new(diffuse_id, normal_id, shader_id, params);
        let material_id = self.get_next_material_id();
        self.material_names.insert(name.to_string(), material_id);
        self.materials.push(material);
        Ok(material_id)
    }
    pub fn create_post_process(&mut self, name: &str, params: PostProcessParams) {
        let texture_id = params.texture.unwrap_or(self.default_diffuse);
        let pass = postprocess::PostProcessPass::new(texture_id, params);
        let postprocess_id = self.get_next_postprocess_id();
        self.postprocess.push(pass);
        self.postprocess_names
            .insert(name.to_string(), postprocess_id);
    }
    pub(crate) fn texture_from_path(&mut self, path: &str) -> ResourceId {
        let texture = {
            let asset_id = self.load_asset(path);
            let store = self
                .asset_store
                .lock()
                .expect("Can't acquire the asset store!");
            let asset = store
                .get(asset_id)
                .ok_or(EngineError::ResourceNotFound)
                .expect("Invalid texture asset!");

            // TODO error handling.
            texture::TextureData::from_file_bytes(Some(asset_id), asset.data.get()).unwrap()
        };
        self.add_texture(texture)
    }
    fn texture_from_bytes(&mut self, bytes: &[u8]) -> ResourceId {
        // TODO error handling
        self.add_texture(texture::TextureData::from_file_bytes(None, bytes).unwrap())
    }
    fn add_texture(&mut self, texture: texture::TextureData) -> ResourceId {
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
        params: FontParams,
    ) -> Result<(), EngineError> {
        if self.fonts.contains_key(name) {
            return Err(EngineError::NameConflict);
        }

        let asset_id = self.load_asset(path);
        let font = Font::new_from_ttf(&params, asset_id);
        self.fonts.insert(name.to_string(), font);
        Ok(())
    }
    pub fn load_font_atlas(
        &mut self,
        name: &str,
        path: &str,
        atlas: AtlasParams,
        params: FontParams,
    ) -> Result<(), EngineError> {
        if self.fonts.contains_key(name) {
            return Err(EngineError::NameConflict);
        }

        let material_params = MaterialParams {
            atlas: Some(atlas),
            diffuse_texture: Some(self.texture_from_path(path)),
            shader: params.shader,
            filtering: params.filtering,
            ..Default::default()
        };
        let material_id = self.create_material(name, material_params)?;
        let font = Font::new_from_atlas(&params, material_id);
        self.fonts.insert(name.to_string(), font);
        Ok(())
    }
    pub(crate) fn has_font_size(&self, name: &str, size: u32) -> Result<bool, EngineError> {
        let font = self.fonts.get(name).ok_or(EngineError::InvalidResource)?;

        match &font.kind {
            font::FontKind::Bitmap(_) => Ok(true),
            font::FontKind::Ttf { sizes, .. } => Ok(sizes.contains_key(&size)),
        }
    }
    pub(crate) fn create_font_size(
        &mut self,
        name: &str,
        size: u32,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<(), EngineError> {
        let font = self.fonts.get(name).ok_or(EngineError::ResourceNotFound)?;

        let asset_id = if let font::FontKind::Ttf { asset_id, .. } = font.kind {
            asset_id
        } else {
            return Err(EngineError::InvalidResource);
        };

        let glyphs = {
            let store = self
                .asset_store
                .lock()
                .expect("Can't acquire the asset store!");

            let asset = store
                .get(asset_id)
                .ok_or(EngineError::ResourceNotFound)
                .expect("Invalid font asset!");

            render_ttf_glyphs(&font.charset, asset.data.get(), size as f32)
        }?;

        let mut material_params = MaterialParams {
            atlas: Some(glyphs.atlas_params),
            diffuse_texture: None,
            shader: font.shader,
            filtering: font.filtering,
            ..Default::default()
        };

        let texture = texture::TextureData::from_raw(
            &glyphs.texture_data,
            glyphs.texture_size.0,
            glyphs.texture_size.1,
        )?;
        let texture_id = self.add_texture(texture);

        material_params.diffuse_texture = Some(texture_id);

        let material_id = self.create_material(&format!("{name}_{size}"), material_params)?;

        if let font::FontKind::Ttf { sizes, .. } = &mut self.fonts.get_mut(name).unwrap().kind {
            sizes.insert(
                size,
                FontSize {
                    material_id,
                    char_metrics: glyphs.char_metrics,
                    line_metrics: glyphs.line_metrics,
                },
            );
        }

        let material_layout = self
            .bind_group_layouts
            .get(&bind_groups::BindGroupLayoutKind::Sprite)
            .ok_or(EngineError::GraphicsInternalError)?;

        self.materials
            .get_mut(material_id.0)
            .unwrap()
            .create_wgpu_data(&self.textures, device, queue, material_layout)?;

        Ok(())
    }
    pub fn get_material_id(&self, name: &str) -> Option<&ResourceId> {
        self.material_names.get(name)
    }
    pub fn get_material(&self, id: ResourceId) -> Option<&material::Material> {
        self.materials.get(id.0)
    }
    pub fn get_font(&self, name: &str) -> Option<&Font> {
        self.fonts.get(name)
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
    pub fn get_postprocess_id(&self, name: &str) -> Option<&ResourceId> {
        self.postprocess_names.get(name)
    }
    pub fn get_postprocess_mut(
        &mut self,
        id: ResourceId,
    ) -> Option<&mut postprocess::PostProcessPass> {
        self.postprocess.get_mut(id.0)
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
    fn get_next_postprocess_id(&self) -> ResourceId {
        ResourceId(self.postprocess.len())
    }
    fn get_next_camera_id(&self) -> ResourceId {
        ResourceId(self.cameras.len())
    }
}
