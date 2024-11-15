use rogalik_common::{EngineError, ResourceId};

mod dev_file_store;
mod embedded_store;

pub use dev_file_store::DevFileStore as AssetStore;

const ROOT_VAR: &str = "ROGALIK_ASSETS";

pub struct Asset {
    pub state: AssetState,
    pub data: Vec<u8>,
}
impl Asset {
    pub fn new(bytes: &[u8]) -> Self {
        Self {
            state: AssetState::Loaded,
            data: bytes.to_vec(), // TODO use &'static [u8]
        }
    }
}
pub enum AssetState {
    Loaded,
    Updated,
}

pub trait AssetStoreTrait: Default {
    fn from_bytes(&mut self, data: &[u8]) -> ResourceId;
    fn load(&mut self, path: &str) -> Result<ResourceId, EngineError>;
    fn get(&self, asset_id: ResourceId) -> Option<&Asset>;
}
