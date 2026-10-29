//! The forge's shared real-loop driver and private codec admission bounds.
use super::fixtures as f;
use skein_fake_forge_protocol::git;
use skein_fake_peers::{Error, forge::Peer};
use skein_shell::{Clock, Host, Kernel, Wait};
use skein_world::{Deadline, Expired};
use std::rc::Rc;

pub(super) struct Loop {
    pub(super) peer: Peer,
    pub(super) kernel: Kernel,
    pub(super) clock: Clock,
}
impl Loop {
    pub(super) fn pass(&mut self) {
        self.kernel.reap(self.peer.completions());
        let now = self.clock.now();
        self.peer.iterate(now.now, now.wall);
        self.kernel.submit(self.peer.submissions(), Wait::No);
    }
    pub(super) fn expired(&mut self, deadline: &Deadline) -> ! {
        match deadline.expired() {
            Expired::Starved(starved) => {
                self.peer.shutdown();
                while !self.peer.is_empty() || self.kernel.in_flight() > 0 {
                    self.pass();
                }
                starved.pass();
            }
            Expired::Failed(figures) => panic!("the fake forge wait did not settle: {figures}"),
        }
    }
}
#[test]
fn peer_git_tree_room_holds_at_bound_and_refuses_one_less() {
    let shared = f::shared();
    let bound = git::tree_bound(&shared.borrow().config.limits).expect("tree bound");
    for (label, writer, request, accepted) in [
        ("answer bound", bound.answer, u64::from(bound.request), true),
        ("answer short", bound.answer.saturating_sub(1), u64::from(bound.request), false),
        ("request short", bound.answer, u64::from(bound.request.saturating_sub(1)), false),
    ] {
        let mut limits = f::protocol_limits();
        limits.documents.writer.length = writer;
        limits.http.body = request;
        let result = Peer::new(f::address(), f::limits(), f::config(), limits, Rc::clone(&shared), f::SEED);
        assert_eq!(result.err(), if accepted { None } else { Some(Error::Limits) }, "{label}");
    }
}
