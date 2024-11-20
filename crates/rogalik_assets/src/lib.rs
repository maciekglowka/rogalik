use rogalik_common::{EngineError, ResourceId};

mod dev_file_store;
mod embedded_store;

#[cfg(debug_assertions)]
pub use dev_file_store::DevFileStore as AssetStore;
#[cfg(not(debug_assertions))]
pub use embedded_store::EmbeddedStore as AssetStore;

const ROOT_VAR: &str = "ROGALIK_ASSETS";

pub struct Asset {
    pub state: AssetState,
    pub data: AssetBytes,
}
impl Asset {
    pub fn owned(bytes: Vec<u8>) -> Self {
        Self {
            state: AssetState::Loaded,
            data: AssetBytes::Owned(bytes),
        }
    }
    pub fn borrowed(bytes: &'static [u8]) -> Self {
        Self {
            state: AssetState::Loaded,
            data: AssetBytes::Borrowed(bytes),
        }
    }
}

pub enum AssetBytes {
    Borrowed(&'static [u8]),
    Owned(Vec<u8>),
}
impl AssetBytes {
    pub fn get(&self) -> &[u8] {
        match self {
            Self::Borrowed(a) => a,
            Self::Owned(a) => a,
        }
    }
}

#[derive(PartialEq, Eq)]
pub enum AssetState {
    Loaded,
    Updated,
}

pub trait AssetContext: Default {
    fn from_bytes(&mut self, data: &'static [u8]) -> ResourceId;
    fn load(&mut self, path: &str) -> Result<ResourceId, EngineError>;
    fn get(&self, asset_id: ResourceId) -> Option<&Asset>;
}
