use std::hash::Hash;
use std::borrow::Borrow;
use std::iter;
use std::collections::{HashSet, HashMap};

use std::collections::hash_map::{Iter, IntoIter};
use std::collections::hash_map::Entry::{Occupied, Vacant};

use either::Either;

pub struct Broker<T: Clone + Hash + Eq, K: Hash + Eq, V: Clone> {
    values: HashMap<K, (V, HashSet<T>)>,
    board: HashMap<T, HashMap<K, V>>,
}

impl<T, K, V> Broker<T, K, V>
    where T: Clone + Hash + Eq, K: Hash + Eq, V: Clone {

    pub fn new() -> Self {
        Broker {
            values: HashMap::new(),
            board: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.values.insert(key, (value, HashSet::new()))
            .map(|(old_value, _)| old_value)
    }

    pub fn remove<Q: ?Sized>(&mut self, key: &Q) -> Option<V>
        where K: Borrow<Q>, Q: Hash + Eq {

        self.values.remove(key).map(|(old_value, topics)| {
            for topic in topics {
                self.board.get_mut(&topic).unwrap().remove(key);
            }

            old_value
        })
    }

    pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<&V>
        where K: Borrow<Q>, Q: Hash + Eq {

        self.values.get(key).map(|tp| &tp.0)
    }

    pub fn listen(&mut self, key: K, topic: T) -> bool {
        if let Some(&mut (ref value, ref mut topics)) =
            self.values.get_mut(&key) {

            topics.insert(topic.clone());
            let value = value.clone();

            match self.board.entry(topic) {
                Occupied(mut entry) => {
                    entry.get_mut().insert(key, value);
                }
                Vacant(entry) => {
                    let mut map = HashMap::new();
                    map.insert(key, value);
                    entry.insert(map);
                }
            }

            true
        } else {
            false
        }
    }

    pub fn ignore<Q: ?Sized>(&mut self, key: &Q, topic: &T) -> bool
        where K: Borrow<Q>, Q: Hash + Eq {

        if let Some(&mut (_, ref mut topics)) = self.values.get_mut(key) {
            if topics.remove(topic) {
                self.board.get_mut(topic).unwrap().remove(key);
                return true;
            }
        }

        false
    }

    pub fn query(&self, topic: &T)
        -> Either<Iter<K, V>, iter::Empty<(&K, &V)>> {

        match self.board.get(topic) {
            Some(listeners) => Either::Left(listeners.iter()),
            None => Either::Right(iter::empty()),
        }
    }

    pub fn remove_topic(&mut self, topic: &T)
        -> Either<IntoIter<K, V>, iter::Empty<(K, V)>> {

        match self.board.remove(topic) {
            Some(listeners) => {
                for key in listeners.keys() {
                    let &mut (_, ref mut topics) =
                        self.values.get_mut(key).unwrap();
                    topics.remove(topic);
                }

                Either::Left(listeners.into_iter())
            }
            None => Either::Right(iter::empty()),
        }
    }
}
