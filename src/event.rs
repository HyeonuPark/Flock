use std::sync::mpsc::Sender;

use kernel::Kernel;
use worker::WorkerId;
use actor::Task;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct EventId(pub u64);

pub struct Event<K: Kernel> {
    pub id: EventId,
    pub topic: K::Token,
    pub data: Option<K::Data>,
}

pub enum Command<K: Kernel> {
    Publish(K::Token, K::Data),
    Kill(K::Token),
    Listen(K::Token, K::Token),
    Ignore(K::Token, K::Token),
    Spawn(K::Token, K::Token, Sender<Event<K>>),
    Sleep(WorkerId, K::Token, EventId),
}

pub enum Syscall<K: Kernel> {
    Command(Command<K>),
    Open(K::Token, K::OpenParam, K::Token),
}

pub enum Request<K: Kernel> {
    Publish(K::Data),
    Kill,
    Listen(K::Token),
    Ignore(K::Token),
    Open(K::OpenParam, K::Token),
    Spawn(Task<K>, K::Token),
}

impl<K: Kernel> Clone for Event<K> {
    fn clone(&self) -> Self {
        Event {
            id: self.id.clone(),
            topic: self.topic.clone(),
            data: self.data.clone(),
        }
    }
}
