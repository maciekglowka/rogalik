#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};
use std::{
    any::{Any, TypeId},
    collections::{HashMap, HashSet},
};

use super::entity::{Entity, IdSize};
use super::errors::WorldError;

const TOMBSTONE: IdSize = IdSize::MAX;

#[derive(Default)]
pub struct Components {
    pub(crate) storage: ComponentStorage,
}
impl Components {
    pub(crate) fn new() -> Self {
        Self {
            storage: HashMap::new(),
        }
    }
    pub(crate) fn get_set<T: 'static>(&self) -> &ComponentSet<T> {
        let type_id = TypeId::of::<T>();
        let c = self.storage.get(&type_id).unwrap();
        (&**c as &dyn Any).downcast_ref().unwrap()
    }

    pub(crate) fn get_set_mut<T: 'static>(&mut self) -> &mut ComponentSet<T> {
        let type_id = TypeId::of::<T>();
        let c = self.storage.get_mut(&type_id).unwrap();
        (&mut **c as &mut dyn Any).downcast_mut().unwrap()
    }

    pub(crate) fn get<T: 'static>(&self, entity: Entity) -> Option<&T> {
        self.get_set().get(entity)
    }

    pub(crate) fn get_mut<T: 'static>(&mut self, entity: Entity) -> Option<&mut T> {
        self.get_set_mut().get_mut(entity)
    }

    pub(crate) fn insert<T: 'static>(&mut self, entity: Entity, value: T) {
        if !self.storage.contains_key(&TypeId::of::<T>()) {
            self.insert_set::<T>();
        }
        self.get_set_mut().insert(entity, value);
    }

    fn insert_set<T: 'static>(&mut self) {
        self.storage
            .insert(TypeId::of::<T>(), Box::new(ComponentSet::<T>::new()));
    }
}

pub(crate) type ComponentStorage = HashMap<TypeId, Box<dyn ComponentSetErased>>;

pub(crate) trait ComponentSetErased: Any {
    fn get(&self, entity: Entity) -> Option<&dyn Any>;
    fn get_mut(&mut self, entity: Entity) -> Option<&mut dyn Any>;
    fn entities(&self) -> HashSet<Entity>;
}

impl<T: 'static> ComponentSetErased for ComponentSet<T> {
    fn get(&self, entity: Entity) -> Option<&dyn Any> {
        self.get(entity).map(|a| a as &dyn Any)
    }
    fn get_mut(&mut self, entity: Entity) -> Option<&mut dyn Any> {
        self.get_mut(entity).map(|a| a as &mut dyn Any)
    }
    fn entities(&self) -> HashSet<Entity> {
        self.entities()
    }
}

pub struct ComponentSet<T> {
    dense: Vec<Entity>,
    sparse: Vec<IdSize>,
    values: Vec<T>,
}
impl<T> ComponentSet<T> {
    fn new() -> Self {
        Self {
            dense: Vec::new(),
            sparse: Vec::new(),
            values: Vec::new(),
        }
    }
    pub fn get(&self, entity: Entity) -> Option<&T> {
        self.values.get(self.get_dense_index(entity)?)
    }
    pub fn get_mut(&mut self, entity: Entity) -> Option<&mut T> {
        let i = self.get_dense_index(entity)?;
        self.values.get_mut(i)
    }
    // Return currently stored entities
    pub fn entities(&self) -> HashSet<Entity> {
        HashSet::from_iter(self.dense.iter().copied())
    }
    // Insert a new component for the entity.
    // Overwrite if already exists.
    pub fn insert(&mut self, entity: Entity, value: T) {
        // check if replacement
        if let Some(index) = self.get_dense_index(entity) {
            self.values[index] = value;
            return;
        }

        let index = entity.id as usize;
        if index >= self.sparse.len() {
            // fill empty values with tombstones
            self.sparse.resize(index + 1, TOMBSTONE);
        }

        // sparse array points to the element in the dense one
        self.sparse[index] = self.dense.len() as IdSize;
        // we push the element at the end of the dense array
        self.dense.push(entity);
        // components array is kept in sync with the dense array
        self.values.push(value);
    }

    // Removes component for a given entity
    // Keeps the values densely packed
    pub fn remove(&mut self, entity: Entity) -> Option<T> {
        let removed_idx = self.get_dense_index(entity)?;

        // we are going to swap the removed value with the last one first
        let last_idx = self.dense.len() - 1;
        let swapped_sparse_idx = self.dense[last_idx].id as usize;

        self.dense.swap(removed_idx, last_idx);
        self.values.swap(removed_idx, last_idx);

        // now remove the last element
        let _ = self.dense.pop();
        let removed = self.values.pop();

        // now fix the sparse vec
        self.sparse[swapped_sparse_idx] = removed_idx as IdSize;
        self.sparse[entity.id as usize] = TOMBSTONE;

        removed
    }

    fn get_dense_index(&self, entity: Entity) -> Option<usize> {
        let i = *self.sparse.get(entity.id as usize)? as usize;
        // validate version
        match *self.dense.get(i)? == entity {
            false => None,
            true => Some(i),
        }
    }
}
