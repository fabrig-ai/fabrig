//! Seeded configuration and world construction.
use super::{
    Boot, Duration, ReplyTo, Setup, Store, System, Token, VecDeque, World, accounts, connector, core, fleet, people,
    root,
};
use jig_domain_link::{Form, Hello, Scheme};
pub use jig_test_profile::effect_fixture as fixture;

pub(super) fn limits() -> root::Limits {
    let mut limits = jig_test_profile::limits();
    limits.core.authority.segments = 2;
    limits.core.people.requests = 16;
    limits.connector.write_lifetime = Duration::from_secs(1);
    limits
}

fn initial_request(configuration: &root::Config, creating: &jig_test_profile::fixtures::Creating) -> people::Ask {
    if configuration.core.seed_projects.is_empty() {
        people::Ask::MarkSeen { seen: people::Seen::Results { through: 0 } }
    } else {
        jig_test_profile::fixtures::create_chat(creating, 1, 0, b"make an effect".as_slice().into())
    }
}

impl World {
    /// Start the root, store and selected scripted neighbours from one setup.
    #[must_use]
    #[expect(clippy::too_many_lines, reason = "startup applies one setup to the root, store and peers")]
    pub fn new(setup: Setup) -> World {
        let Setup {
            seed,
            requirement,
            configuration,
            limits,
            initial,
            rows,
            startup,
            patch_sets,
            disk_store,
            mut peers,
            cut,
            clock,
            boot,
            require_assignment,
            walking,
        } = setup;
        let mut configuration = configuration.unwrap_or_else(|| fixture(seed, requirement).0);
        if let Some(peers) = &peers {
            let workers = peers.workers.iter().filter(|worker| worker.channel != 0).count();
            if workers > configuration.core.worker_names.len() {
                configuration.core.worker_names =
                    (0..workers).map(|index| format!("worker-{index}").into_bytes().into_boxed_slice()).collect();
            }
        }
        let rows = rows
            .unwrap_or_else(|| jig_test_profile::fixtures::account_rows(&configuration.core.settings.llm_providers));
        let mut world = Self::empty(configuration, &limits, patch_sets);
        world.seed = seed;
        world.configure_disk(disk_store, patch_sets);
        if let Some((now, wall)) = clock {
            world.set_clock(now, wall);
        }
        if let Some(initial) = initial {
            world.initial_ask = initial;
        }
        world.fixture_rows = rows;
        world.startup = startup;
        if let Some(startup) = startup {
            startup(&mut world.domain);
        }
        world.walking = walking.map(super::Walking::new);
        if let Some(peers) = &mut peers {
            peers.creating = Some(world.creating.clone());
        }
        world.peers = peers;
        world.cut = cut;
        world.send(root::Event::StartBegin { pending: world.store.generation_pending });
        if world.reached_cut.is_some() {
            return world;
        }
        if world.peers.is_some() {
            world.hello_engine();
            world.send(root::Event::Core(core::Event::Account(accounts::Event::Component(
                accounts::FromComponent::Granted {
                    form: Form::Credential(Scheme::Bearer),
                    account: 1,
                    generation: 1,
                    valid: Duration::from_secs(60),
                },
            ))));
        } else if boot == Boot::Caller {
            let projects: Vec<_> = world
                .restart_config
                .0
                .core
                .seed_projects
                .iter()
                .map(|project| (project.number, project.policy.policy.period_spend))
                .collect();
            world.hello_engine();
            for (project, budget) in projects {
                world.send(root::Event::Core(core::Event::Period { project, period: 1, budget, next: None }));
            }
            world.events.extend([
                root::Event::Core(core::Event::Account(accounts::Event::Component(accounts::FromComponent::Granted {
                    form: Form::Credential(Scheme::Bearer),
                    account: 1,
                    generation: 1,
                    valid: Duration::from_secs(60),
                }))),
                root::Event::Core(core::Event::Fleet(fleet::Event::Hello {
                    worker: 0,
                    channel: Token::new(7),
                    hello: Hello {
                        jobs: Box::default(),
                        bounds: jig_test_profile::fixtures::bounds(),
                        stop_bound: Duration::from_millis(100),
                        slots: limits.core.fleet.slots,
                        workstreams: Box::new([]),
                        hosting: Box::new([]),
                    },
                })),
                root::Event::Core(core::Event::SignIn {
                    held: None,
                    reply_to: ReplyTo::new(Token::new(101)),
                    identity: people::Identity {
                        key: people::IdentityKey { provider: 0, subject: 7_u64.to_be_bytes().into() },
                        login: b"person".as_slice().into(),
                        name: b"Person".as_slice().into(),
                    },
                }),
            ]);
            world.drain();
            if requirement {
                world.waiting_judge();
            }
            if require_assignment && world.walking.is_none() {
                assert!(
                    world.assigned.is_some(),
                    "the task hub and fleet assigned the caller: {:?}; replies {:?}",
                    world.trace,
                    world.people_answers
                );
            }
        }
        world
    }
    /// The local scripted host declares the same models as the worker fixtures.
    pub(super) fn hello_engine(&mut self) {
        if self.limits.core.fleet.engine_slots > 0 {
            self.events.push_back(root::Event::Core(core::Event::Fleet(fleet::Event::Hello {
                worker: 0,
                channel: Token::new(0),
                hello: Hello {
                    jobs: Box::default(),
                    bounds: jig_test_profile::fixtures::bounds(),
                    stop_bound: Duration::ZERO,
                    slots: self.limits.core.fleet.engine_slots,
                    workstreams: Box::new([]),
                    hosting: Box::new([]),
                },
            })));
        }
    }

    #[expect(clippy::too_many_lines, reason = "the world initializes each peer and observation in one place")]
    #[expect(clippy::disallowed_methods, reason = "jig's round: reads JIG_WORLD_TRACE to enable world tracing")]
    fn empty(configuration: root::Config, limits: &root::Limits, sets: jig_protocol_store::PatchSets) -> World {
        let observer = crate::conformance::Observer::new(&configuration);
        let heap = crate::conformance::Heap::new();
        let creating = jig_test_profile::fixtures::Creating::new(&configuration.core, limits.core.tasks.tree_tasks);
        let initial_ask = initial_request(&configuration, &creating);
        let protocol = crate::protocol::Stand::with_models(limits.core, &configuration.core.settings.models);
        let store = Store::starting_with_patch_sets(limits, &configuration.core.settings.models, sets);
        let move_provider = configuration.core.settings.llm_providers.providers.first().map(|provider| provider.number);
        World {
            seed: 0,
            manual: false,
            public_referee: crate::referee::public::Referee::default(),
            returned: 0,
            released: Vec::new(),
            serial: 1000,
            walking: None,
            broken: false,
            oracle_closed: false,
            restart_config: (crate::oracle::configuration(&configuration), *limits),
            startup: None,
            fixture_rows: Box::new([]),
            domain: root::Domain::new(configuration, limits),
            store,
            internal_reads: std::collections::BTreeSet::new(),
            systems: [System::new(), System::new()],
            protocol,
            host_calls: Vec::new(),
            host_answers: Vec::new(),
            inbound: Vec::new(),
            answers: Vec::new(),
            settings_answers: Vec::new(),
            person_answers: Vec::new(),
            proposal_reads: Vec::new(),
            assigned: None,
            last_read: None,
            received_grant: None,
            grant_follows_assignment: false,
            renew_on_running: None,
            fail_account_before_workspace: false,
            hold_accounts: false,
            wait_for_finish: false,
            finish_script: VecDeque::new(),
            finishes: Vec::new(),
            fail_llm_key_keep: None,
            fail_move_open: false,
            moved: std::collections::BTreeMap::new(),
            account_generations: std::collections::BTreeMap::new(),
            kept_llm_keys: std::collections::BTreeSet::new(),
            move_provider,
            assignments: Vec::new(),
            jobs: Vec::new(),
            assignment_transcripts: Vec::new(),
            cancellations: Vec::new(),
            busy_turns: Vec::new(),
            busy_calls: Vec::new(),
            busy_events: Vec::new(),
            stop_deliveries: Vec::new(),
            commits: Vec::new(),
            list_reads: Vec::new(),
            projections: Vec::new(),
            hold_projection_settlements: false,
            sign_in: None,
            initial_ask,
            creating,
            people_answers: Vec::new(),
            project_made: Vec::new(),
            act_answers: Vec::new(),
            project_releases: Vec::new(),
            hold_project_releases: Vec::new(),
            application_answers: Vec::new(),
            carried_unused: Vec::new(),
            procedure: None,
            procedure_inputs: Vec::new(),
            hold_procedures: false,
            pending_reads: Vec::new(),
            hold_lost_reads: false,
            pending_lost_reads: Vec::new(),
            lost_answers: 0,
            hold_writes: false,
            recovery: connector::Recovery::Keyed,
            hold_lookups: false,
            pending_lookups: Vec::new(),
            pending_writes: Vec::new(),
            lose_answer: false,
            trace: Vec::new(),
            tracing: std::env::var_os("JIG_WORLD_TRACE").is_some(),
            restart_steps: Vec::new(),
            restore_ranges: Vec::new(),
            census_ended: 0,
            hold_restart_reads: false,
            hold_restore_pages: false,
            hold_switch: false,
            store_delay: 0,
            fill: 0,
            stopped: false,
            period_started: false,
            peers: None,
            client_peer: None,
            store_peer: None,
            watches: None,
            offers: None,
            reads: None,
            observer,
            heap,
            restoring: None,
            restart_reads: std::collections::BTreeMap::new(),
            events: VecDeque::new(),
            limits: *limits,
            cut: None,
            reached_cut: None,
            hold_kinds: Vec::new(),
            wall: 0,
            wall_lag: 0,
            supplied_wall: None,
        }
    }
}
