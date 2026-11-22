//! Bounds chosen by a sweep, independently of the world's workload (testing/fakes.md, 2).
use super::{LIMITS, REPO, World, forge, raw};
use skein_world::faults::Kind;
use skein_world::streams::Streams;

impl World {
    /// A batch with failed head checks as well as its landing workflow.
    #[must_use]
    pub fn bounds_in(streams: &Streams, workflow: raw::Workflow) -> World {
        let mut config = super::fake_config();
        config.limits.files = 8;
        let mut world = Self::configured_repositories(
            streams.seed(),
            LIMITS.facts,
            false,
            Some(skein_forge_client::Writer { forge: REPO.forge, author: 1 }),
            None,
            false,
            12,
            false,
            Some(Box::new([workflow])),
            config,
            Some(0),
            4,
            None,
            Some(streams),
            None,
        );
        world.env.limits.client.op_bytes = 2048;
        forge::manual_merges(
            &mut world.forge,
            b"org/repo",
            raw::ManualMerges { allow_manual_merge: true, autodetect_manual_merge: true },
        );
        world
    }

    /// Change the fake's file bound without changing the engine's admission limits.
    pub fn file_bound(&mut self, files: u32) -> u32 {
        std::mem::replace(&mut self.forge_env.limits.limits.files, files)
    }

    /// Record a draw excluded at the world's current state, before injecting it.
    pub fn leave_out(&mut self, scenario: &'static str, kind: Kind) {
        self.ledger.leave_out(scenario, kind, self.env.now);
    }
}
