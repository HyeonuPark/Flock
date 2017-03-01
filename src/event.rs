use worker::WorkerId;
use rotor::Rotor;

#[derive(Clone)]
pub struct Event<Token, Data> {
    pub topic: Token,
    pub data: Data,
    pub is_last: bool,
}

pub enum Command<Token, Data, Req> {
    Publish(Event<Token, Data>),
    Request(WorkerId, Token, Req),
    Listen(WorkerId, Token),
    Ignore(WorkerId, Token),
}

pub type EventOf<R> = Event<<R as Rotor>::Token, <R as Rotor>::Data>;
pub type CommandOf<R> = Command<<R as Rotor>::Token, <R as Rotor>::Data, <R as Rotor>::Request>;

pub enum Input<T, D> {
    T(T),
    D(D),
}

pub enum Syscall<T, D, Q> {
    T(T),
    D(D),
    Q(Q),
}

pub type InputOf<R> = Input<<R as Rotor>::Token, <R as Rotor>::Data>;
pub type SyscallOf<R> = Syscall<<R as Rotor>::Token, <R as Rotor>::Data, <R as Rotor>::Request>;
