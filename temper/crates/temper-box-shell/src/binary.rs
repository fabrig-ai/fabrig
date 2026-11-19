//! The box's capture and dispatch (temper/docs/design/cli/README.md, 3 and 4).
use crate::{Command, TABLE};
use skein_shell::{Reach, Reached, Table};
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

pub fn main(version: &'static str, commit: &'static str) -> ExitCode {
    let captured = capture();
    match dispatch(&captured, version, commit) {
        Ok(code) => code,
        Err((code, why)) => {
            eprint!("{why}");
            ExitCode::from(code)
        }
    }
}

fn dispatch(captured: &Capture, version: &'static str, commit: &'static str) -> Result<ExitCode, (u8, String)> {
    let command = match TABLE.reach(&captured.run_as, &captured.arguments) {
        Reached::Help => {
            print!("{}", TABLE.help());
            return Ok(ExitCode::SUCCESS);
        }
        Reached::Version => {
            print!("{}", Table { version, commit, ..TABLE }.version_line());
            return Ok(ExitCode::SUCCESS);
        }
        Reached::CommandHelp(command) => {
            print!("{}", TABLE.command_help(command));
            return Ok(ExitCode::SUCCESS);
        }
        Reached::Unknown(why) => return Err((2, why)),
        Reached::Command(command, arguments) => {
            if command.reach == Reach::Words(&["help"]) {
                if let [page] = arguments
                    && page.as_ref() == b"config"
                {
                    print!(
                        "{}",
                        std::str::from_utf8(&temper_config_shell::help(temper_config_shell::Shape::Standalone))
                            .expect("help UTF-8")
                    );
                } else {
                    print!("{}", temper_shell::table::help(&TABLE, arguments)?);
                }
                return Ok(ExitCode::SUCCESS);
            }
            if command.reach == Reach::Words(&["versions"]) {
                let command =
                    crate::configure_command(command, arguments, &captured.arguments, &captured.environment, b"")
                        .map_err(|why| (2, why))?;
                return crate::run(command, &captured.environment, b"", &captured.run_as, version, commit)
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
        version,
        commit,
    )
    .map(ExitCode::from)
    .map_err(|why| (1, why))
}

#[cfg(test)]
mod tests {
    #[test]
    fn box_commands_refuse_the_former_file_by_name() {
        use temper_config_shell::{FILE, FORMER};
        let scratch = skein_scratch::Scratch::new("box-former-file");
        let directories = crate::tests::configuration::directories(scratch.path());
        std::fs::create_dir(scratch.path().join(FORMER)).expect("former directory");
        for present in [false, true] {
            if present {
                std::fs::write(scratch.path().join(FILE), b"invalid TOML").expect("current file");
            }
            for (words, command) in [
                (&["serve"][..], "temper-box serve"),
                (&["program-login", "source"][..], "temper-box program-login"),
                (&["upgrade", "--check", "/unused"][..], "temper-box upgrade"),
                (&["upgrade", "--workers", "/unused"][..], "temper-box upgrade"),
                (&["upgrade", "--back"][..], "temper-box upgrade"),
                (&["checkpoint"][..], "temper-box checkpoint"),
                (&["restore", "00000000000000000012-00000000000000ff"][..], "temper-box restore"),
                (&[][..], "temper-box"),
            ] {
                let mut arguments: Vec<_> = words.iter().map(|word| Box::from(word.as_bytes())).collect();
                arguments.extend(directories.flags());
                let capture = super::Capture {
                    run_as: b"temper-box".as_slice().into(),
                    arguments: arguments.into(),
                    environment: Box::new([]),
                    working: Ok(scratch.path().into()),
                };
                let (code, why) = super::dispatch(&capture, "test", "test").expect_err("former file refused");
                assert_eq!(code, 1, "{words:?}: {why}");
                assert_eq!(
                    why,
                    format!(
                        "{command}: {FORMER}: moved: the configuration is {FILE}; write it from temper-box help config, then remove {FORMER}\n"
                    )
                );
            }
        }
    }
}
