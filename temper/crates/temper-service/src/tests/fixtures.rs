use crate::{limits, settings};
use jig_domain_host as host;
use jig_domain_link::{Bounds, EndpointBound, Form, Grant};
use skein_lib::{Secret, Token};
use temper_protocol_records_forge as forge_records;

// smith's protocol/agent.md, section 4: OpenAI's public Responses dialect.
pub const DIALECT: u32 = 3;

// The stock catalogue holds twelve charters.
// The closer procedure and close job make seventeen executor places in both limits.
pub const ENGINE: &[u8] = br#"{
"limits": {"profile": "small", "overrides": {"core.run_bytes":32768,"tasks.charters":13,"core.templates":6,"tasks.result_bytes":32,"tasks.run_inbox":2,"hub.turns":1,"people.amendment_bytes":64,"core.policy_bytes":8192,"authority.grants":16,"authority.executors":21,"tasks.executor_kinds":21,"notes.scopes":32,"tasks.spec_bytes":128,"tasks.message_bytes":512,"tasks.hold_bytes":64,"tasks.authority_bytes":64,"people.words":512,"core.call_answer_bytes":2048,"notes.recalled":1}},
"llm": {
"providers":[{"name":"fake-provider","number":1,"endpoint":"fake","sources":{"keys":true}}],
"endpoints": [{
"name": "fake",
"number": 1,
"dialect": 3,
"provider": "fake-provider",
"address": {"socket": "127.0.0.1:7101"},
"transport": "plaintext",
"headers": [],
"connect_ms": 10000,
"handshake_ms": 10000
}]
},
"hosts": {
"workers": [
{"name": "worker-a", "digest": "secrets/worker-a.digest"},
{"name": "worker-b", "digest": "secrets/worker-b.digest"}
],
"listener": [{"in_process": "workers"}]
},
"clients": {
"heartbeat_ms": 10000,
"link": true
},
"deployment": {"directory": "state", "seed": 7},
"owners": [{"provider": 1, "subject": "owner-a"}],
"providers": [{"number": 1, "kind": "link"}, {"number": 2, "kind": "deployment"}],
"connectors": [1],
"memory_bytes": 1152921504606846976,
"projects": [{"number": 1, "policy": 1}]
}"#;

// Jig's inline input is retained only to exercise policy decoding and typed guards.
pub const INLINE_PROJECTS: &[u8] = br#"[{"number": 1, "roles": [{"connector": 1, "permission": 1, "role": 0}],
"policy": {"templates": [], "escalation_role": 0, "period_spend": 100, "projections": [], "requirements": [],
"ceiling": {"tools": 1, "grants": [],
"delegation": {"kinds": [{"kind": "charter", "number": 1}], "tasks": 8, "depth": 4},
"budget": {"spend": 100, "deadline_unix_seconds": null}, "notes": 0, "note_resources": []},
"roles": [{"number": 0, "period_spend": 100, "requests": 511, "decides": 31,
"authority": {"tools": 1, "grants": [],
"delegation": {"kinds": [{"kind": "charter", "number": 1}], "tasks": 8, "depth": 4},
"budget": {"spend": 100, "deadline_unix_seconds": null}, "notes": 0, "note_resources": []}}]
}}]"#;

pub fn engine() -> settings::Engine {
    static ENGINE_SETTINGS: std::sync::OnceLock<jig_protocol_config::Engine> = std::sync::OnceLock::new();
    let jig = ENGINE_SETTINGS.get_or_init(|| parse_engine(ENGINE)).clone();
    // This typed fixture reaches the core's start; policy 1 includes the train’s merge grants with the standing charter executors.

    settings::Engine {
        host_reply_bytes: None,
        agent_charter_bytes: None,
        machine: machine(),
        directories: jig_protocol_config::Directories::working(),
        jig,
        webhooks: None,
        sign_in: None,
        push: None,
        clients: crate::startup::application::Clients {
            listener: jig_protocol_config::Address::Addr(http::address()),
            bundle: None,
            public: None,
        },
        files: settings::Files { entries: skein_lib::List::with_capacity(0) },
        demo: None,
        welcome: false,
        program_environment: Box::new([]),
        host_part: None,
        ways_stock: crate::stock::ways(),
        kinds_stock: temper_engine_domain::landing_stock(&[], &skein_lib::Map::with_capacity(0), 1, 4),
        landings: skein_lib::Map::with_capacity(0),
        limits: small_limits(),
    }
}

// The small worlds admit the stock's larger goal charter while keeping Small's store page.
fn small_limits() -> limits::Limits {
    let mut own = limits::profile(limits::Profile::Small);
    own.root.journal.run_bytes = 32_768;
    own
}

pub fn small_model() -> jig_domain_core::settings::CatalogueModel {
    jig_domain_core::settings::CatalogueModel {
        first_read: None,
        number: 1,
        model: jig_domain_core::Model {
            provider: 1,
            dialect: DIALECT,
            endpoint: 1,
            program: None,
            name: b"small".as_slice().into(),
            cache_life_ms: 0,
            window: 8192,
            output: 1024,
            input_price: 1,
            cached_price: 1,
            cache_write_price: None,
            output_price: 1,
            price_unit: 1000,
            effort: None,
            efforts: Box::default(),
        },
        shown: None,
        description: None,
        picker: true,
        withdrawn: false,
        source: jig_domain_core::settings::ModelSource::Stock,
        estimated: false,
    }
}

/// A parsed engine passed the stock, before any component is made.
pub fn parse_engine(bytes: &[u8]) -> jig_protocol_config::Engine {
    let document = jig_protocol_config::document(
        bytes,
        &jig_protocol_config::DocumentLimits { bytes: 1_048_576, tokens: 16_384, depth: 32 },
    )
    .unwrap();
    let mut jig = jig_protocol_config::engine(
        &document,
        0,
        &jig_protocol_config::Directories::working(),
        jig_protocol_config::Defaults::Passed,
    )
    .unwrap();
    let own = match jig.llm.endpoints.first() {
        Some(endpoint) => endpoint.name.as_ref() == b"fake",
        None => false,
    };
    let defaults = if own {
        crate::stock::checked_with(Box::new([small_model()]), jig.limits.core.tasks.result_bytes).unwrap()
    } else {
        crate::stock::defaults(&jig.llm, jig.limits.core.tasks.result_bytes)
    };
    jig_protocol_config::pass(&mut jig, defaults).unwrap();
    jig
}
/// Startup guards keep the small policy bound; they never make the core.
pub fn guard_engine() -> settings::Engine {
    let mut settings = engine();
    settings.jig.limits.core.policy_bytes = 8192;
    settings
}

/// The API token held by fixture records (temper/docs/design/protocol/credentials.md, 5).
pub(crate) const FORGE_TOKEN: &[u8] = b"api-token";
const FORGE_GIT_TOKEN: &[u8] = b"forge-token-1234";
const FIRST_FORGE_TOKEN: &[u8] = b"first-forge-token";
const SECOND_FORGE_TOKEN: &[u8] = b"second-forge-token";

pub fn forges() -> Box<[settings::Forge]> {
    Box::new([settings::Forge {
        log_origins: None,
        mode: temper_engine_domain_forge::Mode::Connected,
        number: 1,
        kind: settings::Kind::Forgejo,
        api: jig_protocol_config::Address::Host { name: Box::from(&b"forge.example"[..]), port: 3000 },
        transport: settings::Transport::Plaintext,
        git: Box::from(&b"https://forge.example"[..]),
        git_program: None,
        web: None,
        app_read_only: false,
        user: 42,
        identity: 1,
    }])
}

/// A runtime table carrying the document fixture's values.
pub fn forge_table(forges: &[settings::Forge]) -> crate::forges::Forges {
    let mut table = crate::forges::Forges::with_capacity(u32::try_from(forges.len()).unwrap());
    for forge in forges {
        assert!(table.insert(settings::Forge {
            log_origins: forge.log_origins.clone(),
            mode: forge.mode,
            number: forge.number,
            kind: forge.kind,
            api: forge.api.clone(),
            transport: match &forge.transport {
                settings::Transport::Plaintext => settings::Transport::Plaintext,
                settings::Transport::Tls { server_name, trust } =>
                    settings::Transport::Tls { server_name: server_name.clone(), trust: trust.clone() },
            },
            git: forge.git.clone(),
            git_program: forge.git_program.clone(),
            web: forge.web.clone(),
            app_read_only: forge.app_read_only,
            user: forge.user,
            identity: forge.identity,
        }));
    }
    table
}

pub const OWNER: Token = Token::new(91);
pub fn routes() -> crate::store::route::Routes {
    let (root, _, _) = store_limits();
    let bounds = temper_protocol_store::bounds(&root.forge, root.tasks.spec_bytes).expect("family bounds");
    crate::store::route::Routes::new(2, &root, bounds)
}

#[expect(clippy::too_many_lines, reason = "the fixture supplies every retained task field")]
pub fn task_record(number: u64) -> jig_domain_tasks::TaskRecord {
    use jig_domain_tasks as tasks;
    use skein_lib::Wall;
    tasks::TaskRecord {
        session: None,
        talk: None,
        activation_open: false,
        closer: None,
        offered: jig_domain_tasks::Reasons::ALL,
        reasons: jig_domain_tasks::Reasons::FIRST,
        asked_requester: false,
        settings: tasks::SettingsRevision { task: 1, run: None },
        added: Box::new([]),
        hold_bound_since: None,
        label: 0,
        flags: tasks::Flags::PLAIN,
        last_activity: Wall::EPOCH,
        ended_below: tasks::Endings::ZERO,
        ended_delegates: tasks::Endings::ZERO,
        question_lines: Box::new([]),
        hold: 0,
        placed: None,
        proposal_rows: Box::new([]),
        instructions: None,
        llm: None,
        kept_account: None,
        refused: Box::new([]),
        grace: None,
        created_at: Wall::EPOCH,
        changed: Wall::EPOCH,
        phase_since: Wall::EPOCH,
        run_since: None,
        retry_notice: None,
        ended_at: None,
        revision: 0,
        revisions: tasks::Revisions { hold: 0, spec: 0, person: 0 },
        milestone: 0,
        activity: 0,
        due_reason: tasks::Why::Made,
        narrowing: false,
        llm_changed: false,
        result_position: 0,
        last_reply: 0,
        replies: Box::new([]),
        escalation: tasks::Escalation::Unheld { revision: 0 },
        proposal: None,
        result_proposal: false,
        number,
        project: 1,
        requester: tasks::Party::Deployment { project: 1 },
        run_spent: 0,
        run_reserved: 0,
        root: number,
        depth: 0,
        executor: tasks::Executor::Agent { charter: 1 },
        taken_by: None,
        person_answer: None,
        person_wait: None,
        result_followups: Box::new([]),
        tracked: None,
        recurring: None,
        spec: tasks::Spec {
            job: None,
            asks: false,
            words: Box::new([]),
            parameters: Box::new([]),
            inputs: Box::new([]),
        },
        contract: tasks::Contract::Report { words: 32 },
        authority: tasks::Authority {
            tools: tasks::Tools(0),
            models: tasks::Models::NONE,
            grants: Box::new([]),
            delegation: tasks::Delegation { kinds: Box::new([]), tasks: 1, depth: 0 },
            budget: tasks::Budget { spend: 1, deadline: None },
            notes: tasks::Scopes(0),
            note_resources: Box::new([]),
        },
        numbers: tasks::Numbers { budget: 1, spent: 0, spent_below: 0, reserved: 0 },
        funder: tasks::Funder::Period { project: 1, period: 1 },
        waiting_on: Box::new([]),
        holdings: Box::new([]),
        holds_taken: false,
        hold_wait_since: None,
        room_wait: None,
        dependencies: Box::new([]),
        started_from: Box::new([]),
        delegates: Box::new([]),
        references: Box::new([]),
        questions: Box::new([]),
        subscriptions: Box::new([]),
        wake: tasks::WakePolicy::DEFAULT,
        turn: 0,
        last_message: 0,
        inbox: Box::new([]),
        saved: Box::new([]),
        ever_turned: false,
        made: 1,
        attempt: 0,
        last_answer: None,
        tries: tasks::Tries::NONE,
        invalid_result: None,
        refusals: 0,
        phase: tasks::Phase::Waiting,
    }
}

pub fn store_limits() -> (temper_engine_domain::Limits, jig_domain_core::Limits, jig_protocol_store::Limits) {
    let settings = engine();
    store_limits_for(&settings)
}
fn store_limits_for(
    settings: &settings::Engine,
) -> (temper_engine_domain::Limits, jig_domain_core::Limits, jig_protocol_store::Limits) {
    let root = limits::engine(&settings.jig.limits, &settings.limits).expect("root limits");
    (root, settings.jig.limits.core, settings.jig.limits.store)
}

pub fn memory_store() -> crate::store::Store {
    let settings = engine();
    let (root, _, profile) = store_limits_for(&settings);
    let models = jig_protocol_config::core(&settings.jig, [7; 16], 7).settings.models;
    memory_store_for(&root, &profile, &models)
}
fn completed_maps() -> jig_protocol_store::Stores {
    ready().engine.expect("engine").store.host.component.into_stores()
}
pub fn records_store() -> crate::store::Store {
    records_store_from(completed_maps())
}
pub(crate) fn records_store_from(mut maps: jig_protocol_store::Stores) -> crate::store::Store {
    let settings = engine();
    let (root, _, mut profile) = store_limits_for(&settings);
    profile.store.page.rows = 32;
    let models = jig_protocol_config::core(&settings.jig, [7; 16], 7).settings.models;
    // Secret-record fixtures start from a real completed deployment, as the
    // store now refuses secrets until that start's mark is durable.
    let limits = crate::store::limits(&root, &root.forge, &profile).expect("record fixture limits");
    maps.secrets = skein_kv::Store::memory(&limits.secrets);
    let mut store = memory_store_for(&root, &profile, &models);
    store.host.component = jig_protocol_store::Store::over(
        &limits,
        temper_protocol_store::application(&root.forge, root.tasks.spec_bytes).expect("fixture families"),
        &models,
        maps,
    );
    assert!(store.host.component.wires(&crate::store::WIRES));
    store.host.ask(jig_protocol_store::Request::Open {
        owner: OWNER,
        root: jig_protocol_store::Root { token: OWNER, fd: skein_io::kernel::Fd::new(1) },
        installation: seeds().installation,
    });
    let Some(jig_protocol_store::Event::Opened { .. }) = store.host.above.pop() else {
        panic!("record store reopened")
    };
    mark_store(&mut store);
    commit(
        &mut store,
        1,
        Box::new([temper_engine_domain::Write::Save(temper_engine_domain::Record::Forge {
            row: Box::new(temper_engine_domain_forge::Stored::Forge(forge_record(1))),
        })]),
    );
    assert!(store.host.above.pop().is_some());
    store
}
pub(super) fn memory_store_for(
    root: &temper_engine_domain::Limits,
    profile: &jig_protocol_store::Limits,
    models: &[jig_domain_core::Model],
) -> crate::store::Store {
    let limits = crate::store::limits(root, &root.forge, profile).expect("checked store limits");
    let mut store = crate::store::Store::new(root, &limits, models, seeds().installation);
    store.host.component = jig_protocol_store::Store::over(
        &limits,
        temper_protocol_store::application(&root.forge, root.tasks.spec_bytes).expect("application sizes"),
        models,
        jig_protocol_store::Stores {
            store: skein_kv::Store::memory(&limits.store),
            secrets: skein_kv::Store::memory(&limits.secrets),
        },
    );
    assert!(store.host.component.wires(&crate::store::WIRES));
    store.host.ask(jig_protocol_store::Request::Open {
        owner: OWNER,
        root: jig_protocol_store::Root { token: OWNER, fd: skein_io::kernel::Fd::new(1) },
        installation: seeds().installation,
    });
    let Some(jig_protocol_store::Event::Opened { .. }) = store.host.above.pop() else { panic!("store opened") };
    store.opened = true;
    store
}

pub fn landed_row() -> temper_engine_domain_forge::Stored {
    temper_engine_domain_forge::Stored::Landed { commit: [9; 32], task: 3 }
}

pub fn commit(store: &mut crate::store::Store, number: u64, writes: Box<[temper_engine_domain::Write]>) {
    // Appending a fixture row must retain the real first-start counters, as well
    // as its settings rows. Empty stores still receive the minimal test header.
    let owner = Token::new(993);
    store.host.ask(jig_protocol_store::Request::Get {
        owner,
        key: jig_protocol_store::Get::Jig(jig_domain_core::Key::Core(jig_domain_core::CoreKey::Deployment)),
    });
    let mut header = None;
    for _ in 0..store.host.above.len() {
        let event = store.host.above.pop().expect("store event");
        if let jig_protocol_store::Event::Got { owner: found, .. } = &event
            && *found == owner
        {
            let jig_protocol_store::Event::Got { row, .. } = event else { unreachable!("matched get") };
            if let Some(jig_protocol_store::Row::Jig(jig_domain_core::Record::Core(
                jig_domain_core::CoreRecord::Deployment(found),
            ))) = row
            {
                header = Some(found);
            }
        } else {
            store.host.above.push(event);
        }
    }
    let number = match header {
        Some(header) => number.max(header.commits.checked_add(1).unwrap()),
        None => number,
    };
    let mut header = header.unwrap_or(jig_domain_core::Deployment {
        id: [19; 16],
        projects: 0,
        tasks: 0,
        people: 0,
        sign_ins: 0,
        messages: 0,
        runs: 0,
        calls: 0,
        connector_rows: 0,
        applications: 0,
        commits: number,
    });
    header.commits = number;
    let mut all = skein_lib::List::with_capacity(u32::try_from(writes.len().checked_add(1).unwrap()).unwrap());
    all.push(temper_engine_domain::Write::Save(temper_engine_domain::Record::Core(jig_domain_core::Record::Core(
        jig_domain_core::CoreRecord::Deployment(header),
    ))))
    .unwrap();
    for write in writes {
        all.push(write).unwrap();
    }
    assert!(store.down(temper_engine_domain::StoreRequest::Commit { number, writes: all.into_boxed() }));
}

pub fn load(
    store: &mut crate::store::Store,
    range: temper_engine_domain::Range,
) -> Box<[temper_engine_domain::Record]> {
    let mut collected = skein_lib::Queue::uncapped();
    let mut after = None;
    // Every page advances through the fixture's finite, unchanged records.
    loop {
        assert!(store.down(temper_engine_domain::StoreRequest::Load {
            owner: OWNER,
            range: range.clone(),
            after: after.clone(),
            most: 4,
            bytes: 65_536,
        }));
        for _ in 0_u32..store.host.above.len() {
            crate::store::route::up(&mut store.routes, store.host.above.pop().expect("store event"), &mut store.routed);
        }
        let Some(crate::store::route::Up::Root(event)) = store.routed.pop() else { panic!("loaded root event") };
        let temper_engine_domain::Event::Store(temper_engine_domain::StoreInput::Loaded { rows, next, .. }) = *event
        else {
            panic!("loaded page")
        };
        assert!(store.routed.is_empty());
        for row in rows {
            collected.push(row);
        }
        if next.is_none() {
            break;
        }
        assert_ne!(next, after, "the page cursor advances");
        after = next;
    }
    let mut rows = skein_lib::List::with_capacity(collected.len());
    while let Some(row) = collected.pop() {
        rows.push(row).unwrap();
    }
    rows.into_boxed()
}

pub fn seeds() -> settings::Seeds {
    settings::Seeds {
        deployment: [7; 16],
        installation: [9; 16],
        seed: 7,
        crypto: Secret::new(Box::new([7; 32])),
        link_secret: None,
        local_secret: None,
    }
}

/// The worker digests the engine document names.
pub fn fill_files(settings: &mut settings::Engine) {
    settings.files.entries = skein_lib::List::with_capacity(2);
    for worker in &settings.jig.hosts.workers {
        settings.files.entries.push((worker.digest.clone(), Secret::new(WORKER_DIGEST.into()))).unwrap();
    }
}
pub fn started_with_two_forges() -> crate::Service {
    start_with_worker(&[(1, 100)], None, None, true, false)
}
pub fn started() -> crate::Service {
    started_with_projects(&[(1, 100)])
}
pub fn started_with_approval(approval: bool) -> crate::Service {
    let mut settings = engine();
    let clients = if approval {
        br#"{"heartbeat_ms":10000,"link":true,"approval":true}"#.as_slice()
    } else {
        br#"{"heartbeat_ms":10000,"link":true,"approval":false}"#.as_slice()
    };
    settings.jig.clients = parse_engine(&with(ENGINE, b"clients", clients)).clients;
    start_configured_worker(settings, &[(1, 100)], None, None, false, false)
}
pub fn started_with_projects(projects: &[(u32, u64)]) -> crate::Service {
    start_with_store(projects, None)
}
pub fn start_with_store(projects: &[(u32, u64)], stored: Option<crate::store::Store>) -> crate::Service {
    start_with_worker(projects, stored, None, false, false)
}
fn start_with_worker(
    projects: &[(u32, u64)],
    stored: Option<crate::store::Store>,
    worker: Option<crate::Worker>,
    two_forges: bool,
    under_box: bool,
) -> crate::Service {
    start_configured_worker(engine(), projects, stored, worker, two_forges, under_box)
}
#[expect(clippy::too_many_lines, reason = "the fixture joins the configured worker, store and kept forge records")]
fn start_configured_worker(
    mut settings: settings::Engine,
    projects: &[(u32, u64)],
    stored: Option<crate::store::Store>,
    worker: Option<crate::Worker>,
    two_forges: bool,
    under_box: bool,
) -> crate::Service {
    let mut runtime = forge_runtime();
    let mut sources = forges();
    if two_forges {
        let mut second = forges().into_iter().next().unwrap();
        second.number = 2;
        if let jig_protocol_config::Address::Host { port, .. } = &mut second.api {
            *port = 3001;
        }
        let first = forges().into_iter().next().unwrap();
        sources = Box::new([first, second]);
        let first = runtime.into_boxed().into_iter().next().unwrap();
        let second = crate::ForgeRuntime {
            log_origins: skein_lib::List::with_capacity(0),
            number: 2,
            address: ([127, 0, 0, 1], 3001).into(),
            transport: skein_forge_connection::settings::Transport::Plaintext { name: None },
        };
        runtime = skein_lib::List::with_capacity(2);
        runtime.push(first).unwrap();
        runtime.push(second).unwrap();
    }
    if under_box && let temper_forge_setup::setup::Mode::Owned(owned) = owned_setup(1).entry.mode {
        settings.machine.owned = Some(temper_forge_setup::machine::OwnedMachine {
            sign_in: None,
            listen: owned.listen,
            remote: b"http://127.0.0.1:9".as_slice().into(),
            person: owned.person,
            public: owned.public,
            fetch: owned.fetch,
            forgejo: owned.forgejo,
            runner: owned.runner,
        });
    }
    settings.jig.limits.core.people.operations = 2;
    settings.jig.limits.core.tasks.tasks = 2;
    settings.jig.limits.core.tasks.exemptions = 2;
    settings.jig.limits.core.tasks.project_tasks = 2;
    settings.jig.limits.core.tasks.tree_tasks = 2;
    settings.jig.limits.core.tasks.batch = 1;
    settings.jig.limits.core.tasks.holdings = 2;
    settings.jig.limits.core.tasks.run_inbox = 2;
    settings.jig.limits.core.people.people = 2;
    settings.jig.limits.core.people.sign_ins = 2;
    settings.jig.limits.core.people.pending = 2;
    settings.jig.limits.core.people.waiters = 2;
    settings.jig.limits.core.people.inbox_tasks = 4;
    settings.jig.limits.core.fleet.workers = 2;
    settings.jig.limits.core.fleet.attempts = 2;
    settings.jig.limits.core.fleet.calls = 2;
    settings.jig.limits.core.fleet.slots = 1;
    settings.limits.root.journal.deliveries = 128;
    settings.limits.root.journal.held = 384;
    settings.limits.root.journal.writes = 8192;
    settings.jig.limits.clients.streams = 2;
    settings.jig.limits.clients.watches_per_stream = temper_engine_domain::WATCHES_PER_STREAM;
    settings.jig.limits.clients.exchanges = 4;
    settings.jig.limits.clients.exchanges_per_client = 2;
    let first = settings.jig.projects.into_iter().next().unwrap();
    let mut configured = skein_lib::List::with_capacity(u32::try_from(projects.len()).unwrap());
    for (number, budget) in projects {
        let mut project = jig_protocol_config::Project {
            name: project_name(*number),
            number: *number,
            roles: first.roles.clone(),
            templates: first.templates.clone(),
            policy: first.policy.clone(),
        };
        project.policy.period_spend = *budget;
        for role in &mut project.policy.roles {
            role.period_spend = *budget;
        }
        configured.push(project).unwrap();
    }
    settings.jig.projects = configured.into_boxed();
    fill_files(&mut settings);
    if worker.is_some() {
        settings.jig.limits.hub = worker_settings().jig.limits.hub;
        settings.limits.checkout = worker_limits().checkout;
        settings.files.entries.get_mut(0).unwrap().1 =
            Secret::new(b"8f7be71db3941de11249a5e3fba7750a41c7d9958083ef522cb79e9fd7822878".as_slice().into());
    }
    let root = limits::engine(&settings.jig.limits, &settings.limits).unwrap();
    let seed = ForgeSeed::new(&root, &settings.jig);
    let store = match stored {
        Some(store) => store,
        None => memory_store_for(&root, &settings.jig.limits.store, &seed.models),
    };
    let worker_io = if worker.is_some() { Some(worker_settings().jig.limits.io) } else { None };
    let io = crate::startup::checks::process_io(&settings.jig.limits.io, worker_io.as_ref()).unwrap();
    let endpoints = crate::forge::settings(
        &sources,
        runtime,
        settings.webhooks.clone(),
        &mut jig_protocol_config::Problems::empty(),
    )
    .unwrap();
    let mut service = crate::Service::prepared(
        settings,
        &seeds(),
        None,
        endpoints,
        worker,
        io,
        if under_box { Some(system::BOX_BYTES) } else { None },
        None,
    )
    .unwrap();

    service.engine.as_mut().expect("engine").store = store;
    for source in sources {
        let mut record = forge_record(source.number);
        if under_box {
            record.mode = temper_engine_domain_forge::Mode::Owned;
            record.asked = true;
        }
        record.place.api.host = b"forge.example".as_slice().into();
        record.place.api.port = if source.number == 1 { 3000 } else { 3001 };
        if let Some(endpoint) = service.engine.as_ref().unwrap().forge.endpoints.get(&source.number)
            && let skein_io::Addr::Ip(address) = endpoint.address
        {
            service.engine.as_mut().unwrap().forge.names.insert(record.place.api.host.clone(), address.ip()).unwrap();
        }
        keep_forge(&mut service, record);
        let token = if two_forges {
            if source.number == 1 { FIRST_FORGE_TOKEN } else { SECOND_FORGE_TOKEN }
        } else {
            FORGE_TOKEN
        };
        seed_forge_rows(&mut service, source.number, token, &seed);
    }
    service
}
mod agent;
pub mod bundle;
mod checks;
mod clients;
pub mod http;
mod server;
pub use agent::agent;
pub use server::{answered, header, listening, request, server, server_call, server_head, server_response};
pub(super) mod system;
pub use system::{Kernel, hello, one_process, until_hosting};
pub fn turn(service: &mut crate::Service) {
    crate::iterate(service, skein_lib::Time::from_nanos(1), skein_lib::Wall::from_nanos(1));
    clients::kernel(service);
}
pub fn restart(service: &mut crate::Service) {
    for _ in 0_u32..256 {
        turn(service);
        if !service.work_pending(skein_lib::Time::from_nanos(1)) {
            break;
        }
    }
    assert!(service.engine.as_ref().expect("engine").domain.ready(), "restart ready");
    assert_eq!(service.end(), None);
}
pub fn ready() -> crate::Service {
    let mut service = started();
    restart(&mut service);
    service
}
pub fn period(number: u64) -> temper_engine_domain::Event {
    temper_engine_domain::Event::Party(temper_engine_domain::PartyInput::Period {
        project: 1,
        period: number,
        budget: 100,
        next: None,
    })
}
pub fn get(service: &mut crate::Service, key: jig_domain_core::Key) -> Option<jig_domain_core::Record> {
    let engine = service.engine.as_mut().expect("engine");
    let owner = Token::new(991);
    engine.store.host.ask(jig_protocol_store::Request::Get { owner, key: jig_protocol_store::Get::Jig(key) });
    for _ in 0_u32..engine.store.host.above.len() {
        let event = engine.store.host.above.pop().unwrap();
        if let jig_protocol_store::Event::Got { row, .. } = event {
            return match row {
                Some(jig_protocol_store::Row::Jig(record)) => Some(record),
                None => None,
                Some(jig_protocol_store::Row::Application { .. }) => panic!("jig record"),
            };
        }
        engine.store.host.above.push(event);
    }
    panic!("store get answered")
}

pub fn wire_limits() -> temper_protocol_worker_wire::v1::Limits {
    crate::translate::limits(&limits::profile(limits::Profile::Small)).unwrap()
}
pub fn repository(repository: u32, owner: &[u8], name: &[u8]) -> temper_engine_domain::ForgeRepository {
    temper_engine_domain::ForgeRepository {
        delivers: false,
        save: Some(b"temper/7/s7".as_slice().into()),
        tag: temper_engine_domain_forge::tag(1, repository),
        provider: skein_forge::api::Repository { forge: 1, repository },
        name: name.into(),
        host: b"forge.example".as_slice().into(),
        owner: owner.into(),
        start: temper_engine_domain::ForgeStart::Base(b"main".as_slice().into()),
        push: Some(b"temper/task-7".as_slice().into()),
        landing: None,
        identity: 1,
    }
}
pub fn assignment_with(repository: temper_engine_domain::ForgeRepository) -> temper_engine_domain::Assignment {
    let settings = engine();
    assignment_with_settings(repository, &settings)
}
/// A resolution carrying both sides of a large file in its pull section.
pub fn resolution_with_pull(bytes: usize) -> temper_engine_domain::Assignment {
    let mut repository = repository(7, b"acme", b"shop");
    repository.start =
        temper_engine_domain::ForgeStart::Merge { branch: b"temper/task-7".as_slice().into(), base: [7; 32] };
    let mut assignment = assignment_with(repository);
    let mut body = skein_lib::bytes::zeroed(bytes);
    body.fill(b'x');
    assignment.sections = Box::new([temper_engine_domain::BriefSection {
        kind: temper_engine_domain::BriefKind::Forge(temper_engine_domain::ForgeBriefKind::Pull),
        body: temper_engine_domain::BriefBody::Text(body),
    }]);
    assignment
}

pub fn assignment_with_settings(
    repository: temper_engine_domain::ForgeRepository,
    settings: &settings::Engine,
) -> temper_engine_domain::Assignment {
    let task = task_record(7);
    temper_engine_domain::Assignment {
        check_timeout: None,
        charter: 1,
        task: 7,
        attempt: 2,
        conversation: 2,
        live: jig_domain_host::LiveAsk::NONE,
        run: Box::new(temper_engine_domain::RunCharter {
            revision: 1,
            ended_by: jig_domain_tasks::EndedBy::Result,
            sources: Box::new([Some(1)]),
            accounts: Box::new([jig_domain_authority::AccountBinding { provider: 1, account: 1, source: 1 }]),
            placement: jig_domain_fleet::PlacementConstraint {
                channel: Token::new(1),
                kind: jig_domain_fleet::HostKind::Worker,
            },
            policy: settings.jig.charters[0].policy.clone(),
            contract: task.contract,
            authority: task.authority,
            budget: 20,
        }),
        sections: Box::new([]),
        messages: Box::new([]),
        saved: Box::new([]),
        workstream: 7,
        workspace: temper_engine_domain::ForgeWorkspace { repositories: Box::new([repository]) },
        transcript: Box::new([]),
        answered: Box::new([]),
        settled: Box::new([]),
        grants: Box::new([]),
    }
}
pub fn values_with_identity(number: u32, token: &[u8]) -> crate::values::Values {
    let mut values = crate::values::Values::new(2, 4096);
    values.identities = crate::values::Identities::with_capacity(1);
    let mut identity = configured_engine().identities.into_iter().next().unwrap();
    identity.token = Secret::new(token.into());
    let value = crate::translate::identity(&identity, &wire_limits()).unwrap();
    values.identities.put(number, value);
    values
}
pub fn application_grant(account: u32, generation: u64) -> Grant {
    Grant { form: Form::Application, account, generation, valid: skein_lib::Duration::from_secs(30) }
}
pub fn decoded_identity(value: &Secret) -> temper_protocol_worker_wire::v1::Identity {
    temper_protocol_worker_wire::v1::Identity::decode(&wire_limits(), &mut skein_lib::Reader::new(value.expose()))
        .unwrap()
}

pub fn encoding(settings: &settings::Engine) -> crate::assignment::Encoding {
    let mut entries = skein_lib::List::with_capacity(u32::try_from(settings.jig.llm.endpoints.len()).unwrap());
    for endpoint in &settings.jig.llm.endpoints {
        entries
            .push(smith_protocol_channel::Endpoint {
                name: endpoint.name.clone(),
                number: endpoint.number,
                dialect: endpoint.dialect,
            })
            .unwrap();
    }
    crate::assignment::Encoding {
        limits: wire_limits(),
        endpoints: smith_protocol_channel::Endpoints::new(entries),
        deployment: [7; 16],
        charter_bytes: crate::assignment::charter_bound(
            settings.jig.limits.hub.charter_bytes,
            settings.agent_charter_bytes,
        ),
    }
}
pub fn answer(ending: jig_domain_host::Ending) -> jig_domain_host::LinkUp {
    jig_domain_host::LinkUp::Answer {
        run: Token::new(7),
        attempt: Token::new(2),
        answer: jig_domain_host::RunAnswer { read: None, refused: Box::new([]), turns: 0, spent: 0, ending },
    }
}

pub const WORKER_DIGEST: &[u8] = b"0000000000000000000000000000000000000000000000000000000000000000";
/// Inert digest contents for the workers named by these settings.
pub fn worker_digests(settings: &settings::Engine) -> settings::Files {
    let mut entries = skein_lib::List::with_capacity(u32::try_from(settings.jig.hosts.workers.len()).unwrap());
    for worker in &settings.jig.hosts.workers {
        entries.push((worker.digest.clone(), Secret::new(WORKER_DIGEST.into()))).unwrap();
    }
    settings::Files { entries }
}

pub fn workers() -> crate::workers::Workers {
    let settings = engine();
    let encoding = encoding(&settings);
    let limits = crate::workers::limits(&settings).unwrap();
    crate::workers::Workers::new(
        &settings.jig,
        &worker_digests(&settings),
        encoding,
        forges(),
        settings.limits.forge.forges,
        &limits,
    )
    .unwrap()
}

pub fn settle_root(
    service: &mut crate::Service,
    delivered: &mut skein_lib::Queue<temper_engine_domain::Delivery>,
) -> Option<temper_engine_domain::Assignment> {
    use temper_engine_domain as root;
    let engine = service.engine.as_mut().expect("engine");
    let mut assignment = None;
    for _ in 0_u32..256 {
        if let Some(event) = engine.events.pop() {
            engine.step(event);
        }
        root::release(&mut engine.domain, &engine.env, &mut engine.requests);
        if engine.requests.is_empty() {
            engine.step(root::Event::Resume);
            root::release(&mut engine.domain, &engine.env, &mut engine.requests);
        }
        for _ in 0..engine.requests.len() {
            let request = engine.requests.pop().unwrap();
            match request {
                root::Request::Worker(root::WorkerRequest::Deliver(root::Delivery::Assigned {
                    assignment: run,
                    ..
                })) => assignment = Some(run),
                root::Request::Worker(root::WorkerRequest::Deliver(delivery)) => delivered.push(delivery),
                root::Request::Party(root::PartyRequest::Deliver(root::Delivery::WebReply { to, reply }))
                    if to == skein_lib::ReplyTo::new(Token::new(31_052)) =>
                {
                    assert_eq!(
                        reply,
                        jig_domain_people::Reply::Outcome(jig_domain_people::Outcome::SettingsChanged { revision: 2 })
                    );
                }
                root::Request::Party(_) => {}
                request @ root::Request::Worker(_) => crate::route::request(engine, request),
                request @ (root::Request::Store(_) | root::Request::Account(_) | root::Request::ToChild(_)) => {
                    crate::route::request(engine, request);
                }
                root::Request::Setup { .. }
                | root::Request::ForgeKept(_)
                | root::Request::ForgeGone { .. }
                | root::Request::BoxGroup { .. }
                | root::Request::PlaygroundActs { .. }
                | root::Request::ResetAct { .. }
                | root::Request::BoxRemove { .. }
                | root::Request::Connect { .. }
                | root::Request::Replace { .. }
                | root::Request::Manifest { .. }
                | root::Request::AdoptAccess { .. }
                | root::Request::Reachable { .. }
                | root::Request::Install { .. }
                | root::Request::Accept { .. }
                | root::Request::EraseCarried { .. }
                | root::Request::MakingCopied { .. }
                | root::Request::SetUp { .. }
                | root::Request::Bound { .. }
                | root::Request::Forge { .. }
                | root::Request::Busy(_)
                | root::Request::Unsettled(_)
                | root::Request::Drained
                | root::Request::Stop => {
                    panic!("unexpected chat request")
                }
            }
        }
        engine.store.iterate();
        crate::engine::answer(engine);
        if let Some(accounts) = &mut engine.accounts {
            crate::accounts::iterate(
                accounts,
                engine.env.now,
                engine.env.wall,
                &mut engine.values,
                &mut engine.events,
                &mut engine.store,
            );
        }
        engine.domain.reclaim();
        if engine.domain.quiescent() && engine.events.is_empty() && engine.store.idle() {
            return assignment;
        }
    }
    panic!("chat settled");
}
// A restored deployment key, with the first-start people and periods kept by the
// service itself. Ordinary cold-start fixtures stay empty for startup proofs.
fn restored_key() -> crate::Service {
    use jig_domain_accounts::{Holder, Kind, Stored};
    use jig_protocol_store::{SecretOwner, Secrets};
    use temper_engine_domain::{Record, StoreRequest, Write};
    let mut bootstrap = ready();
    let mut header = bootstrap.engine.as_mut().expect("engine").domain.deployment();
    header.commits = header.commits.checked_add(1).unwrap();
    let mut writes = skein_lib::List::with_capacity(4);
    writes
        .push(Write::Save(Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Deployment(header)))))
        .unwrap();
    for row in [
        Stored::Account {
            list: None,
            number: 1,
            provider: 1,
            holder: Holder::Deployment,
            place: 0,
            uses: None,
            subject: Box::new([]),
            display: Box::new([]),
        },
        Stored::Source { account: 1, number: 0, place: 0, kind: Kind::Key, name: Box::new([]), held: None },
        Stored::Next { number: 2 },
    ] {
        writes.push(Write::Save(Record::Core(jig_domain_core::Record::Accounts(row)))).unwrap();
    }
    assert!(
        bootstrap
            .engine
            .as_mut()
            .expect("engine")
            .store
            .down(StoreRequest::Commit { number: header.commits, writes: writes.into_boxed() })
    );
    let _later = bootstrap.engine.as_mut().expect("engine").store.secrets(
        Token::new(crate::tokens::ACCOUNTS),
        Secrets::Replace {
            owner: SecretOwner::Source,
            number: 0,
            value: jig_protocol_store::SecretValue::Key(Secret::new(b"fixture-key".as_slice().into())),
            replaces: None,
        },
    );
    while bootstrap.engine.as_mut().expect("engine").store.host.above.pop().is_some() {}
    let mut service = start_with_store(&[(1, 100)], Some(bootstrap.engine.expect("engine").store));
    restart(&mut service);
    service
}

pub fn signed_worker() -> crate::Service {
    use temper_engine_domain as root;
    let mut service = restored_key();
    service.engine.as_mut().expect("engine").step(root::Event::Party(root::PartyInput::SignedIn {
        held: None,
        reply_to: skein_lib::ReplyTo::new(Token::new(100)),
        identity: jig_domain_people::Identity {
            key: jig_domain_people::IdentityKey { provider: 1, subject: b"owner-a".as_slice().into() },
            login: b"owner".as_slice().into(),
            name: b"Owner".as_slice().into(),
        },
    }));
    settle_root(&mut service, &mut skein_lib::Queue::with_capacity(32));
    // These stories exercise running work with the old fixture's run ceiling.
    // Keep stock at startup, then change the setting through the service's party path.
    let settings = engine();
    let mut rules = settings.jig.rules;
    rules.maximum_run_spend = 100;
    service.engine.as_mut().expect("engine").step(root::Event::Party(root::PartyInput::ChangeSettings {
        kinds: Box::default(),
        reply_to: skein_lib::ReplyTo::new(Token::new(31_052)),
        sign_in: 1,
        key: [52; 16],
        digest: [52; 32],
        change: Box::new(jig_domain_core::settings::Change {
            against: 1,
            items: Box::new([jig_domain_core::settings::Item {
                setting: jig_domain_core::settings::Setting::Rules,
                edit: jig_domain_core::settings::Edit::Set(jig_domain_core::settings::Value::Rules(Box::new(rules))),
            }]),
            running: jig_domain_core::settings::Running::Stays,
        }),
    }));
    settle_root(&mut service, &mut skein_lib::Queue::with_capacity(32));
    service.engine.as_mut().expect("engine").step(root::Event::Worker(root::WorkerInput::Link {
        channel: Token::new(7),
        link: root::Link::Hello {
            worker: 0,
            hello: jig_domain_link::Hello {
                jobs: Box::default(),
                slots: 1,
                stop_bound: skein_lib::Duration::from_millis(100),
                workstreams: Box::new([]),
                hosting: Box::new([]),
                bounds: Bounds {
                    program_endpoints: Box::new([]),
                    endpoints: Box::new([EndpointBound { endpoint: 1, window: 8192, output: 1024 }]),
                    turns: 800,
                    spend: 1000,
                    time: skein_lib::Duration::from_secs(28_800),
                    waiting: skein_lib::Duration::from_secs(60),
                    checks: skein_lib::Duration::from_secs(1),
                    message_bytes: 512,
                    inbox: 8,
                    unacknowledged: 32,
                    reply_bytes: None,
                },
            },
        },
    }));
    settle_root(&mut service, &mut skein_lib::Queue::with_capacity(32));
    service.engine.as_mut().expect("engine").step(root::Event::Worker(root::WorkerInput::Link {
        channel: Token::new(7),
        link: root::Link::Up { up: host::LinkUp::Logins { logins: Box::new([]) }, parts: root::Parts::none() },
    }));
    settle_root(&mut service, &mut skein_lib::Queue::with_capacity(32));
    service
}
pub fn claimed_chat() -> (crate::Service, temper_engine_domain::Assignment) {
    use temper_engine_domain as root;
    let mut service = signed_worker();
    service.engine.as_mut().expect("engine").step(root::Event::Party(root::PartyInput::Ask {
        models: Box::new([]),
        reply_to: skein_lib::ReplyTo::new(Token::new(101)),
        sign_in: 1,
        key: [31; 16],
        ask: chat(1, b"question"),
    }));
    let run = settle_root(&mut service, &mut skein_lib::Queue::with_capacity(32)).expect("chat assigned");
    (service, run)
}

mod worker;
mod worker_git;
pub(crate) mod worker_peers;
pub use worker::{
    AGENT_JSON, NOW, TEMPER, application_value, assignment as worker_assignment, chat_assignment, identity,
    one_repository, repository_assignment, spawn, worker, worker_agent, worker_files, worker_limits, worker_settings,
};
pub use worker_git::{
    IDENTITY, WorkerStand, git_place, github_assignment, github_push, github_record, github_value, started_worker,
};
pub fn clients() -> crate::clients::Clients {
    let engine = engine();
    crate::clients::Clients::new(
        &jig_protocol_config::Directories::working(),
        &engine.jig,
        &engine.clients,
        &seeds(),
        &engine.limits.root.journal,
        None,
        None,
        &[],
    )
    .unwrap()
}
mod startup;
pub use startup::{
    APPLICATION, Json, demo_application, demo_service, document, engine_alone_document, files, owned_worker_agent,
    owned_worker_alone_document, parsed, start, startup_agent, value, webhook_document, with, without,
    worker_alone_document, worker_runtime,
};
pub fn forge_runtime() -> skein_lib::List<crate::ForgeRuntime> {
    let mut runtime = skein_lib::List::with_capacity(1);
    runtime
        .push(crate::ForgeRuntime {
            log_origins: skein_lib::List::with_capacity(0),
            number: 1,
            address: core::net::SocketAddr::from(([127, 0, 0, 1], 3000)),
            transport: skein_forge_connection::settings::Transport::Plaintext { name: None },
        })
        .unwrap();
    runtime
}
mod forge;
pub use forge::{ForgeFace, hook};

pub fn withdrawn_history(task: u64, proposal: u64) -> jig_domain_core::Record {
    use jig_domain_tasks as tasks;
    jig_domain_core::Record::Tasks(tasks::Stored::History(tasks::History {
        task,
        revision: 3,
        by: tasks::Party::Task(task),
        change: tasks::Change::ProposalWithdrawn,
        reason: Box::new([]),
        at: skein_lib::Wall::EPOCH,
        amendment: None,
        proposal: Some(Box::new(tasks::Proposal {
            number: proposal,
            proposer: task,
            project: 1,
            action: tasks::ProposalAction::Release { task },
            reason: Box::new([]),
            as_holder: false,
            state: tasks::ProposalState::Withdrawn,
        })),
    }))
}

pub fn people_member(words: &[u8], spend: u64) -> jig_domain_people::Member {
    use jig_domain_people as people;
    people::Member {
        symbols: Box::new([]),
        executor: people::TaskExecutor::Agent { charter: 1 },
        words: words.into(),
        parameters: Box::new([]),
        inputs: Box::new([]),
        budget: people::Budget::Named,
        deadline: people::Deadline::Named,
        named: Box::new([]),
        contract: people::Contract::Report { words: 16 },
        authority: people::Authority {
            tools: 0,
            models: None,
            grants: Box::new([]),
            delegation: people::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 },
            spend,
            deadline: None,
            notes: 0,
            note_resources: Box::new([]),
        },
        wake: Some(people::WakePolicy {
            words: people::WakeRule::Immediate,
            notices: people::WakeRule::Never,
            news: people::WakeRule::Never,
            results: people::ResultsWake::Never,
            questions: false,
            answers: false,
            timers: false,
        }),
        dependencies: Box::new([]),
        tracked: None,
        label: 0,
        flags: people::Flags::PLAIN,
        llm: None,
    }
}

pub fn chat(project: u32, words: &[u8]) -> jig_domain_people::Ask {
    use jig_domain_people as people;
    let mut member = people_member(words, 0);
    member.budget = people::Budget::AsAllowed { cap: None };
    member.authority.tools = 1;
    member.authority.notes = 0;
    member.contract = people::Contract::Report { words: 32 };
    member.flags = people::Flags {
        ended_by: people::EndedBy::Result,
        conversation: true,
        listed: false,
        retention: people::Retention::Kept,
    };
    member.label = 1;
    people::Ask::Create { project, batch: Box::new([member]) }
}

// Public setup configuration only; the transport story never runs these binaries.
pub fn owned_setup(forge: u16) -> settings::Setup {
    use temper_forge_setup::setup::{Account, Entry, Fetch, Mode, Owned};
    settings::Setup {
        entry: Entry {
            forge,
            directory: b"/data/forge".as_slice().into(),
            bot: Account { login: b"temper".as_slice().into(), email: b"temper@example.test".as_slice().into() },
            asked: true,
            mode: Mode::Owned(Owned {
                public: None,
                fetch: Fetch { forgejo: false, runner: false, mirror: None },
                forgejo: b"/bin/forgejo".as_slice().into(),
                runner: None,
                listen: ([127, 0, 0, 1], 9).into(),
                person: Account { login: b"owner".as_slice().into(), email: b"owner@example.test".as_slice().into() },
            }),
        },
        handover: None,
    }
}

#[expect(clippy::disallowed_macros, reason = "test fixture supplies unique numbered project names")]
fn project_name(number: u32) -> Box<str> {
    alloc::format!("Project {number}").into_boxed_str()
}

/// Seed low application rows with Keep and carried rows through the store's allocator.
pub fn application_request(number: u64, value: Secret) -> jig_protocol_store::Secrets {
    if number >= temper_protocol_records::CARRIED {
        jig_protocol_store::Secrets::Carry { kind: jig_protocol_store::CarriedKind::Value, value }
    } else {
        jig_protocol_store::Secrets::Keep {
            owner: jig_protocol_store::SecretOwner::Application,
            number,
            value: jig_protocol_store::SecretValue::Application(value),
        }
    }
}

/// Reopen a fixture's durable maps as a fresh process, including a fresh mark gate.
pub fn reopened_store(mut store: crate::store::Store, root: &temper_engine_domain::Limits) -> crate::store::Store {
    let settings = engine();
    let models = jig_protocol_config::core(&settings.jig, [7; 16], 7).settings.models;
    let limits = crate::store::limits(root, &root.forge, &settings.jig.limits.store).expect("fixture store limits");
    let maps = store.host.component.into_stores();
    store.host.component = jig_protocol_store::Store::over(
        &limits,
        temper_protocol_store::application(&root.forge, root.tasks.spec_bytes).expect("fixture families"),
        &models,
        maps,
    );
    assert!(store.host.component.wires(&crate::store::WIRES));
    store.host.ask(jig_protocol_store::Request::Open {
        owner: OWNER,
        root: jig_protocol_store::Root { token: OWNER, fd: skein_io::kernel::Fd::new(1) },
        installation: seeds().installation,
    });
    let Some(jig_protocol_store::Event::Opened { pending: false, .. }) = store.host.above.pop() else {
        panic!("reopened fixture")
    };
    store.secret_requests = 0;
    store.forge_loads = skein_lib::List::with_capacity(128);
    store
}

pub fn mark_store(store: &mut crate::store::Store) {
    if store.host.mark_durable() {
        return;
    }
    commit(store, 1, Box::new([temper_engine_domain::Write::Wires]));
    let Some(jig_protocol_store::Event::Committed { .. }) = store.host.above.pop() else {
        panic!("fixture start mark")
    };
}
pub fn marked_store() -> crate::store::Store {
    let mut store = memory_store();
    mark_store(&mut store);
    store
}

pub fn restarting_from(previous: crate::engine::Engine) -> crate::Service {
    let reopened = reopened_store(previous.store, &previous.env.limits);
    start_with_store(&[(1, 100)], Some(reopened))
}

/// The deployment's machine parts; forge records are kept separately.
pub fn machine() -> temper_forge_setup::machine::Machine {
    temper_forge_setup::machine::Machine { owned: None, fake: None, github: None }
}
/// Admit a runtime forge through the root's kept-record request.
pub fn keep_forge(service: &mut crate::Service, record: temper_engine_domain_forge::ForgeRecord) {
    let engine = service.engine.as_mut().expect("engine");
    restore_forge(engine, record.clone());
    crate::route::request(engine, temper_engine_domain::Request::ForgeKept(record));
}
pub fn restore_forge(engine: &mut crate::engine::Engine, record: temper_engine_domain_forge::ForgeRecord) {
    temper_engine_domain::step(
        &mut engine.domain,
        &engine.env,
        temper_engine_domain::Event::Restart { pending: engine.store.pending_generation },
    );
    temper_engine_domain::step(
        &mut engine.domain,
        &engine.env,
        temper_engine_domain::Event::Forge(Box::new(temper_engine_domain_forge::Event::Restore {
            record: temper_engine_domain_forge::Stored::Forge(record),
        })),
    );
}
/// A connected fixture forge, with no access in the document.
pub fn forge_record(number: u16) -> temper_engine_domain_forge::ForgeRecord {
    use temper_engine_domain_forge as forge;
    forge::ForgeRecord {
        person: None,
        git_program: None,
        number,
        kind: forge::Kind::Forgejo,
        mode: forge::Mode::Connected,
        state: forge::ForgeState::Kept,
        place: forge::Place {
            api: forge::Address {
                host: b"forge.example".as_slice().into(),
                port: if number == 1 { 3000 } else { 3001 },
                tls: None,
            },
            git: b"https://forge.example".as_slice().into(),
            web: None,
        },
        bot: forge::Bot {
            login: b"temper".as_slice().into(),
            name: b"temper".as_slice().into(),
            email: b"temper@forge.example".as_slice().into(),
        },
        read_only: false,
        asked: false,
        first: None,
    }
}

/// Runtime fixture inputs, translated into kept records before the service is driven.
#[derive(Debug)]
pub struct ConfiguredEngine {
    pub engine: settings::Engine,
    pub forges: Box<[settings::Forge]>,
    pub identities: Box<[settings::Identity]>,
    pub setup: Box<[settings::Setup]>,
    /// Forges whose setup host must supply records before connecting.
    pub without_records: Box<[u16]>,
}
impl core::ops::Deref for ConfiguredEngine {
    type Target = settings::Engine;
    fn deref(&self) -> &Self::Target {
        &self.engine
    }
}
impl core::ops::DerefMut for ConfiguredEngine {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.engine
    }
}
pub fn configured_engine() -> ConfiguredEngine {
    ConfiguredEngine {
        engine: engine(),
        forges: forges(),
        setup: Box::new([]),
        without_records: Box::new([]),
        identities: Box::new([settings::Identity {
            number: 1,
            forge: 1,
            token: Secret::new(FORGE_GIT_TOKEN.into()),
            login: b"temper".as_slice().into(),
            author: settings::Author {
                name: b"temper".as_slice().into(),
                email: b"temper@forge.example".as_slice().into(),
            },
        }]),
    }
}
pub fn configured_endpoints(
    config: &ConfiguredEngine,
    runtime: skein_lib::List<crate::ForgeRuntime>,
) -> Result<skein_forge_connection::Settings, jig_protocol_config::Problems> {
    let mut found = jig_protocol_config::Problems::empty();
    crate::forge::settings(&config.forges, runtime, config.webhooks.clone(), &mut found).ok_or(found)
}
/// Make all fixture forges through `ForgeKept`; access lives in its secret rows.
pub fn configured_service(
    config: ConfiguredEngine,
    seeds: &settings::Seeds,
    accounts: Option<crate::AccountsRuntime>,
    runtime: skein_lib::List<crate::ForgeRuntime>,
) -> Result<crate::Service, jig_protocol_config::Problems> {
    configured_service_with_store(config, seeds, accounts, runtime, None)
}
pub(crate) fn configured_service_with_store(
    config: ConfiguredEngine,
    seeds: &settings::Seeds,
    accounts: Option<crate::AccountsRuntime>,
    runtime: skein_lib::List<crate::ForgeRuntime>,
    store: Option<crate::store::Store>,
) -> Result<crate::Service, jig_protocol_config::Problems> {
    let endpoints = configured_endpoints(&config, runtime)?;
    configured_prepared_with_store(config, seeds, accounts, endpoints, None, None, store)
}
pub fn configured_prepared(
    config: ConfiguredEngine,
    seeds: &settings::Seeds,
    accounts: Option<crate::AccountsRuntime>,
    endpoints: skein_forge_connection::Settings,
    worker: Option<crate::Worker>,
    box_bytes: Option<u32>,
) -> Result<crate::Service, jig_protocol_config::Problems> {
    configured_prepared_with_store(config, seeds, accounts, endpoints, worker, box_bytes, None)
}
#[expect(
    clippy::too_many_lines,
    reason = "the fixture keeps forge records and supplies both runtime owners' endpoint facts"
)]
fn configured_prepared_with_store(
    mut config: ConfiguredEngine,
    seeds: &settings::Seeds,
    accounts: Option<crate::AccountsRuntime>,
    endpoints: skein_forge_connection::Settings,
    worker: Option<crate::Worker>,
    box_bytes: Option<u32>,
    store: Option<crate::store::Store>,
) -> Result<crate::Service, jig_protocol_config::Problems> {
    use temper_engine_domain_forge as domain;
    for setup in &config.setup {
        match &setup.entry.mode {
            temper_forge_setup::setup::Mode::Owned(owned) => {
                config.engine.machine.owned = Some(temper_forge_setup::machine::OwnedMachine {
                    sign_in: None,
                    listen: owned.listen,
                    remote: b"http://127.0.0.1:9".as_slice().into(),
                    person: owned.person.clone(),
                    public: owned.public.clone(),
                    fetch: owned.fetch.clone(),
                    forgejo: owned.forgejo.clone(),
                    runner: owned.runner.clone(),
                });
            }
            temper_forge_setup::setup::Mode::Fake(fake) => {
                config.engine.machine.fake = Some(temper_forge_setup::machine::FakeMachine {
                    listen: fake.listen,
                    remote: b"http://127.0.0.1:9".as_slice().into(),
                    git: fake.git.clone(),
                });
            }
            temper_forge_setup::setup::Mode::Managed { .. } => {}
        }
    }
    let io = config.engine.jig.limits.io;
    let root = limits::engine(&config.engine.jig.limits, &config.engine.limits).unwrap();
    let seed = ForgeSeed::new(&root, &config.engine.jig);
    let empty = crate::forge::prepare(&config.engine, skein_lib::List::with_capacity(0))?;
    let mut service = crate::Service::prepared(config.engine, seeds, accounts, empty, worker, io, box_bytes, None)?;
    service.engine.as_mut().unwrap().store = match store {
        Some(store) => store,
        None => records_store(),
    };
    for endpoint in endpoints.forges.into_boxed() {
        let engine = service.engine.as_mut().unwrap();
        if let skein_io::Addr::Ip(address) = endpoint.address {
            for source in &config.forges {
                if source.number == endpoint.number
                    && address != crate::forge::HELD_ADDRESS
                    && let jig_protocol_config::Address::Host { name, .. } = &source.api
                {
                    engine.forge.names.insert(name.clone(), address.ip()).unwrap();
                }
            }
        }
        engine.forge.endpoints.insert(endpoint.number, endpoint).unwrap();
    }
    for source in config.forges {
        let mut record = forge_record(source.number);
        record.kind = match source.kind {
            settings::Kind::Forgejo => domain::Kind::Forgejo,
            settings::Kind::GitHub => domain::Kind::GitHub,
        };
        record.place = domain::Place {
            api: match &source.api {
                jig_protocol_config::Address::Host { name, port } => {
                    domain::Address { host: name.clone(), port: *port, tls: None }
                }
                jig_protocol_config::Address::Addr(skein_io::Addr::Ip(address)) => {
                    domain::Address { host: crate::system::socket_host(*address), port: address.port(), tls: None }
                }
                jig_protocol_config::Address::Addr(skein_io::Addr::InProcess(_)) => panic!("fixture IP"),
            },
            git: source.git.clone(),
            web: source.web.clone(),
        };
        record.place.api.tls = match &source.transport {
            settings::Transport::Plaintext => None,
            settings::Transport::Tls { server_name, .. } => Some(server_name.clone()),
        };
        record.read_only = source.app_read_only;
        for identity in &config.identities {
            if identity.number != source.identity {
                continue;
            }
            record.bot = domain::Bot {
                login: identity.login.clone(),
                name: identity.author.name.clone(),
                email: identity.author.email.clone(),
            };
        }
        if record.kind == domain::Kind::GitHub && record.bot.login.is_empty() {
            record.bot = domain::Bot {
                login: b"x-access-token".as_slice().into(),
                name: b"app-fixture[bot]".as_slice().into(),
                email: b"9+app-fixture[bot]@users.noreply.github.com".as_slice().into(),
            };
        }
        for setup in &config.setup {
            if setup.entry.forge == source.number {
                record.mode = match setup.entry.mode {
                    temper_forge_setup::setup::Mode::Owned(_) => domain::Mode::Owned,
                    temper_forge_setup::setup::Mode::Fake(_) => domain::Mode::Fake,
                    temper_forge_setup::setup::Mode::Managed { .. } => domain::Mode::Managed,
                };
                record.bot.login = setup.entry.bot.login.clone();
                record.bot.email = setup.entry.bot.email.clone();
                record.asked = setup.entry.asked;
            }
        }
        let engine = service.engine.as_mut().unwrap();
        commit(
            &mut engine.store,
            1,
            Box::new([temper_engine_domain::Write::Save(temper_engine_domain::Record::Forge {
                row: Box::new(domain::Stored::Forge(record.clone())),
            })]),
        );
        assert!(engine.store.host.above.pop().is_some());
        keep_forge(&mut service, record);
        service.engine.as_mut().unwrap().setup.fixture_locals(&config.setup);
        if !config.without_records.contains(&source.number) && source.kind == settings::Kind::Forgejo {
            seed_forge_rows(&mut service, source.number, FORGE_TOKEN, &seed);
        }
    }
    Ok(service)
}

/// Store inputs for fixture secrets (temper/docs/design/protocol/credentials.md, 2.1).
#[derive(Debug)]
struct ForgeSeed {
    limits: jig_protocol_store::Limits,
    application: jig_protocol_store::sizes::Application,
    models: Box<[jig_domain_core::Model]>,
}
impl ForgeSeed {
    fn new(root: &temper_engine_domain::Limits, config: &jig_protocol_config::Engine) -> Self {
        Self {
            limits: crate::store::limits(root, &root.forge, &config.limits.store).unwrap(),
            application: temper_protocol_store::application(&root.forge, root.tasks.spec_bytes).unwrap(),
            models: jig_protocol_config::core(config, [7; 16], 7).settings.models,
        }
    }
    fn component(&self, maps: jig_protocol_store::Stores) -> jig_protocol_store::Store {
        jig_protocol_store::Store::over(&self.limits, self.application, &self.models, maps)
    }
    fn empty(&self) -> jig_protocol_store::Store {
        self.component(jig_protocol_store::Stores {
            store: skein_kv::Store::memory(&self.limits.store),
            secrets: skein_kv::Store::memory(&self.limits.secrets),
        })
    }
}

fn seed_forge_rows(service: &mut crate::Service, forge: u16, token: &[u8], seed: &ForgeSeed) {
    use temper_protocol_records as records;
    let engine = service.engine.as_mut().unwrap();
    // Directory-admission fixtures reach the store before any forge can connect.
    if !engine.store.opened {
        return;
    }
    let api = match engine.workers.forges.get(forge) {
        Some(source) => source.api.clone(),
        None => {
            let mut api = None;
            for record in &engine.forge_records.pending {
                if record.number == forge {
                    api = Some(jig_protocol_config::Address::Host {
                        name: record.place.api.host.clone(),
                        port: record.place.api.port,
                    });
                    break;
                }
            }
            api.expect("fixture forge record")
        }
    };
    let origin = forge_records::origin(skein_forge_connection::settings::Kind::Forgejo, &api).encode();
    let secret_requests = engine.store.secret_requests;
    let original = if engine.store.host.mark_durable() {
        None
    } else {
        // Seed secrets through Keep without changing the main store's startup state.
        let component = core::mem::replace(&mut engine.store.host.component, seed.empty());
        let mut maps = component.into_stores();
        open_seeded(&mut engine.store);
        mark_store(&mut engine.store);
        exchange_secrets(&mut engine.store, &mut maps.secrets, seed);
        Some(maps)
    };
    for (item, bytes) in [
        (records::Item::Origin, origin.as_ref()),
        (records::Item::User, 42_u64.to_le_bytes().as_slice()),
        (records::Item::ApiToken, token),
        (records::Item::GitToken, FORGE_GIT_TOKEN),
    ] {
        let number = records::number(forge, item);
        let value = Secret::new(records::encode(item, bytes).unwrap().into_boxed_slice());
        engine.forge_records.rows.push(
            number,
            value.clone(),
            limits::profile(limits::Profile::Small).records.value_bytes,
        );
        assert!(engine.store.secrets(Token::new(999), application_request(number, value)).is_none());
        assert!(engine.store.host.above.pop().is_some());
    }
    if let Some(mut maps) = original {
        exchange_secrets(&mut engine.store, &mut maps.secrets, seed);
        engine.store.host.component = seed.component(maps);
        open_seeded(&mut engine.store);
        engine.store.secret_requests = secret_requests;
    }
}

/// Exchange only secrets, preserving the main store's durable start state.
fn exchange_secrets(store: &mut crate::store::Store, secrets: &mut skein_kv::Store, seed: &ForgeSeed) {
    let component = core::mem::replace(&mut store.host.component, seed.empty());
    let mut maps = component.into_stores();
    core::mem::swap(&mut maps.secrets, secrets);
    store.host.component = seed.component(maps);
    open_seeded(store);
}
fn open_seeded(store: &mut crate::store::Store) {
    assert!(store.host.component.wires(&crate::store::WIRES));
    store.host.ask(jig_protocol_store::Request::Open {
        owner: OWNER,
        root: jig_protocol_store::Root { token: OWNER, fd: skein_io::kernel::Fd::new(1) },
        installation: seeds().installation,
    });
    let Some(jig_protocol_store::Event::Opened { .. }) = store.host.above.pop() else {
        panic!("seeded fixture reopened")
    };
}

/// Replace the fake durable store while retaining the fixture's kept forge records.
pub fn replace_store(service: &mut crate::Service, mut store: crate::store::Store) {
    let engine = service.engine.as_mut().unwrap();
    let rows = load(&mut engine.store, temper_engine_domain::Range::Forge);
    let target_rows = load(&mut store, temper_engine_domain::Range::Forge);
    let mut writes = skein_lib::List::with_capacity(u32::try_from(rows.len()).unwrap());
    for row in rows {
        if let temper_engine_domain::Record::Forge { row } = row
            && let temper_engine_domain_forge::Stored::Forge(record) = *row
        {
            let mut ended = false;
            for target in &target_rows {
                if let temper_engine_domain::Record::Forge { row } = target
                    && let temper_engine_domain_forge::Stored::Forge(target) = row.as_ref()
                    && target.number == record.number
                    && target.state != temper_engine_domain_forge::ForgeState::Kept
                {
                    ended = true;
                }
            }
            if ended {
                continue;
            }
            writes
                .push(temper_engine_domain::Write::Save(temper_engine_domain::Record::Forge {
                    row: Box::new(temper_engine_domain_forge::Stored::Forge(record)),
                }))
                .unwrap();
        }
    }
    if !writes.is_empty() {
        commit(&mut store, 1, writes.into_boxed());
        assert!(store.host.above.pop().is_some());
    }
    engine.store = store;
}

pub fn checked_endpoints(
    config: &ConfiguredEngine,
    runtime: skein_lib::List<crate::ForgeRuntime>,
) -> Result<skein_forge_connection::Settings, jig_protocol_config::Problems> {
    let settings = configured_endpoints(config, runtime)?;
    if let Err(why) = skein_forge_connection::limits::check(
        &config.limits.forge_component,
        &crate::forge::shared(&config.limits, &config.jig.limits.io),
        &settings,
    ) {
        let mut problems = jig_protocol_config::Problems::empty();
        crate::forge::problems(crate::forge::indexed(why, &config.forges), &mut problems);
        return Err(problems);
    }
    Ok(settings)
}

/// Send a carried forge request through the authenticated client bytes.
pub fn forge_connecting(
    service: &mut crate::Service,
    bearer: &[u8],
    key: u8,
    request: temper_client_protocol_wire::v1::Request,
    bundle: Secret,
) {
    use jig_client_protocol_wire::v20 as j;
    use skein_lib::Reader;
    use temper_client_protocol_wire::{self as wire, v1 as w};
    let record = wire::requesting(&w::Requesting::new(&w::CEILINGS, w::RequestingParts { request }).unwrap());
    let engine = service.engine.as_mut().unwrap();
    let limits = &engine.clients.env.limits.wire;
    let family = j::FamilyRequest::new(limits, j::FamilyRequestParts { scope: j::Scope::Deployment, record }).unwrap();
    let mut bytes = skein_lib::Writer::new(usize::try_from(family.measure()).unwrap());
    family.encode(&mut bytes).unwrap();
    let keyed = j::Keyed::new(
        limits,
        j::KeyedParts {
            key: [key; 16],
            owner: j::Owner::Family(j::Family::new(limits, j::FamilyParts { number: wire::FORGE }).unwrap()),
            version: wire::CLIENT_VERSION,
            request: bytes.finish(),
            value: Some(bundle),
        },
    )
    .unwrap();
    let mut bytes = skein_lib::Writer::new(usize::try_from(keyed.measure()).unwrap());
    keyed.encode(&mut bytes).unwrap();
    let mut peer = http::Peer::new(engine, b"POST", b"/jig/v1/request", &bytes.finish(), Some(bearer));
    peer.pump(engine, true);
    let at = skein_lib::bytes::find(peer.received(), b"\r\n\r\n").unwrap().checked_add(4).unwrap();
    let j::KeyedAnswer::FinalAnswer(answer) =
        j::KeyedAnswer::decode(&engine.clients.env.limits.wire, &mut Reader::new(&peer.received()[at..])).unwrap()
    else {
        panic!("family terminal")
    };
    let answer = w::Answer::decode(&w::CEILINGS, &mut Reader::new(answer.record())).unwrap().into_parts().outcome;
    assert_eq!(answer, w::Outcome::Connecting);
}

/// Parse the shipped agent with inert destinations and selected payload/output quantities.
#[expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    reason = "ordinary fixture code embeds the shipped profile and returns the shell's text diagnostic"
)]
pub fn stock_configuration(payload: u32, output: u32) -> Result<smith_agent_shell::config::Configuration, String> {
    let bytes = include_str!("../../../temper-init/template/agent.json")
        .replace("\"tool_payload\": 10662", &format!("\"tool_payload\": {payload}"))
        .replace("\"output\": 8192", &format!("\"output\": {output}"))
        .replace("$llm", "127.0.0.1:8370")
        .replace("\"$programs\"", r#"{"claude_code":{"path":"/fixture/claude","directory":"/fixture/programs"}}"#)
        .replace("api.anthropic.com:443", "127.0.0.1:8373")
        .replace("\"server_name\": \"api.anthropic.com\",", "")
        .replace("api.openai.com:443", "127.0.0.1:8372")
        .replace("\"transport\": \"tls\"", "\"transport\": \"plaintext\"")
        .replace("\"server_name\": \"api.openai.com\",", "");
    smith_agent_shell::config::parse(bytes.as_bytes())
}

/// The startup agent derived from the shell configuration, including its hello.
#[expect(clippy::disallowed_methods, reason = "ordinary fixture code traverses the parsed model declarations")]
pub fn parsed_agent(config: smith_agent_shell::config::Configuration) -> settings::Agent {
    use std::os::unix::ffi::OsStrExt;
    let limits = config.service.limits;
    let host_limits = smith_local_service::profile::host_limits(&config.profile, &limits).unwrap();
    let mut protocol = smith_host_protocol::Limits {
        bodies: limits.channel.bodies,
        charter: limits.channel.charter,
        transcript: limits.channel.transcript,
        endpoints: limits.channel.endpoints,
        channel: limits.channel.channel,
        start_bytes: limits.channel.start_bytes,
        calls: host_limits.calls,
    };
    protocol.channel.output_bytes = smith_host_protocol::output_bytes(&protocol).unwrap();
    settings::Agent {
        run_limits: limits.domain.run,
        tool_payload: config.profile.declared.tool_payload,
        programs: config
            .program
            .into_iter()
            .map(|program| settings::Program {
                adapter: settings::Adapter::ClaudeCode,
                path: program.path.as_os_str().as_bytes().into(),
                directory: program.directory.as_os_str().as_bytes().into(),
            })
            .collect(),
        endpoints: config.service.channel_endpoints,
        limits: protocol,
        host_limits,
        bounds: Bounds {
            endpoints: config
                .declarations
                .endpoints
                .iter()
                .map(|endpoint| EndpointBound {
                    endpoint: endpoint.number,
                    window: endpoint.limits.window,
                    output: endpoint.limits.output,
                })
                .collect(),
            program_endpoints: config.program_endpoints.into_iter().collect(),
            turns: config.profile.policy.max_turns,
            spend: config.profile.policy.max_spend,
            time: config.profile.policy.max_time,
            waiting: config.profile.policy.max_waiting,
            checks: config.profile.policy.check_timeout,
            message_bytes: limits.domain.run.message_bytes.checked_sub(2).unwrap(),
            inbox: config.profile.policy.inbox,
            unacknowledged: config.profile.policy.unacknowledged,
            reply_bytes: None,
        },
    }
}

pub(crate) fn assert_worker_only(service: &crate::worker_alone::Service) {
    let crate::worker_alone::Service { process: _, worker: _, held_listeners: _, listeners_closing: _ } = service;
}
