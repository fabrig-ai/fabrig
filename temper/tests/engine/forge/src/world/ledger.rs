//! The forge world's ledger and streams (temper/docs/design/testing/fakes.md, 2).
use super::{LIMITS, REPO, World, forge, raw};
use crate::translate;
use skein_world::{
    faults::{FaultLedger, Injection, Kind, Struck},
    streams::Streams,
};

impl World {
    /// The ordinary connector fixture, with independent fault and scheduling streams.
    #[must_use]
    pub fn new_in(streams: &Streams, config: forge::Config) -> World {
        Self::configured_repositories(
            streams.seed(),
            LIMITS.facts,
            false,
            Some(skein_forge_client::Writer { forge: REPO.forge, author: 1 }),
            None,
            false,
            4,
            false,
            None,
            config,
            None,
            LIMITS.landing_members,
            None,
            Some(streams),
        )
    }

    /// A batch turn with a sweep's forge configuration and independent streams.
    #[must_use]
    pub fn batch_turn_in(streams: &Streams, workflow: Option<raw::Workflow>, mut config: forge::Config) -> World {
        config.limits.files = 8;
        Self::configured_repositories(
            streams.seed(),
            LIMITS.facts,
            false,
            Some(skein_forge_client::Writer { forge: REPO.forge, author: 1 }),
            None,
            false,
            12,
            true,
            workflow.map(|workflow| Box::<[raw::Workflow]>::from([workflow])),
            config,
            None,
            4,
            None,
            Some(streams),
        )
    }

    #[must_use]
    pub fn ledger(&self) -> &FaultLedger {
        &self.ledger
    }

    /// Return this run's ledger, leaving an empty one in the world.
    pub fn take_ledger(&mut self) -> FaultLedger {
        std::mem::take(&mut self.ledger)
    }

    /// Record a world's own fault as it starts, at the world's clock.
    pub fn inject(&mut self, kind: Kind, struck: Struck) -> Injection {
        self.ledger.inject(kind, self.env.now, struck)
    }

    pub fn stop(&mut self, entry: Injection) {
        self.ledger.stop(entry, self.env.now).expect("world fault stops once, after it starts");
    }

    /// Borrow the fake for a sweep's scenario actions.
    pub fn forge_mut(&mut self) -> &mut forge::Domain {
        &mut self.forge
    }

    pub(super) fn copy_fell(&mut self) {
        let lost = self.forge.fell_lost();
        assert_eq!(lost, 0, "seed {}: forge lost {lost} fault records", self.seed);
        while let Some(fell) = self.forge.pop_fell() {
            let entry = self.ledger.inject(
                Kind { family: fell.fault.family(), name: fell.fault.name() },
                fell.at,
                Struck {
                    request: fell.reply_to.map(|token| token.raw().to_string()),
                    gate_run: fell.run.map(|run| run.to_string()),
                    runner: fell.runner.map(|runner| runner.to_string()),
                    candidate: fell.commit.map(|commit| {
                        String::from_utf8(temper_engine_domain_forge::commit_text(&translate::commit(commit)).into())
                            .expect("commit hex is UTF-8")
                    }),
                    ..Struck::default()
                },
            );
            self.ledger.stop(entry, fell.until.unwrap_or(fell.at)).expect("fake fault interval");
        }
    }
}
