//! temper-worker's table and dispatch (temper/docs/design/protocol/binaries.md, 5).
use crate::program_login::PROGRAM_LOGIN;
use skein_io::kernel::Exit;
use std::{env, ffi::OsStr, os::unix::ffi::OsStrExt, process::ExitCode};
use temper_service::startup::{Shape, nearest};
use temper_shell::{
    commands::{DIRECTORIES, HELP, Row, Table, Who},
    directories::{self, Directories},
    help,
};

pub const WORKER: Table = Table { binary: "temper-worker", rows: &ROWS };
const ROWS: [Row; 3] = [HELP, SERVE, PROGRAM_LOGIN];
const SERVE: Row = Row {
    words: &["serve"],
    who: Who::Deployment,
    forms: &["[DIRECTORIES]"],
    line: "Run a worker alone's document until stopped; temper-engine serve runs an engine alone, and the box's temper-box serve runs an engine and its worker in one process.",
    arguments: &[DIRECTORIES],
    settings: &[&directories::SETTINGS],
    notes: "Its stderr carries its startup notes, its linked and lost lines and the reason of a failure. A first termination signal runs its worker's ending; a second aborts. Exit 0 after its ending, 1 on a refused start or any failure. Refuses an engine alone's document, naming temper-engine serve, or an engine and its worker in one process, naming the box's temper-box serve.",
};

#[expect(
    clippy::disallowed_methods,
    reason = "cli C6: arguments select the worker command; temper-worker's table will read them"
)]
#[must_use]
pub fn main() -> ExitCode {
    match run(&skein_shell::arguments()) {
        Ok(code) => ExitCode::from(code),
        Err(why) => {
            eprint!("{why}");
            ExitCode::FAILURE
        }
    }
}

#[expect(
    clippy::disallowed_methods,
    reason = "cli C6: the process environment resolves directories; temper-worker's table will pass it"
)]
fn run(arguments: &[Box<[u8]>]) -> Result<u8, String> {
    if arguments == [Box::from(b"--version".as_slice())] {
        println!("{}", temper_version::line("temper-worker"));
        return Ok(0);
    }
    if arguments == [Box::from(b"--help".as_slice())] {
        print!("{}", WORKER.help());
        return Ok(0);
    }
    let row = WORKER.find(arguments).ok_or_else(|| WORKER.unknown(arguments))?;
    let arguments = &arguments[row.words.len()..];
    if arguments == [Box::from(b"--help".as_slice())] {
        print!("{}", WORKER.row(row));
        return Ok(0);
    }
    let invalid = || WORKER.refused(row, "invalid arguments");
    match row.words {
        ["help"] => {
            let request = help::request(arguments).ok_or_else(invalid)?;
            print!("{}", help::answer(WORKER.binary, &request)?);
            Ok(0)
        }
        ["serve"] => {
            let working = env::current_dir().map_err(|why| WORKER.refused(row, &why.to_string()))?;
            let (directories, rest) =
                directories::resolve(arguments, &skein_shell::environment(), working.as_os_str().as_bytes())
                    .map_err(|why| WORKER.refused(row, &why.to_string()))?;
            if !rest.is_empty() {
                return Err(invalid());
            }
            refuse_document(row, &directories)?;
            Ok(
                match temper_service_shell::run(WORKER.binary, &directories, false, |bytes, owned, _| {
                    crate::worker::alone(&directories, bytes, owned).map(|service| temper_service_shell::Prepared {
                        service,
                        store: Box::new([]),
                        line_bytes: 0,
                    })
                })? {
                    Exit::Code(code) => code,
                    Exit::Signal(_) => 1,
                },
            )
        }
        ["program-login"] => {
            let working = env::current_dir().map_err(|why| WORKER.refused(row, &why.to_string()))?;
            let (directories, rest) =
                directories::resolve(arguments, &skein_shell::environment(), working.as_os_str().as_bytes())
                    .map_err(|why| WORKER.refused(row, &why.to_string()))?;
            let [name] = rest.as_ref() else { return Err(invalid()) };
            refuse_document(row, &directories)?;
            crate::program_login::run(&directories, OsStr::from_bytes(name), None)
                .map_err(|why| format!("temper-worker program-login: {}\n", why.lines().collect::<Vec<_>>().join("; ")))
        }
        _ => Err(invalid()),
    }
}

/// Refuse another binary's nearest shape before the worker's document checks (4.5 and 15.1).
fn refuse_document(row: &Row, directories: &Directories) -> Result<(), String> {
    let Ok(bytes) = directories.read() else { return Ok(()) };
    let why = match nearest(&bytes) {
        Some(Shape::EngineAlone) => "an engine alone; temper-engine serve runs it",
        Some(Shape::Standalone) => "an engine and its worker in one process; the box's temper-box serve runs it",
        Some(Shape::WorkerAlone) | None => return Ok(()),
    };
    Err(format!("{} {}: temper.json holds {why}\n", WORKER.binary, row.words.join(" ")))
}
