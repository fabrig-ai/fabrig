//! Browser harness stories on simulated processes (skein/docs/design/ui/web/contracts.md, 2).
use skein_fake_person_browser::{
    Harness, Lent, Outcome, Play,
    harness::{Hosted, Scripted},
};
use skein_lib::Time;
use skein_world::{Delegate, Host, HostedProgram, Inherited, Memory, Referee, World};
use skein_world_tests::hosted::{Act, RootMachine, Script};

mod keeps;
mod peers {
    mod tests;
}

pub(super) enum Process<'a, 'p, 'l> {
    Harness(Box<Hosted<'a, 'p, 'l>>),
    Child(Box<Script>),
}

impl Delegate for Process<'_, '_, '_> {
    fn host(&self) -> &dyn Host {
        match self {
            Self::Harness(host) => &**host,
            Self::Child(script) => &**script,
        }
    }

    fn host_mut(&mut self) -> &mut dyn Host {
        match self {
            Self::Harness(host) => &mut **host,
            Self::Child(script) => &mut **script,
        }
    }
}

#[derive(Default)]
struct Judge(bool);

impl Referee<Process<'_, '_, '_>> for Judge {
    fn watch(&mut self, _now: Time, procs: &[Process<'_, '_, '_>]) -> Result<(), String> {
        self.0 = procs.first().is_some_and(Host::is_empty);
        Ok(())
    }

    fn passed(&self) -> bool {
        self.0
    }
}

pub(super) struct Run {
    pub outcome: Outcome,
    pub end: Time,
    pub children: Vec<Script>,
}

pub(super) fn run<'a, 'p, 'l>(
    harness: &'a mut Harness,
    people: &'a mut [&'p mut dyn Play],
    peers: &'a mut [Lent<'l>],
    browser: Scripted,
    programs: impl IntoIterator<Item = HostedProgram<Process<'a, 'p, 'l>>>,
) -> Run {
    // The test still owns the scripts, peers and keeps that the host changes.
    let mut world =
        World::new(0, skein_sim::Config::calm(), Judge::default(), Memory::Unchecked).with_machine(RootMachine);
    for program in programs {
        world.host(program);
    }
    world.spawn_root(skein_sim::Handle::new(1), |root| {
        Process::Harness(Box::new(harness.hosted(people, peers, browser).at_root(root)))
    });
    let mut settled = world.run();
    let mut procs = std::mem::take(&mut settled.procs).into_iter();
    let Some(Process::Harness(hosted)) = procs.next() else { panic!("the harness is the root process") };
    Run {
        outcome: hosted.finish(),
        end: settled.end,
        children: procs
            .map(|process| match process {
                Process::Child(script) => *script,
                Process::Harness(_) => panic!("only the root is a harness"),
            })
            .collect(),
    }
}

pub(super) const LISTENING: &str = "listening ";

pub(super) fn server(inherited: &Inherited, status: u16) -> Script {
    let output = inherited.pipes.iter().position(|(at, _)| *at == 2).expect("inherited stderr");
    Script::child(
        inherited,
        &[Act::ListenAndPrint { output, prefix: LISTENING, suffix: "" }, Act::AnswerRequests(status)],
    )
}
