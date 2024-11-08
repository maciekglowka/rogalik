// use std::{
//     collections::{HashMap, VecDeque},
//     rc::{Rc, Weak},
//     sync::Mutex,
// };

// pub struct SubscriberHandle<T> {
//     bus: Weak<Mutex<EventBus<T>>>,
//     id: usize,
// }

// #[derive(Default)]
// pub struct Subscriber {
//     read_until: usize,
// }

// #[derive(Default)]
// pub struct EventQueue<T> {

//     events: VecDeque<T>,
//     subscribers: HashMap<usize, Subscriber>,
// }

// #[derive(Default)]
// pub struct EventBus<T> {
//     next: usize
//     queue: Rc<Mutex<EventQueue<T>>>
// }
// impl<T> EventBus<T> {
//     pub fn publish(&mut self, event: T) {
//         if let Ok(mut queue) = self.queue.lock() {
//             queue.events.push_back(event);
//         }
//     }
//     // pub fn subscribe(&mut self) -> SubscriberHandle<T> {
//     //     let id = self.next;
//     //     self.next += 1;
//     //     let subscriber = Subscriber::default();
//     //     self.subscribers.insert(&id, subscriber);
//     //     SubscriberHandle {
//     //         id,

//     //     }
//     // }
//     pub fn read_events(&mut self, subscriber_id: usize) -> Option<impl Iterator<Item = &T>> {
//         if let Ok(mut queue) = self.queue.lock() {
//             let subscriber = queue.subscribers.get_mut(&subscriber_id)?;
//             subscriber.read_until = queue.events.len().saturating_sub(1);
//             Some(queue.events.iter())
//         } else {
//             None
//         }
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn single_subscriber() {
//         let mut bus = EventBus::default();

//         for i in 0..5 {
//             bus.publish(format!("Some String {}", i));
//         }
//         // assert!(a.intersects(&b));
//     }
// }
