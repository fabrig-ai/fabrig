//! The native line shell on a spent link and its replacement (temper/docs/design/testing/tiers.md, 1.5).
use super::{HarnessProcess, OneProcess, Phase, Proc, Settings, Story, lower_store, parameters};
use crate::{
    ctl::{self, Caller, Command},
    fixtures,
    referee::{self, Store},
    stories,
};
use skein_fake_machine::{Facts, How, Refusal};
use skein_io::kernel::{Exit, Fd, ServiceSignal, Spawn};
use skein_lib::Time;
use skein_shell::Host;
use skein_world::{Controls, HostedProgram, Inherited, Referee, StartupRoot};
use std::{cell::RefCell, rc::Rc};
use temper_client_shell_terminal::{
    address::{self, Target},
    client::{NativeClient, Settings as ClientSettings, Start},
    limits,
    terminal::{Loop, Terminal},
};
use temper_playground::{
    machine::{self, Machine, SignalHost},
    programs,
};

const PROGRAM: &[u8] = b"/world/temper-open";
const DIRECTORY: &[u8] = b"line-private";
const USER: u32 = 1000;
const TERMINAL_SEED: u64 = 0x7465_726d_696e_616c;

fn shell(
    engine: skein_io::Addr,
    start: Start,
    link: Option<skein_lib::Secret>,
    [parent, stdin, stdout, signals]: [Fd; 4],
) -> Terminal {
    let bounds = limits::native();
    let (view, _) = limits::view(&bounds).expect("native view bounds");
    let client = NativeClient::new(
        ClientSettings { engine, directory: DIRECTORY.into(), limits: bounds },
        parent,
        USER,
        TERMINAL_SEED,
        start,
    )
    .unwrap_or_else(|_| panic!("the world's terminal arguments fit"));
    let state = Loop::new(&limits::terminal(), &view.tree, DIRECTORY.into(), link);
    Terminal::new(client, state, stdin, stdout, signals).expect("terminal descriptors")
}

fn terminal(spawn: &Spawn, inherited: &Inherited) -> Proc {
    assert_eq!(spawn.args[0].as_ref(), DIRECTORY);
    let Target::Link { engine, link } = address::target(&spawn.args[1]).unwrap_or_else(|_| panic!("service link"))
    else {
        panic!("the service printed a link, not an address")
    };
    let pipe = |child| inherited.pipes.iter().find(|(at, _)| *at == child).expect("terminal pipe").1;
    Proc::Other(HarnessProcess::Line(Box::new(shell(
        skein_io::Addr::Ip(engine),
        Start::Link(spawn.args[1].clone()),
        Some(link),
        [inherited.roots[0].1, pipe(0), pipe(1), inherited.signal],
    ))))
}

fn program() -> HostedProgram<Proc> {
    // Unused descriptors here: the composition sizes its own reservation before admission.
    let operations =
        shell(skein_io::Addr::Ip(fixtures::CLIENTS), Start::Token, None, [3, 0, 1, 2].map(Fd::new)).operations();
    HostedProgram { program: PROGRAM.into(), make: terminal, instances: 1, operations }
}

fn roots(_: &Spawn) -> Vec<StartupRoot> {
    vec![StartupRoot { name: b"parent".as_slice().into(), path: b"client".as_slice().into() }]
}

enum Stage {
    Link,
    Ctl(Box<[u8]>),
    Refusal(Box<[u8]>),
    Live,
    Quit,
    Done,
}
struct Judge {
    controls: Controls,
    stage: Stage,
    finding: Option<String>,
}

impl Judge {
    fn advance(&mut self, procs: &mut [Proc]) -> Result<(), String> {
        let [Proc::Service(service), Proc::Other(HarnessProcess::Calls(caller)), provider, ..] = procs else {
            return Err(String::from("expected the service, caller and provider"));
        };
        match &self.stage {
            Stage::Link => {
                if let Some(link) = service.service.links().pop() {
                    caller.commands.push_back(Command::sign_in(&link));
                    caller.commands.push_back(Command::new(&[
                        b"account",
                        b"add-key",
                        b"1",
                        b"/setup/llm.key",
                        b"--deployment",
                        b"--key",
                        b"fixture-llm",
                    ]));
                    self.stage = Stage::Ctl(link);
                }
            }
            Stage::Ctl(link) if caller.answers.len() == 2 => {
                if !caller.answers.iter().all(|answer| answer.exit == Exit::Code(0)) {
                    return Err(String::from("ctl must sign in and provision the model key before the first open"));
                }
                if !service.service.links().is_empty() {
                    return Err(String::from("spending the first link must not print a replacement yet"));
                }
                caller.commands.push_back(Command::interactive(PROGRAM, DIRECTORY, vec![link.clone()]));
                self.stage = Stage::Refusal(link.clone());
            }
            Stage::Refusal(spent) if caller.answers.len() == 3 => {
                if caller.answers[2].exit != Exit::Code(1) {
                    return Err(String::from("the first open must refuse the spent link"));
                }
                if let Some(fresh) = service.service.links().pop() {
                    if fresh == *spent {
                        return Err(String::from("the replacement link must be fresh"));
                    }
                    caller.commands.push_back(Command::interactive(PROGRAM, DIRECTORY, vec![fresh]));
                    self.stage = Stage::Live;
                }
            }
            Stage::Live => {
                if caller.answers.len() == 4 {
                    return Err(String::from("the replacement terminal exited before q"));
                }
                if live_page(caller.output())? {
                    caller.type_line(b"q\n")?;
                    self.stage = Stage::Quit;
                }
            }
            Stage::Quit if caller.answers.len() == 4 => {
                caller.closed = true;
                self.controls.signal(0, ServiceSignal::Terminate);
                referee::shutdown_peer(provider);
                self.stage = Stage::Done;
            }
            Stage::Ctl(_) | Stage::Refusal(_) | Stage::Quit | Stage::Done => {}
        }
        Ok(())
    }
}
fn live_page(output: &[u8]) -> Result<bool, String> {
    let Some(end) = output.iter().rposition(|&byte| byte == b'\n') else { return Ok(false) };
    let text = std::str::from_utf8(&output[..end]).map_err(|error| error.to_string())?;
    Ok(text.lines().any(|line| {
        line.strip_prefix("== ")
            .and_then(|page| page.split_once(" == "))
            .is_some_and(|(_, status)| status.starts_with("Live"))
    }))
}

impl Referee<Proc> for Judge {
    fn act(&mut self, _: Time, procs: &mut [Proc]) {
        if self.finding.is_none() {
            self.finding = self.advance(procs).err();
        }
    }
    fn watch(&mut self, _: Time, procs: &[Proc]) -> Result<(), String> {
        if let Some(why) = &self.finding {
            return Err(why.clone());
        }
        if let Some(Proc::Service(service)) = procs.first()
            && let Some(why) = &service.observer.finding
        {
            return Err(why.clone());
        }
        Ok(())
    }
    fn next_deadline(&self) -> Option<Time> {
        (!self.passed()).then(|| Time::ZERO.saturating_add(fixtures::PERSON_WAIT))
    }
    fn overdue(&self, now: Time) -> Option<String> {
        self.next_deadline().filter(|&end| now >= end).map(|_| {
            let step = match self.stage {
                Stage::Link => "initial printed link",
                Stage::Ctl(_) => "ctl sign-in and key provisioning",
                Stage::Refusal(_) => "refusal and fresh link",
                Stage::Live => "live page after the fresh link",
                Stage::Quit => "terminal drain after q",
                Stage::Done => "all processes drained",
            };
            format!("waiting for {step}")
        })
    }
    fn passed(&self) -> bool {
        matches!(self.stage, Stage::Done) && self.finding.is_none()
    }
}

pub struct LineSignIn {
    pub ctl: Exit,
    pub key_added: Exit,
    pub opens: Vec<(Exit, Option<Box<[u8]>>)>,
    pub directory: Facts,
    pub token: Facts,
}

/// Spend the service's link in `ctl`, then host a separate line shell on each link.
#[must_use]
pub fn line_sign_in(settings: Settings) -> LineSignIn {
    stories::set(Story::Chat { words: b"Hello" });
    stories::setup_settings(Phase::SignIn);
    let mut disk = machine::fresh(); // The fake machine and native client use the same user.
    machine::setup(&mut disk);
    let root = disk.reopen_root(2);
    assert_eq!(disk.open(root, DIRECTORY, How::DirectoryNoFollow), Err(Refusal::NotFound));
    let machine = Machine::new(disk);
    let parameters = parameters(&machine, stories::forge(Story::Chat { words: b"Hello" }));
    lower_store(Some(Rc::new(RefCell::new(Store::new()))));
    let outcome = programs::with(parameters, || {
        let mut world =
            skein_world::World::new_controlled(settings.seed, settings.config, settings.memory, |controls| Judge {
                controls,
                stage: Stage::Link,
                finding: None,
            })
            .with_machine(machine);
        for (program, roots) in programs::programs::<OneProcess>() {
            world.host_roots(program, roots);
        }
        world.host_roots(
            ctl::program(|spawn, inherited| Proc::Other(HarnessProcess::Ctl(Box::new(ctl::host(spawn, inherited))))),
            ctl::roots,
        );
        world.host_roots(program(), roots);
        world.spawn_restartable(programs::service_roots(), programs::service::<OneProcess>);
        world.spawn_root(skein_sim::Handle::new(root.raw()), |root| {
            Proc::Other(HarnessProcess::Calls(Box::new(Caller::scripted(root))))
        });
        world.spawn(|| Proc::Peer(Box::new(SignalHost::new(fixtures::provider(), None))));
        world.try_run()
    });
    lower_store(None);
    let outcome = outcome.unwrap_or_else(|why| panic!("{why}"));
    assert!(outcome.procs.iter().all(Host::is_empty), "service, caller and both shells drain");
    let Proc::Other(HarnessProcess::Calls(caller)) = &outcome.procs[1] else { panic!("caller") };
    let opens = outcome
        .procs
        .iter()
        .filter_map(|proc| match proc {
            Proc::Other(HarnessProcess::Line(line)) => {
                Some((line.exit().expect("drained line shell"), line.exit_line()))
            }
            _ => None,
        })
        .collect();
    let mut disk = outcome.machine.0.borrow_mut();
    let parent = disk.reopen_root(2);
    let directory = disk.open(parent, DIRECTORY, How::DirectoryNoFollow).expect("terminal directory");
    let token = disk.open(directory, b"token", How::ReadNoFollow).expect("kept terminal token");
    let seen = LineSignIn {
        ctl: caller.answers[0].exit,
        key_added: caller.answers[1].exit,
        opens,
        directory: disk.stat(directory),
        token: disk.stat(token),
    };
    disk.close(token);
    disk.close(directory);
    disk.close(parent);
    seen
}
