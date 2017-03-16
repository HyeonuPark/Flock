use std::hash::Hash;

use event::{Command, Syscall};

pub trait Kernel: Sized {
    type Token: Clone + Send + Hash + Eq + 'static;
    type Data: Clone + Send + 'static;
    type OpenParam: Send + 'static;
    type Sink: Sink<Item = Syscall<Self>> + Send + 'static;

    fn create_token() -> Self::Token;

    fn create_sink(&self) -> Self::Sink;

    fn run<F>(&self, callback: F) where F: FnMut(Command<Self>);
}

pub trait Sink {
    type Item: Send;

    fn post(&self, msg: Self::Item);
}
