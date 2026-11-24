//! The client's table, capture and dispatch (temper/docs/design/cli/README.md, 3 and 7.4).
#![forbid(unsafe_code)]

use skein_shell::{Argument, Clock, Command as Row, Reach, Reached, Set, Table};
use std::{ffi::OsStr, os::unix::ffi::OsStrExt, path::PathBuf, process::ExitCode};
use temper_shell::{directories, directories::EnvironmentEntry, environment, table};

pub mod box_binary;

/// The loopback HTTPS origin for a test's fake binary site (temper/docs/design/client/box-binary.md, 3).
pub const SITE: skein_shell::Variable = skein_shell::Variable {
    name: "TEMPER_INTERNAL_SITE",
    set: Set::Internal,
    form: "ORIGIN",
    way: skein_shell::Way::Reads,
    line: "Loopback HTTPS origin for a test's temper-box download.",
    absent: "Use https://fabrig.ai (port 443).",
    notes: "Only https://127.0.0.1:PORT, https://[::1]:PORT or https://localhost:PORT, with nothing after the port.",
};

pub const NAME: &str = "temper";

pub const CLIENT: Table = Table {
    binary: "temper",
    line: "temper's TUI, on an engine or in front of a box.",
    version: "",
    commit: "",
    commands: &[TUI, box_binary::BOX, table::HELP, table::CTL, box_binary::FETCH],
    settings: &[],
    variables: &[
        &skein_shell::HOME,
        skein_shell::Base::Config.variable(),
        skein_shell::Base::Data.variable(),
        skein_shell::Base::Cache.variable(),
        skein_shell::Base::State.variable(),
        &environment::XDG_RUNTIME_DIR,
        &environment::SSH_CONNECTION,
        &environment::SSH_TTY,
        &environment::DISPLAY,
        &environment::WAYLAND_DISPLAY,
        &environment::PATH,
        &environment::COLORFGBG,
        &environment::COLORTERM,
        &environment::TERM_PROGRAM,
        &environment::SSL_CERT_FILE,
        &SITE,
        &environment::INTERNAL_COMMIT,
    ],
    more: "temper COMMAND --help prints one command; temper help pages lists the pages.",
};

const TUI: Row = Row {
    reach: Reach::Words(&[]),
    set: Set::Public,
    forms: &["[ADDRESS]"],
    line: "Run temper's full-screen terminal client, the TUI, on an engine; temper help tui explains closing and signing out.",
    arguments: &[Argument {
        shown: "ADDRESS",
        line: "Engine origin, http://HOST:PORT on loopback or https://NAME[:PORT]; defaults to the engine link file's address for this environment, else the TUI's last sign-in address, else http://127.0.0.1:8367.",
    }],
    settings: &[&directories::SETTINGS],
    switches: &[],
    variables: &[&environment::INTERNAL_RECORD],
    notes: "Needs a terminal on stdin and stdout; without one, bare temper says it has no command. Its sign-in link is pasted on its screen, never given. Its directory is $XDG_STATE_HOME/temper/tui, else ~/.local/state/temper/tui. `/exit` or Ctrl+C closes it without signing out; on the sign-in screen, use Ctrl+C. `q` types a character on every screen.",
};

struct Capture {
    run_as: Box<[u8]>,
    arguments: Box<[Box<[u8]>]>,
    environment: Box<[EnvironmentEntry]>,
    working: Result<PathBuf, String>,
    executable: Result<PathBuf, String>,
    version: String,
    commit: String,
}

#[expect(
    clippy::disallowed_methods,
    reason = "the binary's capture: argv's first word, arguments, environment, working directory and executable path are read once"
)]
fn capture(line: &str) -> Capture {
    let (version, commit) = line
        .strip_prefix("temper ")
        .and_then(|line| line.strip_suffix(')'))
        .and_then(|line| line.split_once(" ("))
        .expect("the binary injects its version line");
    Capture {
        run_as: std::env::args_os().next().map(|word| word.as_bytes().into()).unwrap_or_default(),
        arguments: skein_shell::arguments(),
        environment: skein_shell::environment(),
        working: std::env::current_dir().map_err(|why| why.to_string()),
        executable: std::env::current_exe().map_err(|why| why.to_string()),
        version: version.into(),
        commit: commit.into(),
    }
}

pub fn main(line: &str) -> ExitCode {
    let captured = capture(line);
    let reached = CLIENT.reach(&captured.run_as, &captured.arguments);
    match configure(reached, line).and_then(|command| run(command, &captured)) {
        Ok(code) => ExitCode::from(code),
        Err((code, why)) => {
            eprint!("{why}");
            ExitCode::from(code)
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Command<'a> {
    Print(String),
    Tui(Option<&'a [u8]>),
    Box(&'a [Box<[u8]>]),
    FetchBox(&'a [Box<[u8]>]),
    Ctl(&'a [Box<[u8]>]),
}

fn configure<'a>(reached: Reached<'a>, line: &str) -> Result<Command<'a>, (u8, String)> {
    let (command, arguments) = match reached {
        Reached::Help => return Ok(Command::Print(CLIENT.help())),
        Reached::Version => return Ok(Command::Print(format!("{line}\n"))),
        Reached::CommandHelp(command) => return Ok(Command::Print(CLIENT.command_help(command))),
        Reached::Unknown(why) => return Err((2, why)),
        Reached::Command(command, arguments) => (command, arguments),
    };
    match command.reach {
        Reach::Words([]) => tui_argument(arguments).map(Command::Tui),
        Reach::Words(["box"]) => Ok(Command::Box(arguments)),
        Reach::Words(["internal", "fetch-box"]) => Ok(Command::FetchBox(arguments)),
        Reach::Words(["ctl"]) => Ok(Command::Ctl(arguments)),
        Reach::Words(["help"]) => table::help(&CLIENT, arguments).map(Command::Print),
        Reach::Words(_) | Reach::RunAs(_) => Err((2, table::refused(&CLIENT, command, "invalid arguments"))),
    }
}

fn run(command: Command<'_>, captured: &Capture) -> Result<u8, (u8, String)> {
    match command {
        Command::Print(text) => {
            print!("{text}");
            Ok(0)
        }
        Command::Tui(address) => {
            let working = captured.working.as_ref().map_err(|why| (1, format!("temper: {why}\n")))?;
            temper_client_shell_tui::start::run(
                CLIENT.binary,
                address,
                &captured.environment,
                working.as_os_str().as_bytes(),
                format!("{} ({})", captured.version, captured.commit).as_bytes(),
                &Clock::new(),
            )
        }
        Command::Box(arguments) => box_binary::run(arguments, captured),
        Command::FetchBox(arguments) => box_binary::fetch(arguments, captured),
        Command::Ctl(arguments) => {
            let arguments: Vec<_> = arguments.iter().map(|word| OsStr::from_bytes(word).to_owned()).collect();
            Ok(temper_client_shell_cli::start::run(&arguments, &Clock::new()))
        }
    }
}

fn tui_argument(arguments: &[Box<[u8]>]) -> Result<Option<&[u8]>, (u8, String)> {
    let refused = |why: &str| (2, table::refused(&CLIENT, &TUI, why));
    match arguments {
        [] => Ok(None),
        [address] => {
            temper_client_shell_tui::start::address(address).map_err(|why| refused(&why.reason(CLIENT.binary)))?;
            Ok(Some(address))
        }
        _ => Err(refused("invalid arguments")),
    }
}

#[cfg(test)]
mod tests;
