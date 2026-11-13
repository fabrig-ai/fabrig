//! Ready's copies on the service (temper/docs/design/testing/README.md, 2.5 and temper/docs/design/testing/process-tier.md;
//! temper/docs/design/domain/forge/adoption.md, 5; temper/docs/design/protocol/setup/box.md, 1).
use crate::{
    copy::{self, Source},
    ctl::{Caller, Command},
    fixtures,
    referee::{self, Store},
    stories,
    world::{self, HarnessProcess, Person, ServiceObserver, Story},
};
use skein_fake_forge_domain::api::Fill;
use skein_fake_person::{Find, Step};
use skein_io::kernel::{Complete, Done, Exit, Fd, Op, Pipe, ServiceSignal, Spawn, Submit, Way};
use skein_lib::{Queue, Time, Token, Wall};
use skein_shell::Host;
use skein_ui::Role;
use std::{cell::RefCell, collections::BTreeMap, rc::Rc};
use temper_client_domain::{
    Address,
    address::{Choice, Step as Welcome},
};
use temper_engine_domain as root;
use temper_engine_domain_forge::{MakeFrom, MakingState, Stored};
use temper_playground::{
    machine::{Machine, SignalHost},
    programs::{self, Harness},
    service::{Observe, ServiceHost},
};

const SOURCES: &[Source] = &[
    Source { address: b"https://source.example.invalid/acme/widget", repository: b"catalogue/widget", branch: b"main" },
    Source {
        address: b"https://source.example.invalid/acme/gadget",
        repository: b"catalogue/gadget",
        branch: b"trunk",
    },
];
const PROGRAM: &[u8] = b"/world/ready-service";
const BOX_BYTES: u32 = 256;
const DEADLINE: Time = Time::from_nanos(15_000_000_000);

type Proc = programs::Process<ReadyWorld>;
struct ReadyWorld;
impl Harness for ReadyWorld {
    type Service = ServiceObserver;
    type Process = Native;
    fn configure(document: &mut serde_json::Value) {
        document["application"]["welcome"] = true.into();
        document["engine"]["projects"] = serde_json::json!([]);
        document["engine"]["acts"] = serde_json::json!([{"connector": 1, "may": ["adopt"]}]);
        document["application"]["machine"]["owned"] = serde_json::json!({
            "listen": {"socket": "127.0.0.1:3000"},
            "remote": "http://127.0.0.1:3000",
            "forgejo": "/world/forgejo", "runner": null,
            "person": {"login": "owner", "email": "owner@example.invalid"}
        });
    }
    fn seeds(_: u64) -> temper_service::settings::Seeds {
        fixtures::seeds()
    }
    fn files() -> temper_service::settings::Files {
        fixtures::files()
    }
    fn given(document: &serde_json::Value) -> temper_service::stock::Given {
        stories::given(document)
    }
}

enum Native {
    Person(HarnessProcess),
    Box(BoxPeer),
}
impl Native {
    fn host(&self) -> &dyn Host {
        match self {
            Self::Person(host) => host,
            Self::Box(host) => host,
        }
    }
    fn host_mut(&mut self) -> &mut dyn Host {
        match self {
            Self::Person(host) => host,
            Self::Box(host) => host,
        }
    }
}
impl Host for Native {
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
    fn operations(&self) -> u32 {
        self.host().operations()
    }
}

// The external box starts an already seeded fake forge. Its pipe carries no
// copy result: the service's setup plans make and adopt both repositories.
struct BoxPeer {
    root: Option<Fd>,
    pidfd: Option<Fd>,
    answer: Option<Fd>,
    input: Vec<u8>,
    started: bool,
    pending: usize,
    serial: u64,
    completions: Queue<Complete>,
    submissions: Queue<Submit>,
}
impl BoxPeer {
    fn new(root: Fd) -> Self {
        Self {
            root: Some(root),
            pidfd: None,
            answer: None,
            input: Vec::new(),
            started: false,
            pending: 0,
            serial: 0,
            completions: Queue::with_capacity(16),
            submissions: Queue::with_capacity(16),
        }
    }
    fn submit(&mut self, kind: Op) {
        self.serial += 1;
        self.pending += 1;
        self.submissions.push(Submit { op: Token::new(self.serial), kind });
    }
    fn read(&mut self, fd: Fd) {
        self.submit(Op::PipeRead { fd, buf: vec![0; usize::try_from(BOX_BYTES).expect("box bytes")].into() });
    }
}
impl Host for BoxPeer {
    fn iterate(&mut self, _: Time, _: Wall) {
        if let Some(root) = self.root.take() {
            self.submit(Op::Spawn {
                spawn: Box::new(Spawn {
                    program: PROGRAM.into(),
                    args: Box::new([]),
                    env: Box::new([]),
                    root,
                    dir: Box::new([]),
                    pipes: [(3, Way::Out), (4, Way::In)].map(|(child, way)| Pipe { child, way, parent: None }).into(),
                }),
            });
        }
        while let Some(complete) = self.completions.pop() {
            self.pending -= 1;
            match (complete.kind, complete.result.expect("box peer operation")) {
                (Op::Spawn { spawn }, Done::Spawned { pidfd, .. }) => {
                    self.submit(Op::Close { fd: spawn.root });
                    self.pidfd = Some(pidfd);
                    for pipe in spawn.pipes {
                        let fd = pipe.parent.expect("box parent pipe");
                        if pipe.child == 3 {
                            self.read(fd);
                        } else {
                            self.answer = Some(fd);
                        }
                    }
                    self.submit(Op::Wait { pidfd, reap: false });
                }
                (Op::PipeRead { fd, buf }, Done::Count(count)) => {
                    if count == 0 {
                        self.submit(Op::Close { fd });
                        let fd = self.answer.take().expect("box answers");
                        self.submit(Op::Close { fd });
                    } else {
                        self.input.extend_from_slice(&buf[..usize::try_from(count).expect("read size")]);
                        if self.input.ends_with(b"\n") {
                            assert_eq!(self.input, b"start 1 owned\n", "only the owned forge's start is asked");
                            assert!(!self.started, "one forge start");
                            self.started = true;
                            self.input.clear();
                            self.submit(Op::PipeWrite {
                                fd: self.answer.expect("box answers"),
                                bytes: b"started 1\n".as_slice().into(),
                                from: 0,
                            });
                        }
                        self.read(fd);
                    }
                }
                (Op::PipeWrite { fd, bytes, from }, Done::Count(count)) => {
                    let from = from + count;
                    if usize::try_from(from).expect("write offset") < bytes.len() {
                        self.submit(Op::PipeWrite { fd, bytes, from });
                    }
                }
                (Op::Wait { pidfd, reap: false }, Done::Exit(_)) => self.submit(Op::Wait { pidfd, reap: true }),
                (Op::Wait { pidfd, reap: true }, Done::Exit(exit)) => {
                    assert_eq!(exit, Exit::Code(0), "the service drains");
                    self.pidfd = None;
                    self.submit(Op::Close { fd: pidfd });
                }
                (Op::Close { .. }, Done::Nothing) => {}
                (op, done) => panic!("box completion {op:?}: {done:?}"),
            }
        }
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        &mut self.completions
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        &mut self.submissions
    }
    fn work_pending(&self, _: Time) -> bool {
        !self.completions.is_empty()
    }
    fn next_mechanism_deadline(&self) -> Option<Time> {
        None
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        None
    }
    fn is_empty(&self) -> bool {
        self.root.is_none() && self.pending == 0 && self.pidfd.is_none()
    }
    fn operations(&self) -> u32 {
        16
    }
}

fn service(inherited: &skein_world::Inherited, parameters: programs::Parameters) -> Proc {
    let root = inherited.roots.iter().find(|(name, _)| name.as_ref() == b"state").expect("state root").1;
    let worker = inherited.roots.iter().find(|(name, _)| name.as_ref() == b"worker").expect("worker root").1;
    let mut document: serde_json::Value = serde_json::from_slice(&parameters.deployment).expect("deployment JSON");
    ReadyWorld::configure(&mut document);
    let given = ReadyWorld::given(&document);
    temper_playground::documents::settings::strip_models(&mut document);
    let mut service = temper_service::startup::with_setup_given(
        &serde_json::to_vec(&document).expect("deployment bytes"),
        &jig_protocol_config::Directories::working(),
        &ReadyWorld::files(),
        Some(&temper_playground::documents::agent()),
        &ReadyWorld::seeds(parameters.seed),
        None,
        Some(temper_service::WorkerRuntime {
            root: worker,
            path: b"/worker".as_slice().into(),
            agent: temper_playground::documents::pinned_agent(),
        }),
        temper_playground::documents::forges(),
        None,
        Some(BOX_BYTES),
        None,
        &given,
        Box::new([]),
    )
    .unwrap_or_else(|problems| panic!("{}", String::from_utf8_lossy(&temper_service::startup::lines(&problems))));
    service.adopt_store(root, 1000).expect("service store");
    service.adopt_signals(inherited.signal).expect("service signal");
    let pipe = |child| inherited.pipes.iter().find(|(number, _)| *number == child).expect("box pipe").1;
    service.adopt_box(pipe(3), pipe(4), BOX_BYTES).expect("service's box");
    Proc::Service(Box::new(ServiceHost::new(service, ServiceObserver::started(root, inherited), parameters)))
}

fn steps() -> Box<[Step<Address>]> {
    let see = |role, name: &[u8]| Step::See { find: Find::exactly_named(role, name), within: fixtures::PERSON_WAIT };
    let press = |role, name: &[u8]| Step::Press { find: Find::exactly_named(role, name) };
    Box::new([
        Step::Go { address: Address::Welcome { step: Welcome::Forge { forge: Some(Choice::Owned) } } },
        see(Role::Choice, b"Copies of public repositories"),
        press(Role::Choice, b"Copies of public repositories"),
        Step::Type {
            find: Find::named(Role::TextArea, b"Their addresses, one per line"),
            words: b"source.example.invalid/acme/widget\nsource.example.invalid/acme/gadget".as_slice().into(),
        },
        press(Role::Button, b"Run on this machine"),
        Step::Go { address: Address::Welcome { step: Welcome::Ready } },
        see(Role::Heading { level: 1 }, b"Ready"),
        Step::See { find: Find::text(b"connected as temper"), within: fixtures::PERSON_WAIT },
        see(Role::Button, b"Create your project"),
        press(Role::Button, b"Create your project"),
        Step::See { find: Find::text(b"are in Widget"), within: fixtures::PERSON_WAIT },
    ])
}

#[derive(Debug, Default)]
pub struct Outcome {
    pub kept: usize,
    pub adopted: usize,
    pub first_making_second_waiting: bool,
}
struct Judge {
    controls: skein_world::Controls,
    store: Rc<RefCell<Store>>,
    rows: BTreeMap<root::Key, root::Record>,
    seen: Rc<RefCell<Outcome>>,
    handed: bool,
    browsing: bool,
    stopped: bool,
    checking: Option<()>,
    order: Vec<Vec<u8>>,
}
impl Judge {
    fn rows(&mut self) {
        let mut making = 0;
        let mut first = None;
        let mut second = None;
        for record in self.rows.values() {
            if let root::Record::Forge { row } = record
                && let Stored::Making(row) = row.as_ref()
            {
                let index = SOURCES
                    .iter()
                    .position(|source| {
                        source.repository.rsplit(|byte| *byte == b'/').next() == Some(row.key.name.as_ref())
                    })
                    .expect("catalogue target");
                assert_eq!(row.key.forge, 1);
                assert_eq!(row.key.org.as_ref(), b"acme");
                assert_eq!(row.from, MakeFrom::Copy { address: SOURCES[index].address.into() });
                let into = row.into.as_ref().expect("Ready's project");
                assert_eq!((into.project, into.home), (1, index == 0));
                assert!(!matches!(row.state, MakingState::Failed(_)), "no failed copy: {row:?}");
                if row.state == MakingState::Making {
                    making += 1;
                }
                if index == 0 {
                    first = Some(&row.state);
                } else {
                    second = Some(&row.state);
                }
            }
        }
        assert!(making <= 1, "one writing run per forge");
        if second == Some(&MakingState::Making) {
            assert!(
                self.seen.borrow().first_making_second_waiting,
                "request order: widget must be Making while gadget is Waiting first; widget {first:?}, gadget {second:?}"
            );
        }
        self.seen.borrow_mut().first_making_second_waiting |=
            first == Some(&MakingState::Making) && second == Some(&MakingState::Waiting);
    }
}
impl skein_world::Referee<Proc> for Judge {
    #[expect(clippy::too_many_lines, reason = "one referee turn checks commits, sign-in and both adoptions")]
    fn act(&mut self, _: Time, procs: &mut [Proc]) {
        let Some(service_index) = procs.iter().position(|proc| matches!(proc, Proc::Service(_))) else { return };
        let Proc::Service(service) = &mut procs[service_index] else { unreachable!() };
        let link = if self.handed { None } else { service.service.links().pop() };
        for commit in service.observer.commits.drain(..) {
            for write in &commit.writes {
                if let root::Write::Save(root::Record::Forge { row }) = write
                    && let Stored::Making(row) = row.as_ref()
                    && !self.order.contains(&row.key.name.to_vec())
                {
                    self.order.push(row.key.name.to_vec());
                    self.seen.borrow_mut().kept += 1;
                    eprintln!(
                        "Ready copy kept: {} at {:?}, home {}",
                        String::from_utf8_lossy(&row.key.name),
                        row.asked,
                        row.into.as_ref().expect("project").home
                    );
                }
                referee::restart::apply(&mut self.rows, write);
            }
            self.rows();
        }
        let caller = procs
            .iter_mut()
            .find_map(|proc| {
                if let Proc::Other(Native::Person(HarnessProcess::Calls(caller))) = proc { Some(caller) } else { None }
            })
            .expect("native ctl");
        if let Some(link) = link {
            let mut sign_in = Command::new(&[b"sign-in", b"-"]);
            sign_in.input = link;
            caller.commands.push_back(sign_in);
            // Prepare before the page's startup watches fill the exchange edge: a key is sent once.
            caller.commands.push_back(Command::new(&[
                b"account",
                b"add-key",
                b"1",
                b"/setup/llm.key",
                b"--deployment",
                b"--key",
                b"ready-account",
            ]));
            caller.commands.push_back(Command::new(&[b"account", b"list", b"--until", b"usable"]));
            self.handed = true;
        }
        for answer in &caller.answers {
            assert_eq!(answer.exit, Exit::Code(0), "ctl: {}", String::from_utf8_lossy(&answer.error));
        }
        if self.handed && caller.answers.len() == 3 && !self.browsing {
            for proc in procs.iter_mut() {
                if let Proc::Other(Native::Person(HarnessProcess::Client(person))) = proc {
                    person.saved();
                }
            }
            self.browsing = true;
        }
        let repositories: Vec<_> = self
            .rows
            .values()
            .filter_map(|record| {
                if let root::Record::Forge { row } = record
                    && let Stored::Repository(row) = row.as_ref()
                {
                    Some(row)
                } else {
                    None
                }
            })
            .collect();
        let finished = procs
            .iter()
            .any(|proc| matches!(proc, Proc::Other(Native::Person(HarnessProcess::Client(person))) if person.finished));
        if repositories.len() == 2 && finished && !self.stopped {
            let caller = procs
                .iter_mut()
                .find_map(|proc| {
                    if let Proc::Other(Native::Person(HarnessProcess::Calls(caller))) = proc {
                        Some(caller)
                    } else {
                        None
                    }
                })
                .expect("native ctl");
            if self.checking.is_none() {
                caller.commands.push_back(Command::new(&[b"forge", b"list", b"--json"]));
                caller.commands.push_back(Command::new(&[b"project", b"show", b"1", b"--json"]));
                self.checking = Some(());
            }
            let [_, _, _, forges, project] = caller.answers.as_slice() else { return };
            let forges = copy::json(forges);
            let [forge] = forges["forges"].as_array().expect("forge rows").as_slice() else {
                panic!("one connected forge")
            };
            assert_eq!(forge["number"], 1);
            assert_eq!(forge["makings"].as_array().expect("making rows").len(), 0);
            let project = copy::json(project);
            assert_eq!(project["number"], 1);
            let resources = project["resources"].as_array().expect("project resources");
            assert_eq!(resources.len(), 2, "both copies belong to Ready's project");
            assert_eq!(self.order, [b"widget".to_vec(), b"gadget".to_vec()], "requests kept in input order");
            assert!(
                !self.rows.values().any(
                    |record| matches!(record, root::Record::Forge { row } if matches!(row.as_ref(), Stored::Making(_)))
                ),
                "no making row remains"
            );
            for (index, source) in SOURCES.iter().enumerate() {
                let name = source.repository.rsplit(|byte| *byte == b'/').next().expect("source name");
                let repository = repositories.iter().find(|row| name == row.name.as_ref()).expect("copy adopted");
                assert_eq!(repository.project, 1);
                assert_eq!(repository.home, index == 0, "only the first copy is home");
                assert_eq!(repository.owner.as_ref(), b"acme");
                assert_eq!(repository.role, temper_engine_domain_forge::Role::Owned);
                assert_eq!(repository.settings.default_branch.as_ref(), source.branch);
                assert_eq!(repository.provider.forge, 1);
                let key = serde_json::json!(["0001", format!("{:08x}", repository.provider.repository)]);
                let resource = resources.iter().find(|row| row["part"]["key"]["path"] == key).expect("copy in project");
                assert_eq!(resource["role"], "owned");
                assert_eq!(resource["home"], index == 0);
            }
            self.seen.borrow_mut().adopted = repositories.len();
            self.controls.signal(service_index, ServiceSignal::Terminate);
            for proc in procs.iter_mut() {
                match proc {
                    Proc::Other(Native::Person(HarnessProcess::Calls(caller))) => caller.closed = true,
                    Proc::Forge(peer) => peer.shutdown(),
                    _ => {}
                }
            }
            self.stopped = true;
        }
        self.store.borrow_mut().pending.clear();
    }
    fn observe(&mut self, _: Time, _: &[Proc]) {}
    fn next_deadline(&self) -> Option<Time> {
        (!self.stopped).then_some(DEADLINE)
    }
    fn overdue(&self, now: Time) -> Option<String> {
        (!self.stopped && now >= DEADLINE).then(|| {
            format!(
                "Ready copies unfinished: {:?}; forges {:?}",
                self.seen.borrow(),
                self.rows.values().filter(|record| matches!(record, root::Record::Forge { .. })).collect::<Vec<_>>()
            )
        })
    }
    fn passed(&self) -> bool {
        self.stopped
    }
}

/// The welcome's native forms send Ready's burst to the service's copy plans.
#[must_use]
pub fn run() -> Outcome {
    stories::set(Story::Chat { words: b"Hello" });
    stories::setup_settings(world::Phase::Story);
    let transport = copy::catalogue(SOURCES, false);
    let store = Rc::new(RefCell::new(Store::new()));
    world::lower_store(Some(store.clone()));
    let mut disk = temper_playground::machine::fresh();
    temper_playground::machine::setup(&mut disk);
    let parent = disk.reopen_root(2);
    let caller = disk.reopen_root(2);
    let native = disk.reopen_root(2);
    let machine = Machine::new(disk);
    let parameters = world::parameters(&machine, transport.clone());
    let seen = Rc::new(RefCell::new(Outcome::default()));
    let outcome = programs::with(parameters.clone(), || {
        let mut simulation = skein_world::World::new_controlled(
            fixtures::SEED,
            skein_sim::Config::calm(),
            skein_world::Memory::Unchecked,
            |controls| Judge {
                controls,
                store,
                rows: BTreeMap::new(),
                seen: seen.clone(),
                handed: false,
                browsing: false,
                stopped: false,
                checking: None,
                order: Vec::new(),
            },
        )
        .with_machine(machine);
        for (program, roots) in programs::programs::<ReadyWorld>() {
            simulation.host_roots(program, roots);
        }
        simulation.host_roots(
            crate::ctl::program(|spawn, inherited| {
                Proc::Other(Native::Person(HarnessProcess::Ctl(Box::new(crate::ctl::host(spawn, inherited)))))
            }),
            crate::ctl::roots,
        );
        simulation.host_roots(
            skein_world::HostedProgram {
                program: PROGRAM.into(),
                make: move |_, inherited| {
                    // Factory parameters are scoped by programs::with; retain the same service inputs.
                    service(
                        inherited,
                        PARAMETERS.with(|value| value.borrow().as_ref().expect("Ready parameters").clone()),
                    )
                },
                instances: 1,
                operations: 4096,
            },
            |_| programs::service_roots(),
        );
        PARAMETERS.with(|value| *value.borrow_mut() = Some(parameters));
        simulation
            .spawn_root(skein_sim::Handle::new(parent.raw()), |root| Proc::Other(Native::Box(BoxPeer::new(root))));
        simulation.spawn_root(skein_sim::Handle::new(caller.raw()), |root| {
            Proc::Other(Native::Person(HarnessProcess::Calls(Box::new(Caller::scripted(root)))))
        });
        simulation.spawn(|| {
            let mut peer = copy::catalogue_peer(&transport, SOURCES, Fill::Whole, false);
            peer.setup_mut().account(b"owner", true, b"");
            Proc::Forge(Box::new(SignalHost::new(peer, None)))
        });
        simulation.spawn_root(skein_sim::Handle::new(native.raw()), |root| {
            Proc::Other(Native::Person(HarnessProcess::Client(Box::new(SignalHost::new(
                Person::welcome(root, steps()),
                None,
            )))))
        });
        simulation.run()
    });
    assert!(outcome.procs.iter().all(Host::is_empty), "Ready world drains every process");
    world::lower_store(None);
    PARAMETERS.with(|value| *value.borrow_mut() = None);
    Rc::try_unwrap(seen).expect("released Ready referee").into_inner()
}
thread_local! { static PARAMETERS: RefCell<Option<programs::Parameters>> = const { RefCell::new(None) }; }
