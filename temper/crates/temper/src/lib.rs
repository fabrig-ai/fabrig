//! Startup for temper's standalone binary (protocol/configuration.md, section 2).
// tests::configuration includes the shared integration fixture, which names temper::directories.
extern crate self as temper;
use skein_io::kernel::Exit;
#[cfg(test)]
use skein_io::kernel::Fd;
use skein_shell::Clock;
use std::{
    ffi::OsString,
    fmt::Write,
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
};
use temper_service as service;

pub mod directories;
use directories::{Directories, private_root};

#[expect(
    clippy::disallowed_methods,
    reason = "cli C2: TEMPER_WORKSPACE_REUSE selects worker reuse; temper-box's table will pass it"
)]
pub(crate) fn workspace_reuse() -> bool {
    std::env::var_os("TEMPER_WORKSPACE_REUSE").is_some()
}
/// Where a worker's agents keep each model request (temper/docs/design/draft/dogfood-hacks.md, 2.13).
#[expect(
    clippy::disallowed_methods,
    reason = "cli C2: TEMPER_DUMP_LLM selects the agent request dump directory; temper-box's table will pass it"
)]
pub(crate) fn dump_llm() -> Option<PathBuf> {
    dump_directory(std::env::var_os("TEMPER_DUMP_LLM"))
}
fn dump_directory(value: Option<OsString>) -> Option<PathBuf> {
    let value = value?;
    if value.is_empty() {
        return None;
    }
    match std::path::absolute(&value) {
        Ok(directory) => Some(directory),
        Err(why) => {
            use std::io::Write;
            drop(writeln!(std::io::stderr(), "temper serve: TEMPER_DUMP_LLM: {why}"));
            None
        }
    }
}
const DUMP_BOUND: u64 = 4_294_967_296;
mod agent;
pub mod box_command;
pub mod claude_code;
pub mod commands;
pub mod ctl;
pub mod fake;
mod fake_backends;
use fake_backends::json;
pub use temper_engine_shell::forge;
use temper_engine_shell::{address, forge_sign_in, login_passwd};
use temper_forge_setup_shell::process as forge_exec;
pub mod help;
mod init;
pub use temper_shell::link_file;
pub mod open;
mod program_login;
pub use temper_engine_shell::setup;
mod tui;
mod tui_front;
mod worker_alone;
mod worker_half;
use json::CONFIG_BYTES;
#[cfg(test)]
pub(crate) use serve::host;
use temper_shell::directories::{errno, unread};

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Usage(&'static commands::Row),
    Init { directories: Directories, boxed: bool },
    Serve { directories: Directories, boxed: bool },
    Box { directories: Directories, echo: temper_box::host::Echo, no_tui: bool },
    BoxFakeBackends { directories: Directories, bundle: Option<PathBuf>, echo: temper_box::host::Echo, no_tui: bool },
    ProgramLogin { directories: Directories, name: OsString },
    Agent { configuration: PathBuf },
    AgentList { configuration: PathBuf },
    Tui { address: Option<Box<[u8]>> },
    TuiFront { address: Box<[u8]> },
    Open { arguments: Vec<OsString> },
    Ctl { arguments: Vec<OsString> },
    Help { request: help::Request },
    Fake { arguments: fake::Arguments },
    FakeCheck { directories: Directories, check: fake::Check },
    FakeGit { arguments: Box<[Box<[u8]>]> },
    ForgeChild { parent: u32, signal: skein_io::kernel::Signal, program: OsString, arguments: Vec<OsString> },
    Forge { directories: Directories, arguments: Vec<OsString>, ssl_cert_file: Option<PathBuf> },
}
fn prefixed(command: &str, why: &str) -> String {
    let mut lines = String::new();
    for line in why.lines() {
        writeln!(lines, "{command}: {line}").expect("writing into a string");
    }
    lines
}
#[expect(
    clippy::disallowed_methods,
    reason = "cli C2: the process environment resolves standalone commands; temper-box's table will pass it"
)]
pub fn configure(args: impl Iterator<Item = OsString>) -> Result<Command, String> {
    let arguments: Vec<Box<[u8]>> = args.map(|arg| Box::from(arg.as_bytes())).collect();
    let working = std::env::current_dir().map_err(|why| why.to_string())?;
    configure_with(&arguments, &skein_shell::environment(), working.as_os_str().as_bytes())
}
type EnvironmentEntry = (Box<[u8]>, Box<[u8]>);
pub fn configure_with(
    arguments: &[Box<[u8]>],
    environment: &[EnvironmentEntry],
    working: &[u8],
) -> Result<Command, String> {
    // smith's host half prefixes list launches with its selector.
    let arguments = match arguments {
        [flag, internal, list, ..]
            if flag.as_ref() == b"--list" && internal.as_ref() == b"internal" && list.as_ref() == b"agent-list" =>
        {
            &arguments[1..]
        }
        _ => arguments,
    };
    let row = commands::find(arguments).ok_or_else(|| commands::unknown(arguments))?;
    let arguments = &arguments[row.words.len()..];
    if arguments.len() == 1 && arguments[0].as_ref() == b"--help" {
        return Ok(Command::Usage(row));
    }
    configure_row(row, arguments, environment, working).map_err(|why| commands::refused(row, &why))
}
fn configure_row(
    row: &commands::Row,
    arguments: &[Box<[u8]>],
    environment: &[EnvironmentEntry],
    working: &[u8],
) -> Result<Command, String> {
    let invalid = || "invalid arguments".to_string();
    let os =
        |args: &[Box<[u8]>]| args.iter().map(|arg| std::ffi::OsStr::from_bytes(arg).to_owned()).collect::<Vec<_>>();
    match row.words {
        [] => {
            return match arguments {
                [] => Ok(Command::Tui { address: None }),
                [address] => Ok(Command::Tui { address: Some(address.clone()) }),
                _ => Err(invalid()),
            };
        }
        ["fake", ..] => return fake::configure(row, arguments, environment, working),
        ["ctl"] => return Ok(Command::Ctl { arguments: os(arguments) }),
        ["help"] => return help::request(arguments).map(|request| Command::Help { request }).ok_or_else(invalid),
        ["open"] => return Ok(Command::Open { arguments: os(arguments) }),
        ["internal", "agent"] => {
            return match arguments {
                [configuration] => Ok(Command::Agent { configuration: path(configuration).into() }),
                _ => Err(invalid()),
            };
        }
        ["internal", "agent-list"] => {
            return match arguments {
                [configuration] => Ok(Command::AgentList { configuration: path(configuration).into() }),
                _ => Err(invalid()),
            };
        }
        ["internal", "tui"] => {
            return match arguments {
                [flag, address] if flag.as_ref() == b"--box" => Ok(Command::TuiFront { address: address.clone() }),
                _ => Err(invalid()),
            };
        }
        ["forge", _] | ["internal", "forge-child"] => {
            return temper_engine_shell::forge::configure(row, arguments, environment, working).map(|command| {
                match command {
                    forge::Command::Forge { directories, arguments, ssl_cert_file } => {
                        Command::Forge { directories, arguments, ssl_cert_file }
                    }
                    forge::Command::ForgeChild { parent, signal, program, arguments } => {
                        Command::ForgeChild { parent, signal, program, arguments }
                    }
                }
            });
        }
        _ => {}
    }
    let (directories, rest) = directories::resolve(arguments, environment, working).map_err(|why| why.to_string())?;
    let no_tui = row.words == ["box"] && rest.iter().any(|word| word.as_ref() == b"--no-tui");
    let rest = if no_tui {
        if rest.iter().filter(|word| word.as_ref() == b"--no-tui").count() != 1 {
            return Err(invalid());
        }
        rest.into_vec().into_iter().filter(|word| word.as_ref() != b"--no-tui").collect()
    } else {
        rest
    };
    let (echo, rest) = if row.words == ["box"] {
        if rest.iter().any(|word| word.as_ref() == b"--fake-backends") {
            let (echo, _) = box_command::echo(&[], environment)?;
            (echo, rest)
        } else {
            box_command::echo(&rest, environment)?
        }
    } else {
        (temper_box::host::Echo::None, rest)
    };
    let rest = os(&rest);
    match row.words {
        ["init"] if rest.is_empty() => Ok(Command::Init { directories, boxed: false }),
        ["init"] if rest == ["--box"] => Ok(Command::Init { directories, boxed: true }),
        ["serve"] if rest.is_empty() => Ok(Command::Serve { directories, boxed: false }),
        ["internal", "serve"] if rest == ["--box"] => Ok(Command::Serve { directories, boxed: true }),
        ["box"] if rest.is_empty() => Ok(Command::Box { directories, echo, no_tui }),
        ["box"] => {
            let bundle = fake_backends::configure(&rest, working)?;
            Ok(Command::BoxFakeBackends { directories, bundle, echo, no_tui })
        }
        ["program-login"] if rest.len() == 1 => Ok(Command::ProgramLogin { directories, name: rest[0].clone() }),
        _ => Err(invalid()),
    }
}

pub fn run(command: Command) -> Result<u8, String> {
    let exit = match command {
        Command::Usage(row) => {
            print!("{}", commands::row(row));
            return Ok(0);
        }
        Command::Init { directories, boxed } => {
            return init::run(&directories, boxed).map_err(|why| prefixed("temper init", &why));
        }
        Command::Serve { directories, boxed } => serve::run("temper", &directories, boxed, Some(&worker_half::Half))?,
        Command::BoxFakeBackends { directories, bundle, echo, no_tui } => {
            fake_backends::run(&directories, bundle.as_deref(), echo, no_tui)
                .map_err(|why| prefixed("temper box", &why))?
        }
        Command::Box { directories, echo, no_tui } => {
            box_command::run(&directories, echo, no_tui).map_err(|why| prefixed("temper box", &why))?
        }
        Command::Agent { configuration } => agent::decoded(&configuration)
            .inspect_err(|why| {
                use std::io::Write;
                drop(writeln!(std::io::stderr(), "smith: {why}"));
            })
            .and_then(|decoded| match dump_llm() {
                Some(directory) => smith_agent_shell::run_dumping(
                    decoded,
                    smith_agent_shell::dump::Dumps::new(directory, DUMP_BOUND),
                    Box::new(std::io::stderr()),
                ),
                None => smith_agent_shell::run_decoded(decoded, Box::new(std::io::stderr())),
            })
            .map_err(|why| prefixed("temper internal agent", &why))?,
        Command::AgentList { configuration } => agent::decoded(&configuration)
            .and_then(|decoded| smith_agent_shell::list::run_decoded(decoded, Box::new(std::io::stderr())))
            .map_err(|why| prefixed("temper internal agent-list", &why))?,
        Command::ProgramLogin { directories, name } => {
            return program_login::run(&directories, &name)
                .map_err(|why| prefixed("temper program-login", &why.lines().collect::<Vec<_>>().join("; ")));
        }
        Command::Fake { arguments } => return fake::run(&arguments),
        Command::FakeCheck { directories, check } => return Ok(fake::check::run(&directories, &check)),
        Command::FakeGit { arguments } => return Ok(fake::git::run(&arguments)),
        Command::ForgeChild { parent, signal, program, arguments } => {
            return temper_engine_shell::forge::command(
                "temper",
                forge::Command::ForgeChild { parent, signal, program, arguments },
            );
        }
        Command::Forge { directories, arguments, ssl_cert_file } => {
            return temper_engine_shell::forge::command(
                "temper",
                forge::Command::Forge { directories, arguments, ssl_cert_file },
            );
        }
        Command::Help { request } => {
            print!("{}", help::answer(&request)?);
            return Ok(0);
        }
        Command::Ctl { arguments } => return Ok(ctl::run(&arguments, &Clock::new())),
        Command::Tui { address } => return Ok(tui::run(address, &Clock::new())),
        Command::TuiFront { address } => return Ok(tui_front::run(&address, &Clock::new())),
        Command::Open { arguments } => return Ok(open::run(&arguments, &Clock::new())),
    };
    Ok(match exit {
        Exit::Code(code) => code,
        Exit::Signal(_) => 1,
    })
}
fn path(bytes: &[u8]) -> &Path {
    Path::new(std::ffi::OsStr::from_bytes(bytes))
}
pub use serve::host::{Host, ending as service_ending};
pub(crate) use serve::{close, file_path, held_refusal, listeners_refusal, read_files};
use temper_engine_shell::serve;
pub use temper_engine_shell::serve::{bundle, github_logs, printed};
#[cfg(test)]
fn prepare(directories: &Directories, bytes: &[u8], owned: &mut Vec<Fd>) -> Result<service::Service, String> {
    serve::prepare("temper", Some(&worker_half::Half), directories, bytes, owned)
}
#[cfg(test)]
fn prepare_with_store(
    directories: &Directories,
    bytes: &[u8],
    owned: &mut Vec<Fd>,
) -> Result<(service::Service, Box<[u8]>), String> {
    serve::prepare_with_store("temper", Some(&worker_half::Half), directories, bytes, owned)
}
/// The kernel operations a standalone start keeps in flight at most (its ring's size),
/// derived without retaining its descriptors.
pub fn operations_in(directories: &Directories, config: &[u8]) -> Result<u32, String> {
    serve::operations_in("temper", Some(&worker_half::Half), directories, config)
}
/// The same for a boxed start.
pub fn boxed_operations_in(directories: &Directories, config: &[u8]) -> Result<u32, String> {
    serve::boxed_operations_in("temper", Some(&worker_half::Half), directories, config)
}
pub fn prepare_in(directories: &Directories) -> Result<service::Service, String> {
    serve::prepare_in("temper", Some(&worker_half::Half), directories)
}
/// Prepare an in-process world's service with the supplied defaults.
pub fn prepare_in_with(directories: &Directories, given: &service::stock::Given) -> Result<service::Service, String> {
    serve::prepare_in_with("temper", Some(&worker_half::Half), directories, given)
}
/// The same for an in-process world's service, with the supplied defaults.
pub fn operations_in_with(
    directories: &Directories,
    config: &[u8],
    given: &service::stock::Given,
) -> Result<u32, String> {
    serve::operations_in_with("temper", Some(&worker_half::Half), directories, config, given)
}
/// The effective user's login (protocol/setup.md, 8.2); macOS users need not appear in passwd.
pub(crate) fn login(user: u32) -> Result<Box<[u8]>, String> {
    if cfg!(target_os = "macos") { forge_exec::login_id() } else { login_passwd(user) }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod box_tests {
    #[test]
    fn serve_box_flag_is_carried_to_the_command() {
        for boxed in [false, true] {
            let mut args = ["serve", "--config", "/c", "--data", "/d", "--cache", "/k"]
                .iter()
                .map(|word| Box::from(word.as_bytes()))
                .collect::<Vec<_>>();
            if boxed {
                args.insert(0, Box::from(b"internal".as_slice()));
                args.push(Box::from(b"--box".as_slice()));
            }
            let super::Command::Serve { boxed: found, .. } =
                super::configure_with(&args, &[], b"/").expect("serve arguments")
            else {
                panic!("serve command");
            };
            assert_eq!(found, boxed);
        }
    }

    #[test]
    fn serve_refuses_unknown_or_repeated_flags() {
        for flags in [&["--other"][..], &["--box", "--box"][..]] {
            let args = core::iter::once("serve")
                .chain(flags.iter().copied())
                .chain(["--config", "/c", "--data", "/d", "--cache", "/k"])
                .map(|word| Box::from(word.as_bytes()))
                .collect::<Vec<_>>();
            assert!(super::configure_with(&args, &[], b"/").is_err());
        }
    }
}
