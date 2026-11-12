//! Startup for temper's standalone binary (protocol/configuration/README.md, section 2).
// tests::configuration includes the shared integration fixture, which names temper_box_shell::directories.
extern crate self as temper_box_shell;
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
#[cfg(test)]
use temper_worker_shell::agent;

mod binary;
pub use binary::main;

pub mod directories;
use directories::Directories;

pub mod box_command;
pub mod claude_code;
pub mod table;
pub use table::{TABLE, TREE};
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
mod tui_front;
mod worker_half;
use json::CONFIG_BYTES;
#[cfg(test)]
pub(crate) use temper_service_shell::host;
use temper_shell::directories::{errno, unread};

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Overview,
    Version,
    Versions,
    Usage(&'static skein_shell::Command),
    Claude {
        program: OsString,
        arguments: Box<[Box<[u8]>]>,
    },
    Relay,
    ProgramChild,
    Init {
        directories: Directories,
        boxed: bool,
    },
    Serve {
        directories: Directories,
        boxed: bool,
    },
    Box {
        directories: Directories,
        echo: temper_box_machine::host::Echo,
        no_tui: bool,
    },
    BoxFakeBackends {
        directories: Directories,
        bundle: Option<PathBuf>,
        scripts: Vec<PathBuf>,
        strict: bool,
        echo: temper_box_machine::host::Echo,
        no_tui: bool,
    },
    ProgramLogin {
        directories: Directories,
        name: OsString,
    },
    Agent {
        configuration: PathBuf,
    },
    AgentList {
        configuration: PathBuf,
    },
    TuiFront {
        address: Box<[u8]>,
    },
    Open {
        arguments: Vec<OsString>,
    },
    Ctl {
        arguments: Vec<OsString>,
    },
    Help {
        request: help::Request,
    },
    Fake {
        arguments: fake::Arguments,
    },
    FakeCheck {
        directories: Directories,
        check: fake::Check,
    },
    FakeGit {
        arguments: Box<[Box<[u8]>]>,
    },
    ForgeChild {
        parent: u32,
        signal: skein_io::kernel::Signal,
        program: OsString,
        arguments: Vec<OsString>,
    },
    Forge {
        directories: Directories,
        arguments: Vec<OsString>,
        ssl_cert_file: Option<PathBuf>,
        environment: Box<[EnvironmentEntry]>,
    },
}
fn prefixed(command: &str, why: &str) -> String {
    let mut lines = String::new();
    for line in why.lines() {
        writeln!(lines, "{command}: {line}").expect("writing into a string");
    }
    lines
}
type EnvironmentEntry = (Box<[u8]>, Box<[u8]>);
pub fn configure_with(
    arguments: &[Box<[u8]>],
    environment: &[EnvironmentEntry],
    working: &[u8],
) -> Result<Command, String> {
    configure_as(b"temper-box", arguments, environment, working)
}
fn configure_as(
    run_as: &[u8],
    arguments: &[Box<[u8]>],
    environment: &[EnvironmentEntry],
    working: &[u8],
) -> Result<Command, String> {
    match TABLE.reach(run_as, arguments) {
        skein_shell::Reached::Help => Ok(Command::Overview),
        skein_shell::Reached::Version => Ok(Command::Version),
        skein_shell::Reached::CommandHelp(command) => Ok(Command::Usage(command)),
        skein_shell::Reached::Unknown(why) => Err(why),
        skein_shell::Reached::Command(command, rest) => {
            configure_command(command, rest, arguments, environment, working)
        }
    }
}
fn configure_command(
    command: &skein_shell::Command,
    rest: &[Box<[u8]>],
    arguments: &[Box<[u8]>],
    environment: &[EnvironmentEntry],
    working: &[u8],
) -> Result<Command, String> {
    if command.reach == skein_shell::Reach::Words(&[]) {
        return configure_box(arguments, environment, working);
    }
    configure_row(command, rest, environment, working)
        .map_err(|why| temper_shell::table::refused(&TABLE, command, &why))
}
fn configure_box(arguments: &[Box<[u8]>], environment: &[EnvironmentEntry], working: &[u8]) -> Result<Command, String> {
    let refused = |why: &str| temper_shell::table::refused(&TABLE, &table::BOX, why);
    if arguments.first().is_some_and(|word| !word.starts_with(b"-")) {
        return Err(TABLE.unknown(arguments));
    }
    let (directories, rest) =
        directories::resolve(arguments, environment, working).map_err(|why| refused(&why.to_string()))?;
    let no_tui_count = rest.iter().filter(|word| word.as_ref() == b"--no-tui").count();
    if no_tui_count > 1 {
        return Err(refused("invalid arguments"));
    }
    let rest: Box<[_]> = rest.into_vec().into_iter().filter(|word| word.as_ref() != b"--no-tui").collect();
    let (echo, rest) = box_command::echo(&rest, environment).map_err(|why| refused(&why))?;
    let no_tui = no_tui_count == 1;
    if rest.is_empty() {
        return Ok(Command::Box { directories, echo, no_tui });
    }
    let mut words = rest.iter();
    while let Some(word) = words.next() {
        if word.as_ref() == b"--bundle" || word.as_ref() == b"--scripts" {
            words.next();
        } else if !word.starts_with(b"-") {
            return Err(TABLE.unknown(arguments));
        }
    }
    let rest: Vec<_> = rest.iter().map(|arg| std::ffi::OsStr::from_bytes(arg).to_owned()).collect();
    let fake_backends::Options { bundle, scripts, strict } =
        fake_backends::configure(&rest, working).map_err(|why| refused(&why))?;
    Ok(Command::BoxFakeBackends { directories, bundle, scripts, strict, echo, no_tui })
}

fn configure_row(
    row: &skein_shell::Command,
    arguments: &[Box<[u8]>],
    environment: &[EnvironmentEntry],
    working: &[u8],
) -> Result<Command, String> {
    let invalid = || "invalid arguments".to_string();
    let os =
        |args: &[Box<[u8]>]| args.iter().map(|arg| std::ffi::OsStr::from_bytes(arg).to_owned()).collect::<Vec<_>>();
    let words = match row.reach {
        skein_shell::Reach::RunAs("git") => return Ok(Command::FakeGit { arguments: arguments.into() }),
        skein_shell::Reach::RunAs("claude") => {
            return Ok(Command::Claude { program: "claude".into(), arguments: arguments.into() });
        }
        skein_shell::Reach::RunAs(_) => return Err(invalid()),
        skein_shell::Reach::Words(words) => words,
    };
    match words {
        ["versions"] => return if arguments.is_empty() { Ok(Command::Versions) } else { Err(invalid()) },
        ["--mcp-relay"] => return Ok(Command::Relay),
        ["--program-child"] => return Ok(Command::ProgramChild),
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
        ["internal", "agent-list"] | ["--list", "internal", "agent-list"] => {
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
            let legacy = match words {
                ["forge", "fetch"] => &forge::FETCH,
                ["forge", "prepare"] => &forge::PREPARE,
                ["forge", "run"] => &forge::RUN,
                ["forge", "remove"] => &forge::REMOVE,
                ["forge", "source"] => &forge::SOURCE,
                ["forge", "sign-in"] => &forge::SIGN_IN,
                ["internal", "forge-child"] => &forge::FORGE_CHILD,
                _ => unreachable!("the table dispatches forge commands"),
            };
            return temper_engine_shell::forge::configure(legacy, arguments, environment, working).map(|command| {
                match command {
                    forge::Command::Forge { directories, arguments, ssl_cert_file, environment } => {
                        Command::Forge { directories, arguments, ssl_cert_file, environment }
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
    let rest = os(&rest);
    match words {
        ["init"] if rest.is_empty() => Ok(Command::Init { directories, boxed: false }),
        ["init"] if rest == ["--box"] => Ok(Command::Init { directories, boxed: true }),
        ["serve"] if rest.is_empty() => Ok(Command::Serve { directories, boxed: false }),
        ["internal", "serve"] if rest == ["--box"] => Ok(Command::Serve { directories, boxed: true }),
        ["program-login"] if rest.len() == 1 => Ok(Command::ProgramLogin { directories, name: rest[0].clone() }),
        _ => Err(invalid()),
    }
}

pub fn run(command: Command, environment: &[EnvironmentEntry], working: &[u8], run_as: &[u8]) -> Result<u8, String> {
    let exit = match command {
        Command::Overview => {
            print!("{}", TABLE.help());
            return Ok(0);
        }
        Command::Versions => {
            print!("{}", temper_engine_shell::versions());
            return Ok(0);
        }
        Command::Version => {
            print!("{}", TABLE.version_line());
            return Ok(0);
        }
        Command::Claude { program, arguments } => return Ok(claude_code::run(&program, &arguments, environment)),
        Command::Relay | Command::ProgramChild => unreachable!("smith modes are dispatched by main"),
        Command::Usage(row) => {
            print!("{}", TABLE.command_help(row));
            return Ok(0);
        }
        Command::Init { directories, boxed } => {
            return init::run(&directories, boxed, environment).map_err(|why| prefixed("temper init", &why));
        }
        Command::Serve { directories, boxed } => serve::run(
            "temper",
            environment,
            working,
            &directories,
            boxed,
            Some(&worker_half::Half { environment: environment.into() }),
        )?,
        Command::BoxFakeBackends { directories, bundle, scripts, strict, echo, no_tui } => fake_backends::run(
            &directories,
            &fake_backends::Options { bundle, scripts, strict },
            echo,
            no_tui,
            environment,
        )
        .map_err(|why| prefixed("temper-box", &why))?,
        Command::Box { directories, echo, no_tui } => {
            box_command::run(&directories, echo, no_tui, environment).map_err(|why| prefixed("temper-box", &why))?
        }
        Command::Agent { configuration } => {
            return temper_agent_shell::run(
                "temper internal agent",
                &configuration,
                environment,
                &Ok(path(working).into()),
            );
        }
        Command::AgentList { configuration } => {
            return temper_agent_shell::list("temper internal agent-list", &configuration);
        }
        Command::ProgramLogin { directories, name } => {
            return temper_worker_shell::program_login::run(
                &directories,
                &name,
                environment,
                Some(program_login::deployment),
            )
            .map_err(|why| prefixed("temper program-login", &why.lines().collect::<Vec<_>>().join("; ")));
        }
        Command::Fake { arguments } => return fake::run(&arguments),
        Command::FakeCheck { directories, check } => return Ok(fake::check::run(&directories, &check)),
        Command::FakeGit { arguments } => return Ok(fake::git::run(&arguments, run_as, environment, working)),
        Command::ForgeChild { parent, signal, program, arguments } => {
            return temper_engine_shell::forge::command(
                "temper",
                forge::Command::ForgeChild { parent, signal, program, arguments },
            );
        }
        Command::Forge { directories, arguments, ssl_cert_file, environment } => {
            return temper_engine_shell::forge::command(
                "temper",
                forge::Command::Forge { directories, arguments, ssl_cert_file, environment },
            );
        }
        Command::Help { request } => {
            print!("{}", help::answer(&request)?);
            return Ok(0);
        }
        Command::Ctl { arguments } => return Ok(ctl::run(&arguments, &Clock::new())),
        Command::TuiFront { address } => return Ok(tui_front::run(&address, &Clock::new(), environment, working)),
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
use temper_engine_shell::serve;
pub use temper_engine_shell::serve::{bundle, github_logs, printed};
pub use temper_service_shell::host::{Host, ending as service_ending};
pub(crate) use temper_service_shell::{close, file_path, held_refusal, listeners_refusal};
#[cfg(test)]
fn prepare(
    environment: &[EnvironmentEntry],
    working: &[u8],
    directories: &Directories,
    bytes: &[u8],
    owned: &mut Vec<Fd>,
) -> Result<service::Service, String> {
    serve::prepare(
        "temper",
        environment,
        working,
        Some(&worker_half::Half { environment: environment.into() }),
        directories,
        bytes,
        owned,
    )
}
#[cfg(test)]
fn prepare_with_store(
    environment: &[EnvironmentEntry],
    working: &[u8],
    directories: &Directories,
    bytes: &[u8],
    owned: &mut Vec<Fd>,
) -> Result<(service::Service, Box<[u8]>), String> {
    serve::prepare_with_store(
        "temper",
        environment,
        working,
        Some(&worker_half::Half { environment: environment.into() }),
        directories,
        bytes,
        owned,
    )
}
/// The kernel operations a standalone start keeps in flight at most (its ring's size),
/// derived without retaining its descriptors.
pub fn operations_in(
    environment: &[EnvironmentEntry],
    working: &[u8],
    directories: &Directories,
    config: &[u8],
) -> Result<u32, String> {
    serve::operations_in(
        "temper",
        environment,
        working,
        Some(&worker_half::Half { environment: environment.into() }),
        directories,
        config,
    )
}
/// The same for a boxed start.
pub fn boxed_operations_in(
    environment: &[EnvironmentEntry],
    working: &[u8],
    directories: &Directories,
    config: &[u8],
) -> Result<u32, String> {
    serve::boxed_operations_in(
        "temper",
        environment,
        working,
        Some(&worker_half::Half { environment: environment.into() }),
        directories,
        config,
    )
}
pub fn prepare_in(
    environment: &[EnvironmentEntry],
    working: &[u8],
    directories: &Directories,
) -> Result<service::Service, String> {
    serve::prepare_in(
        "temper",
        environment,
        working,
        Some(&worker_half::Half { environment: environment.into() }),
        directories,
    )
}
/// Prepare an in-process world's service with the supplied defaults.
pub fn prepare_in_with(
    environment: &[EnvironmentEntry],
    working: &[u8],
    directories: &Directories,
    given: &service::stock::Given,
) -> Result<service::Service, String> {
    serve::prepare_in_with(
        "temper",
        environment,
        working,
        Some(&worker_half::Half { environment: environment.into() }),
        directories,
        given,
    )
}
/// The same for an in-process world's service, with the supplied defaults.
pub fn operations_in_with(
    environment: &[EnvironmentEntry],
    working: &[u8],
    directories: &Directories,
    config: &[u8],
    given: &service::stock::Given,
) -> Result<u32, String> {
    serve::operations_in_with(
        "temper",
        environment,
        working,
        Some(&worker_half::Half { environment: environment.into() }),
        directories,
        config,
        given,
    )
}
/// The effective user's login (protocol/setup/init.md, 2); macOS users need not appear in passwd.
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
