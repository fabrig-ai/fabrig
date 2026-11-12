//! The deployment's agent (temper/docs/design/protocol/binaries.md, 3).
#![forbid(unsafe_code)]

use skein_shell::{Argument, Command as Row, Reach, Reached, Set, Table};
use std::{
    ffi::OsStr,
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
};
use temper_shell::{directories::EnvironmentEntry, table};

mod entry;
pub use entry::main;

/// The agent's table (temper/docs/design/cli/README.md, 7.3).
pub const AGENT: Table = Table {
    binary: "temper-agent",
    line: "the deployment's agent, which its worker runs for each run.",
    // The identity is main's (temper/docs/design/cli/README.md, 3).
    version: "",
    commit: "",
    commands: &[temper_shell::table::HELP, RUN, LIST, RELAY, PROGRAM_CHILD, CLAUDE],
    settings: &[],
    variables: &[&temper_shell::environment::INTERNAL_DUMP_LLM, &temper_shell::environment::INTERNAL_COMMIT],
    more: "temper-agent COMMAND --help prints one command; temper-agent help pages lists the pages.",
};

const INTERNAL: Row = Row {
    reach: Reach::Words(&[]),
    set: Set::Internal,
    forms: &[],
    line: "",
    arguments: &[],
    settings: &[],
    switches: &[],
    variables: &[],
    notes: "",
};
const RUN: Row = Row {
    reach: Reach::Words(&[]),
    forms: &["AGENT.json"],
    line: "Run one agent for a worker's run, on smith's agent configuration.",
    arguments: &[Argument {
        shown: "AGENT.json",
        line: "The agent's configuration, as the worker's agent.configuration names it.",
    }],
    ..INTERNAL
};
const LIST: Row = Row {
    reach: Reach::Words(&["--list"]),
    forms: &["AGENT.json"],
    line: "Read one endpoint's model list, on smith's agent configuration.",
    arguments: RUN.arguments,
    ..INTERNAL
};
/// A delegated mode shared with the box (temper/docs/design/cli/README.md, 7.3).
pub const RELAY: Row = Row {
    reach: Reach::Words(&["--mcp-relay"]),
    forms: &["127.0.0.1:PORT"],
    line: "Relay a program's MCP connection to its agent.",
    arguments: &[Argument {
        shown: "PORT",
        line: "The agent's relay listener on loopback; the secret is in SMITH_MCP_SECRET.",
    }],
    ..INTERNAL
};
/// A delegated mode shared with the box (temper/docs/design/cli/README.md, 7.3).
pub const PROGRAM_CHILD: Row = Row {
    reach: Reach::Words(&["--program-child"]),
    forms: &["PARENT PROGRAM ARGUMENTS..."],
    line: "Run a program as the agent's child, which ends with its parent.",
    arguments: &[
        Argument { shown: "PARENT", line: "The agent's process id." },
        Argument { shown: "PROGRAM", line: "The program's path, with its ARGUMENTS." },
    ],
    ..INTERNAL
};
/// A delegated mode shared with the box (temper/docs/design/cli/README.md, 7.3).
pub const CLAUDE: Row = Row {
    reach: Reach::RunAs("claude"),
    forms: &["ARGS..."],
    line: "Run the Claude Code launcher with its delegated arguments.",
    variables: &[&temper_shell::environment::SSL_CERT_FILE],
    ..INTERNAL
};

#[derive(Debug, PartialEq, Eq)]
enum Command {
    Print(String),
    Run { configuration: PathBuf },
    List { configuration: PathBuf },
    Relay,
    ProgramChild,
    Claude { arguments: Box<[Box<[u8]>]> },
}

fn configure(reached: Reached<'_>, version: &'static str, commit: &'static str) -> Result<Command, (u8, String)> {
    let (command, arguments) = match reached {
        Reached::Help => return Ok(Command::Print(AGENT.help())),
        Reached::Version => return Ok(Command::Print(Table { version, commit, ..AGENT }.version_line())),
        Reached::CommandHelp(command) => return Ok(Command::Print(AGENT.command_help(command))),
        Reached::Unknown(why) => return Err((2, why)),
        Reached::Command(command, arguments) => (command, arguments),
    };
    let invalid = || (2, table::refused(&AGENT, command, "invalid arguments"));
    match command.reach {
        Reach::Words(["help"]) => table::help(&AGENT, arguments).map(Command::Print),
        Reach::Words([]) => {
            if arguments.first().is_none_or(|argument| argument.starts_with(b"-")) {
                return Err((2, AGENT.unknown(arguments)));
            }
            match arguments {
                [configuration] => Ok(Command::Run { configuration: path(configuration).into() }),
                _ => Err(invalid()),
            }
        }
        Reach::Words(["--list"]) => match arguments {
            [configuration] => Ok(Command::List { configuration: path(configuration).into() }),
            _ => Err(invalid()),
        },
        Reach::Words(["--mcp-relay"]) => Ok(Command::Relay),
        Reach::Words(["--program-child"]) => Ok(Command::ProgramChild),
        Reach::RunAs("claude") => Ok(Command::Claude { arguments: arguments.into() }),
        Reach::Words(_) | Reach::RunAs(_) => Err(invalid()),
    }
}

/// Run one agent, prefixing failures with `words` (protocol/binaries.md, 3).
pub fn run(
    words: &str,
    configuration: &Path,
    environment: &[EnvironmentEntry],
    working: &Result<PathBuf, String>,
) -> Result<u8, String> {
    decoded(configuration)
        .inspect_err(|why| {
            use std::io::Write;
            drop(writeln!(std::io::stderr(), "smith: {why}"));
        })
        .and_then(|decoded| {
            let directory = dump_directory(environment, working).unwrap_or_else(|why| {
                use std::io::Write;
                drop(writeln!(std::io::stderr(), "{words}: TEMPER_INTERNAL_DUMP_LLM: {why}"));
                None
            });
            match directory {
                Some(directory) => smith_agent_shell::run_dumping(
                    decoded,
                    smith_agent_shell::dump::Dumps::new(directory, DUMP_BOUND),
                    Box::new(std::io::stderr()),
                ),
                None => smith_agent_shell::run_decoded(decoded, Box::new(std::io::stderr())),
            }
        })
        .map(exit_code)
        .map_err(|why| prefixed(words, &why))
}

/// Read one endpoint's model list, prefixing failures with `words`.
pub fn list(words: &str, configuration: &Path) -> Result<u8, String> {
    decoded(configuration)
        .and_then(|decoded| smith_agent_shell::list::run_decoded(decoded, Box::new(std::io::stderr())))
        .map(exit_code)
        .map_err(|why| prefixed(words, &why))
}

fn exit_code(exit: skein_io::kernel::Exit) -> u8 {
    match exit {
        skein_io::kernel::Exit::Code(code) => code,
        skein_io::kernel::Exit::Signal(_) => 1,
    }
}

fn decoded(configuration: &Path) -> Result<smith_agent_shell::config::Decoded, String> {
    let bytes = smith_agent_shell::config::read_bytes(configuration)?;
    let bytes = temper_launcher::substituted(&bytes)?.unwrap_or(bytes);
    smith_agent_shell::config::decode(&bytes)
}

fn prefixed(name: &str, why: &str) -> String {
    use std::fmt::Write;
    let mut lines = String::new();
    for line in why.lines() {
        writeln!(lines, "{name}: {line}").expect("writing into a string");
    }
    lines
}

fn path(bytes: &[u8]) -> &Path {
    Path::new(OsStr::from_bytes(bytes))
}

fn dump_directory(
    environment: &[EnvironmentEntry],
    working: &Result<PathBuf, String>,
) -> Result<Option<PathBuf>, String> {
    let Some(value) = temper_shell::environment::INTERNAL_DUMP_LLM.value(environment) else {
        return Ok(None);
    };
    let value = path(value);
    let absolute = if value.is_absolute() { value.into() } else { working.as_ref()?.join(value) };
    std::path::absolute(absolute).map(Some).map_err(|why| why.to_string())
}
const DUMP_BOUND: u64 = 4_294_967_296;

#[cfg(test)]
mod tests;
