//! The worker's capture and dispatch (temper/docs/design/cli/README.md, 3, 4 and 7.2).
use crate::{INTERNAL_WORKSPACE_REUSE, PROGRAM_LOGIN};
use skein_io::kernel::Exit;
use skein_shell::{Argument, Command, Reach, Reached, Set, Table};
use std::{ffi::OsStr, os::unix::ffi::OsStrExt, path::PathBuf, process::ExitCode};
use temper_config_shell::Shape;
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
        &environment::CLOSE_REFUSAL,
        &environment::CLOSE_OUTDATED,
        &temper_service_shell::DISK_BUDGET,
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
        Ok(Reply::Print(text)) => {
            print!("{text}");
            ExitCode::SUCCESS
        }
        Ok(Reply::Exit(code)) => ExitCode::from(code),
        Err((code, why)) => {
            eprint!("{why}");
            ExitCode::from(code)
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Reply {
    Print(String),
    Exit(u8),
}

fn run(captured: &Capture, version: &'static str, commit: &'static str) -> Result<Reply, (u8, String)> {
    let text = match WORKER.reach(&captured.run_as, &captured.arguments) {
        Reached::Help => WORKER.help(),
        Reached::Version => Table { version, commit, ..WORKER }.version_line(),
        Reached::CommandHelp(command) => WORKER.command_help(command),
        Reached::Unknown(why) => return Err((2, why)),
        Reached::Command(command, arguments) if command.reach == Reach::Words(&["help"]) => help(arguments)?,
        Reached::Command(command, arguments) => return dispatch(command, arguments, captured).map(Reply::Exit),
    };
    Ok(Reply::Print(text))
}

fn help(arguments: &[Box<[u8]>]) -> Result<String, (u8, String)> {
    if let [page] = arguments
        && page.as_ref() == b"config"
    {
        Ok(String::from_utf8(temper_config_shell::help(Shape::WorkerAlone).into_vec()).expect("help UTF-8"))
    } else {
        table::help(&WORKER, arguments)
    }
}

fn dispatch(command: &Command, arguments: &[Box<[u8]>], captured: &Capture) -> Result<u8, (u8, String)> {
    match command.reach {
        Reach::Words(["serve"]) => serve_command(arguments, captured, |directories, shape| {
            temper_service_shell::run(
                WORKER.binary,
                directories,
                shape,
                &captured.environment,
                crate::served::SERVED,
                &skein_shell::Clock::new(),
                |_| Ok(()),
                |bytes, programs, owned, ()| {
                    crate::worker::alone_with_programs(directories, bytes, owned, &captured.environment, programs)
                        .map(|service| temper_service_shell::Prepared { service, store: Box::new([]), adoption: () })
                },
                |_, _, ()| Ok(()),
            )
        }),
        Reach::Words(["program-login"]) => login_command(arguments, captured, |directories, name| {
            crate::program_login::run(directories, name, &captured.environment)
        }),
        Reach::Words(_) | Reach::RunAs(_) => Err((2, table::refused(&WORKER, command, "invalid arguments"))),
    }
}

fn serve_command(
    arguments: &[Box<[u8]>],
    captured: &Capture,
    serve: impl FnOnce(&Directories, Shape) -> Result<Exit, String>,
) -> Result<u8, (u8, String)> {
    let refused = |why: &str| (2, table::refused(&WORKER, &SERVE, why));
    let working = captured.working.as_ref().map_err(|why| (1, format!("temper-worker serve: {why}\n")))?;
    let (directories, rest) = directories::resolve(arguments, &captured.environment, working.as_os_str().as_bytes())
        .map_err(|why| refused(&why.to_string()))?;
    if !rest.is_empty() {
        return Err(refused("invalid arguments"));
    }
    Ok(match serve(&directories, Shape::WorkerAlone).map_err(|why| (1, why))? {
        Exit::Code(code) => code,
        Exit::Signal(_) => 1,
    })
}

fn login_command(
    arguments: &[Box<[u8]>],
    captured: &Capture,
    login: impl FnOnce(&Directories, &OsStr) -> Result<u8, String>,
) -> Result<u8, (u8, String)> {
    let refused = |why: &str| (2, table::refused(&WORKER, &PROGRAM_LOGIN, why));
    let working = captured.working.as_ref().map_err(|why| (1, format!("temper-worker program-login: {why}\n")))?;
    let (directories, rest) = directories::resolve(arguments, &captured.environment, working.as_os_str().as_bytes())
        .map_err(|why| refused(&why.to_string()))?;
    let [name] = rest.as_ref() else { return Err(refused("invalid arguments")) };
    login(&directories, OsStr::from_bytes(name))
        .map_err(|why| (1, temper_service_shell::configuration::login_problems("temper-worker program-login", &why)))
}

#[cfg(test)]
mod tests;
