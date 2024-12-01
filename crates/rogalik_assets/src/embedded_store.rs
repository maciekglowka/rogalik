use std::collections::HashMap;

use rogalik_common::{EngineError, ResourceId};

use super::{Asset, AssetContext, AssetState};

include!(concat!(env!("OUT_DIR"), "/included_assets.rs"));

pub struct EmbeddedStore {
    next_id: ResourceId,
    assets: HashMap<ResourceId, Asset>,
    embedded: HashMap<&'static str, &'static [u8]>,
}
impl Default for EmbeddedStore {
    fn default() -> Self {
        log::debug!("Embedded Asset Store init.");
        Self {
            embedded: get_embedded(),
            next_id: ResourceId(0),
            assets: HashMap::new(),
        }
    }
}
impl EmbeddedStore {
    fn bump_id(&mut self) {
        self.next_id = self.next_id.next();
    }
}
impl AssetContext for EmbeddedStore {
    fn from_bytes(&mut self, data: &'static [u8]) -> ResourceId {
        let id = self.next_id;
        self.assets.insert(id, Asset::borrowed(data));
        self.bump_id();
        id
    }
    fn load(&mut self, path: &str) -> Result<ResourceId, EngineError> {
        let id = self.next_id;

        let data = self
            .embedded
            .get(path)
            .ok_or(EngineError::ResourceNotFound)?;

        log::debug!(
            "Loaded embedded asset from: {}. {} bytes.",
            path,
            data.len()
        );
        self.assets.insert(id, Asset::borrowed(data));
        self.bump_id();
        Ok(id)
    }
    fn get(&self, asset_id: ResourceId) -> Option<&Asset> {
        self.assets.get(&asset_id)
    }
}
