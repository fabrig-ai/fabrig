//! The internal journeys binary's table (temper/docs/design/testing/tools.md, 6).
mod arguments;

use std::process::{Command, Stdio};

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
    (&["--version"], 0, version, ""),
    (&[], 2, empty, "temper-client-journeys: no command; temper-client-journeys --help says more\n"),
];

#[test]
fn the_binary_answers_its_table_without_starting_children() {
    let scratch = skein_scratch::Scratch::new("journeys-binary");
    let mut failures = Vec::new();
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
    }
    assert_eq!(std::fs::read_dir(scratch.path()).expect("scratch contents").count(), 0, "usage and help write nothing");
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
