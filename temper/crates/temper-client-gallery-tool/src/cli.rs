//! The gallery's table and entry (temper/docs/design/testing/tools.md, 3 and 4).
use skein_shell::{Argument, Command, Front, Reach, Set, Table};
use std::{ffi::OsStr, os::unix::ffi::OsStrExt, path::Path, process::ExitCode};

const PRINT: Command = Command {
    reach: Reach::Words(&[]),
    set: Set::Internal,
    forms: &["OUT"],
    line: "Print the browser's pages in named states into a directory.",
    arguments: &[Argument {
        shown: "OUT",
        line: "The directory to replace whole if it holds the gallery's index.html, or create if missing.",
    }],
    settings: &[],
    switches: &[],
    variables: &[],
    notes: "",
};

/// The gallery's internal command (temper/docs/design/testing/tools.md, 4).
pub const TABLE: Table = Table {
    binary: "temper-client-gallery",
    line: "the browser's pages in named states, printed for screenshots.",
    version: temper_version::VERSION,
    commit: temper_version::COMMIT,
    commands: &[PRINT],
    settings: &[],
    variables: &[],
    more: "Read temper/docs/design/client/web/gallery.md, section 1.",
};

/// Answer the table, then print the gallery (temper/docs/design/testing/tools.md, 4.2).
#[must_use]
pub fn main() -> ExitCode {
    #[expect(
        clippy::disallowed_methods,
        reason = "the binary's capture: temper-client-gallery's arguments, once (skein/docs/foundation/cli.md, 6)"
    )]
    let arguments = skein_shell::arguments();
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
            let out = match rest {
                [] => return refuse(command, "OUT is not given"),
                [out] if out.starts_with(b"-") => {
                    return refuse(command, &format!("no flag {}", String::from_utf8_lossy(out)));
                }
                [out] => out,
                _ => return refuse(command, "expected one OUT"),
            };
            if let Err(why) = crate::print(Path::new(OsStr::from_bytes(out))) {
                eprintln!("temper-client-gallery: {why:?}");
                return ExitCode::FAILURE;
            }
            for sample in crate::samples::samples() {
                println!("{}", sample.name);
            }
            ExitCode::SUCCESS
        }
    }
}

fn refuse(command: &Command, why: &str) -> ExitCode {
    eprint!("{}", TABLE.refusal(command, why));
    ExitCode::from(2)
}
