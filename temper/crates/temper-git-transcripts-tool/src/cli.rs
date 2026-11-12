//! The recorder's command line (temper/docs/design/testing/tools.md, 4).
use skein_shell::{Command, Fallback, Front, Reach, Set, Setting, Settings, Table, Variable, Way};
use std::{ffi::OsStr, os::unix::ffi::OsStrExt, path::Path, process::ExitCode};

const SETTINGS: Settings = Settings {
    tool: "temper-git-transcripts",
    about: "The git executable, scratch directory and output directory.",
    table: &[
        Setting {
            name: "git",
            value: "PATH",
            about: "Absolute path to git 2.41 or later.",
            fallback: Fallback::Refuse,
            set: Set::Internal,
            twin: None,
            notes: "",
        },
        Setting {
            name: "scratch",
            value: "DIR",
            about: "Scratch directory for the isolated Forgejo.",
            fallback: Fallback::Refuse,
            set: Set::Internal,
            twin: None,
            notes: "",
        },
        Setting {
            name: "out",
            value: "DIR",
            about: "Directory for the masked transcripts and facts.",
            fallback: Fallback::Refuse,
            set: Set::Internal,
            twin: None,
            notes: "",
        },
    ],
};
const TEMPER_TEST_FORGEJO: Variable = Variable {
    name: "TEMPER_TEST_FORGEJO",
    set: Set::Test,
    form: "PATH",
    line: "Absolute path to the pinned Forgejo.",
    absent: "Refused: TEMPER_TEST_FORGEJO must name the pinned Forgejo.",
    notes: "",
    way: Way::Reads,
};
const RECORD: Command = Command {
    reach: Reach::Words(&[]),
    set: Set::Internal,
    forms: &["--git PATH --scratch DIR --out DIR"],
    line: "Record real git's invocations against an isolated Forgejo, with Actions off.",
    arguments: &[],
    settings: &[&SETTINGS],
    switches: &[],
    variables: &[],
    notes: "",
};
/// The recorder's table (temper/docs/design/testing/tools.md, 4.1).
pub const TABLE: Table = Table {
    binary: "temper-git-transcripts",
    line: "git's transcripts, recorded from real git against a throwaway Forgejo.",
    version: temper_version::VERSION,
    commit: temper_version::COMMIT,
    commands: &[RECORD],
    settings: &[],
    variables: &[&TEMPER_TEST_FORGEJO],
    more: "Read temper/docs/design/protocol/git.md, section 9.",
};

/// Capture once and answer the table before recording anything.
#[must_use]
#[expect(
    clippy::disallowed_methods,
    reason = "the binary's capture: temper-git-transcripts' arguments and environment, once (skein/docs/foundation/cli.md, 6)"
)]
pub fn main() -> ExitCode {
    let arguments = skein_shell::arguments();
    let environment = skein_shell::environment();
    match TABLE.front(&arguments) {
        Front::Print(text) => {
            print!("{text}");
            ExitCode::SUCCESS
        }
        Front::Refuse(text) => {
            eprint!("{text}");
            ExitCode::from(2)
        }
        Front::Run { command, rest } => {
            let (git, scratch, out) = match paths(rest) {
                Ok(paths) => paths,
                Err(why) => {
                    eprint!("{}", TABLE.refusal(command, &why));
                    return ExitCode::from(2);
                }
            };
            let result = TEMPER_TEST_FORGEJO
                .value(&environment)
                .ok_or_else(|| "TEMPER_TEST_FORGEJO must name the pinned Forgejo".into())
                .and_then(|forgejo| crate::run(git, scratch, out, Path::new(OsStr::from_bytes(forgejo))));
            match result {
                Ok(()) => ExitCode::SUCCESS,
                Err(why) => {
                    eprintln!("temper-git-transcripts: {why}");
                    ExitCode::FAILURE
                }
            }
        }
    }
}

fn paths(arguments: &[Box<[u8]>]) -> Result<(&Path, &Path, &Path), String> {
    let mut git = None;
    let mut scratch = None;
    let mut out = None;
    let mut arguments = arguments.iter();
    while let Some(argument) = arguments.next() {
        let slot = match argument.as_ref() {
            b"--git" => &mut git,
            b"--scratch" => &mut scratch,
            b"--out" => &mut out,
            other => return Err(format!("unknown argument {}", String::from_utf8_lossy(other))),
        };
        let value = arguments.next().ok_or_else(|| format!("{} takes a value", String::from_utf8_lossy(argument)))?;
        *slot = Some(Path::new(OsStr::from_bytes(value)));
    }
    Ok((git.ok_or("--git is not given")?, scratch.ok_or("--scratch is not given")?, out.ok_or("--out is not given")?))
}
