use anyhow::Result;
use serde::{de::DeserializeOwned, Serialize};
use std::{
    env::current_dir,
    fs::{File, remove_file},
    io::{Read, Write},
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

impl KVStore for FileStore {
    fn load<T: DeserializeOwned + Serialize>(key: &str, path: Option<&str>) -> Result<T> {
        let f = File::open(FileStore::get_path(key, path)?)?;
        Ok(bincode::deserialize_from(f)?)
    }
    fn store<T: DeserializeOwned + Serialize>(key: &str, value: &T, path: Option<&str>) -> Result<()> {
        let mut f = File::create(FileStore::get_path(key, path)?)?;
        bincode::serialize_into(&mut f, value)?;
        Ok(())
    }
    fn load_raw(key: &str, path: Option<&str>) -> Result<Vec<u8>> {
        let mut f = File::open(FileStore::get_path(key, path)?)?;
        let mut buf = Vec::new();
        f.read_to_end(&mut buf)?;
        Ok(buf)
    }
    fn store_raw(key: &str, value: &[u8], path: Option<&str>) -> Result<()> {
        let mut f = File::create(FileStore::get_path(key, path)?)?;
        f.write(value)?;
        Ok(())
    }
    fn remove(key: &str, path: Option<&str>) -> Result<()> {
        Ok(remove_file(FileStore::get_path(key, path)?)?)
    }
}