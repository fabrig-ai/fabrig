//! The journeys binary's capture and dispatch (temper/docs/design/testing/tools.md, 3–4).
use crate::again::{self, Paths};
use crate::{
    EnvironmentEntry, TABLE,
    table::{ADD, AGAIN, CAUTION, CHROMIUM, REPRINT, TARGET},
};
use crate::{JOURNEYS, Refusal, TUI_JOURNEYS, add_to, named, reprint, tui, tui_again};
use skein_shell::Front;
use std::process::Command;
use std::process::ExitCode;
use std::{
    ffi::{OsStr, OsString},
    fs, io,
    os::unix::ffi::OsStrExt,
    os::unix::fs::DirBuilderExt,
    path::{Path, PathBuf},
};
use temper_client_shell_web::journey;

/// Run the journeys binary (temper/docs/design/testing/tools.md, 4.2–4.3).
#[must_use]
#[expect(
    clippy::disallowed_methods,
    reason = "the binary's capture: temper-client-journeys' arguments and environment, once (skein/docs/foundation/cli.md, 6)"
)]
pub fn main() -> ExitCode {
    let arguments = skein_shell::arguments();
    let environment = skein_shell::environment();
    let result = match TABLE.front(&arguments) {
        Front::Print(text) => {
            print!("{text}");
            return ExitCode::SUCCESS;
        }
        Front::Refuse(text) => {
            eprint!("{text}");
            return ExitCode::from(2);
        }
        Front::Run { command, rest } => {
            if *command == ADD {
                let rest: Vec<_> = rest.iter().map(|word| OsStr::from_bytes(word).to_os_string()).collect();
                add_arguments(&rest)
            } else if *command == AGAIN {
                let [name] = rest else {
                    usage(&AGAIN, if rest.is_empty() { "NAME is not given" } else { "expected NAME alone" });
                };
                let Ok(name) = std::str::from_utf8(name) else {
                    fail(&format!("invalid journey name {}", OsStr::from_bytes(name).display()));
                };
                record_again(name, &environment).map(|()| false)
            } else {
                if !rest.is_empty() {
                    usage(&REPRINT, "unexpected arguments");
                }
                reprint_all()
            }
        }
    };
    match result {
        Ok(false) => ExitCode::SUCCESS,
        Ok(true) => ExitCode::FAILURE,
        Err(why) => fail(&why),
    }
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
    if recording.split(|byte| *byte == b'\n').any(|line| line.starts_with(b"screen ")) {
        println!(
            "link: {}; stored: {}; typed: {}; given: {}; approval: {}; credential: {}; bearer: {}; places replaced: {}",
            added.link,
            added.stored,
            added.typed,
            added.given,
            added.approval,
            added.credential,
            added.bearer,
            added.places
        );
    } else {
        println!(
            "link: {}; stored: {}; typed: {}; given: {}; places replaced: {}",
            added.link, added.stored, added.typed, added.given, added.places
        );
    }
    println!("{CAUTION}");
    Ok(false)
}

fn record_again(name: &str, environment: &[EnvironmentEntry]) -> Result<(), String> {
    if !named(name) {
        return Err(refusal(Refusal::Name(name.to_owned())));
    }
    let is_tui = Path::new(TUI_JOURNEYS).join(name).join("walk").exists();
    // Unwalkable journeys are refused before starting the builds or either child.
    let tui_steps = if is_tui {
        Some(tui_again::starting(Path::new(TUI_JOURNEYS), Path::new(JOURNEYS), name).map_err(refusal)?)
    } else {
        let _steps = again::starting(Path::new(JOURNEYS), name).map_err(refusal)?;
        let _setup = again::setup(Path::new(JOURNEYS), name).map_err(refusal)?;
        None
    };
    let chromium = CHROMIUM
        .value(environment)
        .map(OsStr::from_bytes)
        .ok_or_else(|| format!("{name}: set SKEIN_TEST_CHROMIUM to the Chromium binary"))?;
    let chromium = fs::canonicalize(chromium)
        .map_err(|error| format!("{name}: cannot open Chromium {}: {error}", Path::new(&chromium).display()))?;
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let scratch = scratch().map_err(|error| format!("{name}: cannot make a fresh scratch directory: {error}"))?;
    let result = build(&root, &scratch, chromium, is_tui, environment).and_then(|paths| {
        if let Some(steps) = tui_steps {
            tui_again::again(Path::new(JOURNEYS), Path::new(TUI_JOURNEYS), name, &steps, &paths, environment)
                .map_err(refusal)
        } else {
            again::again(Path::new(JOURNEYS), name, &paths, environment).map_err(refusal)
        }
    });
    match result {
        Ok(()) => fs::remove_dir_all(&scratch)
            .map_err(|error| format!("{name}: cannot remove scratch directory {}: {error}", scratch.display())),
        Err(why) => Err(format!("{name}: {why}\nscratch directory kept at {}", scratch.display())),
    }
}

fn scratch() -> io::Result<PathBuf> {
    let base = Path::new("/tmp");
    for attempt in 0..1000 {
        let path = base.join(format!("temper-journey-{}-{attempt}", std::process::id()));
        match fs::DirBuilder::new().mode(0o700).create(&path) {
            Ok(()) => return Ok(path),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error),
        }
    }
    Err(io::Error::new(io::ErrorKind::AlreadyExists, "no fresh journey directory name"))
}

fn build(
    root: &Path,
    scratch: &Path,
    chromium: PathBuf,
    is_tui: bool,
    environment: &[EnvironmentEntry],
) -> Result<Paths, String> {
    let mut cargo = Command::new("cargo");
    cargo.env_clear().envs(environment.iter().map(|(name, value)| (OsStr::from_bytes(name), OsStr::from_bytes(value))));
    cargo.current_dir(root).env("TEMPER_INTERNAL_BUNDLE", "none").args([
        "build",
        "-p",
        "temper-box",
        "-p",
        "smith-fake-claude",
    ]);
    if is_tui {
        cargo.args(["-p", "temper-client", "--features", "temper-client-shell-tui/record"]);
    }
    let status = cargo.status().map_err(|error| format!("cannot start cargo build: {error}"))?;
    if !status.success() {
        return Err(format!("cargo build failed: {status}"));
    }
    let bundle = scratch.join("bundle");
    let mut bundler = Command::new(root.join("scripts/bundle.sh"));
    bundler
        .env_clear()
        .envs(environment.iter().map(|(name, value)| (OsStr::from_bytes(name), OsStr::from_bytes(value))));
    bundler.current_dir(root).env("TEMPER_INTERNAL_BUNDLE", "none");
    let status = bundler.arg(&bundle).status().map_err(|error| format!("cannot start scripts/bundle.sh: {error}"))?;
    if !status.success() {
        return Err(format!("scripts/bundle.sh failed: {status}"));
    }
    let target = TARGET
        .value(environment)
        .map(OsStr::from_bytes)
        .map_or_else(|| root.join("../target"), |target| root.join(target));
    for name in ["temper-box", "fake-claude"] {
        fs::copy(target.join("debug").join(name), scratch.join(name))
            .map_err(|error| format!("cannot copy {name} into {}: {error}", scratch.display()))?;
    }
    if is_tui {
        fs::copy(target.join("debug/temper-client"), scratch.join("temper-client"))
            .map_err(|error| format!("cannot copy temper-client into {}: {error}", scratch.display()))?;
    }
    Ok(Paths { temper: scratch.join("temper-box"), bundle, chromium, scratch: scratch.to_owned() })
}

fn reprint_all() -> Result<bool, String> {
    let stale = reprint(Path::new(JOURNEYS), Path::new(JOURNEYS)).map_err(refusal)?;
    let tui_stale = tui::reprint(Path::new(TUI_JOURNEYS), Path::new(TUI_JOURNEYS)).map_err(refusal)?;
    let mut names = Vec::new();
    for journeys in [JOURNEYS, TUI_JOURNEYS] {
        for entry in fs::read_dir(journeys).map_err(|_| format!("cannot read {journeys}"))? {
            let entry = entry.map_err(|_| format!("cannot read {journeys}"))?;
            let path = entry.path();
            if entry.file_type().map_err(|_| format!("cannot read {}", path.display()))?.is_dir()
                && path.join("recording").is_file()
            {
                names.push(entry.file_name());
            }
        }
    }
    names.sort();
    for name in names {
        println!("{}", name.to_string_lossy());
    }
    for (name, stopped) in &stale {
        eprintln!("temper-client-journeys: {}", stale_message(name, stopped));
    }
    for (name, stopped) in &tui_stale {
        eprintln!(
            "temper-client-journeys: {name} is stale at turn {}, line {}; record it again (client/tui/journeys.md, 8)",
            stopped.turn, stopped.line
        );
    }
    Ok(!stale.is_empty() || !tui_stale.is_empty())
}

fn refusal(why: Refusal) -> String {
    match why {
        Refusal::TuiWalk { name, unreadable } => {
            format!("{name}/walk is unreadable at line {}: {}", unreadable.line, unreadable.why)
        }
        Refusal::TuiAgain { name, step, report } => format!("{name} at step {step}: {report}"),
        Refusal::OtherClient { name } => format!("{name} belongs to the other client's journey directory"),
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
        Refusal::SetupUnreadable { name, unreadable } => unreadable_message(&format!("{name}/setup"), &unreadable),
        Refusal::SetupAfter { name, after, why } => format!("{name}/setup: after {after:?}: {why}"),
        Refusal::SetupTimeout { name, setup, line, sought } => {
            format!("{name}: {setup}/setup line {line}: {sought} not done within 60 s")
        }
        Refusal::MarkTimeout { name, mark, text } => {
            format!("{name}: mark {mark}: text {:?} not shown within 30 s", String::from_utf8_lossy(&text))
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
        Refusal::Unwalkable { name, unwalkable } => match unwalkable {
            journey::Unwalkable::Unreadable(unreadable) => unreadable_message(&name, &unreadable),
            journey::Unwalkable::Drawing { step } => {
                format!("{name} is unwalkable at step {step}: a drawing needs a new walk")
            }
            journey::Unwalkable::Moving { step } => {
                format!("{name} is unwalkable at step {step}: a key mark caught a moving page")
            }
        },
        Refusal::NotStill { name, step } => format!("{name} at step {step}: page not still within 30 s"),
        Refusal::NoControl { name, step } => format!(
            "{name} at step {step}: no control gives the recorded event within 10 s of the page being still; walk it anew"
        ),
        Refusal::WalkDiffers { name, step } => format!("{name} at step {step}: the new walk differs; walk it anew"),
        Refusal::Save { name, step, refusal: why } => match why {
            journey::Refusal::Stale(stale) => stale_refusal_message(&name, &stale),
            why @ (journey::Refusal::Unreadable(_)
            | journey::Refusal::Unproved { .. }
            | journey::Refusal::Differs { .. }) => {
                format!("at step {step}: {}", refusal(Refusal::Proof { name, refusal: why }))
            }
        },
        Refusal::Harness { name, step, report } => format!("{name} at step {step}: browser or box failed: {report}"),
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
