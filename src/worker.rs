use std::sync::mpsc::Receiver;

use event::Event;

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct WorkerId(pub usize);

pub struct Worker<Token, Data, Sink> {
    id: WorkerId,
    inbox: Receiver<Event<Token, Data>>,
    upstream: Sink,
}

impl<T, D, S> Worker<T, D, S> {
    pub fn new(id: WorkerId, inbox: Receiver<Event<T, D>>, upstream: S) -> Self {
        Worker {
            id: id,
            inbox: inbox,
            upstream: upstream,
        }
    }

    pub fn run(self) {
        unimplemented!()
    }
}
