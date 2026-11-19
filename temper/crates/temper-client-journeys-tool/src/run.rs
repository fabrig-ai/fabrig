//! The journeys binary's capture and dispatch (temper/docs/design/testing/tools.md, 3–4).
use crate::again::{self, Paths};
use crate::{
    EnvironmentEntry, TABLE,
    table::{
        ADD, AGAIN, CAUTION, CHROMIUM, DRIVE, NORMALISE, PAGES, RECIPE_HASH, RECIPE_LIST, RECIPE_WALK, REPRINT, TARGET,
    },
};
use crate::{JOURNEYS, Refusal, TUI_JOURNEYS, add_to, named, reprint, tui, tui_again};
use skein_shell::{Front, Table};
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
            } else if *command == PAGES {
                let prepared = match rest {
                    [] => None,
                    [flag, directory] if flag.as_ref() == b"--prepared" => {
                        Some(Path::new(OsStr::from_bytes(directory)))
                    }
                    _ => usage(&PAGES, "expected [--prepared DIR]"),
                };
                return match pages(prepared, &environment) {
                    Ok(false) => ExitCode::SUCCESS,
                    Ok(true) => ExitCode::FAILURE,
                    Err(why) => {
                        eprintln!("temper-client-journeys: {why}");
                        ExitCode::from(3)
                    }
                };
            } else if *command == AGAIN {
                let (prepared, name) = match rest {
                    [name] => (None, name),
                    [flag, directory, name] if flag.as_ref() == b"--prepared" => {
                        (Some(Path::new(OsStr::from_bytes(directory))), name)
                    }
                    _ => usage(
                        &AGAIN,
                        if rest.is_empty() { "NAME is not given" } else { "expected [--prepared DIR] NAME" },
                    ),
                };
                let Ok(name) = std::str::from_utf8(name) else {
                    fail(&format!("invalid journey name {}", OsStr::from_bytes(name).display()));
                };
                record_again(name, prepared, &environment).map(|()| false)
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

fn record_again(name: &str, prepared: Option<&Path>, environment: &[EnvironmentEntry]) -> Result<(), String> {
    let is_tui = Path::new(TUI_JOURNEYS).join(name).join("walk").exists();
    let steps = starting(name, is_tui)?;
    record_walk(name, steps, prepared, environment)
}

fn starting(name: &str, is_tui: bool) -> Result<Option<tui_again::Starting>, String> {
    if !named(name) {
        return Err(refusal(Refusal::Name(name.to_owned())));
    }
    // Unwalkable journeys are refused before starting the builds or either child.
    if is_tui {
        tui_again::starting(Path::new(TUI_JOURNEYS), Path::new(JOURNEYS), name).map(Some).map_err(refusal)
    } else {
        let _steps = again::starting(Path::new(JOURNEYS), name).map_err(refusal)?;
        let _setup = again::setup(Path::new(JOURNEYS), name).map_err(refusal)?;
        Ok(None)
    }
}

fn record_walk(
    name: &str,
    tui_steps: Option<tui_again::Starting>,
    prepared: Option<&Path>,
    environment: &[EnvironmentEntry],
) -> Result<(), String> {
    let is_tui = tui_steps.is_some();
    let chromium = chromium(name, environment)?;
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let base = scratch_base(prepared);
    let scratch = scratch(&base).map_err(|error| format!("{name}: cannot make a fresh scratch directory: {error}"))?;
    let paths = match prepared {
        Some(directory) => prepared_paths(directory, &scratch, chromium, is_tui),
        None => build(&root, &scratch, chromium, is_tui, environment, false),
    };
    let result = paths.and_then(|paths| {
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

fn chromium(name: &str, environment: &[EnvironmentEntry]) -> Result<PathBuf, String> {
    let chromium = CHROMIUM
        .value(environment)
        .map(OsStr::from_bytes)
        .ok_or_else(|| format!("{name}: set SKEIN_TEST_CHROMIUM to the Chromium binary"))?;
    fs::canonicalize(chromium)
        .map_err(|error| format!("{name}: cannot open Chromium {}: {error}", Path::new(&chromium).display()))
}

/// Build once if a walk needs inputs; each `again` still gets its own fresh scratch.
fn pages(prepared: Option<&Path>, environment: &[EnvironmentEntry]) -> Result<bool, String> {
    use crate::pages::{self, Client};
    let web = Path::new(JOURNEYS);
    let tui = Path::new(TUI_JOURNEYS);
    let stale = pages::stale(web, tui).map_err(refusal)?;
    let needs_tui = stale.iter().any(|journey| journey.client == Client::Tui);
    let mut inputs: Option<Result<Paths, String>> = None;
    let result = pages::write(
        web,
        tui,
        &stale,
        |journey| {
            let steps = starting(&journey.name, journey.client == Client::Tui)?;
            let directory = if let Some(directory) = prepared {
                directory
            } else {
                let paths = inputs.get_or_insert_with(|| page_inputs(needs_tui, environment));
                match paths {
                    Ok(paths) => &paths.scratch,
                    Err(why) => return Err(why.clone()),
                }
            };
            record_walk(&journey.name, steps, Some(directory), environment)
        },
        |line| println!("{line}"),
    );
    if let Some(Ok(paths)) = inputs
        && result == Ok(false)
    {
        fs::remove_dir_all(&paths.scratch)
            .map_err(|error| format!("cannot remove inputs {}: {error}", paths.scratch.display()))?;
    }
    result
}

fn page_inputs(is_tui: bool, environment: &[EnvironmentEntry]) -> Result<Paths, String> {
    let chromium = chromium("pages", environment)?;
    let scratch = scratch(&scratch_base(None)).map_err(|error| format!("cannot make inputs directory: {error}"))?;
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    build(&root, &scratch, chromium, is_tui, environment, true)
        .map_err(|why| format!("{why}; scratch directory kept at {}", scratch.display()))
}

fn scratch_base(prepared: Option<&Path>) -> PathBuf {
    prepared.map_or_else(std::env::temp_dir, Path::to_path_buf)
}

fn scratch(base: &Path) -> io::Result<PathBuf> {
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

fn prepared_paths(directory: &Path, scratch: &Path, chromium: PathBuf, is_tui: bool) -> Result<Paths, String> {
    let directory = fs::canonicalize(directory).map_err(|error| format!("cannot open prepared directory: {error}"))?;
    for name in ["temper-box", "fake-claude"].into_iter().chain(is_tui.then_some("temper")) {
        fs::copy(directory.join(name), scratch.join(name))
            .map_err(|error| format!("cannot copy prepared {name}: {error}"))?;
    }
    let bundle = directory.join("bundle");
    if !bundle.join("index.html").is_file() {
        return Err("prepared bundle has no index.html".into());
    }
    let cache = directory.join("cache");
    let cache = cache.is_dir().then_some(cache);
    Ok(Paths { temper: scratch.join("temper-box"), bundle, cache, chromium, scratch: scratch.to_owned() })
}

fn build(
    root: &Path,
    scratch: &Path,
    chromium: PathBuf,
    is_tui: bool,
    environment: &[EnvironmentEntry],
    writer: bool,
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
        cargo.args(["-p", "temper"]);
    }
    if writer {
        cargo.stdout(io::stderr());
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
    if writer {
        bundler.stdout(io::stderr());
    }
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
        fs::copy(target.join("debug/temper"), scratch.join("temper"))
            .map_err(|error| format!("cannot copy temper into {}: {error}", scratch.display()))?;
    }
    Ok(Paths { temper: scratch.join("temper-box"), bundle, cache: None, chromium, scratch: scratch.to_owned() })
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
    reprint_roots(Path::new(JOURNEYS), Path::new(TUI_JOURNEYS))
}

pub(crate) fn reprint_roots(web: &Path, tui: &Path) -> Result<bool, String> {
    let stale = reprint(web, web).map_err(refusal)?;
    let tui_stale = tui::reprint(tui, tui).map_err(refusal)?;
    let mut names = Vec::new();
    for journeys in [web, tui] {
        for entry in fs::read_dir(journeys).map_err(|_| format!("cannot read {}", journeys.display()))? {
            let entry = entry.map_err(|_| format!("cannot read {}", journeys.display()))?;
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

pub(crate) fn refusal(why: Refusal) -> String {
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
        Refusal::PageState { why, address, text } => format!(
            "{}\ncurrent address: {}\ncurrent page:\n{}",
            refusal(*why),
            String::from_utf8_lossy(&address),
            String::from_utf8_lossy(&text)
        ),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scratch_base_uses_the_prepared_or_platform_temporary_directory() {
        assert_eq!(scratch_base(None), std::env::temp_dir());
        assert_eq!(scratch_base(Some(Path::new("prepared"))), Path::new("prepared"));
    }
}
