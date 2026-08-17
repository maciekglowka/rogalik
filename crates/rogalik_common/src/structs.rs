#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};

use crate::data::U8_TO_SRGB;

pub struct ResourceId<T>(pub usize, std::marker::PhantomData<fn() -> T>);
impl<T> ResourceId<T> {
    pub fn new(id: usize) -> Self {
        Self(id, std::marker::PhantomData)
    }
}
impl<T> Clone for ResourceId<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for ResourceId<T> {}

impl<T> Default for ResourceId<T> {
    fn default() -> Self {
        Self::new(0)
    }
}

impl<T> PartialEq for ResourceId<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl<T> Eq for ResourceId<T> {}
impl<T> PartialOrd for ResourceId<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl<T> Ord for ResourceId<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}
impl<T> std::hash::Hash for ResourceId<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}
impl<T> std::fmt::Debug for ResourceId<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ResourceId({})", self.0)
    }
}

// Resource type markers.
pub struct AssetId;
pub struct CameraId;
pub struct ShaderId;
pub struct TextureId;
pub struct TimerId;

#[derive(Debug)]
pub enum EngineError {
    NameConflict,
    InvalidResource,
    ResourceNotFound,
    GraphicsInternalError,
    GraphicsNotReady,
}
impl std::fmt::Display for EngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NameConflict => f.write_str("Name conflict"),
            Self::InvalidResource => f.write_str("Invalid resource"),
            Self::ResourceNotFound => f.write_str("Resource not found"),
            Self::GraphicsInternalError => f.write_str("Graphics internal error"),
            Self::GraphicsNotReady => f.write_str("Graphics not ready"),
        }
    }
}
impl std::error::Error for EngineError {}

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

#[derive(Clone, Copy)]
pub struct AudioDeviceParams {
    pub sample_rate: usize,
    pub buffer_secs: f32,
}
impl Default for AudioDeviceParams {
    fn default() -> Self {
        Self {
            sample_rate: 44100,
            buffer_secs: 0.1,
        }
    }
}
