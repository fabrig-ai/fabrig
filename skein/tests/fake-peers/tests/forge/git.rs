//! The forge's shared real-loop driver and private codec admission bounds.
use skein_fake_peers::forge::Peer;
use skein_shell::{Clock, Host, Kernel, Wait};
use skein_world::{Deadline, Expired};

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
