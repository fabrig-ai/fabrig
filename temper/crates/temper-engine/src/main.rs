//! The engine's entry point (temper/docs/design/protocol/README.md, 4.2).
mod commands;

use commands::ENGINE;
use skein_io::kernel::Exit;
use std::{env, os::unix::ffi::OsStrExt, process::ExitCode};
use temper_engine_shell::{forge, serve};
use temper_service::startup::{Shape, nearest};
use temper_shell::{
    commands::Row,
    directories::{self, Directories},
    help,
};

#[expect(
    clippy::disallowed_methods,
    reason = "cli C2: arguments select the engine command; temper-engine's table will read them"
)]
fn main() -> ExitCode {
    skein_fail_stop::install();
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
    reason = "cli C2: the process environment resolves directories and forge commands; temper-engine's table will pass it"
)]
fn run(arguments: &[Box<[u8]>]) -> Result<u8, String> {
    if arguments == [Box::from(b"--version".as_slice())] {
        println!("{}", temper_version::line("temper-engine"));
        return Ok(0);
    }
    if arguments == [Box::from(b"--help".as_slice())] {
        print!("{}", ENGINE.help());
        return Ok(0);
    }
    let row = ENGINE.find(arguments).ok_or_else(|| ENGINE.unknown(arguments))?;
    let arguments = &arguments[row.words.len()..];
    if arguments == [Box::from(b"--help".as_slice())] {
        print!("{}", ENGINE.row(row));
        return Ok(0);
    }
    let invalid = || ENGINE.refused(row, "invalid arguments");
    match row.words {
        ["help"] => {
            let request = help::request(arguments).ok_or_else(invalid)?;
            print!("{}", help::answer(ENGINE.binary, &request)?);
            Ok(0)
        }
        ["versions"] if arguments.is_empty() => {
            versions();
            Ok(0)
        }
        ["serve"] => {
            let working = env::current_dir().map_err(|why| ENGINE.refused(row, &why.to_string()))?;
            let (directories, rest) =
                directories::resolve(arguments, &skein_shell::environment(), working.as_os_str().as_bytes())
                    .map_err(|why| ENGINE.refused(row, &why.to_string()))?;
            if !rest.is_empty() {
                return Err(invalid());
            }
            refuse_document(row, &directories)?;
            Ok(match serve::run(ENGINE.binary, &directories, false, None)? {
                Exit::Code(code) => code,
                Exit::Signal(_) => 1,
            })
        }
        ["forge", _] | ["internal", "forge-child"] => {
            let working = env::current_dir().map_err(|why| ENGINE.refused(row, &why.to_string()))?;
            let command = forge::configure(row, arguments, &skein_shell::environment(), working.as_os_str().as_bytes())
                .map_err(|why| ENGINE.refused(row, &why))?;
            match &command {
                forge::Command::Forge { directories, .. } => {
                    if forge::reads_document(row) {
                        refuse_document(row, directories)?;
                    }
                }
                forge::Command::ForgeChild { .. } => {}
            }
            forge::command(ENGINE.binary, command)
        }
        _ => Err(invalid()),
    }
}

/// Refuse another binary's nearest shape before the service's document checks (4.2 and 15.1).
fn refuse_document(row: &Row, directories: &Directories) -> Result<(), String> {
    let Ok(bytes) = directories.read() else { return Ok(()) };
    let why = match nearest(&bytes) {
        Some(Shape::WorkerAlone) => "a worker alone; temper-worker serve runs it",
        Some(Shape::Standalone) => "an engine and its worker in one process; the box's temper serve runs it",
        Some(Shape::EngineAlone) | None => return Ok(()),
    };
    Err(format!("{} {}: temper.json holds {why}\n", ENGINE.binary, row.words.join(" ")))
}

/// The store's patch levels and versions, then its worker wires (4.2; jig's protocol/store/records.md, 3).
fn versions() {
    let levels = jig_protocol_store::levels(&jig_protocol_store::PatchSets::NONE);
    for (owner, level) in
        [("skein", levels.skein), ("smith", levels.smith), ("jig", levels.jig), ("temper", levels.application)]
    {
        match level {
            Some(level) => println!("level {owner} {level}"),
            None => println!("level {owner} none"),
        }
    }
    let format = jig_protocol_store::format::current(temper_protocol_store::STORE_VERSION, [0; 16]);
    println!("store {}", format.component);
    for (family, version) in [
        ("core", u32::from(format.core)),
        ("tasks", u32::from(format.tasks)),
        ("people", u32::from(format.people)),
        ("notes", u32::from(format.notes)),
        ("accounts", u32::from(format.accounts)),
        ("settings", u32::from(format.settings)),
        ("temper", format.application),
    ] {
        println!("family {family} {version}");
    }
    for wire in temper_service::store::WIRES {
        println!("wire {} {}", wire.name, wire.version);
    }
}
