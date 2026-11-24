//! Hosted agent startup and cancellation (temper/docs/design/testing/real-loop.md).
use crate::{
    fixtures,
    world::{Kind, Proc},
};
use skein_io::kernel::{Complete, Fd, Spawn, Submit};
use skein_lib::{Queue, Rng, Secret, Time, Wall};
use skein_shell::Host;
use skein_world::Inherited;
use smith_agent_shell::{Agent, ProgramResources, Resources, config};
use std::{ffi::OsStr, io::Write, os::unix::ffi::OsStrExt, path::Path};
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
    let path = Path::new(OsStr::from_bytes(spawn.args.last().expect("agent configuration argument")));
    let configuration = temper_worker_shell::agent::configuration(path).expect("shipped agent configuration");
    let relay = Path::new(OsStr::from_bytes(&spawn.program));
    let directory = match &configuration.program {
        Some(program) => open_programs(&program.directory).expect("hosted programs directory"),
        None => None,
    };
    let (errors, capture) = smith_agent_process_world::AgentErrors::pipe(pipe(inherited, 2));
    let agent = start(configuration, inherited, relay, directory, Box::new(errors)).expect("hosted shipped agent");
    Proc::new(Kind::Agent(Box::new(Hosted { freeze_on_cancel, frozen: false, agent: capture.host(agent) })))
}

fn pipe(inherited: &Inherited, number: u32) -> Fd {
    inherited.pipes.iter().find(|(child, _)| *child == number).expect("agent pipe").1
}

fn open_programs(path: &Path) -> Result<Option<Fd>, String> {
    if smith_agent_shell::program::private_directory(path).map_err(|why| why.to_string())? {
        skein_shell::open_root(path).map(Some).map_err(|why| format!("programs root: {why:?}"))
    } else {
        Ok(None)
    }
}

fn start(
    configuration: config::Configuration,
    inherited: &Inherited,
    relay: &Path,
    directory: Option<Fd>,
    errors: Box<dyn Write>,
) -> Result<Agent, String> {
    // A hosted agent is not a process of its own: keep the decoded parent unset.
    let programs = configuration.program.as_ref().map(|_| ProgramResources {
        relay: relay.into(),
        entropy: program_entropy(),
        directory,
    });
    Agent::new(
        configuration,
        Resources {
            input: pipe(inherited, 0),
            output: pipe(inherited, 1),
            error: Some(pipe(inherited, 2)),
            signals: inherited.signal,
            seed: fixtures::SEED,
            pid: 101,
            trace: None,
            // Start carries the checkout paths after spawn. Open those paths in
            // Start order, just as the shipped process does on this filesystem.
            roots: None,
            programs,
        },
        errors,
    )
}

fn program_entropy() -> Secret {
    let mut entropy = Box::new([0; 32]);
    let mut rng = Rng::new(fixtures::SEED);
    for word in entropy.as_chunks_mut::<8>().0 {
        *word = rng.next_u64().to_le_bytes();
    }
    Secret::new(entropy)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hosted_start_accepts_the_workers_agent_document() {
        let data = Path::new("/fixture");
        let address = ([127, 0, 0, 1], 9).into();
        let (agent, mut deployment) = fixtures::documents(data, address, address);
        let providers = jig_protocol_config::llm::provider_values(&crate::settings::configuration(&deployment).llm);
        let setting = temper_agent_fixtures::compose_setting(&mut deployment, &agent, &providers);
        let shape = temper_config_shell::Shape::Standalone;
        let file = temper_config_shell::render(&fixtures::file::standalone(&deployment), shape);
        let facts =
            temper_config_shell::Facts { login: "fixture".into(), hostname: "localhost".into(), provider_key: false };
        let start_document = temper_config_shell::parse(&file, shape, &facts).expect("standalone fixture start");
        let deployment = serde_json::from_slice(&start_document.document).expect("standalone fixture JSON");
        let document = temper_agent_fixtures::with_figures(&deployment, &setting.figures, data, &setting.providers);
        let bytes = serde_json::to_vec(&document).expect("worker's agent JSON");
        let configuration = temper_worker_shell::agent::decoded_bytes(&bytes)
            .expect("decode the worker's agent document")
            .resolve_with(
                |address| address.parse().map_err(|why| format!("fixture endpoint {address}: {why}").into()),
                |_, _| panic!("the fixture uses plaintext"),
                |_| None,
            )
            .expect("resolve the worker's agent document");
        let inherited = Inherited {
            pipes: vec![(0, Fd::new(100)), (1, Fd::new(101)), (2, Fd::new(102))],
            signal: Fd::new(103),
            roots: Vec::new(),
            appends: Vec::new(),
        };
        let relay = fixtures::pinned_agent_program(data);
        start(configuration, &inherited, &relay, None, Box::new(std::io::sink()))
            .expect("hosted start accepts the worker's agent");
    }
}
