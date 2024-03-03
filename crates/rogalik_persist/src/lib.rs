use anyhow::Result;
use serde::{de::DeserializeOwned, Serialize};

#[cfg(not(target_arch = "wasm32"))]
mod file;
#[cfg(target_arch = "wasm32")]
mod wasm;

#[cfg(not(target_arch = "wasm32"))]
type Store = file::FileStore;

#[cfg(target_arch = "wasm32")]
type Store = wasm::WasmStore;

pub fn load<T: DeserializeOwned + Serialize>(key: &str, path: Option<&str>) -> Result<T> {
    Store::load(key, path)
}
pub fn store<T: DeserializeOwned + Serialize>(key: &str, value: &T, path: Option<&str>) -> Result<()> {
    Store::store(key, value, path)
}
pub fn load_raw(key: &str, path: Option<&str>) -> Result<Vec<u8>> {
    Store::load_raw(key, path)
}
pub fn store_raw(key: &str, value: &[u8], path: Option<&str>) -> Result<()> {
    Store::store_raw(key, value, path)
}
pub fn remove(key: &str, path: Option<&str>) -> Result<()> {
    Store::remove(key, path)
}

trait KVStore {
    fn load<T: DeserializeOwned + Serialize>(key: &str, path: Option<&str>) -> Result<T>;
    fn store<T: DeserializeOwned + Serialize>(key: &str, value: &T, path: Option<&str>) -> Result<()>;
    fn load_raw(key: &str, path: Option<&str>) -> Result<Vec<u8>>;
    fn store_raw(key: &str, value: &[u8], path: Option<&str>) -> Result<()>;
    fn remove(key: &str, path: Option<&str>) -> Result<()>;
}