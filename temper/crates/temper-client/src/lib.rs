//! The deployment's client (temper/docs/design/protocol/binaries.md).
#![forbid(unsafe_code)]

use skein_shell::Clock;
use std::{ffi::OsString, os::unix::ffi::OsStrExt};
use temper_shell::{
    commands::{self, Row},
    help::{self, Request},
    rows,
};

pub const NAME: &str = "temper-client";

pub const COMMANDS: [Row; 3] = [rows::HELP, rows::TUI, rows::CTL];

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Overview,
    Version,
    Usage(&'static Row),
    Help { request: Request },
    Tui { address: Option<Box<[u8]>> },
    Ctl { arguments: Vec<OsString> },
}

pub fn configure(arguments: &[Box<[u8]>]) -> Result<Command, String> {
    match arguments {
        [flag] if flag.as_ref() == b"--help" => return Ok(Command::Overview),
        [flag] if flag.as_ref() == b"--version" => return Ok(Command::Version),
        _ => {}
    }
    let row = commands::find(&COMMANDS, arguments).ok_or_else(|| commands::unknown(NAME, &COMMANDS, arguments))?;
    let arguments = &arguments[row.words.len()..];
    if let [flag] = arguments
        && flag.as_ref() == b"--help"
    {
        return Ok(Command::Usage(row));
    }
    let invalid = || commands::refused(NAME, row, "invalid arguments");
    match row.words {
        [] => match arguments {
            [] => Ok(Command::Tui { address: None }),
            [address] => Ok(Command::Tui { address: Some(address.clone()) }),
            _ => Err(invalid()),
        },
        ["ctl"] => Ok(Command::Ctl {
            arguments: arguments.iter().map(|argument| std::ffi::OsStr::from_bytes(argument).to_owned()).collect(),
        }),
        ["help"] => help::request(arguments).map(|request| Command::Help { request }).ok_or_else(invalid),
        _ => Err(invalid()),
    }
}

pub fn run(command: Command) -> Result<u8, String> {
    match command {
        Command::Overview => print!("{}", commands::help(NAME, &COMMANDS)),
        Command::Version => print!("{}", version()),
        Command::Usage(row) => print!("{}", commands::row(NAME, row)),
        Command::Help { request } => print!("{}", help::answer(NAME, &request)?),
        Command::Tui { address } => {
            return Ok(temper_client_shell_tui::start::run(NAME, address.as_deref(), &Clock::new()));
        }
        Command::Ctl { arguments } => return Ok(temper_client_shell_cli::start::run(&arguments, &Clock::new())),
    }
    Ok(0)
}

/// The binary's version and build commit (temper/docs/design/release.md, section 3).
#[must_use]
pub fn version() -> String {
    format!("{}\n", temper_version::line(NAME))
}

#[cfg(test)]
mod tests;
