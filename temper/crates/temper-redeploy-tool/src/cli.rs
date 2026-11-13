//! The tool's table and composition (temper/docs/design/redeploy.md, 7; testing/tools.md, 4).
use crate::{
    description::{self, Description},
    hosts::{Environment, Hosts, Processes},
    record::{Journal, Record},
    redeploy::{self, Mode, Outcome},
    referee, stage,
};
use skein_shell::{Argument, Command, Fallback, Front, Reach, Set, Setting, Settings, Switch, Table};
use std::{
    ffi::OsStr,
    fs::{self, OpenOptions},
    io::Write,
    os::unix::ffi::OsStrExt,
    path::PathBuf,
    process::ExitCode,
};

const DESCRIPTION: Argument = Argument { shown: "DESCRIPTION", line: "The deployment's JSON description." };
const COMMIT: Argument = Argument { shown: "COMMIT", line: "Twelve lowercase hexadecimal digits naming the build." };
const RECORD: Settings = Settings {
    tool: "temper-redeploy",
    about: "The redeploy's record.",
    table: &[Setting {
        name: "record",
        value: "FILE",
        about: "The new redeploy's JSON record; refused if it exists.",
        fallback: Fallback::Refuse,
        set: Set::Internal,
        twin: None,
        notes: "One record per redeploy, never overwritten.",
    }],
};
const STAGE: Command = Command {
    reach: Reach::Words(&["stage"]),
    set: Set::Internal,
    forms: &["DESCRIPTION COMMIT"],
    line: "Stage each part's binaries without stopping or swapping anything.",
    arguments: &[DESCRIPTION, COMMIT],
    settings: &[],
    switches: &[],
    variables: &[],
    notes: "A staged build answering COMMIT is left as it is.",
};
const RUN: Command = Command {
    reach: Reach::Words(&["run"]),
    set: Set::Internal,
    forms: &["DESCRIPTION COMMIT --record FILE [--workers | --together]"],
    line: "Stage, snapshot, redeploy and referee, writing a record throughout.",
    arguments: &[DESCRIPTION, COMMIT],
    settings: &[&RECORD],
    switches: &[
        Switch {
            name: "workers",
            set: Set::Internal,
            line: "Restart workers in turn after the engine.",
            notes: "Excludes --together.",
        },
        Switch {
            name: "together",
            set: Set::Internal,
            line: "Restart every part together for a breaking change.",
            notes: "Excludes --workers.",
        },
    ],
    variables: &[],
    notes: "The engine alone by default. Exit 0 only when done and the referee passes; otherwise 1.",
};
const REFEREE: Command = Command {
    reach: Reach::Words(&["referee"]),
    set: Set::Internal,
    forms: &["DESCRIPTION RECORD"],
    line: "Read a record and referee against fresh tasks, without writing the record.",
    arguments: &[DESCRIPTION, Argument { shown: "RECORD", line: "A redeploy's JSON record." }],
    settings: &[],
    switches: &[],
    variables: &[],
    notes: "Exit 0 on pass, 1 on fail.",
};
pub const TABLE: Table = Table {
    binary: "temper-redeploy",
    line: "a deployment's redeploy: the engine alone by default, workers in turn, every part together across a breaking change, and its referee.",
    version: "",
    commit: "",
    commands: &[STAGE, RUN, REFEREE],
    settings: &[],
    variables: &[],
    more: "temper-redeploy COMMAND --help prints a command; read temper/docs/design/redeploy.md.",
};

#[derive(Clone, Debug)]
pub enum Action {
    Stage { commit: String },
    Run { commit: String, mode: Mode, record: PathBuf },
    Referee { record: PathBuf },
}
impl Action {
    fn name(&self) -> &str {
        match self {
            Self::Stage { .. } => "stage",
            Self::Run { .. } => "run",
            Self::Referee { .. } => "referee",
        }
    }
}
/// This is the CLI's execution entry, with every host and clock replaced by `Hosts`.
pub fn execute(
    description: &Description,
    action: &Action,
    hosts: &mut dyn Hosts,
    output: Box<dyn Write>,
) -> Result<u8, String> {
    execute_with_steps(
        description,
        action,
        hosts,
        output,
        Steps { before: referee::before, redeploy: redeploy::redeploy, judge: referee::judge },
    )
}
/// The three module seams, so composition can be checked independently of their bodies.
#[doc(hidden)]
#[derive(Clone, Copy)]
pub struct Steps {
    pub before: fn(&Description, &mut dyn Hosts) -> Result<crate::record::Before, String>,
    pub redeploy: fn(&Description, &str, Mode, &mut Journal, &mut dyn Hosts) -> Outcome,
    pub judge: fn(&Description, &mut Journal, &mut dyn Hosts) -> bool,
}
#[doc(hidden)]
pub fn execute_with_steps(
    description: &Description,
    action: &Action,
    hosts: &mut dyn Hosts,
    output: Box<dyn Write>,
    steps: Steps,
) -> Result<u8, String> {
    match action {
        Action::Stage { commit } => {
            let mut journal = Journal::new(Record::new(commit, Mode::Engine), None, output);
            stage::stage(description, commit, &mut journal, hosts)?;
            Ok(0)
        }
        Action::Run { commit, mode, record } => {
            // Reserve before any host call, including staging; create_new also refuses symlinks.
            drop(
                OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(record)
                    .map_err(|why| format!("record {}: {why}", record.display()))?,
            );
            let mut journal = Journal::new(Record::new(commit, *mode), Some(record.clone()), output);
            journal.save()?;
            if let Err(why) = stage::stage(description, commit, &mut journal, hosts) {
                return refuse(&mut journal, hosts, &why);
            }
            match (steps.before)(description, hosts) {
                Ok(before) => journal.record.before = before,
                Err(why) => return refuse(&mut journal, hosts, &why),
            }
            journal.record.began = hosts.now();
            journal.save()?;
            let outcome = (steps.redeploy)(description, commit, *mode, &mut journal, hosts);
            journal.record.outcome = Some(outcome);
            journal.record.ended = Some(hosts.now());
            journal.save()?;
            let passed = (steps.judge)(description, &mut journal, hosts);
            journal.save()?;
            Ok(u8::from(outcome != Outcome::Done || !passed))
        }
        Action::Referee { record } => {
            let text = fs::read_to_string(record).map_err(|why| format!("record {}: {why}", record.display()))?;
            let record = serde_json::from_str(&text).map_err(|why| format!("record: {why}"))?;
            let mut journal = Journal::new(record, None, output);
            Ok(u8::from(!(steps.judge)(description, &mut journal, hosts)))
        }
    }
}
fn refuse(journal: &mut Journal, hosts: &mut dyn Hosts, why: &str) -> Result<u8, String> {
    journal.record.outcome = Some(Outcome::Refused);
    journal.record.ended = Some(hosts.now());
    for line in why.lines() {
        journal.line(hosts.now(), &format!("temper-redeploy run: refused: {line}"))?;
    }
    Ok(1)
}
fn parse(command: &Command, rest: &[Box<[u8]>]) -> Result<(PathBuf, Action), String> {
    let mut workers = false;
    let mut together = false;
    let resolved;
    let (rest, record) = if command == &RUN {
        resolved = skein_shell::resolve(&RECORD, rest, &[]).map_err(|why| why.to_string())?;
        (resolved.rest(), Some(PathBuf::from(OsStr::from_bytes(&resolved.value(0).bytes))))
    } else {
        (rest, None)
    };
    let mut positional = Vec::new();
    for word in rest {
        match word.as_ref() {
            b"--workers" if command == &RUN && !workers => workers = true,
            b"--together" if command == &RUN && !together => together = true,
            word if word.starts_with(b"--") => {
                return Err(format!("unexpected argument {}", String::from_utf8_lossy(word)));
            }
            _ => positional.push(word),
        }
    }
    if workers && together {
        return Err("--workers and --together exclude each other".into());
    }
    let [description, second] = positional.as_slice() else {
        return Err("expected DESCRIPTION and COMMIT or RECORD".into());
    };
    let description = PathBuf::from(OsStr::from_bytes(description));
    if command == &REFEREE {
        return Ok((description, Action::Referee { record: PathBuf::from(OsStr::from_bytes(second)) }));
    }
    let commit = std::str::from_utf8(second).map_err(|_| "COMMIT must be twelve lowercase hexadecimal digits")?;
    if !stage::valid_commit(commit) {
        return Err("COMMIT must be twelve lowercase hexadecimal digits".into());
    }
    let commit = commit.to_string();
    let action = if let Some(record) = record {
        Action::Run {
            commit,
            record,
            mode: if workers {
                Mode::Workers
            } else if together {
                Mode::Together
            } else {
                Mode::Engine
            },
        }
    } else {
        Action::Stage { commit }
    };
    Ok((description, action))
}
#[expect(
    clippy::disallowed_methods,
    reason = "the binary's capture: redeploy's arguments and environment, once (skein/docs/foundation/cli.md, 6)"
)]
pub fn main(version: &'static str, commit: &'static str) -> ExitCode {
    let arguments = skein_shell::arguments();
    let environment = skein_shell::environment();
    front(version, commit, &arguments, &environment)
}
fn front(version: &'static str, commit: &'static str, arguments: &[Box<[u8]>], environment: &Environment) -> ExitCode {
    match (Table { version, commit, ..TABLE }).front(arguments) {
        Front::Print(text) => {
            print!("{text}");
            ExitCode::SUCCESS
        }
        Front::Refuse(text) => {
            eprint!("{text}");
            ExitCode::from(2)
        }
        Front::Run { command, rest } => {
            let (path, action) = match parse(command, rest) {
                Ok(parsed) => parsed,
                Err(why) => {
                    eprint!("{}", TABLE.refusal(command, &why));
                    return ExitCode::from(2);
                }
            };
            let result = fs::read_to_string(&path)
                .map_err(|why| format!("description {}: {why}", path.display()))
                .and_then(|text| description::read(&text))
                .and_then(|description| {
                    let mut hosts = Processes::new(&description, environment);
                    execute(&description, &action, &mut hosts, Box::new(std::io::stdout()))
                });
            match result {
                Ok(code) => ExitCode::from(code),
                Err(why) => {
                    for line in why.lines() {
                        eprintln!("temper-redeploy {}: {line}", action.name());
                    }
                    ExitCode::from(1)
                }
            }
        }
    }
}
