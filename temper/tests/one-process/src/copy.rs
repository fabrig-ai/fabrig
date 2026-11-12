//! Copies on the running service and a catalogue source (temper/docs/design/domain/forge/adoption.md, 5).
use crate::{
    ctl::{Caller, Command},
    fixtures,
    referee::Store,
    stories,
    world::{self, HarnessProcess, ServiceObserver, Story},
};
use skein_fake_forge_domain::api::{Fill, Op, Write};
use skein_fake_peers::forge::{Observation, Peer};
use skein_io::kernel::{Exit, ServiceSignal};
use skein_lib::{Duration, List, Time};
use skein_shell::Host;
use std::{cell::RefCell, rc::Rc};
use temper_engine_domain as root;
use temper_engine_domain_forge::{MakingState, RepositoryMaking, Stored};
use temper_playground::{
    machine::{Machine, SignalHost},
    programs::{self, Harness, Role},
};

const SOURCE: &[u8] = b"https://source.example.invalid/acme/app";
const DESTINATION: &[u8] = b"acme/copied";
const BRANCH: &[u8] = b"trunk";
const SOURCE_WORKFLOW_PATH: &[u8] = b".github/workflows/ci.yml";
const SOURCE_WORKFLOW: &[u8] =
    b"on: [push]\njobs:\n  source:\n    runs-on: ubuntu-latest\n    steps:\n      - run: echo source check\n";
const RUNNER: &[u8] = b"copy-runner";
// The source finishes while the killed engine reopens its store.
const SLOW: Duration = Duration::from_secs(4);
const DEADLINE: Time = Time::from_nanos(15_000_000_000);

#[derive(Default, Debug)]
pub struct Outcome {
    pub copying: bool,
    pub restarted: bool,
    pub reran: bool,
    pub adopted: usize,
    pub copies: usize,
    pub deletions: usize,
    cut: Option<u32>,
}

#[derive(Default)]
struct Starts {
    lives: u32,
    store: Option<Rc<RefCell<Store>>>,
    transport: Option<temper_fake_git_hosted::Typed>,
}
thread_local! { static STARTS: RefCell<Starts> = RefCell::new(Starts::default()); }
struct CopyWorld;
type Proc = programs::Process<CopyWorld>;
impl Harness for CopyWorld {
    type Service = ServiceObserver;
    type Process = HarnessProcess;
    fn configure(document: &mut serde_json::Value) {
        document["engine"]["acts"] = serde_json::json!([{"connector": 1, "may": ["adopt"]}]);
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
    fn spawned(role: Role, _: &skein_world::Inherited, _: &[u8]) {
        if role == Role::Service {
            STARTS.with(|starts| {
                let mut starts = starts.borrow_mut();
                if starts.lives != 0 {
                    let store = starts.store.as_ref().expect("copy's store");
                    assert!(
                        making(&store.borrow()).is_some_and(|row| row.copying),
                        "the restarted engine recovers the durable copying mark"
                    );
                    let forge = starts.transport.as_ref().expect("retained peer").forge.borrow();
                    assert!(forge.domain.metadata(DESTINATION).is_ok(), "migration reserved its destination");
                    assert!(
                        forge.domain.branch(DESTINATION, BRANCH).is_none(),
                        "the engine is killed while the slow source is incomplete"
                    );
                    store.borrow_mut().recovering();
                }
                starts.lives += 1;
            });
        }
    }
}

pub(crate) struct Source {
    pub address: &'static [u8],
    pub repository: &'static [u8],
    pub branch: &'static [u8],
}
const SOURCES: &[Source] = &[Source { address: SOURCE, repository: b"acme/app", branch: BRANCH }];

fn peer(transport: &temper_fake_git_hosted::Typed, fill: Fill, partial: bool) -> Peer {
    catalogue_peer(transport, SOURCES, fill, partial)
}

pub(crate) fn catalogue_peer(
    transport: &temper_fake_git_hosted::Typed,
    sources: &[Source],
    fill: Fill,
    partial: bool,
) -> Peer {
    let mut table = temper_playground::fakes::forge_table();
    let mut repositories = List::with_capacity(16);
    for (index, source) in sources.iter().enumerate() {
        repositories
            .push(skein_fake_forge_forgejo::Repository {
                id: fixtures::REPOSITORY + u32::try_from(index).expect("catalogue size"),
                full: source.repository.into(),
                protected: None,
            })
            .expect("source repository");
    }
    if partial {
        repositories
            .push(skein_fake_forge_forgejo::Repository { id: 3, full: DESTINATION.into(), protected: None })
            .expect("partial copy");
    }
    table.repositories = repositories;
    table.sources = sources
        .iter()
        .map(|source| skein_fake_forge_forgejo::Source {
            address: source.address.into(),
            repository: source.repository.into(),
            private: false,
            fill,
        })
        .collect();
    let mut peer = Peer::new(
        fixtures::FORGE,
        fixtures::peer_limits(),
        table,
        fixtures::forge_limits(),
        transport.forge.clone(),
        fixtures::SEED,
    )
    .expect("catalogue peer");
    temper_playground::fakes::setup_peer(&mut peer);
    peer
}

fn source(workflow: bool) -> temper_fake_git_hosted::Typed {
    catalogue(SOURCES, workflow)
}

pub(crate) fn catalogue(sources: &[Source], workflow: bool) -> temper_fake_git_hosted::Typed {
    use skein_fake_forge_domain::{self as domain, api};
    let mut config = temper_playground::documents::hosted_git::CONFIG;
    // A slow migration and the recovered engine's request can coexist.
    config.limits.calls = 2;
    let mut domain = domain::Domain::new(&config, fixtures::SEED);
    let mut tree = vec![api::File { path: b"README.md".as_slice().into(), content: b"# Source\n".as_slice().into() }];
    if workflow {
        tree.push(api::File { path: SOURCE_WORKFLOW_PATH.into(), content: SOURCE_WORKFLOW.into() });
    }
    for (index, source) in sources.iter().enumerate() {
        domain::repository(
            &mut domain,
            &config,
            api::Setup {
                id: fixtures::REPOSITORY + u32::try_from(index).expect("catalogue size"),
                name: source.repository.into(),
                default: source.branch.into(),
                workflows: Box::new([]),
                labels: Box::new([]),
                protection: None,
                hooked: false,
                tree: tree.clone().into_boxed_slice(),
                checks: api::Checks {
                    cues: Box::new([]),
                    contexts: Box::new([]),
                    latency_min: Duration::ZERO,
                    latency_max: Duration::ZERO,
                    silent: 0,
                    passes: 1000,
                    reruns: 0,
                    cue: None,
                },
            },
        );
        domain::grant(&mut domain, source.repository, 2, api::Permission::Read);
    }
    let forge = Rc::new(RefCell::new(skein_fake_peers::forge::Forge::new(domain, config)));
    temper_fake_git_hosted::Typed::new(forge, &temper_playground::fakes::forge_table())
}

fn command(caller: &mut Caller, words: &[&[u8]]) {
    caller.commands.push_back(Command::new(words));
}
pub(crate) fn json(answer: &crate::ctl::Answer) -> serde_json::Value {
    // Connect and copy print progress before their terminal adoption answers.
    serde_json::Deserializer::from_slice(&answer.output)
        .into_iter::<serde_json::Value>()
        .last()
        .expect("terminal ctl answer")
        .expect("ctl JSON")
}
fn copy(caller: &mut Caller, key: &[u8], until_adopted: bool) {
    let mut words: Vec<&[u8]> = vec![
        b"repository",
        b"copy",
        b"--project",
        b"1",
        b"--forge",
        b"1",
        SOURCE,
        b"--as",
        DESTINATION,
        b"--no-home",
        b"--key",
        key,
        b"--json",
    ];
    if until_adopted {
        words.extend([b"--until".as_slice(), b"adopted"]);
    }
    command(caller, &words);
}
fn making(store: &Store) -> Option<&RepositoryMaking> {
    store.rows.values().find_map(|row| {
        if let root::Record::Forge { row } = row
            && let Stored::Making(row) = row.as_ref()
        {
            Some(row)
        } else {
            None
        }
    })
}
fn acted(store: &Store) -> usize {
    store
        .rows
        .values()
        .filter(|row| {
            matches!(
                row,
                root::Record::Core(jig_domain_core::Record::People(jig_domain_people::Stored::Acted {
                    act: jig_domain_people::Act::Adopt { project: 1, .. },
                    outcome: jig_domain_people::Outcome::Adopted { project: 1 },
                    ..
                }))
            )
        })
        .count()
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SourceStage {
    Slow,
    Cut,
    Closing,
    Whole,
    BesideRepository,
}
struct Judge {
    controls: skein_world::Controls,
    store: Rc<RefCell<Store>>,
    transport: temper_fake_git_hosted::Typed,
    seen: Rc<RefCell<Outcome>>,
    source: SourceStage,
    handed: bool,
    answers: usize,
    peer_seen: usize,
    stopped: bool,
    last_commit: u64,
}
impl Judge {
    fn shutdown(&self, caller: &mut Caller, peer: &mut Peer) {
        caller.closed = true;
        self.controls.signal(0, ServiceSignal::Terminate);
        peer.shutdown();
    }

    fn beside_repository(&self, caller: &Caller, peer: &Peer) {
        let [_, connected, _, copied, forges, project] = caller.answers.as_slice() else {
            panic!("sign-in, connect, connected forge, copy, forges and project answers")
        };
        let connected = json(connected);
        assert_eq!(connected["project"], 1, "the first repository is adopted into project 1");
        assert_eq!(connected["outcome"]["adopted"]["forge"], 1);
        let copied = json(copied);
        assert_eq!(copied["adopted"], true, "ctl waited for adoption");
        let forges = json(forges);
        let [forge] = forges["forges"].as_array().expect("forge rows").as_slice() else {
            panic!("one connected forge")
        };
        assert_eq!(forge["number"], 1);
        assert!(forge["makings"].as_array().expect("making rows").is_empty());
        let provider = peer
            .observations()
            .iter()
            .find_map(|observation| {
                if let Observation::Call {
                    op: Op::Write(Write::Copy { owner, name, .. }),
                    result,
                    ..
                } = observation
                    && let Ok(skein_fake_forge_domain::api::Answer::Copied { repository, .. }) = &**result
                    && owner.as_ref() == b"acme"
                    && name.as_ref() == b"copied"
                {
                    Some(*repository)
                } else {
                    None
                }
            })
            .expect("the forge gave the copied repository its provider id");
        let project = json(project);
        assert_eq!(project["number"], 1);
        let resources = project["resources"].as_array().expect("project resources");
        assert_eq!(resources.len(), 2, "the first repository and the copy share the project");
        // Project resources name the forge and provider id as big-endian hex segments.
        let key = serde_json::json!(["0001", format!("{provider:08x}")]);
        let repository = resources
            .iter()
            .find(|row| row["part"]["key"]["path"] == key)
            .expect("the copy's provider identity joined the first repository's project");
        assert_eq!(repository["role"], "owned");
        assert_eq!(repository["home"], false, "the copy is not the project's home");

        let forge = self.transport.forge.borrow();
        let source_metadata = forge.domain.metadata(b"acme/app").expect("source metadata");
        let metadata = forge.domain.metadata(DESTINATION).expect("copy metadata");
        assert_eq!(metadata.default_branch, source_metadata.default_branch);
        let source = forge.domain.branch(b"acme/app", &source_metadata.default_branch).expect("source head");
        let tip = forge.domain.branch(DESTINATION, &metadata.default_branch).expect("copy head");
        let commit = forge.domain.object(tip).expect("copy's workflow commit");
        assert_eq!(commit.parent, Some(source), "one workflow commit above the source head");
        assert_eq!(commit.merge_parent, None);
        let source_tree = &forge.domain.object(source).expect("source commit").tree;
        assert_eq!(source_tree.get(SOURCE_WORKFLOW_PATH).expect("source's workflow").as_ref(), SOURCE_WORKFLOW);
        let mut expected: std::collections::BTreeMap<_, _> = source_tree.iter().collect();
        let starter = temper_forge_setup::starter_workflow(RUNNER).expect("label's starter workflow");
        let starter_path = temper_forge_setup::STARTER_PATH.into();
        expected.insert(&starter_path, &starter);
        assert_eq!(
            commit.tree.iter().collect::<std::collections::BTreeMap<_, _>>(),
            expected,
            "the source's README and workflow are unchanged beside the label's starter"
        );
    }

    fn caller(&mut self, service: &mut temper_service::Service, caller: &mut Caller) {
        if !self.handed
            && let Some(link) = service.links().pop()
        {
            let mut sign_in = Command::new(&[b"sign-in", b"-"]);
            sign_in.input = link;
            caller.commands.push_back(sign_in);
            let mut connect: Vec<&[u8]> = vec![
                b"forge",
                b"connect",
                b"forgejo-admin",
                b"/setup/admin.token",
                b"--api",
                b"127.0.0.1:3000",
                b"--git",
                b"http://127.0.0.1:3000",
                b"--key",
                b"copy-forge",
                b"--json",
            ];
            let beside = self.source == SourceStage::BesideRepository;
            if beside {
                connect.extend([b"--first".as_slice(), b"acme/first", b"--project", b"1", b"--label", RUNNER]);
            }
            command(caller, &connect);
            command(caller, &[b"forge", b"list", b"--until", b"connected:1", b"--json"]);
            copy(caller, b"first-copy", beside);
            if beside {
                command(caller, &[b"forge", b"list", b"--json"]);
                command(caller, &[b"project", b"show", b"1", b"--json"]);
            }
            self.handed = true;
        }
        while self.answers < caller.answers.len() {
            let answer = &caller.answers[self.answers];
            assert_eq!(
                answer.exit,
                Exit::Code(0),
                "copy ctl {}: {} {}",
                self.answers,
                String::from_utf8_lossy(&answer.output),
                String::from_utf8_lossy(&answer.error)
            );
            self.answers += 1;
        }
    }
    fn calls(&mut self, peer: &Peer) {
        for record in &peer.observations()[self.peer_seen..] {
            if let Observation::Call { op: Op::Write(write), .. } = record {
                match write {
                    Write::Copy { .. } => self.seen.borrow_mut().copies += 1,
                    Write::DeleteRepository => self.seen.borrow_mut().deletions += 1,
                    _ => {}
                }
            }
        }
        self.peer_seen = peer.observations().len();
    }
}
impl skein_world::Referee<Proc> for Judge {
    fn act(&mut self, _: Time, procs: &mut [Proc]) {
        let (first, others) = procs.split_at_mut(1);
        let Proc::Service(service) = &mut first[0] else { panic!("service first") };
        let (callers, peers) = others.split_at_mut(1);
        let Proc::Other(HarnessProcess::Calls(caller)) = &mut callers[0] else { panic!("ctl second") };
        let Proc::Forge(peer_host) = &mut peers[0] else { panic!("forge third") };
        self.calls(peer_host);
        self.caller(&mut service.service, caller);
        let mut store = self.store.borrow_mut();
        let restarted = STARTS.with(|starts| starts.borrow().lives > 1);
        for commit in service.observer.commits.drain(..) {
            if commit.number <= self.last_commit {
                continue;
            }
            self.last_commit = commit.number;
            for write in commit.writes {
                if let root::Write::Save(root::Record::Forge { row }) = &write
                    && let Stored::Making(row) = row.as_ref()
                {
                    self.seen.borrow_mut().reran |= restarted && row.state == MakingState::Making;
                }
            }
        }
        let row = making(&store);
        if let Some(row) = row {
            self.seen.borrow_mut().copying |= row.copying;
        }
        let migrating = peer_host.observations().iter().any(|record| {
            matches!(record,
                Observation::Head { call, .. } if call.target.as_ref() == b"/api/v1/repos/migrate"
            )
        });
        if self.source != SourceStage::BesideRepository
            && migrating
            && self.seen.borrow().cut.is_none()
            && self.transport.forge.borrow().domain.metadata(DESTINATION).is_ok()
        {
            assert!(row.is_some_and(|row| row.copying), "the copy mark is durable before migration");
            assert!(
                self.transport.forge.borrow().domain.branch(DESTINATION, BRANCH).is_none(),
                "the source is still copying"
            );
            self.seen.borrow_mut().cut = Some(service.observer.submitted);
        }
        self.seen.borrow_mut().restarted = restarted;
        if matches!(self.source, SourceStage::Cut | SourceStage::Closing)
            && row.is_some_and(|row| matches!(row.state, MakingState::Failed(_)))
        {
            assert!(row.expect("failed row").copying, "failed copy retains its mark");
            assert!(self.transport.forge.borrow().domain.metadata(DESTINATION).is_ok(), "partial copy remains");
            assert!(self.transport.forge.borrow().domain.branch(DESTINATION, BRANCH).is_none());
            if self.source == SourceStage::Cut {
                peer_host.shutdown();
                self.source = SourceStage::Closing;
            } else if peer_host.is_empty() {
                let mut whole = peer(&self.transport, Fill::Whole, true);
                std::mem::swap(whole.setup_mut(), peer_host.setup_mut());
                **peer_host = SignalHost::new(whole, None);
                self.peer_seen = 0;
                self.source = SourceStage::Whole;
                copy(caller, b"retry-copy", false);
            }
        }
        self.seen.borrow_mut().adopted = acted(&store);
        if self.source == SourceStage::BesideRepository && self.answers == 6 && !self.stopped {
            self.beside_repository(caller, peer_host);
            self.shutdown(caller, peer_host);
            self.stopped = true;
        } else if self.source != SourceStage::BesideRepository
            && self.seen.borrow().adopted == 1
            && row.is_none()
            && !self.stopped
        {
            let adopted: Vec<_> = store
                .rows
                .values()
                .filter_map(|row| {
                    if let root::Record::Forge { row } = row
                        && let Stored::Repository(row) = row.as_ref()
                    {
                        Some(row)
                    } else {
                        None
                    }
                })
                .collect();
            assert_eq!(adopted.len(), 1, "one adopted repository");
            assert_eq!(adopted[0].project, 1);
            assert_eq!(adopted[0].owner.as_ref(), b"acme");
            assert_eq!(adopted[0].name.as_ref(), b"copied");
            assert_eq!(adopted[0].role, temper_engine_domain_forge::Role::Owned);
            assert_eq!(adopted[0].settings.default_branch.as_ref(), BRANCH);
            let forge = self.transport.forge.borrow();
            let source = forge.domain.branch(b"acme/app", BRANCH).expect("source head");
            let tip = forge.domain.branch(DESTINATION, BRANCH).expect("copy head");
            assert_eq!(forge.domain.object(tip).expect("workflow commit").parent, Some(source));
            self.shutdown(caller, peer_host);
            self.stopped = true;
        }
        store.pending.clear();
    }
    fn observe(&mut self, _: Time, _: &[Proc]) {}
    fn next_deadline(&self) -> Option<Time> {
        (!self.stopped).then_some(DEADLINE)
    }
    fn overdue(&self, now: Time) -> Option<String> {
        (!self.stopped && now >= DEADLINE).then(|| {
            format!(
                "copy unfinished: {:?}; making {:?}; ctl answers {}",
                self.seen.borrow(),
                making(&self.store.borrow()),
                self.answers
            )
        })
    }
    fn passed(&self) -> bool {
        self.stopped
    }
}

fn run(fill: Fill, cut: Option<u32>, stage: SourceStage) -> Outcome {
    stories::set(Story::Chat { words: b"Hello" });
    stories::setup_settings(world::Phase::Story);
    let transport = source(stage == SourceStage::BesideRepository);
    let store = Rc::new(RefCell::new(Store::new()));
    STARTS.with(|starts| {
        *starts.borrow_mut() = Starts { lives: 0, store: Some(store.clone()), transport: Some(transport.clone()) }
    });
    world::lower_store(Some(store.clone()));
    let mut disk = temper_playground::machine::fresh();
    let setup =
        disk.lay(&[skein_fake_machine::Item::file(b"admin.token", crate::restart_control::ADMIN_TOKEN).mode(0o600)]);
    disk.close(setup);
    let client = disk.reopen_root(2);
    let machine = Machine::new(disk);
    let seen = Rc::new(RefCell::new(Outcome::default()));
    let parameters = world::parameters(&machine, transport.clone());
    let outcome = programs::with(parameters, || {
        let mut world = skein_world::World::new_controlled(
            fixtures::SEED,
            skein_sim::Config::calm(),
            skein_world::Memory::Unchecked,
            |controls| Judge {
                controls,
                store,
                transport: transport.clone(),
                seen: seen.clone(),
                source: stage,
                handed: false,
                answers: 0,
                peer_seen: 0,
                stopped: false,
                last_commit: 0,
            },
        )
        .with_machine(machine);
        for (program, roots) in programs::programs::<CopyWorld>() {
            world.host_roots(program, roots);
        }
        let limits = temper_client_shell_cli::limits::native();
        world.host_roots(
            skein_world::HostedProgram {
                program: crate::ctl::PROGRAM.into(),
                make: |spawn, inherited| Proc::Other(HarnessProcess::Ctl(Box::new(crate::ctl::host(spawn, inherited)))),
                instances: 1,
                operations: skein_io::operations(&limits.io).expect("ctl operations") * 2 + 5,
            },
            crate::ctl::roots,
        );
        world.spawn_restartable(programs::service_roots(), programs::service::<CopyWorld>);
        world.spawn_root(skein_sim::Handle::new(client.raw()), |root| {
            Proc::Other(HarnessProcess::Calls(Box::new(Caller::scripted(root))))
        });
        world.spawn(|| Proc::Forge(Box::new(SignalHost::new(peer(&transport, fill, false), None))));
        if let Some(before) = cut {
            world.cut(0, before, skein_world::Cut::Kill);
        }
        world.run()
    });
    assert!(outcome.procs.iter().all(Host::is_empty), "copy world drains every process");
    world::lower_store(None);
    STARTS.with(|starts| *starts.borrow_mut() = Starts::default());
    Rc::try_unwrap(seen).expect("released copy referee").into_inner()
}

/// Kill the engine under a slow source, retaining the peer and the disk.
#[must_use]
pub fn restart() -> Outcome {
    let measured = run(Fill::Slow(SLOW), None, SourceStage::Slow);
    run(Fill::Slow(SLOW), Some(measured.cut.expect("migration cut point")), SourceStage::Slow)
}

/// Give the same source first a cut transfer, then a whole transfer.
#[must_use]
pub fn cut_short() -> Outcome {
    run(Fill::Cut, None, SourceStage::Cut)
}

/// Copy through ctl into the first repository's project, preserving the source workflow.
#[must_use]
pub fn beside_repository() -> Outcome {
    run(Fill::Whole, None, SourceStage::BesideRepository)
}
