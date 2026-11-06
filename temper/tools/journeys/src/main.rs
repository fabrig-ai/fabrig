use std::{
    env,
    ffi::OsString,
    fs, io,
    os::unix::ffi::OsStrExt,
    os::unix::fs::DirBuilderExt,
    path::{Path, PathBuf},
    process::{self, Command},
};
use temper_client_journeys::again::{self, Paths};
use temper_client_journeys::{JOURNEYS, Refusal, add, named, reprint};
use temper_client_shell_web::journey;

fn main() {
    skein_fail_stop::install();
    let mut args = env::args_os();
    let _program = args.next();
    let arguments: Vec<_> = args.collect();
    let result = match arguments.as_slice() {
        [] => reprint_all(),
        [command] if command == "--help" => {
            println!("{USAGE}\n{HELP}\n{CAUTION}\n{FILE_CAUTION}");
            Ok(false)
        }
        [command, rest @ ..] if command == "add" => add_arguments(rest),
        [command, name] if command == "again" => {
            let Some(name) = name.to_str() else {
                fail(&format!("invalid journey name {}", name.display()));
            };
            record_again(name).map(|()| false)
        }
        _ => usage(),
    };
    match result {
        Ok(false) => {}
        Ok(true) => process::exit(1),
        Err(why) => fail(&why),
    }
}

const USAGE: &str = "usage: temper-client-journeys [add [--redact TEXT]... NAME FILE | again NAME | --help]";
const CAUTION: &str = "These aren't all a secret could be: any other private text stands unless --redact names it, so read the pages before you commit them.";
const FILE_CAUTION: &str = "FILE keeps every original: keep it out of the repository and remove it once added.";
const HELP: &str = concat!(
    "add takes out the link secret, stored credentials, typed secrets and each --redact TEXT, replacing their bytes with x's. ",
    "It refuses surviving originals and shapes of a key, JWT, bearer, code or refresh token in records, drawn pages or printed pages. ",
    "It proves the redacted replay before writing.\n",
    "A secret split across run deliveries or writing-call input is refused; --redact cannot replace it, so walk it again. ",
    "Text split across other records is seen only where a page draws it whole. Partial secrets and other encodings may remain unseen. ",
    "Values the client reshapes can stale the replay. Reload a settled page: stored requests in flight refuse. ",
    "--redact TEXT remains in shell history."
);

fn usage() -> ! {
    eprintln!("{USAGE}");
    process::exit(2);
}

fn add_arguments(arguments: &[OsString]) -> Result<bool, String> {
    let mut positional = Vec::new();
    let mut given = Vec::new();
    let mut args = arguments.iter();
    while let Some(argument) = args.next() {
        if argument == "--redact" {
            let Some(text) = args.next() else { usage() };
            let bytes = text.as_bytes();
            if bytes.is_empty() || bytes.starts_with(b"--") {
                usage();
            }
            given.push(bytes.into());
        } else if argument.as_bytes().starts_with(b"--") {
            usage();
        } else {
            positional.push(argument);
        }
    }
    let [name, file] = positional.as_slice() else { usage() };
    let Some(name) = name.to_str() else {
        return Err("invalid journey name; expected [a-z][a-z0-9-]*; nothing was written".into());
    };
    if !named(name) {
        return Err(format!("{}; nothing was written", refusal(Refusal::Name(name.to_owned()))));
    }
    let recording = fs::read(file).map_err(|_| "cannot read FILE; nothing was written".to_owned())?;
    let added = add(Path::new(JOURNEYS), name, &recording, &given)
        .map_err(|why| format!("{}; nothing was written", refusal(why)))?;
    println!(
        "link: {}; stored: {}; typed: {}; given: {}; places replaced: {}",
        added.link, added.stored, added.typed, added.given, added.places
    );
    println!("{CAUTION}");
    Ok(false)
}

fn record_again(name: &str) -> Result<(), String> {
    // Unwalkable journeys are refused before starting the builds or either child.
    let _steps = again::starting(Path::new(JOURNEYS), name).map_err(refusal)?;
    let _setup = again::setup(Path::new(JOURNEYS), name).map_err(refusal)?;
    let chromium = env::var_os("SKEIN_TEST_CHROMIUM")
        .ok_or_else(|| format!("{name}: set SKEIN_TEST_CHROMIUM to the Chromium binary"))?;
    let chromium = fs::canonicalize(&chromium)
        .map_err(|error| format!("{name}: cannot open Chromium {}: {error}", Path::new(&chromium).display()))?;
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let scratch = scratch().map_err(|error| format!("{name}: cannot make a fresh scratch directory: {error}"))?;
    let result = build(&root, &scratch, chromium)
        .and_then(|paths| again::again(Path::new(JOURNEYS), name, &paths).map_err(refusal));
    match result {
        Ok(()) => fs::remove_dir_all(&scratch)
            .map_err(|error| format!("{name}: cannot remove scratch directory {}: {error}", scratch.display())),
        Err(why) => Err(format!("{name}: {why}\nscratch directory kept at {}", scratch.display())),
    }
}

fn scratch() -> io::Result<PathBuf> {
    let base = Path::new("/tmp");
    for attempt in 0..1000 {
        let path = base.join(format!("temper-journey-{}-{attempt}", process::id()));
        match fs::DirBuilder::new().mode(0o700).create(&path) {
            Ok(()) => return Ok(path),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error),
        }
    }
    Err(io::Error::new(io::ErrorKind::AlreadyExists, "no fresh journey directory name"))
}

fn build(root: &Path, scratch: &Path, chromium: PathBuf) -> Result<Paths, String> {
    let status = Command::new("cargo")
        .current_dir(root)
        .env("TEMPER_BUNDLE", "none")
        .args(["build", "-p", "temper", "-p", "smith-fake-claude"])
        .status()
        .map_err(|error| format!("cannot start cargo build: {error}"))?;
    if !status.success() {
        return Err(format!("cargo build failed: {status}"));
    }
    let bundle = scratch.join("bundle");
    let status = Command::new(root.join("scripts/bundle.sh"))
        .current_dir(root)
        .env("TEMPER_BUNDLE", "none")
        .args(["--loopback", "--record"])
        .arg(&bundle)
        .status()
        .map_err(|error| format!("cannot start scripts/bundle.sh: {error}"))?;
    if !status.success() {
        return Err(format!("scripts/bundle.sh failed: {status}"));
    }
    let target = env::var_os("CARGO_TARGET_DIR").map_or_else(|| root.join("../target"), |target| root.join(target));
    for name in ["temper", "fake-claude"] {
        fs::copy(target.join("debug").join(name), scratch.join(name))
            .map_err(|error| format!("cannot copy {name} into {}: {error}", scratch.display()))?;
    }
    Ok(Paths { temper: scratch.join("temper"), bundle, chromium, scratch: scratch.to_owned() })
}

fn reprint_all() -> Result<bool, String> {
    let stale = reprint(Path::new(JOURNEYS), Path::new(JOURNEYS)).map_err(refusal)?;
    let mut names = Vec::new();
    for entry in fs::read_dir(JOURNEYS).map_err(|_| format!("cannot read {JOURNEYS}"))? {
        let entry = entry.map_err(|_| format!("cannot read {JOURNEYS}"))?;
        let path = entry.path();
        if entry.file_type().map_err(|_| format!("cannot read {}", path.display()))?.is_dir()
            && path.join("recording").is_file()
        {
            names.push(entry.file_name());
        }
    }
    names.sort();
    for name in names {
        println!("{}", name.to_string_lossy());
    }
    for (name, stopped) in &stale {
        eprintln!("temper-client-journeys: {}", stale_message(name, stopped));
    }
    Ok(!stale.is_empty())
}

fn refusal(why: Refusal) -> String {
    match why {
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
// the recording (client/journeys.md, 4.3). Reprint keeps the full message below.
fn stale_refusal_message(name: &str, stale: &journey::Stale) -> String {
    let recorded = stale.recorded.split_whitespace().next().unwrap_or("");
    let asked = stale.asked.split_whitespace().next().unwrap_or("");
    format!(
        "{name} is stale at turn {}, line {}: recorded {recorded}, asked {asked}; record it again (client/journeys.md, 8)",
        stale.turn, stale.line
    )
}

fn stale_message(name: &str, stale: &journey::Stale) -> String {
    format!(
        "{name} is stale at turn {}, line {}: recorded {}, asked {}; record it again (client/journeys.md, 8)",
        stale.turn, stale.line, stale.recorded, stale.asked
    )
}

fn fail(why: &str) -> ! {
    eprintln!("temper-client-journeys: {why}");
    process::exit(1);
}
