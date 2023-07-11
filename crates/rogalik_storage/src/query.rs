use std::{
    cell::{Ref, RefMut},
    collections::HashSet
};
use super::{Component, Entity, World};

pub struct EntityQuery<'a> {
    world: &'a World,
    inner: HashSet<Entity>
}
impl<'a> EntityQuery<'a> {
    pub fn new<T: 'static + Component>(world: &World) -> EntityQuery {
        let entities = match world.get_component_set::<T>() {
            Some(c) => c.hashset(),
            _ => HashSet::new()
        };
        EntityQuery { inner: entities, world }
    }
    pub fn with<T: 'static + Component>(self) -> EntityQuery<'a> {
        let h = match self.world.get_component_set::<T>() {
            Some(c) => c.hashset(),
            _ => HashSet::new()
        };
        let entities = self.inner.intersection(&h);
        EntityQuery {
            inner: entities.map(|e| *e).collect(),
            world: self.world
        }
    }
    pub fn iter(&self) -> EntityQueryIterator<'_> {
        EntityQueryIterator { inner: self.inner.iter(), world: self.world }
    }
}
pub struct EntityQueryIterator<'a> {
    inner: std::collections::hash_set::Iter<'a, Entity>,
    world: &'a World
}
impl<'a> Iterator for EntityQueryIterator<'a> {
    type Item = EntityQueryItem<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        Some(EntityQueryItem { 
            entity: *self.inner.next()?,
            world: self.world
        })
    }
}

pub struct EntityQueryItem<'a> {
    pub entity: Entity,
    world: &'a World
}
impl<'a> EntityQueryItem<'a> {
    // get and get_mut does not check at the moment
    // if the requested type was part of the query
    pub fn get<T: 'static + Component>(&self) -> Option<Ref<'_, T>> {
        // TODO avoid retrieving the set each time?
        let set = self.world.get_component_set::<T>()?;
        Ref::filter_map(set, |s| s.get(self.entity)).ok()
    }
    pub fn get_mut<T: 'static + Component>(&self) -> Option<RefMut<'_, T>> {
        // TODO avoid retrieving the set each time?
        let set = self.world.get_component_set_mut::<T>()?;
        RefMut::filter_map(set, |s| s.get_mut(self.entity)).ok()
    }
}
