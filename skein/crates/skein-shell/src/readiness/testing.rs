//! Readiness observations for the real-loop tests (runtime/readiness.md, 4 and 5).

use super::{Config, Fd, Op, Queue, Submit, Token, Wait};
use crate::OpenError;
use skein_io::kernel::Complete;

pub struct Readiness(super::Readiness);

impl Readiness {
    pub fn open(config: Config) -> Result<Self, OpenError> {
        super::Readiness::open(config).map(Self)
    }

    pub fn submit(&mut self, records: &mut Queue<Submit>, wait: Wait) {
        self.0.submit(records, wait);
    }

    pub fn reap(&mut self, out: &mut Queue<Complete>) {
        self.0.reap(out);
    }

    #[must_use]
    pub fn read_waiter(&self, fd: Fd) -> Option<Option<u32>> {
        self.0.waiters.get(&fd).map(|waiters| waiters.read)
    }

    #[must_use]
    pub fn waiters_are_empty(&self) -> bool {
        self.0.waiters.is_empty()
    }

    #[must_use]
    pub fn ready_is_empty(&self) -> bool {
        self.0.ready.is_empty()
    }

    #[must_use]
    pub fn is_nonblocking(&self, fd: Fd) -> bool {
        self.0.nonblocking.contains(&fd)
    }

    #[must_use]
    pub fn nonblocking_is_empty(&self) -> bool {
        self.0.nonblocking.is_empty()
    }

    #[cfg(target_os = "linux")]
    pub fn read_interest(&mut self, fd: Fd, existed: bool, read: bool) -> Result<(), i32> {
        self.0.poller().interest(fd, existed, read, false)
    }
}

pub fn submit(kernel: &mut Readiness, token: u64, kind: Op) {
    submit_one(&mut kernel.0, token, kind);
}

pub(super) fn submit_one(kernel: &mut super::Readiness, token: u64, kind: Op) {
    let mut records = Queue::with_capacity(1);
    records.push(Submit { op: Token::new(token), kind });
    kernel.submit(&mut records, Wait::No);
    assert!(records.is_empty(), "the test kernel has room");
}
