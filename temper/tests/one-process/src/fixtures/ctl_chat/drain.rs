//! Independent deployments share only the process that hosts them (temper/docs/design/testing/tiers.md, 1.5).
use super::{FIRST, Proc, Start, WORDS, command, parked, peer};
use crate::{
    ctl::{self, Caller, Command},
    fixtures,
    referee::Store,
    stories,
    world::{self, HarnessProcess, Story},
};
use skein_io::kernel::{Exit, ServiceSignal};
use skein_lib::Time;
use skein_shell::Host;
use std::{cell::RefCell, rc::Rc};
use temper_playground::{
    machine::{self, Machine, SignalHost},
    programs,
};

type Receipt = (u64, u64);
const SETUP_CALLS: usize = 4;
const CALLS_PER_CHAT: usize = 2;

fn created_at(chat: usize) -> usize {
    SETUP_CALLS + chat * CALLS_PER_CHAT
}

fn start_chat(caller: &mut Caller, key: &[u8]) {
    command(caller, &[b"chat", b"start", b"--project", b"1", WORDS, b"--key", key, b"--json"]);
}

#[derive(Clone, Copy)]
enum Stage {
    Link,
    Created(usize),
    Replied { chat: usize, number: u64 },
    Closed(usize),
    Draining,
    Done,
}

struct Judge {
    controls: skein_world::Controls,
    keys: [&'static [u8]; 2],
    store: Rc<RefCell<Store>>,
    stage: Stage,
    wake: Option<Time>,
}

impl skein_world::Referee<Proc> for Judge {
    fn act(&mut self, _: Time, procs: &mut [Proc]) {
        let [Proc::Service(service), Proc::Other(HarnessProcess::Calls(caller)), Proc::Peer(peer), ..] = procs else {
            panic!("service, ctl caller and provider")
        };
        self.wake = None;
        match self.stage {
            Stage::Link => {
                if let Some(link) = service.service.links().pop() {
                    caller.commands.push_back(Command::sign_in(&link));
                    command(
                        caller,
                        &[b"account", b"add-key", b"1", b"/setup/llm.key", b"--deployment", b"--key", b"drain-model"],
                    );
                    command(caller, &[b"account", b"rank", b"1", b"1"]);
                    command(caller, &[b"account", b"list", b"--until", b"usable"]);
                    start_chat(caller, self.keys[0]);
                    self.stage = Stage::Created(0);
                }
            }
            Stage::Created(chat) => {
                if let Some(answer) = caller.answers.get(created_at(chat)) {
                    self.stage = Stage::Replied { chat, number: receipt(answer).0 };
                }
            }
            Stage::Replied { chat, number } => {
                // Exercise the hosted agent roots, then free the owner's purse.
                if parked(&self.store.borrow().rows, number) {
                    let key = [self.keys[chat], b"-close"].concat();
                    command(caller, &[b"chat", b"close", number.to_string().as_bytes(), b"--key", &key]);
                    self.stage = Stage::Closed(chat);
                }
            }
            Stage::Closed(chat) => {
                if caller.answers.get(created_at(chat) + 1).is_some() {
                    if let Some(key) = self.keys.get(chat + 1) {
                        start_chat(caller, key);
                        self.stage = Stage::Created(chat + 1);
                    } else {
                        caller.closed = true;
                        self.controls.signal(0, ServiceSignal::Terminate);
                        self.stage = Stage::Draining;
                    }
                }
            }
            Stage::Draining if service.is_empty() => {
                peer.shutdown();
                self.stage = Stage::Done;
            }
            Stage::Draining | Stage::Done => {}
        }
    }

    fn watch(&mut self, _: Time, procs: &[Proc]) -> Result<(), String> {
        let [Proc::Service(service), Proc::Other(HarnessProcess::Calls(caller)), ..] = procs else {
            panic!("service and ctl caller")
        };
        if let Some(why) = &service.observer.finding {
            return Err(why.clone());
        }
        for (call, answer) in caller.answers.iter().enumerate() {
            if answer.exit != Exit::Code(0) {
                return Err(format!(
                    "ctl call {} exited {:?}: {}",
                    call + 1,
                    answer.exit,
                    String::from_utf8_lossy(&answer.error)
                ));
            }
        }
        Ok(())
    }

    fn idle(&mut self, now: Time, procs: &[Proc], _: bool) {
        if matches!(self.stage, Stage::Draining) && procs[0].is_empty() {
            self.wake = Some(now);
        }
    }

    fn next_deadline(&self) -> Option<Time> {
        self.wake
    }

    fn passed(&self) -> bool {
        matches!(self.stage, Stage::Done)
    }
}

fn receipt(answer: &ctl::Answer) -> Receipt {
    let value: serde_json::Value = serde_json::from_slice(&answer.output).expect("chat start JSON");
    let created = value.pointer("/outcome/done/document/created").expect("committed creation receipt");
    (created["first"].as_u64().expect("first task number"), created["last"].as_u64().expect("last task number"))
}

fn number_two_chats(
    mut disk: skein_fake_machine::Machine,
    directory: &'static [u8],
    keys: [&'static [u8]; 2],
) -> (skein_fake_machine::Machine, [Receipt; 2]) {
    let root = disk.reopen_root(2);
    let machine = Machine::new(disk);
    let mut parameters = world::parameters(&machine, stories::forge(Story::Chat { words: WORDS }));
    let mut document = fixtures::world_document();
    document["world_settings"]["llm"]["endpoints"][0]["address"]["socket"] = FIRST.to_string().into();
    parameters.deployment = document.into();
    let store = Rc::new(RefCell::new(Store::new()));
    world::lower_store(Some(store.clone()));
    let result = programs::with(parameters, || {
        let mut simulation = skein_world::World::new_controlled(
            fixtures::SEED,
            skein_sim::Config::calm(),
            skein_world::Memory::Unchecked,
            |controls| Judge { controls, keys, store: store.clone(), stage: Stage::Link, wake: None },
        )
        .with_machine(machine);
        for (program, roots) in programs::programs::<super::Chat>() {
            simulation.host_roots(program, roots);
        }
        simulation.host_roots(
            ctl::program(|spawn, inherited| Proc::Other(HarnessProcess::Ctl(Box::new(ctl::host(spawn, inherited))))),
            ctl::roots,
        );
        simulation.spawn_restartable(programs::service_roots(), programs::service::<super::Chat>);
        simulation.spawn_root(skein_sim::Handle::new(root.raw()), |root| {
            Proc::Other(HarnessProcess::Calls(Box::new(Caller::at_directory(root, directory))))
        });
        simulation.spawn(|| Proc::Peer(Box::new(SignalHost::new(peer(Start::Before), None))));
        simulation.try_run()
    });
    world::lower_store(None);
    // The run checks all hosts, descriptors, operations and hosted children.
    let outcome = result.expect("the numbered service and its world settle");
    let [Proc::Service(service), Proc::Other(HarnessProcess::Calls(caller)), ..] = outcome.procs.as_slice() else {
        panic!("settled service and ctl caller")
    };
    assert_eq!(service.service.end(), Some(&temper_service::End::Engine(temper_service::EngineEnd::Drained)));
    let receipts = [0, 1].map(|chat| receipt(&caller.answers[created_at(chat)]));
    let disk = outcome.machine.0.clone();
    drop(outcome);
    assert_eq!(Rc::strong_count(&store), 1, "the store observer is no longer installed");
    let disk = Rc::into_inner(disk).expect("no factory parameters retain the world disk").into_inner();
    assert_eq!(disk.open_handles(), 0, "the fake disk has no open handles");
    assert!(machine::roots_are_released(), "no agent roots remain before the next fresh disk resets the registry");
    (disk, receipts)
}

fn new_disk() -> skein_fake_machine::Machine {
    let mut disk = machine::fresh();
    machine::setup(&mut disk);
    disk
}

/// Number two chats in each of two separately drained services on one thread.
#[must_use]
pub fn receipts_across_drained_services() -> [[Receipt; 2]; 2] {
    stories::set(Story::Chat { words: WORDS });
    let (disk, first) = number_two_chats(new_disk(), b"/client/first", [b"drain-a-first", b"drain-a-second"]);
    drop(disk);
    let (_, second) = number_two_chats(new_disk(), b"/client/second", [b"drain-b-first", b"drain-b-second"]);
    [first, second]
}
