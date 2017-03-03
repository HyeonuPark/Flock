use std::collections::vec_deque::Drain;

use kernel::Kernel;
use event::{Event, Syscall};

pub trait Task: Send {
    type Kernel: Kernel;

    fn resume(&mut self, events: Drain<Event<Self::Kernel>>)
        -> Option<Syscall<Self::Kernel>>;
}

pub type BoxTask<K> = Box<Task<Kernel = K>>;
