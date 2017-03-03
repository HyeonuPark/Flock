use std::collections::{VecDeque, HashSet};
use std::rc::Rc;
use std::cell::RefCell;

use kernel::Kernel;
use event::Event;
use task::Task;

pub struct Actor<K: Kernel> {
    inbox: VecDeque<Event<K>>,
    interests: HashSet<K::Token>,
    task: Box<Task<Kernel = K>>,
}

pub struct ActorHandle<K: Kernel>(Rc<RefCell<Actor<K>>>);

impl<K: Kernel> ActorHandle<K> {
    pub fn new(task: Box<Task<Kernel = K>>) -> ActorHandle<K> {
        ActorHandle(Rc::new(RefCell::new(Actor {
            inbox: VecDeque::new(),
            interests: HashSet::new(),
            task: task,
        })))
    }

    pub fn push(&self, event: Event<K>) {
        self.0.borrow_mut().inbox.push_back(event);
    }

    pub fn inbox_len(&self) -> usize {
        self.0.borrow().inbox.len()
    }

    pub fn run(&self, upstream: &K::Sink) {
        unimplemented!()
    }
}

impl<K: Kernel> Clone for ActorHandle<K> {
    fn clone(&self) -> Self {
        ActorHandle(self.0.clone())
    }
}
