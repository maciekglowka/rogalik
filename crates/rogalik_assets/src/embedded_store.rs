use std::collections::HashMap;

use rogalik_arena::{Arena, ResourceId};

use crate::{Asset, AssetContext, AssetError};

include!(env!("ROGALIK_ASSET_FILE"));

pub struct EmbeddedStore {
    assets: Arena<Asset>,
    embedded: HashMap<&'static str, &'static [u8]>,
}
impl Default for EmbeddedStore {
    fn default() -> Self {
        log::debug!("Embedded Asset Store init.");
        Self {
            embedded: get_embedded(),
            assets: Arena::new(),
        }
    }
}
impl AssetContext for EmbeddedStore {
    fn load_bytes(&mut self, data: &'static [u8]) -> ResourceId<Asset> {
        self.assets.insert(Asset::borrowed(data))
    }
    fn load(&mut self, path: &str) -> Result<ResourceId<Asset>, AssetError> {
        let data = self.embedded.get(path).ok_or(AssetError::PathError(
            path.to_string(),
            "embedded path not found".to_string(),
        ))?;

        log::debug!(
            "Loaded embedded asset from: {}. {} bytes.",
            path,
            data.len()
        );
        Ok(self.assets.insert(Asset::borrowed(data)))
    }
    fn get(&self, asset_id: ResourceId<Asset>) -> Option<&Asset> {
        self.assets.get(&asset_id)
    }
}
