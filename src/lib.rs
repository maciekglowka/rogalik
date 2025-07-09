#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]

pub use rogalik_audio as audio;
pub use rogalik_engine as engine;
pub use rogalik_math as math;
pub use rogalik_persist as persist;
pub use rogalik_wgpu as wgpu;

pub mod prelude {
    pub use rogalik_assets::{AssetContext, AssetState};
    pub use rogalik_common::*;
    pub use rogalik_engine::{Context, EngineBuilder, Game, Scene, SceneChange};
    pub use rogalik_math::vectors::{Vector2f, Vector2i};
}
