//! The journeys binary's capture and dispatch (temper/docs/design/testing/tools.md, 3–4).
use crate::{Build, Refusal};
use crate::{
    EnvironmentEntry, TABLE,
    table::{CAUTION, DRIVE, JOURNEY_WALK, PRINT, RECIPE_HASH, RECIPE_LIST, RECIPE_WALK},
};
use skein_shell::{Front, Table};
use std::process::ExitCode;
use std::{
    ffi::{OsStr, OsString},
    fs,
    os::unix::ffi::OsStrExt,
    path::Path,
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
    let result = match (Table { version, commit, ..TABLE }).front(&arguments) {
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
            } else if *command == PRINT {
                let rest: Vec<_> = rest.iter().map(|word| OsStr::from_bytes(word).to_os_string()).collect();
                print_arguments(
                    &rest,
                    &Build { wire: jig_client_protocol_wire::VERSION, temper: format!("{version} ({commit})") },
                )
            } else {
                unreachable!("the table names every dispatched command")
            }
        }
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(why) => fail(&why),
    }
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

fn usage(command: &skein_shell::Command, why: &str) -> ! {
    eprint!("{}", TABLE.refusal(command, why));
    std::process::exit(2);
}

fn print_arguments(arguments: &[OsString], build: &Build) -> Result<(), String> {
    let mut positional = Vec::new();
    let mut given = Vec::new();
    let mut args = arguments.iter();
    while let Some(argument) = args.next() {
        if argument == "--redact" {
            let Some(text) = args.next() else { usage(&PRINT, "--redact TEXT is not given") };
            let bytes = text.as_bytes();
            if bytes.is_empty() || bytes.starts_with(b"--") {
                usage(&PRINT, "--redact TEXT is empty or starts with --");
            }
            given.push(bytes.into());
        } else if argument.as_bytes().starts_with(b"--") {
            usage(&PRINT, "unknown flag");
        } else {
            positional.push(argument);
        }
    }
    let [file, out] = positional.as_slice() else { usage(&PRINT, "expected FILE and OUT") };
    let destination = match crate::destination(Path::new(out)) {
        Ok(out) => out,
        Err(Refusal::OutputRoot) => usage(&PRINT, "OUT is inside a journeys root"),
        Err(why) => return Err(format!("{}; nothing was written", refusal(why))),
    };
    let recording = fs::read(file).map_err(|_| "cannot read FILE; nothing was written".to_owned())?;
    let printed = crate::print(file, &recording, &given, &destination, build)
        .map_err(|why| format!("{}; nothing was written", refusal(why)))?;
    let is_tui = recording.split(|byte| *byte == b'\n').any(|line| line.starts_with(b"screen "));
    println!("{}", printing(&printed, is_tui, Path::new(out)));
    Ok(())
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

fn fail(why: &str) -> ! {
    eprintln!("temper-client-journeys: {why}");
    std::process::exit(1);
}
