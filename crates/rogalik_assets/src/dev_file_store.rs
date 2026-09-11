use std::{
    collections::HashMap,
    fs::{self},
    path::Path,
};

use rogalik_arena::{Arena, Id};

use crate::{Asset, AssetBytes, AssetContext, AssetError, AssetState};

include!(env!("ROGALIK_ASSET_FILE"));

pub struct DevFileStore {
    assets: Arena<Asset>,
    meta: HashMap<Id<Asset>, FileAssetMeta>,
    root: String,
}
impl Default for DevFileStore {
    fn default() -> Self {
        log::debug!("Dev Asset Store init.");
        Self {
            assets: Arena::new(),
            meta: HashMap::new(),
            root: ASSET_ROOT.to_string(),
        }
    }
}
impl DevFileStore {
    pub fn reload_modified(&mut self) {
        log::debug!("Reloading the assets");
        for (id, asset) in self.assets.iter_mut() {
            // skips assets loaded from memory
            let Some(meta) = self.meta.get_mut(&id) else {
                continue;
            };

            let Ok(file_meta) = fs::metadata(meta.path.as_path()) else {
                continue;
            };
            let Ok(modified) = get_modified_u64(&file_meta) else {
                continue;
            };
            if modified == meta.modified {
                continue;
            };
            if let Ok(data) = fs::read(&meta.path) {
                asset.data = AssetBytes::Owned(data);
                asset.state = AssetState::Updated;
                meta.modified = modified;
                log::debug!("Reloaded {:?}", meta.path);
            }
        }
    }
}
impl AssetContext for DevFileStore {
    fn load_bytes(&mut self, data: &'static [u8]) -> Id<Asset> {
        self.assets.insert(Asset::borrowed(data))
    }
    fn load(&mut self, path: &str) -> Result<Id<Asset>, AssetError> {
        let abs_path = Path::new(&self.root).join(path);
        let data = fs::read(&abs_path)
            .map_err(|e| AssetError::PathError(path.to_string(), e.to_string()))?;

        let meta = fs::metadata(abs_path.as_path())
            .map_err(|e| AssetError::MetaDataError(e.to_string()))?;
        let modified = get_modified_u64(&meta)?;

        log::debug!("Loaded asset from: {}. {} bytes.", path, data.len());
        let id = self.assets.insert(Asset::owned(data));
        self.meta.insert(
            id,
            FileAssetMeta {
                path: abs_path,
                modified,
            },
        );
        Ok(id)
    }
    fn get(&self, asset_id: Id<Asset>) -> Option<&Asset> {
        self.assets.get(&asset_id)
    }
    fn mark_read(&mut self, asset_id: Id<Asset>) {
        if let Some(asset) = self.assets.get_mut(&asset_id) {
            asset.state = AssetState::Loaded;
        }
    }
}
struct FileAssetMeta {
    path: std::path::PathBuf,
    modified: u64,
}

fn get_modified_u64(meta: &std::fs::Metadata) -> Result<u64, AssetError> {
    Ok(meta
        .modified()
        .map_err(|e| AssetError::MetaDataError(e.to_string()))?
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .map_err(|e| AssetError::MetaDataError(e.to_string()))?
        .as_secs())
}
