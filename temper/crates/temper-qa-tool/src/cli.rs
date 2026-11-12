//! The table and command boundary (testing/tools.md, 4; qa/README.md, 2).
use crate::{CaseInput, Home, Refusal, refused};
use skein_shell::{Argument, Command, Front, Reach, Set, Table, Variable, Way};
use std::{ffi::OsStr, io::Read, os::unix::ffi::OsStrExt, path::Path, process::ExitCode};

pub(crate) type Environment = [(Box<[u8]>, Box<[u8]>)];

pub const HOME: Variable = Variable {
    name: "TEMPER_INTERNAL_QA_HOME",
    set: Set::Internal,
    form: "DIR",
    line: "The deployment's QA home, holding qa.json and ledger.json.",
    absent: "Required by every command but the front's.",
    notes: "",
    way: Way::Reads,
};
pub const SESSION: Variable = Variable {
    name: "TEMPER_INTERNAL_QA_SESSION",
    set: Set::Internal,
    form: "N",
    line: "The running triage session the agent's commands record for.",
    absent: "Triage commands are refused.",
    notes: "Set by the loop for the agent it starts.",
    way: Way::Reads,
};
const KEEP: Command = Command {
    reach: Reach::Words(&["build", "keep"]),
    set: Set::Internal,
    forms: &["COMMIT DIR"],
    line: "Every executable must print that commit as its version, the files are moved into the builds directory under the commit, a manifest names the commit and each file's checksum, and the ledger records the build.",
    arguments: &[
        Argument { shown: "COMMIT", line: "The landed tip, 40 or 64 lowercase hexadecimal digits." },
        Argument { shown: "DIR", line: "The staged build, on the builds directory's filesystem." },
    ],
    settings: &[],
    switches: &[],
    variables: &[&HOME],
    notes: "",
};
const NONE: Command = Command {
    reach: Reach::Words(&["triage", "none"]),
    set: Set::Internal,
    forms: &["WHY"],
    line: "Answer no QA for the session's landing, with a line saying why.",
    arguments: &[Argument { shown: "WHY", line: "One nonempty argument saying why no QA is needed." }],
    settings: &[],
    switches: &[],
    variables: &[&HOME, &SESSION],
    notes: "",
};
const CASE: Command = Command {
    reach: Reach::Words(&["triage", "case"]),
    set: Set::Internal,
    forms: &["FILE"],
    line: "Queue a checked case, or join the queued case for its surface and page.",
    arguments: &[Argument { shown: "FILE", line: "A case JSON file, or - to read stdin." }],
    settings: &[],
    switches: &[],
    variables: &[&HOME, &SESSION],
    notes: "",
};
const RUN: Command = Command {
    reach: Reach::Words(&["run"]),
    set: Set::Internal,
    forms: &[""],
    line: "Watch main and triage its landings, one service per QA home.",
    arguments: &[],
    settings: &[],
    switches: &[],
    variables: &[&HOME],
    notes: "Holds run.lock for its life; reads qa.json once at start.",
};
const SHOW: Command = Command {
    reach: Reach::Words(&["show"]),
    set: Set::Internal,
    forms: &[""],
    line: "Print the ledger's landings, cases, builds and sessions, oldest first.",
    arguments: &[],
    settings: &[],
    switches: &[],
    variables: &[&HOME],
    notes: "",
};
const LINES: Command = Command {
    reach: Reach::Words(&["lines"]),
    set: Set::Internal,
    forms: &[""],
    line: "Print the ledger's lines, oldest first.",
    arguments: &[],
    settings: &[],
    switches: &[],
    variables: &[&HOME],
    notes: "",
};
pub const TABLE: Table = Table {
    binary: "temper-qa",
    line: "QA after landing: main's landings triaged into cases, and builds kept for them.",
    version: temper_version::VERSION,
    commit: temper_version::COMMIT,
    commands: &[RUN, KEEP, NONE, CASE, SHOW, LINES],
    settings: &[],
    variables: &[&HOME, &SESSION],
    more: "temper-qa COMMAND --help prints a command; read temper/docs/design/testing/qa/README.md.",
};

#[must_use]
#[expect(
    clippy::disallowed_methods,
    reason = "the binary's capture: temper-qa's arguments and environment, once (skein/docs/foundation/cli.md, 6)"
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
            let argument = match argument(command, rest) {
                Ok(argument) => argument,
                Err(why) => {
                    eprint!("{}", TABLE.refusal(command, &why));
                    return ExitCode::from(2);
                }
            };
            match run(command, argument, &environment) {
                Ok(text) => {
                    print!("{text}");
                    ExitCode::SUCCESS
                }
                Err(why) => {
                    eprintln!("temper-qa: {why}");
                    ExitCode::from(1)
                }
            }
        }
    }
}
fn argument<'a>(command: &Command, rest: &'a [Box<[u8]>]) -> Result<&'a [Box<[u8]>], String> {
    if command == &KEEP {
        return match rest {
            [commit, directory] if !commit.starts_with(b"--") && !directory.starts_with(b"--") => Ok(rest),
            [] => Err("COMMIT and DIR are not given".into()),
            [_] => Err("DIR is not given".into()),
            _ => Err("expected COMMIT and DIR".into()),
        };
    }
    if command == &NONE || command == &CASE {
        match rest {
            [value] if !value.starts_with(b"--") => Ok(rest),
            [] => Err(format!("{} is not given", if command == &NONE { "WHY" } else { "FILE" })),
            _ => Err("expected one argument".into()),
        }
    } else if rest.is_empty() {
        Ok(rest)
    } else {
        Err("no arguments are taken".into())
    }
}
fn run(command: &Command, arguments: &[Box<[u8]>], environment: &Environment) -> Result<String, Refusal> {
    let session = if command == &NONE || command == &CASE {
        let value = SESSION
            .value(environment)
            .ok_or_else(|| refused("TEMPER_INTERNAL_QA_SESSION is not set: triage commands run in a triage session"))?;
        Some(
            std::str::from_utf8(value)
                .ok()
                .and_then(|value| value.parse::<u64>().ok())
                .ok_or_else(|| refused("TEMPER_INTERNAL_QA_SESSION is not a session number"))?,
        )
    } else {
        None
    };
    let directory = HOME.value(environment).ok_or_else(|| refused("TEMPER_INTERNAL_QA_HOME is not set"))?;
    let directory = Path::new(OsStr::from_bytes(directory));
    // Session commands run from another working directory, so the home placed
    // in their environment must not depend on this command's working directory.
    let absolute = if command == &RUN {
        Some(std::fs::canonicalize(directory).map_err(|why| refused(format!("{}: {why}", directory.display())))?)
    } else {
        None
    };
    let home = Home::open(absolute.as_deref().unwrap_or(directory))?;
    if command == &RUN {
        let environment = environment.to_vec();
        let git = crate::faces::RealGit {
            directory: home.configuration.git.directory.clone(),
            environment: environment.clone(),
        };
        let processes = crate::faces::RealProcesses::new(environment.clone());
        crate::runner::Run::start(home, git, processes, crate::faces::RealClock::default(), environment)?.serve()?;
        return Ok(String::new());
    }
    if command == &SHOW {
        return home.read(|ledger| Ok(ledger.show()));
    }
    if command == &LINES {
        return home.read(|ledger| Ok(ledger.lines_text()));
    }
    if command == &KEEP {
        let [commit, directory] = arguments else { unreachable!("checked keep arguments") };
        let commit = std::str::from_utf8(commit)
            .map_err(|_| refused(format!("{} is no commit id", String::from_utf8_lossy(commit))))?;
        return crate::build::keep(&home, commit, Path::new(OsStr::from_bytes(directory)), environment);
    }
    let session = session.expect("triage command");
    let argument = arguments.first().expect("triage argument").as_ref();
    home.transact(|ledger| {
        ledger.command_session(session)?;
        if command == &NONE {
            let why = std::str::from_utf8(argument).map_err(|_| refused("no QA why must be UTF-8"))?;
            let landing = ledger.answer_none(session, why)?;
            return Ok(format!("landing {landing}: no QA\n"));
        }
        let file = String::from_utf8_lossy(argument);
        let bytes = if argument == b"-" {
            let mut bytes = Vec::new();
            std::io::stdin().read_to_end(&mut bytes).map_err(|why| refused(format!("case refused: {file}: {why}")))?;
            bytes
        } else {
            std::fs::read(Path::new(OsStr::from_bytes(argument)))
                .map_err(|why| refused(format!("case refused: {file}: {why}")))?
        };
        let input = CaseInput::decode(&bytes, &file)?;
        let now = skein_shell::Clock::new().now().wall.as_secs();
        Ok(format!("{}\n", ledger.add_case(session, &input, now)?))
    })
}
