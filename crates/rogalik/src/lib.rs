#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../README.md"))]

#[cfg(target_os = "android")]
mod android;
mod app;
mod engine;
pub mod input;
mod scenes;
mod time;
mod traits;
#[cfg(target_arch = "wasm32")]
mod wasm;

pub use log;
pub use time::Instant;
pub use traits::{Game, Scene, SceneChange};

pub use rogalik_audio as audio;
pub use rogalik_math as math;
pub use rogalik_persist as persist;
pub use rogalik_wgpu as wgpu;

pub mod prelude {
    pub use crate::engine::{Context, EngineBuilder};
    pub use crate::traits::{Game, Scene, SceneChange};
    pub use rogalik_assets::{AssetContext, AssetState};
    pub use rogalik_common::*;
    pub use rogalik_math::vectors::{Vector2f, Vector2i};
}
