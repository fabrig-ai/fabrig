use crate::{limits, settings};
use jig_domain_link::{Bounds, EndpointBound};
use skein_lib::{Secret, Time, Token};
use temper_protocol_records_forge as forge_records;

#[path = "../../../../testing/engine_limit_figures.rs"]
mod engine_figures;
use engine_figures::over as jig_limits_over;

// An inline project policy is refused with its enclosing moved member.
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

pub fn fill_files(settings: &mut settings::Engine) {
    settings.files.entries = skein_lib::List::with_capacity(2);
}

pub fn assignment_with(repository: temper_engine_domain::ForgeRepository) -> temper_engine_domain::Assignment {
    let settings = engine();
    assignment_with_settings(repository, &settings)
}

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

#[expect(clippy::disallowed_macros, reason = "the fixture renders its project name")]
fn project_name(number: u32) -> Box<str> {
    alloc::format!("Project {number}").into_boxed_str()
}

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

pub fn stock_document() -> Start {
    super::stock::generated()
}
pub fn stock_files() -> settings::Files {
    super::stock::files()
}

#[expect(clippy::disallowed_types, reason = "ordinary fixture code renders the shipped agent profile")]
pub fn stock_configuration(payload: u32, output: u32) -> Result<smith_agent_shell::config::Configuration, String> {
    let mut stock = temper_agent_fixtures::stock(true);
    stock["profile"]["declared"]["tool_payload"] = payload.into();
    for endpoint in stock["endpoints"].as_array_mut().expect("stock endpoints") {
        endpoint["limits"]["output"] = output.into();
    }
    let bytes = serde_json::to_string_pretty(&stock)
        .expect("stock agent JSON")
        .replace("$llm", "127.0.0.1:8370")
        .replace("\"$programs\"", r#"{"claude_code":{"path":"/fixture/claude","directory":"/fixture/programs"}}"#)
        .replace("api.anthropic.com:443", "127.0.0.1:8373")
        .replace("\"server_name\": \"api.anthropic.com\",", "")
        .replace("api.openai.com:443", "127.0.0.1:8372")
        .replace("\"transport\": \"tls\"", "\"transport\": \"plaintext\"")
        .replace("\"server_name\": \"api.openai.com\",", "");
    smith_agent_shell::config::parse(bytes.as_bytes())
}

#[expect(clippy::disallowed_methods, reason = "ordinary fixture code traverses parsed model declarations")]
pub fn parsed_agent(config: smith_agent_shell::config::Configuration) -> settings::Agent {
    use std::os::unix::ffi::OsStrExt;
    let limits = config.service.limits;
    let host_limits =
        smith_local_service::profile::host_limits(&config.profile, &limits, &config.service.domain.limits).unwrap();
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

pub fn worker_digests(_settings: &settings::Engine) -> settings::Files {
    settings::Files { entries: skein_lib::List::with_capacity(0) }
}

pub use link_setup::{
    DIALECT, ENGINE, assignment_with_settings, encoding, forges, parse_engine, parse_engine_over, repository,
    small_model, wire_limits, worker_runtime,
};
pub(crate) use temper_service_link_fixtures::link_setup;

pub fn engine() -> settings::Engine {
    static ENGINE_SETTINGS: std::sync::OnceLock<jig_protocol_config::Engine> = std::sync::OnceLock::new();
    link_setup::engine_with(ENGINE_SETTINGS.get_or_init(link_setup::engine_jig).clone())
}

pub(crate) use temper_service_link_fixtures::link_peer;
pub(crate) use temper_service_link_fixtures::link_profiles;
mod worker;
use temper_service_link_fixtures::worker_git;
pub mod worker_peers;
pub use worker::{worker_agent, worker_files, worker_limits, worker_settings};
pub use worker_git::started_worker;
pub mod http;
pub mod system;
pub use system::{Kernel, hello, one_process, until_hosting};
mod startup;
pub use startup::*;
pub use temper_worker_service_fixtures::agent;
pub use worker::{
    AGENT_JSON, NOW, PINNED_AGENT_JSON, PINNED_TEMPER, assignment as worker_assignment, chat_assignment,
    one_repository, pinned_agent, spawn, worker,
};

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

pub fn commit(store: &mut crate::store::Store, number: u64, writes: Box<[temper_engine_domain::Write]>) {
    // Appending a fixture row must retain the real first-start counters, as well
    // as its settings rows. Empty stores still receive the minimal test header.
    let owner = Token::new(993);
    store.ask(jig_protocol_store::Request::Get {
        owner,
        key: jig_protocol_store::Get::Jig(jig_domain_core::Key::Core(jig_domain_core::CoreKey::Deployment)),
    });
    let mut header = None;
    for _ in 0..store.events().len() {
        let event = store.events().pop().expect("store event");
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
            store.events().push(event);
        }
    }
    let number = match header {
        Some(header) => number.max(header.commits.checked_add(1).unwrap()),
        None => number,
    };
    let mut header = header.unwrap_or(jig_domain_core::Deployment {
        id: [19; 16],
        restart_wait_ms: None,
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

pub fn mark_store(store: &mut crate::store::Store) {
    if store.mark_durable() {
        return;
    }
    commit(store, 1, Box::new([temper_engine_domain::Write::Wires]));
    let Some(jig_protocol_store::Event::Committed { .. }) = store.events().pop() else { panic!("fixture start mark") };
}

fn open_seeded(store: &mut crate::store::Store) {
    store.ask(jig_protocol_store::Request::Open {
        purpose: jig_protocol_store::Purpose::Serve,
        owner: OWNER,
        root: jig_protocol_store::Root { token: OWNER, fd: skein_io::kernel::Fd::new(1) },
        installation: seeds().installation,
    });
    let Some(jig_protocol_store::Event::Opened { .. }) = store.events().pop() else {
        panic!("seeded fixture reopened")
    };
    // This fixture handoff does not consume the census (jig jig/docs/design/protocol/store/growth.md, 1.3).
    let Some(jig_protocol_store::Event::Census { .. }) = store.events().pop() else { panic!("opened census") };
}

fn exchange_secrets(store: &mut crate::store::Store, secrets: &mut skein_kv::Store, seed: &ForgeSeed) {
    let component = store.exchange_component(seed.empty());
    let mut maps = component.into_stores();
    core::mem::swap(&mut maps.secrets, secrets);
    drop(store.exchange_component(seed.component(maps)));
    open_seeded(store);
}

pub(super) fn start_with_worker(
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
        settings.setups = Box::new([(
            1,
            temper_forge_setup::setup_file::SetupFile::Owned(temper_forge_setup::machine::OwnedMachine {
                sign_in: None,
                listen: owned.listen,
                remote: b"http://127.0.0.1:9".as_slice().into(),
                person: owned.person,
                public: owned.public,
                fetch: owned.fetch,
                forgejo: owned.forgejo,
                runner: owned.runner,
            }),
        )]);
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
    settings.jig.limits.core.fleet.slots = 4;
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
        settings.jig.limits.hub = worker_settings().jig.limits.expect("checked worker limits").hub;
        settings.limits.checkout = worker_limits().checkout;
    }
    let root = limits::engine(&settings.jig.limits, &settings.limits).unwrap();
    let seed = ForgeSeed::new(&root, &settings.jig);
    let store = match stored {
        Some(store) => store,
        None => memory_store_for(&root, &settings.jig.limits.store, &seed.models),
    };
    let worker_io =
        if worker.is_some() { Some(worker_settings().jig.limits.expect("checked worker limits").io) } else { None };
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
        None,
    )
    .unwrap();

    service.engine_component().replace_store(store);
    for source in sources {
        let mut record = forge_record(source.number);
        if under_box {
            record.mode = temper_engine_domain_forge::Mode::Owned;
            record.asked = true;
        }
        record.place.api.host = b"forge.example".as_slice().into();
        record.place.api.port = if source.number == 1 { 3000 } else { 3001 };
        if let Some(skein_io::Addr::Ip(address)) = service.engine.as_ref().unwrap().forge_endpoint(source.number) {
            service.engine_component().resolve_name(record.place.api.host.clone(), address.ip());
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

pub fn started() -> crate::Service {
    started_with_projects(&[(1, 100)])
}

pub fn started_with_projects(projects: &[(u32, u64)]) -> crate::Service {
    start_with_store(projects, None)
}

pub fn start_with_store(projects: &[(u32, u64)], stored: Option<crate::store::Store>) -> crate::Service {
    start_with_worker(projects, stored, None, false, false)
}

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

#[derive(Debug)]
pub struct ConfiguredEngine {
    pub engine: settings::Engine,
    pub forges: Box<[settings::Forge]>,
    pub identities: Box<[settings::Identity]>,
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

pub fn values_with_identity(number: u32, token: &[u8]) -> crate::values::Values {
    let mut values = crate::values::Values::new(2, 4096);
    values.identities = crate::values::Identities::with_capacity(1);
    let mut identity = configured_engine().identities.into_iter().next().unwrap();
    identity.token = Secret::new(token.into());
    let value = crate::translate::identity(&identity, &wire_limits()).unwrap();
    values.identities.put(number, value);
    values
}

pub fn memory_store_for(
    root: &temper_engine_domain::Limits,
    profile: &jig_protocol_store::Limits,
    models: &[jig_domain_core::Model],
) -> crate::store::Store {
    use jig_protocol_store::host::{Hosted, NotReady};
    let limits = crate::store::limits(root, &root.forge, profile).unwrap();
    let application = temper_protocol_store::application(&root.forge, root.tasks.spec_bytes).unwrap();
    let mut host = Hosted::new(&limits, application, models, 7, NotReady::InPass);
    host.component = jig_protocol_store::Store::over(
        &limits,
        application,
        models,
        jig_protocol_store::Stores {
            store: skein_kv::Store::memory(&limits.store),
            secrets: skein_kv::Store::memory(&limits.secrets),
        },
    );
    assert!(host.component.wires(&crate::store::WIRES));
    host.ask(jig_protocol_store::Request::Open {
        purpose: jig_protocol_store::Purpose::Serve,
        owner: OWNER,
        root: jig_protocol_store::Root { token: OWNER, fd: skein_io::kernel::Fd::new(1) },
        installation: seeds().installation,
    });
    let Some(jig_protocol_store::Event::Opened { installation, pending, .. }) = host.above.pop() else {
        panic!("store opened")
    };
    // This fixture handoff does not consume the census (jig jig/docs/design/protocol/store/growth.md, 1.3).
    let Some(jig_protocol_store::Event::Census { .. }) = host.above.pop() else { panic!("opened census") };
    crate::store::Store::new(root, &limits, models, &seeds()).on_opened(host.component, installation, pending)
}
pub fn keep_forge(service: &mut crate::Service, record: temper_engine_domain_forge::ForgeRecord) {
    service.engine_component().restore_forge(record);
}
fn seed_forge_rows(service: &mut crate::Service, forge: u16, token: &[u8], seed: &ForgeSeed) {
    use temper_protocol_records as records;
    let engine = service.engine_component();
    if !engine.store().opened() {
        return;
    }
    let api = engine.forge_address(forge).expect("fixture forge record");
    let origin = forge_records::origin(skein_forge_connection::settings::Kind::Forgejo, &api).encode();
    let original = if engine.store().mark_durable() {
        None
    } else {
        let component = engine.store().exchange_component(seed.empty());
        let mut maps = component.into_stores();
        open_seeded(engine.store());
        mark_store(engine.store());
        exchange_secrets(engine.store(), &mut maps.secrets, seed);
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
        engine.remember_record(number, value.clone(), temper_limits_fixtures::small().records.value_bytes);
        assert!(engine.store().secrets(Token::new(999), application_request(number, value)).is_none());
        assert!(engine.store().events().pop().is_some());
    }
    if let Some(mut maps) = original {
        exchange_secrets(engine.store(), &mut maps.secrets, seed);
        drop(engine.store().exchange_component(seed.component(maps)));
        open_seeded(engine.store());
    }
}
pub fn ready() -> crate::Service {
    let mut service = started();
    for _ in 0_u16..256 {
        crate::iterate(&mut service, Time::from_nanos(1), skein_lib::Wall::from_nanos(1));
        Kernel::new().answer(&mut service);
        if !service.work_pending(Time::from_nanos(1)) {
            break;
        }
    }
    assert!(service.engine_domain().unwrap().ready());
    assert_eq!(service.end(), None);
    service
}
pub fn unopened_store() -> crate::store::Store {
    let settings = engine();
    let root = limits::engine(&settings.jig.limits, &settings.limits).unwrap();
    let profile = crate::store::limits(&root, &root.forge, &settings.jig.limits.store).unwrap();
    let models = jig_protocol_config::core(&settings.jig, [7; 16], 7).settings.models;
    crate::store::Store::new(&root, &profile, &models, &seeds())
}

/// The prepare job's declared request and result must fit a turn fixture.
pub fn turn_document() -> Start {
    let document = document();
    let mut overrides: serde_json::Value =
        serde_json::from_slice(&value(&document, b"engine.limits.overrides")).unwrap();
    overrides["tasks.spec_bytes"] = 16_384_u32.into();
    overrides["tasks.result_bytes"] = 4_096_u32.into();
    overrides["tasks.message_bytes"] = 4_096_u32.into();
    let document = with(&document, b"engine.limits.overrides", &serde_json::to_vec(&overrides).unwrap());
    let mut overrides: serde_json::Value =
        serde_json::from_slice(&value(&document, b"application.limits.overrides")).unwrap();
    overrides["forge.client.op_bytes"] = 4_096_u32.into();
    let document = with(&document, b"application.limits.overrides", &serde_json::to_vec(&overrides).unwrap());
    let mut overrides: serde_json::Value =
        serde_json::from_slice(&value(&document, b"engine.limits.overrides")).unwrap();
    overrides["hub.event_bytes"] = 4_096_u32.into();
    with(&document, b"engine.limits.overrides", &serde_json::to_vec(&overrides).unwrap())
}

#[path = "../../../../testing/startup_fixture.rs"]
mod typed;
pub use typed::{Input, Start};

pub fn named_with(
    input: &(impl Input + ?Sized),
    directories: &jig_protocol_config::Directories,
    given: &crate::stock::Given,
) -> Result<crate::startup::Named, crate::startup::Problems> {
    input.configured().named_with(directories, given)
}

pub fn document_store_limits(
    input: &(impl Input + ?Sized),
    directories: &jig_protocol_config::Directories,
    agent: Option<&temper_service_core::agent::Declaration>,
) -> Result<crate::startup::StoreLimits, crate::startup::Problems> {
    let input = input.configured();
    crate::startup::store_limits_with_limits(&input.document, directories, agent, &input.jig, &input.own)
}
