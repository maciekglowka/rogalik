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
        let mut file = File::open(&abs_path).map_err(|_| EngineError::ResourceNotFound)?;
        let mut data = Vec::new();
        file.read(&mut data)
            .map_err(|_| EngineError::ResourceNotFound)?;

        let meta = fs::metadata(&abs_path.as_path()).map_err(|_| EngineError::ResourceNotFound)?;
        let modified = meta
            .modified()
            .map_err(|_| EngineError::ResourceNotFound)?
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .map_err(|_| EngineError::ResourceNotFound)?
            .as_secs();

        self.assets.insert(id, Asset::new(&data));
        self.meta.insert(
            id,
            FileAssetMeta {
                path: abs_path,
                modified,
            },
        );
        self.bump_id();
        log::debug!("Loaded asset from: {}", path);
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
