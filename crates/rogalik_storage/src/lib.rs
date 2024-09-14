use std::any::{Any, TypeId};

mod component;
mod component_storage;
mod entity;
mod errors;
mod resource;
#[cfg(feature = "serialize")]
mod serialize;
mod world;

pub use component::Component;
pub use component_storage::ComponentSet;
pub use entity::Entity;
pub use world::World;

pub trait Storage {
    fn as_any(&self) -> &dyn Any;
}

#[derive(Clone, Copy)]
pub enum WorldEvent {
    ComponentSpawned(Entity, TypeId),
    ComponentRemoved(Entity, TypeId),
}
