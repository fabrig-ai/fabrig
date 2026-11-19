//! The walk tool's table (temper/docs/design/testing/tools.md, 3 and 4).
use skein_shell::{Argument, Command, Fallback, Front, Reach, Set, Setting, Settings, Table};
use std::{
    ffi::OsStr,
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
    process::ExitCode,
};

const OUT: Settings = Settings {
    tool: "temper-client-walk print",
    about: "Where to print.",
    table: &[Setting {
        name: "out",
        value: "DIR",
        about: "Print into DIR, outside either journeys root.",
        fallback: Fallback::Text(""),
        set: Set::Internal,
        twin: None,
        notes: "When absent, a new temper-client-walk- directory under the platform's temporary directory.",
    }],
};
const PRINT: Command = Command {
    reach: Reach::Words(&["print"]),
    set: Set::Internal,
    forms: &["print [--out DIR] [NAME]..."],
    line: "Walk moved browser journeys and print their marks into scratch.",
    arguments: &[Argument {
        shown: "NAME",
        line: "A moved journey; absent, all moved browser journeys in name order.",
    }],
    settings: &[&OUT],
    switches: &[],
    variables: &[],
    notes: "Pages are laid out from the repository root down. The checkout's journey pages are the writer's.",
};
const PAGES: Command = Command {
    reach: Reach::Words(&["pages"]),
    set: Set::Internal,
    forms: &["pages"],
    line: "Print every moved browser journey in place, writing only differing bytes.",
    arguments: &[],
    settings: &[],
    switches: &[],
    variables: &[],
    notes: "The writer's command. A stopped walk leaves its journey's files untouched.",
};

pub const TABLE: Table = Table {
    binary: "temper-client-walk",
    line: "journeys walked in a world, their marks printed as pages.",
    version: "",
    commit: "",
    commands: &[PRINT, PAGES],
    settings: &[],
    variables: &[],
    more: "temper-client-walk COMMAND --help prints a command; read temper/docs/design/client/journey-walks.md, sections 4 and 5.",
};

pub fn main(version: &'static str, commit: &'static str) -> ExitCode {
    #[expect(
        clippy::disallowed_methods,
        reason = "the binary's capture: arguments once (skein/docs/foundation/cli.md, 6)"
    )]
    let arguments = skein_shell::arguments();
    let repository = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../../.."));
    let answer = answer(repository, &arguments, version, commit);
    print!("{}", answer.stdout);
    eprint!("{}", answer.stderr);
    ExitCode::from(answer.code)
}

/// The shipped argument path with the source repository supplied by its caller.
#[must_use]
pub fn answer(
    repository: &Path,
    arguments: &[Box<[u8]>],
    version: &'static str,
    commit: &'static str,
) -> crate::Answer {
    match (Table { version, commit, ..TABLE }).front(arguments) {
        Front::Print(stdout) => crate::Answer { code: 0, stdout, stderr: String::new() },
        Front::Refuse(stderr) => crate::Answer { code: 2, stdout: String::new(), stderr },
        Front::Run { command, rest } => {
            let in_place = command == &PAGES;
            let mut out = None;
            let mut names = Vec::new();
            let mut rest = rest.iter();
            while let Some(argument) = rest.next() {
                if in_place {
                    return refuse(command, "pages takes no arguments");
                }
                if argument.as_ref() == b"--out" {
                    let Some(value) = rest.next() else {
                        return refuse(command, "--out needs DIR");
                    };
                    if out.is_some() {
                        return refuse(command, "--out given twice");
                    }
                    out = Some(PathBuf::from(OsStr::from_bytes(value)));
                } else if argument.starts_with(b"-") {
                    return refuse(command, &format!("no flag {}", String::from_utf8_lossy(argument)));
                } else {
                    let Ok(name) = std::str::from_utf8(argument) else {
                        return refuse(command, "NAME is not UTF-8");
                    };
                    names.push(name.to_owned());
                }
            }
            crate::run(repository, command, &names, out.as_deref(), in_place)
        }
    }
}
fn refuse(command: &Command, why: &str) -> crate::Answer {
    crate::Answer { code: 2, stdout: String::new(), stderr: TABLE.refusal(command, why) }
}
