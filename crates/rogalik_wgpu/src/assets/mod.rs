use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

use rogalik_arena::{Arena, Id};
use rogalik_assets::{Asset, AssetContext, AssetState, AssetStore};
use rogalik_math::vectors::Vector2f;

use crate::{assets::bind_groups::BindGroupLayoutKind, GraphicsError};

pub(crate) mod atlas;
pub(crate) mod bind_groups;
pub(crate) mod camera;
pub(crate) mod font;
pub(crate) mod material;
pub(crate) mod postprocess;
pub(crate) mod shader;
pub(crate) mod texture;

use atlas::AtlasParams;
use camera::Camera2d;
use font::{render_ttf_glyphs, text_key_size, Font, FontParams, FontSize};
use material::{Material, MaterialParams};
use postprocess::{PostProcessParams, PostProcessPass};
use shader::{BuiltInShader, Shader, ShaderKind};
use texture::TextureData;

const DIFFUSE_1PX: &[u8] = &[0xff; 4];

pub(crate) struct WgpuAssets {
    pub(crate) asset_store: Arc<Mutex<AssetStore>>,
    pub(crate) bind_group_layouts: HashMap<BindGroupLayoutKind, wgpu::BindGroupLayout>,
    pub(crate) builtin_shaders: HashMap<BuiltInShader, Id<Shader>>,
    pub(crate) cameras: Arena<camera::Camera2d>,
    pub(crate) default_shader: Option<Id<Shader>>,
    // pub(crate) default_normal: Option<Id<TextureData>>,
    pub(crate) default_diffuse: Option<Id<TextureData>>,
    pub(crate) fonts: HashMap<String, Font>,
    pub(crate) pipeline_layouts: HashMap<ShaderKind, wgpu::PipelineLayout>,
    material_names: HashMap<String, Id<Material>>, // lookup
    materials: Arena<Material>,
    pub(crate) postprocess: Arena<PostProcessPass>,
    postprocess_names: HashMap<String, Id<PostProcessPass>>, // lookup
    shaders: Arena<Shader>,
    pub(crate) textures: Arena<TextureData>,
}
impl WgpuAssets {
    pub(crate) fn new(asset_store: Arc<Mutex<AssetStore>>) -> Self {
        let mut assets = Self {
            // perhaps this clone could be avoided?
            asset_store: asset_store.clone(),
            bind_group_layouts: HashMap::new(),
            builtin_shaders: HashMap::new(),
            cameras: Arena::new(),
            default_shader: None,
            // default_normal: None,
            default_diffuse: None,
            fonts: HashMap::new(),
            material_names: HashMap::new(),
            materials: Arena::new(),
            pipeline_layouts: HashMap::new(),
            postprocess: Arena::new(),
            postprocess_names: HashMap::new(),
            shaders: Arena::new(),
            textures: Arena::new(),
        };
        assets.load_builtins();
        assets
    }
    /// Only (and immediately) to be called in a constructor
    fn load_builtins(&mut self) {
        let unlit_id = self.load_builtin_shader(
            include_bytes!("include/sprite_unlit.wgsl"),
            ShaderKind::Sprite,
            BuiltInShader::SpriteUnlit,
        );
        // self.load_builtin_shader(
        //     include_bytes!("include/sprite_lit.wgsl"),
        //     ShaderKind::Sprite,
        //     BuiltInShader::SpriteLit,
        // );
        self.load_builtin_shader(
            include_bytes!("include/sprite_pass_upscale.wgsl"),
            ShaderKind::PostProcess,
            BuiltInShader::Upscale,
        );
        // self.load_builtin_shader(
        //     include_bytes!("include/lut.wgsl"),
        //     ShaderKind::PostProcess,
        //     BuiltInShader::Lut,
        // );

        self.default_shader = Some(unlit_id);
        // self.default_normal =
        //     Some(self.texture_from_bytes(include_bytes!("include/
        // default_normal.png")));

        // FIXME
        // self.default_diffuse =
        // Some(self.texture_from_bytes(include_bytes!("include/white.png")));
        self.default_diffuse = Some(self.texture_from_bytes(DIFFUSE_1PX, 1, 1, false).unwrap())
    }
    pub(crate) fn create_wgpu_data(
        &mut self,
        w: u32,
        h: u32,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_format: &wgpu::TextureFormat,
    ) -> Result<(), GraphicsError> {
        self.init_bind_group_layouts(device);

        let mut store = self
            .asset_store
            .lock()
            .expect("Can't acquire the asset store!");

        for texture in self.textures.values_mut() {
            log::debug!("Creating texture");
            texture.create_wgpu_data(&mut *store, device, queue);
        }

        for material in self.materials.values_mut() {
            let material_layout = self
                .bind_group_layouts
                .entry(material.bind_group_layout_kind)
                .or_insert(bind_groups::get_bind_group_layout(
                    device,
                    material.bind_group_layout_kind,
                ));

            log::debug!("Creating material: {:?}", material);
            material.create_wgpu_data(&self.textures, device, material_layout)?;
        }

        for shader in self.shaders.values_mut() {
            log::debug!("Creating shader: {:?}", shader);
            shader.create_wgpu_data(&mut store, device, texture_format, &self.pipeline_layouts)?;
        }

        drop(store);
        self.update_postprocess_wgpu_data(w, h, device, queue, texture_format)?;

        self.create_pipeline_layouts(device)?;

        for camera in self.cameras.values_mut() {
            camera.create_wgpu_data(
                device,
                self.bind_group_layouts
                    .get(&crate::assets::bind_groups::BindGroupLayoutKind::Uniform)
                    .ok_or(GraphicsError::InternalError)?,
            );
        }

        Ok(())
    }
    pub(crate) fn update_postprocess_wgpu_data(
        &mut self,
        w: u32,
        h: u32,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_format: &wgpu::TextureFormat,
    ) -> Result<(), GraphicsError> {
        let postprocess_layout = self
            .bind_group_layouts
            .get(&bind_groups::BindGroupLayoutKind::PostProcess)
            .ok_or(GraphicsError::InternalError)?;

        for pass in self.postprocess.values_mut() {
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
    pub(crate) fn update_assets(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_format: &wgpu::TextureFormat,
    ) -> Result<(), GraphicsError> {
        let mut store = self
            .asset_store
            .lock()
            .expect("Can't acquire the asset store!");

        let material_layout = self
            .bind_group_layouts
            .get(&bind_groups::BindGroupLayoutKind::Sprite)
            .ok_or(GraphicsError::InternalError)?;

        let mut updated_textures = HashSet::new();

        // FIXME
        for (id, texture) in self.textures.iter_mut() {
            if let Some((asset_id, asset)) = texture
                .asset_id()
                .and_then(|id| store.get(id).map(|a| (id, a)))
            {
                if asset.state == AssetState::Updated {
                    log::debug!("Updating texture {:?}, Asset: {:?}", id, asset_id);

                    if texture
                        .create_wgpu_data(&mut *store, device, queue)
                        .is_err()
                    {
                        log::error!("Material reload failed!");
                    }

                    updated_textures.insert(id);

                    #[cfg(dev_tools)]
                    store.mark_read(asset_id.unwrap());
                }
            }
        }

        for material in self.materials.values_mut() {
            if material
                .textures
                .iter()
                .any(|t| updated_textures.contains(t))
            {
                log::debug!("Updating material {:?}", material);
                if material
                    .create_wgpu_data(&self.textures, device, queue, material_layout)
                    .is_err()
                {
                    log::error!("Material reload failed!");
                }
            }
        }

        for shader in self.shaders.values_mut() {
            if shader
                .create_wgpu_data(&mut store, device, texture_format, &self.pipeline_layouts)
                .is_err()
            {
                log::debug!("Shader reload failed!");
            }
            #[cfg(dev_tools)]
            store.mark_read(shader.asset_id);
        }

        Ok(())
    }
    fn init_bind_group_layouts(&mut self, device: &wgpu::Device) {
        self.bind_group_layouts = HashMap::from_iter([(
            BindGroupLayoutKind::Uniform,
            bind_groups::get_bind_group_layout(device, BindGroupLayoutKind::Uniform),
        )]);
    }
    fn create_pipeline_layouts(&mut self, device: &wgpu::Device) -> Result<(), GraphicsError> {
        self.pipeline_layouts = shader::get_pipeline_layouts(&self.bind_group_layouts, device)?;
        Ok(())
    }
    pub(crate) fn create_shader(&mut self, kind: ShaderKind, path: &str) -> Id<Shader> {
        let asset_id = self.load_asset(path);
        let shader = shader::Shader::new(kind, asset_id);
        self.shaders.insert(shader)
    }
    pub(crate) fn create_material(
        &mut self,
        name: &str,
        mut params: MaterialParams,
    ) -> Result<Id<Material>, GraphicsError> {
        if self.material_names.contains_key(name) {
            return Err(GraphicsError::MaterialError(format!(
                "name conflict: {name}"
            )));
        }

        // Handle default shader.
        if params.shader.is_none() {
            params.shader = self.default_shader;
            if params.textures.is_empty() {
                params.textures.push(self.default_diffuse.unwrap());
            }
        }

        let material = material::Material::new(params);
        let material_id = self.materials.insert(material);
        self.material_names.insert(name.to_string(), material_id);
        Ok(material_id)
    }
    pub(crate) fn create_post_process(&mut self, name: &str, params: PostProcessParams) {
        let texture_id = params.texture.unwrap_or(self.default_diffuse.unwrap());
        let pass = postprocess::PostProcessPass::new(texture_id, params);
        let postprocess_id = self.postprocess.insert(pass);
        self.postprocess_names
            .insert(name.to_string(), postprocess_id);
    }
    pub(crate) fn texture_from_path(
        &mut self,
        path: &str,
        is_data: bool,
    ) -> Result<Id<TextureData>, GraphicsError> {
        // let texture = {
        // let store = self
        //     .asset_store
        //     .lock()
        //     .expect("Can't acquire the asset store!");

        // let asset = store
        //     .get(asset_id)
        //     .ok_or_else(|| GraphicsError::ResourceNotFound(path.to_string()))
        //     .expect("Invalid texture asset!");

        // TODO error handling.
        // texture::TextureData::from_file_bytes(Some(asset_id),
        // asset.data.get()).unwrap() texture
        // };
        let asset_id = self.load_asset(path);
        let texture = texture::TextureData::from_asset(asset_id, is_data)?;
        Ok(self.add_texture(texture))
    }
    fn texture_from_bytes(
        &mut self,
        bytes: &[u8],
        width: u32,
        height: u32,
        is_data: bool,
    ) -> Result<Id<TextureData>, GraphicsError> {
        Ok(self.add_texture(texture::TextureData::from_raw(
            bytes, width, height, is_data,
        )?))
        // TODO error handling
        // self.add_texture(texture::TextureData::from_file_bytes(None,
        // bytes).unwrap())
    }
    fn add_texture(&mut self, texture: texture::TextureData) -> Id<TextureData> {
        self.textures.insert(texture)
    }
    pub(crate) fn create_camera(
        &mut self,
        vw: f32,
        vh: f32,
        rw: f32,
        rh: f32,
        scale: f32,
        target: Vector2f,
    ) -> Id<Camera2d> {
        let camera = camera::Camera2d::new(vw, vh, rw, rh, scale, target);
        self.cameras.insert(camera)
    }
    pub(crate) fn load_font(
        &mut self,
        name: &str,
        path: &str,
        params: FontParams,
    ) -> Result<(), GraphicsError> {
        if self.fonts.contains_key(name) {
            return Err(GraphicsError::FontError(format!("name conflict: {name}")));
        }

        let asset_id = self.load_asset(path);
        let font = Font::new_from_ttf(&params, asset_id);
        self.fonts.insert(name.to_string(), font);
        Ok(())
    }
    pub(crate) fn load_font_atlas(
        &mut self,
        name: &str,
        path: &str,
        atlas: AtlasParams,
        params: FontParams,
    ) -> Result<(), GraphicsError> {
        if self.fonts.contains_key(name) {
            return Err(GraphicsError::FontError(format!("name conflict: {name}")));
        }

        let material_params = MaterialParams {
            atlas: Some(atlas),
            textures: vec![self.texture_from_path(path, false)?],
            shader: params.shader,
            filtering: params.filtering,
            ..Default::default()
        };
        let material_id = self.create_material(name, material_params)?;
        let font = Font::new_from_atlas(&params, material_id);
        self.fonts.insert(name.to_string(), font);
        Ok(())
    }
    pub(crate) fn has_font_size(&self, name: &str, size: f32) -> Result<bool, GraphicsError> {
        let font = self
            .fonts
            .get(name)
            .ok_or_else(|| GraphicsError::ResourceNotFound(name.to_string()))?;

        match &font.kind {
            font::FontKind::Bitmap(_) => Ok(true),
            font::FontKind::Ttf { sizes, .. } => Ok(sizes.contains_key(&text_key_size(size))),
        }
    }
    pub(crate) fn create_font_size(
        &mut self,
        name: &str,
        size: f32,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<(), GraphicsError> {
        let font = self
            .fonts
            .get(name)
            .ok_or_else(|| GraphicsError::ResourceNotFound(name.to_string()))?;

        let asset_id = if let font::FontKind::Ttf { asset_id, .. } = font.kind {
            asset_id
        } else {
            return Err(GraphicsError::FontError(
                "invalid font type, expected ttf".to_string(),
            ));
        };

        let glyphs = {
            let store = self
                .asset_store
                .lock()
                .expect("Can't acquire the asset store!");

            let asset = store
                .get(asset_id)
                .ok_or_else(|| GraphicsError::ResourceNotFound(format!("font {asset_id:?}")))?;

            render_ttf_glyphs(&font.charset, asset.data.get(), size)
        }?;

        let mut material_params = MaterialParams {
            atlas: Some(glyphs.atlas_params),
            shader: font.shader,
            filtering: font.filtering,
            ..Default::default()
        };

        let texture = texture::TextureData::from_raw(
            &glyphs.texture_data,
            glyphs.texture_size.0,
            glyphs.texture_size.1,
            false,
        )?;
        let texture_id = self.add_texture(texture);

        material_params.textures.push(texture_id);

        let material_id = self.create_material(&format!("{name}_{size}"), material_params)?;

        if let font::FontKind::Ttf { sizes, .. } = &mut self.fonts.get_mut(name).unwrap().kind {
            sizes.insert(
                text_key_size(size),
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
            .ok_or(GraphicsError::InternalError)?;

        self.materials
            .get_mut(&material_id)
            .unwrap()
            .create_wgpu_data(&self.textures, device, queue, material_layout)?;

        Ok(())
    }
    pub(crate) fn get_material_id(&self, name: &str) -> Option<&Id<Material>> {
        self.material_names.get(name)
    }
    pub(crate) fn get_material(&self, id: &Id<Material>) -> Option<&material::Material> {
        self.materials.get(id)
    }
    pub(crate) fn get_font(&self, name: &str) -> Option<&Font> {
        self.fonts.get(name)
    }
    pub(crate) fn get_shader(&self, id: &Id<Shader>) -> Option<&shader::Shader> {
        self.shaders.get(id)
    }
    pub(crate) fn get_camera(&self, id: &Id<Camera2d>) -> Option<&camera::Camera2d> {
        self.cameras.get(id)
    }
    pub(crate) fn get_camera_mut(&mut self, id: &Id<Camera2d>) -> Option<&mut camera::Camera2d> {
        self.cameras.get_mut(id)
    }
    pub(crate) fn get_postprocess_id(&self, name: &str) -> Option<&Id<PostProcessPass>> {
        self.postprocess_names.get(name)
    }
    pub(crate) fn get_postprocess_mut(
        &mut self,
        id: &Id<PostProcessPass>,
    ) -> Option<&mut postprocess::PostProcessPass> {
        self.postprocess.get_mut(id)
    }
    fn load_asset(&self, path: &str) -> Id<Asset> {
        let mut store = self
            .asset_store
            .lock()
            .expect("Can't acquire the asset store!");
        store
            .load(path)
            .unwrap_or_else(|_| panic!("Can't load {}!", path))
    }
    fn load_builtin_shader(
        &mut self,
        bytes: &'static [u8],
        kind: ShaderKind,
        builtin_id: BuiltInShader,
    ) -> Id<Shader> {
        let mut store = self
            .asset_store
            .lock()
            .expect("Can't acquire the asset store!");
        let asset_id = store.load_bytes(bytes);
        let shader = shader::Shader::new(kind, asset_id);
        let id = self.shaders.insert(shader);
        self.builtin_shaders.insert(builtin_id, id);
        id
    }
}
