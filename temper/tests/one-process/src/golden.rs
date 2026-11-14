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

// The start's copy uses this build's brief floor; kept bytes stay unchanged (baseline.md, 6).
const BRIEF_SECTIONS: u32 = jig_domain_core::CORE_SECTIONS;

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
            if let temper_service::Note::Engine(temper_engine_service::Note::Patched { of, patch, counts }) = note {
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
    finding: Option<String>,
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
    fn start_client(&mut self, others: &mut [Proc]) {
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
            assert_history(&self.store.borrow())?;
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
            self.start_client(others);
        }
        if self.upgrade {
            for proc in others.iter() {
                if let Proc::Other(HarnessProcess::Calls(caller)) = proc {
                    for answer in &caller.answers {
                        match (&(answer.exit), &(skein_io::kernel::Exit::Code(0))) {
                            (left, right) if left != right => {
                                let finding = format!("ctl failed: {}", String::from_utf8_lossy(&answer.error));
                                return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                            }
                            _ => {}
                        }
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
                && live(&store).any(|task| {
                    task.number > 7 && task.spec.words.as_ref() == b"Golden after upgrade"
                        && store.agent_records.iter().any(|row| {
                            matches!(row, referee::AgentRecord::Start { task: number, .. } if *number == task.number)
                        })
                })
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
                        assert_client(&caller.answers)?;
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

        if !self.upgrade {
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
                "golden did not settle; checked {}, started {}, tasks {:?}",
                self.history_checked,
                self.client_started,
                live(&self.store.borrow()).map(|t| (t.number, t.attempt, &t.phase)).collect::<Vec<_>>()
            )
        })
    }
    fn passed(&self) -> bool {
        self.finding.is_none() && (self.stopped)
    }
}

fn assert_client(answers: &[crate::ctl::Answer]) -> Result<(), String> {
    for answer in answers {
        match (&(answer.exit), &(skein_io::kernel::Exit::Code(0))) {
            (left, right) if left != right => {
                let finding = format!("current client: {}", String::from_utf8_lossy(&answer.error));
                return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
            }
            _ => {}
        }
    }
    let [me, first, last, _new] = answers else { return Err(String::from("me, two archived turns, and a new chat")) };
    let me: serde_json::Value =
        serde_json::from_slice(&me.output).map_err(|why| format!("current me JSON: {why:?}"))?;
    match (&(me["party"]["number"]), &(1)) {
        (left, right) if left != right => {
            let finding = String::from("the kept token still names person 1");
            return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
        }
        _ => {}
    }
    match (&(me["party"]["login"]["bytes"]), &("owner")) {
        (left, right) if left != right => {
            let finding = String::from("assertion `left == right` failed");
            return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
        }
        _ => {}
    }
    if !(me["projects"]
        .as_array()
        .ok_or_else(|| String::from("project memberships"))?
        .iter()
        .any(|project| project["project"] == 1 && project["name"]["bytes"] == "Demo"))
    {
        return Err(format!(
            "assertion failed: {}",
            stringify!(
                me["projects"]
                    .as_array()
                    .ok_or_else(|| String::from("project memberships"))?
                    .iter()
                    .any(|project| project["project"] == 1 && project["name"]["bytes"] == "Demo")
            )
        ));
    }
    if skein_lib::bytes::find(&first.output, b"delegate this change in playground/widget").is_none() {
        return Err(String::from("first kept chat turn read by current client"));
    }
    if skein_lib::bytes::find(&last.output, b"CSV export is available from widget").is_none() {
        return Err(String::from("last kept chat turn read by current client"));
    }

    Ok(())
}

fn assert_history(store: &Store) -> Result<(), String> {
    match (&(store.installation), &(Some([9; 16]))) {
        (left, right) if left != right => {
            let finding = String::from("baseline installation id");
            return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
        }
        _ => {}
    }
    let rows = &store.rows;
    if !(rows
        .values()
        .any(|row| matches!(row, Row::Core(Record::Core(CoreRecord::Deployment(id))) if id.id == [7; 16])))
    {
        return Err(String::from("baseline deployment id"));
    }
    if !(rows.values().any(|row| matches!(row, Row::Core(Record::People(jig_domain_people::Stored::Person { number: 1, identity })) if identity.login.as_ref() == b"owner" && identity.key.subject.as_ref() == b"owner-a"))) { return Err(String::from("baseline person")); }
    if !(rows.values().any(|row| matches!(row, Row::Core(Record::People(jig_domain_people::Stored::Policy { project: 1, name, .. })) if name.as_ref() == "Demo"))) { return Err(String::from("baseline project")); }
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
                finding: None,
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
    assert_history(&store.borrow()).expect("golden history");
    files::export(&mut outcome.machine.0.borrow_mut(), directory, &start.deployment);
    world::lower_store(None);
}

/// Load the permanent baseline into a fresh machine and use this build's service.
pub fn read(directory: &Path) {
    stories::set(Story::PlaygroundA);
    stories::setup_settings(world::Phase::Story);
    PATCHED.with(|lines| lines.borrow_mut().clear());
    let start = temper_playground::stories::A.start(fixtures::SEED, skein_sim::Config::calm().wall);
    let configuration = read_configuration(directory);
    let document: serde_json::Value = serde_json::from_slice(&configuration).expect("configuration JSON");
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
                finding: None,
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

/// The kept world configuration with current admission bounds for its defaults.
pub(crate) fn read_configuration(directory: &Path) -> Vec<u8> {
    let configuration = std::fs::read(directory.join("configuration.json")).expect("kept startup configuration");
    let mut document: serde_json::Value = serde_json::from_slice(&configuration).expect("kept configuration JSON");
    let sections =
        document.pointer_mut("/engine/limits/overrides/brief.sections").expect("kept brief section override");
    *sections = sections.as_u64().expect("brief section count").max(u64::from(BRIEF_SECTIONS)).into();
    let temper_service::stock::Given::Own { jig, .. } = temper_playground::documents::given(&document) else {
        panic!("golden world defaults")
    };
    for (name, count) in [
        ("tasks.charters", jig.charters.len()),
        ("core.templates", jig.templates.len()),
        ("authority.executors", jig.rules.ceiling.delegation.kinds.len()),
    ] {
        let count = u64::try_from(count).expect("configured default count");
        let bound = document["engine"]["limits"]["overrides"].get_mut(name).expect("kept count override");
        *bound = bound.as_u64().expect("count override").max(count).into();
    }
    serde_json::to_vec(&document).expect("startup configuration")
}
