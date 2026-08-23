use std::collections::VecDeque;

pub struct ResourceId<T, S = u16> {
    id: S,
    generation: S,
    _marker: std::marker::PhantomData<fn() -> T>,
}
impl<T, S> ResourceId<T, S> {
    fn new(id: S, generation: S) -> Self {
        Self {
            id,
            generation,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T, S: Copy> Clone for ResourceId<T, S> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T, S: Copy> Copy for ResourceId<T, S> {}

pub struct Arena<T, S = u16> {
    sparse: Vec<SparseEntry<S>>,
    dense: Vec<DenseEntry<T, S>>,
    recycled: VecDeque<S>,
}
impl<T, S> Arena<T, S>
where
    S: ArenaIndex + Eq,
{
    pub fn new() -> Self {
        Self {
            sparse: vec![],
            dense: vec![],
            recycled: VecDeque::new(),
        }
    }

    /// Insert a new item into the arena.
    ///
    /// If a recycled id is available arena size won't grow.
    pub fn insert(&mut self, item: T) -> ResourceId<T, S> {
        // Try recycled
        match self.recycled.pop_front() {
            Some(id) => {
                // let i = usize::try_from(id).unwrap();
                let sparse = &mut self.sparse[id.to_index()];
                sparse.dense_index = S::from_index(self.dense.len());
                self.dense.push(item);
                ResourceId::new(id, sparse.generation)
            }
            None => {
                // Create new, when no recycled id is available.
                let id = S::from_index(self.sparse.len());
                let dense_index = S::from_index(self.dense.len());
                let generation = S::zero();

                self.sparse.push(SparseEntry {
                    generation,
                    dense_index,
                });
                self.dense.push(item);
                ResourceId::new(id, generation)
            }
        }
    }

    pub fn remove(&mut self, id: ResourceId<T, S>) -> Option<T> {
        if self.dense.is_empty() {
            return None;
        }
        let sparse = self.sparse.get_mut(id.id.to_index())?;
        if sparse.generation != id.generation {
            return None;
        }
        let dense_index = sparse.dense_index;

        let item = self.dense.swap_remove(sparse.dense_index.to_index());

        // Bump generation and add to recycle queue.
        sparse.generation = sparse.generation.increment();
        self.recycled.push_back(id.id);

        // Fix swapped index. [O(n)]
        let swapped_sparse_index = self
            .sparse
            .iter_mut()
            // FIXME len - 1
            .find(|e| e.dense_index == S::from_index(self.dense.len() - 1))
            .unwrap();
        swapped_sparse_index.dense_index = dense_index;

        Some(item)
    }

    pub fn get(&self, id: &ResourceId<T, S>) -> Option<&T> {
        let sparse = self.sparse.get(id.id.to_index())?;
        if sparse.generation != id.generation {
            return None;
        }
        self.dense.get(sparse.dense_index.to_index())
    }

    pub fn get_mut(&mut self, id: &ResourceId<T, S>) -> Option<&mut T> {
        let sparse = self.sparse.get(id.id.to_index())?;
        if sparse.generation != id.generation {
            return None;
        }
        self.dense.get_mut(sparse.dense_index.to_index())
    }
}

#[derive(Clone, Copy)]
struct DenseEntry<T, S> {
    inner: T,
    sparse_index: S,
}

#[derive(Clone, Copy)]
struct SparseEntry<S> {
    generation: S,
    dense_index: S,
}

pub trait ArenaIndex: Copy {
    fn zero() -> Self;
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
            fn to_index(self) -> usize {
                self as usize
            }
            fn from_index(i: usize) -> Self {
                i as Self
            }
            fn increment(self) -> Self {
                self + 1
            }
        }
    };
}

impl_arena_index!(u8);
impl_arena_index!(u16);
impl_arena_index!(u32);
impl_arena_index!(u64);
impl_arena_index!(usize);
