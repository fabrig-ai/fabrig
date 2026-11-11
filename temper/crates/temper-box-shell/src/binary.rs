//! The box's capture and dispatch (temper/docs/design/cli/README.md, 3 and 4).
use crate::{Command, TABLE};
use skein_shell::{Reach, Reached};
use std::{os::unix::ffi::OsStrExt, path::PathBuf, process::ExitCode};

struct Capture {
    run_as: Box<[u8]>,
    arguments: Box<[Box<[u8]>]>,
    environment: Box<[crate::EnvironmentEntry]>,
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

#[must_use]
pub fn main() -> ExitCode {
    let captured = capture();
    match dispatch(&captured) {
        Ok(code) => code,
        Err((code, why)) => {
            eprint!("{why}");
            ExitCode::from(code)
        }
    }
}

fn dispatch(captured: &Capture) -> Result<ExitCode, (u8, String)> {
    let command = match TABLE.reach(&captured.run_as, &captured.arguments) {
        Reached::Help => {
            print!("{}", TABLE.help());
            return Ok(ExitCode::SUCCESS);
        }
        Reached::Version => {
            print!("{}", TABLE.version_line());
            return Ok(ExitCode::SUCCESS);
        }
        Reached::CommandHelp(command) => {
            print!("{}", TABLE.command_help(command));
            return Ok(ExitCode::SUCCESS);
        }
        Reached::Unknown(why) => return Err((2, why)),
        Reached::Command(command, arguments) => {
            if command.reach == Reach::Words(&["help"]) {
                print!("{}", temper_shell::table::help(&TABLE, arguments)?);
                return Ok(ExitCode::SUCCESS);
            }
            if command.reach == Reach::Words(&["versions"]) {
                let command =
                    crate::configure_command(command, arguments, &captured.arguments, &captured.environment, b"")
                        .map_err(|why| (2, why))?;
                return crate::run(command, &captured.environment, b"", &captured.run_as)
                    .map(ExitCode::from)
                    .map_err(|why| (1, why));
            }
            let working = captured.working.as_ref().map_err(|why| (1, format!("temper-box: {why}\n")))?;
            crate::configure_command(
                command,
                arguments,
                &captured.arguments,
                &captured.environment,
                working.as_os_str().as_bytes(),
            )
            .map_err(|why| (2, why))?
        }
    };
    if let Command::Relay = command {
        return Ok(smith_agent_shell::relay::main());
    }
    if let Command::ProgramChild = command {
        return Ok(smith_agent_shell::program_child::main());
    }
    if let Command::Claude { arguments, .. } = command {
        return Ok(ExitCode::from(crate::claude_code::run(
            std::ffi::OsStr::from_bytes(&captured.run_as),
            &arguments,
            &captured.environment,
        )));
    }
    if matches!(&command, Command::Box { front: true, .. } | Command::BoxFakeBackends { front: true, .. })
        && ![3, 4].into_iter().all(|fd| skein_shell::descriptor_writes(skein_io::kernel::Fd::new(fd)) == Ok(true))
    {
        return Err((2, "temper-box --front: fd 3 and fd 4 are its parent's pipes\n".into()));
    }
    crate::run(
        command,
        &captured.environment,
        captured.working.as_ref().map_err(|why| (1, format!("temper-box: {why}\n")))?.as_os_str().as_bytes(),
        &captured.run_as,
    )
    .map(ExitCode::from)
    .map_err(|why| (1, why))
}
