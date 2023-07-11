use std::any::{Any, TypeId};

use rogalik_events;

mod component;
mod component_storage;
mod entity;
mod errors;
pub mod query;
mod resource;
mod world;

pub use component::Component;
pub use entity::Entity;
pub use component_storage::ComponentSet;
pub use world::World;


pub trait Storage {
    fn as_any(&self) -> &dyn Any;
}


#[derive(Clone, Copy)]
pub enum WorldEvent {
    ComponentSpawned(Entity, TypeId),
    ComponentRemoved(Entity, TypeId)
}