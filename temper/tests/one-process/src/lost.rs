//! A named ctl answer cut after its commit (temper/docs/design/testing/tiers.md, 1.5).
use crate::{
    ctl::{self, Caller, Command},
    fixtures,
    referee::Store,
    stories,
    world::{self, HarnessProcess, OneProcess, Proc, Story},
};
use skein_io::kernel::{Exit, ServiceSignal};
use skein_lib::Time;
use skein_shell::Host;
use std::{cell::RefCell, rc::Rc};
use temper_playground::{
    machine::{self, Machine, SignalHost},
    programs::{self, Process},
};

const FIRST: &[u8] = b"/client/private";
const SECOND: &[u8] = b"/client/fresh-private";
const KEY: &[u8] = b"lost-answer";
const PROJECT: &[u8] = b"1";
const START: &[&[u8]] = &[b"chat", b"start", b"--project", PROJECT, b"--key", KEY, b"Hello"];
const BOUND: Time = Time::from_nanos(15_000_000_000);

#[derive(Debug)]
enum Stage {
    Link,
    Setup,
    Held,
    Killed(u64),
    Spent(u64),
    Fresh(u64),
    Replay(u64),
    Reply(u64),
    Readback(u64),
    Done,
}

struct Judge {
    stage: Stage,
    controls: skein_world::Controls,
    store: Rc<RefCell<Store>>,
    first_link: Box<[u8]>,
    party: Option<u64>,
}

fn caller(proc: &mut Proc) -> &mut Caller {
    let Process::Other(HarnessProcess::Calls(caller)) = proc else { panic!("ctl caller") };
    caller
}

fn ok(answer: &ctl::Answer) {
    assert_eq!(answer.exit, Exit::Code(0), "{}", String::from_utf8_lossy(&answer.error));
}

fn contains(bytes: &[u8], text: &[u8]) -> bool {
    skein_lib::bytes::find(bytes, text).is_some()
}

impl Judge {
    #[expect(clippy::too_many_lines, reason = "the referee keeps the cut, second sign-in, and readback in order")]
    fn advance(&mut self, procs: &mut [Proc]) {
        let [Process::Service(service), first, second, peers @ ..] = procs else { panic!("lost-answer roots") };
        let first = caller(first);
        let second = caller(second);
        let sign_ins = || {
            peers
                .iter()
                .filter_map(|proc| match proc {
                    Process::Other(HarnessProcess::Ctl(ctl)) => ctl.signed_party(),
                    _ => None,
                })
                .collect::<Vec<_>>()
        };
        match self.stage {
            Stage::Link => {
                let Some(link) = service.service.links().pop() else { return };
                first.sign_in(&link);
                self.first_link = link;
                for words in [
                    &[
                        b"account".as_slice(),
                        b"add-key",
                        b"1",
                        b"/setup/llm.key",
                        b"--deployment",
                        b"--key",
                        b"lost-account",
                    ][..],
                    &[b"account".as_slice(), b"list", b"--until", b"usable"],
                ] {
                    first.commands.push_back(Command::new(words));
                }
                self.stage = Stage::Setup;
            }
            Stage::Setup if first.answers.len() == 3 => {
                first.answers.iter().for_each(ok);
                let signed = sign_ins();
                let [party] = signed.as_slice() else { panic!("first directory signed in once: {signed:?}") };
                self.party = Some(*party);
                first.commands.push_back(Command::new(START));
                self.stage = Stage::Held;
            }
            Stage::Held => {
                assert_eq!(first.answers.len(), 3, "the held call must not return");
                let held = peers
                    .iter()
                    .any(|proc| matches!(proc, Process::Other(HarnessProcess::Ctl(ctl)) if ctl.answer_held()));
                let name = temper_client_tools::jig::key::Name::new(KEY).expect("request name");
                let key = jig_domain_people::RequestKey {
                    person: self.party.expect("signed party"),
                    key: temper_client_tools::key::engine(&name),
                };
                if held && let Some(task) = self.store.borrow().created_task(key) {
                    first.kill_child();
                    self.stage = Stage::Killed(task);
                }
            }
            Stage::Killed(task) if first.answers.len() == 4 => {
                let killed = &first.answers[3];
                assert_eq!((killed.exit, killed.output.as_slice()), (Exit::Signal(9), b"".as_slice()));
                first.closed = true;
                second.sign_in(&self.first_link);
                self.stage = Stage::Spent(task);
            }
            Stage::Spent(task) if second.answers.len() == 1 => {
                let spent = &second.answers[0];
                assert_eq!(spent.exit, Exit::Code(1));
                assert!(contains(&spent.error, b"it was already used"), "{}", String::from_utf8_lossy(&spent.error));
                let Some(link) = service.service.links().pop() else { return };
                assert_ne!(link, self.first_link, "second attempt uses a fresh link");
                second.sign_in(&link);
                self.stage = Stage::Fresh(task);
            }
            Stage::Fresh(task) if second.answers.len() == 2 => {
                ok(&second.answers[1]);
                let party = self.party.expect("signed party");
                assert_eq!(sign_ins(), [party, party], "two directories signed in as the same party");
                second.commands.push_back(Command::new(START));
                self.stage = Stage::Replay(task);
            }
            Stage::Replay(task) if second.answers.len() == 3 => {
                let again = &second.answers[2];
                ok(again);
                let output = std::str::from_utf8(&again.output).expect("ctl output");
                let named = output
                    .split_once("done: task ")
                    .and_then(|(_, rest)| rest.split(|c: char| !c.is_ascii_digit()).next()?.parse::<u64>().ok());
                assert_eq!(named, Some(task), "retry prints the committed task: {output}");
                self.stage = Stage::Reply(task);
            }
            Stage::Reply(task) if self.store.borrow().has_reply(task, fixtures::ANSWER) => {
                second.commands.push_back(Command::new(&[b"chat", b"show", task.to_string().as_bytes()]));
                second.commands.push_back(Command::new(&[
                    b"task",
                    b"list",
                    b"--project",
                    PROJECT,
                    b"--label",
                    b"chat",
                ]));
                self.stage = Stage::Readback(task);
            }
            Stage::Readback(task) if second.answers.len() == 5 => {
                let reply = &second.answers[3];
                ok(reply);
                assert!(
                    contains(&reply.output, fixtures::ANSWER),
                    "chat reply: {}",
                    String::from_utf8_lossy(&reply.output)
                );
                let tasks = &second.answers[4];
                ok(tasks);
                let listed: Vec<u64> = std::str::from_utf8(&tasks.output)
                    .expect("task list")
                    .lines()
                    .filter_map(|line| line.split_once(" · ").and_then(|(number, _)| number.parse().ok()))
                    .collect();
                assert_eq!(listed, [task], "the party has exactly the original chat");
                second.closed = true;
                self.controls.signal(0, ServiceSignal::Terminate);
                for proc in peers {
                    if let Process::Peer(peer) = proc {
                        peer.shutdown();
                    }
                }
                self.stage = Stage::Done;
            }
            _ => {}
        }
    }
}

impl skein_world::Referee<Proc> for Judge {
    fn act(&mut self, _: Time, procs: &mut [Proc]) {
        self.advance(procs);
    }
    fn watch(&mut self, _: Time, procs: &[Proc]) -> Result<(), String> {
        let Process::Service(service) = &procs[0] else { return Err("service first".into()) };
        service.observer.finding.clone().map_or(Ok(()), Err)
    }
    fn next_deadline(&self) -> Option<Time> {
        (!matches!(self.stage, Stage::Done)).then_some(BOUND)
    }
    fn overdue(&self, now: Time) -> Option<String> {
        (!matches!(self.stage, Stage::Done) && now >= BOUND).then(|| format!("lost answer: {:?}", self.stage))
    }
    fn passed(&self) -> bool {
        matches!(self.stage, Stage::Done)
    }
}

fn reply_provider() -> skein_fake_peers::llm::Peer {
    use skein_fake_llm_domain::api::{Finish, Line, Script, Turn};
    fixtures::scripted_provider(Box::new([Script {
        cue: Box::default(),
        turns: Box::new([Turn {
            lines: Box::new([
                Line::Text { text: fixtures::ANSWER.into() },
                Line::Call { name: b"wait".as_slice().into(), arguments: b"{}".as_slice().into() },
            ]),
            finish: Finish::ToolCalls,
            tokens: 1,
        }]),
    }]))
}

/// Two independent ctl directories replay one durable named chat.
pub fn run() {
    let scenario = Story::ChatReadThenWait { words: b"Hello" };
    stories::set(scenario);
    stories::setup_settings(world::Phase::Story);
    let store = Rc::new(RefCell::new(Store::new()));
    world::lower_store(Some(store.clone()));
    let mut disk = machine::fresh();
    machine::setup(&mut disk);
    let first = disk.reopen_root(2);
    let second = disk.reopen_root(2);
    let machine = Machine::new(disk);
    let parameters = world::parameters(&machine, stories::forge(scenario));
    let outcome = programs::with(parameters, || {
        let mut simulation = skein_world::World::new_controlled(
            fixtures::SEED,
            skein_sim::Config::calm(),
            skein_world::Memory::Unchecked,
            |controls| Judge { stage: Stage::Link, controls, store, first_link: Box::default(), party: None },
        )
        .with_machine(machine);
        for (program, roots) in programs::programs::<OneProcess>() {
            simulation.host_roots(program, roots);
        }
        simulation.host_roots(
            ctl::program(|spawn, inherited| {
                let host = ctl::host(spawn, inherited);
                let host = if spawn.args[0].as_ref() == FIRST
                    && spawn.args.get(1).is_some_and(|word| word.as_ref() == b"chat")
                {
                    host.with_answer_cut()
                } else {
                    host
                };
                Process::Other(HarnessProcess::Ctl(Box::new(host)))
            }),
            ctl::roots,
        );
        simulation.spawn_restartable(programs::service_roots(), programs::service::<OneProcess>);
        for (root, directory) in [(first, FIRST), (second, SECOND)] {
            simulation.spawn_root(skein_sim::Handle::new(root.raw()), |root| {
                Process::Other(HarnessProcess::Calls(Box::new(Caller::at_directory(root, directory))))
            });
        }
        simulation.spawn(|| Process::Peer(Box::new(SignalHost::new(reply_provider(), None))));
        simulation.run()
    });
    world::lower_store(None);
    assert!(outcome.procs.iter().all(Host::is_empty), "lost-answer world drains");
    assert_eq!(outcome.killed.len(), 1, "the simulator observed the cut");
    assert_eq!(outcome.killed[0].exit, Exit::Signal(9));
}
