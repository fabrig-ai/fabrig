//! The once-made floor deployment (temper/docs/design/protocol/floor.md, 5).
pub(crate) mod files;
use crate::{
    ctl, fixtures,
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
use std::{cell::RefCell, fmt::Write as _, path::Path, rc::Rc};
use temper_client_domain::Address;
use temper_engine_domain::Record as Row;
use temper_playground::{
    Links,
    machine::{self, Machine, SignalHost},
    programs::{self, Harness},
    service::Observe,
};

thread_local! { static UPGRADED: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) }; }

const CHAT_REPLY: &str =
    "CSV export is available from widget’s table. The change landed on Playground forge. This was a scripted change.";

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
            let (of, line) = match note {
                temper_service::Note::Engine(temper_engine_service::Note::Ran { of, job, rows, commits, counts }) => {
                    let mut text = String::new();
                    for count in &counts {
                        write!(text, " {}={}", count.name, count.count).expect("string write");
                    }
                    (of, format!("{job} {rows} {commits}{text}\n"))
                }
                _ => continue,
            };
            let owner = match of {
                jig_protocol_store::Owner::Skein => "skein",
                jig_protocol_store::Owner::Smith => "smith",
                jig_protocol_store::Owner::Jig => "jig",
                jig_protocol_store::Owner::Application => "temper",
            };
            UPGRADED.with(|lines| lines.borrow_mut().push(format!("{owner} {line}")));
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
#[derive(Clone, Copy, PartialEq, Eq)]
enum Opening {
    Make,
    Read,
}
struct Judge {
    finding: Option<String>,
    controls: skein_world::Controls,
    store: Rc<RefCell<Store>>,
    forge: referee::forge::Forge,
    stopped: bool,
    released: bool,
    opening: Opening,
    checked: bool,
    ticks: u32,
}
impl Judge {
    fn stop_peers(&mut self, others: &mut [Proc]) -> Result<(), String> {
        let ended = others.iter().all(|proc| !matches!(proc, Proc::Agent(agent) if !agent.is_empty()));
        for proc in others {
            match proc {
                Proc::Peer(peer) => {
                    if ended && !self.released {
                        peer.release(stories::STOP_HOLD).map_err(|why| format!("stop hold release: {why:?}"))?;
                        self.released = true;
                    }
                    peer.shutdown();
                }
                Proc::Forge(peer) => peer.shutdown(),
                _ => {}
            }
        }

        Ok(())
    }
}
impl Judge {
    fn act_checked(&mut self, _: Time, procs: &mut [Proc]) -> Result<(), String> {
        self.ticks += 1;
        let (first, others) = procs.split_at_mut(1);
        let Proc::Service(service) = &mut first[0] else { return Err(String::from("service first")) };
        if self.ticks >= 100_000 {
            return Err(String::from("golden story's bounded turns"));
        }
        if self.opening == Opening::Make
            && let Some(link) = service.service.links().pop()
        {
            for proc in others.iter_mut() {
                if let Proc::Other(HarnessProcess::Client(person)) = proc {
                    person.sign_in(&link);
                }
            }
        }
        if self.opening == Opening::Read
            && !self.checked
            && service.service.wait_state().engine.is_some_and(|state| state.mark_durable)
        {
            assert_history(&self.store.borrow())?;
            restart_history(&self.store.borrow())?;
            for proc in others.iter_mut() {
                if let Proc::Other(HarnessProcess::Calls(caller)) = proc {
                    caller.commands.push_back(ctl::Command::new(&[b"me", b"--json"]));
                    caller.commands.push_back(ctl::Command::new(&[b"chat", b"show", b"1", b"--json"]));
                    caller.commands.push_back(ctl::Command::new(&[b"task", b"show", b"2", b"--json"]));
                    caller.closed = true;
                }
            }
            self.checked = true;
        }
        let complete = match self.opening {
            Opening::Make => self.forge.forge_complete() && in_flight(&self.store.borrow()).is_some(),
            Opening::Read => {
                if !self.stopped
                    && let Some(end) = service.service.end()
                {
                    return Err(format!("floor start refused: {end:?}"));
                }
                others
                    .iter()
                    .any(|proc| matches!(proc, Proc::Other(HarnessProcess::Calls(caller)) if caller.answers.len() == 3))
            }
        };
        if !self.stopped && complete {
            for proc in others.iter_mut() {
                if let Proc::Other(HarnessProcess::Client(person)) = proc {
                    person.finish();
                }
            }
            self.controls.signal(0, ServiceSignal::Terminate);
            self.stopped = true;
        }
        if self.stopped {
            self.stop_peers(others)?;
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
            if let Proc::Service(service) = proc
                && let Some(why) = &service.observer.inner.finding
            {
                return Err(why.clone());
            }
        }

        if self.opening == Opening::Make {
            self.forge.observe(procs)?;
        }

        Ok(())
    }
    fn next_deadline(&self) -> Option<Time> {
        (!self.stopped).then_some(Time::from_nanos(60_000_000_000))
    }
    fn overdue(&self, now: Time) -> Option<String> {
        (!self.stopped && now >= Time::from_nanos(60_000_000_000)).then(|| {
            format!(
                "golden did not settle; tasks {:?}",
                live(&self.store.borrow()).map(|t| (t.number, t.attempt, &t.phase)).collect::<Vec<_>>()
            )
        })
    }
    fn passed(&self) -> bool {
        self.finding.is_none() && (self.stopped)
    }
}

fn assert_history(store: &Store) -> Result<(), String> {
    match (&(store.installation), &(Some([9; 16]))) {
        (left, right) if left != right => {
            let finding = String::from("floor installation id");
            return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
        }
        _ => {}
    }
    let rows = &store.rows;
    if !(rows
        .values()
        .any(|row| matches!(row, Row::Core(Record::Core(CoreRecord::Deployment(id))) if id.id == [7; 16])))
    {
        return Err(String::from("floor deployment id"));
    }
    if !(rows.values().any(|row| matches!(row, Row::Core(Record::People(jig_domain_people::Stored::Person { number: 1, identity })) if identity.login.as_ref() == b"owner" && identity.key.subject.as_ref() == b"owner-a"))) { return Err(String::from("floor person")); }
    if !(rows.values().any(|row| matches!(row, Row::Core(Record::People(jig_domain_people::Stored::Policy { project: 1, name, .. })) if name.as_ref() == "Demo"))) { return Err(String::from("floor project")); }
    if !(rows.values().any(|row| {
        matches!(
            row,
            Row::Core(Record::Accounts(jig_domain_accounts::Stored::Account {
                number: 3,
                provider: 2,
                holder: jig_domain_accounts::Holder::Deployment,
                place: 1,
                ..
            }))
        )
    })) {
        return Err(String::from("added account"));
    }
    if !(rows.values().any(|row| {
        matches!(
            row,
            Row::Core(Record::Accounts(jig_domain_accounts::Stored::Source {
                account: 3,
                number: 2,
                kind: jig_domain_accounts::Kind::Key,
                ..
            }))
        )
    })) {
        return Err(String::from("added account source"));
    }
    if !(rows.values().any(|row| matches!(row, Row::Forge { row } if matches!(row.as_ref(), temper_engine_domain_forge::Stored::Repository(repo) if repo.project == 1 && repo.provider.forge == 1 && repo.provider.repository == 1 && repo.owner.as_ref() == b"playground" && repo.name.as_ref() == b"widget")))) { return Err(String::from("adopted repository")); }
    if !(rows.values().any(|row| matches!(row, Row::Core(Record::Tasks(tasks::Stored::Ended(task))) if task.number == 2 && matches!(task.phase, tasks::Phase::Ended(_))))) { return Err(String::from("landed change task ended")); }
    let task = rows
        .values()
        .find_map(|row| match row {
            Row::Core(Record::Tasks(tasks::Stored::Ended(task))) if task.number == 2 => Some(task),
            _ => None,
        })
        .ok_or_else(|| String::from("kept ended task"))?;
    if task.session.is_some() {
        return Err(String::from("kept task has no session"));
    }
    let charter = rows
        .values()
        .find_map(|row| {
            let Row::Core(Record::Core(CoreRecord::Settings(jig_domain_core::settings::Record::Default {
                setting: jig_domain_core::settings::Setting::Charter(1),
                default,
            }))) = row
            else {
                return None;
            };
            let jig_domain_core::settings::Value::Charter(charter) = &default.value else { return None };
            Some(charter)
        })
        .ok_or_else(|| String::from("charter model"))?;
    match (&(charter.policy.model.cache_life_ms), &(0)) {
        (left, right) if left != right => {
            let finding = String::from("model declares no cache life");
            return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
        }
        _ => {}
    }
    let rule = rows
        .values()
        .find_map(|row| {
            let Row::Forge { row } = row else { return None };
            let temper_engine_domain_forge::Stored::Setting(setting) = row.as_ref() else { return None };
            let temper_engine_domain_forge::settings::Edit::Set(value) = &setting.edit else { return None };
            let temper_engine_domain_forge::settings::Value::Rule(rule) = &value.value else { return None };
            Some(rule)
        })
        .ok_or_else(|| String::from("kept landing rule"))?;
    if !(rule.sets.is_empty()) {
        return Err(String::from("kept landing rule names no set"));
    }
    let commit =
        [29, 11, 20, 228, 219, 1, 143, 237, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    if !(rows.values().any(|row| matches!(row, Row::Forge { row } if matches!(row.as_ref(), temper_engine_domain_forge::Stored::Landed { task: 2, commit: kept } if *kept == commit)))) { return Err(String::from("landing marker")); }
    if !(rows.values().any(|row| matches!(row, Row::Forge { row } if matches!(row.as_ref(), temper_engine_domain_forge::Stored::LandingLog(log) if log.landings.iter().any(|landing| landing.commit == commit && landing.branch.as_ref() == b"main" && landing.changes.iter().any(|change| change.task == 2 && change.pull == Some(1))))))) { return Err(String::from("landing recorded")); }
    if !(rows.values().any(|row| matches!(row, Row::Core(Record::Core(CoreRecord::Terminal(end))) if end.task == 7 && end.attempt == 6 && end.end == tasks::End::Recheck))) { return Err(String::from("stopped coding attempt crossed")); }
    if !(live(store).any(|task| task.number == 1 && task.phase == tasks::Phase::Active(tasks::Active::Idle))) {
        return Err(String::from("replied chat waits"));
    }

    Ok(())
}

pub(crate) fn configuration(
    deployment: &temper_playground::documents::Configuration,
) -> temper_playground::documents::Configuration {
    let mut document = deployment.clone();
    document.jig_over("people.inbox_tasks", 16);
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
        document.jig_over(name, 8);
    }
    document
}

/// An ordinary world story; the caller supplies only its test's temporary directory.
pub fn make(directory: &Path) {
    stories::set(Story::PlaygroundA);
    stories::setup_settings(world::Phase::Story);
    let mut start = temper_playground::stories::A.start(fixtures::SEED, skein_sim::Config::calm().wall);
    start.deployment = configuration(&start.deployment);
    let document = start.deployment.clone();
    let mut disk = machine::fresh();
    temper_playground::story::prepare(&mut disk, &start.deployment, start.seed);
    let store = Rc::new(RefCell::new(Store::new()));
    world::lower_store(Some(store.clone()));
    let client = disk.reopen_root(2);
    let machine = Machine::new(disk);
    let params = programs::Parameters {
        deployment: start.deployment.clone().into(),
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
                finding: None,
                controls,
                store: store.clone(),
                forge: referee::forge::Forge::new(start.fakes.forge.clone(), Story::PlaygroundA),
                stopped: false,
                released: false,
                opening: Opening::Make,
                checked: false,
                ticks: 0,
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
    assert_history(&store.borrow()).expect("golden history");
    files::export(&mut outcome.machine.0.borrow_mut(), directory, &start.deployment);
    world::lower_store(None);
}

// A refused open has no commits or running neighbours to observe.
struct Refusal {
    stopped: bool,
    ticks: u32,
}
thread_local! {
    static READ_DEFAULTS: RefCell<Option<temper_service::stock::Given>> = const { RefCell::new(None) };
}
impl Harness for Refusal {
    type Service = ();
    type Process = programs::Never;
    fn seeds(seed: u64) -> temper_service::settings::Seeds {
        Golden::seeds(seed)
    }
    fn files() -> temper_service::settings::Files {
        Golden::files()
    }
    fn given(_: &temper_playground::documents::Configuration) -> temper_service::stock::Given {
        READ_DEFAULTS.with(|defaults| defaults.borrow_mut().take().expect("one floorless open"))
    }
}
impl skein_world::Referee<programs::Process<Refusal>> for Refusal {
    fn watch(&mut self, _: Time, procs: &[programs::Process<Self>]) -> Result<(), String> {
        self.ticks += 1;
        if self.ticks >= 100_000 {
            return Err(String::from("golden open's bounded turns"));
        }
        for proc in procs {
            if let programs::Process::Service(service) = proc
                && service.service.wait_state().engine.is_some_and(|state| state.mark_durable)
            {
                return Err(String::from("floorless start passed the floor"));
            }
        }
        self.stopped = procs.iter().all(Host::is_empty);
        Ok(())
    }
    fn next_deadline(&self) -> Option<Time> {
        (!self.stopped).then_some(Time::from_nanos(60_000_000_000))
    }
    fn overdue(&self, now: Time) -> Option<String> {
        (!self.stopped && now >= Time::from_nanos(60_000_000_000)).then(|| String::from("golden open did not settle"))
    }
    fn passed(&self) -> bool {
        self.stopped
    }
}

/// The permanent floor registers later layouts and serves the kept person.
pub fn read(directory: &Path) {
    stories::set(Story::PlaygroundA);
    stories::setup_settings(world::Phase::Story);
    UPGRADED.with(|lines| lines.borrow_mut().clear());
    let start = temper_playground::stories::A.start(fixtures::SEED, skein_sim::Config::calm().wall);
    let configuration = read_configuration(directory);
    let mut document = configuration;
    // The reader holds the playground reset while inspecting the permanent history.
    document["world_settings"] = temper_playground::documents::world_document()["world_settings"].clone();
    document["world_settings"]["adopt_only"] = true.into();
    let configuration = document.clone();
    let mut disk = files::load(directory);
    // The pre-N5 floor has no setup file; supply the operator's prepare step to this copy.
    let directories =
        jig_protocol_config::Directories::given(b"/config", b"/setup", b"/cache").expect("world directories");
    let setup = temper_forge_setup::setup_file::chosen(
        temper_forge_setup::machine::BoxKind::Fake,
        temper_playground::documents::FORGE.port(),
        b"owner",
        false,
        &directories,
    )
    .expect("kept fake forge setup");
    let setup_path = directories
        .join(jig_protocol_config::Directory::Data, &temper_forge_setup::setup_file::path(1, setup.kind()))
        .expect("world setup path");
    machine::lay_handover(&mut disk, &setup_path, &temper_forge_setup::setup_file::encode(&setup));
    let active = files::active(&mut disk);
    machine::lay_handover(&mut disk, b"client/private/address", format!("http://{}\n", fixtures::CLIENTS).as_bytes());
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
                finding: None,
                controls,
                store: store.clone(),
                forge: referee::forge::Forge::new(start.fakes.forge.clone(), Story::PlaygroundA),
                stopped: false,
                released: false,
                opening: Opening::Read,
                checked: false,
                ticks: 0,
            },
        )
        .with_machine(machine);
        for (program, roots) in programs::programs::<Golden>() {
            world.host_roots(program, roots);
        }
        world.spawn_restartable(programs::service_roots(), programs::service::<Golden>);
        world.host_roots(
            ctl::program(|spawn, inherited| Proc::Other(HarnessProcess::Ctl(Box::new(ctl::host(spawn, inherited))))),
            ctl::roots,
        );
        world.spawn_restartable(
            vec![skein_world::StartupRoot { name: b"private".as_slice().into(), path: b"client".as_slice().into() }],
            |inherited| Proc::Other(HarnessProcess::Calls(Box::new(ctl::Caller::scripted_restartable(inherited)))),
        );
        world.spawn(|| Proc::Peer(Box::new(SignalHost::new(llm_peer(), None))));
        world.spawn(|| Proc::Forge(Box::new(SignalHost::new(start.fakes.peer, None))));
        world.run()
    });
    let Proc::Service(service) = &outcome.procs[0] else { panic!("service first") };
    assert!(service.service.is_empty());
    assert_eq!(service.service.end(), Some(&temper_service::End::Engine(temper_engine_service::End::Drained)));
    let caller = outcome
        .procs
        .iter()
        .find_map(|proc| if let Proc::Other(HarnessProcess::Calls(caller)) = proc { Some(caller) } else { None })
        .expect("kept-token caller");
    assert_served(caller);
    assert_eq!(active, b"1\n", "the kept floor's generation");
    assert_eq!(
        files::active(&mut outcome.machine.0.borrow_mut()),
        b"2\n",
        "the current layouts register in one new generation"
    );
    let upgraded = std::fs::read_to_string(directory.join("upgraded.txt")).expect("kept upgrade events");
    let jobs = "jig escalation-deciders 4 2 tasks=0\ntemper test-ids-landing 0 1\ntemper test-ids-receipt 0 1 many=0\ntemper test-ids-tier 0 1 many=0 flakes=0 posted=0\ntemper test-ids-verification 0 1 many=0\n";
    assert_eq!(UPGRADED.with(|lines| lines.borrow().concat()), format!("{jobs}{upgraded}"));
    world::lower_store(None);
}

/// The floorless copy refuses before decoding, without switching generations.
pub fn refuse(directory: &Path) {
    stories::set(Story::PlaygroundA);
    stories::setup_settings(world::Phase::Story);
    let (configuration, defaults) = startup_configuration(directory);
    READ_DEFAULTS.with(|at| *at.borrow_mut() = Some(defaults));
    let mut disk = files::load(directory);
    files::change_record(&mut disk, &[0, 13], None);
    let before = files::image(&mut disk);
    let active = files::active(&mut disk);
    let machine = Machine::new(disk);
    // Parameters carries a git transport even though no program can run before this refusal.
    let config = temper_playground::documents::hosted_git::config();
    let forge =
        skein_fake_peers::forge::Forge::new(skein_fake_forge_domain::Domain::new(&config, fixtures::SEED), config);
    let params = programs::Parameters {
        deployment: configuration.into(),
        seed: fixtures::SEED,
        links: Links::Keep,
        forge: temper_fake_git_hosted::Typed::new(Rc::new(RefCell::new(forge)), &stories::forge_table()),
        route: None,
        disk: machine.0.clone(),
    };
    let outcome = programs::with(params, || {
        let mut world = skein_world::World::new(
            fixtures::SEED,
            skein_sim::Config::calm(),
            Refusal { stopped: false, ticks: 0 },
            skein_world::Memory::Unchecked,
        )
        .with_machine(machine);
        world.spawn_restartable(programs::service_roots(), programs::service::<Refusal>);
        world.run()
    });
    let programs::Process::Service(service) = &outcome.procs[0] else { panic!("service first") };
    assert!(service.service.is_empty());
    let why = jig_protocol_store::OpenRefusal::OlderBuild { why: jig_protocol_store::Older::Floor };
    assert_eq!(
        service.service.end(),
        Some(&temper_service::End::Engine(temper_service::EngineEnd::Stopped(temper_service::EngineStop::StoreOpen {
            why: why.clone()
        })))
    );
    assert!(why.words().join("").contains(jig_protocol_store::FLOOR_BRIDGE));
    files::refused(&before, &mut outcome.machine.0.borrow_mut());
    assert_eq!(active, b"1\n", "the kept floor's generation");
    assert_eq!(
        files::active(&mut outcome.machine.0.borrow_mut()),
        active,
        "refusal never switches the active generation"
    );
}

fn assert_served(caller: &ctl::Caller) {
    assert_eq!(caller.answers.len(), 3);
    for (index, answer) in caller.answers.iter().enumerate() {
        assert_eq!(
            answer.exit,
            skein_io::kernel::Exit::Code(0),
            "read {index}: {}",
            String::from_utf8_lossy(&answer.error)
        );
    }
    let identity: serde_json::Value = serde_json::from_slice(&caller.answers[0].output).expect("me JSON");
    assert_eq!(identity["party"]["number"], 1);
    assert_eq!(identity["party"]["login"]["bytes"], "owner");
    let chat: serde_json::Value = serde_json::from_slice(&caller.answers[1].output).expect("chat JSON");
    assert_eq!(chat["activity"]["cut"], false);
    assert!(chat["activity"]["before"].is_null());
    assert!(chat["activity"]["after"].is_null());
    let turns: Vec<_> = chat["activity"]["rows"]
        .as_array()
        .expect("chat activity")
        .iter()
        .filter_map(|row| {
            let name = row.pointer("/event/owner/decoded/name")?;
            Some((name["attempt"].as_u64().expect("attempt"), name["number"].as_u64().expect("turn")))
        })
        .collect();
    assert_eq!(turns, [(1, 1), (1, 2), (4, 1)], "kept chat turns served");
    assert!(std::str::from_utf8(&caller.answers[1].output).expect("chat UTF-8").contains(CHAT_REPLY));
    let task: serde_json::Value = serde_json::from_slice(&caller.answers[2].output).expect("task JSON");
    assert_eq!(task["chip"]["task"], 2);
    assert_eq!(task["chip"]["phase"], "ended");
    assert_eq!(task["chip"]["title"]["bytes"], "Export widget's table as CSV");
}

/// The kept world configuration with current admission bounds for its defaults.
pub(crate) fn read_configuration(directory: &Path) -> temper_playground::documents::Configuration {
    startup_configuration(directory).0
}

fn startup_configuration(
    directory: &Path,
) -> (temper_playground::documents::Configuration, temper_service::stock::Given) {
    let configuration = std::fs::read(directory.join("configuration.json")).expect("kept startup configuration");
    let document: serde_json::Value = serde_json::from_slice(&configuration).expect("kept configuration JSON");
    let mut document = if document.pointer("/engine/limits/profile").is_some_and(|profile| profile == "small") {
        bridge_kept_profiles(document)
    } else if document.pointer("/engine/limits").is_some() {
        temper_playground::documents::Configuration::from_jig(document)
    } else {
        let mut configured = self::configuration(&temper_playground::stories::A.configuration());
        configured.document = document;
        configured
    };
    temper_playground::documents::settings::strip_fixed(&mut document);
    document.jig.core.brief.sections = document.jig.core.brief.sections.max(jig_domain_core::CORE_SECTIONS);
    let given = temper_playground::documents::given(&document);
    let temper_service::stock::Given::Own { jig, .. } = &given else { panic!("golden world defaults") };
    for (name, count) in [
        ("tasks.charters", jig.charters.len()),
        ("core.templates", jig.templates.len()),
        ("authority.executors", jig.rules.ceiling.delegation.kinds.len()),
    ] {
        let count = u64::try_from(count).expect("configured default count");
        let row = jig_protocol_config::names::NAMES.iter().find(|row| row.name == name).expect("kept count limit");
        document.jig_over(name, (row.value)(&document.jig).max(count));
    }
    (document, given)
}

fn restart_history(store: &Store) -> Result<(), String> {
    let header =
        store
            .rows
            .values()
            .find_map(|row| {
                if let Row::Core(Record::Core(CoreRecord::Deployment(header))) = row { Some(header) } else { None }
            })
            .ok_or("kept header")?;
    if header.tasks != 7 || header.tasks + 1 != 8 || store.rows.values().any(|row| {
        matches!(row, Row::Core(Record::Tasks(tasks::Stored::Live(task) | tasks::Stored::Ended(task))) if task.number >= 8)
    }) { return Err(String::from("next task number 8 is above every kept task")); }
    let turns: Vec<_> = store
        .rows
        .values()
        .filter_map(|row| {
            if let Row::Core(Record::Core(CoreRecord::Turn(turn))) = row
                && turn.task == 1
            {
                Some(turn)
            } else {
                None
            }
        })
        .collect();
    let places: Vec<_> = turns.iter().map(|turn| (turn.attempt, turn.turn, turn.spent, turn.read)).collect();
    if places != [(1, 1, 13, None), (1, 2, 26, None), (4, 1, 14, Some(3))] {
        return Err(format!("paused chat's kept turns: {places:?}"));
    }
    if !turns.iter().any(|turn| skein_lib::bytes::find(&turn.transcript, CHAT_REPLY.as_bytes()).is_some()) {
        return Err(String::from("paused chat's reply read"));
    }
    if !live(store).any(|task| {
        task.number == 7
            && task.attempt == 6
            && task.turn == 1
            && task.phase == tasks::Phase::Active(tasks::Active::Due)
            && task.session.is_none()
    }) {
        return Err(String::from("signal-stopped coding task due at the same attempt on a plain reopen"));
    }
    Ok(())
}

/// The permanent floor keeps its bytes; only its working copy spells out
/// the fixture figures that its retired profile selected (protocol/floor.md, 5).
pub(crate) fn bridge_kept_profiles(mut document: serde_json::Value) -> temper_playground::documents::Configuration {
    let jig = jig_protocol_config_fixtures::small();
    let limits = document.pointer_mut("/workers/0/limits").expect("kept worker limits");
    assert_eq!(limits["profile"], "small");
    limits.as_object_mut().expect("worker limits").remove("profile");
    let overrides = limits["overrides"].as_object_mut().expect("worker overrides");
    for name in &jig_protocol_config::names::NAMES {
        if name.name != "views.watchers" && name.name != "views.watchers_per_party" {
            overrides.entry(name.name).or_insert_with(|| (name.value)(&jig).into());
        }
    }
    temper_playground::documents::Configuration::from_jig_over(document, &jig, &temper_limits_fixtures::small())
}
