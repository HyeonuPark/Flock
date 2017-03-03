use std::sync::mpsc::Receiver;

use kernel::{Kernel, Sink};
use event::Event;
use actor::ActorHandle;
use task::BoxTask;
use board::Board;

use event::Syscall::{Yield, Return, Listen, Ignore, Request, Spawn};
use event::CoreCommand::{Publish as CPublish, Listen as CListen, Ignore as CIgnore};
use event::KernelCommand::{ToCore as Core, Request as KRequest};

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct WorkerId(pub usize);

pub fn run<K: Kernel>(wid: WorkerId, initial_task: Option<BoxTask<K>>,
    downstream: Receiver<Event<K>>, upstream: K::Sink) {

    let mut board: Board<K::Token, K::Token, ActorHandle<K>> = Board::new();
    let mut exec_buf = Vec::new();
    let mut exec_stack = initial_task.into_iter()
        .map(|task| ActorHandle::new(K::create_token(), task))
        .collect::<Vec<_>>();

    macro_rules! accept {
        ($event:ident) => {
            if let Some(actors) = board.query(&$event.topic) {
                for actor in actors {
                    actor.inbox_mut().push_back($event.clone());

                    if actor.inbox().len() == 1 {
                        exec_buf.push((*actor).clone());
                    }
                }
            }
        }
    }

    macro_rules! process {
        ($actor: ident, $syscall: ident, $state: ident) => (match $syscall {
            Yield(data) => {
                upstream.post(Core(CPublish($actor.id(), data, false)));
            }
            Return(data) => {
                upstream.post(Core(CPublish($actor.id(), data, true)));

                for topic in $actor.interests().iter() {
                    board.unsubscribe(topic.clone(), $actor.id());
                }

                break;
            }
            Listen(topic) => {
                $actor.interests_mut().insert(topic.clone());
                upstream.post(Core(CListen(wid.clone(), topic)));
            }
            Ignore(topic) => {
                $actor.interests_mut().remove(&topic);
                upstream.post(Core(CIgnore(wid.clone(), topic)));
            }
            Request(topic, req) => {
                upstream.post(KRequest(wid.clone(), topic, req));
            }
            Spawn(token, task) => {
                $state = Some((token, task));
                break;
            }
        })
    }

    while let Ok(event) = downstream.recv() {
        accept!(event);

        loop {
            match exec_stack.pop() {
                Some(actor) => {
                    let mut state = None;

                    for syscall in actor.run() {
                        process!(actor, syscall, state);
                    }

                    if let Some((token, task)) = state {
                        exec_stack.push(actor);
                        exec_stack.push(ActorHandle::new(token, task));
                    }
                }
                None => match exec_buf.pop() {
                    Some(actor) => exec_stack.push(actor),
                    None => break,
                }
            }

            for event in downstream.try_iter() {
                accept!(event);
            }
        }
    }
}
