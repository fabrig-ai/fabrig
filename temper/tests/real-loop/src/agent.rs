//! Hosted agents on real pipes and independently opened roots.
//! A story may freeze the agent at cancellation (testing/real-loop.md).
use crate::{
    fixtures,
    world::{Kind, Proc},
};
use skein_io::kernel::{Complete, Spawn, Submit};
use skein_lib::{Queue, Time, Wall};
use skein_shell::Host;
use skein_world::{Inherited, StartupRoot};
use std::{os::unix::ffi::OsStrExt, path::Path};
pub(crate) type Factory = fn(&Spawn, &Inherited) -> Proc;

/// Keeps the shipped agent's owners alive, but may stop driving it at cancel.
pub(crate) struct Hosted {
    agent: skein_world::PipeHost<smith_agent_shell::Agent>,
    freeze_on_cancel: bool,
    frozen: bool,
}
impl Hosted {
    pub(crate) fn cancel(&mut self) {
        self.frozen = self.freeze_on_cancel;
    }
}
impl Host for Hosted {
    fn iterate(&mut self, now: Time, wall: Wall) {
        if self.frozen {
            self.agent.completions().clear();
        } else {
            self.agent.iterate(now, wall);
        }
    }
    fn drain(&mut self) {
        if !self.frozen {
            self.agent.drain();
        }
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        self.agent.completions()
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        self.agent.submissions()
    }
    fn work_pending(&self, now: Time) -> bool {
        !self.frozen && self.agent.work_pending(now)
    }
    fn next_mechanism_deadline(&self) -> Option<Time> {
        if self.frozen { None } else { self.agent.next_mechanism_deadline() }
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        if self.frozen { None } else { self.agent.next_policy_deadline() }
    }
    fn is_empty(&self) -> bool {
        !self.frozen && self.agent.is_empty()
    }
    fn exit(&self) -> Option<skein_io::kernel::Exit> {
        if self.frozen { None } else { self.agent.exit() }
    }
    fn operations(&self) -> u32 {
        self.agent.operations()
    }
}
pub(crate) fn stubborn(spawn: &Spawn, inherited: &Inherited) -> Proc {
    hosted(spawn, inherited, true)
}
pub(crate) fn agent(spawn: &Spawn, inherited: &Inherited) -> Proc {
    hosted(spawn, inherited, false)
}
fn hosted(spawn: &Spawn, inherited: &Inherited, freeze_on_cancel: bool) -> Proc {
    let configuration =
        Path::new(std::ffi::OsStr::from_bytes(spawn.args.last().expect("agent configuration argument")));
    let configuration = temper_worker_shell::agent::configuration(configuration).expect("shipped agent configuration");
    let programs = configuration.program.is_some().then(|| smith_agent_shell::ProgramResources {
        relay: Path::new(std::ffi::OsStr::from_bytes(crate::claude::RELAY)).into(),
        entropy: skein_lib::Secret::new(Box::new([7; 32])),
        directory: inherited.roots.iter().find(|(name, _)| name.as_ref() == b"programs").map(|(_, fd)| *fd),
    });
    let stderr = inherited.pipes.iter().find(|(child, _)| *child == 2).expect("stderr").1;
    let (errors, capture) = smith_agent_process_world::AgentErrors::pipe(stderr);
    Proc::new(Kind::Agent(Box::new(Hosted {
        freeze_on_cancel,
        frozen: false,
        agent: capture.host(
            smith_agent_shell::Agent::new(
                configuration,
                smith_agent_shell::Resources {
                    input: inherited.pipes.iter().find(|(child, _)| *child == 0).expect("stdin").1,
                    output: inherited.pipes.iter().find(|(child, _)| *child == 1).expect("stdout").1,
                    error: Some(stderr),
                    signals: inherited.signal,
                    seed: fixtures::SEED,
                    pid: 101,
                    trace: None,
                    // Start carries the checkout paths after spawn. Open those paths in
                    // Start order, just as the shipped process does on this filesystem.
                    roots: None,
                    programs,
                },
                Box::new(errors),
            )
            .expect("hosted shipped agent"),
        ),
    })))
}
pub(crate) fn roots(spawn: &Spawn) -> Vec<StartupRoot> {
    let configuration =
        Path::new(std::ffi::OsStr::from_bytes(spawn.args.last().expect("agent configuration argument")));
    let configuration = temper_worker_shell::agent::configuration(configuration).expect("agent programs root");
    configuration
        .program
        .iter()
        .filter(|program| {
            smith_agent_shell::program::private_directory(&program.directory).expect("private programs root")
        })
        .map(|program| StartupRoot {
            name: b"programs".as_slice().into(),
            path: program.directory.as_os_str().as_bytes().into(),
        })
        .collect()
}
