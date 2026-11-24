//! Simulated startup for the shipped program account and its hosted children
//! (smith/docs/design/testing.md, sections 2.1 and 4).
use super::Proc;
use skein_io::kernel::Spawn;
use skein_lib::{List, Map};
use skein_world::{Inherited, StartupAppend, StartupRoot};
use smith_local_shell::credentials::{Location, Prepared, Source};

pub(crate) const PROGRAMS: &[u8] = b"/programs";
pub(crate) const ACCOUNT: &str = "work";
pub(crate) const CLAUDE: &[u8] = b"/programs/claude";
pub(crate) const RELAY: &[u8] = b"/programs/smith-agent";

/// Opt-in program-account inputs for the hosted local shell and agent.
#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct ProgramLaunch {
    /// Person-supplied command environment, before the program adapters filter it.
    #[serde(default)]
    pub environment: Vec<(String, String)>,
    /// Use the non-plain screen when the launch selects the interactive front.
    #[serde(default)]
    pub live_view: bool,
}

pub(super) fn program_root() -> StartupRoot {
    StartupRoot { name: b"programs".as_slice().into(), path: PROGRAMS.into() }
}

pub(super) fn prepared_accounts(io: skein_io::Limits) -> Prepared {
    let name = ACCOUNT.to_owned();
    let mut locations = List::with_capacity(1);
    locations.push(Location { account: 0, name: name.clone(), source: Source::Program { name } }).unwrap();
    let mut lendable = Map::with_capacity(1);
    lendable.insert(0, (0, smith_protocol_translate::GrantValue::Program(ACCOUNT.as_bytes().into()))).unwrap();
    Prepared {
        sources: List::with_capacity(0),
        locations,
        lendable,
        limits: smith_local_shell::credentials::limits(
            io,
            1,
            smith_agent_service::profile::standard().policy.refresh_lead,
        ),
    }
}

pub(crate) fn fake_roots(spawn: &Spawn) -> Vec<StartupRoot> {
    let invocation = smith_fake_claude::simulated_launch(spawn, PROGRAMS, b"/repo");
    smith_fake_claude::config_root(&invocation).into_iter().chain(smith_fake_claude::mcp_root(&invocation)).collect()
}

pub(crate) fn fake_appends(spawn: &Spawn) -> Vec<StartupAppend> {
    smith_fake_claude::observation_append(spawn).into_iter().collect()
}

pub(crate) fn make_fake(spawn: &Spawn, inherited: &Inherited) -> Proc {
    let invocation = smith_fake_claude::simulated_launch(spawn, PROGRAMS, b"/repo");
    Proc::Claude(Box::new((smith_fake_claude::hosted(CLAUDE).make)(&invocation, inherited)))
}

pub(crate) fn make_relay(spawn: &Spawn, inherited: &Inherited) -> Proc {
    let pipe = |number| inherited.pipes.iter().find(|(child, _)| *child == number).map(|(_, fd)| *fd);
    Proc::Relay(Box::new(smith_agent_shell::relay::Process::new(
        &spawn.args,
        &spawn.env,
        smith_agent_shell::relay::Faces {
            stdin: pipe(0).expect("relay input"),
            stdout: pipe(1).expect("relay output"),
            stderr: pipe(2),
            signal: inherited.signal,
        },
    )))
}
