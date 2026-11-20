//! The internal journeys binary's table (temper/docs/design/testing/tools.md, 6).
mod arguments;
mod refusals;

use std::process::{Command, Stdio};

fn empty(text: &str) -> bool {
    text.is_empty()
}

fn opening(text: &str) -> String {
    text.split("\n\n").next().unwrap_or_default().split_whitespace().collect::<Vec<_>>().join(" ")
}

fn help(text: &str) -> bool {
    opening(text) == "temper-client-journeys: recorded walks, replayed through the shipped hosts and printed."
        && !text.split_whitespace().any(|word| ["add", "normalise", "again", "pages"].contains(&word))
        && text
            .split_once("environment:\n")
            .is_some_and(|(_, environment)| environment.contains("PATH") && environment.contains("CARGO_TARGET_DIR"))
}

fn internal(text: &str) -> bool {
    opening(text) == "internal: not part of temper-client-journeys's public command line; it may change without notice."
}

fn add_help(text: &str) -> bool {
    let words = text.split_whitespace().collect::<Vec<_>>().join(" ");
    internal(text)
        && words.contains("These aren't all a secret could be: any other private text stands unless --redact names it, so read the pages before you commit them.")
        && words.contains("FILE keeps every original: keep it out of the repository and remove it once added.")
}

fn normalise_help(text: &str) -> bool {
    let words = text.split_whitespace().collect::<Vec<_>>().join(" ");
    internal(text)
        && words.contains("temper-client-journeys normalise")
        && words.contains("Normalise every browser and TUI recording, then reprint both directories.")
        && words.contains("Write each changed recording only after its normalised replay holds, naming each written.")
        && words.contains("A stale recording is named with its turn and line kind and left as it was; after writing the others and reprinting, exit 1.")
        && words.contains("Reads no configuration, environment, clock or network.")
}

fn pages_help(text: &str) -> bool {
    let words = text.split_whitespace().collect::<Vec<_>>().join(" ");
    internal(text)
        && words.contains("temper-client-journeys pages [--prepared DIR] [--within DURATION]")
        && words.contains("temper-client-journeys pages --check")
        && words.contains("Digits then s, m or h (90m).")
        && words.contains("Every other word goes to stderr.")
        && words.contains("Walk each stale journey again")
        && words.contains("Directory holding temper-box, fake-claude, temper-client for TUI, and bundle/index.html.")
        && words.contains(
            "Exit 0: every journey printed; 1: a journey left unprinted; 2: usage; 3: a root unreadable or unwritable.",
        )
}

fn journey_walk_help(text: &str) -> bool {
    let words = text.split_whitespace().collect::<Vec<_>>().join(" ");
    internal(text) && words.contains("temper-client-journeys journey walk NAME --bin DIR [--bundle DIR] --scratch DIR [--strict] [--ports LIST] [--widths LIST] [--deployment DIR]")
}
fn drive_help(text: &str) -> bool {
    internal(text) && text.contains("--journey") && text.contains("--widths LIST")
}

fn version(text: &str) -> bool {
    let prefix = format!("temper-client-journeys {} (", env!("CARGO_PKG_VERSION"));
    let Some(commit) = text.strip_prefix(&prefix).and_then(|text| text.strip_suffix(")\n")) else {
        return false;
    };
    let hash = commit.strip_suffix("-dirty").unwrap_or(commit);
    commit == "unknown" || (hash.len() == 12 && hash.bytes().all(|byte| byte.is_ascii_hexdigit()))
}

type Check = fn(&str) -> bool;
type Case = (&'static [&'static str], i32, Check, &'static str);
const ROWS: &[Case] = &[
    (&["--help"], 0, help, ""),
    (&["-h"], 0, help, ""),
    (&["help"], 0, help, ""),
    (&["--version"], 0, version, ""),
    (&["add", "--help"], 0, add_help, ""),
    (&["journey", "walk", "--help"], 0, journey_walk_help, ""),
    (&["drive", "--help"], 0, drive_help, ""),
    (&["again", "--help"], 0, internal, ""),
    (&["pages", "--help"], 0, pages_help, ""),
    (
        &["pages", "extra"],
        2,
        empty,
        "temper-client-journeys pages: expected [--prepared DIR] [--within DURATION] or --check\nusage:\n  temper-client-journeys pages [--prepared DIR] [--within DURATION]\n  temper-client-journeys pages --check\n",
    ),
    (
        &["pages", "--within", "90"],
        2,
        empty,
        "temper-client-journeys pages: expected [--prepared DIR] [--within DURATION] or --check\nusage:\n  temper-client-journeys pages [--prepared DIR] [--within DURATION]\n  temper-client-journeys pages --check\n",
    ),
    (
        &["pages", "--check", "--prepared", "DIR"],
        2,
        empty,
        "temper-client-journeys pages: expected [--prepared DIR] [--within DURATION] or --check\nusage:\n  temper-client-journeys pages [--prepared DIR] [--within DURATION]\n  temper-client-journeys pages --check\n",
    ),
    (&["normalise", "--help"], 0, normalise_help, ""),
    (&["normalise", "-h"], 0, normalise_help, ""),
    (
        &["normalise", "extra"],
        2,
        empty,
        "temper-client-journeys normalise: unexpected arguments\nusage:\n  temper-client-journeys normalise\n",
    ),
    (
        &["again"],
        2,
        empty,
        "temper-client-journeys again: NAME is not given\nusage:\n  temper-client-journeys again [--prepared DIR] NAME\n",
    ),
    (&["nope"], 2, empty, "temper-client-journeys: unexpected arguments\nusage:\n  temper-client-journeys\n"),
];

#[test]
fn the_binary_answers_its_table_without_starting_children() {
    let scratch = skein_scratch::Scratch::new("journeys-binary");
    let mut failures = Vec::new();
    let mut page = None;
    for (arguments, code, stdout, stderr) in ROWS {
        let output = Command::new(skein_scratch::paths::binary("temper-client-journeys"))
            .args(*arguments)
            .env_clear()
            .current_dir(scratch.path())
            .stdin(Stdio::null())
            .output()
            .expect("run temper-client-journeys");
        let out = String::from_utf8(output.stdout).expect("UTF-8 stdout");
        let err = String::from_utf8(output.stderr).expect("UTF-8 stderr");
        if output.status.code() != Some(*code) || !stdout(&out) || err != *stderr {
            failures.push(format!(
                "{arguments:?}: exit {:?}, stdout {out:?}, stderr {err:?}; expected exit {code}, stderr {stderr:?}",
                output.status.code()
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
