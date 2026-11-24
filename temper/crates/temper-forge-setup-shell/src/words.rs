//! Diagnostics for the owned forge commands (temper/docs/design/testing/box/README.md, 2.6).
use crate::{command, prepare, run as host};
use std::fmt::Write;
use temper_forge_setup::{
    place::{File, Place},
    prepare::{Exit, Failure},
    run::{self, Refusal},
    spec::{FORGEJO_VERSION, RUNNER_VERSION, Which},
    upgrade::Course,
};

/// A command's diagnostic, without any credential contents.
#[derive(Debug)]
pub struct Refused {
    verb: &'static str,
    number: Option<u16>,
    why: String,
}
impl Refused {
    #[must_use]
    pub const fn new(verb: &'static str, number: Option<u16>, why: String) -> Self {
        Self { verb, number, why }
    }

    #[must_use]
    pub fn line(&self) -> String {
        self.line_with_binary("temper")
    }
    #[must_use]
    pub fn line_with_binary(&self, binary: &str) -> String {
        let forge = self.number.map_or_else(String::new, |n| format!("forge {n}: "));
        let reason = format!("{forge}{}", self.why);
        let mut lines = String::new();
        for line in reason.lines() {
            writeln!(lines, "{binary} forge {}: {line}", self.verb).expect("writing into a string");
        }
        lines
    }
}
/// Emits the command's diagnostic, distinguishing the source command's status line.
#[must_use]
pub fn report(
    binary: &str,
    result: Result<u8, Refused>,
    source: bool,
    stdout: impl FnOnce(String),
    stderr: impl FnOnce(String),
) -> u8 {
    match result {
        Ok(code) => code,
        Err(why) if source => {
            let reason: String = why
                .why
                .bytes()
                .take(1024)
                .map(|c| if c.is_ascii_graphic() || c == b' ' { char::from(c) } else { ' ' })
                .collect();
            stdout(format!("source failed {reason}"));
            0
        }
        Err(why) => {
            stderr(why.line_with_binary(binary));
            1
        }
    }
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}
fn status(exit: Exit) -> String {
    match exit {
        Exit::Code(n) => n.to_string(),
        Exit::Signal(n) => format!("signal {n}"),
    }
}
#[must_use]
pub fn command_error(why: command::Error) -> String {
    match why {
        command::Error::Interrupted => "interrupted".into(),
        command::Error::Kernel(why) => format!("command kernel: {why}"),
        command::Error::Spawn(why) => format!("command could not start: {why:?}"),
    }
}
#[must_use]
pub fn prepare_failure(binary: &str, why: prepare::Failed, place: &Place) -> String {
    match why {
        prepare::Failed::Held(file) => format!(
            "{} is running or being prepared already",
            if file == File::RunnerLock { "runner" } else { "forgejo" }
        ),
        prepare::Failed::Io(file, why) => format!("{}: {file}: {why}", text(&place.directory)),
        prepare::Failed::Command(why) => command_error(why),
        prepare::Failed::Refused(why) => match why {
            Failure::Binary { program, printed, .. } => {
                format!("{} printed {printed:?}, not the pinned release", text(&program))
            }
            Failure::Command { command, exit, tail } => {
                format!("{command:?} ended with status {}: {}", status(exit), text(&tail))
            }
            Failure::Token { exit, tail } => {
                format!("no token for temper-admin: ended with status {}: {}", status(exit), text(&tail))
            }
            Failure::PersonToken { file, exit, tail } => {
                format!(
                    "{}: no person token: ended with status {}: {}",
                    text(&place.path(file)),
                    status(exit),
                    text(&tail)
                )
            }
            Failure::Unread { command } => format!("{command:?} printed no required value"),
            Failure::State(why) => format!("{}: state cannot be decoded: {why:?}", text(&place.directory)),
            Failure::Unrecorded => format!("{}: Forgejo files have no preparation state", text(&place.directory)),
            Failure::Newer { which, recorded } => {
                let (name, pin) = pinned(which);
                format!(
                    "{name} is prepared at {}, newer than this {binary}'s pin {pin}: its migrations only go forward",
                    text(&recorded)
                )
            }
            Failure::Unreadable { which } => {
                format!("{} is prepared at a version this {binary} cannot read", pinned(which).0)
            }
        },
    }
}
fn pinned(which: Which) -> (&'static str, &'static str) {
    match which {
        Which::Forgejo => ("forgejo", FORGEJO_VERSION),
        Which::Runner => ("runner", RUNNER_VERSION),
    }
}
#[must_use]
pub fn prepare_hint(
    binary: &str,
    why: &str,
    number: u16,
    directories: &temper_shell::directories::Directories,
) -> String {
    let flags = directories.flags().iter().map(|arg| text(arg)).collect::<Vec<_>>().join(" ");
    format!("{why}: run {binary} forge prepare {flags} {number}")
}
fn run_refusal(binary: &str, why: Refusal, launch: &run::Launch, prepare: impl Fn(&str) -> String) -> String {
    match why {
        Refusal::NoRunner => "the setup has no runner".into(),
        Refusal::NotPrepared => prepare("not prepared"),
        Refusal::Unfinished => prepare("its preparation did not end"),
        Refusal::Runner { recorded } => prepare(if recorded {
            "prepared with a runner, and the setup has none"
        } else {
            "prepared without a runner, and the setup has one"
        }),
        Refusal::Versions { course: Course::Unreadable, .. } => {
            format!("prepared at a version this {binary} cannot read: no command of this {binary} runs it")
        }
        Refusal::Versions { forgejo, runner, course } => {
            let recorded =
                runner.map_or_else(|| text(&forgejo), |runner| format!("{} / {}", text(&forgejo), text(&runner)));
            let pinned =
                if launch.runner { format!("{FORGEJO_VERSION} / {RUNNER_VERSION}") } else { FORGEJO_VERSION.into() };
            let reason = format!("prepared at {recorded}, this {binary} pins {pinned}");
            if course == Course::Newer {
                format!("{reason}: no command of this {binary} runs it")
            } else {
                prepare(&reason)
            }
        }
        Refusal::Binary { printed } => format!("{} printed {printed:?}, not the pinned release", text(&launch.program)),
    }
}
#[must_use]
pub fn run_failure(
    binary: &str,
    why: host::Failed,
    launch: &run::Launch,
    number: u16,
    directories: &temper_shell::directories::Directories,
) -> String {
    match why {
        host::Failed::Refused(why) => {
            run_refusal(binary, why, launch, |why| prepare_hint(binary, why, number, directories))
        }
        host::Failed::Locked => format!(
            "{} is running or being prepared already",
            if launch.which == Which::Forgejo { "forgejo" } else { "runner" }
        ),
        host::Failed::Lock(why) => format!("{}: lock: {why}", text(&launch.lock)),
        host::Failed::State(why) => format!("{}: state {why}", text(&launch.state)),
        host::Failed::Version(why) => command_error(why),
        host::Failed::Exec(why) => format!("{}: {why}", text(&launch.program)),
    }
}
