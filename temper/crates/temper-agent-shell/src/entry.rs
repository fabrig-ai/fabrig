//! The agent's capture and dispatch (temper/docs/design/cli/README.md, 3 and 7.3).
use crate::{AGENT, Command};
use std::{os::unix::ffi::OsStrExt, path::PathBuf, process::ExitCode};
use temper_shell::directories::EnvironmentEntry;

struct Capture {
    run_as: Box<[u8]>,
    arguments: Box<[Box<[u8]>]>,
    environment: Box<[EnvironmentEntry]>,
    working: Result<PathBuf, String>,
}

#[expect(
    clippy::disallowed_methods,
    reason = "the binary's capture: argv's first word, arguments, environment and working directory are read once"
)]
fn capture() -> Capture {
    Capture {
        run_as: std::env::args_os().next().map(|word| word.as_bytes().into()).unwrap_or_default(),
        arguments: skein_shell::arguments(),
        environment: skein_shell::environment(),
        working: std::env::current_dir().map_err(|why| why.to_string()),
    }
}

pub fn main(version: &'static str, commit: &'static str) -> ExitCode {
    let captured = capture();
    match run(&captured, version, commit) {
        Ok(code) => code,
        Err((code, why)) => {
            eprint!("{why}");
            ExitCode::from(code)
        }
    }
}

fn run(captured: &Capture, version: &'static str, commit: &'static str) -> Result<ExitCode, (u8, String)> {
    let command = crate::configure(AGENT.reach(&captured.run_as, &captured.arguments), version, commit)?;
    match command {
        Command::Print(text) => {
            print!("{text}");
            Ok(ExitCode::SUCCESS)
        }
        Command::Relay => Ok(smith_agent_shell::relay::main()),
        Command::ProgramChild => Ok(smith_agent_shell::program_child::main()),
        Command::Claude { arguments } => Ok(ExitCode::from(temper_launcher::run(
            AGENT.binary,
            std::ffi::OsStr::from_bytes(&captured.run_as),
            &arguments,
            &captured.environment,
        ))),
        Command::Run { configuration } => {
            crate::run(AGENT.binary, &configuration, &captured.environment, &captured.working)
                .map(ExitCode::from)
                .map_err(|why| (1, why))
        }
        Command::List { configuration } => {
            crate::list("temper-agent --list", &configuration).map(ExitCode::from).map_err(|why| (1, why))
        }
    }
}
