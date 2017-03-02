use kernel::Kernel;
use event::{Event, Syscall};

pub trait Task: Send {
    type Kernel: Kernel;

    fn resume(&mut self, input: &[Event<Self::Kernel>]) -> Option<Syscall<Self::Kernel>>;
}
