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
    dispatch_with(
        captured,
        version,
        commit,
        |line| print!("{line}"),
        |command, working| crate::run(command, &captured.environment, working, &captured.run_as, version, commit),
    )
}

fn dispatch_with(
    captured: &Capture,
    version: &'static str,
    commit: &'static str,
    stdout: impl FnOnce(&str),
    run: impl FnOnce(Command, &[u8]) -> Result<u8, String>,
) -> Result<ExitCode, (u8, String)> {
    dispatch_routed(
        captured,
        version,
        commit,
        stdout,
        |command, working| {
            if let Command::Relay = command {
                return Ok(smith_agent_shell::relay::main());
            }
            if let Command::ProgramChild = command {
                return Ok(smith_agent_shell::program_child::main());
            }
            run(command, working).map(ExitCode::from)
        },
        || front_pipes(skein_shell::descriptor_writes),
    )
}

fn dispatch_routed(
    captured: &Capture,
    version: &'static str,
    commit: &'static str,
    stdout: impl FnOnce(&str),
    run: impl FnOnce(Command, &[u8]) -> Result<ExitCode, String>,
    check_front: impl FnOnce() -> Result<(), (u8, String)>,
) -> Result<ExitCode, (u8, String)> {
    let command = match TABLE.reach(&captured.run_as, &captured.arguments) {
        Reached::Help => {
            stdout(&TABLE.help());
            return Ok(ExitCode::SUCCESS);
        }
        Reached::Version => {
            stdout(&Table { version, commit, ..TABLE }.version_line());
            return Ok(ExitCode::SUCCESS);
        }
        Reached::CommandHelp(command) => {
            stdout(&TABLE.command_help(command));
            return Ok(ExitCode::SUCCESS);
        }
        Reached::Unknown(why) => return Err((2, why)),
        Reached::Command(command, arguments) => {
            if command.reach == Reach::Words(&["help"]) {
                if let [page] = arguments
                    && page.as_ref() == b"config"
                {
                    stdout(
                        std::str::from_utf8(&temper_config_shell::help(temper_config_shell::Shape::Standalone))
                            .expect("help UTF-8"),
                    );
                } else {
                    stdout(&temper_shell::table::help(&TABLE, arguments)?);
                }
                return Ok(ExitCode::SUCCESS);
            }
            if command.reach == Reach::Words(&["versions"]) {
                let command =
                    crate::configure_command(command, arguments, &captured.arguments, &captured.environment, b"")
                        .map_err(|why| (2, why))?;
                return run(command, b"").map_err(|why| (1, why));
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
    if let Command::Claude { arguments, .. } = command {
        return Ok(ExitCode::from(crate::claude_code::run(
            std::ffi::OsStr::from_bytes(&captured.run_as),
            &arguments,
            &captured.environment,
        )));
    }
    if matches!(&command, Command::Box { front: true, .. } | Command::BoxFakeBackends { front: true, .. }) {
        check_front()?;
    }
    run(command, captured.working.as_ref().map_err(|why| (1, format!("temper-box: {why}\n")))?.as_os_str().as_bytes())
        .map_err(|why| (1, why))
}

fn front_pipes(mut writes: impl FnMut(skein_io::kernel::Fd) -> Result<bool, i32>) -> Result<(), (u8, String)> {
    for fd in [3, 4] {
        if writes(skein_io::kernel::Fd::new(fd)) != Ok(true) {
            return Err((2, "temper-box --front: fd 3 and fd 4 are its parent's pipes\n".into()));
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "binary/argv_tests.rs"]
mod argv_tests;

#[cfg(test)]
pub(crate) fn dispatch_fixture<'a>(
    words: impl IntoIterator<Item = &'a str>,
    environment: Box<[crate::EnvironmentEntry]>,
    working: PathBuf,
    stdout: impl FnOnce(&str),
    run: impl FnOnce(Command, &[u8]) -> Result<u8, String>,
) -> Result<ExitCode, (u8, String)> {
    let capture = Capture {
        run_as: b"temper-box".as_slice().into(),
        arguments: words.into_iter().map(|word| word.as_bytes().into()).collect(),
        environment,
        working: Ok(working),
    };
    dispatch_with(&capture, "fixture", "fixture", stdout, run)
}

#[cfg(test)]
mod tests {
    #[test]
    fn serve_and_bare_box_refuse_moved_limits_at_the_configuration_read() {
        use crate::{Command, configure_with};
        use temper_config_shell::{FILE, Facts, Shape};
        use temper_service_shell::configuration::Read;

        let read = |directories: &crate::Directories, shape| {
            assert_eq!(directories.config.as_ref(), b"/fixture");
            assert_eq!(shape, Shape::Standalone);
            let facts = Facts { login: "ada".into(), hostname: "fixture".into(), provider_key: false };
            temper_config_shell::parse(b"limits = {}\n", shape, &facts)
                .map(|temper_config_shell::Read { file, document, keys }| Read { file, document, keys })
                .map_err(|why| why.to_string())
        };
        for (words, prefix) in [(&["serve"][..], "temper serve"), (&[][..], "temper-box")] {
            let arguments = words
                .iter()
                .copied()
                .chain(["--config", "/fixture", "--data", "/fixture", "--cache", "/fixture"])
                .map(|word| Box::from(word.as_bytes()))
                .collect::<Vec<_>>();
            let command = configure_with(&arguments, &[], b"/").expect("command arguments");
            let result = if let Command::Serve { directories, boxed: false, record: false } = command {
                crate::serve::prepare_in_with_read("temper", &[], b"/", &directories, read).map(|_| ())
            } else {
                let Command::Box { directories, echo, front, record } = command else {
                    panic!("expected ordinary serve or bare box");
                };
                crate::box_status(crate::box_command::run_with_configuration(
                    &directories,
                    echo,
                    front,
                    record,
                    &[],
                    |directories, _| read(directories, Shape::Standalone).map(|read| (read, Vec::new(), None)),
                ))
                .map(|_| ())
            };
            assert_eq!(
                result.expect_err("refused before preparation"),
                format!(
                    "{prefix}: {FILE}:1: limits: moved: limits are settings; ask temper, or use temper ctl settings apply\n"
                ),
                "{words:?}"
            );
        }
    }

    #[test]
    fn boxed_serve_reports_bad_pipes_before_stdin_or_configuration() {
        use crate::{
            Command,
            serve::{
                PipeKind::{Absent, Open, Other},
                box_inputs_with,
            },
        };
        use temper_service_shell::{read_start, serve_error};

        let arguments =
            ["internal", "serve", "--config", "/unused", "--data", "/unused", "--cache", "/unused", "--box"]
                .map(|word| Box::from(word.as_bytes()));
        let captured = super::Capture {
            run_as: Box::from(b"temper-box".as_slice()),
            arguments: Box::new(arguments),
            environment: Box::new([]),
            working: Ok("/".into()),
        };
        // Supply only the number of listeners already admitted, never the listener itself.
        for (listeners, first) in [(0, 3), (1, 4)] {
            for (name, facts, offset) in [
                ("closed first", &[Absent][..], 0),
                ("non-pipe first", &[Other][..], 0),
                ("closed second", &[Open, Absent][..], 1),
            ] {
                let mut stdout = Vec::new();
                let mut next_fd = first;
                let mut facts = facts.iter().copied();
                let (status, stderr) = super::dispatch_with(
                    &captured,
                    "test",
                    "test",
                    |line| stdout.extend_from_slice(line.as_bytes()),
                    |command, _| {
                        let Command::Serve { boxed: true, .. } = command else {
                            panic!("expected boxed serve");
                        };
                        read_start(
                            listeners,
                            |count| {
                                box_inputs_with(
                                    true,
                                    count,
                                    |fd| {
                                        assert_eq!(fd, next_fd, "{name}, {listeners} listeners");
                                        next_fd += 1;
                                        facts.next().expect("no probe after the first bad pipe")
                                    },
                                    || panic!("pipes are checked before stdin"),
                                )
                            },
                            || panic!("pipes are checked before the configuration read"),
                        )
                        .map_err(|why| serve_error("temper-box", &why))?;
                        panic!("no start with a bad pipe");
                    },
                )
                .expect_err("pipe refusal");
                assert!(facts.next().is_none(), "{name}, {listeners} listeners: every expected probe ran");
                assert_eq!(status, 1, "{name}, {listeners} listeners");
                assert_eq!(stdout.as_slice(), b"", "{name}, {listeners} listeners");
                assert_eq!(stderr, format!("temper-box serve: --box: fd {} is not an open pipe\n", first + offset));
            }
        }
    }

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
