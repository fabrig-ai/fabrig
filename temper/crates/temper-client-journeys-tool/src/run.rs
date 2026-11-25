//! The journeys binary's capture and dispatch (temper/docs/design/testing/tools.md, 3–4).
use crate::{Build, REPOSITORY, Refusal, output};
use crate::{
    EnvironmentEntry, TABLE,
    host::Host,
    table::{CAUTION, DRIVE, JOURNEY_WALK, LOOK_REFRESH, PRINT, RECIPE_HASH, RECIPE_LIST, RECIPE_WALK, WRITER},
};
use skein_shell::{Front, Table};
use std::process::ExitCode;
use std::{
    ffi::OsStr,
    fs, io,
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
};
use temper_client_shell_web::journey;

/// Run the journeys binary (temper/docs/design/testing/tools.md, 4.2–4.3).
#[expect(
    clippy::disallowed_methods,
    reason = "the binary's capture: temper-client-journeys' arguments and environment, once (skein/docs/foundation/cli.md, 6)"
)]
pub fn main(version: &'static str, commit: &'static str) -> ExitCode {
    let arguments = skein_shell::arguments();
    let environment = skein_shell::environment();
    let outcome = match (Table { version, commit, ..TABLE }).front(&arguments) {
        Front::Print(text) => {
            print!("{text}");
            return ExitCode::SUCCESS;
        }
        Front::Refuse(text) => {
            eprint!("{text}");
            return ExitCode::from(2);
        }
        Front::Run { command, rest } => {
            if *command == RECIPE_WALK || *command == RECIPE_HASH || *command == RECIPE_LIST {
                let mode = if *command == RECIPE_WALK {
                    crate::recipe::Mode::Walk
                } else if *command == RECIPE_HASH {
                    crate::recipe::Mode::Hash
                } else {
                    crate::recipe::Mode::List
                };
                return session_exit(crate::recipe::run(mode, rest, &environment));
            } else if *command == JOURNEY_WALK {
                return session_exit(crate::journey_walk::run(rest, &environment));
            } else if *command == DRIVE {
                return drive(rest, &environment);
            } else if *command == WRITER {
                let mut host = Host::real(environment);
                return ExitCode::from(crate::writer::run(rest, &mut host, std::process::id()));
            } else if *command == LOOK_REFRESH {
                return ExitCode::from(crate::look_refresh::run(
                    rest,
                    &mut Host::real(environment),
                    std::process::id(),
                ));
            } else if *command == PRINT {
                print_command(
                    rest,
                    &Build { wire: jig_client_protocol_wire::VERSION, temper: format!("{version} ({commit})") },
                    |out| {
                        crate::destination(
                            out,
                            Path::new(REPOSITORY),
                            |path| std::path::absolute(path),
                            |path| fs::canonicalize(path),
                        )
                    },
                    |file| fs::read(file),
                    &mut output::Disk,
                )
            } else {
                unreachable!("the table names every dispatched command")
            }
        }
    };
    print!("{}", outcome.stdout);
    eprint!("{}", outcome.stderr);
    ExitCode::from(outcome.exit)
}

fn drive(arguments: &[Box<[u8]>], environment: &[EnvironmentEntry]) -> ExitCode {
    let exit = match crate::drive::Options::parse(arguments) {
        Ok(options) => match crate::drive::Session::start(options, environment) {
            Ok(session) => session.stdio(),
            Err(exit) => exit,
        },
        Err(why) => crate::drive::Exit::own(why),
    };
    session_exit(exit)
}

fn session_exit(exit: crate::drive::Exit) -> ExitCode {
    for line in exit.shown {
        println!("{line}");
    }
    if let Some(error) = exit.error {
        eprintln!("{error}");
    }
    ExitCode::from(exit.status)
}

pub(super) struct Outcome {
    pub(super) exit: u8,
    pub(super) stdout: String,
    pub(super) stderr: String,
}

impl Outcome {
    fn usage(why: &str) -> Self {
        Self { exit: 2, stdout: String::new(), stderr: TABLE.refusal(&PRINT, why) }
    }

    fn failed(why: &str) -> Self {
        Self { exit: 1, stdout: String::new(), stderr: format!("temper-client-journeys: {why}; nothing was written\n") }
    }
}

pub(super) struct PrintArguments<'a> {
    pub(super) file: &'a OsStr,
    pub(super) out: &'a Path,
    pub(super) given: Vec<Box<[u8]>>,
}

pub(super) fn print_arguments(arguments: &[Box<[u8]>]) -> Result<PrintArguments<'_>, &'static str> {
    let mut positional = Vec::new();
    let mut given = Vec::new();
    let mut args = arguments.iter().map(AsRef::as_ref);
    while let Some(argument) = args.next() {
        if argument == b"--redact" {
            let text = args.next().ok_or("--redact TEXT is not given")?;
            if text.is_empty() || text.starts_with(b"--") {
                return Err("--redact TEXT is empty or starts with --");
            }
            given.push(text.into());
        } else if argument.starts_with(b"--") {
            return Err("unknown flag");
        } else {
            positional.push(OsStr::from_bytes(argument));
        }
    }
    let &[file, out] = positional.as_slice() else { return Err("expected FILE and OUT") };
    Ok(PrintArguments { file, out: Path::new(out), given })
}

pub(super) fn print_command(
    arguments: &[Box<[u8]>],
    build: &Build,
    mut resolve: impl FnMut(&Path) -> Result<PathBuf, Refusal>,
    read: impl FnOnce(&OsStr) -> io::Result<Vec<u8>>,
    output: &mut impl output::Output,
) -> Outcome {
    let PrintArguments { file, out, given } = match print_arguments(arguments) {
        Ok(arguments) => arguments,
        Err(why) => return Outcome::usage(why),
    };
    let destination = match resolve(out) {
        Ok(out) => out,
        Err(Refusal::OutputRoot) => return Outcome::usage("OUT is inside a journeys root"),
        Err(why) => return Outcome::failed(&refusal(why)),
    };
    let Ok(recording) = read(file) else { return Outcome::failed("cannot read FILE") };
    // Recheck the resolved destination after the read, before admitting output.
    let destination = match resolve(&destination) {
        Ok(out) => out,
        Err(why) => return Outcome::failed(&refusal(why)),
    };
    let admitted = match crate::admit(file, &recording, &given, build) {
        Ok(admitted) => admitted,
        Err(why) => return Outcome::failed(&refusal(why)),
    };
    if let Err(why) = output::publish(output, &destination, &admitted) {
        return Outcome::failed(&refusal(why));
    }
    let stdout = printing(&admitted.printed, admitted.client == crate::Client::Tui, out);
    Outcome { exit: 0, stdout: format!("{stdout}\n"), stderr: String::new() }
}

pub(crate) fn printing(printed: &crate::Printed, is_tui: bool, out: &Path) -> String {
    let redaction = if is_tui {
        format!(
            "link: {}; stored: {}; typed: {}; given: {}; approval: {}; credential: {}; bearer: {}; places replaced: {}",
            printed.link,
            printed.stored,
            printed.typed,
            printed.given,
            printed.approval,
            printed.credential,
            printed.bearer,
            printed.places
        )
    } else {
        format!(
            "link: {}; stored: {}; typed: {}; given: {}; places replaced: {}",
            printed.link, printed.stored, printed.typed, printed.given, printed.places
        )
    };
    format!("{redaction}\nprinted {} into {}\n{CAUTION}", printed.name, out.display())
}

pub(crate) fn refusal(why: Refusal) -> String {
    match why {
        Refusal::OutputRoot => "OUT is inside a journeys root".into(),
        Refusal::Layout { file, recorded, read } => {
            format!("{file} was recorded {recorded}; {read}: print it with the build that recorded it")
        }
        Refusal::TuiProof { name, refusal } => match refusal {
            temper_client_shell_tui::journey::Refusal::Unreadable(why) => {
                format!("{name} is unreadable at line {}: {}", why.line, why.why)
            }
            temper_client_shell_tui::journey::Refusal::Stale(why) => format!(
                "{name} is stale at turn {}, line {}: recorded {}, asked {}; record it again (client/tui/journeys.md, 8)",
                why.turn,
                why.line,
                String::from_utf8_lossy(&why.recorded),
                String::from_utf8_lossy(&why.asked)
            ),
            temper_client_shell_tui::journey::Refusal::Differs { mark, line } => {
                format!("{name} differs at mark {mark}, frame line {line}")
            }
        },
        Refusal::TuiFound { name, found, file } => {
            let place = match found.place {
                temper_client_shell_tui::journey::Place::Field => "field".into(),
                temper_client_shell_tui::journey::Place::Frame { turn } => format!("frame at turn {turn}"),
                temper_client_shell_tui::journey::Place::Printed => {
                    format!("printed file {}", file.unwrap_or_default())
                }
            };
            format!("{name} at line {}, {}, {place}: found {}", found.line, found.record, what_message(found.what))
        }
        Refusal::Undecoded { name, line } => format!("{name} has an undecodable stream record at line {line}"),
        Refusal::Found { name, line, record, place, what } => {
            format!("{name} at line {line}, {record}, {}: found {}", place_message(place), what_message(what))
        }
        Refusal::FoundDrawn { name, turn, what } => {
            format!("{name} at turn {turn}, drawn page: found {}", what_message(what))
        }
        Refusal::FoundPrinted { name, page, what } => format!("{name} at page {page}: found {}", what_message(what)),
        Refusal::Proof { name, refusal } => match refusal {
            journey::Refusal::Unreadable(unreadable) => unreadable_message(&name, &unreadable),
            journey::Refusal::Stale(stale) => stale_refusal_message(&name, &stale),
            journey::Refusal::Unproved { mark } => format!("{name} has no live print at mark {mark}"),
            journey::Refusal::Differs { mark, byte } => format!("{name} differs at mark {mark}, byte {byte}"),
        },
        Refusal::Io { path } => format!("cannot read or write {}", path.display()),
    }
}

fn place_message(place: journey::Place) -> &'static str {
    match place {
        journey::Place::Field => "field",
        journey::Place::Record => "stream record",
        journey::Place::Plain => "stream plain text",
        journey::Place::RunText => "joined run text",
    }
}

fn what_message(what: journey::What) -> &'static str {
    match what {
        journey::What::Original => "an original",
        journey::What::Shape(journey::Shape::Key) => "a key",
        journey::What::Shape(journey::Shape::Jwt) => "a JWT",
        journey::What::Shape(journey::Shape::Bearer) => "a bearer",
        journey::What::Shape(journey::Shape::Code) => "a code",
        journey::What::Shape(journey::Shape::RefreshToken) => "a refresh token",
    }
}

fn unreadable_message(name: &str, unreadable: &journey::Unreadable) -> String {
    format!("{name} is unreadable at line {}: {}", unreadable.line, unreadable.why)
}

// A failed proof may hold bytes the redaction missed: only record kinds leave
// the recording (client/web/journeys-recordings.md, 2.3).
fn stale_refusal_message(name: &str, stale: &journey::Stale) -> String {
    let recorded = stale.recorded.split_whitespace().next().unwrap_or("");
    let asked = stale.asked.split_whitespace().next().unwrap_or("");
    format!(
        "{name} is stale at turn {}, line {}: recorded {recorded}, asked {asked}; record it again (client/web/journeys.md, 8)",
        stale.turn, stale.line
    )
}
