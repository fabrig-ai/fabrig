//! The table and command boundary (testing/tools.md, 4; qa/README.md, 2).
use crate::{CaseInput, Home, Refusal, refused};
use skein_shell::{Argument, Command, Front, Reach, Set, Table, Variable, Way};
use std::{ffi::OsStr, io::Read, os::unix::ffi::OsStrExt, path::Path, process::ExitCode};

type Environment = [(Box<[u8]>, Box<[u8]>)];

pub const HOME: Variable = Variable {
    name: "TEMPER_QA_HOME",
    set: Set::Internal,
    form: "DIR",
    line: "The deployment's QA home, holding qa.json and ledger.json.",
    absent: "Required by every command but the front's.",
    notes: "",
    way: Way::Reads,
};
pub const SESSION: Variable = Variable {
    name: "TEMPER_QA_SESSION",
    set: Set::Internal,
    form: "N",
    line: "The running triage session the agent's commands record for.",
    absent: "Triage commands are refused.",
    notes: "Set by the loop for the agent it starts.",
    way: Way::Reads,
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
    commands: &[RUN, NONE, CASE, SHOW, LINES],
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
fn argument<'a>(command: &Command, rest: &'a [Box<[u8]>]) -> Result<Option<&'a [u8]>, String> {
    if command == &NONE || command == &CASE {
        match rest {
            [value] if !value.starts_with(b"--") => Ok(Some(value)),
            [] => Err(format!("{} is not given", if command == &NONE { "WHY" } else { "FILE" })),
            _ => Err("expected one argument".into()),
        }
    } else if rest.is_empty() {
        Ok(None)
    } else {
        Err("no arguments are taken".into())
    }
}
fn run(command: &Command, argument: Option<&[u8]>, environment: &Environment) -> Result<String, Refusal> {
    let session = if command == &NONE || command == &CASE {
        let value = SESSION
            .value(environment)
            .ok_or_else(|| refused("TEMPER_QA_SESSION is not set: triage commands run in a triage session"))?;
        Some(
            std::str::from_utf8(value)
                .ok()
                .and_then(|value| value.parse::<u64>().ok())
                .ok_or_else(|| refused("TEMPER_QA_SESSION is not a session number"))?,
        )
    } else {
        None
    };
    let directory = HOME.value(environment).ok_or_else(|| refused("TEMPER_QA_HOME is not set"))?;
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
    let session = session.expect("triage command");
    let argument = argument.expect("triage argument");
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
