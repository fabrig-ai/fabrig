//! The native welcome's trial through the program-enabled worker (temper/docs/design/domain/trial.md, 2 and 6).
use super::{Local, Proc, ProgramMachine, host_programs, lay_source, observations, service};
use crate::{
    ctl::{self, Caller, Command},
    fixtures, ready_copies,
    referee::Store,
    stories,
    world::{self, HarnessProcess, Person, Story},
};
use jig_domain_core::Record as Core;
use jig_domain_tasks::{Executor, Stored};
use skein_fake_person::{Find, Step};
use skein_io::kernel::{Exit, ServiceSignal};
use skein_lib::Time;
use skein_shell::Host;
use skein_ui::{Role, Tree};
use skein_world::StartupRoot;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use temper_client_domain::{Address, Step as Welcome};
use temper_engine_domain::Record;
use temper_playground::{
    documents,
    machine::{self, Machine, SignalHost},
    programs,
};

const SOURCE: &str = "claude-owner";
const MODEL: &str = "claude-opus-5-5";
const TRIAL: Executor = Executor::Agent { charter: 10 };
const DEADLINE: Time = Time::from_nanos(15_000_000_000);
// The fake's init takes simulated time before the answering turn.
const INIT_DELAY_MS: u64 = 1100;

thread_local! { static WELCOME: Cell<bool> = const { Cell::new(false) }; }

pub(super) fn active() -> bool {
    WELCOME.with(Cell::get)
}

pub(super) fn configure(document: &mut documents::Configuration) {
    document["world_settings"]["empty_projects"] = true.into();
    let llm = &mut document["world_settings"]["llm"];
    llm["providers"][0] = serde_json::json!({
        "name": "anthropic", "number": 1, "manage_usage": "https://claude.ai/settings/usage",
        "models": [MODEL], "sources": {"programs": ["claude_code"]}
    });
    llm["endpoints"][0]["provider"] = "anthropic".into();
    llm["models"][0]["name"] = MODEL.into();
    document.jig_over_all([("tasks.funders", 6), ("people.projects", 3)]);
}

fn steps() -> Box<[Step<Address>]> {
    let subscription = b"Use your Claude subscription";
    let signed = "I’ve signed in".as_bytes();
    let press = |role, name: &[u8]| Step::Press { find: Find::exactly_named(role, name) };
    let text = |words: &[u8]| Step::See { find: Find::text(words), within: fixtures::PERSON_WAIT };
    Box::new([
        Step::Go { address: Address::Welcome { step: Welcome::Model { way: None } } },
        ready_copies::see(Role::Link, subscription),
        press(Role::Link, subscription),
        ready_copies::see(Role::Button, signed),
        // The button may draw before the opening accounts read is delivered.
        // Let the calm simulator finish that exchange before pressing it.
        Step::Pause { span: skein_lib::Duration::from_millis(1) },
        press(Role::Button, signed),
        ready_copies::ready_heading(),
        ready_copies::see(Role::Button, b"Create your project"),
        Step::Gone {
            find: Find::exactly_named(Role::Heading { level: 1 }, b"Almost ready"),
            within: fixtures::PERSON_WAIT,
        },
        text(b"answered in "),
        text(b"you@example.com"),
    ])
}

fn seconds(tree: &Tree) -> Option<u64> {
    tree.nodes().iter().find_map(|node| {
        let text: Vec<_> = node.text.as_slice().iter().flat_map(|run| run.text.iter().copied()).collect();
        let (_, after) = std::str::from_utf8(&text).ok()?.split_once("answered in ")?;
        let (number, unit) = after.split_once(' ')?;
        let seconds = number.parse().ok()?;
        (unit.split_whitespace().next() == Some("s")).then_some(seconds)
    })
}

struct Judge {
    controls: skein_world::Controls,
    store: Rc<RefCell<Store>>,
    signed: bool,
    opened: bool,
    task: Option<u64>,
    stopped: bool,
}

impl Judge {
    fn trial_ended(&mut self) -> bool {
        let store = self.store.borrow();
        // The trial belongs to the person, not any project a ctl wait could read.
        // Discover its number from a synced charter-10 task row, then join its
        // own ending, even if the person has already seen the answer on Ready.
        if self.task.is_none() {
            self.task = store.rows.values().find_map(|row| match row {
                Record::Core(Core::Tasks(Stored::Live(task) | Stored::Ended(task))) if task.executor == TRIAL => {
                    Some(task.number)
                }
                _ => None,
            });
        }
        self.task.is_some_and(|task| {
            store.rows.contains_key(&temper_engine_domain::Key::Core(jig_domain_core::Key::Tasks(
                jig_domain_tasks::Key::Ended(task),
            )))
        })
    }
}

impl skein_world::Referee<Proc> for Judge {
    fn act(&mut self, _: Time, procs: &mut [Proc]) {
        let [
            Proc::Service(service),
            Proc::Other(Local::World(HarnessProcess::Calls(caller))),
            Proc::Other(Local::World(HarnessProcess::Client(person))),
            peers @ ..,
        ] = procs
        else {
            panic!("trial roots");
        };
        if !self.signed
            && let Some(link) = service.service.links().pop()
        {
            caller.sign_in(&link);
            caller.commands.push_back(Command::new(&[
                b"settings",
                b"apply",
                b"/setup/show-welcome.json",
                b"--against",
                b"1",
                b"--key",
                b"show-welcome",
            ]));
            caller.commands.push_back(Command::new(&[
                b"forge",
                b"connect",
                b"forgejo-admin",
                b"/setup/admin.token",
                b"--api",
                b"127.0.0.1:3000",
                b"--git",
                b"http://127.0.0.1:3000",
                b"--key",
                b"welcome-forge",
            ]));
            caller.commands.push_back(Command::new(&[b"forge", b"list", b"--until", b"connected:1"]));
            self.signed = true;
        }
        if !self.opened && caller.answers.len() == 4 {
            for answer in &caller.answers {
                assert_eq!(answer.exit, Exit::Code(0), "welcome setup: {}", String::from_utf8_lossy(&answer.error));
            }
            caller.closed = true;
            person.saved();
            self.opened = true;
        }
        if !self.stopped && self.trial_ended() && person.finished {
            assert!(seconds(person.tree()).is_some_and(|n| n >= 1), "Ready answers in positive seconds");
            self.controls.signal(0, ServiceSignal::Terminate);
            for proc in peers {
                match proc {
                    Proc::Peer(peer) => peer.shutdown(),
                    Proc::Forge(peer) => peer.shutdown(),
                    _ => {}
                }
            }
            self.stopped = true;
        }
    }
    fn watch(&mut self, _: Time, procs: &[Proc]) -> Result<(), String> {
        let Proc::Service(service) = &procs[0] else { return Err("trial service first".into()) };
        if !self.stopped
            && let Some(end) = service.service.end()
        {
            return Err(format!("service ended before the trial's durable end and Ready: {end:?}"));
        }
        service.observer.finding.clone().map_or(Ok(()), Err)
    }
    fn next_deadline(&self) -> Option<Time> {
        (!self.stopped).then_some(DEADLINE)
    }
    fn overdue(&self, now: Time) -> Option<String> {
        (!self.stopped && now >= DEADLINE).then(|| format!("welcome trial task {:?} did not answer and end", self.task))
    }
    fn passed(&self) -> bool {
        self.stopped
    }
}

/// Follow the Claude subscription from the welcome's controls to an answered trial on Ready.
pub fn welcome_trial() {
    let scenario = Story::Chat { words: b"" };
    stories::set(scenario);
    // The owner signs in and shows the welcome with ctl; the native person runs the trial.
    stories::setup_settings(world::Phase::Restart);
    let store = Rc::new(RefCell::new(Store::new()));
    world::lower_store(Some(store.clone()));
    let mut disk = machine::fresh();
    machine::setup(&mut disk);
    machine::lay_handover(
        &mut disk,
        b"/setup/show-welcome.json",
        br#"{"items":[{"setting":{"kind":"welcome"},"set":{"shown":true}}]}"#,
    );
    lay_source(
        &mut disk,
        SOURCE,
        &smith_fake_claude::Story {
            init: smith_fake_claude::Init { delay_ms: Some(INIT_DELAY_MS), ..smith_fake_claude::Init::default() },
            turns: vec![smith_fake_claude::Turn::new(vec![smith_fake_claude::Step::Text("Yes".into())])],
            ..smith_fake_claude::Story::default()
        },
    );
    let caller = disk.reopen_root(2);
    let native = disk.reopen_root(2);
    let machine = Machine::new(disk);
    let transport = stories::forge(scenario);
    let parameters = world::parameters(&machine, transport.clone());
    WELCOME.with(|active| active.set(true));
    let outcome = programs::with(parameters, || {
        let mut simulation = skein_world::World::new_controlled(
            fixtures::SEED,
            skein_sim::Config::calm(),
            skein_world::Memory::Unchecked,
            |controls| Judge { controls, store, signed: false, opened: false, task: None, stopped: false },
        )
        .with_machine(ProgramMachine(machine));
        host_programs(&mut simulation);
        simulation.host_roots(
            ctl::program(|spawn, inherited| {
                Proc::Other(Local::World(HarnessProcess::Ctl(Box::new(ctl::host(spawn, inherited)))))
            }),
            ctl::roots,
        );
        let mut roots = programs::service_roots();
        roots.push(StartupRoot { name: b"programs_parent".as_slice().into(), path: b"worker".as_slice().into() });
        simulation.spawn_restartable(roots, service);
        simulation.spawn_root(skein_sim::Handle::new(caller.raw()), |root| {
            Proc::Other(Local::World(HarnessProcess::Calls(Box::new(Caller::scripted(root)))))
        });
        simulation.spawn_root(skein_sim::Handle::new(native.raw()), |root| {
            Proc::Other(Local::World(HarnessProcess::Client(Box::new(SignalHost::new(
                Person::welcome(root, steps()),
                None,
            )))))
        });
        simulation.spawn(|| Proc::Peer(Box::new(SignalHost::new(fixtures::provider().keeping_bodies(), None))));
        simulation.spawn(|| {
            Proc::Forge(Box::new(SignalHost::new(
                temper_playground::fakes::forge_peer(&transport, fixtures::SEED),
                None,
            )))
        });
        simulation.run()
    });
    WELCOME.with(|active| active.set(false));
    world::lower_store(None);
    let [Proc::Service(service), _, _, Proc::Peer(peer), ..] = outcome.procs.as_slice() else { panic!("trial roots") };
    assert_eq!(service.service.end(), Some(&temper_service::End::Engine(temper_service::EngineEnd::Drained)));
    assert!(outcome.procs.iter().all(Host::is_empty), "welcome world drains every process");
    assert!(peer.observations().is_empty() && peer.bodies().is_empty(), "no direct model request");
    let bytes = observations(&outcome.machine, SOURCE);
    let prints: Vec<_> = smith_fake_claude::observations(&bytes)
        .into_iter()
        .filter(|seen| seen.arguments.first().is_some_and(|arg| arg == "-p"))
        .collect();
    let [print] = prints.as_slice() else { panic!("exactly one -p invocation: {prints:?}") };
    assert!(print.arguments.windows(2).any(|args| args == ["--model", MODEL]), "stock model: {print:?}");
    assert_eq!(
        smith_fake_claude::selections(&bytes).first(),
        Some(&smith_fake_claude::Seen::Selected { script: smith_fake_claude::Selection::Otherwise, turn: 0 })
    );
}
