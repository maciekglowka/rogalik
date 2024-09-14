#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};

pub type IdSize = u16;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub struct Entity {
    pub id: IdSize,
    pub version: IdSize,
}

#[derive(Default)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub struct EntityStorage {
    entities: Vec<Entity>,
    next_recycled: Option<IdSize>,
    last_recycled: Option<IdSize>,
}
impl EntityStorage {
    pub fn new() -> Self {
        EntityStorage {
            entities: Vec::new(),
            next_recycled: None,
            last_recycled: None,
        }
    }
    pub fn spawn(&mut self) -> Entity {
        if let Some(entitiy) = self.recycle() {
            entitiy
        } else {
            self.spawn_new()
        }
    }
    fn recycle(&mut self) -> Option<Entity> {
        let next = self.next_recycled?;
        let mut entity = self.entities[next as usize];
        if self.last_recycled == Some(next) {
            // no more recycled entities at the moment
            self.last_recycled = None;
            self.next_recycled = None;
        } else {
            // get new next temporary stored in the id
            self.next_recycled = Some(entity.id);
        }
        // restore entity id
        entity.id = next;
        Some(entity)
    }
    fn spawn_new(&mut self) -> Entity {
        let id = self.entities.len();
        let entity = Entity {
            id: id as IdSize,
            version: 0,
        };
        self.entities.push(entity);
        entity
    }
    pub fn despawn(&mut self, entity: Entity) {
        // TODO should we check version if not already despawned?
        self.entities[entity.id as usize].version += 1;
        if let Some(last) = self.last_recycled {
            // temp store index for next
            self.entities[last as usize].id = entity.id;
        } else {
            // this is the first one on the recycled list
            self.next_recycled = Some(entity.id);
        }
        self.last_recycled = Some(entity.id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recycle_single() {
        let mut store = EntityStorage::new();
        let entity = store.spawn();
        let id = entity.id;
        let v = entity.version;
        store.despawn(entity);
        let new = store.spawn();
        assert!(id == new.id);
        assert!(v + 1 == new.version);
    }
    #[test]
    fn recycle_multiple() {
        let mut store = EntityStorage::new();
        let count = 50;
        for _ in 0..count {
            store.spawn();
        }

        store.despawn(Entity { id: 23, version: 0 });
        store.despawn(Entity { id: 33, version: 0 });
        store.despawn(Entity { id: 35, version: 0 });

        let new_23 = store.spawn();
        assert!(new_23.id == 23);
        assert!(new_23.version == 1);

        let new_33 = store.spawn();
        assert!(new_33.id == 33);
        assert!(new_33.version == 1);

        let new_35 = store.spawn();
        assert!(new_35.id == 35);
        assert!(new_35.version == 1);
    }
    #[test]
    fn recycle_and_new() {
        let mut store = EntityStorage::new();
        let count = 5;
        for _ in 0..count {
            store.spawn();
        }

        store.despawn(Entity { id: 3, version: 0 });

        let recycled = store.spawn();
        assert!(recycled.id == 3);
        assert!(recycled.version == 1);

        let new = store.spawn();
        assert!(new.id == count);
        assert!(new.version == 0);
    }
    #[test]
    fn recycle_more() {
        let count = 5;
        let mut store = EntityStorage::new();
        for _ in 0..count {
            store.spawn();
        }

        for i in 0..3 {
            store.despawn(Entity { id: 2, version: i });
            let new = store.spawn();
            assert!(2 == new.id);
            assert!(i + 1 == new.version);
        }
    }
}
