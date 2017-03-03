use std::hash::Hash;
use std::collections::HashMap;

use std::collections::hash_map::Values;
use std::collections::hash_map::Entry::{Occupied, Vacant};

pub struct Board<T, I, S> where T: Hash + Eq, I: Hash + Eq {
    map: HashMap<T, HashMap<I, S>>,
}

impl<T: Hash + Eq, I: Hash + Eq, S> Board<T, I, S> where I: Hash + Eq {
    pub fn new() -> Self {
        Board {
            map: HashMap::new(),
        }
    }

    pub fn query(&self, topic: &T) -> Option<Values<I, S>> {
        self.map.get(topic).map(|listeners| listeners.values())
    }

    pub fn subscribe(&mut self, topic: T, listener_id: I, listener: S) -> bool {
        match self.map.entry(topic) {
            Occupied(mut entry) => {
                entry.get_mut().insert(listener_id, listener).is_none()
            }
            Vacant(entry) => {
                let mut listeners = HashMap::new();
                listeners.insert(listener_id, listener);
                entry.insert(listeners);
                true
            }
        }
    }

    pub fn unsubscribe(&mut self, topic: T, listener_id: I) -> bool {
        match self.map.get_mut(&topic) {
            Some(listeners) => listeners.remove(&listener_id).is_some(),
            None => false,
        }
    }
}
