use std::{
    collections::HashMap,
    fs::{self, File},
    io::prelude::*,
    path::Path,
};

use rogalik_common::{EngineError, ResourceId};

use super::{Asset, AssetState, AssetStoreTrait, ROOT_VAR};

pub struct DevFileStore {
    next_id: ResourceId,
    assets: HashMap<ResourceId, Asset>,
    meta: HashMap<ResourceId, FileAssetMeta>,
    root: String,
}
impl Default for DevFileStore {
    fn default() -> Self {
        Self {
            next_id: ResourceId(0),
            assets: HashMap::new(),
            meta: HashMap::new(),
            root: std::env::var(ROOT_VAR).expect(&format!("{} variable not set!", ROOT_VAR)),
        }
    }
}
impl DevFileStore {
    pub fn reload_modified(&mut self) {
        log::debug!("Reloading the assets");
        for (id, asset) in self.assets.iter_mut() {
            // skips assets loaded from memory
            let Some(meta) = self.meta.get_mut(id) else {
                continue;
            };

            let Ok(file_meta) = fs::metadata(&meta.path.as_path()) else {
                continue;
            };
            let Ok(modified) = get_modified_u64(&file_meta) else {
                continue;
            };
            if modified == meta.modified {
                continue;
            };
            if let Ok(data) = fs::read(&meta.path) {
                asset.data = data;
                asset.state = AssetState::Updated;
                meta.modified = modified;
                log::debug!("Reloaded {:?}", meta.path);
            }
        }
    }
    pub fn mark_read(&mut self, asset_id: ResourceId) {
        if let Some(asset) = self.assets.get_mut(&asset_id) {
            asset.state = AssetState::Loaded;
        }
    }
    fn bump_id(&mut self) {
        self.next_id = self.next_id.next();
    }
}
impl AssetStoreTrait for DevFileStore {
    fn from_bytes(&mut self, data: &[u8]) -> ResourceId {
        let id = self.next_id;
        self.assets.insert(id, Asset::new(data));
        self.bump_id();
        id
    }
    fn load(&mut self, path: &str) -> Result<ResourceId, EngineError> {
        let id = self.next_id;

        let abs_path = Path::new(&self.root).join(path);
        let data = fs::read(&abs_path).map_err(|_| EngineError::ResourceNotFound)?;

        let meta = fs::metadata(&abs_path.as_path()).map_err(|_| EngineError::ResourceNotFound)?;
        let modified = get_modified_u64(&meta)?;

        self.assets.insert(id, Asset::new(&data));
        self.meta.insert(
            id,
            FileAssetMeta {
                path: abs_path,
                modified,
            },
        );
        self.bump_id();
        log::debug!("Loaded asset from: {}. {} bytes.", path, data.len());
        Ok(id)
    }
    fn get(&self, asset_id: ResourceId) -> Option<&Asset> {
        self.assets.get(&asset_id)
    }
}
struct FileAssetMeta {
    path: std::path::PathBuf,
    modified: u64,
}

fn get_modified_u64(meta: &std::fs::Metadata) -> Result<u64, EngineError> {
    Ok(meta
        .modified()
        .map_err(|_| EngineError::ResourceNotFound)?
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .map_err(|_| EngineError::ResourceNotFound)?
        .as_secs())
}
