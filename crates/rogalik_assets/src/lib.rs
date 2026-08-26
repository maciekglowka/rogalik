use rogalik_common::{structs::AssetId, EngineError, ResourceId};

#[cfg(dev_tools)]
mod dev_file_store;

#[cfg(not(dev_tools))]
mod embedded_store;

#[cfg(dev_tools)]
pub use dev_file_store::DevFileStore as AssetStore;

#[cfg(not(dev_tools))]
pub use embedded_store::EmbeddedStore as AssetStore;

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
    fn load_bytes(&mut self, data: &'static [u8]) -> ResourceId<AssetId>;
    fn load(&mut self, path: &str) -> Result<ResourceId<AssetId>, EngineError>;
    fn get(&self, asset_id: ResourceId<AssetId>) -> Option<&Asset>;
    fn mark_read(&mut self, _asset_id: ResourceId<AssetId>) {}
}
