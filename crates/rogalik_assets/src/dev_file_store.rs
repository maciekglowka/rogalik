use std::{
    collections::HashMap,
    fs::{self, File},
    io::prelude::*,
    path::Path,
};

use rogalik_common::ResourceId;

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
impl AssetStoreTrait for DevFileStore {
    fn load(&mut self, path: &str) -> Option<ResourceId> {
        let id = self.next_id;

        let abs_path = Path::new(&self.root).join(path);
        let mut file = File::open(&abs_path).ok()?;
        let mut data = Vec::new();
        file.read(&mut data).ok()?;

        let meta = fs::metadata(&abs_path.as_path()).ok()?;
        let modified = meta
            .modified()
            .ok()?
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .ok()?
            .as_secs();

        self.assets.insert(id, Asset::new(&data));
        self.meta.insert(
            id,
            FileAssetMeta {
                path: abs_path,
                modified,
            },
        );
        self.next_id = id.next();
        log::debug!("Loaded asset from: {}", path);
        Some(id)
    }
    fn get(&self, asset_id: ResourceId) -> Option<&Asset> {
        self.assets.get(&asset_id)
    }
}
struct FileAssetMeta {
    path: std::path::PathBuf,
    modified: u64,
}
