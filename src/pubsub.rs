use std::hash::Hash;
use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::sync::mpsc::Sender;

use event::Event;

pub struct Broker<T, D, K, S>
    where T: Hash + Eq, K: Hash + Eq, S: Sink<Item = Event<T, D>> {
        map: HashMap<T, HashMap<K, S>>,
    }

impl<T, D, K, S> Broker<T, D, K, S>
    where T: Clone + Hash + Eq, D: Clone, K: Hash + Eq, S: Sink<Item = Event<T, D>>{
    pub fn new() -> Self {
        Broker {
            map: HashMap::new(),
        }
    }

    pub fn publish(&self, event: Event<T, D>) {
        if let Some(listeners) = self.map.get(&event.topic) {
            for listener in listeners.values() {
                listener.post(event.clone());
            }
        }
    }

    pub fn subscribe(&mut self, topic: T, listener_key: K, listener: S) -> bool {
        match self.map.entry(topic) {
            Occupied(mut entry) => entry.get_mut().insert(listener_key, listener).is_none(),
            Vacant(entry) => {
                let mut listeners = HashMap::new();
                listeners.insert(listener_key, listener);
                entry.insert(listeners);
                true
            }
        }
    }

    pub fn unsubscribe(&mut self, topic: T, listener_key: K) -> bool {
        match self.map.get_mut(&topic) {
            Some(listeners) => listeners.remove(&listener_key).is_some(),
            None => false,
        }
    }
}

pub trait Sink {
    type Item;

    fn post(&self, msg: Self::Item);
}

impl<T> Sink for Sender<T> {
    type Item = T;

    fn post(&self, msg: T) {
        self.send(msg).unwrap();
    }
}
