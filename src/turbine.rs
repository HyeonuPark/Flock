use std::convert::Into;
use std::sync::mpsc;
use std::thread;
use std::collections::HashMap;
use std::fmt::Write;

use num_cpus;

use rotor::Rotor;
use task::Task;
use worker::{Worker, WorkerId};
use pubsub::Broker;

use event::Command::{Publish, Request, Listen, Ignore};

pub struct Turbine<R: Rotor> {
    rotor: R,
    worker_count: usize,
    prefix: String,
}

impl<R: Rotor> Turbine<R> {
    pub fn new(rotor: R) -> Self {
        Turbine {
            rotor: rotor,
            worker_count: num_cpus::get(),
            prefix: "turbine-".into(),
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

    pub fn run(self, task: Box<Task<Rotor = R>>) {
        assert!(self.worker_count > 0);

        let (rotor, worker_count, prefix) = (&self.rotor, self.worker_count, self.prefix);

        let mut broker = Broker::new();
        let sink = rotor.create_sink();
        let channels = (0..worker_count).map(|_| mpsc::channel());

        let mut workers = HashMap::new();
        let mut threads = Vec::new();

        for (index, (sender, receiver)) in channels.enumerate() {
            let id = WorkerId(index);
            let worker = Worker::new(id.clone(), receiver, sink.clone());

            let mut thread_name = String::new();
            write!(thread_name, "{}{}", prefix, index)
                .expect("Failed to construct worker thread's name");

            let thread_handle = thread::Builder::new()
                .name(thread_name)
                .spawn(move || { worker.run(); })
                .expect("Failed to spawn worker thread");

            threads.push(thread_handle);
            workers.insert(id, sender);
        }

        // TODO: pass initial task to some worker to execute it

        rotor.run(|command| {
            match command {
                Publish(event) => {
                    broker.publish(event);
                    Ok(())
                }
                Request(wid, token, req) => {
                    broker.subscribe(token.clone(), wid.clone(), workers[&wid].clone());
                    rotor.start_stream(token, req)
                }
                Listen(wid, token) => {
                    broker.subscribe(token, wid.clone(), workers[&wid].clone());
                    Ok(())
                }
                Ignore(wid, token) => {
                    broker.unsubscribe(token, wid);
                    Ok(())
                }
            }
        });

        for handle in threads {
            handle.join().unwrap();
        }
    }
}
