use std::{
    any::{Any, TypeId},
    vec::IntoIter,
};

use crate::{
    components::{ComponentSet, Components},
    entity::Entity,
};

pub struct Query<'a, T>
where
    T: IntoQuery,
{
    entities: Vec<Entity>,
    state: T::State<'a>,
}
impl<'a, T: IntoQuery> Query<'a, T> {
    pub(crate) fn new(components: &'a Components) -> Self {
        Self {
            entities: T::entities(components),
            state: T::state(components),
        }
    }
    pub fn iter(&'a self) -> QueryIter<'a, T> {
        QueryIter {
            offset: 0,
            entities: &self.entities,
            state: &self.state,
        }
    }
}

pub struct QueryIter<'a, T>
where
    T: IntoQuery,
{
    offset: usize,
    entities: &'a Vec<Entity>,
    state: &'a T::State<'a>,
}
impl<'a, T: IntoQuery + 'static> Iterator for QueryIter<'a, T> {
    type Item = <T::State<'a> as Fetch>::Item<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.offset += 1;
        Some(self.state.get(*self.entities.get(self.offset - 1)?))
    }
}

pub struct QueryMut<'a, T>
where
    T: IntoQuery,
{
    entities: std::slice::Iter<'a, Entity>,
    state: T::StateMut<'a>,
}
impl<'a, T: IntoQuery> QueryMut<'a, T> {
    // pub(crate) fn new(components: &'a mut Components) -> Self {
    //     Self {
    //         entities: T::entities(components),
    //         state: T::state_mut(components),
    //     }
    // }
    pub fn next(&'a mut self) -> <T::StateMut<'a> as FetchMut>::Item<'a> {
        self.state.get_mut(*self.entities.next().unwrap())
    }
}

// pub struct QueryIterMut<'a, T: 'static>
// where
//     T: IntoQuery,
// {
//     entities: std::slice::Iter<'a, Entity>,
//     state: &'a mut T::StateMut<'a>,
// }
// impl<'a, T: IntoQuery + 'static> Iterator for QueryIterMut<'a, T> {
//     type Item = <T::StateMut<'a> as FetchMut>::Item<'a>;

//     fn next(&mut self) -> Option<Self::Item> {
//         Some(self.state.get_mut(*self.entities.next()?))
//     }
// }

pub trait IntoQuery {
    type State<'a>: Fetch;
    type StateMut<'a>: FetchMut;

    fn entities(components: &Components) -> Vec<Entity>;
    fn state<'a>(components: &'a Components) -> Self::State<'a>;
    fn state_mut<'a>(components: &'a mut Components) -> Self::StateMut<'a>;
}

pub trait Fetch {
    type Item<'a>
    where
        Self: 'a;

    fn get<'a>(&'a self, entity: Entity) -> Self::Item<'a>;
}

pub trait FetchMut {
    type Item<'a>
    where
        Self: 'a;

    fn get_mut<'a>(&'a mut self, entity: Entity) -> Self::Item<'a>;
}

impl<A: 'static, B: 'static> IntoQuery for (A, B) {
    type State<'a> = (&'a ComponentSet<A>, &'a ComponentSet<B>);
    type StateMut<'a> = (&'a mut ComponentSet<A>, &'a mut ComponentSet<B>);

    fn entities(components: &Components) -> Vec<Entity> {
        components
            .get_set::<A>()
            .entities()
            .intersection(&components.get_set::<B>().entities())
            .copied()
            .collect()
    }

    fn state<'a>(components: &'a Components) -> Self::State<'a> {
        (components.get_set::<A>(), components.get_set::<B>())
    }
    fn state_mut<'a>(components: &'a mut Components) -> Self::StateMut<'a> {
        let [a, b] = components
            .storage
            .get_disjoint_mut([&TypeId::of::<A>(), &TypeId::of::<B>()]);
        (
            (&mut **a.unwrap() as &mut dyn Any).downcast_mut().unwrap(),
            (&mut **b.unwrap() as &mut dyn Any).downcast_mut().unwrap(),
        )
    }
}
impl<A: 'static, B: 'static> Fetch for (&ComponentSet<A>, &ComponentSet<B>) {
    type Item<'a>
        = (&'a A, &'a B)
    where
        Self: 'a;

    fn get<'a>(&'a self, entity: Entity) -> Self::Item<'a> {
        (self.0.get(entity).unwrap(), self.1.get(entity).unwrap())
    }
}
impl<A: 'static, B: 'static> FetchMut for (&mut ComponentSet<A>, &mut ComponentSet<B>) {
    type Item<'a>
        = (&'a mut A, &'a mut B)
    where
        Self: 'a;

    fn get_mut<'a>(&'a mut self, entity: Entity) -> Self::Item<'a> {
        (
            self.0.get_mut(entity).unwrap(),
            self.1.get_mut(entity).unwrap(),
        )
    }
}
