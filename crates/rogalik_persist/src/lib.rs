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

trait KVStore<T: DeserializeOwned + Serialize> {
    fn load(key: &str, path: Option<&str>) -> Result<T>;
    fn store(key: &str, value: &T, path: Option<&str>) -> Result<()>;
}