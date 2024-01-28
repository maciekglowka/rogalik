use std::{
    any::TypeId,
    cell::{Ref, RefCell, RefMut,},
    collections::{HashMap, HashSet}
};

use rogalik_events::EventBus;

use super::{Storage, WorldEvent};
use super::component::Component;
use super::component_storage::{ComponentSet, ComponentCell, ComponentStorage};
use super::entity::{Entity, EntityStorage};
use super::errors::EntityError;
use super::resource::ResourceCell;

pub struct World {
    component_storage: HashMap<TypeId, Box<dyn ComponentStorage>>,
    entity_storage: EntityStorage,
    resource_storage: HashMap<TypeId, Box<dyn Storage>>,
    // pub events: EventBus<WorldEvent>
}
impl World {
    pub fn new() -> Self {
        let mut world = World { 
            component_storage: HashMap::new(),
            resource_storage: HashMap::new(),
            entity_storage: EntityStorage::new(),
        };
        let events = EventBus::<WorldEvent>::new();
        world.insert_resource(events);
        world
    }

    // events

    fn publish(&self, event: WorldEvent) {
        if let Some(mut bus) = self.get_resource_mut::<EventBus<WorldEvent>>() {
            bus.publish(event);
        }
    }

    // entities

    pub fn spawn_entity(&mut self) -> Entity {
        self.entity_storage.spawn()
    }
    pub fn despawn_entity(&mut self, entity: Entity) {
        self.entity_storage.despawn(entity);
        for (type_id, storage) in self.component_storage.iter() {
            if storage.remove_untyped(entity).is_some() {
                self.publish(WorldEvent::ComponentRemoved(entity, *type_id))
            }
        }
    }

    // components
    pub fn get_component_set<T: Component + 'static>(&self) -> Option<Ref<ComponentSet<T>>> {
        let type_id = TypeId::of::<T>();
        let storage = self.component_storage.get(&type_id)?;
        let cell: &ComponentCell<T> = storage.as_any().downcast_ref()?;
        Some(cell.inner.borrow())
    }
    pub fn get_component_set_mut<T: Component + 'static>(&self) -> Option<RefMut<ComponentSet<T>>> {
        let type_id = TypeId::of::<T>();
        let storage = self.component_storage.get(&type_id)?;
        let cell: &ComponentCell<T> = storage.as_any().downcast_ref()?;
        Some(cell.inner.borrow_mut())
    }
    fn insert_component_storage<T: Component + 'static>(&mut self) {
        let type_id = TypeId::of::<T>();
        let set = ComponentSet::<T>::new();
        let storage = ComponentCell { inner: RefCell::new(set) };
        self.component_storage.insert(
            type_id,
            Box::new(storage)
        );
    }
    pub fn insert_component<T: Component + 'static>(
        &mut self,
        entity: Entity,
        component: T
    ) -> Result<(), EntityError> {
        let type_id = TypeId::of::<T>();
        if !self.component_storage.contains_key(&type_id) {
            self.insert_component_storage::<T>()
        }
        let res = self.get_component_set_mut()
            .ok_or(EntityError)?
            .insert(entity, component);
        if res.is_ok() { self.publish(WorldEvent::ComponentSpawned(entity, type_id)) }
        res
    }
    pub fn remove_component<T: Component + 'static>(&mut self, entity: Entity) -> Option<T> {
        let type_id = TypeId::of::<T>();
        let res = self.get_component_set_mut()?
            .remove(entity);
        if res.is_some() { self.publish(WorldEvent::ComponentRemoved(entity, type_id)) }
        res
    }
    pub fn get_component<T: Component + 'static>(&self, entity: Entity) -> Option<Ref<T>> {
        let set = self.get_component_set::<T>()?;
        Ref::filter_map(set, |s| s.get(entity)).ok()
    }
    pub fn get_component_mut<T: Component + 'static>(&self, entity: Entity) -> Option<RefMut<T>> {
        let set = self.get_component_set_mut::<T>()?;
        RefMut::filter_map(set, |s| s.get_mut(entity)).ok()
    }
    pub fn get_entity_components(&self, entity: Entity) -> Vec<Box<Ref<'_, dyn Component>>> {
        self.component_storage.values()
            .filter_map(|a| a.get_as_component(entity))
            .collect::<Vec<_>>()
    }

    // resources

    pub fn get_resource<T: 'static>(&self) -> Option<Ref<T>> {
        let type_id = TypeId::of::<T>();
        let storage = self.resource_storage.get(&type_id)?;
        let cell: &ResourceCell<T> = storage.as_any().downcast_ref()?;
        Some(cell.inner.borrow())
    }
    pub fn get_resource_mut<T: 'static>(&self) -> Option<RefMut<T>> {
        let type_id = TypeId::of::<T>();
        let storage = self.resource_storage.get(&type_id)?;
        let cell: &ResourceCell<T> = storage.as_any().downcast_ref()?;
        Some(cell.inner.borrow_mut())
    }
    pub fn insert_resource<T: 'static>(&mut self, resource: T) {
        let type_id = TypeId::of::<T>();
        let storage = ResourceCell { inner: RefCell::new(resource) };
        self.resource_storage.insert(
            type_id,
            Box::new(storage)
        );
    }

    // query

    pub fn query<T: 'static + Component>(&self) -> QueryBuilder {
        QueryBuilder::new::<T>(self)
    }
}

pub struct QueryBuilder<'a> {
    world: &'a World,
    inner: HashSet<Entity>
}
impl<'a> QueryBuilder<'a> {
    pub fn new<T: 'static + Component>(world: &World) -> QueryBuilder {
        let entities = match world.get_component_set::<T>() {
            Some(c) => c.hashset(),
            _ => HashSet::new()
        };
        QueryBuilder { inner: entities, world }
    }
    pub fn with<T: 'static + Component>(self) -> QueryBuilder<'a> {
        let h = match self.world.get_component_set::<T>() {
            Some(c) => c.hashset(),
            _ => HashSet::new()
        };
        let entities = self.inner.intersection(&h);
        QueryBuilder {
            inner: entities.map(|e| *e).collect(),
            world: self.world
        }
    }
    pub fn without<T: 'static + Component>(self) -> QueryBuilder<'a> {
        let h = match self.world.get_component_set::<T>() {
            Some(c) => c.hashset(),
            _ => HashSet::new()
        };
        let entities = self.inner.difference(&h);
        QueryBuilder {
            inner: entities.map(|e| *e).collect(),
            world: self.world
        }
    }
    pub fn build(self) -> EntityQuery<'a> {
        EntityQuery { world: self.world, entities: Vec::from_iter(self.inner) }
    }
}

pub struct EntityQuery<'a> {
    world: &'a World,
    entities: Vec<Entity>
}
impl<'a> EntityQuery<'a> {
    pub fn iter<T: Component + 'static>(&self) -> ComponentIterator<'_, T> {
        ComponentIterator::new(self.world, &self.entities)
    }
    pub fn iter_mut<T: Component + 'static>(&self) -> ComponentIteratorMut<'_, T> {
        ComponentIteratorMut::new(self.world, &self.entities)
    }
    pub fn entities(&self) -> std::slice::Iter<Entity> {
        self.entities.iter()
    }
    pub fn single_entity(&self) -> Option<Entity> {
        self.entities.get(0).map(|a| *a)
    }
    pub fn single<T: Component + 'static>(&self) -> Option<Ref<T>> {
        let entity = self.entities.get(0)?;
        self.world.get_component::<T>(*entity)
    }
    pub fn single_mut<T: Component + 'static>(&self) -> Option<RefMut<T>> {
        let entity = self.entities.get(0)?;
        self.world.get_component_mut::<T>(*entity)
    }
}

pub struct ComponentIterator<'a, T: Component + 'static> {
    inner: std::slice::Iter<'a, Entity>,
    cell: Option<&'a ComponentCell<T>>
}
impl<'a, T: Component + 'static> ComponentIterator<'a, T> {
    pub fn new(world: &'a World, entities: &'a Vec<Entity>) -> Self {
        let type_id = TypeId::of::<T>();
        let cell = if let Some(storage) = world.component_storage.get(&type_id) {
            let cell: &ComponentCell<T> = storage.as_any().downcast_ref().unwrap();
            Some(cell)
        } else {
            None
        };
        ComponentIterator { 
            inner: entities.iter(),
            cell
        }
    }
}
impl<'a, T: Component + 'static> Iterator for ComponentIterator<'a, T> {
    type Item = Ref<'a, T>;
    fn next(&mut self) -> Option<Self::Item> {
        let entity = self.inner.next()?;
        let set = self.cell?.inner.borrow();
        Some(Ref::filter_map(set, |s| s.get(*entity)).ok()?)
    }
}

pub struct ComponentIteratorMut<'a, T: Component + 'static> {
    inner: std::slice::Iter<'a, Entity>,
    cell: Option<&'a ComponentCell<T>>
}
impl<'a, T: Component + 'static> ComponentIteratorMut<'a, T> {
    pub fn new(world: &'a World, entities: &'a Vec<Entity>) -> Self {
        let type_id = TypeId::of::<T>();
        let cell = if let Some(storage) = world.component_storage.get(&type_id) {
            let cell: &ComponentCell<T> = storage.as_any().downcast_ref().unwrap();
            Some(cell)
        } else {
            None
        };
        ComponentIteratorMut { 
            inner: entities.iter(),
            cell
        }
    }
}
impl<'a, T: Component + 'static> Iterator for ComponentIteratorMut<'a, T> {
    type Item = RefMut<'a, T>;
    fn next(&mut self) -> Option<Self::Item> {
        let entity = self.inner.next()?;
        let set = self.cell?.inner.borrow_mut();
        Some(RefMut::filter_map(set, |s| s.get_mut(*entity)).ok()?)
    }
}