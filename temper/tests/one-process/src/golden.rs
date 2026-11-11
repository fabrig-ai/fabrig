//! The once-made baseline deployment (temper/docs/design/protocol/baseline.md, 6).
mod files;
use crate::{fixtures, referee::{self, Store}, stories, world::{self, HarnessProcess, Person, ServiceObserver, Story}};
use jig_domain_core::Record;
use jig_domain_tasks::{self as tasks, TaskRecord};
use skein_fake_person::{Find, Step};
use skein_io::kernel::{Fd, ServiceSignal};
use skein_lib::{Duration, Time};
use skein_shell::Host;
use std::{cell::RefCell, path::Path, rc::Rc};
use temper_client_domain::Address;
use temper_engine_domain::Record as Row;
use temper_playground::{Links, machine::{self, Machine, SignalHost}, programs::{self, Harness}, service::Observe};

struct Golden;
type Proc = programs::Process<Golden>;
impl Harness for Golden {
    type Service = Observer;
    type Process = HarnessProcess;
    fn seeds(_: u64) -> temper_service::settings::Seeds { fixtures::seeds() }
    fn agent_seed(_: u64) -> u64 { fixtures::SEED }
    fn files() -> temper_service::settings::Files { fixtures::files() }
}
struct Observer {
    inner: ServiceObserver,
}
impl Observe for Observer {
    fn started(root: Fd, inherited: &skein_world::Inherited) -> Self {
        Self { inner: ServiceObserver::started(root, inherited) }
    }
    fn complete(&mut self, complete: skein_io::kernel::Complete, service: &mut temper_service::Service) -> Option<skein_io::kernel::Complete> {
        self.inner.complete(complete, service)
    }
    fn before_iterate(&mut self, service: &mut temper_service::Service, now: Time, wall: skein_lib::Wall) -> bool {
        self.inner.before_iterate(service, now, wall)
    }
    fn submitted(&mut self, submit: &skein_io::kernel::Submit) { self.inner.submitted(submit); }
    fn after_iterate(&mut self, service: &mut temper_service::Service, submitted: u32) { self.inner.after_iterate(service, submitted); }
    fn handover_read(&mut self, _: &temper_service::HandoverRead) -> Option<Result<Option<skein_lib::Secret>, temper_service::HandoverUnread>> { None }
    fn drain(&mut self, service: &mut temper_service::Service, now: Time) { self.inner.drain(service, now); }
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
fn person_steps() -> Box<[Step<Address>]> {
    let (mut steps, replies) = stories::playground::person();
    steps.truncate(usize::try_from(replies[0] + 1).expect("first reply step"));
    steps.extend(new_chat(b"Golden coding"));
    steps.into_boxed_slice()
}
fn llm_peer() -> skein_fake_peers::llm::Peer {
    let mut scripts: serde_json::Value = serde_json::from_slice(temper_playground::stories::A.llm).expect("story A scripts");
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
    let bytes = serde_json::to_vec(&scripts).expect("golden scripts");
    let document = temper_fake::llm::Document::decode(&bytes).expect("golden LLM document");
    temper_fake::llm::peer(temper_fake::llm::configuration(&document, temper_playground::documents::KEY, temper_playground::documents::seeds(fixtures::SEED).llm)).expect("golden LLM peer")
}

fn live(store: &Store) -> impl Iterator<Item = &TaskRecord> {
    store.rows.values().filter_map(|row| match row {
        Row::Core(Record::Tasks(tasks::Stored::Live(task))) => Some(task.as_ref()),
        _ => None,
    })
}
fn in_flight(store: &Store) -> Option<&TaskRecord> {
    live(store).find(|task| task.spec.words.as_ref() == b"Golden coding in flight" && task.turn > 0 && matches!(task.phase, tasks::Phase::Active(tasks::Active::Running { .. })))
}
struct Judge {
    controls: skein_world::Controls,
    store: Rc<RefCell<Store>>,
    forge: referee::forge::Forge,
    stopped: bool,
    released: bool,
}
impl skein_world::Referee<Proc> for Judge {
    fn act(&mut self, _: Time, procs: &mut [Proc]) {
        let (first, others) = procs.split_at_mut(1);
        let Proc::Service(service) = &mut first[0] else { panic!("service first") };
        if let Some(link) = service.service.links().pop() {
            for proc in others.iter_mut() {
                if let Proc::Other(HarnessProcess::Client(person)) = proc { person.sign_in(&link); }
            }
        }
        if !self.stopped && self.forge.forge_complete() && in_flight(&self.store.borrow()).is_some() {
            eprintln!("golden in-flight {:?}", in_flight(&self.store.borrow()));
            for proc in others.iter_mut() {
                if let Proc::Other(HarnessProcess::Client(person)) = proc { person.finish(); }
            }
            self.controls.signal(0, ServiceSignal::Terminate);
            self.stopped = true;
        }
        if self.stopped {
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
    fn observe(&mut self, _: Time, procs: &[Proc]) { self.forge.observe(procs); }
    fn next_deadline(&self) -> Option<Time> { (!self.stopped).then_some(Time::from_nanos(60_000_000_000)) }
    fn overdue(&self, now: Time) -> Option<String> { (!self.stopped && now >= Time::from_nanos(60_000_000_000)).then(|| format!("golden did not settle: live {:?}", live(&self.store.borrow()).collect::<Vec<_>>())) }
    fn passed(&self) -> bool { self.stopped }
}

/// An ordinary world story; the caller supplies only its test's temporary directory.
pub fn make(directory: &Path) {
    stories::set(Story::PlaygroundA);
    stories::setup_settings(world::Phase::Story);
    let mut start = temper_playground::stories::A.start(fixtures::SEED, skein_sim::Config::calm().wall);
    let mut document: serde_json::Value = serde_json::from_slice(&start.deployment).expect("deployment");
    document["engine"]["limits"]["overrides"]["people.inbox_tasks"] = 16.into();
    let mut provider = document["engine"]["llm"]["providers"][0].clone();
    provider["number"] = 2.into();
    provider["name"] = "golden-provider".into();
    provider["models"] = serde_json::json!(["fake-key"]);
    let catalogue: serde_json::Value = serde_json::from_slice(include_bytes!("../../../crates/temper-playground/stories/a/catalogue.json")).expect("story catalogue");
    document["engine"]["llm"]["models"] = catalogue["models"].clone();
    let mut model = catalogue["models"][0].clone();
    model["name"] = "fake-key".into();
    document["engine"]["llm"]["models"].as_array_mut().expect("models").push(model);
    document["engine"]["llm"]["providers"][0]["models"] = serde_json::json!(["fake"]);
    document["engine"]["llm"]["providers"].as_array_mut().expect("providers").push(provider);
    for name in ["tasks.tasks", "tasks.project_tasks"] {
        document["engine"]["limits"]["overrides"][name] = 8.into();
    }
    start.deployment = serde_json::to_vec(&document).expect("golden configuration").into();
    let mut disk = machine::fresh();
    temper_playground::story::prepare(&mut disk, temper_playground::stories::A.deployment, start.seed);
    let store = Rc::new(RefCell::new(Store::new()));
    world::lower_store(Some(store.clone()));
    let client = disk.reopen_root(2);
    let machine = Machine::new(disk);
    let params = programs::Parameters { deployment: start.deployment.clone(), seed: start.seed, links: Links::Keep, forge: start.fakes.forge.clone(), route: temper_playground::story::route(&document), disk: machine.0.clone() };
    let outcome = programs::with(params, || {
        let mut world = skein_world::World::new_controlled(fixtures::SEED, skein_sim::Config::calm(), skein_world::Memory::Unchecked, |controls| Judge { controls, store: store.clone(), forge: referee::forge::Forge::new(start.fakes.forge.clone(), Story::PlaygroundA), stopped: false, released: false }).with_machine(machine);
        for (program, roots) in programs::programs::<Golden>() { world.host_roots(program, roots); }
        world.spawn_restartable(programs::service_roots(), programs::service::<Golden>);
        world.spawn_root(skein_sim::Handle::new(client.raw()), |root| Proc::Other(HarnessProcess::Client(Box::new(SignalHost::new(Person::welcome(root, person_steps()).with_account(), None)))));
        world.spawn(|| Proc::Peer(Box::new(SignalHost::new(llm_peer(), None))));
        world.spawn(|| Proc::Forge(Box::new(SignalHost::new(start.fakes.peer, None))));
        world.run()
    });
    let Proc::Service(service) = &outcome.procs[0] else { panic!("service first") };
    assert!(service.service.is_empty(), "settled service stop");
    assert!(store.borrow().pending.is_empty(), "no pending commits at stop");
    for row in store.borrow().rows.values() { eprintln!("golden kept {row:?}"); }
    files::export(&mut outcome.machine.0.borrow_mut(), directory, &start.deployment);
    world::lower_store(None);
}
