use event_loop::EventLoop;
use event::{InputOf, SyscallOf};

pub trait Task {
    type EventLoop: EventLoop;

    fn resume(&mut self, input: InputOf<Self::EventLoop>) -> SyscallOf<Self::EventLoop>;
}
