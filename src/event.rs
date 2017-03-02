use worker::WorkerId;
use event_loop::EventLoop;

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

pub type EventOf<L> = Event<<L as EventLoop>::Token, <R as EventLoop>::Data>;
pub type CommandOf<L> = Command<<L as EventLoop>::Token,
    <L as EventLoop>::Data, <L as EventLoop>::Request>;

pub enum Input<T, D> {
    T(T),
    D(D),
}

pub enum Syscall<T, D, Q> {
    T(T),
    D(D),
    Q(Q),
}

pub type InputOf<L> = Input<<L as EventLoop>::Token, <L as EventLoop>::Data>;
pub type SyscallOf<L> = Syscall<<L as EventLoop>::Token,
    <L as EventLoop>::Data, <L as EventLoop>::Request>;
