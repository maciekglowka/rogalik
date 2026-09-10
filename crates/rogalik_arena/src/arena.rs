pub struct Id<T, S = u16> {
    id: S,
    generation: S,
    _marker: std::marker::PhantomData<fn() -> T>,
}
impl<T, S> Id<T, S> {
    fn new(id: S, generation: S) -> Self {
        Self {
            id,
            generation,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T, S: Copy> Clone for Id<T, S> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T, S: Copy> Copy for Id<T, S> {}

impl<T, S: ArenaIndex> PartialEq for Id<T, S> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.generation == other.generation
    }
}
impl<T, S: ArenaIndex> Eq for Id<T, S> {}
impl<T, S: std::hash::Hash> std::hash::Hash for Id<T, S> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.generation.hash(state);
    }
}
impl<T, S: ArenaIndex + Ord> Ord for Id<T, S> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id
            .cmp(&other.id)
            .then(self.generation.cmp(&other.generation))
    }
}
impl<T, S: ArenaIndex + Ord> PartialOrd for Id<T, S> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> std::fmt::Debug for Id<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Id({}, {})", self.id, self.generation)
    }
}

pub struct Arena<T, S = u16> {
    sparse: Vec<SparseEntry<S>>,
    dense: Zipped<T, S>,
    recycled_head: Option<S>,
}

impl<T, S: ArenaIndex> Default for Arena<T, S> {
    fn default() -> Self {
        Self::new()
    }
}
impl<T, S> Arena<T, S>
where
    S: ArenaIndex,
{
    pub fn new() -> Arena<T, S> {
        Arena {
            sparse: vec![],
            dense: Zipped::new(),
            recycled_head: None,
        }
    }
    /// Insert a new item into the arena.
    ///
    /// If a recycled id is available arena size won't grow.
    pub fn insert(&mut self, item: T) -> Id<T, S> {
        // Try recycled
        match self.pop_recycled() {
            Some(id) => {
                let sparse = &mut self.sparse[id.to_index()];
                sparse.dense_index = S::from_index(self.dense.len());
                self.dense.push(item, id);
                Id::new(id, sparse.generation)
            }
            None => {
                // Create new, when no recycled id is available.
                assert!(self.sparse.len() < <S as ArenaIndex>::max().to_index() - 1);

                let id = S::from_index(self.sparse.len());
                let dense_index = S::from_index(self.dense.len());
                let generation = S::zero();

                self.sparse.push(SparseEntry {
                    generation,
                    dense_index,
                });
                self.dense.push(item, id);
                Id::new(id, generation)
            }
        }
    }

    pub fn remove(&mut self, id: Id<T, S>) -> Option<T> {
        if self.dense.len() == 0 {
            return None;
        }
        let sparse = self.sparse.get_mut(id.id.to_index())?;
        if sparse.generation != id.generation {
            return None;
        }
        let dense_index = sparse.dense_index;

        let swapped_sparse_index = self.dense.last().map(|(_, i)| *i);

        let (item, _) = self.dense.swap_remove(dense_index.to_index());

        // Bump generation and add to recycle queue.
        sparse.generation = sparse.generation.increment();

        // Fix swapped element's dense index.
        if let Some(swapped_sparse_index) = swapped_sparse_index {
            self.sparse[swapped_sparse_index.to_index()].dense_index = dense_index;
        }

        // This has to be done as a last step, as otherwise fixing dense index
        // of a swap element might overwrite recycled linked list (if
        // removed element is last in the dense array).
        self.push_recycled(id.id);

        Some(item)
    }

    pub fn get(&self, id: &Id<T, S>) -> Option<&T> {
        let sparse = self.sparse.get(id.id.to_index())?;
        if sparse.generation != id.generation {
            return None;
        }
        self.dense.get_left(sparse.dense_index.to_index())
    }

    pub fn get_mut(&mut self, id: &Id<T, S>) -> Option<&mut T> {
        let sparse = self.sparse.get(id.id.to_index())?;
        if sparse.generation != id.generation {
            return None;
        }
        self.dense.get_left_mut(sparse.dense_index.to_index())
    }

    pub fn shrink_to_fit(&mut self) {
        self.dense.shrink_to_fit();
    }

    fn push_recycled(&mut self, id: S) {
        if let Some(head) = self.recycled_head {
            // Temporarily using dense index (since it's not valid for recycled
            // entries anyway) as a pointer in a
            // recycled-linked-list.
            self.sparse[id.to_index()].dense_index = head;
            self.recycled_head = Some(id)
        } else {
            // There is no `next-recycled` - use max as a tombstone.
            self.sparse[id.to_index()].dense_index = <S as ArenaIndex>::max();
            self.recycled_head = Some(id)
        }
    }

    fn pop_recycled(&mut self) -> Option<S> {
        if let Some(head) = self.recycled_head {
            let next = self.sparse[head.to_index()].dense_index;
            // S::MAX is a tombstone - meaning this was the last recycled id.
            if next < <S as ArenaIndex>::max() {
                self.recycled_head = Some(next);
            } else {
                self.recycled_head = None;
            }
            Some(head)
        } else {
            None
        }
    }

    /// Immutably iterates over stored items.
    ///
    /// Iteration order is arbitrary.
    pub fn values(&self) -> std::slice::Iter<'_, T> {
        self.dense.0.iter()
    }
    /// Mutably iterates over stored items.
    ///
    /// Iteration order is arbitrary.
    pub fn values_mut(&mut self) -> std::slice::IterMut<'_, T> {
        self.dense.0.iter_mut()
    }

    /// Immutably iterates over id and stored items.
    ///
    /// Unless id values are needed `Self::values` method should be used as more
    /// performant. Iteration order is arbitrary.
    pub fn iter(&self) -> Iter<'_, S, T> {
        Iter {
            sparse: &self.sparse,
            items: self.dense.0.iter(),
            ids: self.dense.1.iter(),
        }
    }

    /// Mutably iterates over id and stored items.
    ///
    /// Unless id values are needed `Self::values_mut` method should be used as
    /// more performant. Iteration order is arbitrary.
    pub fn iter_mut(&mut self) -> IterMut<'_, S, T> {
        IterMut {
            sparse: &self.sparse,
            items: self.dense.0.iter_mut(),
            ids: self.dense.1.iter_mut(),
        }
    }
}

impl<T, S> IntoIterator for Arena<T, S> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.dense.0.into_iter()
    }
}

pub struct Iter<'a, S, T> {
    sparse: &'a Vec<SparseEntry<S>>,
    items: std::slice::Iter<'a, T>,
    ids: std::slice::Iter<'a, S>,
}
impl<'a, S: ArenaIndex, T> Iterator for Iter<'a, S, T> {
    type Item = (Id<T, S>, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        let v = self.items.next()?;
        let id = self.ids.next()?;
        let generation = self.sparse.get(id.to_index())?.generation;

        Some((Id::new(*id, generation), v))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.items.size_hint()
    }
}
pub struct IterMut<'a, S, T> {
    sparse: &'a Vec<SparseEntry<S>>,
    items: std::slice::IterMut<'a, T>,
    ids: std::slice::IterMut<'a, S>,
}
impl<'a, S: ArenaIndex, T> Iterator for IterMut<'a, S, T> {
    type Item = (Id<T, S>, &'a mut T);

    fn next(&mut self) -> Option<Self::Item> {
        let v = self.items.next()?;
        let id = self.ids.next()?;
        let generation = self.sparse.get(id.to_index())?.generation;

        Some((Id::new(*id, generation), v))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.items.size_hint()
    }
}

#[derive(Clone, Copy)]
struct SparseEntry<S> {
    /// In case of an occupied entry - current generation.
    /// In case of a free entry - next generation.
    generation: S,
    /// In case of an occupied entry - dense array index.
    /// In case of a free entry - next free (recycled) sparse index.
    dense_index: S,
}

/// Helper struct making sure both dense vecs are always in sync.
struct Zipped<T, U>(Vec<T>, Vec<U>);
impl<T, U> Zipped<T, U> {
    fn new() -> Self {
        Self(vec![], vec![])
    }
    fn len(&self) -> usize {
        self.0.len()
    }
    fn push(&mut self, t: T, u: U) {
        self.0.push(t);
        self.1.push(u);
    }
    fn swap_remove(&mut self, i: usize) -> (T, U) {
        let t = self.0.swap_remove(i);
        let u = self.1.swap_remove(i);
        (t, u)
    }
    fn shrink_to_fit(&mut self) {
        self.0.shrink_to_fit();
        self.1.shrink_to_fit();
    }
    fn get_left(&self, i: usize) -> Option<&T> {
        self.0.get(i)
    }
    fn get_left_mut(&mut self, i: usize) -> Option<&mut T> {
        self.0.get_mut(i)
    }
    #[allow(dead_code)]
    fn get_right(&self, i: usize) -> Option<&U> {
        self.1.get(i)
    }
    #[allow(dead_code)]
    fn get_right_mut(&mut self, i: usize) -> Option<&mut U> {
        self.1.get_mut(i)
    }
    fn last(&self) -> Option<(&T, &U)> {
        Some((self.0.last()?, self.1.last()?))
    }
}

pub trait ArenaIndex: Copy + Eq + Ord + std::fmt::Debug {
    fn zero() -> Self;
    fn max() -> Self;
    fn to_index(self) -> usize;
    fn from_index(i: usize) -> Self;
    fn increment(self) -> Self;
}

macro_rules! impl_arena_index {
    ($t: ty) => {
        impl ArenaIndex for $t {
            fn zero() -> Self {
                0
            }
            fn max() -> Self {
                <$t>::MAX
            }
            fn to_index(self) -> usize {
                self as usize
            }
            fn from_index(i: usize) -> Self {
                i as Self
            }
            fn increment(self) -> Self {
                // It is very unlikely that this will wrap
                // and even less likely that there will be a collision with a kept id.
                self.wrapping_add(1)
            }
        }
    };
}

impl_arena_index!(u8);
impl_arena_index!(u16);
impl_arena_index!(u32);
impl_arena_index!(u64);
impl_arena_index!(usize);

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn insert_get() {
        let mut arena = Arena::<_, u16>::new();

        let _a = arena.insert("a");
        let b = arena.insert("b");
        let c = arena.insert("c");
        let _d = arena.insert("d");

        assert_eq!(arena.get(&c), Some(&"c"));
        assert_eq!(arena.get_mut(&b), Some(&mut "b"));
    }
    #[test]
    #[should_panic]
    fn insert_overflow() {
        let mut arena = Arena::<_, u8>::new();

        for _ in 0..=254 {
            let _ = arena.insert(0);
        }
    }
    #[test]
    fn remove() {
        let mut arena = Arena::<_, u16>::new();

        let a = arena.insert("a");
        let b = arena.insert("b");
        let c = arena.insert("c");
        let d = arena.insert("d");

        assert_eq!(arena.remove(b), Some("b"));
        assert_eq!(arena.get(&b), None);

        // Test other keys still match right values.
        assert_eq!(arena.get(&a), Some(&"a"));
        assert_eq!(arena.get(&c), Some(&"c"));
        assert_eq!(arena.get(&d), Some(&"d"));

        assert_eq!(arena.remove(a), Some("a"));
        assert_eq!(arena.get(&a), None);

        // Test other keys still match right values.
        assert_eq!(arena.get(&c), Some(&"c"));
        assert_eq!(arena.get(&d), Some(&"d"));
    }
    #[test]
    fn remove_last() {
        let mut arena = Arena::<_, u16>::new();

        let a = arena.insert("a");
        let b = arena.insert("b");

        assert_eq!(arena.remove(b), Some("b"));
        assert_eq!(arena.get(&b), None);
        assert_eq!(arena.remove(a), Some("a"));
        assert_eq!(arena.get(&a), None);
    }
    #[test]
    fn remove_invalid() {
        let mut arena = Arena::<_, u16>::new();

        let _a = arena.insert("a");
        let b = arena.insert("b");

        assert_eq!(arena.remove(b), Some("b"));
        assert_eq!(arena.get(&b), None);
        assert_eq!(arena.remove(b), None);
    }
    #[test]
    fn insert_remove_insert() {
        let mut arena = Arena::<_, u16>::new();

        let a = arena.insert("a");
        let b = arena.insert("b");
        let c = arena.insert("c");

        assert_eq!(arena.remove(b), Some("b"));

        let d = arena.insert("d");
        assert_eq!(arena.remove(a), Some("a"));

        // Test other keys still match right values.
        assert_eq!(arena.get(&c), Some(&"c"));
        assert_eq!(arena.get(&d), Some(&"d"));
    }
    #[test]
    fn recycle_single_slot() {
        let mut arena = Arena::<_, u16>::new();

        let a = arena.insert("a");
        let b = arena.insert("b");
        let c = arena.insert("c");

        assert_eq!(arena.remove(b), Some("b"));
        assert_eq!(arena.get(&b), None);

        let d = arena.insert("d");

        assert_eq!(b.id, d.id);
        assert_eq!(b.generation + 1, d.generation);

        assert_eq!(arena.remove(d), Some("d"));
        assert_eq!(arena.get(&d), None);

        let e = arena.insert("e");
        assert_eq!(d.id, e.id);
        assert_eq!(d.generation + 1, e.generation);

        // Test keys still match right values.
        assert_eq!(arena.get(&a), Some(&"a"));
        assert_eq!(arena.get(&b), None);
        assert_eq!(arena.get(&c), Some(&"c"));
        assert_eq!(arena.get(&d), None);
        assert_eq!(arena.get(&e), Some(&"e"));

        arena.shrink_to_fit();
        assert_eq!(arena.dense.0.capacity(), 3);
    }
    #[test]
    fn recycle_many() {
        let mut arena = Arena::<_, u16>::new();

        let a = arena.insert("a");
        let b = arena.insert("b");
        let c = arena.insert("c");
        let d = arena.insert("d");
        let e = arena.insert("e");

        assert_eq!(arena.remove(b), Some("b"));
        assert_eq!(arena.remove(d), Some("d"));
        assert_eq!(arena.remove(a), Some("a"));

        let f = arena.insert("f");
        assert!(f.id < 5);
        let g = arena.insert("g");
        assert!(g.id < 5);
        let h = arena.insert("h");
        assert!(h.id < 5);

        arena.shrink_to_fit();
        assert_eq!(arena.dense.0.capacity(), 5);

        let i = arena.insert("i");
        assert_eq!(i.id, 5);

        arena.shrink_to_fit();
        assert_eq!(arena.dense.0.capacity(), 6);

        // Test keys still match right values.
        assert_eq!(arena.get(&a), None);
        assert_eq!(arena.get(&b), None);
        assert_eq!(arena.get(&c), Some(&"c"));
        assert_eq!(arena.get(&d), None);
        assert_eq!(arena.get(&e), Some(&"e"));
        assert_eq!(arena.get(&f), Some(&"f"));
        assert_eq!(arena.get(&g), Some(&"g"));
        assert_eq!(arena.get(&h), Some(&"h"));
        assert_eq!(arena.get(&i), Some(&"i"));
    }
    #[test]
    fn iter_values() {
        let mut arena = Arena::<_, u16>::new();

        let _ = arena.insert(1);
        let id = arena.insert(2);
        let _ = arena.remove(id);
        let _ = arena.insert(3);
        let _ = arena.insert(4);

        assert_eq!(8, arena.values().sum());
    }
    #[test]
    fn iter_values_mut() {
        let mut arena = Arena::<_, u16>::new();

        let _ = arena.insert(1);
        let _ = arena.insert(2);
        let id = arena.insert(3);
        let _ = arena.insert(4);

        for item in arena.values_mut() {
            *item += 7;
        }
        assert_eq!(arena.get(&id), Some(&10));
    }
    #[test]
    fn iter_key_value() {
        let mut arena = Arena::<_, u16>::new();

        let a = arena.insert("a");
        let b = arena.insert("b");
        let c = arena.insert("c");
        let d = arena.insert("d");
        let _ = arena.remove(c);
        let e = arena.insert("e");

        let mut expected: HashMap<Id<&str, u16>, &str, std::hash::RandomState> =
            HashMap::from_iter([(a, "a"), (b, "b"), (d, "d"), (e, "e")]);

        for (k, v) in arena.iter() {
            let ev = expected.remove(&k).unwrap();
            assert_eq!(*v, ev);
        }

        assert!(expected.is_empty());
    }
    #[test]
    fn iter_key_value_mut() {
        let mut arena = Arena::<_, u16>::new();

        let a = arena.insert(1);
        let b = arena.insert(2);
        let c = arena.insert(3);
        let d = arena.insert(4);
        let _ = arena.remove(c);
        let e = arena.insert(5);

        let mut expected: HashMap<Id<u32, u16>, u32, std::hash::RandomState> =
            HashMap::from_iter([(a, 1), (b, 2), (d, 4), (e, 5)]);

        for (k, v) in arena.iter_mut() {
            let ev = expected.remove(&k).unwrap();
            assert_eq!(*v, ev);
            *v += 13;
        }

        assert!(expected.is_empty());
        assert_eq!(arena.values().sum::<u32>(), 4 * 13 + 1 + 2 + 4 + 5);
    }
}
