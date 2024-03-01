use std::{
    any::{Any, TypeId},
    collections::HashMap
};
use serde::{
    Serialize,
    de::DeserializeOwned
};

use crate::{
    component::Component,
    component_storage::{ComponentCell, ComponentStorage}
};


pub(crate) struct ComponentRegistry {
    pub tags: HashMap<TypeId, String>,
    pub type_ids: HashMap<String, TypeId>,
    pub serializers: HashMap<
        TypeId,
        Box<dyn Fn(&Box<dyn ComponentStorage>) -> String>
    >,
    pub deserializers: HashMap<
        TypeId,
        Box<dyn Fn(&str) -> Box<dyn ComponentStorage>>
    >
}
impl ComponentRegistry {
    pub fn new() -> Self {
        Self {
            tags: HashMap::new(),
            type_ids: HashMap::new(),
            serializers: HashMap::new(),
            deserializers: HashMap::new(),
        }
    }
    pub fn register<T>(&mut self, tag: &str)
    where T: Component + DeserializeOwned + Serialize + 'static {
        let type_id = TypeId::of::<T>();
        self.tags.insert(type_id, tag.to_string());
        self.type_ids.insert(tag.to_string(), type_id);

        let tag_str = tag.to_string();
        let serializer = move |val: &Box<dyn ComponentStorage>| {
            let c = val.as_any().downcast_ref::<ComponentCell<T>>()
                .expect(&format!("Can't downcast {}", tag_str));
            serde_yaml::to_string(c)
                .expect(&format!("Can't serialize {}", tag_str))
        };
        self.serializers.insert(type_id, Box::new(serializer));

        let tag_str = tag.to_string();
        let deserializer = move |val: &str| {
            Box::new(
                serde_yaml::from_str::<ComponentCell<T>>(val)
                    .expect(&format!("Can't deserialize {}", tag_str))
            ) as Box<dyn ComponentStorage>
        };
        self.deserializers.insert(type_id, Box::new(deserializer));
    }
}