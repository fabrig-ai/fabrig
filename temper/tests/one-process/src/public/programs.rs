//! Programs composed on one simulated machine (temper/docs/design/testing/README.md, 2.5).
use super::{
    http::{self, Answer},
    proxy::Proxy,
    visitor::Visitor,
    wire::{Event, Wire},
};
use skein_io::kernel::{Complete, Exit, Submit};
use skein_lib::{Queue, Time, Wall};
use skein_shell::Host;
use std::net::SocketAddr;

pub(super) struct Upstream {
    wire: Wire,
    pub answer: Answer,
    pub requests: Vec<Box<[u8]>>,
    pub hold: bool,
    held: Vec<u64>,
}
impl Upstream {
    pub fn new(address: SocketAddr, answer: Answer) -> Self {
        Self { wire: Wire::new(Some(address)), answer, requests: Vec::new(), hold: false, held: Vec::new() }
    }
    pub fn ready(&self) -> bool {
        self.wire.ready
    }
    pub fn idle(&self) -> bool {
        self.wire.idle()
    }
    pub fn release(&mut self) {
        self.hold = false;
        for id in self.held.drain(..) {
            self.wire.respond(id, self.answer.wire());
        }
    }
    pub fn stop(&mut self) {
        self.wire.stop();
    }
}
impl Host for Upstream {
    fn iterate(&mut self, _: Time, _: Wall) {
        self.wire.iterate();
        while let Some(event) = self.wire.events.pop_front() {
            match event {
                Event::Request { id, bytes, .. } => {
                    http::request(&bytes).expect("scripted upstream's HTTP request");
                    self.requests.push(bytes);
                    if self.hold {
                        self.held.push(id);
                    } else {
                        self.wire.respond(id, self.answer.wire());
                    }
                }
                Event::Answer { .. } => panic!("scripted upstream never connects"),
            }
        }
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        &mut self.wire.completions
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        &mut self.wire.submissions
    }
    fn work_pending(&self, _: Time) -> bool {
        !self.wire.completions.is_empty() || !self.wire.submissions.is_empty()
    }
    fn next_mechanism_deadline(&self) -> Option<Time> {
        None
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        None
    }
    fn is_empty(&self) -> bool {
        self.wire.empty()
    }
    fn operations(&self) -> u32 {
        256
    }
}

pub(super) enum Program {
    Proxy(Box<Proxy>),
    Visitor(Box<Visitor>),
    Upstream(Box<Upstream>),
}
impl Program {
    fn host(&self) -> &dyn Host {
        match self {
            Self::Proxy(h) => h.as_ref(),
            Self::Visitor(h) => h.as_ref(),
            Self::Upstream(h) => h.as_ref(),
        }
    }
    fn host_mut(&mut self) -> &mut dyn Host {
        match self {
            Self::Proxy(h) => h.as_mut(),
            Self::Visitor(h) => h.as_mut(),
            Self::Upstream(h) => h.as_mut(),
        }
    }
}
impl Host for Program {
    fn drain(&mut self) {
        self.host_mut().drain();
    }
    fn iterate(&mut self, now: Time, wall: Wall) {
        self.host_mut().iterate(now, wall);
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        self.host_mut().completions()
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        self.host_mut().submissions()
    }
    fn work_pending(&self, now: Time) -> bool {
        self.host().work_pending(now)
    }
    fn next_mechanism_deadline(&self) -> Option<Time> {
        self.host().next_mechanism_deadline()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        self.host().next_policy_deadline()
    }
    fn is_empty(&self) -> bool {
        self.host().is_empty()
    }
    fn exit(&self) -> Option<Exit> {
        self.host().exit()
    }
    fn operations(&self) -> u32 {
        self.host().operations()
    }
}
