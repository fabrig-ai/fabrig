//! The journeys table without a process (temper/docs/design/testing/tools.md, 6).
use super::{Memory, words};
use crate::{TABLE, run};
use skein_shell::{Front, Table};

const FIXED: Table = Table { version: "0.0.0", commit: "0123456789ab", ..TABLE };

fn empty(text: &str) -> bool {
    text.is_empty()
}

fn opening(text: &str) -> String {
    text.split("\n\n").next().unwrap_or_default().split_whitespace().collect::<Vec<_>>().join(" ")
}

fn help(text: &str) -> bool {
    opening(text) == "temper-client-journeys: recorded walks, replayed through the shipped hosts and printed."
        && !text.split_whitespace().any(|word| ["add", "normalise", "again", "pages", "reprint"].contains(&word))
        && text
            .split_once("environment:\n")
            .is_some_and(|(_, environment)| environment.contains("PATH") && !environment.contains("CARGO_TARGET_DIR"))
}

fn internal(text: &str) -> bool {
    opening(text) == "internal: not part of temper-client-journeys's public command line; it may change without notice."
}

fn print_help(text: &str) -> bool {
    let words = text.split_whitespace().collect::<Vec<_>>().join(" ");
    internal(text)
        && words.contains("These aren't all a secret could be: any other private text stands unless --redact names it, so read the pages before you commit them.")
        && words.contains("FILE keeps every original: keep it out of the repository and remove it once printed.")
}

fn journey_walk_help(text: &str) -> bool {
    let words = text.split_whitespace().collect::<Vec<_>>().join(" ");
    internal(text) && words.contains("temper-client-journeys journey walk NAME --bin DIR [--bundle DIR] --scratch DIR [--strict] [--ports LIST] [--widths LIST] [--deployment DIR]")
}
fn drive_help(text: &str) -> bool {
    internal(text) && text.contains("--journey") && text.contains("--widths LIST")
}

fn version(text: &str) -> bool {
    text == "temper-client-journeys 0.0.0 (0123456789ab)\n"
}

type Check = fn(&str) -> bool;
type Case = (&'static [&'static str], i32, Check, &'static str);
const ROWS: &[Case] = &[
    (&["--help"], 0, help, ""),
    (&["-h"], 0, help, ""),
    (&["help"], 0, help, ""),
    (&["--version"], 0, version, ""),
    (&["print", "--help"], 0, print_help, ""),
    (&["journey", "walk", "--help"], 0, journey_walk_help, ""),
    (&["drive", "--help"], 0, drive_help, ""),
    (&["add"], 2, empty, "temper-client-journeys: no command add; temper-client-journeys --help says more\n"),
    (
        &["normalise"],
        2,
        empty,
        "temper-client-journeys: no command normalise; temper-client-journeys --help says more\n",
    ),
    (
        &["print", "FILE"],
        2,
        empty,
        "temper-client-journeys print: expected FILE and OUT\nusage:\n  temper-client-journeys print [--redact TEXT]... FILE OUT\n",
    ),
    (
        &["print", "FILE", "OUT", "extra"],
        2,
        empty,
        "temper-client-journeys print: expected FILE and OUT\nusage:\n  temper-client-journeys print [--redact TEXT]... FILE OUT\n",
    ),
    (&[], 2, empty, "temper-client-journeys: no command; temper-client-journeys --help says more\n"),
    (&["again"], 2, empty, "temper-client-journeys: no command again; temper-client-journeys --help says more\n"),
    (
        &["again", "NAME"],
        2,
        empty,
        "temper-client-journeys: no command again; temper-client-journeys --help says more\n",
    ),
    (
        &["again", "--help"],
        2,
        empty,
        "temper-client-journeys: no command again; temper-client-journeys --help says more\n",
    ),
    (&["pages"], 2, empty, "temper-client-journeys: no command pages; temper-client-journeys --help says more\n"),
    (
        &["pages", "--check"],
        2,
        empty,
        "temper-client-journeys: no command pages; temper-client-journeys --help says more\n",
    ),
    (
        &["pages", "--help"],
        2,
        empty,
        "temper-client-journeys: no command pages; temper-client-journeys --help says more\n",
    ),
    (&["nope"], 2, empty, "temper-client-journeys: no command nope; temper-client-journeys --help says more\n"),
];

#[test]
fn the_binary_answers_its_table_without_starting_children() {
    let mut failures = Vec::new();
    let mut page = None;
    for (arguments, code, stdout, stderr) in ROWS {
        let words = words(arguments);
        let (exit, out, err) = match FIXED.front(&words) {
            Front::Print(text) => (0, text, String::new()),
            Front::Refuse(text) => (2, String::new(), text),
            Front::Run { command, rest } => {
                assert_eq!(*command, crate::table::PRINT);
                let mut output = Memory::default();
                let answer = run::print_command(
                    rest,
                    &super::build(),
                    |_| panic!("usage does not resolve OUT"),
                    |_| panic!("usage does not read FILE"),
                    &mut output,
                );
                assert!(output.nodes.is_empty() && output.changed_paths.is_empty(), "usage writes nothing");
                (i32::from(answer.exit), answer.stdout, answer.stderr)
            }
        };
        if exit != *code || !stdout(&out) || err != *stderr {
            failures.push(format!(
                "{arguments:?}: exit {exit}, stdout {out:?}, stderr {err:?}; expected exit {code}, stderr {stderr:?}"
            ));
        }
        if [Some(&"--help"), Some(&"-h"), Some(&"help")].contains(&arguments.first()) {
            if let Some(expected) = &page {
                assert_eq!(&out, expected, "all three help forms print the same page");
            } else {
                page = Some(out);
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn help_says_what_is_left_to_the_person() {
    let Front::Print(text) = FIXED.front(&words(&["--help"])) else { panic!("bare help prints") };
    assert!(help(&text), "the bare page lists only public entries");
    let Front::Print(text) = FIXED.front(&words(&["print", "--help"])) else { panic!("print help prints") };
    let help = text.split_whitespace().collect::<Vec<_>>().join(" ");
    for sentence in [
        "temper-client-journeys print [--redact TEXT]... FILE OUT",
        "These aren't all a secret could be: any other private text stands unless --redact names it, so read the pages before you commit them.",
        "FILE keeps every original: keep it out of the repository and remove it once printed.",
        "link",
        "stored",
        "typed",
        "--redact",
        "key",
        "JWT",
        "bearer",
        "code",
        "refresh token",
    ] {
        assert!(help.contains(sentence), "help includes the promised fact");
    }
}
