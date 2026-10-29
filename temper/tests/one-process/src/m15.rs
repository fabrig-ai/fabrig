//! Exact secret-boundary observations on the simulated service (temper/docs/design/domain/testing.md, 2.5).
pub(crate) mod observer;
mod pages;
mod run;
use crate::{
    ctl::{Caller, Command},
    fixtures,
    lower::github,
    machine::{Machine, SignalHost},
    referee::{self, Store},
    stories,
    world::{self, Person, Proc, Story},
};
pub use observer::Counts;
pub use run::delivery;
use skein_io::kernel::Exit;
use skein_lib::Time;
use skein_shell::Host;
use std::{cell::RefCell, rc::Rc};

fn paste(app: u64, key: &[u8]) -> Command {
    Command::new(&[
        b"forge",
        b"connect",
        b"1",
        b"github-app",
        app.to_string().as_bytes(),
        b"/setup/app.pem",
        b"--key",
        key,
    ])
}
fn commands(caller: &mut Caller, link: Box<[u8]>, app: u64) -> usize {
    let mut sign_in = Command::new(&[b"sign-in", b"-"]);
    sign_in.input = link;
    caller.commands.push_back(sign_in);
    caller.commands.push_back(paste(u64::MAX, b"failed-paste"));
    caller.commands.push_back(Command::new(&[b"forge", b"list", b"--until", b"settled", b"--json"]));
    caller.commands.push_back(paste(u64::MAX, b"failed-paste"));
    caller.commands.push_back(paste(app, b"good-paste"));
    caller.commands.push_back(Command::new(&[b"forge", b"list", b"--until", b"connected:1", b"--json"]));
    caller.commands.push_back(paste(app, b"good-paste"));
    caller.commands.push_back(paste(app, b"replacement-refused"));
    caller.commands.push_back(Command::new(&[b"forge", b"install", b"1", b"--key", b"secret-install"]));
    caller.commands.push_back(Command::browse(8));
    caller.commands.push_back(Command::new(&[b"forge", b"list", b"--until", b"settled", b"--json"]));
    11
}
struct Judge {
    controls: skein_world::Controls,
    store: Rc<RefCell<Store>>,
    app: u64,
    calls: usize,
    handed: bool,
    browsing: bool,
    stopping: bool,
}
impl skein_world::Referee<Proc> for Judge {
    fn act(&mut self, _now: Time, procs: &mut [Proc]) {
        let (first, other) = procs.split_at_mut(1);
        let Proc::Service(service) = &mut first[0] else { panic!("service first") };
        let Proc::Calls(caller) = &mut other[0] else { panic!("native caller second") };
        if !self.handed
            && let Some(link) = service.service.links().pop()
        {
            self.calls = commands(caller, link, self.app);
            self.handed = true;
        }
        if !self.handed || caller.answers.len() != self.calls {
            return;
        }
        if !self.browsing {
            let Proc::Calls(caller) = &other[0] else { unreachable!() };
            let peer = other
                .iter()
                .find_map(|proc| if let Proc::Forge(peer) = proc { Some(peer) } else { None })
                .expect("GitHub peer");
            assert!(peer.git_calls().expect("git observation").is_empty(), "pastes start no git writes");
            assert_eq!(peer.github_layer().expect("GitHub").installations().iter().count(), 1);
            for (index, answer) in caller.answers.iter().enumerate() {
                assert!(
                    answer.exit == Exit::Code(if index == 7 { 2 } else { 0 }),
                    "paste call {index} settles as scripted"
                );
                observer::artifact("ctl stdout", &answer.output);
                observer::artifact("ctl stderr", &answer.error);
            }
            assert!(caller.answers[1].output == caller.answers[3].output, "failed paste replay keeps its run");
            assert!(caller.answers[4].output == caller.answers[6].output, "good paste replay keeps its run");
            checkpoint(service, &self.store.borrow());
            for proc in other.iter_mut() {
                if let Proc::Client(person) = proc {
                    person.saved();
                }
            }
            self.browsing = true;
        }
        if other.iter().any(|proc| matches!(proc, Proc::Client(person) if person.finished)) && !self.stopping {
            let peer = other
                .iter()
                .find_map(|proc| if let Proc::Forge(peer) = proc { Some(peer) } else { None })
                .expect("GitHub peer");
            assert_eq!(observer::tokens() as u64, peer.github_layer().expect("GitHub").installations().minted());
            checkpoint(service, &self.store.borrow());
            let Proc::Calls(caller) = &mut other[0] else { unreachable!() };
            caller.closed = true;
            self.controls.signal(0, skein_io::kernel::ServiceSignal::Terminate);
            for proc in other {
                referee::shutdown_peer(proc);
            }
            self.stopping = true;
        }
    }
    fn observe(&mut self, _now: Time, procs: &[Proc]) {
        for proc in procs {
            if let Proc::Peer(peer) = proc {
                observer::context(peer);
            }
        }
    }
    fn next_deadline(&self) -> Option<Time> {
        (!self.stopping).then_some(Time::from_nanos(10_000_000_000))
    }
    fn overdue(&self, now: Time) -> Option<String> {
        (!self.stopping && now >= Time::from_nanos(10_000_000_000))
            .then(|| format!("M15 pastes: handed {}, browsing {}", self.handed, self.browsing))
    }
    fn passed(&self) -> bool {
        self.stopping
    }
}
fn checkpoint(service: &world::ServiceHost, store: &Store) {
    observer::artifact("public roots", format!("{:?}", referee::secrets::Roots(&service.service)).as_bytes());
    observer::artifact("nonsecret records", format!("{:?}", store.rows).as_bytes());
    observer::checkpoint();
}
/// Refused and replayed pasted keys remain absent from every public surface in this story.
#[must_use]
pub fn refused_replayed_pastes() -> Counts {
    observer::start();
    github::set_active(true);
    stories::set(Story::Adopt);
    stories::setup_settings(world::Phase::Story);
    let store = Rc::new(RefCell::new(Store::new()));
    world::lower_store(Some(store.clone()));
    let (_forge, mut peer) = github::peer();
    let (app, files) = observer::input(&mut peer);
    let mut disk = crate::machine::fresh();
    let input = disk.lay(&files);
    disk.close(input);
    let caller = disk.reopen_root(2);
    let native = disk.reopen_root(2);
    let mut world = skein_world::World::new_controlled(
        fixtures::SEED,
        skein_sim::Config::calm(),
        skein_world::Memory::Unchecked,
        |controls| Judge { controls, store, app, calls: 0, handed: false, browsing: false, stopping: false },
    )
    .with_machine(Machine(disk));
    world::host_programs(&mut world);
    world.spawn_restartable(world::service_roots(), world::service);
    world.spawn_root(skein_sim::Handle::new(caller.raw()), |root| Proc::Calls(Box::new(Caller::scripted(root))));
    world.spawn(|| Proc::Forge(Box::new(SignalHost::new(peer, None))));
    world.spawn_root(skein_sim::Handle::new(native.raw()), |root| {
        Proc::Client(Box::new(SignalHost::new(Person::pages(root, pages::steps()), None)))
    });
    let mut outcome = world.run();
    assert!(outcome.procs.iter().all(Host::is_empty), "M15 processes drain");
    let counts = observer::finish(&mut outcome.machine.0);
    world::lower_store(None);
    github::set_active(false);
    counts
}

pub(crate) fn configure(value: &mut serde_json::Value) {
    if !observer::enabled() {
        return;
    }
    for project in value["world_settings"]["policies"].as_array_mut().expect("world policies") {
        for pointer in ["/policy/ceiling/grants", "/policy/roles/0/authority/grants"] {
            let grants = project.pointer_mut(pointer).expect("world grants");
            let prototype = grants[0].clone();
            *grants = temper_engine_domain_forge::CHANGE_KINDS
                .into_iter()
                .map(|kind| {
                    let mut grant = prototype.clone();
                    grant["kind"] = kind.into();
                    grant
                })
                .collect();
        }
    }
    value["engine"]["limits"]["overrides"]["tasks.authority_grants"] = 8.into();
    value["engine"]["limits"]["overrides"]["authority.grants"] = 8.into();
    value["engine"]["limits"]["overrides"]["notes.scopes"] = 12.into();
    value["engine"]["limits"]["overrides"]["people.amendment_bytes"] = 2048.into();
    value["engine"]["limits"]["overrides"]["tasks.authority_bytes"] = 256.into();
}
