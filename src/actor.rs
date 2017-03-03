use std::collections::{VecDeque, HashSet};
use std::rc::Rc;
use std::cell::{RefCell, Ref, RefMut};

use kernel::Kernel;
use event::{Event, Syscall};
use task::Task;

pub struct Actor<K: Kernel> {
    id: K::Token,
    inbox: VecDeque<Event<K>>,
    interests: HashSet<K::Token>,
    task: Box<Task<Kernel = K>>,
}

pub struct ActorHandle<K: Kernel>(Rc<RefCell<Actor<K>>>);

impl<K: Kernel> ActorHandle<K> {
    pub fn new(token: K::Token, task: Box<Task<Kernel = K>>) -> ActorHandle<K> {
        ActorHandle(Rc::new(RefCell::new(Actor {
            id: token,
            inbox: VecDeque::new(),
            interests: HashSet::new(),
            task: task,
        })))
    }

    pub fn id(&self) -> K::Token {
        self.0.borrow().id.clone()
    }

    pub fn inbox(&self) -> Ref<VecDeque<Event<K>>> {
        Ref::map(self.0.borrow(), |a| &a.inbox)
    }

    pub fn inbox_mut(&self) -> RefMut<VecDeque<Event<K>>> {
        RefMut::map(self.0.borrow_mut(), |a| &mut a.inbox)
    }

    pub fn interests(&self) -> Ref<HashSet<K::Token>> {
        Ref::map(self.0.borrow(), |a| &a.interests)
    }

    pub fn interests_mut(&self) -> RefMut<HashSet<K::Token>> {
        RefMut::map(self.0.borrow_mut(), |a| &mut a.interests)
    }

    pub fn run(&self) -> SyscallIter<K> {
        SyscallIter {
            inbox: self.inbox_mut(),
            task: RefMut::map(self.0.borrow_mut(), |a| &mut a.task),
        }
    }
}

impl<K: Kernel> Clone for ActorHandle<K> {
    fn clone(&self) -> Self {
        ActorHandle(self.0.clone())
    }
}

pub struct SyscallIter<'a, K: Kernel + 'a> {
    inbox: RefMut<'a, VecDeque<Event<K>>>,
    task: RefMut<'a, Box<Task<Kernel =K>>>,
}

impl<'a, K: Kernel> Iterator for SyscallIter<'a, K> {
    type Item = Syscall<K>;

    fn next(&mut self) -> Option<Self::Item> {
        self.task.resume(self.inbox.drain(..))
    }
}
