use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use rogalik_assets::{AssetContext, AssetStore};
use rogalik_common::{EngineError, ResourceId};

use crate::source::AudioSource;

pub(crate) struct AudioAssets {
    asset_store: Arc<Mutex<AssetStore>>,
    source_names: HashMap<String, ResourceId>, // lookup
    pub(crate) sources: Vec<AudioSource>,
}
impl AudioAssets {
    pub(crate) fn new(asset_store: Arc<Mutex<AssetStore>>) -> Self {
        Self {
            asset_store,
            source_names: HashMap::new(),
            sources: Vec::new(),
        }
    }
    pub(crate) fn load_source(&mut self, name: &str, path: &str) -> Result<(), EngineError> {
        let mut store = self
            .asset_store
            .lock()
            .expect("Can't acquire the asset store!");

        let asset_id = store.load(path).expect(&format!("Can't load {path}!"));

        let source = AudioSource::new(asset_id, &store)?;

        let source_id = ResourceId(self.sources.len());
        self.sources.push(source);
        self.source_names.insert(name.to_string(), source_id);

        Ok(())
    }
    pub(crate) fn with_source_mut(
        &mut self,
        name: &str,
        mut f: impl FnMut(&mut AudioSource),
    ) -> Result<(), EngineError> {
        let source = self
            .sources
            .get_mut(
                self.source_names
                    .get(name)
                    .ok_or(EngineError::ResourceNotFound)?
                    .0,
            )
            .ok_or(EngineError::ResourceNotFound)?;
        f(source);
        Ok(())
    }
    pub(crate) fn update_assets(&mut self) {
        let mut store = self
            .asset_store
            .lock()
            .expect("Can't acquire the asset store");
        for source in self.sources.iter_mut() {
            source.check_update(&mut store);
        }
    }
}
