use std::hash::Hash;
use std::error::Error;

use event::{KernelCommand, CoreCommand};

pub trait Kernel: Sized {
    type Token: Clone + Send + Hash + Eq + 'static;
    type Data: Clone + Send + 'static;
    type Request: Send + 'static;
    type Error: Error + 'static;
    type Sink: Sink<Item = KernelCommand<Self>> + Send + 'static;

    /// Generate an unique token out of the thin air.
    ///
    /// Note that this function usually be called in worker thread.
    fn create_token() -> Self::Token;

    fn create_sink(&self) -> Self::Sink;

    fn run<F>(&self, callback: F) where F: FnMut(CoreCommand<Self>);
}

pub trait Sink {
    type Item;

    fn post(&self, msg: Self::Item);
}
