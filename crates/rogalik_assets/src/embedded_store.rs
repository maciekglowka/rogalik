use std::collections::HashMap;

use rogalik_common::{structs::AssetId, EngineError, ResourceId};

use super::{Asset, AssetContext, AssetState};

include!(concat!(env!("OUT_DIR"), "/included_assets.rs"));

pub struct EmbeddedStore {
    next_id: ResourceId<AssetId>,
    assets: HashMap<ResourceId<AssetId>, Asset>,
    embedded: HashMap<&'static str, &'static [u8]>,
}
impl Default for EmbeddedStore {
    fn default() -> Self {
        log::debug!("Embedded Asset Store init.");
        Self {
            embedded: get_embedded(),
            next_id: ResourceId::new(0),
            assets: HashMap::new(),
        }
    }
}
impl EmbeddedStore {
    fn bump_id(&mut self) {
        self.next_id = ResourceId::new(self.next_id.0 + 1);
    }
}
impl AssetContext for EmbeddedStore {
    fn from_bytes(&mut self, data: &'static [u8]) -> ResourceId<AssetId> {
        let id = self.next_id;
        self.assets.insert(id, Asset::borrowed(data));
        self.bump_id();
        id
    }
    fn load(&mut self, path: &str) -> Result<ResourceId<AssetId>, EngineError> {
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
    fn get(&self, asset_id: ResourceId<AssetId>) -> Option<&Asset> {
        self.assets.get(&asset_id)
    }
}
