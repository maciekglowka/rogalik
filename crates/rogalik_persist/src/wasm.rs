use anyhow::{anyhow, Result};
use base64::prelude::*;
use serde::{de::DeserializeOwned, Serialize};
use serde_json;

use crate::KVStore;

pub struct WasmStore;
impl WasmStore {
    fn get_storage() -> Result<web_sys::Storage> {
        let window = web_sys::window().ok_or(anyhow!("Window not available"))?;
        window.local_storage()
            .map_err(|_| anyhow!("Storage not available"))?
            .ok_or(anyhow!("Storage not available"))
    }
}
impl KVStore for WasmStore {
    fn load<T: DeserializeOwned + Serialize>(key: &str, path: Option<&str>) -> Result<T> {
        let storage = WasmStore::get_storage()?;
        let item = storage.get_item(key)
            .map_err(|_| anyhow!("Item not available"))?
            .ok_or(anyhow!("Item not available"))?;
        Ok(serde_json::from_str::<T>(&item)?)
    }
    fn store<T: DeserializeOwned + Serialize>(key: &str, value: &T, path: Option<&str>) -> Result<()> {
        let storage = WasmStore::get_storage()?;
        let json = serde_json::to_string(value)?;
        storage.set_item(key, &json)
            .map_err(|_| anyhow!("Store attempt failed", ))?;
        Ok(())
    }
    fn load_raw(key: &str, path: Option<&str>) -> Result<Vec<u8>> {
        let storage = WasmStore::get_storage()?;
        let item = storage.get_item(key)
            .map_err(|_| anyhow!("Item not available"))?
            .ok_or(anyhow!("Item not available"))?;
        Ok(BASE64_STANDARD.decode(item)?)
    }
    fn store_raw(key: &str, value: &[u8], path: Option<&str>) -> Result<()> {
        let storage = WasmStore::get_storage()?;
        let s = BASE64_STANDARD.encode(value);
        storage.set_item(key, &s)
            .map_err(|_| anyhow!("Store attempt failed", ))?;
        Ok(())
    }
    fn remove(key: &str, path: Option<&str>) -> Result<()> {
        let storage = WasmStore::get_storage()?;
        storage.remove_item(key)
            .map_err(|_| anyhow!("Remove attempt failed", ))?;;
        Ok(())
    }
}