#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};

pub type IdSize = u16;

/// Unique world object identifier.
#[derive(Clone, Copy, Debug, Default, Hash, Eq, PartialEq, PartialOrd, Ord)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub struct Entity {
    pub id: IdSize,
    pub version: IdSize,
}

/// EntityStorage responsible for spawning and despawning of the entities.
/// Entity id's are recycled internally and versioned to avoid dead entity
/// usage.
/// ```ignore
/// use wunderkammer::prelude::*;
/// let mut storage = EntityStorage::default();
/// let a = storage.spawn();
/// let b = storage.spawn();
///
/// storage.despawn(a);
/// let c = storage.spawn();
/// assert_eq!(c.id, a.id);
/// assert_eq!(c.version, a.version + 1);
/// ```
#[derive(Default)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub struct Entities {
    entities: Vec<Entity>,
    last_recycled: Option<IdSize>,
    first_recycled: Option<IdSize>,
}
impl Entities {
    pub(crate) fn new() -> Self {
        Self::default()
    }
    /// Spawn an Entity
    pub(crate) fn spawn(&mut self) -> Entity {
        if let Some(entity) = self.recycle() {
            return entity;
        }
        self.spawn_new()
    }
    /// Despawn Entity from the storage
    pub(crate) fn despawn(&mut self, entity: Entity) {
        if self.entities[entity.id as usize].version != entity.version {
            // already despawned!
            return;
        }
        self.entities[entity.id as usize].version += 1;
        if let Some(last) = self.last_recycled {
            // push on the existing recycle list
            self.entities[last as usize].id = entity.id;
        } else {
            // this is the first entity on the recycle list
            self.first_recycled = Some(entity.id);
        }
        // now this one is the prev_recycled
        self.last_recycled = Some(entity.id);
    }
    /// Validates the given entity
    pub(crate) fn is_valid(&self, entity: Entity) -> bool {
        let Some(existing) = self.entities.get(entity.id as usize) else {
            return false;
        };
        existing.version == entity.version
            && existing.id == entity.id
            && self.last_recycled != Some(entity.id)
            && self.first_recycled != Some(entity.id)
    }
    /// Iterate through valid entities
    pub(crate) fn all(&self) -> impl Iterator<Item = Entity> + use<'_> {
        self.entities.iter().filter(|e| self.is_valid(**e)).copied()
    }

    /// Spawns a fresh entity, with version 0
    fn spawn_new(&mut self) -> Entity {
        let id = self.entities.len();
        let entity = Entity {
            id: id as IdSize,
            version: 0,
        };
        self.entities.push(entity);
        entity
    }
    /// Recycles the previously despawned entity
    fn recycle(&mut self) -> Option<Entity> {
        let recycled_id = self.first_recycled?;
        let recycled = &mut self.entities[recycled_id as usize];

        if self.last_recycled == Some(recycled_id) {
            // no more recycled entities
            self.last_recycled = None;
            self.first_recycled = None;
        } else {
            // the next recycled index was temporarily stored in the id
            self.first_recycled = Some(recycled.id);
        }
        // restore the id to the valid index
        recycled.id = recycled_id;
        Some(*recycled)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawn_new() {
        let mut storage = Entities::default();
        for i in 0..5 {
            let e = storage.spawn_new();
            assert_eq!(i, e.id);
            assert_eq!(0, e.version);
        }

        assert_eq!(storage.entities.len(), 5);
    }

    #[test]
    fn despawn() {
        let mut storage = Entities::default();
        let entities = (0..5).map(|_| storage.spawn_new()).collect::<Vec<_>>();
        storage.despawn(entities[2]);
        assert!(!storage.entities.contains(&entities[2]));
    }

    #[test]
    fn recycle_single() {
        let mut storage = Entities::default();
        let a = storage.spawn();
        let _ = storage.spawn();
        storage.despawn(a);
        let c = storage.spawn();
        assert_eq!(a.id, c.id);
        assert_eq!(a.version + 1, c.version);

        storage.despawn(c);
        let d = storage.spawn();
        assert_eq!(a.id, d.id);
        assert_eq!(a.version + 2, d.version);
    }

    #[test]
    fn recycle_many() {
        let mut storage = Entities::default();
        let entities = (0..10).map(|_| storage.spawn_new()).collect::<Vec<_>>();
        storage.despawn(entities[2]);
        storage.despawn(entities[3]);
        storage.despawn(entities[7]);

        let a = storage.spawn();
        assert_eq!(a.id, entities[2].id);
        assert_eq!(a.version, entities[2].version + 1);

        let b = storage.spawn();
        assert_eq!(b.id, entities[3].id);
        assert_eq!(b.version, entities[3].version + 1);

        let c = storage.spawn();
        assert_eq!(c.id, entities[7].id);
        assert_eq!(c.version, entities[7].version + 1);

        // no more entities to recycle
        assert_eq!(storage.spawn().id, 10);
    }

    #[test]
    fn spawn() {
        let mut storage = Entities::default();
        let a = storage.spawn();
        let _ = storage.spawn();

        storage.despawn(a);
        let c = storage.spawn();
        assert_eq!(c.id, a.id);

        let d = storage.spawn();
        assert_eq!(d.id, 2);
    }

    #[test]
    fn is_valid_not_spawned() {
        let mut storage = Entities::default();
        for _ in 0..10 {
            storage.spawn();
        }

        assert!(!storage.is_valid(Entity { id: 11, version: 0 }));
    }

    #[test]
    fn is_valid_despawned() {
        let mut storage = Entities::default();
        for _ in 0..10 {
            storage.spawn();
        }

        let entity = Entity { id: 5, version: 0 };
        assert!(storage.is_valid(entity));
        storage.despawn(entity);
        assert!(!storage.is_valid(entity));
    }

    #[test]
    fn is_valid_recycled() {
        let mut storage = Entities::default();
        for _ in 0..10 {
            storage.spawn();
        }

        let entity = Entity { id: 5, version: 0 };
        assert!(storage.is_valid(entity));
        storage.despawn(entity);
        let recycled = storage.spawn();
        assert_eq!(Entity { id: 5, version: 1 }, recycled);
        assert!(!storage.is_valid(entity));
    }

    #[test]
    fn all() {
        let mut storage = Entities::default();
        for _ in 0..10 {
            storage.spawn();
        }

        storage.despawn(Entity { id: 1, version: 0 });
        storage.despawn(Entity { id: 5, version: 0 });
        assert_eq!(8, storage.all().collect::<Vec<_>>().len());

        // recycle
        storage.spawn();
        assert_eq!(9, storage.all().collect::<Vec<_>>().len());
        storage.despawn(Entity { id: 1, version: 1 });
        assert_eq!(8, storage.all().collect::<Vec<_>>().len());
    }
}
