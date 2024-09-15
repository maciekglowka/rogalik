use std::{
    collections::HashMap,
    rc::{Rc, Weak},
    sync::Mutex,
};

// at the moment not thread safe

pub struct SubscriberHandle<T: Copy> {
    queue: Weak<Mutex<Vec<T>>>,
    pub id: usize,
}
impl<'a, T: Copy> SubscriberHandle<T> {
    pub fn read(&self) -> Option<Vec<T>> {
        let strong = self.queue.upgrade()?;
        let mut queue = strong.lock().ok()?;
        Some(queue.drain(..).collect())
    }
    // TODO add drop impl to prevent memory leaks
}

pub struct EventBus<T> {
    subscribers: HashMap<usize, Rc<Mutex<Vec<T>>>>,
    next: usize,
}
impl<T> Default for EventBus<T> {
    fn default() -> Self {
        Self {
            subscribers: HashMap::new(),
            next: 0,
        }
    }
}
impl<'a, T: Copy> EventBus<T> {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn publish(&mut self, e: T) {
        for (_, s) in self.subscribers.iter_mut() {
            if let Ok(mut v) = s.lock() {
                v.push(e);
            }
        }
    }
    pub fn subscribe(&mut self) -> SubscriberHandle<T> {
        let queue = Rc::new(Mutex::new(Vec::new()));
        let id = self.next;
        let handle = SubscriberHandle {
            id,
            queue: Rc::downgrade(&queue),
        };
        self.subscribers.insert(id, queue);
        self.next += 1;
        handle
    }
    pub fn unsubscribe(&mut self, handle: SubscriberHandle<T>) {
        self.subscribers.retain(|k, _| *k != handle.id);
    }
}
