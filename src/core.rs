use std::sync::mpsc;
use std::iter;
use std::thread;
use std::fmt::Write;
use std::collections::HashMap;

use num_cpus;
use rand::{Rng, thread_rng};
use crossbeam::sync::chase_lev;

use kernel::Kernel;
use actor::{Actor, Task};
use worker::{self, WorkerId};
use event::{Event, EventId};
use broker::Broker;

use event::Command::{Publish, Kill, Spawn, Listen, Ignore, Sleep};

struct ActorState {
    pub wid: WorkerId,
    pub last_eid: EventId,
    pub is_sleep: bool,
}

impl ActorState {
    fn new(wid: WorkerId) -> Self {
        ActorState {
            wid: wid,
            last_eid: EventId(0),
            is_sleep: false,
        }
    }
}

fn new_event<K: Kernel>(topic: &K::Token, data: Option<K::Data>,
    id_ctx: &mut u64) -> Event<K> {

    match (*id_ctx).checked_add(1) {
        Some(nid) => {
            *id_ctx = nid;
            Event {
                id: EventId(nid),
                topic: topic.clone(),
                data: data,
            }
        }
        None => {
            // lie, in fact it's 2^64 - 1
            panic!("A single flock instance can create up to 2^64 events")
        }
    }
}

pub fn run<K: Kernel + 'static>(kernel: K, task: Task<K>) {
    Builder::new(kernel).run(task)
}

pub struct Builder<K: Kernel> {
    kernel: K,
    worker_count: usize,
    prefix: String,
}

impl<K: Kernel + 'static> Builder<K> {
    pub fn new(kernel: K) -> Self {
        Builder {
            kernel: kernel,
            worker_count: num_cpus::get(),
            prefix: "flock-".into(),
        }
    }

    pub fn num_workers(mut self, worker_count: usize) -> Self {
        self.worker_count = worker_count;
        self
    }

    pub fn name_prefix<T: Into<String>>(mut self, prefix: T) -> Self {
        self.prefix = prefix.into();
        self
    }

    pub fn run(self, task: Task<K>) {
        assert!(self.worker_count > 0);

        let (schedules, stealers): (Vec<_>, Vec<_>) =
            (0..self.worker_count)
            .map(|_| chase_lev::deque())
            .unzip();

        let (thread_senders, receivers): (HashMap<_, _>, Vec<_>) =
            (0..self.worker_count)
            .map(|idx| (WorkerId(idx), mpsc::channel()))
            .map(|(id, (sender, receiver))| ((id, sender), receiver))
            .unzip();

        let mut broker = Broker::<K::Token, _, _>::new();
        let mut actor_states = HashMap::new();

        let (init_sender, init_actor) = Actor::create(task);
        let init_state = ActorState::new(WorkerId(0));
        broker.insert(init_actor.id.clone(), init_sender);
        actor_states.insert(init_actor.id.clone(), init_state);

        let threads = iter::once(Some(init_actor))
            .chain(iter::repeat(()).map(|_| None))
            .enumerate()
            .zip(schedules)
            .zip(receivers)
            .map(|(((i, a), s), r)| (i, a, s, r))
            .map(|(index, actor, schedule, receiver)| {
                let id = WorkerId(index);

                let mut thread_name = String::new();
                write!(thread_name, "{}{}", self.prefix, index)
                    .expect("Failed to construct worker thread's name");

                let mut colleagues = stealers.iter()
                    .enumerate()
                    .filter(|&(idx, _)| idx != index)
                    .map(|(_, stealer)| stealer.clone())
                    .collect::<Vec<_>>();

                thread_rng().shuffle(colleagues.as_mut_slice());

                let sink = self.kernel.create_sink();

                thread::Builder::new()
                    .name(thread_name)
                    .spawn(move || {
                        worker::run::<K>(
                            id, actor,
                            schedule, colleagues,
                            receiver, sink,
                        );
                    })
                    .expect("Failed to spawn worker thread")
            })
            .collect::<Vec<_>>();

        let id_ctx = &mut 0;

        macro_rules! send {
            ($event: ident, $aid: ident, $sender: ident) => {
                $sender.send($event.clone()).unwrap();

                let state = actor_states.get_mut(&$aid).unwrap();
                state.last_eid = $event.id.clone();

                if state.is_sleep {
                    thread_senders[&state.wid].send($aid.clone()).unwrap();
                }
            }
        }

        self.kernel.run(|command| {
            match command {
                Publish(topic, data) => {
                    let event = new_event(&topic, Some(data), id_ctx);

                    for (aid, sender) in broker.query(&topic) {
                        send!(event, aid, sender);
                    }
                }
                Kill(topic) => {
                    let event = new_event(&topic, None, id_ctx);

                    for (aid, sender) in broker.remove_topic(&topic) {
                        send!(event, aid, sender);
                    }
                }
                Spawn(parent, child, sender) => {
                    let wid = actor_states[&parent].wid.clone();
                    actor_states.insert(child.clone(), ActorState::new(wid));
                    broker.insert(child.clone(), sender);

                    broker.listen(parent, child);
                }
                Listen(aid, topic) => {
                    broker.listen(aid, topic);
                }
                Ignore(aid, topic) => {
                    broker.ignore(&aid, &topic);
                }
                Sleep(wid, aid, last_eid) => {
                    let state = actor_states.get_mut(&aid).unwrap();

                    if state.last_eid > last_eid {
                        thread_senders[&wid].send(aid).unwrap();
                    } else {
                        state.is_sleep = true;
                    }
                }
            }
        });

        // shutdown phase
        for thread in threads {
            thread.join().unwrap();
        }
    }
}
