//! The visitor's deployment on the world's machine (temper/docs/design/testing/README.md, 2.5).
use super::{checks, proxy::Proxy, visitor::Visitor};
use crate::{
    ctl::{Caller, Command},
    fixtures,
    referee::Store,
    stories,
    world::{self, HarnessProcess, Story},
};
use skein_io::kernel::{Complete, Done, Exit, Op, Submit};
use skein_lib::{Queue, Time, Wall};
use skein_shell::Host;
use std::{cell::RefCell, rc::Rc};
use temper_playground::{
    machine::Machine,
    programs::{self, Harness},
};

thread_local! { static BEARER: RefCell<Option<Box<[u8]>>> = const { RefCell::new(None) }; }
pub(super) fn bearer() -> Box<[u8]> {
    BEARER.with(|value| value.borrow().as_ref().expect("native signed-in request observed").clone())
}
pub(super) struct Deployment;
pub(super) type Proc = programs::Process<Deployment>;
impl Harness for Deployment {
    type Service = super::observation::Observer;
    type Process = Extra;
    fn complete(role: programs::Role, complete: &Complete) {
        if role != programs::Role::Other {
            return;
        }
        if let (Op::Send { bytes, from, .. }, Ok(Done::Count(count))) = (&complete.kind, &complete.result) {
            let from = usize::try_from(*from).expect("send offset");
            let count = usize::try_from(*count).expect("send count");
            for line in bytes[from..from + count].split(|byte| *byte == b'\n') {
                if let Some(colon) = line.iter().position(|byte| *byte == b':')
                    && line[..colon].eq_ignore_ascii_case(b"authorization")
                {
                    let value = &line[colon + 1..];
                    let value = value.strip_prefix(b" ").unwrap_or(value);
                    if let Some(token) = value.strip_prefix(b"Bearer ") {
                        BEARER
                            .with(|held| *held.borrow_mut() = Some(token.strip_suffix(b"\r").unwrap_or(token).into()));
                    }
                }
            }
        }
    }
    fn seeds(_: u64) -> temper_service::settings::Seeds {
        fixtures::seeds()
    }
    fn agent_seed(_: u64) -> u64 {
        fixtures::SEED
    }
    fn files() -> temper_service::settings::Files {
        fixtures::files()
    }
    fn agent_profile() -> smith_agent_service::profile::Profile {
        agent_profile()
    }
    fn agent_settings() -> temper_service::settings::Agent {
        agent_settings()
    }
    fn given(document: &serde_json::Value) -> temper_service::stock::Given {
        stories::given(document)
    }
}
pub(super) fn agent_profile() -> smith_agent_service::profile::Profile {
    let mut profile = temper_playground::documents::agent_profile();
    profile.declared.tool_payload = 4098;
    profile
}
pub(super) fn agent_settings() -> temper_service::settings::Agent {
    let mut agent = temper_playground::documents::agent_for(&agent_profile());
    agent.bounds.message_bytes = 4096;
    agent
}

pub(super) enum Extra {
    Native(HarnessProcess),
    Proxy(Box<Proxy>),
    Visitor(Box<Visitor>),
}
impl Extra {
    fn host(&self) -> &dyn Host {
        match self {
            Self::Native(h) => h,
            Self::Proxy(h) => h.as_ref(),
            Self::Visitor(h) => h.as_ref(),
        }
    }
    fn host_mut(&mut self) -> &mut dyn Host {
        match self {
            Self::Native(h) => h,
            Self::Proxy(h) => h.as_mut(),
            Self::Visitor(h) => h.as_mut(),
        }
    }
}
impl Host for Extra {
    fn drain(&mut self) {
        self.host_mut().drain();
    }
    fn iterate(&mut self, now: Time, wall: Wall) {
        self.host_mut().iterate(now, wall);
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        self.host_mut().completions()
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        self.host_mut().submissions()
    }
    fn work_pending(&self, now: Time) -> bool {
        self.host().work_pending(now)
    }
    fn next_mechanism_deadline(&self) -> Option<Time> {
        self.host().next_mechanism_deadline()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        self.host().next_policy_deadline()
    }
    fn is_empty(&self) -> bool {
        self.host().is_empty()
    }
    fn operations(&self) -> u32 {
        self.host().operations()
    }
    fn exit(&self) -> Option<Exit> {
        self.host().exit()
    }
}
pub(super) fn command(caller: &mut Caller, words: &[&str]) {
    caller.commands.push_back(Command::new(&words.iter().map(|word| word.as_bytes()).collect::<Vec<_>>()));
}

pub(super) fn run(seed: u64, name: &str) {
    let mut disk = temper_playground::machine::fresh();
    temper_playground::machine::setup(&mut disk);
    phase(seed, name, disk, Rc::new(RefCell::new(Store::new())));
}
pub(super) fn phase(
    seed: u64,
    name: &str,
    mut disk: skein_fake_machine::Machine,
    records: Rc<RefCell<Store>>,
) -> skein_fake_machine::Machine {
    super::ACTIVE.with(|active| active.set(true));
    BEARER.with(|held| *held.borrow_mut() = None);
    let advanced = matches!(
        name,
        "every_owner_version_and_kind_answers_as_declared"
            | "every_inaccessible_identifier_is_one_unknown_byte_for_byte"
    );
    let story = if advanced { Story::Merge } else { Story::Chat { words: b"Hello" } };
    stories::set(story);
    stories::setup_settings(if advanced { world::Phase::Story } else { world::Phase::Restart });
    let root = disk.reopen_root(2);
    let machine = Machine::new(disk);
    world::lower_store(Some(records.clone()));
    let transport = stories::forge(story);
    let parameters = world::parameters(&machine, transport.clone());
    let outcome = programs::with(parameters, || {
        let mut world = skein_world::World::new_controlled(
            seed,
            skein_sim::Config::calm(),
            skein_world::Memory::Unchecked,
            |controls| super::service_checks::Judge::new(seed, name, controls, records),
        )
        .with_machine(machine);
        for (program, roots) in programs::programs::<Deployment>() {
            world.host_roots(program, roots);
        }
        world.host_roots(
            crate::ctl::program(|spawn, inherited| {
                Proc::Other(Extra::Native(HarnessProcess::Ctl(Box::new(crate::ctl::host(spawn, inherited)))))
            }),
            crate::ctl::roots,
        );
        world.spawn_restartable(programs::service_roots(), programs::service::<Deployment>);
        world.spawn_root(skein_sim::Handle::new(root.raw()), |root| {
            Proc::Other(Extra::Native(HarnessProcess::Calls(Box::new(Caller::scripted(root)))))
        });
        world.spawn(|| {
            Proc::Other(Extra::Proxy(Box::new(Proxy::new(checks::PROXY, checks::config(fixtures::CLIENTS)))))
        });
        world.spawn(|| Proc::Other(Extra::Visitor(Box::new(Visitor::new()))));
        if advanced {
            world.spawn(|| {
                Proc::Peer(Box::new(temper_playground::machine::SignalHost::new(super::deployment::provider(), None)))
            });
            world.spawn(|| {
                Proc::Forge(Box::new(temper_playground::machine::SignalHost::new(
                    temper_playground::fakes::forge_peer(&transport, seed),
                    None,
                )))
            });
        }
        world.run()
    });
    assert!(outcome.procs.iter().all(Host::is_empty), "public service world drains, seed {seed}");
    world::lower_store(None);
    std::mem::replace(&mut *outcome.machine.0.borrow_mut(), temper_playground::machine::fresh())
}
