//! A need reaches a push service and Gone erases its device (attention.md, 7.3; testing/fakes.md, 6).
use crate::{
    ctl::{Caller, Command},
    fixtures,
    referee::{self, Store},
    stories,
    world::{self, HarnessProcess, OneProcess, Proc, Story},
};
use skein_io::kernel::{Exit, ServiceSignal};
use skein_lib::{Secret, Time};
use skein_shell::Host;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use temper_playground::{
    machine::{Machine, SignalHost},
    programs,
};
const PEM: &[u8] = include_bytes!("fixtures/vapid.pem");
const ORIGIN: &str = "http://127.0.0.1:31999";
const DEADLINE: Time = Time::from_nanos(10_000_000_000);
thread_local! { static ACTIVE: Cell<bool> = const { Cell::new(false) }; }
pub(crate) fn configure(document: &mut serde_json::Value) {
    if ACTIVE.with(Cell::get) {
        for (name, count) in [
            ("tasks.tasks", 8),
            ("tasks.project_tasks", 8),
            ("tasks.tree_tasks", 4),
            ("tasks.delegates", 2),
            ("people.inbox_tasks", 16),
            ("people.operations", 3),
        ] {
            document["engine"]["limits"]["overrides"][name] = count.into();
        }
        // A conversation asks its person through a person delegate (jig's tasks/messages.md, 5).
        for project in document["world_settings"]["policies"].as_array_mut().expect("policies") {
            project["policy"]["period_spend"] = 1000.into();
            project["policy"]["roles"][0]["period_spend"] = 1000.into();
            for pointer in ["/policy/ceiling", "/policy/roles/0/authority"] {
                let authority = project.pointer_mut(pointer).expect("project authority");
                authority["tools"] = 1.into();
                authority["delegation"]["kinds"] =
                    serde_json::json!([{"kind":"charter","number":1},{"kind":"role","number":0}]);
            }
        }
        document["application"]["push"] = serde_json::json!({
            "key":"push.key", "subject":"mailto:operator@example.test", "services":[ORIGIN]
        });
    }
}
pub(crate) fn given(given: &mut temper_service::stock::Given) {
    if ACTIVE.with(Cell::get) {
        let temper_service::stock::Given::Own { jig, .. } = given else { panic!("fixture defaults") };
        jig.charters[0].policy.waiting = skein_lib::Duration::from_millis(100);
    }
}
pub(crate) fn files(files: &mut temper_service::settings::Files) {
    if ACTIVE.with(Cell::get) {
        files.entries.push((b"push.key".as_slice().into(), Secret::new(PEM.into()))).expect("push fixture");
    }
}
fn provider() -> skein_fake_peers::llm::Peer {
    use skein_fake_llm_domain::api::{Finish, Line, Script, Turn};
    fixtures::scripted_provider(Box::new([Script {
        cue: b"@push".as_slice().into(),
        turns: Box::new([
            Turn {
                lines: Box::new([Line::Call { name: b"delegate".as_slice().into(), arguments: br#"{"batch":[{"executor":{"kind":"person","person":1},"spec":{"words":"Choose a colour? Blue or Green?"},"contract":{"kind":"report","words":80},"authority":{"tools":0,"grants":[],"delegation":{"kinds":[],"tasks":0,"depth":0},"budget":{"spend":0},"notes":0}}]}"#.as_slice().into() }]),
                finish: Finish::ToolCalls, tokens: 1,
            },
            Turn { lines: Box::new([Line::Call { name: b"wait".as_slice().into(), arguments: b"{}".as_slice().into() }]), finish: Finish::ToolCalls, tokens: 1 },
            Turn { lines: Box::new([Line::Call { name: b"finish".as_slice().into(), arguments: br#"{"form":"report","text":"The question waits with the person.","fields":{}}"#.as_slice().into() }]), finish: Finish::ToolCalls, tokens: 1 },
        ]),
    }]))
}
#[derive(Clone, Copy, Debug)]
enum Stage {
    Setup,
    First,
    Devices,
    Gone,
    Empty,
    Done,
}
struct Judge {
    controls: skein_world::Controls,
    store: Rc<RefCell<Store>>,
    stage: Stage,
    handed: bool,
    drained: bool,
    checked: Cell<u64>,
}
impl Judge {
    fn chat(caller: &mut Caller, key: &[u8]) {
        caller.commands.push_back(Command::new(&[b"chat", b"start", b"--project", b"1", b"--key", key, b"@push"]));
    }
    fn devices(&self) -> usize {
        self.store.borrow().rows.values().filter(|row| matches!(row,
            temper_engine_domain::Record::Forge { row } if matches!(row.as_ref(), temper_engine_domain_forge::Stored::Device(_))
        )).count()
    }
}
impl skein_world::Referee<Proc> for Judge {
    fn act(&mut self, _now: Time, procs: &mut [Proc]) {
        let (first, others) = procs.split_at_mut(1);
        let Proc::Service(service) = &mut first[0] else { panic!("service first") };
        let (callers, peers) = others.split_at_mut(1);
        let Proc::Other(HarnessProcess::Calls(caller)) = &mut callers[0] else { panic!("native caller second") };
        let push = peers
            .iter_mut()
            .find_map(|proc| match proc {
                Proc::Other(HarnessProcess::Push(peer)) => Some(peer),
                _ => None,
            })
            .expect("fake push peer");
        if !self.handed
            && push.address().is_some()
            && let Some(link) = service.service.links().pop()
        {
            let mut sign_in = Command::new(&[b"sign-in", b"-"]);
            sign_in.input = link;
            caller.commands.push_back(sign_in);
            caller.commands.push_back(Command::new(&[
                b"account",
                b"add-key",
                b"1",
                b"/setup/llm.key",
                b"--deployment",
                b"--key",
                b"push-account",
            ]));
            caller.commands.push_back(Command::new(&[
                b"push",
                b"add",
                b"/setup/subscription.json",
                b"--label",
                b"Alice",
                b"--key",
                b"push-device",
            ]));
            Self::chat(caller, b"push-first");
            self.handed = true;
            self.stage = Stage::First;
        }
        for answer in &caller.answers {
            assert_eq!(
                answer.exit,
                Exit::Code(0),
                "push {:?}: {} {}",
                self.stage,
                String::from_utf8_lossy(&answer.output),
                String::from_utf8_lossy(&answer.error)
            );
            assert!(answer.error.is_empty());
        }
        match self.stage {
            Stage::First if caller.answers.len() == 4 && push.received().len() == 1 => {
                let received = &push.received()[0];
                assert_eq!(received.target.as_ref(), b"/push/alice");
                assert_eq!(received.topic.as_deref(), Some(b"p1".as_slice()));
                assert_eq!(received.ttl, 86_400);
                assert_eq!(received.urgency.as_ref(), b"high");
                assert_eq!(self.devices(), 1);
                caller.commands.push_back(Command::new(&[b"push", b"list", b"--json"]));
                self.stage = Stage::Devices;
            }
            Stage::Devices if caller.answers.len() == 5 => {
                let document: serde_json::Value =
                    serde_json::from_slice(&caller.answers[4].output).expect("devices JSON");
                assert_eq!(document["devices"].as_array().expect("devices").len(), 1);
                push.script(b"/push/alice".as_slice().into(), skein_fake_push::Script::Gone);
                Self::chat(caller, b"push-second");
                self.stage = Stage::Gone;
            }
            Stage::Gone if caller.answers.len() == 6 && self.devices() == 0 => {
                assert_eq!(push.received().len(), 1, "Gone records no accepted push");
                caller.commands.push_back(Command::new(&[b"push", b"list", b"--json"]));
                self.stage = Stage::Empty;
            }
            Stage::Empty if caller.answers.len() == 7 => {
                let document: serde_json::Value =
                    serde_json::from_slice(&caller.answers[6].output).expect("devices JSON");
                assert!(document["devices"].as_array().expect("devices").is_empty(), "Gone marks the devices document");
                caller.closed = true;
                push.shutdown();
                self.controls.signal(0, ServiceSignal::Terminate);
                self.stage = Stage::Done;
            }
            Stage::Setup | Stage::First | Stage::Devices | Stage::Gone | Stage::Empty | Stage::Done => {}
        }
        if matches!(self.stage, Stage::Done) && service.service.is_empty() {
            self.drained = true;
            for proc in peers {
                if let Proc::Peer(peer) = proc {
                    peer.shutdown();
                }
            }
        }
    }
    fn observe(&mut self, _now: Time, procs: &[Proc]) {
        let Proc::Service(service) = &procs[0] else { panic!("service") };
        let applied = self.store.borrow().applied;
        if self.checked.replace(applied) != applied {
            referee::Referee::check_roots(&service.service);
        }
    }
    fn next_deadline(&self) -> Option<Time> {
        (!self.drained).then_some(DEADLINE)
    }
    fn overdue(&self, now: Time) -> Option<String> {
        (!self.drained && now >= DEADLINE).then(|| {
            format!(
                "push story {:?}, devices {}, agent {:?}, tasks {:?}",
                self.stage,
                self.devices(),
                self.store.borrow().agent_records,
                self.store
                    .borrow()
                    .rows
                    .values()
                    .filter_map(|row| match row {
                        temper_engine_domain::Record::Core(jig_domain_core::Record::Tasks(
                            jig_domain_tasks::Stored::Live(task),
                        )) => Some((task.number, task.executor, &task.phase)),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
            )
        })
    }
    fn passed(&self) -> bool {
        self.drained
    }
}
/// One wire path: the fake checks the token before recording the push, then answers Gone.
pub fn run() {
    ACTIVE.with(|active| active.set(true));
    stories::set(Story::Chat { words: b"@push" });
    stories::setup_settings(world::Phase::Story);
    let transport = stories::forge(Story::Chat { words: b"@push" });
    let store = Rc::new(RefCell::new(Store::new()));
    world::lower_store(Some(store.clone()));
    let mut disk = temper_playground::machine::fresh();
    // RFC 8291 Appendix A's user-agent public key and authentication secret.
    let subscription = serde_json::to_vec(&serde_json::json!({
        "endpoint":format!("{ORIGIN}/push/alice"),
        "keys":{"p256dh":"BCVxsr7N_eNgVRqvHtD0zTZsEc6-VV-JvLexhqUzORcxaOzi6-AYWXvTBHm4bjyPjs7Vd8pZGH6SRpkNtoIAiw4", "auth":"BTBZMqHH6r4Tts7J_aSIgg"}
    })).expect("subscription JSON");
    let setup = disk.lay(&[
        skein_fake_machine::Item::file(b"llm.key", fixtures::LLM_TOKEN).mode(0o600),
        skein_fake_machine::Item::file(b"subscription.json", &subscription).mode(0o600),
    ]);
    disk.close(setup);
    let client = disk.reopen_root(2);
    let machine = Machine::new(disk);
    let parameters = world::parameters(&machine, transport);
    let outcome = programs::with(parameters, || {
        let mut simulation = skein_world::World::new_controlled(
            fixtures::SEED,
            skein_sim::Config::calm(),
            skein_world::Memory::Unchecked,
            |controls| Judge {
                controls,
                store,
                stage: Stage::Setup,
                handed: false,
                drained: false,
                checked: Cell::new(u64::MAX),
            },
        )
        .with_machine(machine);
        for (program, roots) in programs::programs::<OneProcess>() {
            simulation.host_roots(program, roots);
        }
        simulation.host_roots(
            crate::ctl::program(|spawn, inherited| {
                Proc::Other(HarnessProcess::Ctl(Box::new(crate::ctl::host(spawn, inherited))))
            }),
            crate::ctl::roots,
        );
        simulation.spawn_restartable(programs::service_roots(), programs::service::<OneProcess>);
        simulation.spawn_root(skein_sim::Handle::new(client.raw()), |root| {
            Proc::Other(HarnessProcess::Calls(Box::new(Caller::scripted(root))))
        });
        simulation.spawn(|| Proc::Peer(Box::new(SignalHost::new(provider(), None))));
        simulation.spawn(|| {
            Proc::Other(HarnessProcess::Push(Box::new(
                skein_fake_peers::push::Peer::new(
                    ([127, 0, 0, 1], 31999).into(),
                    fixtures::peer_limits(),
                    *skein_push::Vapid::from_pkcs8_pem(PEM).expect("VAPID fixture").public(),
                    fixtures::SEED,
                )
                .expect("fake push peer limits"),
            )))
        });
        simulation.run()
    });
    assert!(outcome.procs.iter().all(Host::is_empty), "every push story process drains");
    world::lower_store(None);
    ACTIVE.with(|active| active.set(false));
}
