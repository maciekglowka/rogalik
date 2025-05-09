use std::{
    any::TypeId,
    cell::{Ref, RefCell, RefMut},
    collections::{HashMap, HashSet},
};

use rogalik_events::EventBus;

use crate::components::Components;
use crate::entity::{Entities, Entity};
use crate::errors::WorldError;
use crate::query::{IntoQuery, Query};

#[derive(Default)]
pub struct World {
    entities: Entities,
    components: Components,
}
impl World {
    pub fn new() -> Self {
        Self {
            entities: Entities::new(),
            components: Components::new(),
        }
    }

    // entities

    pub fn spawn(&mut self) -> Entity {
        self.entities.spawn()
    }
    pub fn despawn(&mut self, entity: Entity) {
        self.entities.despawn(entity);
        // for (type_id, storage) in self.component_storage.iter() {
        //     // TODO
        //     if storage.remove_untyped(entity).is_some() {}
        // }
    }

    // components

    pub fn insert<T: 'static>(&mut self, entity: Entity, value: T) {
        self.components.insert(entity, value);
    }

    // query

    pub fn query<'a, T: IntoQuery>(&'a self) -> Query<'a, T> {
        Query::new(&self.components)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_two() {
        let mut world = World::new();

        let a = world.spawn();
        let b = world.spawn();
        let c = world.spawn();

        world.insert(a, 235);
        world.insert(b, 24);
        world.insert(c, 25);

        world.insert(a, "A".to_string());
        world.insert(c, "C".to_string());

        let query = world.query::<(i32, String)>();
        let v = query.iter().collect::<Vec<_>>();

        assert_eq!(2, v.len());
        assert!(v.contains(&(&235, &"A".to_string())));
        assert!(v.contains(&(&25, &"C".to_string())));
    }
}
