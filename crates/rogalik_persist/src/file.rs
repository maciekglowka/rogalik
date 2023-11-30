use anyhow::Result;
use serde::{de::DeserializeOwned, Serialize};
use std::{
    env::current_dir,
    fs::File,
    path::PathBuf
};

use crate::KVStore;

pub struct FileStore;
impl FileStore {
    fn get_path(key: &str, base: Option<&str>) -> Result<PathBuf> {
        let dir = match base {
            Some(a) => a.into(),
            None => current_dir()?
        };
        Ok(dir.as_path().join(format!("{}.bin", key)))
    }
}
impl<T: DeserializeOwned + Serialize> KVStore<T> for FileStore {
    fn load(key: &str, path: Option<&str>) -> Result<T> {
        let f = File::open(FileStore::get_path(key, path)?)?;
        Ok(bincode::deserialize_from(f)?)
    }
    fn store(key: &str, value: &T, path: Option<&str>) -> Result<()> {
        let mut f = File::create(FileStore::get_path(key, path)?)?;
        bincode::serialize_into(&mut f, value)?;
        Ok(())
    }
}