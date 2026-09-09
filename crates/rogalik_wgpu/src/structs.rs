use rogalik_common::{
    structs::{CameraId, ShaderId},
    ResourceId,
};

#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub struct Color(pub u8, pub u8, pub u8, pub u8);
impl Color {
    pub fn as_f32(&self) -> [f32; 4] {
        [
            self.0 as f32 / 255.,
            self.1 as f32 / 255.,
            self.2 as f32 / 255.,
            self.3 as f32 / 255.,
        ]
    }

    #[inline(always)]
    pub fn as_srgb(&self) -> [f32; 4] {
        [
            U8_TO_SRGB[self.0 as usize],
            U8_TO_SRGB[self.1 as usize],
            U8_TO_SRGB[self.2 as usize],
            self.3 as f32 / 255.,
        ]
    }
}
impl Default for Color {
    fn default() -> Self {
        Self(255, 255, 255, 255)
    }
}

#[derive(Clone, Copy, Default)]
pub struct SpriteParams {
    pub color: Color,
    pub flip_x: bool,
    pub flip_y: bool,
    pub rotate: f32,
    pub slice: Option<u32>,
}

#[derive(Clone, Default)]
pub struct MaterialParams {
    pub atlas: Option<AtlasParams>,
    pub diffuse_texture: Option<ResourceId<TextureId>>,
    pub normal_texture: Option<ResourceId<TextureId>>,
    pub shader: Option<ResourceId<ShaderId>>,
    pub repeat: TextureRepeat,
    pub filtering: TextureFiltering,
}

#[derive(Clone, Copy)]
pub struct PostProcessParams {
    pub texture: Option<ResourceId<TextureId>>,
    pub shader: ResourceId<ShaderId>,
    pub repeat: TextureRepeat,
    pub filtering: TextureFiltering,
}

#[derive(Copy, Clone, Debug)]
pub struct AtlasPosition {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}
impl AtlasPosition {
    pub fn new(x: u32, y: u32, w: u32, h: u32) -> Self {
        Self { x, y, w, h }
    }
}

#[derive(Clone, Debug)]
pub enum AtlasParams {
    Grid {
        cols: usize,
        rows: usize,
        padding: Option<(u32, u32)>,
    },
    Free(Vec<AtlasPosition>),
}

#[derive(Clone, Copy, Debug, Default)]
pub struct FontParams<'a> {
    /// For TTF determines which glyphs should be rendered into atlas.
    /// For bitmap fonts specifies the order of glyphs on the provided atlas.
    ///
    /// If not provided ASCII mapping is used.
    pub charset: Option<&'a [char]>,
    pub filtering: TextureFiltering,
    pub shader: Option<ResourceId<ShaderId>>,
    /// Horizontal spacing between characters.
    ///
    /// Typically this only should be set for bitmap atlas fonts.
    ///
    /// Relative to font size.
    /// E.g. spacing value 0.25 will result in 2px gap
    /// on 8px font and 4px gap on 16px font.
    pub character_spacing: Option<f32>,
    /// Line spacing, relative to font size.
    pub line_spacing: Option<f32>,
}

#[derive(Clone, Copy, Default)]
pub enum TextureRepeat {
    #[default]
    Clamp,
    Repeat,
    MirrorRepeat,
}

#[derive(Clone, Copy, Debug, Default)]
pub enum TextureFiltering {
    #[default]
    Nearest,
    Linear,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum ShaderKind {
    Sprite,
    PostProcess,
}

#[derive(Hash, Eq, PartialEq, Debug)]
pub enum BuiltInShader {
    SpriteUnlit,
    SpriteLit,
    Upscale,
    Lut,
}

pub(crate) type Quad = ([Vertex; 4], [u16; 6]);

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
    pub tex_coords: [f32; 2],
}
impl Vertex {
    const ATTRS: [wgpu::VertexAttribute; 3] = wgpu::vertex_attr_array![
        0 => Float32x3,
        1 => Float32x4,
        2 => Float32x2,
    ];
    pub(crate) fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRS,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct Triangle {
    pub indices: [u16; 3],
    pub z_index: i32,
    pub params: BindParams,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct BindParams {
    pub shader_id: ResourceId<ShaderId>,
    pub material_id: ResourceId<MaterialId>,
    pub camera_id: ResourceId<CameraId>,
}
