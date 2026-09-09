use rogalik_arena::ResourceId;

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
    fn load_bytes(&mut self, data: &'static [u8]) -> ResourceId<Asset>;
    fn load(&mut self, path: &str) -> Result<ResourceId<Asset>, AssetError>;
    fn get(&self, asset_id: ResourceId<Asset>) -> Option<&Asset>;
    fn mark_read(&mut self, _asset_id: ResourceId<Asset>) {}
}

#[derive(Debug)]
pub enum AssetError {
    PathError(String, String),
    MetaDataError(String),
}
impl std::fmt::Display for AssetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PathError(path, inner) => {
                write!(f, "reading path failed: {path}, reason: {inner}")
            }
            Self::MetaDataError(inner) => write!(f, "reading meta data failed: {inner}"),
        }
    }
}
impl std::error::Error for AssetError {}
