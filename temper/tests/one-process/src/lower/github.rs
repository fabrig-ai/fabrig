//! Installation scope through the simulated service and authenticated git route (temper/docs/design/domain/testing.md, 2.4–2.5).
use crate::{
    ctl::{Caller, Command},
    fixtures,
    machine::{Machine, SignalHost},
    referee::{self, Store},
    stories,
    world::{self, Proc, Story},
};
use serde_json::{Value, json};
use skein_fake_forge_domain::{self as domain, api};
use skein_fake_forge_github as layer;
use skein_fake_peers::forge::{Forge, Peer};
use skein_io::kernel::Exit;
use skein_lib::{Duration, List, Secret, Time};
use skein_shell::Host;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use temper_github_app::APP;
const ACME: u64 = 10;
const UMBRA: u64 = 20;
const REPOSITORY: u32 = 2;
const GIT_RECORDS: u32 = 128;
#[derive(Clone, Copy, Debug)]
pub enum Scenario {
    Uninstalled,
    ReadOnly,
    Delivered,
}
thread_local! { static ACTIVE: Cell<bool> = const { Cell::new(false) }; }
pub(crate) fn active() -> bool {
    ACTIVE.with(Cell::get)
}
pub(crate) fn configure(value: &mut Value) {
    if !active() {
        return;
    }
    value["application"]["forges"] = json!([{"number":1,"kind":"github","api":{"socket":fixtures::FORGE.to_string()},"transport":"plaintext","git":format!("http://{}",fixtures::FORGE),"web":format!("http://{}",fixtures::FORGE)}]);
    value["application"]["identities"] = json!([{"number":1,"forge":1,"login":null,"author":null}]);
    for project in value["world_settings"]["policies"].as_array_mut().expect("policies") {
        for pointer in ["/policy/ceiling/grants", "/policy/roles/0/authority/grants"] {
            for grant in project.pointer_mut(pointer).expect("grants").as_array_mut().expect("grants") {
                grant["pattern"]["segments"] = json!(["forge"]);
                grant["pattern"]["value"] = fixtures::FORGE.to_string().into();
            }
        }
    }
}
pub(crate) fn protocol() -> skein_fake_forge_protocol::Limits {
    let mut wire = fixtures::forge_limits();
    wire.http.body = 32768;
    wire.documents.json.string = 32768;
    wire.documents.json.length = 32768;
    wire.documents.writer.length = 32768;
    wire.documents.body_bytes = 32768;
    wire
}
fn config_layer() -> layer::Config {
    let mut accounts = List::with_capacity(2);
    let mut repositories = List::with_capacity(2);
    for (login, id, name, repository) in [("acme", ACME, "app", REPOSITORY), ("umbra", UMBRA, "tool", 3)] {
        accounts
            .push(layer::service::Account { login: login.as_bytes().into(), id, organization: true })
            .expect("two accounts");
        repositories
            .push(layer::service::Row {
                id: repository,
                owner: login.as_bytes().into(),
                owner_id: id,
                organization: true,
                name: name.as_bytes().into(),
                private: true,
            })
            .expect("two repositories");
    }
    let mut permissions = List::with_capacity(16);
    for (name, access) in skein_forge::github::setup::PERMISSIONS {
        permissions
            .push(skein_forge::github::document::Permission { name: (*name).into(), access })
            .expect("App permissions");
    }
    let mut events = List::with_capacity(16);
    for event in skein_forge::github::setup::EVENTS {
        events.push((*event).into()).expect("App events");
    }
    layer::Config {
        deliveries: layer::delivery::Config {
            log_min: Duration::from_millis(10),
            log_max: Duration::from_millis(50),
            ..layer::delivery::Config::standard()
        },
        rate_quota: 5000,
        rate_window: Duration::from_secs(3600),
        token_life: layer::installations::TOKEN_LIFE,
        base: b"http://127.0.0.1".as_slice().into(),
        blob: b"http://127.0.0.1".as_slice().into(),
        third_origin: None,
        blob_redirect: None,
        app: layer::app::App {
            id: APP.id,
            slug: APP.slug.as_bytes().into(),
            owner: ACME,
            permissions: permissions.into_boxed(),
            events: events.into_boxed(),
            key: layer::keys::KEYS[APP.key],
            webhook: Secret::new(b"fake-webhook".as_slice().into()),
        },
        app_facts: layer::service::AppFacts::Complete,
        bot: APP.bot,
        shown: layer::service::Shown::Ruleset,
        ruleset_id: 1,
        pull_requests_only: false,
        required_signatures: false,
        hook: b"https://temper-test.invalid/".as_slice().into(),
        repositories,
        accounts,
        person: b"acme".as_slice().into(),
        tokens: List::with_capacity(0),
        faults: layer::Faults { reset: 0, drop: 0, server_error: 0 },
    }
}
pub(crate) fn peer() -> (Rc<RefCell<Forge>>, Peer) {
    let wire = protocol();
    let mut config = temper_fake_git_hosted::fixtures::CONFIG;
    config.limits.files = 4;
    config.limits.content_bytes = 1024;
    config.limits.calls = wire.calls;
    config.limits.title_bytes = 512;
    config.limits.body_bytes = 2048;
    assert!(skein_fake_forge_protocol::git::fits(&wire, &config.limits));
    let mut domain = domain::Domain::new(&config, fixtures::SEED);
    for (id, name) in [(REPOSITORY, b"acme/app".as_slice()), (3, b"umbra/tool".as_slice())] {
        domain::repository(
            &mut domain,
            &config,
            api::Setup {
                id,
                name: name.into(),
                default: b"main".as_slice().into(),
                tree: Box::new([
                    api::File { path: b"README.md".as_slice().into(), content: b"Scope fixture\n".as_slice().into() },
                    api::File { path: b"conflict.txt".as_slice().into(), content: b"base\n".as_slice().into() },
                ]),
                workflows: Box::default(),
                labels: Box::default(),
                protection: None,
                hooked: false,
                checks: api::Checks {
                    cues: Box::default(),
                    contexts: Box::default(),
                    latency_min: Duration::ZERO,
                    latency_max: Duration::ZERO,
                    silent: 0,
                    passes: 1000,
                    reruns: 0,
                    cue: None,
                },
            },
        );
    }
    domain::grant(&mut domain, b"acme/app", APP.bot, api::Permission::Admin);
    let forge = Rc::new(RefCell::new(Forge::new(domain, config)));
    let limits = layer::Limits {
        deliveries: layer::delivery::Limits { entries: 64, payload_bytes: wire.documents.body_bytes },
        http: wire.http,
        github: skein_forge::github::limits::Limits {
            json: wire.documents.json,
            writer: wire.documents.writer,
            string_bytes: wire.documents.string_bytes,
            body_bytes: wire.documents.body_bytes,
            inner: wire.documents.inner,
            target_bytes: wire.documents.target_bytes,
        },
        installations: layer::installations::Limits { installations: 8, tokens: 32, listed: 8 },
        pages: layer::pages::Limits { states: 8, manifest_bytes: 8192 },
        page: 32,
        body: wire.documents.body_bytes,
        response: wire.documents.body_bytes,
        blob_records: 16,
        git_records: GIT_RECORDS,
    };
    let mut hosted = fixtures::peer_limits();
    hosted.connections = wire.calls + 3;
    let mut peer = Peer::github(
        fixtures::FORGE,
        ([127, 0, 0, 1], 3001).into(),
        hosted,
        config_layer(),
        limits,
        forge.clone(),
        fixtures::SEED,
    )
    .expect("GitHub peer");
    peer.github_layer_mut()
        .expect("GitHub")
        .set_setup_url(Some(format!("http://{}/forges/1/github/setup", fixtures::CLIENTS).into_bytes().into()))
        .expect("setup redirect");
    while forge.borrow_mut().domain.pop_observation().is_some() {}
    (forge, peer)
}
fn adopt(owner: &[u8], name: &[u8], key: &[u8]) -> Command {
    Command::new(&[
        b"repository",
        b"adopt",
        b"--project",
        b"1",
        b"--owner",
        owner,
        b"--name",
        name,
        b"--landing",
        b"main",
        b"--key",
        key,
        b"--json",
    ])
}
fn commands(caller: &mut Caller, link: Box<[u8]>, scenario: Scenario) -> usize {
    let mut sign_in = Command::new(&[b"sign-in", b"-"]);
    sign_in.input = link;
    caller.commands.push_back(sign_in);
    caller.commands.push_back(Command::new(&[
        b"forge",
        b"connect",
        b"1",
        b"github-app",
        b"4242",
        b"/setup/app.pem",
        b"--key",
        b"scope-app",
    ]));
    caller.commands.push_back(Command::new(&[b"forge", b"list", b"--until", b"connected:1", b"--json"]));
    caller.commands.push_back(adopt(b"umbra", b"tool", b"before-install"));
    if matches!(scenario, Scenario::Uninstalled) {
        return 4;
    }
    caller.commands.push_back(Command::new(&[b"forge", b"install", b"1", b"--key", b"scope-install"]));
    caller.commands.push_back(Command::browse(4));
    caller.commands.push_back(Command::new(&[b"forge", b"list", b"--until", b"settled", b"--json"]));
    caller.commands.push_back(adopt(b"acme", b"app", b"valid-install"));
    caller.commands.push_back(adopt(b"umbra", b"tool", b"after-install"));
    caller.commands.push_back(Command::new(&[
        b"account",
        b"add-key",
        b"1",
        b"/setup/llm.key",
        b"--deployment",
        b"--key",
        b"scope-account",
    ]));
    caller.commands.push_back(Command::new(&[b"account", b"list", b"--until", b"usable"]));
    let words = if matches!(scenario, Scenario::Delivered) {
        b"@plan Fix the adopted repository".as_slice()
    } else {
        b"Read the README"
    };
    caller.commands.push_back(Command::new(&[b"chat", b"start", b"--project", b"1", b"--key", b"scope-chat", words]));
    12
}
struct Judge {
    controls: skein_world::Controls,
    scenario: Scenario,
    store: Rc<RefCell<Store>>,
    forge: Rc<RefCell<Forge>>,
    answers: usize,
    calls: usize,
    stopping: bool,
    handed: bool,
    changed: Vec<u64>,
    read: bool,
    moves: usize,
}
impl skein_world::Referee<Proc> for Judge {
    fn act(&mut self, _now: Time, procs: &mut [Proc]) {
        let (first, other) = procs.split_at_mut(1);
        let Proc::Service(service) = &mut first[0] else { panic!("service first") };
        let Proc::Calls(caller) = &mut other[0] else { panic!("caller second") };
        if !self.handed
            && let Some(link) = service.service.links().pop()
        {
            self.calls = commands(caller, link, self.scenario);
            self.handed = true;
        }
        let peer = other
            .iter()
            .find_map(|proc| if let Proc::Forge(peer) = proc { Some(peer) } else { None })
            .expect("GitHub peer");
        let git = peer.git_calls().expect("git observer");
        let installed = peer.github_layer().expect("GitHub").installations();
        let Proc::Calls(caller) = &other[0] else { unreachable!() };
        for index in self.answers..caller.answers.len() {
            let answer = &caller.answers[index];
            let refused = index == 3 || index == 8;
            assert!(
                answer.exit == Exit::Code(if refused { 2 } else { 0 }),
                "GitHub call {index} accepted/refused as scripted"
            );
            if refused {
                let public = [&answer.output[..], &answer.error].concat();
                assert!(
                    String::from_utf8_lossy(&public).to_ascii_lowercase().replace('_', "").contains("noaccess"),
                    "installation refusal"
                );
                assert!(git.is_empty(), "adoption refusal starts no git request");
                assert!(self.changed.is_empty(), "adoption refusal moves no branch");
                assert!(!installed.iter().any(|row| row.account == UMBRA), "no other-account installation");
            }
        }
        self.answers = caller.answers.len();
        if !self.handed || self.answers != self.calls {
            return;
        }
        let complete = match self.scenario {
            Scenario::Uninstalled => {
                assert_eq!(installed.minted(), 0, "uninstalled adoption mints no token");
                assert_eq!(installed.iter().count(), 0);
                true
            }
            Scenario::ReadOnly => referee::chat_waited(&self.store.borrow()),
            Scenario::Delivered => !self.changed.is_empty(),
        };
        if complete && !self.stopping {
            if !matches!(self.scenario, Scenario::Uninstalled) {
                assert_eq!(installed.iter().map(|row| (row.id, row.account)).collect::<Vec<_>>(), [(1, ACME)]);
                assert!(!git.is_empty(), "authenticated checkout was exercised");
                if matches!(self.scenario, Scenario::ReadOnly) {
                    assert!(self.read, "the agent read the installed README");
                    assert_eq!(self.moves, 0);
                }
                if matches!(self.scenario, Scenario::Delivered) {
                    assert_eq!(self.changed.len(), 1, "one branch holds the delivered edit");
                }
                for request in git {
                    let minted = request.minted.expect("every git request has an installation token");
                    assert_eq!(minted.installation, 1);
                    assert!(request.status == 200 || (!request.writes && request.status == 404));
                    if matches!(self.scenario, Scenario::ReadOnly) {
                        assert!(!request.writes);
                    }
                }
                if matches!(self.scenario, Scenario::Delivered) {
                    assert!(git.iter().any(|request| request.writes));
                }
            }
            let Proc::Calls(caller) = &mut other[0] else { unreachable!() };
            caller.closed = true;
            self.controls.signal(0, skein_io::kernel::ServiceSignal::Terminate);
            for proc in other {
                referee::shutdown_peer(proc);
            }
            self.stopping = true;
        }
    }
    fn observe(&mut self, _now: Time, procs: &[Proc]) {
        for proc in procs {
            if let Proc::Peer(peer) = proc {
                self.read |= peer.observations().iter().any(|seen| {
                let skein_fake_peers::llm::Observation::Query { query, .. } = seen else { return false };
                query.messages.iter().flat_map(|message| message.parts.iter()).any(|part| matches!(part, skein_fake_llm_domain::api::Part::ToolOutput { output, is_error: false, .. } if output.windows(b"Scope fixture".len()).any(|bytes| bytes == b"Scope fixture")))
            });
            }
        }

        let mut forge = self.forge.borrow_mut();
        while let Some(observed) = forge.domain.pop_observation() {
            if let domain::Observation::Moved { repository, branch, to, .. } = observed {
                self.moves += 1;
                assert_ne!(branch.as_ref(), b"main");
                let object = forge.domain.object(to).expect("pushed object");
                if object
                    .tree
                    .iter()
                    .any(|(path, content)| path.as_ref() == b"conflict.txt" && content.as_ref() == b"fixed\n")
                {
                    assert_eq!(repository.as_ref(), b"acme/app");
                    self.changed.push(to);
                }
            }
        }
    }
    fn next_deadline(&self) -> Option<Time> {
        (!self.stopping).then_some(Time::from_nanos(10_000_000_000))
    }
    fn overdue(&self, now: Time) -> Option<String> {
        (!self.stopping && now >= Time::from_nanos(10_000_000_000)).then(|| {
            format!(
                "GitHub {:?}: {} of {} calls, {} changed branches; tasks {:?}; agents {:?}",
                self.scenario,
                self.answers,
                self.calls,
                self.changed.len(),
                self.store
                    .borrow()
                    .rows
                    .values()
                    .filter_map(|row| match row {
                        temper_engine_domain::Record::Core(jig_domain_core::Record::Tasks(
                            jig_domain_tasks::Stored::Live(task),
                        )) => Some((task.number, &task.phase)),
                        temper_engine_domain::Record::Core(_) | temper_engine_domain::Record::Forge { .. } => None,
                    })
                    .collect::<Vec<_>>(),
                self.store.borrow().agent_records
            )
        })
    }
    fn passed(&self) -> bool {
        self.stopping
    }
}
/// Exercise installation scope, checkout and the authenticated git route on simulated sockets/files.
pub fn run(scenario: Scenario) {
    ACTIVE.with(|active| active.set(true));
    stories::set(Story::Adopt);
    stories::setup_settings(world::Phase::Story);
    let store = Rc::new(RefCell::new(Store::new()));
    world::lower_store(Some(store.clone()));
    let (forge, peer) = peer();
    let mut disk = crate::machine::fresh();
    let pem = layer::keys::pem(layer::keys::KEYS[APP.key]).expect("fake App PEM");
    let setup = disk.lay(&[
        skein_fake_machine::Item::file(b"app.pem", &pem).mode(0o600),
        skein_fake_machine::Item::file(b"llm.key", fixtures::LLM_TOKEN).mode(0o600),
    ]);
    disk.close(setup);
    let client = disk.reopen_root(2);
    let mut world = skein_world::World::new_controlled(
        fixtures::SEED,
        skein_sim::Config::calm(),
        skein_world::Memory::Unchecked,
        |controls| Judge {
            controls,
            scenario,
            store,
            forge,
            answers: 0,
            calls: 0,
            stopping: false,
            handed: false,
            changed: Vec::new(),
            read: false,
            moves: 0,
        },
    )
    .with_machine(Machine(disk));
    world::host_programs(&mut world);
    world.spawn_restartable(world::service_roots(), world::service);
    world.spawn_root(skein_sim::Handle::new(client.raw()), |root| Proc::Calls(Box::new(Caller::scripted(root))));
    world.spawn(|| Proc::Forge(Box::new(SignalHost::new(peer, None))));
    world.spawn(|| {
        Proc::Peer(Box::new(SignalHost::new(
            if matches!(scenario, Scenario::ReadOnly) {
                fixtures::read_then_wait_provider()
            } else {
                stories::provider(Story::Adopt)
            },
            None,
        )))
    });
    let mut outcome = world.run();
    assert!(outcome.procs.iter().all(Host::is_empty), "GitHub processes drain");
    if matches!(scenario, Scenario::Uninstalled) {
        let worker = outcome.machine.0.reopen_root(1);
        for directory in [b"workspaces".as_slice(), b"trash", b"home"] {
            let dir =
                outcome.machine.0.open(worker, directory, skein_fake_machine::How::Directory).expect("worker scaffold");
            assert!(
                outcome.machine.0.list(dir, 16, 4096).expect("worker entries").is_empty(),
                "refusal leaves no checkout files"
            );
            outcome.machine.0.close(dir);
        }
        outcome.machine.0.close(worker);
    }
    world::lower_store(None);
    ACTIVE.with(|active| active.set(false));
}

pub(crate) fn set_active(active: bool) {
    ACTIVE.with(|slot| slot.set(active));
}
