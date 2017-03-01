use std::hash::Hash;
use std::error::Error;

use event::Command;
use pubsub::Sink;

pub trait Rotor: Sized {
    type Token: Clone + Send + Hash + Eq + 'static;
    type Data: Clone + Send + 'static;
    type Request: Send + 'static;
    type Error: Error + 'static;
    type Sink: Sink<Item = Command<Self::Token, Self::Data,
        Self::Request>> + Clone + Send + 'static;

    /// Generate an unique token out of the thin air.
    ///
    /// Note that this function usually be called in worker thread.
    fn create_token() -> Self::Token;

    fn create_sink(&self) -> Self::Sink;

    fn start_stream(&self, token: Self::Token,
        param: Self::Request) -> Result<(), Self::Error>;

    /// Cancel living stream if possible.
    ///
    /// This method has default implementation, which simply do nothing.
    /// You can implement your own logic here if you have some cancelable stream.
    fn cancel_stream(&self, _token: Self::Token) -> Result<(), Self::Error> {
        Ok(())
    }

    fn run<F>(&self, callback: F)
        where F: FnMut(Command<Self::Token, Self::Data, Self::Request>)
            -> Result<(), Self::Error>;
}
