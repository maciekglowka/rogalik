use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use rogalik_arena::{Arena, Id};
use rogalik_assets::{AssetContext, AssetStore};

use crate::{source::AudioSource, AudioError};

pub(crate) struct AudioAssets {
    asset_store: Arc<Mutex<AssetStore>>,
    source_names: HashMap<String, Id<AudioSource>>, // lookup
    pub(crate) sources: Arena<AudioSource>,
}
impl AudioAssets {
    pub(crate) fn new(asset_store: Arc<Mutex<AssetStore>>) -> Self {
        Self {
            asset_store,
            source_names: HashMap::new(),
            sources: Arena::new(),
        }
    }
    pub(crate) fn load_source(&mut self, name: &str, path: &str) -> Result<(), AudioError> {
        let mut store = self
            .asset_store
            .lock()
            .expect("Can't acquire the asset store!");

        let asset_id = store.load(path)?;

        let source = AudioSource::new(asset_id, &store)?;

        let source_id = self.sources.insert(source);
        self.source_names.insert(name.to_string(), source_id);

        Ok(())
    }
    pub(crate) fn with_source_mut(
        &mut self,
        name: &str,
        mut f: impl FnMut(&mut AudioSource),
    ) -> Result<(), AudioError> {
        let source = self
            .sources
            .get_mut(
                self.source_names
                    .get(name)
                    .ok_or(AudioError::SourceNotFound(name.to_string()))?,
            )
            .ok_or(AudioError::SourceNotFound(name.to_string()))?;
        f(source);
        Ok(())
    }
    pub(crate) fn update_assets(&mut self) {
        let mut store = self
            .asset_store
            .lock()
            .expect("Can't acquire the asset store");
        for source in self.sources.values_mut() {
            source.check_update(&mut store);
        }
    }
}
