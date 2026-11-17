//! A task document read once for a shown card, filter or requester (client/pages/needs-you.md, 1).
use super::{Request, handle};
use alloc::boxed::Box;
use jig_client::{Handle, Readable, Told, contract as c};
use skein_lib::Queue;

#[derive(Clone, Debug)]
pub struct TaskRead {
    pub number: u64,
    pub handle: Handle,
    pub document: Option<Box<c::Task>>,
}
impl TaskRead {
    pub(crate) fn new(number: u64, next: &mut u64, out: &mut Queue<Request>) -> Self {
        let handle = handle(*next);
        *next = next.checked_add(1).expect("page handles fit");
        out.push(Request::Client(jig_client::Event::Read {
            handle,
            read: Readable::Jig(c::Read::Task { task: number }),
        }));
        Self { number, handle, document: None }
    }
    pub(crate) fn told(&mut self, told: &Told) -> bool {
        if let Told::Page { handle, page } = told
            && *handle == self.handle
        {
            self.document = match super::super::decided::task_read(page, self.number) {
                Some((task, _)) => Some(Box::new(task.clone())),
                None => None,
            };
            return true;
        }
        false
    }
    pub(crate) fn leave(&self, out: &mut Queue<Request>) {
        out.push(Request::Client(jig_client::Event::Unwatch { handle: self.handle }));
    }
}
