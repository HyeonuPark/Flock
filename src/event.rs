use worker::WorkerId;
use kernel::Kernel;
use task::Task;

pub struct Event<K: Kernel> {
    id: u64,
    pub topic: K::Token,
    pub data: K::Data,
    pub is_last: bool,
}

impl<K: Kernel> Event<K> {
    pub fn new(topic: K::Token, data: K::Data, is_last: bool, id_state: &mut u64) -> Self {
        let (nid, overflowed) = (*id_state).overflowing_add(1);
        assert!(!overflowed, "Failed to allocate a new id for the event");
        *id_state = nid;

        Event {
            id: nid,
            topic: topic,
            data: data,
            is_last: is_last,
        }
    }
}

impl<K: Kernel> Clone for Event<K> {
    fn clone(&self) -> Self {
        Event {
            id: self.id,
            topic: self.topic.clone(),
            data: self.data.clone(),
            is_last: self.is_last,
        }
    }
}

pub enum CoreCommand<K: Kernel> {
    Publish(K::Token, K::Data, bool),
    Listen(WorkerId, K::Token),
    Ignore(WorkerId, K::Token),
}

pub enum KernelCommand<K: Kernel> {
    ToCore(CoreCommand<K>),
    Request(WorkerId, K::Token, K::Request),
}

pub enum Syscall<K: Kernel> {
    Yield(K::Data),
    Return(K::Data),
    Listen(K::Token),
    Ignore(K::Token),
    Request(K::Token, K::Request),
    Spawn(K::Token, Box<Task<Kernel = K>>),
}
