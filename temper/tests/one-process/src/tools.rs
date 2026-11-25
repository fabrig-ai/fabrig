//! Native ctl calls on one simulated service and client DIRECTORY.
use crate::{
    ctl::{self, Caller, Command, Recorded},
    fixtures,
    referee::{Store, client::Received},
    stories,
    world::{self, HarnessProcess, OneProcess, Proc, Story},
};
use jig_client::contract::Delivery;
use skein_io::kernel::{Exit, ServiceSignal};
use skein_lib::Time;
use skein_shell::Host;
use std::{cell::RefCell, rc::Rc};
use temper_playground::{
    machine::{self, Machine, SignalHost},
    programs,
};

const CALLERS: usize = 3;
const WORDS: &[u8] = b"Hello";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Stage {
    Link,
    SignIn,
    Start,
    Replied(u64),
    FirstWatch(u64),
    SecondWatch(u64),
    Me(u64),
    Close,
    Done,
}

struct Judge {
    controls: skein_world::Controls,
    stage: Stage,
    sign_in_only: bool,
    close_sent: bool,
    finding: Option<String>,
}

fn caller(proc: &mut Proc) -> &mut Caller {
    let Proc::Other(HarnessProcess::Calls(caller)) = proc else { panic!("ctl caller") };
    caller
}

fn recorded(procs: &[Proc]) -> impl Iterator<Item = &Recorded> {
    procs.iter().filter_map(|proc| match proc {
        Proc::Other(HarnessProcess::Ctl(ctl)) if ctl.receipts().is_some() => Some(ctl.as_ref()),
        _ => None,
    })
}

fn watchers(procs: &[Proc], task: u64) -> usize {
    recorded(procs)
        .filter(|ctl| {
            ctl.exit().is_none()
                && ctl.receipts().expect("recording enabled").received.iter().any(
                    |receipt| matches!(receipt, Received::Delivery(Delivery::Task(value)) if value.chip.task == task),
                )
        })
        .count()
}

fn close_sent(procs: &[Proc]) -> bool {
    // The close is the only recorded command that sends a keyed request.
    recorded(procs).any(|ctl| ctl.receipts().expect("recording enabled").sent_jig())
}

fn wait(caller: &mut Caller, task: u64, until: &[u8]) {
    caller.commands.push_back(Command::new(&[b"task", b"show", task.to_string().as_bytes(), b"--until", until]));
}

fn start(caller: &mut Caller) {
    caller.commands.extend([
        Command::new(&[b"account", b"add-key", b"1", b"/setup/llm.key", b"--deployment", b"--key", b"tools-account"]),
        Command::new(&[b"account", b"list", b"--until", b"usable"]),
        Command::new(&[b"chat", b"start", b"--project", b"1", b"--key", b"tools-chat", WORDS, b"--json"]),
    ]);
}

impl Judge {
    fn count(&self) -> usize {
        if self.sign_in_only { 1 } else { CALLERS }
    }

    fn stop(&mut self, callers: &mut [Proc], others: &mut [Proc]) {
        for proc in callers {
            caller(proc).closed = true;
        }
        self.controls.signal(0, ServiceSignal::Terminate);
        for proc in others {
            if let Proc::Peer(peer) = proc {
                peer.shutdown();
            }
        }
        self.stage = Stage::Done;
    }

    fn advance(&mut self, procs: &mut [Proc]) -> Result<(), String> {
        let (service, rest) = procs.split_at_mut(1);
        let Proc::Service(service) = &mut service[0] else { panic!("service first") };
        let (callers, others) = rest.split_at_mut(self.count());
        let answers = caller(&mut callers[0]).answers.len();
        match self.stage {
            Stage::Link => {
                if let Some(link) = service.service.links().pop() {
                    let mut sign_in = Command::new(&[b"sign-in", b"-"]);
                    sign_in.input = link;
                    caller(&mut callers[0]).commands.push_back(sign_in);
                    self.stage = Stage::SignIn;
                }
            }
            Stage::SignIn if answers == 1 => {
                if self.sign_in_only {
                    self.stop(callers, others);
                } else {
                    start(caller(&mut callers[0]));
                    self.stage = Stage::Start;
                }
            }
            Stage::Start if answers == 4 => {
                let started: serde_json::Value = serde_json::from_slice(&caller(&mut callers[0]).answers[3].output)
                    .map_err(|why| format!("chat start JSON: {why}"))?;
                let task = started
                    .pointer("/outcome/done/document/created/first")
                    .and_then(serde_json::Value::as_u64)
                    .ok_or_else(|| format!("chat start did not name the task: {started}"))?;
                wait(caller(&mut callers[0]), task, b"replied");
                self.stage = Stage::Replied(task);
            }
            Stage::Replied(task) if answers == 5 => {
                wait(caller(&mut callers[1]), task, b"ended");
                self.stage = Stage::FirstWatch(task);
            }
            Stage::FirstWatch(task) if watchers(others, task) == 1 => {
                wait(caller(&mut callers[2]), task, b"ended");
                self.stage = Stage::SecondWatch(task);
            }
            Stage::SecondWatch(task) if watchers(others, task) == 2 => {
                caller(&mut callers[0]).commands.push_back(Command::new(&[b"me", b"--json"]));
                self.stage = Stage::Me(task);
            }
            Stage::Me(task) if answers == 6 => {
                if watchers(others, task) != 2 {
                    return Err("me must answer while both task watches are open".into());
                }
                caller(&mut callers[0]).commands.push_back(Command::new(&[
                    b"chat",
                    b"close",
                    task.to_string().as_bytes(),
                    b"--key",
                    b"tools-close",
                ]));
                self.stage = Stage::Close;
            }
            Stage::Close => {
                self.close_sent |= close_sent(others);
                if self.close_sent
                    && answers == 7
                    && !caller(&mut callers[1]).answers.is_empty()
                    && !caller(&mut callers[2]).answers.is_empty()
                {
                    self.stop(callers, others);
                }
            }
            Stage::SignIn
            | Stage::Start
            | Stage::Replied(_)
            | Stage::FirstWatch(_)
            | Stage::SecondWatch(_)
            | Stage::Me(_)
            | Stage::Done => {}
        }
        Ok(())
    }
}

impl skein_world::Referee<Proc> for Judge {
    fn act(&mut self, _: Time, procs: &mut [Proc]) {
        self.finding = self.advance(procs).err();
    }

    fn watch(&mut self, _: Time, procs: &[Proc]) -> Result<(), String> {
        if let Some(why) = &self.finding {
            return Err(why.clone());
        }
        if let Proc::Service(service) = &procs[0]
            && let Some(why) = &service.observer.finding
        {
            return Err(why.clone());
        }
        let (callers, others) = procs[1..].split_at(self.count());
        if !self.close_sent && !close_sent(others) && recorded(others).any(|ctl| ctl.exit().is_some()) {
            return Err("an end wait exited before the close request was sent".into());
        }
        for (index, proc) in callers.iter().enumerate() {
            let Proc::Other(HarnessProcess::Calls(caller)) = proc else { panic!("ctl caller") };
            for (call, answer) in caller.answers.iter().enumerate() {
                if answer.exit != Exit::Code(0) {
                    return Err(format!("{:?}: caller {index} call {call} exited {:?}", self.stage, answer.exit));
                }
            }
        }
        Ok(())
    }

    fn passed(&self) -> bool {
        self.stage == Stage::Done
    }
}

fn run(sign_in_only: bool) -> skein_fake_machine::Machine {
    let story = Story::ChatReadThenWait { words: WORDS };
    stories::set(story);
    stories::setup_settings(world::Phase::Story);
    world::lower_store(Some(Rc::new(RefCell::new(Store::new()))));
    let mut disk = machine::fresh();
    let count = if sign_in_only { 1 } else { CALLERS };
    let roots: Vec<_> = (0..count).map(|_| disk.reopen_root(2)).collect();
    if !sign_in_only {
        machine::setup(&mut disk);
    }
    let machine = Machine::new(disk);
    let mut parameters = world::parameters(&machine, stories::forge(story));
    if !sign_in_only {
        Rc::make_mut(&mut parameters.deployment).jig_over_all([("clients.streams", 3), ("clients_http.streams", 3)]);
    }
    let outcome = programs::with(parameters, || {
        let mut simulation = skein_world::World::new_controlled(
            fixtures::SEED,
            skein_sim::Config::calm(),
            skein_world::Memory::Unchecked,
            |controls| Judge { controls, stage: Stage::Link, sign_in_only, close_sent: false, finding: None },
        )
        .with_machine(machine);
        for (program, roots) in programs::programs::<OneProcess>() {
            simulation.host_roots(program, roots);
        }
        let mut ctl = ctl::program(|spawn, inherited| {
            let host = ctl::host(spawn, inherited);
            let end_wait = spawn.args.last().is_some_and(|word| word.as_ref() == b"ended");
            let close = spawn.args.get(2).is_some_and(|word| word.as_ref() == b"close");
            Proc::Other(HarnessProcess::Ctl(Box::new(if end_wait || close { host.observing() } else { host })))
        });
        ctl.instances = u32::try_from(count).expect("ctl instance count");
        simulation.host_roots(ctl, ctl::roots);
        simulation.spawn_restartable(programs::service_roots(), programs::service::<OneProcess>);
        for root in roots {
            simulation.spawn_root(skein_sim::Handle::new(root.raw()), |root| {
                Proc::Other(HarnessProcess::Calls(Box::new(Caller::scripted(root))))
            });
        }
        if !sign_in_only {
            simulation.spawn(|| Proc::Peer(Box::new(SignalHost::new(stories::provider(story), None))));
        }
        simulation.run()
    });
    assert!(outcome.procs.iter().all(Host::is_empty), "every ctl story process drains");
    world::lower_store(None);
    std::mem::take(&mut *outcome.machine.0.borrow_mut())
}

/// Run a single ctl sign-in and return the drained world's fake disk for metadata inspection.
#[must_use]
pub fn signed_in_disk() -> skein_fake_machine::Machine {
    run(true)
}

/// Two independent end waits, with a third caller answering `me` and closing their chat.
pub fn waits() {
    run(false);
}
