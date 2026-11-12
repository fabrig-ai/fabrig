//! The worker's capture and dispatch (temper/docs/design/cli/README.md, 3, 4 and 7.2).
use crate::{INTERNAL_WORKSPACE_REUSE, PROGRAM_LOGIN};
use skein_io::kernel::Exit;
use skein_shell::{Argument, Command, Reach, Reached, Set, Table};
use std::{ffi::OsStr, os::unix::ffi::OsStrExt, path::PathBuf, process::ExitCode};
use temper_service::startup::{Shape, nearest};
use temper_shell::{
    directories::{self, Directories, EnvironmentEntry},
    environment, table,
};

pub const WORKER: Table = Table {
    binary: "temper-worker",
    line: "the deployment's worker: a worker alone, and its programs' sign-in.",
    // The identity is main's (temper/docs/design/cli/README.md, 3).
    version: "",
    commit: "",
    commands: &[SERVE, PROGRAM_LOGIN, table::HELP],
    settings: &[&directories::SETTINGS],
    variables: &[
        &environment::JOB,
        &INTERNAL_WORKSPACE_REUSE,
        &environment::INTERNAL_DUMP_LLM,
        &temper_shell::environment::INTERNAL_COMMIT,
        &temper_service_shell::WAIT_CHECK,
        &temper_service_shell::CUT_AT_LISTEN,
    ],
    more: "temper-worker COMMAND --help prints one command; temper-worker help pages lists the pages.",
};
const SERVE: Command = Command {
    reach: Reach::Words(&["serve"]),
    set: Set::Public,
    forms: &["[DIRECTORIES]"],
    line: "Run a worker alone's document until stopped; temper-engine serve runs an engine alone, and the box's temper-box serve runs an engine and its worker in one process.",
    arguments: &[Argument {
        shown: temper_shell::commands::DIRECTORIES.shown,
        line: temper_shell::commands::DIRECTORIES.line,
    }],
    settings: &[&directories::SETTINGS],
    switches: &[],
    variables: &[&environment::LISTEN_FDS, &environment::XDG_RUNTIME_DIR],
    notes: "Its stderr carries its startup notes, its linked and lost lines and the reason of a failure. A first termination signal runs its worker's ending; a second aborts. Exit 0 after its ending, 1 on a refused start or any failure. Refuses an engine alone's document, naming temper-engine serve, or an engine and its worker in one process, naming the box's temper-box serve.",
};

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
        Ok(code) => ExitCode::from(code),
        Err((code, why)) => {
            eprint!("{why}");
            ExitCode::from(code)
        }
    }
}

fn run(captured: &Capture, version: &'static str, commit: &'static str) -> Result<u8, (u8, String)> {
    match WORKER.reach(&captured.run_as, &captured.arguments) {
        Reached::Help => {
            print!("{}", WORKER.help());
            Ok(0)
        }
        Reached::Version => {
            print!("{}", Table { version, commit, ..WORKER }.version_line());
            Ok(0)
        }
        Reached::CommandHelp(command) => {
            print!("{}", WORKER.command_help(command));
            Ok(0)
        }
        Reached::Unknown(why) => Err((2, why)),
        Reached::Command(command, arguments) => dispatch(command, arguments, captured),
    }
}

fn dispatch(command: &Command, arguments: &[Box<[u8]>], captured: &Capture) -> Result<u8, (u8, String)> {
    let refused = |why: &str| (2, table::refused(&WORKER, command, why));
    match command.reach {
        Reach::Words(["help"]) => {
            print!("{}", table::help(&WORKER, arguments)?);
            Ok(0)
        }
        Reach::Words(["serve"]) => {
            let working = captured.working.as_ref().map_err(|why| (1, format!("temper-worker serve: {why}\n")))?;
            let (directories, rest) =
                directories::resolve(arguments, &captured.environment, working.as_os_str().as_bytes())
                    .map_err(|why| refused(&why.to_string()))?;
            if !rest.is_empty() {
                return Err(refused("invalid arguments"));
            }
            refuse_document(command, &directories).map_err(|why| (1, why))?;
            Ok(
                match temper_service_shell::run(
                    WORKER.binary,
                    &directories,
                    &captured.environment,
                    crate::served::SERVED,
                    |_| Ok(()),
                    |bytes, owned, ()| {
                        crate::worker::alone(&directories, bytes, owned, &captured.environment).map(|service| {
                            temper_service_shell::Prepared { service, store: Box::new([]), adoption: () }
                        })
                    },
                    |_, _, ()| Ok(()),
                )
                .map_err(|why| (1, why))?
                {
                    Exit::Code(code) => code,
                    Exit::Signal(_) => 1,
                },
            )
        }
        Reach::Words(["program-login"]) => {
            let working =
                captured.working.as_ref().map_err(|why| (1, format!("temper-worker program-login: {why}\n")))?;
            let (directories, rest) =
                directories::resolve(arguments, &captured.environment, working.as_os_str().as_bytes())
                    .map_err(|why| refused(&why.to_string()))?;
            let [name] = rest.as_ref() else { return Err(refused("invalid arguments")) };
            refuse_document(command, &directories).map_err(|why| (1, why))?;
            crate::program_login::run(&directories, OsStr::from_bytes(name), &captured.environment, None).map_err(
                |why| (1, format!("temper-worker program-login: {}\n", why.lines().collect::<Vec<_>>().join("; "))),
            )
        }
        Reach::Words(_) | Reach::RunAs(_) => Err(refused("invalid arguments")),
    }
}

/// Refuse another binary's nearest shape before the worker's document checks (protocol/binaries.md, 5).
fn refuse_document(command: &Command, directories: &Directories) -> Result<(), String> {
    let Ok(bytes) = directories.read() else { return Ok(()) };
    let why = match nearest(&bytes) {
        Some(Shape::EngineAlone) => "an engine alone; temper-engine serve runs it",
        Some(Shape::Standalone) => "an engine and its worker in one process; temper-box serve runs it",
        Some(Shape::WorkerAlone) | None => return Ok(()),
    };
    let words = match command.reach {
        Reach::Words(words) => words.join(" "),
        Reach::RunAs(name) => name.into(),
    };
    Err(format!("{} {words}: temper.json holds {why}\n", WORKER.binary))
}
