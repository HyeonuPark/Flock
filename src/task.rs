use rotor::Rotor;
use event::{InputOf, SyscallOf};

pub trait Task {
    type Rotor: Rotor;

    fn resume(&mut self, input: InputOf<Self::Rotor>) -> SyscallOf<Self::Rotor>;
}
