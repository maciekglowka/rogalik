use serde::{de::DeserializeOwned, Serialize};
use std::{any::TypeId, collections::HashMap};

use crate::{
    component::Component,
    component_storage::{ComponentCell, ComponentStorage},
    errors::WorldError,
    resource::ResourceCell,
    world::World,
    Storage,
};

pub(crate) struct StorageRegistry<S> {
    pub tags: HashMap<TypeId, String>,
    pub type_ids: HashMap<String, TypeId>,
    pub serializers: HashMap<TypeId, Box<dyn Fn(&S) -> Result<Vec<u8>, WorldError>>>,
    pub deserializers: HashMap<TypeId, Box<dyn Fn(&[u8]) -> Result<S, WorldError>>>,
}
impl<S> StorageRegistry<S> {
    pub fn new() -> Self {
        Self {
            tags: HashMap::new(),
            type_ids: HashMap::new(),
            serializers: HashMap::new(),
            deserializers: HashMap::new(),
        }
    }
    fn register_tag<T>(&mut self, tag: &str) -> TypeId
    where
        T: DeserializeOwned + Serialize + 'static,
    {
        let type_id = TypeId::of::<T>();
        self.tags.insert(type_id, tag.to_string());
        self.type_ids.insert(tag.to_string(), type_id);
        type_id
    }
}
impl<S> Default for StorageRegistry<S> {
    fn default() -> Self {
        Self {
            tags: HashMap::new(),
            type_ids: HashMap::new(),
            serializers: HashMap::new(),
            deserializers: HashMap::new(),
        }
    }
}

impl StorageRegistry<Box<dyn ComponentStorage>> {
    pub fn register<T>(&mut self, tag: &str)
    where
        T: Component + DeserializeOwned + Serialize + 'static,
    {
        let type_id = self.register_tag::<T>(tag);

        let tag_str = tag.to_string();
        let serializer = move |val: &Box<dyn ComponentStorage>| {
            let c = val.as_any().downcast_ref::<ComponentCell<T>>().ok_or(
                WorldError::SerializationError(format!("Can't serialize {}", tag_str)),
            )?;
            Ok(bincode::serialize(c).map_err(|_| {
                WorldError::SerializationError(format!("Can't serialize {}", tag_str))
            })?)
        };
        self.serializers.insert(type_id, Box::new(serializer));

        let tag_str = tag.to_string();
        let deserializer = move |val: &[u8]| {
            Ok(
                Box::new(bincode::deserialize::<ComponentCell<T>>(val).map_err(|_| {
                    WorldError::DeserializationError(format!("Can't deserialize {}", tag_str))
                })?) as Box<dyn ComponentStorage>,
            )
        };
        self.deserializers.insert(type_id, Box::new(deserializer));
    }
}

impl StorageRegistry<Box<dyn Storage>> {
    pub fn register<T>(&mut self, tag: &str)
    where
        T: DeserializeOwned + Serialize + 'static,
    {
        let type_id = self.register_tag::<T>(tag);

        let tag_str = tag.to_string();
        let serializer = move |val: &Box<dyn Storage>| {
            let c = val.as_any().downcast_ref::<ResourceCell<T>>().ok_or(
                WorldError::SerializationError(format!("Can't serialize {}", tag_str)),
            )?;
            Ok(bincode::serialize(c).map_err(|_| {
                WorldError::SerializationError(format!("Can't serialize {}", tag_str))
            })?)
        };
        self.serializers.insert(type_id, Box::new(serializer));

        let tag_str = tag.to_string();
        let deserializer = move |val: &[u8]| {
            Ok(
                Box::new(bincode::deserialize::<ResourceCell<T>>(val).map_err(|_| {
                    WorldError::DeserializationError(format!("Can't deserialize {}", tag_str))
                })?) as Box<dyn Storage>,
            )
        };
        self.deserializers.insert(type_id, Box::new(deserializer));
    }
}

impl World {
    pub fn serialize(&self) -> Result<Vec<u8>, WorldError> {
        let mut map = HashMap::new();

        map.insert("entities", self.serialize_entities()?);
        map.insert("resources", self.serialize_resources()?);
        map.insert("components", self.serialize_components()?);

        Ok(bincode::serialize(&map).map_err(|_| {
            WorldError::SerializationError("Can't serialize the world!".to_string())
        })?)
    }
    pub fn deserialize(&mut self, data: &[u8]) -> Result<(), WorldError> {
        let map: HashMap<String, &[u8]> =
            bincode::deserialize(data).expect("Can't deserialize world!");

        if let Some(entity_data) = map.get("entities") {
            self.deserialize_entities(entity_data)?;
        }
        if let Some(component_data) = map.get("components") {
            self.deserialize_components(component_data)?;
        }
        if let Some(resource_data) = map.get("resources") {
            self.deserialize_resources(resource_data)?;
        }
        Ok(())
    }

    // entities

    fn serialize_entities(&self) -> Result<Vec<u8>, WorldError> {
        Ok(bincode::serialize(&self.entity_storage).map_err(|_| {
            WorldError::SerializationError("Can't serialize the entities!".to_string())
        })?)
    }

    fn deserialize_entities(&mut self, data: &[u8]) -> Result<(), WorldError> {
        self.entity_storage = bincode::deserialize(data).map_err(|_| {
            WorldError::DeserializationError("Can't deserialize the entities!".to_string())
        })?;
        Ok(())
    }

    // components

    pub fn register_serializable_component<T>(&mut self, tag: &str)
    where
        T: Component + DeserializeOwned + Serialize + 'static,
    {
        self.component_registry.register::<T>(tag);
    }

    fn serialize_components(&self) -> Result<Vec<u8>, WorldError> {
        let mut map = HashMap::new();
        for (type_id, val) in self.component_storage.iter() {
            let Some(f) = self.component_registry.serializers.get(type_id) else {
                continue;
            };
            let Some(tag) = self.component_registry.tags.get(type_id) else {
                continue;
            };
            let s = f(val)?;
            map.insert(tag.to_string(), s);
        }
        Ok(bincode::serialize(&map).map_err(|_| {
            WorldError::SerializationError("Can't serialize component map!".to_string())
        })?)
    }

    fn deserialize_components(&mut self, data: &[u8]) -> Result<(), WorldError> {
        let map: HashMap<String, &[u8]> = bincode::deserialize(data).map_err(|_| {
            WorldError::DeserializationError("Can't deserialize component map!".to_string())
        })?;
        for (tag, value) in map.iter() {
            let Some(type_id) = self.component_registry.type_ids.get(tag) else {
                continue;
            };
            let Some(f) = self.component_registry.deserializers.get(type_id) else {
                continue;
            };
            let c = f(value)?;
            self.component_storage.insert(*type_id, c);
        }
        Ok(())
    }

    // resources

    pub fn register_serializable_resource<T>(&mut self, tag: &str)
    where
        T: DeserializeOwned + Serialize + 'static,
    {
        self.resource_registry.register::<T>(tag);
    }

    fn serialize_resources(&self) -> Result<Vec<u8>, WorldError> {
        let mut map = HashMap::new();
        for (type_id, val) in self.resource_storage.iter() {
            let Some(f) = self.resource_registry.serializers.get(type_id) else {
                continue;
            };
            let Some(tag) = self.resource_registry.tags.get(type_id) else {
                continue;
            };
            let s = f(val)?;
            map.insert(tag.to_string(), s);
        }
        Ok(bincode::serialize(&map).map_err(|_| {
            WorldError::SerializationError("Can't serialize resource map!".to_string())
        })?)
    }

    fn deserialize_resources(&mut self, data: &[u8]) -> Result<(), WorldError> {
        let map: HashMap<String, &[u8]> = bincode::deserialize(data).map_err(|_| {
            WorldError::DeserializationError("Can't deserialize resource map!".to_string())
        })?;
        for (tag, value) in map.iter() {
            let Some(type_id) = self.resource_registry.type_ids.get(tag) else {
                continue;
            };
            let Some(f) = self.resource_registry.deserializers.get(type_id) else {
                continue;
            };
            let c = f(value)?;
            self.resource_storage.insert(*type_id, c);
        }
        Ok(())
    }
}
