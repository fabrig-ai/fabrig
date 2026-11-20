//! The journeys binary's capture and dispatch (temper/docs/design/testing/tools.md, 3–4).
use crate::{
    EnvironmentEntry, TABLE,
    table::{ADD, CAUTION, DRIVE, NORMALISE, RECIPE_HASH, RECIPE_LIST, RECIPE_WALK},
};
use crate::{JOURNEYS, Refusal, TUI_JOURNEYS, add_to, named, reprint, tui};
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
            } else if *command == DRIVE {
                return drive(rest, &environment);
            } else if *command == ADD {
                let rest: Vec<_> = rest.iter().map(|word| OsStr::from_bytes(word).to_os_string()).collect();
                add_arguments(&rest)
            } else if *command == NORMALISE {
                if !rest.is_empty() {
                    usage(&NORMALISE, "unexpected arguments");
                }
                normalise_all()
            } else {
                unreachable!("the table names every dispatched command")
            }
        }
    };
    match result {
        Ok(false) => ExitCode::SUCCESS,
        Ok(true) => ExitCode::FAILURE,
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

fn add_arguments(arguments: &[OsString]) -> Result<bool, String> {
    let mut positional = Vec::new();
    let mut given = Vec::new();
    let mut args = arguments.iter();
    while let Some(argument) = args.next() {
        if argument == "--redact" {
            let Some(text) = args.next() else { usage(&ADD, "--redact TEXT is not given") };
            let bytes = text.as_bytes();
            if bytes.is_empty() || bytes.starts_with(b"--") {
                usage(&ADD, "--redact TEXT is empty or starts with --");
            }
            given.push(bytes.into());
        } else if argument.as_bytes().starts_with(b"--") {
            usage(&ADD, "unknown flag");
        } else {
            positional.push(argument);
        }
    }
    let [name, file] = positional.as_slice() else { usage(&ADD, "expected NAME and FILE") };
    let Some(name) = name.to_str() else {
        return Err("invalid journey name; expected [a-z][a-z0-9-]*; nothing was written".into());
    };
    if !named(name) {
        return Err(format!("{}; nothing was written", refusal(Refusal::Name(name.to_owned()))));
    }
    let recording = fs::read(file).map_err(|_| "cannot read FILE; nothing was written".to_owned())?;
    let added = add_to(Path::new(JOURNEYS), Path::new(TUI_JOURNEYS), name, &recording, &given)
        .map_err(|why| format!("{}; nothing was written", refusal(why)))?;
    let is_tui = recording.split(|byte| *byte == b'\n').any(|line| line.starts_with(b"screen "));
    println!("{}", addition(&added, is_tui));
    Ok(false)
}

pub(crate) fn addition(added: &crate::Added, is_tui: bool) -> String {
    let redaction = if is_tui {
        format!(
            "link: {}; stored: {}; typed: {}; given: {}; approval: {}; credential: {}; bearer: {}; places replaced: {}",
            added.link,
            added.stored,
            added.typed,
            added.given,
            added.approval,
            added.credential,
            added.bearer,
            added.places
        )
    } else {
        format!(
            "link: {}; stored: {}; typed: {}; given: {}; places replaced: {}",
            added.link, added.stored, added.typed, added.given, added.places
        )
    };
    let placeholders = placeholder_counts(added.build, added.port, added.code, added.placeholder_places, added.stood);
    format!("{redaction}\n{placeholders}\n{CAUTION}")
}

fn placeholder_counts(build: u32, port: u32, code: u32, places: u32, stood: u32) -> String {
    format!("placeholders: build: {build}; port: {port}; code: {code}; places replaced: {places}; stood: {stood}")
}

fn normalise_all() -> Result<bool, String> {
    let (written, stale) = crate::normalise(Path::new(JOURNEYS), Path::new(TUI_JOURNEYS)).map_err(refusal)?;
    for (name, normalised) in written {
        println!("{name}: normalised");
        println!(
            "{}",
            placeholder_counts(normalised.build, normalised.port, normalised.code, normalised.places, normalised.stood)
        );
    }
    let has_stale = !stale.is_empty();
    for why in stale {
        eprintln!("temper-client-journeys: {}; recording left as it was", refusal(why));
    }
    let reprint_stale = reprint_all()?;
    Ok(has_stale || reprint_stale)
}

fn reprint_all() -> Result<bool, String> {
    reprint_roots(Path::new(JOURNEYS), Path::new(TUI_JOURNEYS), Destination::Stdout, output)
        .map(|stale| !stale.is_empty())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Destination {
    Stdout,
    Stderr,
}

fn output(destination: Destination, line: &str) {
    match destination {
        Destination::Stdout => println!("{line}"),
        Destination::Stderr => eprintln!("{line}"),
    }
}

fn reprint_roots(
    web: &Path,
    tui: &Path,
    names_to: Destination,
    mut show: impl FnMut(Destination, &str),
) -> Result<Vec<String>, String> {
    let stale = reprint(web, web).map_err(refusal)?;
    let tui_stale = tui::reprint(tui, tui).map_err(refusal)?;
    let mut names = Vec::new();
    for journeys in [web, tui] {
        for entry in fs::read_dir(journeys).map_err(|_| format!("cannot read {}", journeys.display()))? {
            let entry = entry.map_err(|_| format!("cannot read {}", journeys.display()))?;
            let path = entry.path();
            if entry.file_type().map_err(|_| format!("cannot read {}", path.display()))?.is_dir()
                && !crate::moved(&path).map_err(refusal)?
                && path.join("recording").is_file()
            {
                names.push(entry.file_name());
            }
        }
    }
    names.sort();
    for name in names {
        show(names_to, &name.to_string_lossy());
    }
    for (name, stopped) in &stale {
        show(Destination::Stderr, &format!("temper-client-journeys: {}", stale_message(name, stopped)));
    }
    for (name, stopped) in &tui_stale {
        show(
            Destination::Stderr,
            &format!(
                "temper-client-journeys: {name} is stale at turn {}, line {}; record it again (client/tui/journeys.md, 8)",
                stopped.turn, stopped.line
            ),
        );
    }
    Ok(stale.into_iter().map(|(name, _)| name).chain(tui_stale.into_iter().map(|(name, _)| name)).collect())
}

pub(crate) fn refusal(why: Refusal) -> String {
    match why {
        Refusal::OtherClient { name } => format!("{name} belongs to the other client's journey directory"),
        Refusal::Moved { name } => format!("{name} has a walk of version 2; use temper-client-walk"),
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
        Refusal::Name(name) => format!("invalid journey name {name:?}; expected [a-z][a-z0-9-]*"),
        Refusal::Proof { name, refusal } => match refusal {
            journey::Refusal::Unreadable(unreadable) => unreadable_message(&name, &unreadable),
            journey::Refusal::Stale(stale) => stale_refusal_message(&name, &stale),
            journey::Refusal::Unproved { mark } => format!("{name} has no live print at mark {mark}"),
            journey::Refusal::Differs { mark, byte } => format!("{name} differs at mark {mark}, byte {byte}"),
        },
        Refusal::Unreadable { name, unreadable } => unreadable_message(&name, &unreadable),
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
// the recording (client/web/journeys.md, 4.3). Reprint keeps the full message below.
fn stale_refusal_message(name: &str, stale: &journey::Stale) -> String {
    let recorded = stale.recorded.split_whitespace().next().unwrap_or("");
    let asked = stale.asked.split_whitespace().next().unwrap_or("");
    format!(
        "{name} is stale at turn {}, line {}: recorded {recorded}, asked {asked}; record it again (client/web/journeys.md, 8)",
        stale.turn, stale.line
    )
}

fn stale_message(name: &str, stale: &journey::Stale) -> String {
    format!(
        "{name} is stale at turn {}, line {}: recorded {}, asked {}; record it again (client/web/journeys.md, 8)",
        stale.turn, stale.line, stale.recorded, stale.asked
    )
}

fn fail(why: &str) -> ! {
    eprintln!("temper-client-journeys: {why}");
    std::process::exit(1);
}
