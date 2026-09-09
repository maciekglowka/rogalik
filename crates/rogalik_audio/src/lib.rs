use rogalik_assets::AssetError;

mod assets;
mod engine;
mod source;

pub use engine::AudioEngine;

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

pub trait AudioSetup {
    /// Creates and initializes the audio context and device.
    fn create_context(&mut self);
    /// Checks if the audio context has been successfully created.
    fn has_context(&self) -> bool;
    /// Updates audio assets, checking for any changes or reloads.
    fn update_assets(&mut self);
}

#[derive(Debug)]
pub enum AudioError {
    AssetError(String),
    SourceNotFound(String),
    SourceError(String),
}
impl std::fmt::Display for AudioError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AssetError(inner) => {
                write!(f, "audio asset error: {inner}")
            }
            Self::SourceNotFound(name) => {
                write!(f, "audio source not found: {name}")
            }
            Self::SourceError(inner) => {
                write!(f, "audio source error: {inner}")
            }
        }
    }
}
impl std::error::Error for AudioError {}

impl From<AssetError> for AudioError {
    fn from(value: AssetError) -> Self {
        Self::AssetError(value.to_string())
    }
}
