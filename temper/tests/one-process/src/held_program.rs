//! Host a signed-in program source for the stop and landing stories (temper/docs/design/testing/tiers.md, 1.5).
use crate::{
    ctl::{Caller, Command},
    fixtures,
    referee::Store,
    stories,
    world::{self, HarnessProcess, ServiceObserver, Story},
};
use serde_json::{Value, json};
use skein_fake_machine::{How, Opened};
use skein_io::kernel::{Complete, Error, Exit, ServiceSignal, Spawn, Submit};
use skein_lib::{Queue, Secret, Time, Wall};
use skein_shell::Host;
use skein_world::{Inherited, StartupRoot};
use std::{cell::RefCell, rc::Rc};
use temper_playground::{
    documents,
    machine::{self, Machine},
    programs::{self, Harness},
};

pub mod landing;
mod model_check;
pub use model_check::welcome_trial;

const CLAUDE: &[u8] = b"/usr/bin/claude";
const RELAY: &[u8] = b"/world/relay";
const SOURCE: &str = "held";

struct Held;
type Proc = programs::Process<Held>;
impl Harness for Held {
    type Service = ServiceObserver;
    type Process = Local;
    fn seeds(_: u64) -> temper_service::settings::Seeds {
        fixtures::seeds()
    }
    fn files() -> temper_service::settings::Files {
        fixtures::files()
    }
    fn configure(value: &mut temper_playground::documents::Configuration) {
        value["world_settings"] = fixtures::world_document()["world_settings"].clone();
        value["world_settings"]["llm"]["providers"][0]["sources"]["programs"] = json!(["claude_code"]);
        value["world_settings"]["llm"]["providers"][0]["manage_usage"] = "https://provider.example/usage".into();
        value["world_settings"]["llm"]["endpoints"][0]["program"] = "claude_code".into();
        if model_check::active() {
            model_check::configure(value);
        }
        if landing::active() {
            landing::configure(value);
        }
    }
    fn given(document: &temper_playground::documents::Configuration) -> temper_service::stock::Given {
        let mut given = documents::given_with(document, model_check::active(), false);
        if landing::active() {
            landing::give(&mut given);
        }
        given
    }
    fn agent_settings() -> temper_service::settings::Agent {
        let mut agent = documents::agent();
        agent.programs = Box::new([temper_service::settings::Program {
            adapter: temper_service::settings::Adapter::ClaudeCode,
            path: CLAUDE.into(),
            // The adapter below roots this namespace in the worker's disk.
            directory: b"/programs".as_slice().into(),
        }]);
        agent.bounds.program_endpoints = Box::new([1]);
        agent
    }
}

// Keep the adapters local while the other one-process modules are reshaped.
enum Local {
    World(HarnessProcess),
    Relay(Box<smith_agent_shell::relay::Process>),
}
impl Local {
    fn host(&self) -> &dyn Host {
        match self {
            Self::World(host) => host,
            Self::Relay(host) => host.as_ref(),
        }
    }
    fn host_mut(&mut self) -> &mut dyn Host {
        match self {
            Self::World(host) => host,
            Self::Relay(host) => host.as_mut(),
        }
    }
}
impl Host for Local {
    fn iterate(&mut self, now: Time, wall: Wall) {
        self.host_mut().iterate(now, wall);
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        self.host_mut().completions()
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        self.host_mut().submissions()
    }
    fn work_pending(&self, now: Time) -> bool {
        self.host().work_pending(now)
    }
    fn next_mechanism_deadline(&self) -> Option<Time> {
        self.host().next_mechanism_deadline()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        self.host().next_policy_deadline()
    }
    fn is_empty(&self) -> bool {
        self.host().is_empty()
    }
    fn exit(&self) -> Option<Exit> {
        self.host().exit()
    }
    fn operations(&self) -> u32 {
        self.host().operations()
    }
}

/// The program's absolute namespace is the worker disk. Under `/.` the fake
/// can open both `/programs` run files and `/workspaces` built-in act paths.
/// Other startup roots keep the shared world's meanings.
struct ProgramMachine(Machine);
impl skein_world::Machine for ProgramMachine {
    fn open_root(&mut self, path: &[u8]) -> Result<skein_sim::Handle, Error> {
        if path == b"." || path == b"/." || path == b"/programs" || path.starts_with(b"/programs/") {
            let mut disk = self.0.0.borrow_mut();
            let root = disk.reopen_root(1);
            if path == b"." || path == b"/." {
                return Ok(skein_sim::Handle::new(root.raw()));
            }
            let opened = disk.open(root, &path[1..], How::DirectoryNoFollow);
            disk.close(root);
            return opened.map(|fd| skein_sim::Handle::new(fd.raw())).map_err(skein_fake_machine::kernel_error);
        }
        self.0.open_root(path)
    }
    fn close_root(&mut self, root: skein_sim::Handle) {
        self.0.close_root(root);
    }
    fn open_append(&mut self, root: skein_sim::Handle, path: &[u8], mode: u32) -> Result<skein_sim::Handle, Error> {
        self.0
            .0
            .borrow_mut()
            .open(Opened::new(root.raw()), path, How::Append { mode })
            .map(|fd| skein_sim::Handle::new(fd.raw()))
            .map_err(skein_fake_machine::kernel_error)
    }
    fn step(&mut self, call: skein_sim::Call, answers: &mut Queue<skein_sim::Answer>) {
        self.0.step(call, answers);
    }
}

fn service(inherited: &Inherited) -> Proc {
    let Proc::Service(mut host) = programs::service::<Held>(inherited) else { unreachable!("service factory") };
    let parent =
        inherited.roots.iter().find(|(name, _)| name.as_ref() == b"programs_parent").expect("programs parent").1;
    let resources = temper_service::program_sources::configured(
        &Held::agent_settings().programs[0],
        Box::new([]),
        1000,
        Some(parent),
    )
    .expect("program resources");
    host.service.configure_program_sources(resources).expect("program source");
    if landing::active() {
        let mut document = programs::parameters().deployment.as_ref().clone();
        Held::configure(&mut document);
        let limits = temper_service::limits::engine(&document.jig, &document.limits).expect("landing engine limits");
        let codec = jig_protocol_store::codec::limits(&document.jig.core).expect("landing store codec");
        let families = temper_protocol_store::bounds(&limits.forge, limits.tasks.spec_bytes).expect("landing families");
        host.observer.set_codec(&codec, &families);
    }
    Proc::Service(host)
}

fn agent_configuration() -> smith_agent_service::Config {
    let mut configuration = documents::agent_configuration();
    configuration.domain.programs = Box::new([smith_domain::Program {
        endpoint: smith_domain::run::charter::Endpoint(0),
        name: b"claude_code".as_slice().into(),
    }]);
    // Charge the added declaration as smith-agent-document's config/programs does.
    configuration.limits.domain.configured_endpoint_bytes +=
        u64::try_from(std::mem::size_of::<smith_domain::Program>() + b"claude_code".len())
            .expect("program declaration size");
    configuration.program = Some(smith_agent_service::ProgramBackend {
        keep_program_prompt: false,
        program: smith_protocol_program::Programs { claude: CLAUDE.into(), directory: b"/programs".as_slice().into() },
        endpoints: Box::new([0]),
        relay: RELAY.into(),
        entropy: Secret::new(Box::new([7; 32])),
        effective_user: 1000,
        parent: None,
    });
    configuration
}
fn agent(_: &Spawn, inherited: &Inherited) -> Proc {
    let is_programs = |name: &[u8]| name == b"programs";
    let programs = inherited.roots.iter().find(|(name, _)| is_programs(name)).expect("programs root").1;
    let workspace = inherited.roots.iter().filter(|(name, _)| !is_programs(name)).map(|(_, fd)| *fd).collect();
    let pipe = |number| inherited.pipes.iter().find(|(child, _)| *child == number).expect("agent pipe").1;
    let (errors, capture) = skein_sim_world::PipeWriter::new(pipe(2), 4096);
    let agent = smith_agent_shell::Agent::new(
        smith_agent_shell::config::Configuration {
            service: agent_configuration(),
            memory: documents::agent_profile().declared.memory,
            trace: None,
            profile: documents::agent_profile(),
            declarations: documents::agent_declarations(),
            program: None,
            program_endpoints: std::collections::BTreeSet::from([0]),
        },
        smith_agent_shell::Resources {
            input: pipe(0),
            output: pipe(1),
            error: Some(pipe(2)),
            signals: inherited.signal,
            seed: fixtures::SEED,
            pid: 101,
            trace: None,
            roots: Some(workspace),
            programs: Some(smith_agent_shell::ProgramResources {
                relay: "/world/relay".into(),
                entropy: Secret::new(Box::new([7; 32])),
                directory: Some(programs),
            }),
        },
        Box::new(errors),
    )
    .expect("program-enabled agent");
    Proc::Agent(Box::new(machine::Rooted::new(capture.host(agent), inherited).with_hosted_workspace(CLAUDE)))
}
fn fake_launch(spawn: &Spawn) -> Spawn {
    let mut launch = Spawn {
        reach: None,
        program: spawn.program.clone(),
        args: spawn.args.clone(),
        env: spawn.env.clone(),
        root: spawn.root,
        dir: spawn.dir.clone(),
        pipes: spawn.pipes.clone(),
    };
    // Workspace prints already have a path from Start; the other prints run below /programs.
    if launch.args.first().is_some_and(|arg| arg.as_ref() == b"-p") && !launch.dir.starts_with(b"/") {
        launch.dir = [b"/programs/", launch.dir.as_ref()].concat().into();
    }
    launch
}
fn fake(spawn: &Spawn, inherited: &Inherited) -> Proc {
    Proc::Other(Local::World(HarnessProcess::ProgramSource(Box::new((smith_fake_claude::hosted(CLAUDE).make)(
        &fake_launch(spawn),
        inherited,
    )))))
}
fn relay(spawn: &Spawn, inherited: &Inherited) -> Proc {
    let pipe = |number| inherited.pipes.iter().find(|(child, _)| *child == number).map(|(_, fd)| *fd);
    Proc::Other(Local::Relay(Box::new(smith_agent_shell::relay::Process::new(
        &spawn.args,
        &spawn.env,
        smith_agent_shell::relay::Faces {
            stdin: pipe(0).expect("relay input"),
            stdout: pipe(1).expect("relay output"),
            stderr: pipe(2),
            signal: inherited.signal,
        },
    ))))
}

fn host_programs<R: skein_world::Referee<Proc>>(simulation: &mut skein_world::World<Proc, R, ProgramMachine>) {
    for (mut program, roots) in programs::programs::<Held>() {
        if program.program.as_ref() == documents::PINNED_AGENT_PROGRAM {
            program.make = agent;
            program.operations =
                smith_agent_service::program_operations(&agent_configuration()).expect("program operations") + 8;
            simulation.host_roots(program, agent_roots);
        } else {
            simulation.host_roots(program, roots);
        }
    }
    let factory = smith_fake_claude::hosted(CLAUDE);
    simulation.host_startup(
        skein_world::HostedProgram {
            program: factory.program,
            make: fake,
            instances: 2,
            operations: factory.operations,
        },
        |spawn| {
            let launch = fake_launch(spawn);
            smith_fake_claude::config_root(&launch).into_iter().chain(smith_fake_claude::mcp_root(&launch)).collect()
        },
        |spawn| smith_fake_claude::observation_append(spawn).into_iter().collect(),
    );
    simulation.host(skein_world::HostedProgram {
        program: RELAY.into(),
        make: relay,
        instances: 1,
        operations: smith_agent_shell::relay::operations(),
    });
}

fn lay_source(disk: &mut skein_fake_machine::Machine, source: &str, story: &smith_fake_claude::Story) {
    for (name, bytes) in
        [(smith_fake_claude::STORY_FILE, story.encode()), (smith_fake_claude::LOGIN_FILE, b"claude.ai".to_vec())]
    {
        machine::lay_handover(disk, format!("/worker/programs/{source}/{name}").as_bytes(), &bytes);
    }
}

fn observations(machine: &ProgramMachine, source: &str) -> Vec<u8> {
    let mut disk = machine.0.0.borrow_mut();
    let root = disk.reopen_root(1);
    let file = disk
        .open(root, format!("programs/{source}/{}", smith_fake_claude::OBSERVATIONS_FILE).as_bytes(), How::ReadNoFollow)
        .expect("fake observations");
    let size = u32::try_from(disk.stat(file).size).expect("bounded observations");
    let bytes = disk.read(file, 0, size).expect("observations read");
    disk.close(file);
    disk.close(root);
    bytes
}

struct Judge {
    controls: skein_world::Controls,
    stage: u32,
    answered: usize,
    finding: Option<String>,
    deadline: Time,
}
fn command(caller: &mut Caller, words: &[&str]) {
    caller.commands.push_back(Command::new(&words.iter().map(|word| word.as_bytes()).collect::<Vec<_>>()));
}
fn sign_in(caller: &mut Caller, link: Box<[u8]>) {
    let mut sign_in = Command::new(&[b"sign-in", b"-"]);
    sign_in.input = link;
    caller.commands.push_back(sign_in);
}

fn declare(caller: &mut Caller) {
    command(caller, &["account", "declare", "1", "claude_code", SOURCE, "--key", "held-source"]);
    command(caller, &["account", "list", "--json"]);
}

fn verify(caller: &mut Caller, list: usize) -> Result<(), String> {
    let listed: Value = serde_json::from_slice(&caller.answers[list].output).map_err(|why| why.to_string())?;
    let account = listed["accounts"]["party"]
        .as_array()
        .and_then(|accounts| accounts.iter().find(|account| account["sources"][0]["kind"] == "program"))
        .ok_or_else(|| format!("declared program account: {listed}"))?;
    let number = account["number"].as_u64().ok_or_else(|| String::from("account number"))?.to_string();
    for words in [
        vec!["account", "verify", &number, "--source", "program", "--json"],
        vec!["account", "list", "--until", "settled", "--json"],
        vec!["account", "list", "--until", "usable", "--json"],
    ] {
        command(caller, &words);
    }
    Ok(())
}

fn task_number(output: &[u8]) -> Result<&str, String> {
    let output = std::str::from_utf8(output).map_err(|why| why.to_string())?;
    output
        .split("done: task ")
        .nth(1)
        .and_then(|suffix| suffix.split(|c: char| !c.is_ascii_digit()).next())
        .ok_or_else(|| format!("chat task: {output}"))
}

impl Judge {
    fn act_checked(&mut self, now: Time, procs: &mut [Proc]) -> Result<(), String> {
        let (services, rest) = procs.split_at_mut(1);
        let Proc::Service(service) = &mut services[0] else { return Err("service first".into()) };
        let (callers, hosted) = rest.split_at_mut(1);
        let Proc::Other(Local::World(HarnessProcess::Calls(caller))) = &mut callers[0] else {
            return Err("caller second".into());
        };
        if self.stage == 0
            && let Some(link) = service.service.links().pop()
        {
            sign_in(caller, link);
            declare(caller);
            self.stage = 1;
        }
        while self.answered < caller.answers.len() {
            let answer = &caller.answers[self.answered];
            if answer.exit != Exit::Code(0) {
                return Err(format!(
                    "call {}: {:?}: {} {}",
                    self.answered,
                    answer.exit,
                    String::from_utf8_lossy(&answer.output),
                    String::from_utf8_lossy(&answer.error)
                ));
            }
            self.answered += 1;
            self.deadline = now.saturating_add(temper_client_shell_cli::limits::DEADLINE);
        }
        if self.stage == 1 && self.answered == 3 {
            verify(caller, 2)?;
            command(caller, &["chat", "start", "--project", "1", "--key", "held-chat", "Hello"]);
            self.stage = 2;
        }
        if self.stage == 2 && self.answered == 7 {
            let task = task_number(&caller.answers[6].output)?.to_owned();
            command(caller, &["task", "show", &task, "--until", "phase:running"]);
            self.stage = 3;
        }
        if self.stage == 3 && self.answered == 8
            && hosted.iter().any(|proc| matches!(proc, Proc::Other(Local::World(HarnessProcess::ProgramSource(fake))) if fake.active_turn() == Some(1))) {
            caller.closed = true;
            self.controls.signal(0, ServiceSignal::Terminate);
            for proc in hosted {
                if let Proc::Forge(peer) = proc { peer.shutdown(); }
            }
            self.stage = 4;
        }
        Ok(())
    }
}
impl skein_world::Referee<Proc> for Judge {
    fn act(&mut self, now: Time, procs: &mut [Proc]) {
        if self.finding.is_none() {
            self.finding = self.act_checked(now, procs).err();
        }
    }
    fn watch(&mut self, _: Time, procs: &[Proc]) -> Result<(), String> {
        if let Some(why) = &self.finding {
            return Err(why.clone());
        }
        for proc in procs {
            if let Proc::Service(service) = proc {
                if let Some(why) = &service.observer.finding {
                    return Err(why.clone());
                }
                if let Some(end) = service.service.end() {
                    if self.stage < 4 {
                        return Err(format!("service ended before the held-turn stop: {end:?}"));
                    }
                    if procs.iter().any(|proc| matches!(proc, Proc::Other(Local::World(HarnessProcess::ProgramSource(fake))) if fake.exit().is_none())) {
                        return Err("hosted program survives serve".into());
                    }
                    self.stage = 5;
                }
            }
        }
        Ok(())
    }
    fn next_deadline(&self) -> Option<Time> {
        (self.stage < 5).then_some(self.deadline)
    }
    fn overdue(&self, now: Time) -> Option<String> {
        (self.stage < 5 && now >= self.deadline)
            .then(|| format!("held program stage {}, {} answers", self.stage, self.answered))
    }
    fn passed(&self) -> bool {
        self.finding.is_none() && self.stage == 5
    }
}

/// Start a chat on a signed-in program source and signal serve during its held turn.
pub fn stop() {
    stories::set(Story::Adopt);
    stories::setup_settings(world::Phase::Restart);
    world::lower_store(Some(Rc::new(RefCell::new(Store::new()))));
    let story = smith_fake_claude::Story {
        turns: vec![smith_fake_claude::Turn::new(vec![smith_fake_claude::Step::Hang])],
        ..smith_fake_claude::Story::default()
    };
    let outcome = run(&story, &stories::forge(Story::Adopt), |controls| Judge {
        controls,
        stage: 0,
        answered: 0,
        finding: None,
        deadline: Time::ZERO.saturating_add(temper_client_shell_cli::limits::DEADLINE),
    });
    let Proc::Service(service) = &outcome.procs[0] else { panic!("service first") };
    assert_eq!(service.service.end(), Some(&temper_service::End::Engine(temper_service::EngineEnd::Drained)));
    let programs: Vec<_> = outcome
        .procs
        .iter()
        .filter_map(|proc| {
            if let Proc::Other(Local::World(HarnessProcess::ProgramSource(fake))) = proc { Some(fake) } else { None }
        })
        .collect();
    assert!(!programs.is_empty(), "the source ran its program");
    assert!(programs.iter().all(|program| program.exit().is_some()), "no hosted program survives serve");
    assert!(outcome.procs.iter().all(Host::is_empty), "held-program world drains every process");
    let seen = smith_fake_claude::seen(&observations(&outcome.machine, SOURCE));
    assert!(
        seen.contains(&smith_fake_claude::Seen::Interrupted),
        "agent's cancel must interrupt its program: {seen:?}"
    );
    world::lower_store(None);
}

fn agent_roots(spawn: &Spawn) -> Vec<StartupRoot> {
    let mut roots = if landing::active() { machine::agent_roots::<Held>(spawn) } else { Vec::new() };
    roots.push(StartupRoot { name: b"programs".as_slice().into(), path: b"/programs".as_slice().into() });
    roots
}

fn run<R: skein_world::Referee<Proc>>(
    story: &smith_fake_claude::Story,
    transport: &temper_fake_git_hosted::Typed,
    judge: impl FnOnce(skein_world::Controls) -> R,
) -> skein_world::Outcome<Proc, ProgramMachine> {
    let mut disk = machine::fresh();
    machine::setup(&mut disk);
    lay_source(&mut disk, SOURCE, story);
    let caller = disk.reopen_root(2);
    let machine = Machine::new(disk);
    let parameters = world::parameters(&machine, transport.clone());
    programs::with(parameters, || {
        let mut simulation = skein_world::World::new_controlled(
            fixtures::SEED,
            skein_sim::Config::calm(),
            skein_world::Memory::Unchecked,
            judge,
        )
        .with_machine(ProgramMachine(machine));
        host_programs(&mut simulation);
        simulation.host_roots(
            crate::ctl::program(|spawn, inherited| {
                Proc::Other(Local::World(HarnessProcess::Ctl(Box::new(crate::ctl::host(spawn, inherited).capturing()))))
            }),
            crate::ctl::roots,
        );
        let mut roots = programs::service_roots();
        roots.push(StartupRoot { name: b"programs_parent".as_slice().into(), path: b"worker".as_slice().into() });
        simulation.spawn_restartable(roots, service);
        simulation.spawn_root(skein_sim::Handle::new(caller.raw()), |root| {
            Proc::Other(Local::World(HarnessProcess::Calls(Box::new(Caller::scripted(root)))))
        });
        simulation.spawn(|| {
            Proc::Forge(Box::new(machine::SignalHost::new(
                temper_playground::fakes::forge_peer(transport, fixtures::SEED),
                None,
            )))
        });
        if landing::active() {
            simulation.spawn(|| {
                Proc::Peer(Box::new(machine::SignalHost::new(fixtures::scripted_provider(Box::new([])), None)))
            });
        }
        simulation.run()
    })
}
