//! The deployment's agent (temper/docs/design/protocol/README.md, 4.3).
#![forbid(unsafe_code)]

use std::{
    ffi::{OsStr, OsString},
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
};
use temper_shell::{
    commands::{self, Argument, Row, Who},
    help::{self, Request},
    rows,
};

pub const NAME: &str = "temper-agent";

pub const RUN: Row = Row {
    words: &[],
    who: Who::Spawned,
    forms: &["AGENT.json"],
    line: "Run one agent for a worker's run, on smith's agent configuration.",
    arguments: &[Argument {
        shown: "AGENT.json",
        line: "The agent's configuration, as the worker's agent.configuration names it.",
    }],
    settings: &[],
    notes: "",
};
pub const LIST: Row = Row {
    words: &["--list"],
    who: Who::Spawned,
    forms: &["AGENT.json"],
    line: "Read one endpoint's model list, on smith's agent configuration.",
    arguments: RUN.arguments,
    settings: &[],
    notes: "",
};
pub const RELAY: Row = Row {
    words: &["--mcp-relay"],
    who: Who::Spawned,
    forms: &["127.0.0.1:PORT"],
    line: "Relay a program's MCP connection to its agent.",
    arguments: &[Argument {
        shown: "PORT",
        line: "The agent's relay listener on loopback; the secret is in SMITH_MCP_SECRET.",
    }],
    settings: &[],
    notes: "",
};
pub const PROGRAM_CHILD: Row = Row {
    words: &["--program-child"],
    who: Who::Spawned,
    forms: &["PARENT PROGRAM ARGUMENTS..."],
    line: "Run a program as the agent's child, which ends with its parent.",
    arguments: &[
        Argument { shown: "PARENT", line: "The agent's process id." },
        Argument { shown: "PROGRAM", line: "The program's path, with its ARGUMENTS." },
    ],
    settings: &[],
    notes: "",
};
pub const COMMANDS: [Row; 5] = [rows::HELP, RUN, LIST, RELAY, PROGRAM_CHILD];

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Overview,
    Version,
    Usage(&'static Row),
    Help { request: Request },
    Run { configuration: PathBuf },
    List { configuration: PathBuf },
}

pub fn configure(arguments: &[Box<[u8]>]) -> Result<Command, String> {
    match arguments {
        [] => return Err(commands::unknown(NAME, &COMMANDS, arguments)),
        [flag] if flag.as_ref() == b"--help" => return Ok(Command::Overview),
        [flag] if flag.as_ref() == b"--version" => return Ok(Command::Version),
        _ => {}
    }
    if let Some(row) = commands::find(&COMMANDS, arguments) {
        let arguments = &arguments[row.words.len()..];
        if let [flag] = arguments
            && flag.as_ref() == b"--help"
        {
            return Ok(Command::Usage(row));
        }
        let invalid = || commands::refused(NAME, row, "invalid arguments");
        return match row.words {
            ["help"] => help::request(arguments).map(|request| Command::Help { request }).ok_or_else(invalid),
            ["--list"] => match arguments {
                [configuration] => Ok(Command::List { configuration: path(configuration).into() }),
                _ => Err(invalid()),
            },
            [] => run_arguments(arguments),
            _ => Err(invalid()),
        };
    }
    if arguments.first().is_some_and(|argument| !argument.starts_with(b"-")) {
        return run_arguments(arguments);
    }
    Err(commands::unknown(NAME, &COMMANDS, arguments))
}

fn run_arguments(arguments: &[Box<[u8]>]) -> Result<Command, String> {
    match arguments {
        [configuration] => Ok(Command::Run { configuration: path(configuration).into() }),
        _ => Err(commands::refused(NAME, &RUN, "invalid arguments")),
    }
}

pub fn run(command: Command) -> Result<u8, String> {
    let exit = match command {
        Command::Overview => {
            print!("{}", commands::help(NAME, &COMMANDS));
            return Ok(0);
        }
        Command::Version => {
            print!("{}", version());
            return Ok(0);
        }
        Command::Usage(row) => {
            print!("{}", commands::row(NAME, row));
            return Ok(0);
        }
        Command::Help { request } => {
            print!("{}", help::answer(NAME, &request)?);
            return Ok(0);
        }
        Command::Run { configuration } => decoded(&configuration)
            .inspect_err(|why| {
                use std::io::Write;
                drop(writeln!(std::io::stderr(), "smith: {why}"));
            })
            .and_then(|decoded| match dump_directory(std::env::var_os("TEMPER_DUMP_LLM")) {
                Some(directory) => smith_agent_shell::run_dumping(
                    decoded,
                    smith_agent_shell::dump::Dumps::new(directory, DUMP_BOUND),
                    Box::new(std::io::stderr()),
                ),
                None => smith_agent_shell::run_decoded(decoded, Box::new(std::io::stderr())),
            })
            .map_err(|why| prefixed(NAME, &why))?,
        Command::List { configuration } => decoded(&configuration)
            .and_then(|decoded| smith_agent_shell::list::run_decoded(decoded, Box::new(std::io::stderr())))
            .map_err(|why| prefixed("temper-agent --list", &why))?,
    };
    Ok(match exit {
        skein_io::kernel::Exit::Code(code) => code,
        skein_io::kernel::Exit::Signal(_) => 1,
    })
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

/// The binary's version and build commit (temper/docs/design/release.md, section 3).
#[must_use]
pub fn version() -> String {
    format!("{NAME} {} ({})\n", env!("CARGO_PKG_VERSION"), env!("TEMPER_AGENT_COMMIT"))
}

fn dump_directory(value: Option<OsString>) -> Option<PathBuf> {
    let value = value?;
    if value.is_empty() {
        return None;
    }
    match std::path::absolute(&value) {
        Ok(directory) => Some(directory),
        Err(why) => {
            use std::io::Write;
            drop(writeln!(std::io::stderr(), "temper-agent: TEMPER_DUMP_LLM: {why}"));
            None
        }
    }
}
const DUMP_BOUND: u64 = 4_294_967_296;

#[cfg(test)]
mod tests;
