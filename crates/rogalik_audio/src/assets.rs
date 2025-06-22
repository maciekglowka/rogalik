use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use rogalik_assets::{AssetContext, AssetState, AssetStore};
use rogalik_common::{EngineError, ResourceId};

use crate::source::AudioSource;

pub(crate) struct AudioAssets {
    asset_store: Arc<Mutex<AssetStore>>,
    source_names: HashMap<String, ResourceId>, // lookup
    pub(crate) sources: Arc<Mutex<Vec<AudioSource>>>,
}
impl AudioAssets {
    pub fn new(asset_store: Arc<Mutex<AssetStore>>) -> Self {
        Self {
            asset_store,
            source_names: HashMap::new(),
            sources: Arc::new(Mutex::new(Vec::new())),
        }
    }
    pub fn load_source(&mut self, name: &str, path: &str) -> Result<(), EngineError> {
        let mut store = self
            .asset_store
            .lock()
            .expect("Can't acquire the asset store!");

        let asset_id = store.load(path).expect(&format!("Can't load {}!", path));

        let source = AudioSource::new(asset_id, &store)?;
        let mut sources = self.sources.lock().unwrap();

        let source_id = ResourceId(sources.len());
        sources.push(source);
        self.source_names.insert(name.to_string(), source_id);

        Ok(())
    }
    pub fn with_source_mut(
        &self,
        name: &str,
        mut f: impl FnMut(&mut AudioSource),
    ) -> Result<(), EngineError> {
        let mut sources = self.sources.lock().unwrap();
        let source = sources
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
}
