//! Exact secret-boundary observations on the simulated service (temper/docs/design/testing/README.md, 2.5).
use temper_playground::{
    machine::{Machine, SignalHost},
    programs,
};
pub(crate) mod observer;
mod pages;
mod run;
use crate::{
    ctl::{Caller, Command},
    fixtures,
    lower::github,
    referee::{self, Store},
    stories,
    world::{self, HarnessProcess, OneProcess, Person, Proc, Story},
};
pub use observer::Counts;
pub use run::delivery;
use skein_io::kernel::Exit;
use skein_lib::Time;
use skein_shell::Host;
use std::{cell::RefCell, rc::Rc};

fn paste(app: u64, key: &[u8], fresh: bool) -> Command {
    let app = app.to_string();
    let mut words: Vec<&[u8]> =
        vec![b"forge", b"connect", b"github-app", app.as_bytes(), b"/setup/app.pem", b"--key", key];
    if !fresh {
        words.insert(2, b"1");
    }
    Command::new(&words)
}

fn commands(caller: &mut Caller, link: Box<[u8]>, app: u64) -> usize {
    let mut sign_in = Command::new(&[b"sign-in", b"-"]);
    sign_in.input = link;
    caller.commands.push_back(sign_in);
    caller.commands.push_back(paste(u64::MAX, b"failed-paste", true));
    caller.commands.push_back(Command::new(&[b"forge", b"list", b"--until", b"settled", b"--json"]));
    caller.commands.push_back(paste(u64::MAX, b"failed-paste", true));
    caller.commands.push_back(paste(app, b"good-paste", false));
    caller.commands.push_back(Command::new(&[b"forge", b"list", b"--until", b"connected:1", b"--json"]));
    caller.commands.push_back(paste(app, b"good-paste", false));
    caller.commands.push_back(paste(app, b"replacement-refused", false));
    caller.commands.push_back(Command::new(&[b"forge", b"install", b"1", b"--key", b"secret-install"]));
    caller.commands.push_back(Command::browse(8));
    caller.commands.push_back(Command::new(&[b"forge", b"list", b"--until", b"settled", b"--json"]));
    11
}
struct Judge {
    finding: Option<String>,
    controls: skein_world::Controls,
    store: Rc<RefCell<Store>>,
    app: u64,
    calls: usize,
    handed: bool,
    browsing: bool,
    stopping: bool,
}
impl Judge {
    fn act_checked(&mut self, _now: Time, procs: &mut [Proc]) -> Result<(), String> {
        let (first, other) = procs.split_at_mut(1);
        let Proc::Service(service) = &mut first[0] else { return Err(String::from("service first")) };
        let Proc::Other(HarnessProcess::Calls(caller)) = &mut other[0] else {
            return Err(String::from("native caller second"));
        };
        if !self.handed
            && let Some(link) = service.service.links().pop()
        {
            self.calls = commands(caller, link, self.app);
            self.handed = true;
        }
        if !self.handed || caller.answers.len() != self.calls {
            return Ok(());
        }
        if !self.browsing {
            let Proc::Other(HarnessProcess::Calls(caller)) = &other[0] else { return Err(String::from("unreachable")) };
            let peer = other
                .iter()
                .find_map(|proc| if let Proc::Forge(peer) = proc { Some(peer) } else { None })
                .ok_or_else(|| String::from("GitHub peer"))?;
            if !(peer.git_calls().ok_or_else(|| String::from("git observation"))?.is_empty()) {
                return Err(String::from("pastes start no git writes"));
            }
            match (&(peer.github_layer().ok_or_else(|| String::from("GitHub"))?.installations().iter().count()), &(1)) {
                (left, right) if left != right => {
                    let finding = String::from("assertion `left == right` failed");
                    return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                }
                _ => {}
            }
            for (index, answer) in caller.answers.iter().enumerate() {
                if answer.exit != Exit::Code(if index == 7 { 2 } else { 0 }) {
                    return Err(format!("paste call {index} settles as scripted"));
                }
                observer::artifact("ctl stdout", &answer.output);
                observer::artifact("ctl stderr", &answer.error);
            }
            if caller.answers[1].output != caller.answers[3].output {
                return Err(String::from("failed paste replay keeps its run"));
            }
            if caller.answers[4].output != caller.answers[6].output {
                return Err(String::from("good paste replay keeps its run"));
            }
            checkpoint(service, &self.store.borrow())?;
            for proc in other.iter_mut() {
                if let Proc::Other(HarnessProcess::Client(person)) = proc {
                    person.saved();
                }
            }
            self.browsing = true;
        }
        if other.iter().any(|proc| matches!(proc, Proc::Other(HarnessProcess::Client(person)) if person.finished))
            && !self.stopping
        {
            let peer = other
                .iter()
                .find_map(|proc| if let Proc::Forge(peer) = proc { Some(peer) } else { None })
                .ok_or_else(|| String::from("GitHub peer"))?;
            match (
                &(observer::tokens() as u64),
                &(peer.github_layer().ok_or_else(|| String::from("GitHub"))?.installations().minted()),
            ) {
                (left, right) if left != right => {
                    let finding = String::from("assertion `left == right` failed");
                    return Err(format!("{finding}\n left: {left:?}\nright: {right:?}"));
                }
                _ => {}
            }
            checkpoint(service, &self.store.borrow())?;
            let Proc::Other(HarnessProcess::Calls(caller)) = &mut other[0] else {
                return Err(String::from("unreachable"));
            };
            caller.closed = true;
            self.controls.signal(0, skein_io::kernel::ServiceSignal::Terminate);
            for proc in other {
                referee::shutdown_peer(proc);
            }
            self.stopping = true;
        }

        Ok(())
    }
}
impl skein_world::Referee<Proc> for Judge {
    fn act(&mut self, now: Time, procs: &mut [Proc]) {
        if self.finding.is_none() {
            self.finding = self.act_checked(now, procs).err();
        }
    }
    fn watch(&mut self, _now: Time, procs: &[Proc]) -> Result<(), String> {
        if let Some(why) = &self.finding {
            return Err(why.clone());
        }
        for proc in procs {
            if let Proc::Service(service) = proc
                && let Some(why) = &service.observer.finding
            {
                return Err(why.clone());
            }
        }

        for proc in procs {
            if let Proc::Peer(peer) = proc {
                observer::context(peer)?;
            }
        }

        Ok(())
    }
    fn next_deadline(&self) -> Option<Time> {
        (!self.stopping).then_some(Time::from_nanos(10_000_000_000))
    }
    fn overdue(&self, now: Time) -> Option<String> {
        (!self.stopping && now >= Time::from_nanos(10_000_000_000))
            .then(|| format!("M15 pastes: handed {}, browsing {}", self.handed, self.browsing))
    }
    fn passed(&self) -> bool {
        self.finding.is_none() && (self.stopping)
    }
}
fn checkpoint(service: &world::ServiceHost, store: &Store) -> Result<(), String> {
    observer::artifact("public roots", format!("{:?}", referee::secrets::Roots(&service.service)).as_bytes());
    observer::artifact("nonsecret records", format!("{:?}", store.rows).as_bytes());
    observer::checkpoint()?;

    Ok(())
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
    let transport = stories::forge(Story::Adopt);
    let mut disk = temper_playground::machine::fresh();
    let input = disk.lay(&files);
    disk.close(input);
    let caller = disk.reopen_root(2);
    let native = disk.reopen_root(2);
    let machine = Machine::new(disk);
    let parameters = world::parameters(&machine, transport.clone());
    let outcome = programs::with(parameters, || {
        let mut world = skein_world::World::new_controlled(
            fixtures::SEED,
            skein_sim::Config::calm(),
            skein_world::Memory::Unchecked,
            |controls| Judge {
                finding: None,
                controls,
                store,
                app,
                calls: 0,
                handed: false,
                browsing: false,
                stopping: false,
            },
        )
        .with_machine(machine);
        for (program, roots) in programs::programs::<OneProcess>() {
            world.host_roots(program, roots);
        }
        world.host_roots(
            crate::ctl::program(|spawn, inherited| {
                Proc::Other(HarnessProcess::Ctl(Box::new(crate::ctl::host(spawn, inherited))))
            }),
            crate::ctl::roots,
        );
        world.spawn_restartable(programs::service_roots(), programs::service::<OneProcess>);
        world.spawn_root(skein_sim::Handle::new(caller.raw()), |root| {
            Proc::Other(HarnessProcess::Calls(Box::new(Caller::scripted(root))))
        });
        world.spawn(|| Proc::Forge(Box::new(SignalHost::new(peer, None))));
        world.spawn_root(skein_sim::Handle::new(native.raw()), |root| {
            Proc::Other(HarnessProcess::Client(Box::new(SignalHost::new(Person::pages(root, pages::steps()), None))))
        });
        world.run()
    });
    assert!(outcome.procs.iter().all(Host::is_empty), "M15 processes drain");
    let counts = observer::finish(&mut outcome.machine.0.borrow_mut());
    world::lower_store(None);
    github::set_active(false);
    counts
}

// The delivery story pushes, opens and merges; a change also holds its branch's re-run kind.
const DELIVERY_KINDS: [u16; 4] = [2, 3, 4, 17];

pub(crate) fn configure(value: &mut serde_json::Value) {
    if !observer::enabled() {
        return;
    }
    for project in value["world_settings"]["policies"].as_array_mut().expect("world policies") {
        for pointer in ["/policy/ceiling/grants", "/policy/roles/0/authority/grants"] {
            let grants = project.pointer_mut(pointer).expect("world grants");
            let prototype = grants[0].clone();
            *grants = DELIVERY_KINDS
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
