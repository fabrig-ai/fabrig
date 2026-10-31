//! Public readback over the service's cold restart (temper testing.md, 2.5).
use crate::{
    ctl::{Caller, Command},
    machine::{Machine, SignalHost},
    referee::{self, Store},
    stories,
    world::{self, Proc, Settings, Story},
};
use serde_json::Value;
use skein_io::kernel::Exit;
use skein_lib::Time;
use skein_shell::Host;
use std::{cell::RefCell, rc::Rc};

// These are the public documents read by the adoption world's native person and setup.
const READS: &[&[&[u8]]] = &[
    &[b"me", b"--json"],
    &[b"inbox", b"--json"],
    &[b"project", b"show", b"1", b"--json"],
    &[b"settings", b"show", b"--json"],
    &[b"forge", b"list", b"--until", b"connected:1", b"--json"],
];
#[derive(Default)]
struct State {
    active: bool,
    sent: bool,
    before: Vec<Value>,
    link: Box<[u8]>,
}
thread_local! { static STATE: RefCell<State> = RefCell::new(State::default()); }

pub(crate) fn begin() {
    STATE.with(|state| *state.borrow_mut() = State { active: true, ..State::default() });
}
pub(crate) fn linked(link: &[u8]) {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        if state.active {
            state.link = link.into();
        }
    });
}
fn commands(caller: &mut Caller) {
    for words in READS {
        caller.commands.push_back(Command::new(words));
    }
    caller.closed = true;
}
fn documents(caller: &Caller, phase: &str, skip: usize) -> Vec<Value> {
    assert_eq!(caller.answers.len(), READS.len() + skip, "every oracle document answers");
    caller
        .answers
        .iter()
        .skip(skip)
        .zip(READS)
        .map(|(answer, read)| {
            assert_eq!(
                answer.exit,
                Exit::Code(0),
                "oracle {phase} read {read:?}: {}",
                String::from_utf8_lossy(&answer.error)
            );
            assert!(answer.error.is_empty(), "oracle read has no diagnostic");
            serde_json::from_slice(&answer.output).expect("public document JSON")
        })
        .collect()
}
/// Hold normal shutdown until the live service has answered every public read.
pub(crate) fn readback(procs: &mut [Proc]) -> bool {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        if !state.active {
            return true;
        }
        let caller = procs
            .iter_mut()
            .find_map(|proc| if let Proc::Calls(caller) = proc { Some(caller) } else { None })
            .expect("crash world has an oracle caller");
        if !state.sent {
            assert!(!state.link.is_empty(), "oracle has the printed sign-in link");
            let mut sign_in = Command::new(&[b"sign-in", b"-"]);
            sign_in.input.clone_from(&state.link);
            caller.commands.push_back(sign_in);
            commands(caller);
            state.sent = true;
        }
        if !caller.is_empty() {
            return false;
        }
        state.before = documents(caller, "live", 1);
        true
    })
}
struct Judge {
    controls: skein_world::Controls,
    before: Vec<Value>,
    sent: bool,
    stopping: bool,
}
impl skein_world::Referee<Proc> for Judge {
    fn act(&mut self, _: Time, procs: &mut [Proc]) {
        let (first, other) = procs.split_at_mut(1);
        let Proc::Service(service) = &mut first[0] else { panic!("service first") };
        let Proc::Calls(caller) = &mut other[0] else { panic!("oracle caller second") };
        if !self.sent && !service.service.links().is_empty() {
            commands(caller);
            self.sent = true;
        }
        if self.sent && caller.is_empty() && !self.stopping {
            for ((before, after), read) in self.before.iter().zip(documents(caller, "cold", 0)).zip(READS) {
                assert_eq!(*before, after, "restart oracle public document {read:?}");
            }
            self.controls.signal(0, skein_io::kernel::ServiceSignal::Terminate);
            self.stopping = true;
        }
        if self.stopping {
            for proc in other {
                referee::shutdown_peer(proc);
            }
        }
    }
    fn observe(&mut self, _: Time, _: &[Proc]) {}
    fn passed(&self) -> bool {
        self.stopping
    }
}

pub(crate) fn cold(
    settings: Settings,
    mut disk: skein_fake_machine::Machine,
    mut previous: Store,
    run: &Rc<RefCell<referee::Run>>,
) {
    let before = STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.active = false;
        std::mem::take(&mut state.before)
    });
    assert_eq!(before.len(), READS.len(), "live documents precede cold readback");
    previous.recovering();
    let store = Rc::new(RefCell::new(previous));
    world::lower_store(Some(store));
    world::share(run.borrow().forge.clone());
    let client = disk.reopen_root(2);
    let mut world = skein_world::World::new_controlled(
        settings.seed,
        settings.config,
        skein_world::Memory::Unchecked,
        |controls| Judge { controls, before, sent: false, stopping: false },
    )
    .with_machine(Machine(disk));
    world::host_programs(&mut world);
    world.spawn_restartable(world::service_roots(), world::service);
    world.spawn_root(skein_sim::Handle::new(client.raw()), |root| Proc::Calls(Box::new(Caller::oracle(root))));
    world.spawn(|| Proc::Peer(Box::new(SignalHost::new(stories::provider(Story::Adoption), None))));
    world.spawn(|| Proc::Forge(Box::new(SignalHost::new(world::peer(), None))));
    let outcome = world.run();
    assert!(outcome.procs.iter().all(Host::is_empty), "cold public readback drains every process");
    world::lower_store(None);
    world::share(None);
}

pub(crate) fn caller(inherited: &skein_world::Inherited) -> Proc {
    Proc::Calls(Box::new(Caller::restartable(inherited)))
}

pub(crate) fn spawn<R: skein_world::Referee<Proc>>(world: &mut skein_world::World<Proc, R, Machine>) {
    world.spawn_restartable(
        vec![skein_world::StartupRoot { name: b"oracle".as_slice().into(), path: b"client".as_slice().into() }],
        caller,
    );
}
