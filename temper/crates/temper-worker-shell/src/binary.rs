//! The worker's capture and dispatch (temper/docs/design/cli/README.md, 3, 4 and 7.2).
use crate::{INTERNAL_WORKSPACE_REUSE, PROGRAM_LOGIN};
use skein_io::kernel::Exit;
use skein_shell::{Argument, Command, Reach, Reached, Set, Table};
use std::{ffi::OsStr, os::unix::ffi::OsStrExt, path::PathBuf, process::ExitCode};
use temper_shell::{
    directories::{self, EnvironmentEntry},
    environment, table,
};

pub const WORKER: Table = Table {
    binary: "temper-worker",
    line: "the deployment's worker: a worker alone, and its programs' sign-in.",
    // The identity is main's (temper/docs/design/cli/README.md, 3).
    version: "",
    commit: "",
    commands: &[SERVE, PROGRAM_LOGIN, table::HELP],
    settings: &[&directories::SETTINGS],
    variables: &[
        &environment::JOB,
        &INTERNAL_WORKSPACE_REUSE,
        &environment::INTERNAL_DUMP_LLM,
        &temper_shell::environment::INTERNAL_COMMIT,
        &temper_service_shell::WAIT_CHECK,
        &temper_service_shell::CUT_AT_LISTEN,
    ],
    more: "temper-worker COMMAND --help prints one command; temper-worker help pages lists the pages.",
};
const SERVE: Command = Command {
    reach: Reach::Words(&["serve"]),
    set: Set::Public,
    forms: &["[DIRECTORIES]"],
    line: "Run a worker alone's document until stopped; temper-engine serve runs an engine alone, and the box's temper-box serve runs an engine and its worker in one process.",
    arguments: &[Argument {
        shown: temper_shell::commands::DIRECTORIES.shown,
        line: temper_shell::commands::DIRECTORIES.line,
    }],
    settings: &[&directories::SETTINGS],
    switches: &[],
    variables: &[&environment::LISTEN_FDS, &environment::XDG_RUNTIME_DIR],
    notes: "Its stderr carries its startup notes, its linked and lost lines and the reason of a failure. A first termination signal runs its worker's ending; a second aborts. Exit 0 after its ending, 1 on a refused start or any failure. Refuses an engine alone's document, naming temper-engine serve, or an engine and its worker in one process, naming the box's temper-box serve.",
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
    match WORKER.reach(&captured.run_as, &captured.arguments) {
        Reached::Help => {
            print!("{}", WORKER.help());
            Ok(0)
        }
        Reached::Version => {
            print!("{}", Table { version, commit, ..WORKER }.version_line());
            Ok(0)
        }
        Reached::CommandHelp(command) => {
            print!("{}", WORKER.command_help(command));
            Ok(0)
        }
        Reached::Unknown(why) => Err((2, why)),
        Reached::Command(command, arguments) => dispatch(command, arguments, captured),
    }
}

fn dispatch(command: &Command, arguments: &[Box<[u8]>], captured: &Capture) -> Result<u8, (u8, String)> {
    let refused = |why: &str| (2, table::refused(&WORKER, command, why));
    match command.reach {
        Reach::Words(["help"]) => {
            if let [page] = arguments
                && page.as_ref() == b"config"
            {
                print!(
                    "{}",
                    std::str::from_utf8(&temper_config_shell::help(temper_config_shell::Shape::WorkerAlone))
                        .expect("help UTF-8")
                );
            } else {
                print!("{}", table::help(&WORKER, arguments)?);
            }
            Ok(0)
        }
        Reach::Words(["serve"]) => {
            let working = captured.working.as_ref().map_err(|why| (1, format!("temper-worker serve: {why}\n")))?;
            let (directories, rest) =
                directories::resolve(arguments, &captured.environment, working.as_os_str().as_bytes())
                    .map_err(|why| refused(&why.to_string()))?;
            if !rest.is_empty() {
                return Err(refused("invalid arguments"));
            }
            Ok(
                match temper_service_shell::run(
                    WORKER.binary,
                    &directories,
                    temper_config_shell::Shape::WorkerAlone,
                    &captured.environment,
                    crate::served::SERVED,
                    &skein_shell::Clock::new(),
                    |_| Ok(()),
                    |bytes, programs, owned, ()| {
                        crate::worker::alone_with_programs(&directories, bytes, owned, &captured.environment, programs)
                            .map(|service| temper_service_shell::Prepared {
                                service,
                                store: Box::new([]),
                                adoption: (),
                            })
                    },
                    |_, _, ()| Ok(()),
                )
                .map_err(|why| (1, why))?
                {
                    Exit::Code(code) => code,
                    Exit::Signal(_) => 1,
                },
            )
        }
        Reach::Words(["program-login"]) => {
            let working =
                captured.working.as_ref().map_err(|why| (1, format!("temper-worker program-login: {why}\n")))?;
            let (directories, rest) =
                directories::resolve(arguments, &captured.environment, working.as_os_str().as_bytes())
                    .map_err(|why| refused(&why.to_string()))?;
            let [name] = rest.as_ref() else { return Err(refused("invalid arguments")) };
            crate::program_login::run(&directories, OsStr::from_bytes(name), &captured.environment).map_err(|why| {
                (1, temper_service_shell::configuration::login_problems("temper-worker program-login", &why))
            })
        }
        Reach::Words(_) | Reach::RunAs(_) => Err(refused("invalid arguments")),
    }
}

#[cfg(test)]
mod configuration_tests {
    #[test]
    fn toml_worker_login_prints_each_refusal_on_its_own_line() {
        let scratch = skein_scratch::Scratch::new("worker-login-refusal-lines");
        let directories = crate::tests::configuration::directories(scratch.path());
        std::fs::write(
            scratch.path().join(temper_config_shell::FILE),
            "engine = \"127.0.0.1:7200\"\nalpha = 1\nbeta = 2\n",
        )
        .expect("worker file");
        let mut arguments = vec![Box::from(b"program-login".as_slice())];
        arguments.extend(directories.flags());
        arguments.push(b"source".as_slice().into());
        let capture = super::Capture {
            run_as: b"temper-worker".as_slice().into(),
            arguments: arguments.into(),
            environment: Box::new([]),
            working: Ok(scratch.path().into()),
        };
        let (code, why) = super::run(&capture, "test", "test").expect_err("file refused");
        assert_eq!(code, 1);
        assert_eq!(
            why.lines().collect::<Vec<_>>(),
            [
                format!(
                    "temper-worker program-login: {}:2: alpha: unknown; temper-worker help config lists the keys",
                    temper_config_shell::FILE
                ),
                format!(
                    "temper-worker program-login: {}:3: beta: unknown; temper-worker help config lists the keys",
                    temper_config_shell::FILE
                ),
            ]
        );
    }

    #[test]
    fn worker_commands_refuse_the_former_file_by_name() {
        use temper_config_shell::{FILE, FORMER};
        let scratch = skein_scratch::Scratch::new("worker-former-file");
        let directories = crate::tests::configuration::directories(scratch.path());
        std::fs::create_dir(scratch.path().join(FORMER)).expect("former directory");
        for present in [false, true] {
            if present {
                std::fs::write(scratch.path().join(FILE), b"invalid TOML").expect("current file");
            }
            for words in [&["serve"][..], &["program-login", "source"][..]] {
                let mut arguments: Vec<_> = words.iter().map(|word| Box::from(word.as_bytes())).collect();
                arguments.extend(directories.flags());
                let capture = super::Capture {
                    run_as: b"temper-worker".as_slice().into(),
                    arguments: arguments.into(),
                    environment: Box::new([]),
                    working: Ok(scratch.path().into()),
                };
                assert_eq!(
                    super::run(&capture, "test", "test"),
                    Err((
                        1,
                        format!(
                            "temper-worker {}: {FORMER}: moved: the configuration is {FILE}; write it from temper-worker help config, then remove {FORMER}\n",
                            words.first().expect("command")
                        )
                    ))
                );
            }
        }
    }
}
