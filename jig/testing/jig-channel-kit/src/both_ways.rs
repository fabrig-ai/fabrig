//! One Claude model reached by a program login or a key (domain/accounts.md, 3.4 and 4.3).
use crate::{
    account_setup, fixtures,
    processes::{self, Proc},
    program_runs,
    referee::Rules,
    stories::Story,
    world::{self, Store, Views},
};
use jig_domain_core as core;
use jig_protocol_worker_wire as wire;
use skein_fake_machine::How;
use skein_lib::{Duration, List, Reader, Time};
use skein_world::{HostedProgram, Inherited, StartupRoot};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

mod files;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Case {
    Key,
    Login,
    LoginToKey,
    KeyToLogin,
}
impl Case {
    fn crossing(self) -> bool {
        matches!(self, Self::LoginToKey | Self::KeyToLogin)
    }
}

fn engine(inherited: &Inherited) -> Proc {
    let mut document: serde_json::Value = serde_json::from_slice(&fixtures::engine_document()).expect("engine JSON");
    let llm = &mut document["engine"]["llm"];
    llm["providers"][0]["endpoint"] = "fake".into();
    llm["providers"][0]["direct"] = "direct".into();
    llm["providers"][0]["sources"] = serde_json::json!({"keys":true,"programs":["claude_code"]});
    llm["providers"][0]["manage_usage"] = "https://usage.example/".into();
    llm["endpoints"][0]["program"] = "claude_code".into();
    let mut direct = llm["endpoints"][0].clone();
    direct.as_object_mut().expect("endpoint").remove("program");
    direct["name"] = "direct".into();
    direct["number"] = 2.into();
    llm["endpoints"].as_array_mut().expect("endpoints").push(direct);
    llm["models"][0]["window"] = 272_000.into();
    processes::engine_document(inherited, &serde_json::to_vec(&document).expect("both ways JSON"))
}

fn declarations() -> smith_agent_service::profile::Configuration {
    let mut declarations = program_runs::declarations();
    declarations.endpoints[0].provider = skein_llm::Provider::Anthropic;
    let mut direct = declarations.endpoints[0].clone();
    direct.number = 1;
    declarations.endpoints = Box::new([declarations.endpoints[0].clone(), direct]);
    declarations
}
fn configuration() -> smith_agent_service::Config {
    let mut config = program_runs::configuration();
    config.limits.domain.endpoints = 2;
    config.limits.llm.llms = 2;
    config.channel_endpoints = endpoints();
    let endpoints = [0, 1].map(|number| {
        let (destinations, _) = program_runs::configuration().llm_endpoints.into_parts();
        let mut destination = destinations.into_boxed().into_vec().pop().expect("destination");
        destination.llm = skein_llm::Endpoint::anthropic();
        smith_protocol_llm::ConfiguredEndpoint {
            name: smith_domain::llm::Endpoint(number),
            destination,
            connect: Duration::from_secs(1),
            handshake: Duration::from_secs(1),
            oversized_reasoning: smith_protocol_llm::OversizedReasoning::Fail,
        }
    });
    config.llm_endpoints = smith_protocol_llm::Endpoints::new(Box::new(endpoints), 2).expect("both ways endpoints");
    config.domain.endpoints =
        Box::new([smith_domain::run::charter::Endpoint(0), smith_domain::run::charter::Endpoint(1)]);
    let mut direct = config.domain.limits[0].clone();
    direct.endpoint = smith_domain::run::charter::Endpoint(1);
    config.domain.limits = Box::new([config.domain.limits[0].clone(), direct]);
    config
}
fn endpoints() -> smith_protocol_channel::Endpoints {
    let mut entries = List::with_capacity(2);
    for (name, number) in [(b"fake".as_slice(), 0), (b"direct".as_slice(), 1)] {
        entries
            .push(smith_protocol_channel::Endpoint { name: name.into(), number, dialect: 3 })
            .expect("two endpoints");
    }
    smith_protocol_channel::Endpoints::new(entries)
}
pub(crate) fn worker_agent() -> jig_test_service::Agent {
    let mut agent = program_runs::worker_agent();
    agent.endpoints = endpoints();
    let bound = jig_domain_link::EndpointBound { endpoint: 1, window: 272_000, output: 512 };
    agent.bounds.endpoints = Box::new([bound.clone(), jig_domain_link::EndpointBound { endpoint: 2, ..bound }]);
    agent
}
fn agent(_spawn: &skein_io::kernel::Spawn, inherited: &Inherited) -> Proc {
    program_runs::configured_agent(inherited, configuration(), declarations())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Grant {
    Key,
    Login,
}
#[derive(Debug)]
pub struct Assignment {
    pub task: u64,
    pub attempt: u64,
    pub endpoint: Vec<u8>,
    pub model: Vec<u8>,
    pub source: u32,
    pub grant: Grant,
}
#[derive(Default, Debug)]
pub struct Run {
    pub assignments: Vec<Assignment>,
    pub terminals: Vec<core::TerminalRecord>,
    pub turns: Vec<core::TurnRecord>,
    pub requests: Vec<serde_json::Value>,
    pub request_heads: usize,
    pub credentials: Vec<(skein_llm::Provider, Option<skein_llm::Scheme>, Option<u32>)>,
    pub observations: Vec<smith_fake_claude::Observation>,
    pub program_spawns: usize,
    pub received: Vec<serde_json::Value>,
    pub diagnostics: String,
    pub sources: Vec<(u32, u32, u32, jig_domain_accounts::Kind)>,
    pub login_states: Vec<wire::LoginState>,
}
struct Judge {
    finding: Option<String>,
    shared: Rc<RefCell<Run>>,
    store: Store,
    rules: Rules,
    views: Views,
    accounts: account_setup::Setup,
    party: jig_fake_parties::Party,
    case: Case,
    period: bool,
    done: bool,
    sent: bool,
    login: Rc<Cell<Option<bool>>>,
    check_after: Option<usize>,
}
impl skein_world::Referee<Proc> for Judge {
    fn act(&mut self, now: Time, procs: &mut [Proc]) {
        if self.finding.is_none() {
            self.finding = self.check_act(now, procs).err();
        }
    }
    fn watch(&mut self, _now: Time, _procs: &[Proc]) -> Result<(), String> {
        if let Some(why) = &self.finding {
            return Err(why.clone());
        }
        Ok(())
    }
    fn overdue(&self, now: Time) -> Option<String> {
        (now > Time::ZERO.saturating_add(Duration::from_secs(45)))
            .then(|| format!("both ways settles: {:?}", self.shared.borrow()))
    }
    fn passed(&self) -> bool {
        self.finding.is_none() && self.done
    }
    fn last_word(&self) -> skein_world::LastWord {
        skein_world::LastWord::LastExpectation
    }
}
impl Judge {
    fn cross(&mut self, procs: &mut [Proc], parked: bool) -> Result<(), String> {
        if !self.case.crossing() || self.sent {
            return Ok(());
        }
        let signed_in = self.case == Case::KeyToLogin;
        if let Some(after) = self.check_after {
            let target = if signed_in { wire::LoginState::Usable } else { wire::LoginState::NotSignedIn };
            if parked && logins(procs)?.iter().any(|(frame, state)| *frame >= after && *state == target) {
                self.party.extend(Box::new([jig_fake_parties::Act::Request {
                    key: [2; 16],
                    ask: jig_fake_parties::Ask::Say {
                        task: jig_fake_parties::Task::Last,
                        words: b"Continue on the other source".as_slice().into(),
                        release: None,
                    },
                }]));
                self.sent = true;
            }
        } else if waiting(procs)? {
            // The first run still owns its slot while the worker's periodic check
            // reports the host-local change. The next activation sees that inventory.
            self.login.set(Some(signed_in));
            self.check_after = Some(engine_wire(procs)?.frames.len());
        }
        Ok(())
    }
}
fn engine_wire(procs: &[Proc]) -> Result<&processes::Wire, String> {
    procs
        .iter()
        .find_map(|proc| if let Proc::Engine(engine) = proc { Some(&engine.wire) } else { None })
        .ok_or_else(|| String::from("engine wire"))
}
fn logins(procs: &[Proc]) -> Result<Vec<(usize, wire::LoginState)>, String> {
    let mut found = Vec::new();
    for (frame, (_, kind, body)) in engine_wire(procs)?.frames.iter().enumerate() {
        if *kind == wire::Kind::Logins.number() {
            let logins = wire::Logins::decode(&wire::CEILINGS, &mut Reader::new(body))
                .map_err(|why| format!("login inventory: {why:?}"))?;
            if let Some(login) = logins.logins().iter().find(|login| login.name() == b"alice") {
                found.push((frame, login.state().clone()));
            }
        }
    }
    Ok(found)
}
fn waiting(procs: &[Proc]) -> Result<bool, String> {
    for (_, kind, body) in &world::check_worker_wire(procs)?.frames {
        if *kind == wire::Kind::Fact.number() {
            let fact =
                wire::Fact::decode(&wire::CEILINGS, &mut Reader::new(body)).map_err(|why| format!("fact: {why:?}"))?;
            if matches!(fact.value(), wire::FactValue::Phase(phase) if *phase.value() == wire::Phase::Waiting) {
                return Ok(true);
            }
        }
    }
    Ok(false)
}
fn assignments(observed: &processes::Wire) -> Result<Vec<Assignment>, String> {
    observed
        .frames
        .iter()
        .filter(|(_, kind, _)| *kind == wire::Kind::Assignment.number())
        .map(|(_, _, bytes)| {
            let assignment = wire::Assignment::decode(&wire::CEILINGS, &mut Reader::new(bytes))
                .map_err(|why| format!("assignment: {why:?}"))?;
            let charter =
                smith_charter::Charter::decode(&smith_charter::CEILINGS, &mut Reader::new(assignment.charter()))
                    .map_err(|why| format!("charter: {why:?}"))?;
            let source = charter.main().account().ok_or("selected source")?;
            let grant = assignment.grants().iter().find(|grant| grant.number() == source).ok_or("main's grant")?;
            let grant = match grant.value() {
                wire::GrantValue::Credential(value) => {
                    {
                        let (observed_value, wanted) = (value.scheme(), &(wire::Scheme::Key));
                        if observed_value != wanted {
                            return Err(format!(
                                "assertion `left == right` failed: left: {observed_value:?}, right: {wanted:?}"
                            ));
                        }
                    }
                    {
                        let (observed_value, wanted) = (&(value.bytes().expose()), &(fixtures::KEY));
                        if observed_value != wanted {
                            return Err(format!(
                                "assertion `left == right` failed: left: {observed_value:?}, right: {wanted:?}"
                            ));
                        }
                    }
                    Grant::Key
                }
                wire::GrantValue::Program(value) => {
                    {
                        let (observed_value, wanted) = (&(value.name()), &(b"alice"));
                        if observed_value != wanted {
                            return Err(format!(
                                "assertion `left == right` failed: left: {observed_value:?}, right: {wanted:?}"
                            ));
                        }
                    }
                    Grant::Login
                }
                wire::GrantValue::Application(_) => return Err(String::from("main uses the LLM grant")),
            };
            Ok(Assignment {
                task: assignment.task(),
                attempt: assignment.attempt(),
                endpoint: charter.main().endpoint().to_vec(),
                model: charter.main().model().to_vec(),
                source,
                grant,
            })
        })
        .collect()
}

#[must_use]
pub fn run(case: Case, story: &smith_fake_claude::Story, scripts: skein_fake_llm_domain::api::Scripts) -> Run {
    let mut files = skein_fake_machine::Machine::new();
    let base = files.lay(&program_runs::login_files(story));
    if case == Case::KeyToLogin {
        let directory = files.open(base, b"private/programs/alice", How::Directory).expect("login directory");
        files.remove(directory, b"fake-claude.login", false).expect("initially signed out");
        files.close(directory);
    }
    if case == Case::Key {
        let executable = configuration().program.expect("program declared").program.claude;
        assert_eq!(
            files.open(base, executable.strip_prefix(b"/").expect("absolute executable"), How::Read),
            Err(skein_fake_machine::Refusal::NotFound),
            "the key-only host has no configured program executable",
        );
    }
    let login = Rc::new(Cell::new(None));
    let client = files.open(base, b".", How::Directory).expect("client root");
    let shared = Rc::new(RefCell::new(Run::default()));
    let judge = Judge {
        finding: None,
        shared: shared.clone(),
        store: Store::new(),
        rules: Rules::default(),
        views: Views::default(),
        accounts: if case.crossing() {
            account_setup::Setup::program_and_key()
        } else if case == Case::Key {
            account_setup::Setup::default()
        } else {
            account_setup::Setup::program()
        },
        party: jig_fake_parties::Party::new(
            1,
            jig_fake_parties::Role::Owner,
            100,
            fixtures::party_script(&fixtures::one_chat(b"Hello")),
        ),
        case,
        period: false,
        done: false,
        sent: false,
        login: login.clone(),
        check_after: None,
    };
    let mut world = skein_world::World::new(912, skein_sim::Config::calm(), judge, skein_world::Memory::Unchecked)
        .with_machine(files::Files { inner: crate::program_sources::Files { files, base }, login });
    launch(&mut world, case, client, scripts);
    let mut outcome = world.run();
    let mut run = std::mem::take(&mut *shared.borrow_mut());
    observations(&mut outcome, base, &mut run);
    run
}

fn observations(
    outcome: &mut skein_world::Outcome<Proc, files::Files>,
    base: skein_fake_machine::Opened,
    run: &mut Run,
) {
    let executable = configuration().program.expect("declared program").program.claude;
    let program = skein_sim::Text::of(&executable);
    run.program_spawns = outcome.trace.iter().filter(|entry| matches!(entry.event,
        skein_sim::Event::Submit { kind: skein_sim::Summary::Spawn { program: seen, .. }, .. } if seen == program
    )).count();
    assert!(!run.terminals.is_empty(), "both ways ran: {:?}; {}", outcome.trace, run.diagnostics);
    for proc in &outcome.procs {
        if let Proc::Peer(peer) = proc {
            run.request_heads = peer
                .observations()
                .iter()
                .filter(|row| matches!(row, skein_fake_peers::llm::Observation::Head { .. }))
                .count();
            run.requests =
                peer.bodies().iter().map(|body| serde_json::from_slice(body).expect("Anthropic request")).collect();
            for observation in peer.observations() {
                if let skein_fake_peers::llm::Observation::Credential { route, scheme, index, .. } = observation {
                    run.credentials.push((*route, *scheme, *index));
                }
            }
        }
    }
    if let Ok(fd) =
        outcome.machine.inner.files.open(base, b"private/programs/alice/fake-claude.observations", How::Read)
    {
        let bytes = outcome.machine.inner.files.read(fd, 0, 65_536).expect("observations");
        assert!(bytes.len() < 65_536, "complete observations");
        run.observations = smith_fake_claude::observations(&bytes);
        run.received = smith_fake_claude::received(&bytes);
        outcome.machine.inner.files.close(fd);
    }
    outcome.machine.inner.files.close(base);
}

fn launch(
    world: &mut skein_world::World<Proc, Judge, files::Files>,
    case: Case,
    client: skein_fake_machine::Opened,
    scripts: skein_fake_llm_domain::api::Scripts,
) {
    host_programs(world, case);
    world.spawn_restartable(
        vec![StartupRoot { name: b"state".as_slice().into(), path: b"state".as_slice().into() }],
        engine,
    );
    world.spawn_restartable(
        vec![
            StartupRoot { name: b"worker".as_slice().into(), path: b".".as_slice().into() },
            StartupRoot { name: b"parent".as_slice().into(), path: b"private".as_slice().into() },
        ],
        processes::worker_both_ways,
    );
    world.spawn_root(skein_sim::Handle::new(client.raw()), |root| {
        let mut client = jig_client_protocol_kit::native::NativeClient::new(
            jig_client_protocol_kit::native::settings(),
            root,
            1000,
            fixtures::SEED,
            jig_client_protocol_kit::native::Script::account_sign_in(),
        );
        client.hold_for_account_sign_in();
        Proc::Client(Box::new(client))
    });
    world.spawn(|| {
        Proc::Peer(Box::new(
            fixtures::provider_played_at(
                (std::net::Ipv4Addr::LOCALHOST, 443).into(),
                scripts,
                skein_llm::Credential::key(fixtures::KEY.into()),
                Duration::ZERO,
                skein_fake_peers::Transport::Plaintext,
                skein_fake_llm_protocol::documents::Provider::Anthropic,
            )
            .keeping_bodies(),
        ))
    });
}

fn host_programs(world: &mut skein_world::World<Proc, Judge, files::Files>, case: Case) {
    world.host_startup(
        HostedProgram {
            program: b"program-agent".as_slice().into(),
            make: agent,
            instances: 2,
            operations: smith_agent_service::program_operations(&configuration())
                .expect("program routes")
                .checked_add(8)
                .expect("shell operations"),
        },
        |_| vec![StartupRoot { name: b"programs".as_slice().into(), path: b"/private/programs".as_slice().into() }],
        |_| vec![],
    );
    // On the key-only host the declared executable has neither a file nor a
    // hosted implementation. The agent configuration is unchanged.
    if case != Case::Key {
        let fake = smith_fake_claude::hosted(b"/private/programs/claude");
        world.host_startup(
            HostedProgram {
                program: fake.program,
                make: program_runs::claude,
                instances: 2,
                operations: fake.operations,
            },
            |spawn| {
                let launch = program_runs::claude_launch(spawn);
                smith_fake_claude::config_root(&launch)
                    .into_iter()
                    .chain(smith_fake_claude::mcp_root(&launch))
                    .collect()
            },
            |spawn| smith_fake_claude::observation_append(spawn).into_iter().collect(),
        );
    }
    world.host(HostedProgram {
        program: b"/private/programs/relay".as_slice().into(),
        make: program_runs::relay,
        instances: 1,
        operations: smith_agent_shell::relay::operations(),
    });
}

impl Judge {
    fn check_act(&mut self, now: Time, procs: &mut [Proc]) -> Result<(), String> {
        let story = if self.case == Case::Key { Story::Key { replaced: false } } else { Story::Chat };
        self.accounts.act(procs, story, self.done)?;
        if !self.done {
            self.accounts.party(now, procs, &mut self.party, &mut self.views, Story::Chat)?;
        }
        for proc in procs.iter_mut() {
            if let Proc::Engine(host) = proc {
                world::answer_store(host, &mut self.store, &mut self.rules, false, Story::Chat, now)?;
                let engine = host.service.engine().ok_or("engine")?;
                world::answer_parties(engine, &mut self.party, &mut self.views)?;
                if !self.period && self.party.sign_in.is_some() {
                    engine.parties_in().push(core::Event::Period { project: 1, period: 1, budget: 100, next: None });
                    self.period = true;
                }
            }
        }
        if self.case == Case::Key && !self.sent && waiting(procs)? {
            self.party.extend(Box::new([jig_fake_parties::Act::Request {
                key: [2; 16],
                ask: jig_fake_parties::Ask::Say {
                    task: jig_fake_parties::Task::Last,
                    words: b"The tool ran".as_slice().into(),
                    release: None,
                },
            }]));
            self.sent = true;
        }
        let parked = self.store.rows.values().any(|row| {
            matches!(row,
            jig_test_domain::Record::Core(core::Record::Core(core::CoreRecord::Terminal(answer)))
            if answer.end == jig_domain_tasks::End::Parked)
        });
        self.cross(procs, parked)?;
        let mut run = self.shared.borrow_mut();
        run.turns = world::turns(&self.store).into_iter().cloned().collect();
        run.terminals = self
            .store
            .rows
            .values()
            .filter_map(|row| {
                if let jig_test_domain::Record::Core(core::Record::Core(core::CoreRecord::Terminal(answer))) = row {
                    Some(answer.clone())
                } else {
                    None
                }
            })
            .collect();
        self.done = run.terminals.iter().any(|answer| answer.end != jig_domain_tasks::End::Parked);
        run.sources =
            self.store
                .rows
                .values()
                .filter_map(|row| {
                    if let jig_test_domain::Record::Core(core::Record::Accounts(
                        jig_domain_accounts::Stored::Source { account, number, place, kind, .. },
                    )) = row
                    {
                        Some((*account, *number, *place, *kind))
                    } else {
                        None
                    }
                })
                .collect();
        run.login_states = logins(procs)?.into_iter().map(|(_, state)| state).collect();
        run.assignments = assignments(world::check_worker_wire(procs)?)?;
        run.diagnostics = world::check_diagnostics(procs)?;
        if self.done {
            for proc in procs {
                match proc {
                    Proc::Engine(host) | Proc::Worker(host) if !host.stopping => {
                        host.stopping = true;
                        host.service.shutdown();
                    }
                    Proc::Peer(peer) => peer.shutdown(),
                    _ => {}
                }
            }
        }

        Ok(())
    }
}
