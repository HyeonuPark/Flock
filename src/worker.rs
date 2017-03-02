use std::sync::mpsc::Receiver;

use kernel::Kernel;
use event::Event;
use actor::ActorHandle;
use board::Board;

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct WorkerId(pub usize);

pub fn run<K: Kernel>(id: WorkerId, inbox: Receiver<Event<K>>, upstream: K::Sink) {
    let mut board: Board<K::Token, K::Token, ActorHandle<K>> = Board::new();
    let mut exec_buf = Vec::new();

    while let Ok(event) = inbox.recv() {
        loop {
            for event in Some(event.clone()).into_iter().chain(inbox.try_iter()) {
                if let Some(actors) = board.query(&event.topic) {
                    for actor in actors {
                        actor.push(event.clone());

                        if actor.inbox_len() == 1 {
                            exec_buf.push((*actor).clone());
                        }
                    }
                }
            }

            match exec_buf.pop() {
                None => break,
                Some(actor) => actor.run(&upstream),
            }
        }
    }
}
