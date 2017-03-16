use std::sync::mpsc::{Receiver, TryRecvError};
use std::collections::HashMap;

use crossbeam::sync::chase_lev::{Worker, Stealer};
use crossbeam::sync::chase_lev::Steal::{Abort, Data, Empty};

use kernel::{Kernel, Sink};
use actor::Actor;

use event::{EventId, Command as Cmd};
use event::Syscall::{Command, Open as SysOpen};
use event::Request::{Publish, Listen, Ignore, Open, Spawn, Kill};

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct WorkerId(pub usize);

pub fn run<K: Kernel>(wid: WorkerId, init_actor: Option<Actor<K>>,
    mut schedule: Worker<Actor<K>>, colleagues: Vec<Stealer<Actor<K>>>,
    downstream: Receiver<K::Token>, upstream: K::Sink) {

    let mut backlog = HashMap::<K::Token, Actor<K>>::new();

    if let Some(actor) = init_actor {
        schedule.push(actor);
    }

    let mut colleagues = colleagues.iter().cycle();

    let err_invalid_token = "Worker received invalid token from downstream";

    loop {
        loop {
            match downstream.try_recv() {
                Ok(token) => {
                    schedule.push(
                        backlog.remove(&token).expect(err_invalid_token)
                    );
                }
                Err(err) => match err {
                    TryRecvError::Empty => break,
                    TryRecvError::Disconnected => {
                        // cleanup logic here
                        return;
                    }
                }
            }
        }

        let maybe_actor = schedule.try_pop()
            .or_else(|| {
                let stealer = colleagues.next().unwrap();

                loop {
                    match stealer.steal() {
                        Empty => return None,
                        Abort => continue,
                        Data(actor) => return Some(actor),
                    }
                }
            });

        if let Some(mut actor) = maybe_actor {
            let actor_id = actor.id.clone();
            let id = || actor_id.clone();

            let mut flag_push_actor = false;
            let mut flag_kill = false;

            for req in &mut actor {
                upstream.post(match req {
                    Publish(data) => Command(Cmd::Publish(id(), data)),
                    Kill => {
                        flag_kill = true;

                        Command(Cmd::Kill(id()))
                    }
                    Listen(topic) => Command(Cmd::Listen(id(), topic)),
                    Ignore(topic) => Command(Cmd::Ignore(id(), topic)),
                    Open(param, topic) => SysOpen(id(), param, topic),
                    Spawn(task, topic) => {
                        let (sender, new_actor) = Actor::create(task);
                        schedule.push(new_actor);
                        flag_push_actor = true;

                        Command(Cmd::Spawn(id(), topic, sender))
                    }
                });

                if flag_push_actor || flag_kill {
                    break;
                }
            }

            if !flag_kill {
                let eid = match actor.last_eid() {
                    Some(eid) => eid,
                    None => EventId(0),
                };
                upstream.post(Command(Cmd::Sleep(wid.clone(), id(), eid)));
            }

            if flag_push_actor {
                schedule.push(actor);
            }
        }
    }
}
