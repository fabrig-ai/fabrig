//! Stage one's internal table and front (temper/docs/design/testing/tools.md, 4).
use skein_shell::{Argument, Command, Fallback, Front, Reach, Set, Setting, Settings, Switch, Table};
use std::process::ExitCode;

const JSON: Switch = Switch { name: "json", set: Set::Internal, line: "Print JSON instead of text.", notes: "" };
const EVIDENCE: Argument = Argument { shown: "EVIDENCE", line: "The collected evidence directory." };

pub(crate) const OBLIGATIONS: Command = Command {
    reach: Reach::Words(&["obligations"]),
    set: Set::Internal,
    forms: &["[--json]"],
    line: "Print the obligations, or the whole golden as JSON.",
    arguments: &[],
    settings: &[],
    switches: &[JSON],
    variables: &[],
    notes: "",
};

const COLLECT_SETTINGS: Settings = Settings {
    tool: "temper-stage-one",
    about: "The ctl reader and the accepted proposal's identifiers.",
    table: &[
        Setting {
            name: "temper",
            value: "PATH",
            about: "The temper binary used for ctl reads.",
            fallback: Fallback::Refuse,
            set: Set::Internal,
            twin: None,
            notes: "",
        },
        Setting {
            name: "ctl",
            value: "DIR",
            about: "The ctl directory of the box.",
            fallback: Fallback::Refuse,
            set: Set::Internal,
            twin: None,
            notes: "",
        },
        Setting {
            name: "project",
            value: "NUMBER",
            about: "The project's positive u32 number.",
            fallback: Fallback::Refuse,
            set: Set::Internal,
            twin: None,
            notes: "",
        },
        Setting {
            name: "revision",
            value: "NUMBER",
            about: "The accepted revision's positive u32 number.",
            fallback: Fallback::Refuse,
            set: Set::Internal,
            twin: None,
            notes: "",
        },
        Setting {
            name: "proposal",
            value: "TASK/NUMBER",
            about: "The accepted proposal.",
            fallback: Fallback::Refuse,
            set: Set::Internal,
            twin: None,
            notes: "",
        },
        Setting {
            name: "chat",
            value: "NUMBER",
            about: "The chat's positive u32 number.",
            fallback: Fallback::Refuse,
            set: Set::Internal,
            twin: None,
            notes: "",
        },
        Setting {
            name: "forge",
            value: "HOST",
            about: "The box's forge.",
            fallback: Fallback::Refuse,
            set: Set::Internal,
            twin: None,
            notes: "",
        },
        Setting {
            name: "repository",
            value: "OWNER/NAME",
            about: "The project's repository.",
            fallback: Fallback::Refuse,
            set: Set::Internal,
            twin: None,
            notes: "",
        },
    ],
};

pub(crate) const COLLECT: Command = Command {
    reach: Reach::Words(&["collect"]),
    forms: &[
        "--temper PATH --ctl DIR --project NUMBER --revision NUMBER --proposal TASK/NUMBER --chat NUMBER --forge HOST --repository OWNER/NAME EVIDENCE",
    ],
    line: "Collect the accepted revision and current settings through ctl reads.",
    arguments: &[EVIDENCE],
    settings: &[&COLLECT_SETTINGS],
    switches: &[],
    ..OBLIGATIONS
};
pub(crate) const CHECK: Command = Command {
    reach: Reach::Words(&["check"]),
    forms: &["EVIDENCE [--json]"],
    line: "Check the evidence against the golden and print the binding and obligations.",
    arguments: &[EVIDENCE],
    ..OBLIGATIONS
};
pub(crate) const JUDGE: Command = Command {
    reach: Reach::Words(&["judge"]),
    forms: &["EVIDENCE"],
    line: "Print the judge's complete brief for an independent LLM session.",
    switches: &[],
    notes: "The judge reads what code cannot: the rules' reach, the description and what needs a restart.",
    ..CHECK
};
pub(crate) const VERDICT: Command = Command {
    reach: Reach::Words(&["verdict"]),
    line: "Print the stage's verdict from the check and judge.json.",
    ..CHECK
};

/// The stage-one binary's table (temper/docs/design/testing/tools.md, 4).
pub const TABLE: Table = Table {
    binary: "temper-stage-one",
    line: "configuring's stage 1: its golden, check, judge and evidence.",
    version: temper_version::VERSION,
    commit: temper_version::COMMIT,
    commands: &[OBLIGATIONS, COLLECT, CHECK, JUDGE, VERDICT],
    settings: &[],
    variables: &[],
    more: "temper-stage-one COMMAND --help prints a command; read temper/docs/design/testing/checks.md, section 5.",
};

/// Capture once, answer the front, then run a command (temper/docs/design/testing/tools.md, 4.2).
#[must_use]
#[expect(
    clippy::disallowed_methods,
    reason = "the binary's capture: temper-stage-one's arguments, once (skein/docs/foundation/cli.md, 6)"
)]
pub fn main() -> ExitCode {
    let arguments = skein_shell::arguments();
    match TABLE.front(&arguments) {
        Front::Print(text) => {
            print!("{text}");
            ExitCode::SUCCESS
        }
        Front::Refuse(text) => refuse(&text),
        Front::Run { command, rest } => run(command, rest),
    }
}

fn run(command: &Command, rest: &[Box<[u8]>]) -> ExitCode {
    let arguments: Result<Vec<_>, _> = rest.iter().map(|word| String::from_utf8(word.to_vec())).collect();
    let Ok(arguments) = arguments else {
        return refuse(&TABLE.refusal(command, "an argument is not UTF-8"));
    };
    match *command {
        OBLIGATIONS => crate::golden::main(&arguments),
        COLLECT => crate::collect::main(&arguments),
        CHECK => crate::check::main(&arguments),
        JUDGE => crate::judge::main(&arguments),
        VERDICT => crate::judge::verdict_main(&arguments),
        _ => unreachable!("the front runs only a command of TABLE"),
    }
}

pub(crate) fn refuse(text: &str) -> ExitCode {
    eprint!("{text}");
    ExitCode::from(2)
}

pub(crate) fn evidence_arguments<'a>(args: &'a [String], command: &Command) -> Result<(&'a str, bool), String> {
    match args {
        [dir] if !dir.starts_with('-') => Ok((dir, false)),
        [dir, flag] if !dir.starts_with('-') && flag == "--json" && !command.switches.is_empty() => Ok((dir, true)),
        [] => Err(TABLE.refusal(command, "EVIDENCE is not given")),
        _ => Err(TABLE.refusal(command, "invalid arguments")),
    }
}
