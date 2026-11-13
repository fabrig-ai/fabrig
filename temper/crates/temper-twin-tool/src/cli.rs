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
    forms: &[
        "--data DIR --base FILE --out FILE [--cue TASK=PHRASE]... [--replace OLD=NEW]... [--takes FILE]",
        "--from FILE --out FILE [--replace OLD=NEW]... [--takes FILE]",
    ],
    line: "Make a fake LLM document from stopped transcripts or transform an existing twin.",
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
            fallback: Fallback::Text("none with --from"),
            set: Set::Internal,
            twin: None,
            notes: "Required without --from.",
        },
        Setting {
            name: "base",
            value: "FILE",
            about: "The fake LLM document whose script lists are replaced.",
            fallback: Fallback::Text("none with --from"),
            set: Set::Internal,
            twin: None,
            notes: "Required without --from.",
        },
        Setting {
            name: "from",
            value: "FILE",
            about: "Transform an existing twin instead of transcripts.",
            fallback: Fallback::Text("none"),
            set: Set::Internal,
            twin: None,
            notes: "Cannot be combined with --data, --base or --cue.",
        },
        Setting {
            name: "takes",
            value: "FILE",
            about: "Take recorded values from the replay's conversation.",
            fallback: Fallback::Text("none"),
            set: Set::Internal,
            twin: None,
            notes: "A JSON list of at most 8 distinct named takes; applied after --replace.",
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

enum Source {
    Transcripts { data: PathBuf, base: PathBuf, cues: Box<[Take]> },
    Document(PathBuf),
}
struct Options {
    source: Source,
    values: Option<PathBuf>,
    out: PathBuf,
    replaces: Box<[Replace]>,
}

fn parse(arguments: &[Box<[u8]>]) -> Result<Options, String> {
    let mut data = None;
    let mut base = None;
    let mut out = None;
    let mut from = None;
    let mut values = None;
    let mut takes = Vec::new();
    let mut replaces = Vec::new();
    let mut args = arguments.iter();
    while let Some(arg) = args.next() {
        if ![b"--data".as_slice(), b"--base", b"--out", b"--cue", b"--replace", b"--from", b"--takes"]
            .contains(&arg.as_ref())
        {
            return Err(format!("unknown option: {}", String::from_utf8_lossy(arg).escape_debug()));
        }
        let value = args.next().ok_or_else(|| format!("{} takes a value", String::from_utf8_lossy(arg)))?;
        match arg.as_ref() {
            b"--data" if data.is_none() => data = Some(PathBuf::from(OsStr::from_bytes(value))),
            b"--base" if base.is_none() => base = Some(PathBuf::from(OsStr::from_bytes(value))),
            b"--out" if out.is_none() => out = Some(PathBuf::from(OsStr::from_bytes(value))),
            b"--from" if from.is_none() => from = Some(PathBuf::from(OsStr::from_bytes(value))),
            b"--takes" if values.is_none() => values = Some(PathBuf::from(OsStr::from_bytes(value))),
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
    let source = if let Some(from) = from {
        if data.is_some() || base.is_some() || !takes.is_empty() {
            return Err("--from cannot be combined with --data, --base or --cue".into());
        }
        Source::Document(from)
    } else {
        Source::Transcripts {
            data: data.ok_or("--data is not given")?,
            base: base.ok_or("--base is not given")?,
            cues: takes.into_boxed_slice(),
        }
    };
    Ok(Options { source, values, out: out.ok_or("--out is not given")?, replaces: replaces.into_boxed_slice() })
}

fn convert(options: Options) -> Result<(), String> {
    let Options { source, values, out, replaces } = options;
    let values = match values {
        Some(path) => crate::read_takes(&read(&path)?).map_err(|why| line(&why))?,
        None => Box::new([]),
    };
    let result = match source {
        Source::Document(path) => crate::from_document(&read(&path)?, &replaces, &values),
        Source::Transcripts { data, base, cues } => crate::twin(&Input {
            data,
            base: read(&base)?.into_boxed_slice(),
            takes: cues,
            replaces: replaces.iter().map(|r| Replace { old: r.old.clone(), new: r.new.clone() }).collect(),
            values: values.clone(),
        }),
    };
    let twin = result.map_err(|why| line(&why))?;
    if std::fs::write(&out, &twin.document).is_err() {
        return Err(format!("write: {}", out.to_string_lossy().escape_debug()));
    }
    for (task, turns) in &twin.report.turns {
        println!("task {task}: {turns} turns");
    }
    for (task, blocks) in &twin.report.dropped {
        println!("task {task}: {blocks} opaque blocks dropped");
    }
    for (replace, count) in replaces.iter().zip(&twin.report.replaced) {
        println!("{}: {count}", String::from_utf8_lossy(&replace.old).escape_debug());
    }
    for (take, count) in values.iter().zip(&twin.report.taken) {
        println!("take {}: {count} uses", take.name);
    }
    Ok(())
}
fn read(path: &PathBuf) -> Result<Vec<u8>, String> {
    std::fs::read(path).map_err(|_| format!("read: {}", path.to_string_lossy().escape_debug()))
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
        Refusal::Takes { why } => format!("takes: {why}"),
        Refusal::UnusedTake { name } => format!("take {name}: no turn uses it"),
    }
}
