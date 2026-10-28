//! Bind the fake before publishing its endpoint; its listener stays open into the story's world.
use skein_fake_peers::llm::Peer;
use skein_io::kernel::{Done, Fd};
use skein_lib::Queue;
use skein_shell::{Clock, Config, Host, Kernel, Wait};
use std::{fs, net::SocketAddr, path::Path};

pub struct Listening {
    peer: Peer,
    listener: Fd,
}

impl Listening {
    #[must_use]
    pub fn new(mut peer: Peer) -> Self {
        let clock = Clock::new();
        let deadline = clock.now().now.saturating_add(super::WITHIN);
        let mut kernel = Kernel::open(Config { operations: peer.operations() }).expect("fake listener kernel");
        let mut arrived = Queue::with_capacity(peer.operations());
        let mut listener = None;
        loop {
            kernel.reap(&mut arrived);
            while let Some(complete) = arrived.pop() {
                if let Ok(Done::Fd(fd)) = &complete.result {
                    assert!(listener.replace(*fd).is_none(), "only the listener opens before clients start");
                }
                peer.completions().push(complete);
            }
            let stamp = clock.now();
            assert!(stamp.now < deadline, "fake listener did not bind within its deadline");
            peer.iterate(stamp.now, stamp.wall);
            if peer.address().is_some() {
                // Accepts remain queued in the peer for the story's kernel. Nothing
                // remains in flight on this startup kernel; the descriptor stays open.
                assert_eq!(kernel.in_flight(), 0, "binding operations have completed");
                return Self { peer, listener: listener.expect("listening socket") };
            }
            kernel.submit(peer.submissions(), Wait::Until(deadline));
        }
    }

    #[must_use]
    pub fn address(&self) -> SocketAddr {
        self.peer.address().expect("bound fake listener")
    }

    /// Both consumers dial the same held listener, before either starts.
    pub fn configure(&self, directory: &Path) {
        let address = self.address().to_string();
        for name in ["temper.json", "agent.json"] {
            let path = directory.join(name);
            let mut value: serde_json::Value =
                serde_json::from_slice(&fs::read(&path).expect("LLM configuration")).expect("JSON");
            if name == "temper.json" {
                value["engine"]["llm"]["endpoints"][0]["address"] = serde_json::json!({"socket": address});
            } else {
                value["endpoints"][0]["address"] = address.clone().into();
            }
            super::write_private(&path, &serde_json::to_vec(&value).expect("LLM configuration JSON"));
        }
    }

    /// The world takes ownership of the still-open descriptor and the peer's queued accepts.
    pub(super) fn spawn<R: skein_world::Referee<super::Process>>(
        self,
        world: &mut skein_world::real::World<super::Process, R>,
    ) -> usize {
        world.spawn_with_fds(vec![self.listener], || super::Process::Peer(Box::new(self.peer)))
    }
}
