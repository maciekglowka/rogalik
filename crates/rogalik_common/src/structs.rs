#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct ResourceId(pub usize);
impl ResourceId {
    pub fn next(&self) -> Self {
        Self(self.0 + 1)
    }
}

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
    pub fn as_srgb(&self) -> [f32; 4] {
        let f = self.as_f32();
        [
            srgb_single(f[0]),
            srgb_single(f[1]),
            srgb_single(f[2]),
            f[3],
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

#[inline(always)]
fn srgb_single(v: f32) -> f32 {
    ((v + 0.055) / 1.055).powf(2.4)
}

#[derive(Clone, Default)]
pub struct MaterialParams {
    pub atlas: Option<AtlasParams>,
    pub diffuse_texture: Option<ResourceId>,
    pub normal_texture: Option<ResourceId>,
    pub shader: Option<ResourceId>,
    pub repeat: TextureRepeat,
    pub filtering: TextureFiltering,
}

#[derive(Clone, Copy, Default)]
pub struct PostProcessParams {
    pub texture: Option<ResourceId>,
    pub shader: ResourceId,
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
    pub shader: Option<ResourceId>,
    /// Horizontal spacing between characters.
    ///
    /// Typically this only should be set for bitmap atlas fonts.
    ///
    /// Relative to font size.
    /// E.g. spacing value 0.25 will result in 2px gap
    /// on 8px font and 4px gap on 16px font.
    pub character_spacing: Option<f32>,
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
