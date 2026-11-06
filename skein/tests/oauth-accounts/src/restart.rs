//! A host cut at a signal and started again, for the restart cuts of the
//! private-file and sign-in stories (skein/docs/design/simulator.md, 3.3).

use skein_io::kernel::{Complete, Fd, Op, Submit};
use skein_lib::{Queue, Time, Token, Wall};
use skein_world::Host;

const SIGNAL: Token = Token::new(0x7fff_ffff_ffff_ffff);
const ROOM: u32 = 64;

#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    Held,
    Closing,
    Closed,
}

/// A host retaining its inherited signal until its owner settles.
pub struct Restartable<P: Host> {
    pub(crate) inner: P,
    signal: Fd,
    state: State,
    /// Keep the signal open until the host has no work pending, as well as being empty.
    wait_idle: bool,
    issued: u32,
    // Preparation precedes completion routing; marks precede submission forwarding.
    prepare: fn(&mut P),
    mark: fn(&mut P, u32),
    completions: Queue<Complete>,
    submissions: Queue<Submit>,
}
impl<P: Host> Restartable<P> {
    pub(crate) fn new(inner: P, signal: Fd, wait_idle: bool, prepare: fn(&mut P), mark: fn(&mut P, u32)) -> Self {
        Self {
            inner,
            signal,
            state: State::Held,
            wait_idle,
            issued: 0,
            prepare,
            mark,
            completions: Queue::with_capacity(ROOM),
            submissions: Queue::with_capacity(ROOM),
        }
    }
}
impl<P: Host> Host for Restartable<P> {
    fn iterate(&mut self, now: Time, wall: Wall) {
        (self.prepare)(&mut self.inner);
        while let Some(complete) = self.completions.pop() {
            if complete.op == SIGNAL {
                assert!(complete.result.is_ok());
                self.state = State::Closed;
            } else {
                self.inner.completions().push(complete);
            }
        }
        self.inner.iterate(now, wall);
        (self.mark)(&mut self.inner, self.issued);
        while let Some(submit) = self.inner.submissions().pop() {
            assert_ne!(submit.op, SIGNAL, "finite child namespaces leave the signal token free");
            self.submissions.push(submit);
        }
        if self.inner.is_empty() && self.state == State::Held && (!self.wait_idle || !self.inner.work_pending(now)) {
            self.state = State::Closing;
            self.submissions.push(Submit { op: SIGNAL, kind: Op::Close { fd: self.signal } });
        }
        self.issued += self.submissions.len();
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        &mut self.completions
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        &mut self.submissions
    }
    fn work_pending(&self, now: Time) -> bool {
        !self.completions.is_empty()
            || self.inner.work_pending(now)
            || (self.inner.is_empty() && self.state == State::Held)
    }
    fn next_mechanism_deadline(&self) -> Option<Time> {
        self.inner.next_mechanism_deadline()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        self.inner.next_policy_deadline()
    }
    fn is_empty(&self) -> bool {
        self.state == State::Closed
            && self.inner.is_empty()
            && self.completions.is_empty()
            && self.submissions.is_empty()
    }
    fn operations(&self) -> u32 {
        self.inner.operations() + 1
    }
}
