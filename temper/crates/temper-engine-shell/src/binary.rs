//! The engine's capture and dispatch (temper/docs/design/cli/README.md, 3 and 4).
use crate::{ENGINE, forge, serve};
use skein_io::kernel::Exit;
use skein_shell::{Command, Reach, Reached, Table};
use std::{os::unix::ffi::OsStrExt, path::PathBuf, process::ExitCode};
use temper_engine_service::startup::{Shape, nearest};
use temper_shell::{
    commands::Row,
    directories::{self, Directories, EnvironmentEntry},
    table,
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
    match ENGINE.reach(&captured.run_as, &captured.arguments) {
        Reached::Help => {
            print!("{}", ENGINE.help());
            Ok(0)
        }
        Reached::Version => {
            print!("{}", Table { version, commit, ..ENGINE }.version_line());
            Ok(0)
        }
        Reached::CommandHelp(command) => {
            print!("{}", ENGINE.command_help(command));
            Ok(0)
        }
        Reached::Unknown(why) => Err((2, why)),
        Reached::Command(command, arguments) => dispatch(command, arguments, captured),
    }
}

fn dispatch(command: &Command, arguments: &[Box<[u8]>], captured: &Capture) -> Result<u8, (u8, String)> {
    let refused = |why: &str| (2, table::refused(&ENGINE, command, why));
    match command.reach {
        Reach::Words(["help"]) => {
            print!("{}", table::help(&ENGINE, arguments)?);
            Ok(0)
        }
        Reach::Words(["versions"]) => {
            if !arguments.is_empty() {
                return Err(refused("invalid arguments"));
            }
            print!("{}", versions());
            Ok(0)
        }
        Reach::Words(["upgrade", mode @ ("--check" | "--back")]) => {
            let working = captured.working.as_ref().map_err(|why| (1, format!("temper-engine upgrade: {why}\n")))?;
            let (directories, request) = crate::upgrade::configure(
                *mode == "--check",
                arguments,
                &captured.environment,
                working.as_os_str().as_bytes(),
            )
            .map_err(|why| refused(&why))?;
            refuse_document(command, &directories).map_err(|why| (1, why))?;
            crate::upgrade::run(ENGINE.binary, &directories, &request).map_err(|why| (1, why))
        }
        Reach::Words(["serve"]) => {
            let working = captured.working.as_ref().map_err(|why| (1, format!("temper-engine serve: {why}\n")))?;
            let (directories, rest) =
                directories::resolve(arguments, &captured.environment, working.as_os_str().as_bytes())
                    .map_err(|why| refused(&why.to_string()))?;
            let record = recording(&rest).ok_or_else(|| refused("invalid arguments"))?;
            refuse_document(command, &directories).map_err(|why| (1, why))?;
            Ok(
                match serve::run(
                    ENGINE.binary,
                    &captured.environment,
                    working.as_os_str().as_bytes(),
                    &directories,
                    record,
                )
                .map_err(|why| (1, why))?
                {
                    Exit::Code(code) => code,
                    Exit::Signal(_) => 1,
                },
            )
        }
        Reach::Words(words) => {
            let row = match words {
                ["forge", "fetch"] => &forge::FETCH,
                ["forge", "prepare"] => &forge::PREPARE,
                ["forge", "run"] => &forge::RUN,
                ["forge", "remove"] => &forge::REMOVE,
                ["forge", "source"] => &forge::SOURCE,
                ["forge", "sign-in"] => &forge::SIGN_IN,
                ["internal", "forge-child"] => &forge::FORGE_CHILD,
                _ => return Err(refused("invalid arguments")),
            };
            run_forge(command, row, arguments, captured)
        }
        Reach::RunAs(_) => Err(refused("invalid arguments")),
    }
}

fn run_forge(command: &Command, row: &Row, arguments: &[Box<[u8]>], captured: &Capture) -> Result<u8, (u8, String)> {
    let working =
        captured.working.as_ref().map_err(|why| (1, format!("temper-engine {}: {why}\n", row.words.join(" "))))?;
    let configured = forge::configure(row, arguments, &captured.environment, working.as_os_str().as_bytes())
        .map_err(|why| (2, table::refused(&ENGINE, command, &why)))?;
    match &configured {
        forge::Command::Forge { directories, .. } => {
            if forge::reads_document(row) {
                refuse_document(command, directories).map_err(|why| (1, why))?;
            }
        }
        forge::Command::ForgeChild { .. } => {}
    }
    forge::command(ENGINE.binary, configured).map_err(|why| (1, why))
}

/// Refuse another binary's nearest shape before the service's document checks (protocol/binaries.md, 2).
fn refuse_document(command: &Command, directories: &Directories) -> Result<(), String> {
    let Ok(bytes) = directories.read() else { return Ok(()) };
    let why = match nearest(&bytes) {
        Some(Shape::WorkerAlone) => "a worker alone; temper-worker serve runs it",
        Some(Shape::Standalone) => "an engine and its worker in one process; temper-box serve runs it",
        Some(Shape::EngineAlone) | None => return Ok(()),
    };
    let words = match command.reach {
        Reach::Words(words) => words.join(" "),
        Reach::RunAs(name) => name.into(),
    };
    Err(format!("{} {words}: temper.json holds {why}\n", ENGINE.binary))
}

/// The store's patch levels and versions, then its worker wires (protocol/binaries.md, 2),
/// each line ending in a newline.
#[must_use]
pub fn versions() -> String {
    use std::fmt::Write;
    let mut text = String::new();
    let levels = jig_protocol_store::levels(&temper_engine_service::store::PATCH_SETS);
    for (owner, level) in
        [("skein", levels.skein), ("smith", levels.smith), ("jig", levels.jig), ("temper", levels.application)]
    {
        match level {
            Some(level) => writeln!(text, "level {owner} {level}").expect("writing into a string"),
            None => writeln!(text, "level {owner} none").expect("writing into a string"),
        }
    }
    let format = jig_protocol_store::format::current(temper_protocol_store::STORE_VERSION, [0; 16]);
    writeln!(text, "store {}", format.component).expect("writing into a string");
    for (family, version) in [
        ("core", u32::from(format.core)),
        ("tasks", u32::from(format.tasks)),
        ("people", u32::from(format.people)),
        ("notes", u32::from(format.notes)),
        ("accounts", u32::from(format.accounts)),
        ("settings", u32::from(format.settings)),
        ("temper", format.application),
    ] {
        writeln!(text, "family {family} {version}").expect("writing into a string");
    }
    for wire in &temper_engine_service::wires() {
        writeln!(text, "wire {} {} {}", wire.name, wire.lowest, wire.highest()).expect("writing into a string");
        for version in wire.lowest..=wire.highest() {
            write!(text, "layout {} {version} ", wire.name).expect("writing into a string");
            for byte in wire.layout(version).expect("every version has a layout") {
                write!(text, "{byte:02x}").expect("writing into a string");
            }
            text.push('\n');
        }
    }
    text
}

fn recording(arguments: &[Box<[u8]>]) -> Option<bool> {
    match arguments {
        [] => Some(false),
        [flag] if flag.as_ref() == b"--record" => Some(true),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn versions_print_each_wire_range_then_its_layout() {
        use std::fmt::Write;
        let text = super::versions();
        eprintln!("{text}");
        let mut lines = text.lines().skip_while(|line| !line.starts_with("wire "));
        let wires = temper_engine_service::wires();
        assert_eq!(wires.len(), 6);
        for (wire, (name, lowest, highest)) in wires.iter().zip([
            ("jig-worker-wire", 12, 14),
            ("smith-pieces", 1, 1),
            ("smith-charter", 4, 5),
            ("smith-transcript", 7, 7),
            ("temper-worker-wire", 5, 8),
            ("temper-host-tools", 1, 2),
        ]) {
            assert_eq!((wire.name, wire.lowest, wire.highest()), (name, lowest, highest));
            assert_eq!(lines.next(), Some(format!("wire {name} {lowest} {highest}").as_str()));
            for version in lowest..=highest {
                let layout = lines.next().expect("one layout follows for each spoken version");
                let fingerprint =
                    layout.strip_prefix(&format!("layout {name} {version} ")).expect("layout name and version");
                let mut expected = String::new();
                for byte in wire.layout(version).expect("kept layout") {
                    write!(expected, "{byte:02x}").expect("writing into a string");
                }
                assert_eq!(fingerprint, expected);
            }
        }
        assert_eq!(lines.next(), None);
    }

    #[test]
    fn engine_serve_accepts_record_once_only() {
        for (words, expected) in [
            (&[][..], Some(false)),
            (&["--record"][..], Some(true)),
            (&["--record", "--record"][..], None),
            (&["--other"][..], None),
        ] {
            let arguments: Vec<_> = words.iter().map(|word| Box::from(word.as_bytes())).collect();
            assert_eq!(super::recording(&arguments), expected);
        }
        let switch =
            crate::commands::SERVE.switches.iter().find(|switch| switch.name == "record").expect("public switch");
        assert_eq!(switch.set, skein_shell::Set::Public);
        assert_eq!(switch.line, "Have every page served record a journey in its browser, saved by Alt+Shift+S.");
    }
}
