//! History through a fresh-window start (temper/docs/design/testing/README.md, 2.5).
use super::{FIXTURE_SETS, maps::Maps};
use crate::{ctl, fixtures, golden, referee, stories, world::{self, HarnessProcess, Person, ServiceObserver, Story}};
use jig_domain_core::{CoreRecord, Record};
use skein_io::kernel::{Complete, Fd, ServiceSignal, Submit};
use skein_lib::{Time, Wall};
use skein_shell::Host;
use std::{cell::RefCell, rc::Rc};
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
    fn given(document: &serde_json::Value) -> temper_service::stock::Given {
        let temper_service::stock::Given::Own {mut jig, forge, ways} = temper_playground::documents::given(document) else { panic!("world defaults") };
        let effect = jig_domain_core::ToolFamilies::JIG.effect.0;
        jig.rules.ceiling.tools.0 |= effect;
        for number in [1,2] {
            let policy = &mut jig.policies.get_mut(&number).expect("world policy").policy;
            policy.ceiling.tools.0 |= effect;
            for role in &mut policy.roles { role.authority.tools.0 |= effect; }
        }
        temper_service::stock::Given::Own {jig, forge, ways}
    }
}
#[derive(Default)]
struct State {
    upgrade: bool,
    before: Option<Maps>,
    checked: bool,
    starts: u32,
    issue: bool,
    armed: bool,
    faulted: bool,
    write_at: Option<u32>,
    old_wires: u64,
    disk: Option<Rc<RefCell<skein_fake_machine::Machine>>>,
    task: u64,
    attempt: u64,
    tokens: Vec<(skein_lib::Secret, u64)>,
    users: Vec<(u64, Box<[u8]>)>,
    kept_answer: Vec<u8>,
}
thread_local! { static STATE: RefCell<State> = RefCell::new(State::default()); }
struct Observer { inner: ServiceObserver }
impl Observe for Observer {
    fn started(root: Fd, inherited: &skein_world::Inherited) -> Self {
        STATE.with(|s| s.borrow_mut().starts += 1);
        let mut inner = ServiceObserver::started(root, inherited);
        inner.wire.keep_starts();
        Self { inner }
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
            Some(jig_domain_core::Key::Core(jig_domain_core::CoreKey::Deployment)) => {
                let decode = |value: &Vec<u8>| jig_protocol_store::codec::decode(named.as_ref().expect("key"), value, &stories::playground::codec(), fixtures::store_names()).expect("header");
                let Record::Core(CoreRecord::Deployment(mut expected)) = decode(old.expect("header kept")) else { panic!("header") };
                let Record::Core(CoreRecord::Deployment(found)) = decode(new.expect("header kept")) else { panic!("header") };
                assert!(found.commits > expected.commits, "upgrade commits carry on");
                expected.commits = found.commits;
                assert_eq!(found, expected, "all identity high-water marks preserved");
                allowed.push("patch, restore and wires commit position");
            }
            _ => {
                let own = jig_protocol_store::keys::decode_own(key);
                if matches!(own, Some(jig_protocol_store::keys::Own::Format)) {
                    let mut expected = before.format();
                    expected.levels.application = expected.levels.application.map(|level| level + 1);
                    assert_eq!(after.format(),expected,"only the fixture owner level changes");
                    allowed.push("fixture owner level");
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
    let parent = scripts["asked"].as_array_mut().expect("asked").iter_mut().find(|s| s["cue"] == "Golden coding").expect("coding parent");
    let delegate = &mut parent["turns"][0]["lines"][1]["arguments"]["batch"][0];
    delegate["executor"] = serde_json::json!({"kind":"agent", "charter":1});
    delegate["contract"] = serde_json::json!({"kind":"report", "words":128});
    delegate["spec"]["parameters"] = serde_json::json!([]);
    delegate["authority"]["tools"] = 579.into();
    delegate["authority"]["grants"].as_array_mut().expect("grants").push(serde_json::json!({"connector":1,"kind":8,"segments":["forge"],"terminal":"open","last":""}));
    let coding = scripts["cued"].as_array_mut().expect("cued").iter_mut().find(|s| s["cue"] == "## Task\n\nGolden coding in flight").expect("coding script");
    coding["turns"][0]["lines"][0] = serde_json::json!({"call":"read_task", "arguments":{"task":5,"field":"specification"}});
    coding["turns"][0]["lines"].as_array_mut().expect("lines").insert(0, serde_json::json!({"text":"upgrade-tail-marker"}));
    coding["turns"].as_array_mut().expect("coding turns").insert(1, serde_json::json!({"lines":[{"call":"effect_forge", "arguments":{"repository":{"forge":1,"repository":1},"resource":{"kind":"repository"},"write":{"kind":"create_issue","key":"upgrade-write","title":"Upgrade once","body":"kept across starts"}}}],"finish":"tool_calls","tokens":7}));
    if STATE.with(|s| s.borrow().upgrade) {
        coding["turns"] = serde_json::json!([{"lines":[{"call":"finish","arguments":{"form":"report","text":"Fresh after upgrade","fields":{}}}],"finish":"tool_calls","tokens":1}]);
    }
    if STATE.with(|s| s.borrow().upgrade) {
        scripts["asked"][0]["turns"] = serde_json::json!([{"lines":[{"text":"Paused chat resumed"},{"call":"wait","arguments":{}}],"finish":"tool_calls","tokens":2}]);
    }
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
    finished: bool,
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
        if !self.upgrade && !STATE.with(|s| s.borrow().armed) && golden::in_flight(&self.store.borrow()).is_some() && self.forge.forge_complete() {
            STATE.with(|s| {
                let mut s = s.borrow_mut();
                if !s.armed {
                    let store = self.store.borrow();
                    let task = golden::in_flight(&store).expect("mid-run task");
                    s.task = task.number;
                    s.attempt = task.attempt;
                    for proc in others.iter_mut() {
                        if let Proc::Other(HarnessProcess::Calls(caller)) = proc { caller.commands.push_back(ctl::Command::new(&[b"task",b"message",s.task.to_string().as_bytes(),b"remember tail",b"--key",b"remember-tail",b"--json"])); }
                    }
                    s.armed = true;
                }
            });
        }
        if !self.upgrade && STATE.with(|s| s.borrow().armed && !s.borrow().faulted) && self.store.borrow().rows.values().any(|r| matches!(r,temper_engine_domain::Record::Forge {row} if matches!(row.as_ref(),temper_engine_domain_forge::Stored::Entry {entry,..} if entry.attempt.is_some() && STATE.with(|s| entry.task == s.borrow().task)))) {
            self.transport.forge.borrow_mut().config.timeouts = 1000;
            STATE.with(|s| s.borrow_mut().faulted = true);
        }
        let checked = STATE.with(|s| s.borrow().checked);
        if self.upgrade && checked && !self.client_started {
            for proc in others.iter_mut() {
                if let Proc::Other(HarnessProcess::Calls(caller)) = proc {
                    let task = STATE.with(|s| s.borrow().task.to_string());
                    caller.commands.push_back(ctl::Command::new(&[b"task",b"message",task.as_bytes(),b"remember tail",b"--key",b"remember-tail",b"--json"]));
                    caller.commands.push_back(ctl::Command::new(&[b"task",b"message",b"1",b"Resume paused chat",b"--key",b"resume-paused",b"--json"]));
                    caller.commands.push_back(ctl::Command::new(&[b"chat", b"start", b"After upgrade", b"--project", b"1", b"--key", b"after-upgrade", b"--json"]));
                    caller.closed = true;
                }
            }
            self.client_started = true;
        }
        let complete = if self.upgrade {
            checked && self.store.borrow().agent_records.iter().any(|r| matches!(r, referee::AgentRecord::Start {task, attempt, ..} if STATE.with(|s| *task == s.borrow().task && *attempt > s.borrow().attempt)))
                && others.iter().any(|p| matches!(p, Proc::Other(HarnessProcess::Calls(c)) if c.answers.len() == 3))
                && self.store.borrow().rows.values().any(|r| matches!(r, temper_engine_domain::Record::Core(Record::Core(CoreRecord::Turn(turn))) if turn.task == 1 && skein_lib::bytes::find(&turn.transcript,b"Paused chat resumed").is_some()))
                && !self.store.borrow().rows.values().any(|r| matches!(r, temper_engine_domain::Record::Forge {row} if matches!(row.as_ref(),temper_engine_domain_forge::Stored::Flight {task,..} if STATE.with(|s| *task == s.borrow().task))))
        } else { STATE.with(|s| s.borrow().issue) && others.iter().any(|p| matches!(p, Proc::Other(HarnessProcess::Calls(c)) if c.answers.len() == 1)) };
        if complete && !self.stopped {
            for proc in others.iter() {
                if let Proc::Other(HarnessProcess::Calls(caller)) = proc {
                    for answer in &caller.answers { assert_eq!(answer.exit,skein_io::kernel::Exit::Code(0),"ctl: {}",String::from_utf8_lossy(&answer.error)); }
                    if self.upgrade { STATE.with(|s| assert_eq!(caller.answers[0].output,s.borrow().kept_answer,"replayed key is answered from its record")); }
                    else { STATE.with(|s| s.borrow_mut().kept_answer = caller.answers[0].output.clone()); }
                }
            }
            if self.upgrade { continued(&self.store.borrow()); }

            for proc in others.iter_mut() {
                if let Proc::Other(HarnessProcess::Client(person)) = proc { person.finish(); }
                if let Proc::Other(HarnessProcess::Calls(caller)) = proc { caller.closed = true; }
            }
            self.controls.signal(0, ServiceSignal::Terminate);
            self.stopped = true;
        }
        if self.stopped { self.stop_peers(others); self.finished = service.is_empty() && others.iter().all(Host::is_empty); }
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
    fn next_deadline(&self) -> Option<Time> { Some(Time::from_nanos(60_000_000_000)) }
    fn overdue(&self, now: Time) -> Option<String> {
        (!self.stopped && now >= Time::from_nanos(60_000_000_000)).then(|| format!("upgrade stalled: checked {}, issue {}, tasks {:?}", STATE.with(|s| s.borrow().checked), STATE.with(|s| s.borrow().issue), self.store.borrow().rows.values().filter_map(|row| if let temper_engine_domain::Record::Core(Record::Tasks(jig_domain_tasks::Stored::Live(task))) = row { Some((task.number, task.attempt, task.turn, &task.phase)) } else { None }).collect::<Vec<_>>()))
    }
    fn passed(&self) -> bool { self.finished }
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
        s.old_wires = store.borrow().applied;
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
    machine::lay_handover(&mut disk, b"client/private/address", format!("http://{}\n", fixtures::CLIENTS).as_bytes());
    let client = (!upgrade).then(|| disk.reopen_root(2));
    let caller_root = disk.reopen_root(2);
    let machine = Machine::new(disk);
    STATE.with(|s| s.borrow_mut().disk = Some(machine.0.clone()));
    let document = serde_json::from_slice(configuration).expect("configuration");
    let params = programs::Parameters { deployment: configuration.into(), seed: fixtures::SEED, links: Links::Keep, forge: transport.clone(), route: temper_playground::story::route(&document), disk: machine.0.clone() };
    let mut outcome = programs::with(params, || {
        let mut world = skein_world::World::new_controlled(fixtures::SEED, skein_sim::Config::calm(), skein_world::Memory::Unchecked, |controls| Judge { controls, store: store.clone(), forge: referee::forge::Forge::new(transport.clone(), Story::PlaygroundA).keeping_ledger(), transport: transport.clone(), upgrade, stopped:false, released:false, client_started:false, ledger:ledger.clone(), finished:false }).with_machine(machine);
        for (program, roots) in programs::programs::<Upgrade>() { world.host_roots(program, roots); }
        world.host_roots(ctl::program(|spawn, inherited| Proc::Other(HarnessProcess::Ctl(Box::new(ctl::host(spawn,inherited))))), ctl::roots);
        world.spawn_restartable(programs::service_roots(), programs::service::<Upgrade>);
        world.spawn_root(skein_sim::Handle::new(caller_root.raw()), |root| Proc::Other(HarnessProcess::Calls(Box::new(ctl::Caller::scripted(root)))));
        if !upgrade {
            world.spawn_root(skein_sim::Handle::new(client.expect("first client").raw()), |root| Proc::Other(HarnessProcess::Client(Box::new(SignalHost::new(Person::welcome(root, golden::person_steps()).with_account_after(7),None)))));
        }
        world.spawn(|| Proc::Peer(Box::new(SignalHost::new(provider(), None))));
        world.spawn(|| Proc::Forge(Box::new(SignalHost::new(forge_peer(transport),None))));
        world.run()
    });
    for proc in &mut outcome.procs {
        if let Proc::Forge(peer) = proc {
            STATE.with(|s| {
                let mut s = s.borrow_mut();
                s.tokens = peer.setup_mut().issued().iter().map(|t| (t.secret.clone(), t.user)).collect();
                s.users = [b"temper".as_slice(), b"owner"].into_iter().filter_map(|login| peer.setup_mut().find(login).map(|a| (a.id, a.login.clone()))).collect();
            });
        }
    }
    let Proc::Service(service) = &outcome.procs[0] else { panic!("service") };
    assert!(service.service.is_empty(), "settled service stop");
    std::mem::replace(&mut outcome.machine.0.borrow_mut(), machine::fresh())
}

fn forge_peer(transport: &temper_fake_git_hosted::Typed) -> skein_fake_peers::forge::Peer {
    let document = temper_fake::forge::Document::decode(temper_playground::stories::A.forge).expect("forge document");
    let seed = temper_playground::documents::seeds(fixtures::SEED).forge;
    let (_, mut table) = document.seed(seed);
    let table = STATE.with(|s| {
        let s = s.borrow();
        let mut tokens = skein_lib::List::with_capacity(table.tokens.len() + u32::try_from(s.tokens.len()).expect("tokens"));
        for token in table.tokens.into_boxed() { tokens.push(token).expect("token room"); }
        for token in &s.tokens { tokens.push(token.clone()).expect("token room"); }
        table.tokens = tokens;
        let mut users = skein_lib::List::with_capacity(table.users.len() + u32::try_from(s.users.len()).expect("users"));
        for user in table.users.into_boxed() { users.push(user).expect("user room"); }
        for user in &s.users { users.push(user.clone()).expect("user room"); }
        table.users = users;
        table
    });
    let mut peer = skein_fake_peers::forge::Peer::new(document.listen(), temper_fake::limits::peer(), table, temper_fake::forge::git_limits(), transport.forge.clone(), seed).expect("forge peer").without_records();
    let token = temper_fake::credential::credential(seed, temper_fake::credential::ADMIN_TOKEN);
    temper_fake::forge::setup(peer.setup_mut(), &token);
    peer
}

fn continued(store: &referee::Store) {
    let (task, old_attempt) = STATE.with(|s| (s.borrow().task,s.borrow().attempt));
    let start = store.agent_records.iter().find_map(|row| match row {
        referee::AgentRecord::Start {task:t,attempt,input:Some(input),..} if *t == task && *attempt > old_attempt => Some(input),
        _ => None,
    }).expect("mid-run task gets a new attempt");
    assert!(start.transcript().is_empty(),"fresh attempt, not the closed conversation");
    let charter = smith_charter::Charter::decode(&smith_charter::CEILINGS,&mut skein_lib::Reader::new(start.charter())).expect("charter");
    assert!(charter.brief().as_slice().iter().any(|section| skein_lib::bytes::find(section.body(), b"upgrade-tail-marker").is_some()),"fresh brief carries the closed window's tail");
    assert!(!store.rows.values().any(|row| matches!(row, temper_engine_domain::Record::Forge {row} if matches!(row.as_ref(),temper_engine_domain_forge::Stored::Flight {task:t,..} if *t == task))),"crossed claim's forge write settled");
    let new = store.rows.values().find_map(|row| match row {
        temper_engine_domain::Record::Core(Record::Tasks(jig_domain_tasks::Stored::Live(task) | jig_domain_tasks::Stored::Ended(task))) if task.spec.words.as_ref() == b"After upgrade" => Some(task.number),
        _ => None,
    }).expect("new chat allocated");
    STATE.with(|s| {
        let s = s.borrow();
        let maps = s.before.as_ref().expect("before");
        let key = jig_domain_core::Key::Core(jig_domain_core::CoreKey::Deployment);
        let encoded = jig_protocol_store::keys::encode(&key,64).expect("header key");
        let Record::Core(CoreRecord::Deployment(header)) = jig_protocol_store::codec::decode(&key,maps.store.get(encoded.as_ref()).expect("header"),&stories::playground::codec(),fixtures::store_names()).expect("header decodes") else {panic!("header")};
        assert!(new > header.tasks,"new task above every kept task");
        assert!(start.activation() > header.runs,"new attempt above every kept attempt");
    });
}
