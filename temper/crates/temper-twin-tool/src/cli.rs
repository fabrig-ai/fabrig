//! The twin's table and run (temper/docs/design/testing/tools.md, 4).
use crate::{Input, Refusal, Replace, Take};
use skein_shell::{Command, Fallback, Front, Reach, Set, Setting, Settings, Table};
use std::{ffi::OsStr, os::unix::ffi::OsStrExt, path::PathBuf, process::ExitCode};

pub const TABLE: Table = Table {
    binary: "temper-twin",
    line: "a live run's twin: a fake LLM document that replays its agents.",
    version: "",
    commit: "",
    commands: &[TWIN],
    settings: &[],
    variables: &[],
    more: "Read temper/docs/design/testing/fakes.md, section 1.",
};

const TWIN: Command = Command {
    reach: Reach::Words(&[]),
    set: Set::Internal,
    forms: &["--data DIR --base FILE --out FILE [--cue TASK=PHRASE]... [--replace OLD=NEW]..."],
    line: "Turn a cleanly stopped deployment's selected transcripts into a fake LLM document.",
    arguments: &[],
    settings: &[&FLAGS],
    switches: &[],
    variables: &[],
    notes: "Read transcripts only after a successful drained stop. No output is opened on a conversion refusal.",
};

const FLAGS: Settings = Settings {
    tool: "temper-twin",
    about: "The transcripts, base document, output and selections.",
    table: &[
        Setting {
            name: "data",
            value: "DIR",
            about: "The deployment directory holding transcripts.",
            fallback: Fallback::Refuse,
            set: Set::Internal,
            twin: None,
            notes: "",
        },
        Setting {
            name: "base",
            value: "FILE",
            about: "The fake LLM document whose script lists are replaced.",
            fallback: Fallback::Refuse,
            set: Set::Internal,
            twin: None,
            notes: "",
        },
        Setting {
            name: "out",
            value: "FILE",
            about: "Write the validated twin here.",
            fallback: Fallback::Refuse,
            set: Set::Internal,
            twin: None,
            notes: "",
        },
        Setting {
            name: "cue",
            value: "TASK=PHRASE",
            about: "Select a task's completions by its opening cue.",
            fallback: Fallback::Text("none"),
            set: Set::Internal,
            twin: None,
            notes: "Repeats. Each cue must be distinct.",
        },
        Setting {
            name: "replace",
            value: "OLD=NEW",
            about: "Replace literal bytes in assistant text and call arguments.",
            fallback: Fallback::Text("none"),
            set: Set::Internal,
            twin: None,
            notes: "Repeats. Replacements apply in option order.",
        },
    ],
};

#[must_use]
#[expect(
    clippy::disallowed_methods,
    reason = "the binary's capture: temper-twin's arguments, once (skein/docs/foundation/cli.md, 6)"
)]
pub fn main(version: &'static str, commit: &'static str) -> ExitCode {
    run(&skein_shell::arguments(), &Table { version, commit, ..TABLE })
}

fn run(arguments: &[Box<[u8]>], table: &Table) -> ExitCode {
    match table.front(arguments) {
        Front::Print(text) => {
            print!("{text}");
            ExitCode::SUCCESS
        }
        Front::Refuse(text) => {
            eprint!("{text}");
            ExitCode::from(2)
        }
        Front::Run { command, rest } => {
            debug_assert_eq!(command, &TWIN);
            let input = match parse(rest) {
                Ok(input) => input,
                Err(why) => {
                    eprint!("{}", table.refusal(&TWIN, &why));
                    return ExitCode::from(2);
                }
            };
            match convert(input) {
                Ok(()) => ExitCode::SUCCESS,
                Err(line) => {
                    eprintln!("{line}");
                    ExitCode::from(2)
                }
            }
        }
    }
}

struct Options {
    data: PathBuf,
    base: PathBuf,
    out: PathBuf,
    takes: Box<[Take]>,
    replaces: Box<[Replace]>,
}

fn parse(arguments: &[Box<[u8]>]) -> Result<Options, String> {
    let mut data = None;
    let mut base = None;
    let mut out = None;
    let mut takes = Vec::new();
    let mut replaces = Vec::new();
    let mut args = arguments.iter();
    while let Some(arg) = args.next() {
        if ![b"--data".as_slice(), b"--base", b"--out", b"--cue", b"--replace"].contains(&arg.as_ref()) {
            return Err(format!("unknown option: {}", String::from_utf8_lossy(arg).escape_debug()));
        }
        let value = args.next().ok_or_else(|| format!("{} takes a value", String::from_utf8_lossy(arg)))?;
        match arg.as_ref() {
            b"--data" if data.is_none() => data = Some(PathBuf::from(OsStr::from_bytes(value))),
            b"--base" if base.is_none() => base = Some(PathBuf::from(OsStr::from_bytes(value))),
            b"--out" if out.is_none() => out = Some(PathBuf::from(OsStr::from_bytes(value))),
            b"--cue" => {
                let text = std::str::from_utf8(value).map_err(|_| "cue must be UTF-8")?;
                let (task, cue) = text.split_once('=').ok_or("expected --cue TASK=PHRASE")?;
                let Ok(task) = task.parse() else {
                    return Err(String::from("cue task must be a u64"));
                };
                takes.push(Take { task, cue: cue.as_bytes().into() });
            }
            b"--replace" => {
                let text = std::str::from_utf8(value).map_err(|_| "replace must be UTF-8")?;
                let (old, new) = text.split_once('=').ok_or("expected --replace OLD=NEW")?;
                replaces.push(Replace { old: old.as_bytes().into(), new: new.as_bytes().into() });
            }
            _ => return Err(format!("repeated option: {}", String::from_utf8_lossy(arg).escape_debug())),
        }
    }
    Ok(Options {
        data: data.ok_or("--data is not given")?,
        base: base.ok_or("--base is not given")?,
        out: out.ok_or("--out is not given")?,
        takes: takes.into_boxed_slice(),
        replaces: replaces.into_boxed_slice(),
    })
}

fn convert(options: Options) -> Result<(), String> {
    let Options { data, base, out, takes, replaces } = options;
    let Ok(bytes) = std::fs::read(&base) else {
        return Err(format!("read: {}", base.to_string_lossy().escape_debug()));
    };
    let input = Input { data, base: bytes.into_boxed_slice(), takes, replaces };
    let twin = match crate::twin(&input) {
        Ok(twin) => twin,
        Err(refusal) => return Err(line(&refusal)),
    };
    if std::fs::write(&out, &twin.document).is_err() {
        return Err(format!("write: {}", out.to_string_lossy().escape_debug()));
    }
    for (task, turns) in &twin.report.turns {
        println!("task {task}: {turns} turns");
    }
    for (task, blocks) in &twin.report.dropped {
        println!("task {task}: {blocks} opaque blocks dropped");
    }
    for (replace, count) in input.replaces.iter().zip(&twin.report.replaced) {
        println!("{}: {count}", String::from_utf8_lossy(&replace.old).escape_debug());
    }
    Ok(())
}
fn line(refusal: &Refusal) -> String {
    match refusal {
        Refusal::Read { path } => format!("read transcript: {}", path.to_string_lossy().escape_debug()),
        Refusal::Frame { task, at } => format!("task {task}: bad or uncommitted frame at {at}"),
        Refusal::Version { task, version } => format!("task {task}: unsupported version {version}"),
        Refusal::Turn { task, turn, why } => format!("task {task} turn {turn}: {why:?}"),
        Refusal::SameCue { cue } => format!("same cue: {:?}", String::from_utf8_lossy(cue)),
        Refusal::Bounds { what } => format!("past bounds: {what}"),
        Refusal::Base => String::from("base: invalid fake LLM document"),
    }
}
