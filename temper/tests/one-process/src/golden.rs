//! The once-made baseline deployment (temper/docs/design/protocol/baseline.md, 6).
pub(crate) mod files;
use crate::{
    fixtures,
    referee::{self, Store},
    stories,
    world::{self, HarnessProcess, Person, ServiceObserver, Story},
};
use jig_domain_core::{CoreRecord, Record};
use jig_domain_tasks::{self as tasks, TaskRecord};
use skein_fake_person::{Find, Step};
use skein_io::kernel::{Fd, ServiceSignal};
use skein_lib::{Duration, Time};
use skein_shell::Host;
use std::{cell::RefCell, path::Path, rc::Rc};
use temper_client_domain::Address;
use temper_engine_domain::Record as Row;
use temper_playground::{
    Links,
    machine::{self, Machine, SignalHost},
    programs::{self, Harness},
    service::Observe,
};

struct Golden;
type Proc = programs::Process<Golden>;
impl Harness for Golden {
    type Service = Observer;
    type Process = HarnessProcess;
    fn seeds(_: u64) -> temper_service::settings::Seeds {
        fixtures::seeds()
    }
    fn agent_seed(_: u64) -> u64 {
        fixtures::SEED
    }
    fn files() -> temper_service::settings::Files {
        fixtures::files()
    }
}
thread_local! { static PATCHED: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) }; }
struct Observer {
    inner: ServiceObserver,
}
impl Observe for Observer {
    fn started(root: Fd, inherited: &skein_world::Inherited) -> Self {
        Self { inner: ServiceObserver::started(root, inherited) }
    }
    fn complete(
        &mut self,
        complete: skein_io::kernel::Complete,
        service: &mut temper_service::Service,
    ) -> Option<skein_io::kernel::Complete> {
        self.inner.complete(complete, service)
    }
    fn before_iterate(&mut self, service: &mut temper_service::Service, now: Time, wall: skein_lib::Wall) -> bool {
        while let Some(note) = service.notes().pop() {
            if let temper_service::Note::Patched { of, patch, counts } = note {
                let owner = match of {
                    jig_protocol_store::Owner::Skein => "skein",
                    jig_protocol_store::Owner::Smith => "smith",
                    jig_protocol_store::Owner::Jig => "jig",
                    jig_protocol_store::Owner::Application => "temper",
                };
                PATCHED.with(|lines| {
                    lines.borrow_mut().push(format!("{owner} {patch} {} {}\n", counts.records, counts.commits));
                });
            }
        }
        self.inner.before_iterate(service, now, wall)
    }
    fn submitted(&mut self, submit: &skein_io::kernel::Submit) {
        self.inner.submitted(submit);
    }
    fn after_iterate(&mut self, service: &mut temper_service::Service, submitted: u32) {
        self.inner.after_iterate(service, submitted);
    }
    fn handover_read(
        &mut self,
        _: &temper_service::HandoverRead,
    ) -> Option<Result<Option<skein_lib::Secret>, temper_service::HandoverUnread>> {
        None
    }
    fn drain(&mut self, service: &mut temper_service::Service, now: Time) {
        self.inner.drain(service, now);
    }
}

fn see(text: &[u8]) -> Step<Address> {
    Step::See { find: Find::text(text), within: Duration::from_secs(60) }
}
fn new_chat(words: &[u8]) -> Vec<Step<Address>> {
    use skein_ui::Role;
    vec![
        Step::Go { address: Address::NewChat { draft: false } },
        Step::Type { find: Find::named(Role::TextArea, b"What do you want done?"), words: words.into() },
        Step::Press { find: Find::named(Role::Button, b"Send") },
        see(words),
    ]
}
pub(crate) fn person_steps() -> Box<[Step<Address>]> {
    let (mut steps, replies) = stories::playground::person();
    steps.truncate(usize::try_from(replies[0] + 1).expect("first reply step"));
    steps.insert(7, Step::Go { address: Address::NewChat { draft: false } });
    steps.extend(new_chat(b"Golden coding"));
    steps.into_boxed_slice()
}
pub(crate) fn llm_scripts() -> serde_json::Value {
    let mut scripts: serde_json::Value =
        serde_json::from_slice(temper_playground::stories::A.llm).expect("story A scripts");
    scripts["pace"]["reply_ms"] = 0.into();
    let mut model = scripts["models"][0].clone();
    model["name"] = "fake-key".into();
    scripts["models"].as_array_mut().expect("models").push(model);
    let mut coding = scripts["asked"][0].clone();
    coding["cue"] = "Golden coding".into();
    coding["turns"][0]["lines"][0]["text"] = "A second change is running.".into();
    coding["turns"][0]["lines"][1]["arguments"]["batch"][0]["spec"]["words"] = "Golden coding in flight".into();
    coding["turns"].as_array_mut().expect("turns").truncate(2);
    scripts["asked"].as_array_mut().expect("asked").push(coding);
    scripts["cued"].as_array_mut().expect("cued").push(serde_json::json!({
        "cue":"## Task\n\nGolden coding in flight",
        "turns":[
            {"lines":[{"call":"read","arguments":{"path":"src/main.rs"}}],"finish":"tool_calls","tokens":7},
            {"lines":[{"pieces":[{"text":"after stop","hold":"stop-mid-turn"}]}],"finish":"stop","tokens":1}
        ]
    }));
    scripts["control"] = serde_json::json!({"socket":"127.0.0.1:444"});
    scripts
}

pub(crate) fn llm_peer() -> skein_fake_peers::llm::Peer {
    let bytes = serde_json::to_vec(&llm_scripts()).expect("golden scripts");
    let document = temper_fake::llm::Document::decode(&bytes).expect("golden LLM document");
    temper_fake::llm::peer(temper_fake::llm::configuration(
        &document,
        temper_playground::documents::KEY,
        temper_playground::documents::seeds(fixtures::SEED).llm,
    ))
    .expect("golden LLM peer")
}

fn live(store: &Store) -> impl Iterator<Item = &TaskRecord> {
    store.rows.values().filter_map(|row| match row {
        Row::Core(Record::Tasks(tasks::Stored::Live(task))) => Some(task.as_ref()),
        _ => None,
    })
}
pub(crate) fn in_flight(store: &Store) -> Option<&TaskRecord> {
    live(store).find(|task| {
        task.spec.words.as_ref() == b"Golden coding in flight"
            && task.turn > 0
            && matches!(task.phase, tasks::Phase::Active(tasks::Active::Running { .. }))
    })
}
#[expect(
    clippy::struct_excessive_bools,
    reason = "the stop, hold release, upgrade, history and client observations are independent"
)]
struct Judge {
    controls: skein_world::Controls,
    store: Rc<RefCell<Store>>,
    forge: referee::forge::Forge,
    stopped: bool,
    released: bool,
    upgrade: bool,
    history_checked: bool,
    ticks: u32,
    client_started: bool,
}
impl Judge {
    fn stop_peers(&mut self, others: &mut [Proc]) {
        let ended = others.iter().all(|proc| !matches!(proc, Proc::Agent(agent) if !agent.is_empty()));
        for proc in others {
            match proc {
                Proc::Peer(peer) => {
                    if ended && !self.released {
                        peer.release(stories::STOP_HOLD).expect("stop hold release");
                        self.released = true;
                    }
                    peer.shutdown();
                }
                Proc::Forge(peer) => peer.shutdown(),
                _ => {}
            }
        }
    }
}
impl skein_world::Referee<Proc> for Judge {
    fn act(&mut self, _: Time, procs: &mut [Proc]) {
        self.ticks += 1;
        let (first, others) = procs.split_at_mut(1);
        let Proc::Service(service) = &mut first[0] else { panic!("service first") };
        assert!(self.ticks < 100_000, "golden story's bounded turns");
        if !self.upgrade
            && let Some(link) = service.service.links().pop()
        {
            for proc in others.iter_mut() {
                if let Proc::Other(HarnessProcess::Client(person)) = proc {
                    person.sign_in(&link);
                }
            }
        }
        if self.upgrade && !self.history_checked && self.store.borrow().wires.is_some_and(|number| number > 98) {
            assert_history(&self.store.borrow());
            self.history_checked = true;
        }
        if self.upgrade
            && self.history_checked
            && !self.client_started
            && self
                .store
                .borrow()
                .agent_records
                .iter()
                .any(|row| matches!(row, referee::AgentRecord::Start { task: 7, attempt, .. } if *attempt > 6))
        {
            for proc in others.iter_mut() {
                if let Proc::Other(HarnessProcess::Calls(caller)) = proc {
                    for args in [
                        vec![b"me".as_slice(), b"--json"],
                        vec![b"task".as_slice(), b"turn", b"1", b"1.1", b"--json"],
                        vec![b"task".as_slice(), b"turn", b"1", b"4.1", b"--json"],
                        vec![
                            b"chat".as_slice(),
                            b"start",
                            b"Golden after upgrade",
                            b"--project",
                            b"1",
                            b"--key",
                            b"golden-after-upgrade",
                            b"--json",
                        ],
                    ] {
                        caller.commands.push_back(crate::ctl::Command::new(&args));
                    }
                    caller.closed = true;
                }
            }
            self.client_started = true;
        }
        if self.upgrade {
            for proc in others.iter() {
                if let Proc::Other(HarnessProcess::Calls(caller)) = proc {
                    for answer in &caller.answers {
                        assert_eq!(
                            answer.exit,
                            skein_io::kernel::Exit::Code(0),
                            "ctl failed: {}",
                            String::from_utf8_lossy(&answer.error)
                        );
                    }
                }
            }
        }
        let complete = if self.upgrade {
            let store = self.store.borrow();
            self.history_checked
                && store
                    .agent_records
                    .iter()
                    .any(|row| matches!(row, referee::AgentRecord::Start { task: 7, attempt, .. } if *attempt > 6))
                && live(&store).any(|task| task.number > 7 && task.spec.words.as_ref() == b"Golden after upgrade")
                && others
                    .iter()
                    .any(|proc| matches!(proc, Proc::Other(HarnessProcess::Calls(caller)) if caller.answers.len() == 4))
        } else {
            self.forge.forge_complete() && in_flight(&self.store.borrow()).is_some()
        };
        if !self.stopped && complete {
            if self.upgrade {
                for proc in others.iter() {
                    if let Proc::Other(HarnessProcess::Calls(caller)) = proc {
                        assert_client(&caller.answers);
                    }
                }
            }
            for proc in others.iter_mut() {
                if let Proc::Other(HarnessProcess::Client(person)) = proc {
                    person.finish();
                }
            }
            self.controls.signal(0, ServiceSignal::Terminate);
            self.stopped = true;
        }
        if self.stopped {
            self.stop_peers(others);
        }
    }
    fn observe(&mut self, _: Time, procs: &[Proc]) {
        if !self.upgrade {
            self.forge.observe(procs);
        }
    }
    fn next_deadline(&self) -> Option<Time> {
        (!self.stopped).then_some(Time::from_nanos(60_000_000_000))
    }
    fn overdue(&self, now: Time) -> Option<String> {
        (!self.stopped && now >= Time::from_nanos(60_000_000_000)).then(|| {
            format!(
                "golden did not settle; checked {}, started {}, tasks {:?}",
                self.history_checked,
                self.client_started,
                live(&self.store.borrow()).map(|t| (t.number, t.attempt, &t.phase)).collect::<Vec<_>>()
            )
        })
    }
    fn passed(&self) -> bool {
        self.stopped
    }
}

fn assert_client(answers: &[crate::ctl::Answer]) {
    for answer in answers {
        assert_eq!(
            answer.exit,
            skein_io::kernel::Exit::Code(0),
            "current client: {}",
            String::from_utf8_lossy(&answer.error)
        );
    }
    let [me, first, last, _new] = answers else { panic!("me, two archived turns, and a new chat") };
    let me: serde_json::Value = serde_json::from_slice(&me.output).expect("current me JSON");
    assert_eq!(me["party"]["number"], 1, "the kept token still names person 1");
    assert_eq!(me["party"]["login"]["bytes"], "owner");
    assert!(
        me["projects"]
            .as_array()
            .expect("project memberships")
            .iter()
            .any(|project| project["project"] == 1 && project["name"]["bytes"] == "Demo")
    );
    assert!(
        skein_lib::bytes::find(&first.output, b"delegate this change in playground/widget").is_some(),
        "first kept chat turn read by current client"
    );
    assert!(
        skein_lib::bytes::find(&last.output, b"CSV export is available from widget").is_some(),
        "last kept chat turn read by current client"
    );
}

fn assert_history(store: &Store) {
    assert_eq!(store.installation, Some([9; 16]), "baseline installation id");
    let rows = &store.rows;
    assert!(
        rows.values().any(|row| matches!(row, Row::Core(Record::Core(CoreRecord::Deployment(id))) if id.id == [7; 16])),
        "baseline deployment id"
    );
    assert!(rows.values().any(|row| matches!(row, Row::Core(Record::People(jig_domain_people::Stored::Person { number: 1, identity })) if identity.login.as_ref() == b"owner" && identity.key.subject.as_ref() == b"owner-a")), "baseline person");
    assert!(rows.values().any(|row| matches!(row, Row::Core(Record::People(jig_domain_people::Stored::Policy { project: 1, name, .. })) if name.as_ref() == "Demo")), "baseline project");
    assert!(
        rows.values().any(|row| matches!(
            row,
            Row::Core(Record::Accounts(jig_domain_accounts::Stored::Account {
                number: 3,
                provider: 2,
                holder: jig_domain_accounts::Holder::Deployment,
                place: 1,
                ..
            }))
        )),
        "added account"
    );
    assert!(
        rows.values().any(|row| matches!(
            row,
            Row::Core(Record::Accounts(jig_domain_accounts::Stored::Source {
                account: 3,
                number: 2,
                kind: jig_domain_accounts::Kind::Key,
                ..
            }))
        )),
        "added account source"
    );
    assert!(rows.values().any(|row| matches!(row, Row::Forge { row } if matches!(row.as_ref(), temper_engine_domain_forge::Stored::Repository(repo) if repo.project == 1 && repo.provider.forge == 1 && repo.provider.repository == 1 && repo.owner.as_ref() == b"playground" && repo.name.as_ref() == b"widget"))), "adopted repository");
    assert!(rows.values().any(|row| matches!(row, Row::Core(Record::Tasks(tasks::Stored::Ended(task))) if task.number == 2 && matches!(task.phase, tasks::Phase::Ended(_)))), "landed change task ended");
    let commit =
        [29, 11, 20, 228, 219, 1, 143, 237, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    assert!(rows.values().any(|row| matches!(row, Row::Forge { row } if matches!(row.as_ref(), temper_engine_domain_forge::Stored::Landed { task: 2, commit: kept } if *kept == commit))), "landing marker");
    assert!(rows.values().any(|row| matches!(row, Row::Forge { row } if matches!(row.as_ref(), temper_engine_domain_forge::Stored::LandingLog(log) if log.landings.iter().any(|landing| landing.commit == commit && landing.branch.as_ref() == b"main" && landing.changes.iter().any(|change| change.task == 2 && change.pull == Some(1)))))), "landing recorded");
    assert!(rows.values().any(|row| matches!(row, Row::Core(Record::Core(CoreRecord::Terminal(end))) if end.task == 7 && end.attempt == 6 && end.end == tasks::End::Recheck)), "stopped coding attempt crossed");
    assert!(
        live(store).any(|task| task.number == 1 && task.phase == tasks::Phase::Active(tasks::Active::Idle)),
        "replied chat waits"
    );
}

pub(crate) fn configuration(deployment: &[u8]) -> Box<[u8]> {
    let mut document: serde_json::Value = serde_json::from_slice(deployment).expect("deployment");
    document["engine"]["limits"]["overrides"]["people.inbox_tasks"] = 16.into();
    let mut provider = document["engine"]["llm"]["providers"][0].clone();
    provider["number"] = 2.into();
    provider["name"] = "golden-provider".into();
    provider["models"] = serde_json::json!(["fake-key"]);
    let catalogue: serde_json::Value =
        serde_json::from_slice(include_bytes!("../../../crates/temper-playground/stories/a/catalogue.json"))
            .expect("story catalogue");
    document["engine"]["llm"]["models"] = catalogue["models"].clone();
    let mut model = catalogue["models"][0].clone();
    model["name"] = "fake-key".into();
    document["engine"]["llm"]["models"].as_array_mut().expect("models").push(model);
    document["engine"]["llm"]["providers"][0]["models"] = serde_json::json!(["fake"]);
    document["engine"]["llm"]["providers"].as_array_mut().expect("providers").push(provider);
    for name in ["tasks.tasks", "tasks.project_tasks"] {
        document["engine"]["limits"]["overrides"][name] = 8.into();
    }
    serde_json::to_vec(&document).expect("golden configuration").into()

}

/// An ordinary world story; the caller supplies only its test's temporary directory.
pub fn make(directory: &Path) {
    PATCHED.with(|lines| lines.borrow_mut().clear());
    stories::set(Story::PlaygroundA);
    stories::setup_settings(world::Phase::Story);
    let mut start = temper_playground::stories::A.start(fixtures::SEED, skein_sim::Config::calm().wall);
    start.deployment = configuration(&start.deployment);
    let document = serde_json::from_slice(&start.deployment).expect("golden configuration JSON");
    let mut disk = machine::fresh();
    temper_playground::story::prepare(&mut disk, temper_playground::stories::A.deployment, start.seed);
    let store = Rc::new(RefCell::new(Store::new()));
    world::lower_store(Some(store.clone()));
    let client = disk.reopen_root(2);
    let machine = Machine::new(disk);
    let params = programs::Parameters {
        deployment: start.deployment.clone(),
        seed: start.seed,
        links: Links::Keep,
        forge: start.fakes.forge.clone(),
        route: temper_playground::story::route(&document),
        disk: machine.0.clone(),
    };
    let outcome = programs::with(params, || {
        let mut world = skein_world::World::new_controlled(
            fixtures::SEED,
            skein_sim::Config::calm(),
            skein_world::Memory::Unchecked,
            |controls| Judge {
                controls,
                store: store.clone(),
                forge: referee::forge::Forge::new(start.fakes.forge.clone(), Story::PlaygroundA),
                stopped: false,
                released: false,
                upgrade: false,
                history_checked: false,
                ticks: 0,
                client_started: false,
            },
        )
        .with_machine(machine);
        for (program, roots) in programs::programs::<Golden>() {
            world.host_roots(program, roots);
        }
        world.spawn_restartable(programs::service_roots(), programs::service::<Golden>);
        world.spawn_root(skein_sim::Handle::new(client.raw()), |root| {
            Proc::Other(HarnessProcess::Client(Box::new(SignalHost::new(
                Person::welcome(root, person_steps()).with_account_after(7),
                None,
            ))))
        });
        world.spawn(|| Proc::Peer(Box::new(SignalHost::new(llm_peer(), None))));
        world.spawn(|| Proc::Forge(Box::new(SignalHost::new(start.fakes.peer, None))));
        world.run()
    });
    let Proc::Service(service) = &outcome.procs[0] else { panic!("service first") };
    assert!(service.service.is_empty(), "settled service stop");
    assert!(store.borrow().pending.is_empty(), "no pending commits at stop");
    assert_history(&store.borrow());
    files::export(&mut outcome.machine.0.borrow_mut(), directory, &start.deployment);
    world::lower_store(None);
}

/// Load the permanent baseline into a fresh machine and use this build's service.
pub fn read(directory: &Path) {
    stories::set(Story::PlaygroundA);
    stories::setup_settings(world::Phase::Story);
    PATCHED.with(|lines| lines.borrow_mut().clear());
    let start = temper_playground::stories::A.start(fixtures::SEED, skein_sim::Config::calm().wall);
    let configuration = std::fs::read(directory.join("configuration.json")).expect("kept startup configuration");
    let document = serde_json::from_slice(&configuration).expect("kept configuration JSON");
    let mut disk = files::load(directory);
    // The terminal keeps its token; ctl additionally reads the engine's address.
    machine::lay_handover(&mut disk, b"client/private/address", format!("http://{}\n", fixtures::CLIENTS).as_bytes());
    let client = disk.reopen_root(2);
    let machine = Machine::new(disk);
    let store = Rc::new(RefCell::new(Store::new()));
    store.borrow_mut().recovering();
    files::transcripts(directory, &mut store.borrow_mut());
    world::lower_store(Some(store.clone()));
    let params = programs::Parameters {
        deployment: configuration.into(),
        seed: start.seed,
        links: Links::Keep,
        forge: start.fakes.forge.clone(),
        route: temper_playground::story::route(&document),
        disk: machine.0.clone(),
    };
    let outcome = programs::with(params, || {
        let mut world = skein_world::World::new_controlled(
            fixtures::SEED,
            skein_sim::Config::calm(),
            skein_world::Memory::Unchecked,
            |controls| Judge {
                controls,
                store: store.clone(),
                forge: referee::forge::Forge::new(start.fakes.forge.clone(), Story::PlaygroundA),
                stopped: false,
                released: false,
                upgrade: true,
                history_checked: false,
                ticks: 0,
                client_started: false,
            },
        )
        .with_machine(machine);
        for (program, roots) in programs::programs::<Golden>() {
            world.host_roots(program, roots);
        }
        world.spawn_restartable(programs::service_roots(), programs::service::<Golden>);
        world.host_roots(
            crate::ctl::program(|spawn, inherited| {
                Proc::Other(HarnessProcess::Ctl(Box::new(crate::ctl::host(spawn, inherited))))
            }),
            crate::ctl::roots,
        );
        world.spawn_root(skein_sim::Handle::new(client.raw()), |root| {
            Proc::Other(HarnessProcess::Calls(Box::new(crate::ctl::Caller::scripted(root))))
        });
        world.spawn(|| Proc::Peer(Box::new(SignalHost::new(llm_peer(), None))));
        world.spawn(|| Proc::Forge(Box::new(SignalHost::new(start.fakes.peer, None))));
        world.run()
    });
    let Proc::Service(service) = &outcome.procs[0] else { panic!("service first") };
    assert!(service.service.is_empty());
    let expected = std::fs::read_to_string(directory.join("patched.txt")).expect("per-patch table");
    PATCHED.with(|lines| assert_eq!(lines.borrow().concat(), expected, "one reviewed line per applied patch"));
    world::lower_store(None);
}
