//! History through a fresh-window start (temper/docs/design/testing/README.md, 2.5).
use super::{FIXTURE_SETS, maps::Maps};
use crate::{ctl, fixtures, golden, referee, stories, world::{self, HarnessProcess, Person, ServiceObserver, Story}};
use jig_domain_core::{CoreRecord, Record};
use jig_domain_tasks as tasks;
use skein_io::kernel::{Complete, Fd, ServiceSignal, Submit};
use skein_lib::{Time, Wall};
use skein_shell::Host;
use std::{cell::RefCell, rc::Rc};
use temper_engine_domain::Record as Row;
use temper_playground::{Links, machine::{self, Machine, SignalHost}, programs::{self, Harness}, service::Observe};

struct Upgrade;
type Proc = programs::Process<Upgrade>;
impl Harness for Upgrade {
    type Service = Observer;
    type Process = HarnessProcess;
    fn patch_sets() -> Option<jig_protocol_store::PatchSets> {
        Some(STATE.with(|s| if s.borrow().upgrade { FIXTURE_SETS } else { temper_service::store::PATCH_SETS }))
    }
    fn seeds(_: u64) -> temper_service::settings::Seeds { fixtures::seeds() }
    fn agent_seed(_: u64) -> u64 { fixtures::SEED }
    fn files() -> temper_service::settings::Files { fixtures::files() }
}
#[derive(Default)]
struct State {
    upgrade: bool,
    before: Option<Maps>,
    checked: bool,
    starts: u32,
    issue: bool,
    armed: bool,
    write_at: Option<u32>,
    old_wires: u64,
    disk: Option<Rc<RefCell<skein_fake_machine::Machine>>>,
}
thread_local! { static STATE: RefCell<State> = RefCell::new(State::default()); }
struct Observer { inner: ServiceObserver }
impl Observe for Observer {
    fn started(root: Fd, inherited: &skein_world::Inherited) -> Self {
        STATE.with(|s| s.borrow_mut().starts += 1);
        Self { inner: ServiceObserver::started(root, inherited) }
    }
    fn complete(&mut self, complete: Complete, service: &mut temper_service::Service) -> Option<Complete> {
        self.inner.complete(complete, service)
    }
    fn before_iterate(&mut self, service: &mut temper_service::Service, now: Time, wall: Wall) -> bool {
        let pending = self.inner.before_iterate(service, now, wall);
        STATE.with(|state| {
            let mut state = state.borrow_mut();
            if state.upgrade && !state.checked {
                let wires = STORE.with(|s| s.borrow().as_ref().expect("store").borrow().wires);
                if wires.is_some_and(|n| n > state.old_wires) {
                    let after = Maps::read(&mut state.disk.as_ref().expect("disk").borrow_mut());
                    history(state.before.as_ref().expect("stop snapshot"), &after);
                    state.checked = true;
                }
            }
        });
        pending
    }
    fn submitted(&mut self, submit: &Submit) { self.inner.submitted(submit); }
    fn after_iterate(&mut self, service: &mut temper_service::Service, submitted: u32) {
        self.inner.after_iterate(service, submitted);
    }
    fn handover_read(&mut self, _: &temper_service::HandoverRead) -> Option<Result<Option<skein_lib::Secret>, temper_service::HandoverUnread>> { None }
    fn drain(&mut self, service: &mut temper_service::Service, now: Time) { self.inner.drain(service, now); }
}
thread_local! { static STORE: RefCell<Option<Rc<RefCell<referee::Store>>>> = const { RefCell::new(None) }; }

/// Every key is compared; exceptions are enumerated and checked, never removed silently.
fn history(before: &Maps, after: &Maps) {
    assert_eq!(after.active, before.active + 1, "one whole copied generation");
    assert_eq!(before.secrets, after.secrets, "every secret record preserved");
    let keys: std::collections::BTreeSet<_> = before.store.keys().chain(after.store.keys()).collect();
    let mut allowed = Vec::new();
    for key in keys {
        let old = before.store.get(key);
        let new = after.store.get(key);
        if old == new { continue; }
        let named = jig_protocol_store::keys::decode(key);
        match named {
            Some(jig_domain_core::Key::Core(jig_domain_core::CoreKey::RunProof(_))) => {
                let decode = |value: &Vec<u8>| jig_protocol_store::codec::decode(named.as_ref().expect("key"), value, &stories::playground::codec(), fixtures::store_names()).expect("proof");
                let Record::Core(CoreRecord::RunProof(mut expected)) = decode(old.expect("proof kept")) else { panic!("proof") };
                expected.crossed = true;
                if let Some(window) = expected.window.as_mut() { window.closed = true; }
                assert_eq!(decode(new.expect("proof kept")), Record::Core(CoreRecord::RunProof(expected)), "only windows closed and claims crossed");
                allowed.push("fresh-windows: closed window and crossed claim");
            }
            _ => {
                let own = jig_protocol_store::keys::decode_own(key);
                if matches!(own, Some(jig_protocol_store::keys::Own::Format | jig_protocol_store::keys::Own::Layouts | jig_protocol_store::keys::Own::Wires)) {
                    allowed.push("component levels, layouts and wires");
                } else {
                    panic!("unlisted history difference key {key:02x?}: {named:?}, {own:?}\nold {old:02x?}\nnew {new:02x?}");
                }
            }
        }
    }
    assert!(allowed.contains(&"fresh-windows: closed window and crossed claim"), "a live window exercised the mark");
    eprintln!("upgrade history allowed: {allowed:?}");
}

fn provider() -> skein_fake_peers::llm::Peer {
    let mut scripts = golden::llm_scripts();
    scripts["asked"].as_array_mut().expect("asked").iter_mut().find(|s| s["cue"] == "Golden coding").expect("coding")["turns"][1] = serde_json::json!({
        "lines":[
            {"pieces":[{"text":"write after the landing", "hold":"upgrade-write"}]},
            {"call":"effect_forge", "arguments":{"repository":{"forge":1,"repository":1},"resource":{"kind":"repository"},"write":{"kind":"create_issue","key":"upgrade-write","title":"Upgrade once","body":"kept across starts"}}}
        ],"finish":"tool_calls","tokens":7
    });
    let document = temper_fake::llm::Document::decode(&serde_json::to_vec(&scripts).expect("scripts")).expect("LLM scripts");
    temper_fake::llm::peer(temper_fake::llm::configuration(&document, temper_playground::documents::KEY, temper_playground::documents::seeds(fixtures::SEED).llm)).expect("peer")
}

#[expect(clippy::struct_excessive_bools, reason = "independent stop, fault, client and upgrade observations")]
struct Judge {
    controls: skein_world::Controls,
    store: Rc<RefCell<referee::Store>>,
    forge: referee::forge::Forge,
    transport: temper_fake_git_hosted::Typed,
    upgrade: bool,
    stopped: bool,
    released: bool,
    client_started: bool,
    ledger: Rc<RefCell<Vec<skein_fake_forge_domain::Observation>>>,
}
impl Judge {
    fn stop_peers(&mut self, procs: &mut [Proc]) {
        let ended = procs.iter().all(|p| !matches!(p, Proc::Agent(a) if !a.is_empty()));
        for p in procs {
            match p {
                Proc::Peer(peer) => {
                    if ended && !self.released {
                        peer.release(stories::STOP_HOLD).expect("mid-run hold released");
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
        let (first, others) = procs.split_at_mut(1);
        let Proc::Service(service) = &mut first[0] else { panic!("service first") };
        if !self.upgrade && let Some(link) = service.service.links().pop() {
            for proc in others.iter_mut() {
                if let Proc::Other(HarnessProcess::Client(person)) = proc { person.sign_in(&link); }
            }
        }
        if !self.upgrade && self.forge.forge_complete() && golden::in_flight(&self.store.borrow()).is_some() {
            STATE.with(|s| {
                let mut s = s.borrow_mut();
                if !s.armed {
                    let mut forge = self.transport.forge.borrow_mut();
                    forge.config.timeouts = 1000;
                    forge.config.latency_min = skein_lib::Duration::from_secs(1);
                    forge.config.latency_max = forge.config.latency_min;
                    for proc in others.iter_mut() {
                        if let Proc::Peer(peer) = proc { peer.release(b"upgrade-write").expect("release write"); }
                    }
                    s.armed = true;
                }
            });
        }
        let checked = STATE.with(|s| s.borrow().checked);
        if self.upgrade && checked && !self.client_started {
            for proc in others.iter_mut() {
                if let Proc::Other(HarnessProcess::Calls(caller)) = proc {
                    caller.commands.push_back(ctl::Command::new(&[b"chat", b"start", b"After upgrade", b"--project", b"1", b"--key", b"after-upgrade", b"--json"]));
                    caller.closed = true;
                }
            }
            self.client_started = true;
        }
        let complete = if self.upgrade {
            checked && self.store.borrow().agent_records.iter().any(|r| matches!(r, referee::AgentRecord::Start {task:7, attempt, ..} if *attempt > 6))
                && others.iter().any(|p| matches!(p, Proc::Other(HarnessProcess::Calls(c)) if c.answers.len() == 1))
        } else { STATE.with(|s| s.borrow().issue) };
        if complete && !self.stopped {
            for proc in others.iter_mut() {
                if let Proc::Other(HarnessProcess::Client(person)) = proc { person.finish(); }
            }
            self.controls.signal(0, ServiceSignal::Terminate);
            self.stopped = true;
        }
        if self.stopped { self.stop_peers(others); }
    }
    fn observe(&mut self, _: Time, procs: &[Proc]) {
        // Keep the fake's write ledger outside the service and peer lifetimes.
        // Forge's branch referee consumes the same observations after this copy.
        self.forge.observe(procs);
        self.ledger.borrow_mut().extend(self.forge.take_ledger());
        let issue = self.ledger.borrow().iter().any(|o| matches!(o, skein_fake_forge_domain::Observation::Opened {title, ..} if title.as_ref() == b"Upgrade once"));
        if issue {
            STATE.with(|s| {
                let mut s = s.borrow_mut();
                if !s.issue {
                    s.issue = true;
                    let Proc::Service(service) = &procs[0] else { panic!("service") };
                    s.write_at = Some(service.observer.inner.submitted);
                    self.transport.forge.borrow_mut().config.timeouts = 0;
                }
            });
        }
    }
    fn next_deadline(&self) -> Option<Time> { (!self.stopped).then_some(Time::from_nanos(60_000_000_000)) }
    fn overdue(&self, now: Time) -> Option<String> {
        (!self.stopped && now >= Time::from_nanos(60_000_000_000)).then(|| format!("upgrade stalled: checked {}, issue {}, tasks {:?}", STATE.with(|s| s.borrow().checked), STATE.with(|s| s.borrow().issue), self.store.borrow().rows))
    }
    fn passed(&self) -> bool { self.stopped }
}

/// The service stops settled, then starts with the mark-only fixture sets.
pub fn settled() {
    STATE.with(|s| *s.borrow_mut() = State::default());
    stories::set(Story::PlaygroundA);
    stories::setup_settings(world::Phase::Story);
    let start = temper_playground::stories::A.start(fixtures::SEED, skein_sim::Config::calm().wall);
    let configuration = golden::configuration(&start.deployment);
    let mut disk = machine::fresh();
    temper_playground::story::prepare(&mut disk, &start.deployment, start.seed);
    let store = Rc::new(RefCell::new(referee::Store::new()));
    let ledger = Rc::new(RefCell::new(Vec::new()));
    let disk = phase(disk, &configuration, &start.fakes.forge, &store, &ledger, false);
    assert!(store.borrow().pending.is_empty(), "settled stop has no pending commit");
    let mut disk = disk;
    let before = Maps::read(&mut disk);
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        s.before = Some(before);
        s.upgrade = true;
        s.old_wires = store.borrow().wires.expect("first wires");
    });
    store.borrow_mut().recovering();
    let _disk = phase(disk, &configuration, &start.fakes.forge, &store, &ledger, true);
    assert!(STATE.with(|s| s.borrow().checked), "history checked before the root acts");
    assert_eq!(ledger.borrow().iter().filter(|o| matches!(o, skein_fake_forge_domain::Observation::Opened {title,..} if title.as_ref() == b"Upgrade once")).count(), 1, "uncertain write made once across starts");
    world::lower_store(None);
    STORE.with(|s| *s.borrow_mut() = None);
}

fn phase(disk: skein_fake_machine::Machine, configuration: &[u8], transport: &temper_fake_git_hosted::Typed, store: &Rc<RefCell<referee::Store>>, ledger: &Rc<RefCell<Vec<skein_fake_forge_domain::Observation>>>, upgrade: bool) -> skein_fake_machine::Machine {
    world::lower_store(Some(store.clone()));
    STORE.with(|s| *s.borrow_mut() = Some(store.clone()));
    let mut disk = disk;
    if upgrade { machine::lay_handover(&mut disk, b"client/private/address", format!("http://{}\n", fixtures::CLIENTS).as_bytes()); }
    let client = disk.reopen_root(2);
    let machine = Machine::new(disk);
    STATE.with(|s| s.borrow_mut().disk = Some(machine.0.clone()));
    let document = serde_json::from_slice(configuration).expect("configuration");
    let params = programs::Parameters { deployment: configuration.into(), seed: fixtures::SEED, links: Links::Keep, forge: transport.clone(), route: temper_playground::story::route(&document), disk: machine.0.clone() };
    let outcome = programs::with(params, || {
        let mut world = skein_world::World::new_controlled(fixtures::SEED, skein_sim::Config::calm(), skein_world::Memory::Unchecked, |controls| Judge { controls, store: store.clone(), forge: referee::forge::Forge::new(transport.clone(), Story::PlaygroundA), transport: transport.clone(), upgrade, stopped:false, released:false, client_started:false, ledger:ledger.clone() }).with_machine(machine);
        for (program, roots) in programs::programs::<Upgrade>() { world.host_roots(program, roots); }
        world.host_roots(ctl::program(|spawn, inherited| Proc::Other(HarnessProcess::Ctl(Box::new(ctl::host(spawn,inherited))))), ctl::roots);
        world.spawn_restartable(programs::service_roots(), programs::service::<Upgrade>);
        world.spawn_root(skein_sim::Handle::new(client.raw()), |root| {
            if upgrade { Proc::Other(HarnessProcess::Calls(Box::new(ctl::Caller::scripted(root)))) }
            else { Proc::Other(HarnessProcess::Client(Box::new(SignalHost::new(Person::welcome(root, golden::person_steps()).with_account_after(7),None)))) }
        });
        world.spawn(|| Proc::Peer(Box::new(SignalHost::new(provider(), None))));
        world.spawn(|| Proc::Forge(Box::new(SignalHost::new(temper_playground::fakes::forge_peer(transport, fixtures::SEED),None))));
        world.run()
    });
    let Proc::Service(service) = &outcome.procs[0] else { panic!("service") };
    assert!(service.service.is_empty(), "settled service stop");
    std::mem::replace(&mut outcome.machine.0.borrow_mut(), machine::fresh())
}
