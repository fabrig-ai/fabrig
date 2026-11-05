//! Home's looks through the joined roots (temper/docs/design/domain/testing.md, 2.1).
use super::{Joint, client, engine_limits, fixtures, walking};
use jig_client_world::harness::ClientId;
use skein_lib::Duration;
use temper_engine_domain::{Parts, Record};
use temper_engine_domain_world::{forge_stand::ForgeStand, store::Store};

const SESSION: Duration = Duration::from_secs(2 * 60 * 60);

impl Joint {
    /// The person's session spans two sittings of Home's fake clock.
    #[must_use]
    pub fn with_status_people(seed: u64, limits: client::Limits) -> Self {
        let mut bounds = engine_limits();
        bounds.people.sign_in_lifetime = SESSION;
        let configs = std::array::from_fn(|_| {
            let mut config = walking::config(seed);
            config.core.connectors = Box::new([fixtures::FORGE]);
            // As in dismissed.rs, reserve watches for the people's facts rather than goals.
            config.core.defaults.templates = config
                .core
                .defaults
                .templates
                .into_vec()
                .into_iter()
                .filter(|template| template.number != jig_client::shapes::GOAL)
                .collect();
            config
        });
        Self::with_engine(seed, &limits, Store::new(), configs, &bounds, 1)
    }

    /// Home and a pushed change share the forge's real CI, review and landing path.
    #[must_use]
    pub fn with_status_landing(seed: u64, limits: client::Limits) -> Self {
        let mut front = Self::change_landing(seed, &limits, Some(SESSION), false);
        front.engine.forge = Some(ForgeStand::with_checks(fixtures::REPOSITORIES, &[b"build"]));
        fixtures::adopt_change_repository(&mut front);
        front
    }

    /// Hand an already pushed branch to the real change procedure.
    pub fn begin_status_landing(&mut self) -> u64 {
        const BRANCH: &[u8] = b"temper/status";
        self.engine.forge.as_mut().expect("fake forge").push_branch(7, BRANCH);
        // The fixture reads its caller from a durable sign-in reply; Home has drained the adoption's.
        self.engine.sign_in();
        self.engine.settle();
        fixtures::goal_with_pushed_change(self, BRANCH)
    }

    /// Answer the blocking gate once the worker has received its assignment.
    pub fn approve_status_landing(&mut self, goal: u64) {
        use jig_domain_tasks as tasks;
        let gate = self.engine.store.rows.values().find_map(|record| {
            if let Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row))) = record
                && row.root == goal
                && row.number != goal
                && self.hosted_attempt(row.number) == Some(row.attempt)
                && matches!(row.contract, tasks::Contract::Verdict { .. })
            {
                Some((row.number, row.attempt))
            } else {
                None
            }
        });
        if let Some((task, attempt)) = gate {
            self.engine.answer(
                fixtures::QUESTION_WORKER,
                task,
                attempt,
                0,
                tasks::End::Finished {
                    result: tasks::TaskResult::Verdict {
                        code: 1,
                        fields: Box::new([]),
                        words: b"approved".as_slice().into(),
                    },
                    cancel_delegates: false,
                },
                Parts::none(),
            );
            self.engine.settle();
        }
    }

    /// Wire sends of project 1's entry look, including duplicate attempts.
    #[must_use]
    pub fn status_looks(&self, who: ClientId) -> usize {
        self.status_looks.get(&who).copied().unwrap_or(0)
    }

    pub(super) fn count_status_look(&mut self, who: ClientId, request: &client::Request) {
        if let client::Request::Unlinked {
            request: jig_client::UnlinkedRequest::Down { down: jig_client::Down::Send { request, .. } },
        } = request
            && Some(request) == client::forge::status::ask(1, &self.limits.forge).as_ref()
        {
            *self.status_looks.entry(who).or_default() += 1;
        }
    }
}
