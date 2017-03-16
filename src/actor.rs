use std::sync::mpsc::{self, Sender, Receiver};

use kernel::Kernel;
use event::{Event, EventId, Request};

pub type Task<K> = Box<(FnMut(&[Event<K>]) -> Option<Request<K>>) + Send>;

pub struct Actor<K: Kernel> {
    pub id: K::Token,
    task: Task<K>,
    inbox: Receiver<Event<K>>,
    inbox_buf: Vec<Event<K>>,
}

impl<K: Kernel> Actor<K> {
    pub fn create(task: Task<K>) -> (Sender<Event<K>>, Self) {
        let (sender, receiver) = mpsc::channel();
        let actor = Actor {
            id: K::create_token(),
            task: task,
            inbox: receiver,
            inbox_buf: Vec::new(),
        };

        (sender, actor)
    }

    pub fn last_eid(&self) -> Option<EventId> {
        self.inbox_buf.last().map(|event| event.id.clone())
    }
}

impl<'a, K: Kernel> Iterator for &'a mut Actor<K> {
    type Item = Request<K>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inbox_buf.clear();
        self.inbox_buf.extend(self.inbox.try_iter());
        (self.task)(self.inbox_buf.as_slice())
    }
}
