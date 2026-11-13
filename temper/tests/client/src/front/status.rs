//! Home's looks through the joined roots (temper/docs/design/testing/README.md, 2.1).
use super::{Joint, client, engine_limits, fixtures, walking};
use jig_client_kit::harness::ClientId;
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
        let mut front = Self::change_landing(seed, &limits, Some(SESSION), false, false, None);
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

impl Joint {
    /// Stock funding and delegation beside the landing stand; the person adds their plan by sign-in
    /// (temper/docs/design/domain/forge/attention.md, 3; temper/docs/design/protocol/configuration/stock.md).
    #[must_use]
    pub fn with_plan_status(seed: u64, limits: client::Limits) -> Self {
        let mut bounds = temper_engine_domain_world::forge_routes::box_limits();
        bounds.people.sign_in_lifetime = SESSION;
        bounds.tasks.run_inbox = temper_engine_domain_world::protocol::bounds().inbox;
        let configs = std::array::from_fn(|_| {
            let mut config = walking::config(seed);
            let models = config
                .core
                .settings
                .models
                .iter()
                .enumerate()
                .map(|(at, model)| jig_domain_core::settings::CatalogueModel {
                    first_read: None,
                    number: u32::try_from(at).expect("fixture model number") + 1,
                    model: model.clone(),
                    shown: None,
                    description: None,
                    picker: true,
                    withdrawn: false,
                    source: jig_domain_core::settings::ModelSource::Stock,
                    estimated: false,
                })
                .collect();
            let mut defaults = temper_engine_service_stock::stock::with_models(models, bounds.tasks.result_bytes);
            // Scripted workers need the world's small run bounds, with stock policy and delegation.
            for charter in &mut defaults.charters {
                let number = charter.number;
                *charter = walking::charter();
                charter.number = number;
            }
            config.core.seed_projects[0].policy = defaults.policies.get(&1).expect("stock policy 1").clone();
            config.core.defaults = defaults;
            config.core.connectors = Box::new([fixtures::FORGE]);
            let provider = &mut config.core.settings.llm_providers.providers[0];
            provider.name = b"openai".as_slice().into();
            provider.signed_in_plan = true;
            let landing = temper_engine_domain::LandingRule {
                sets: Box::new([]),
                connector: fixtures::FORGE,
                kind: 4,
                enforce: true,
                pattern: jig_domain_authority::Pattern {
                    segments: [b"forge".as_slice(), fixtures::FORGE_HOST, b"acme", b"shop", b"branch"]
                        .map(Box::from)
                        .into(),
                    last: jig_domain_authority::Last::Exact(b"main".as_slice().into()),
                },
                ci: true,
                up_to_date: true,
                gates: Box::new([temper_engine_domain::Gate {
                    number: 1,
                    blocking: true,
                    freshness: temper_engine_domain::Freshness::Exact,
                }]),
                approvals: Box::new([]),
            };
            config.kinds_stock =
                temper_engine_domain::landing_stock(&[landing], &skein_lib::Map::with_capacity(0), 1, 4);
            config
        });
        let bootstrap = temper_engine_domain_world::oracle::config::copy(&configs[0]);
        // Start with production's durable rows and no deployment account sources.
        let mut driver =
            temper_engine_domain_world::direct::Driver::cold(Store::with_limits(&bounds), bootstrap, &bounds);
        driver.period_sent = true;
        driver.settle();
        driver.send(temper_engine_domain::Event::Party(temper_engine_domain::PartyInput::Period {
            project: 1,
            period: 1,
            budget: configs[1].core.defaults.rules.period_spend,
            next: None,
        }));
        driver.settle();
        let mut store = driver.take_store();
        store.seed(plan_status_forge(&configs[1].forges[0]), Box::new([]));
        let mut front = Self::with_engine(seed, &limits, store, configs, &bounds, 1);
        // The story answers assigned workers while their run deadlines remain armed.
        front.settles_with_a_deadline_armed = true;
        front.engine.forge = Some(ForgeStand::with_checks(fixtures::REPOSITORIES, &[b"build"]));
        front
    }
}

fn plan_status_forge(configured: &temper_engine_domain_forge::Configured) -> Box<[Record]> {
    use temper_engine_domain_forge::{Address, Bot, ForgeRecord, ForgeState, Place, Stored};
    Box::new([
        Record::Forge {
            row: Box::new(Stored::Forge(ForgeRecord {
                person: None,
                git_program: None,
                number: configured.number,
                kind: configured.kind,
                mode: configured.mode,
                state: ForgeState::Kept,
                place: Place {
                    api: Address { host: fixtures::FORGE_HOST.into(), port: 443, tls: None },
                    git: configured.git.clone(),
                    web: configured.web.clone(),
                },
                read_only: configured.read_only,
                bot: Bot { login: configured.bot.clone(), name: Box::new([]), email: Box::new([]) },
                asked: configured.asked,
                first: None,
            })),
        },
        Record::Forge { row: Box::new(Stored::ForgeNumber { last: configured.number }) },
    ])
}
