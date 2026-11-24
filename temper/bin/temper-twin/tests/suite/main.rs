//! The twin binary's paths through main (temper/docs/design/testing/tools.md, 6).
mod cli;

use std::process::{Command, Stdio};

const TABLE: skein_shell::Table =
    skein_shell::Table { version: temper_version::VERSION, commit: temper_version::COMMIT, ..temper_twin_tool::TABLE };

fn empty(text: &str) -> bool {
    text.is_empty()
}

fn help(text: &str) -> bool {
    text.lines().next() == Some("temper-twin: a live run's twin: a fake LLM document that replays its agents.")
        && !text.contains("--data")
        && text == TABLE.help()
}

fn version(text: &str) -> bool {
    let prefix = format!("temper-twin {} (", env!("CARGO_PKG_VERSION"));
    let Some(commit) = text.strip_prefix(&prefix).and_then(|text| text.strip_suffix(")\n")) else {
        return false;
    };
    let hash = commit.strip_suffix("-dirty").unwrap_or(commit);
    commit == "unknown" || (hash.len() == 12 && hash.bytes().all(|byte| byte.is_ascii_hexdigit()))
}

fn unknown(text: &str) -> bool {
    text.starts_with("temper-twin: unknown option: --nope\nusage:\n")
        && text == TABLE.refusal(TABLE.commands.first().expect("bare command"), "unknown option: --nope")
}

type Check = fn(&str) -> bool;
type Case = (&'static [&'static str], i32, Check, Check);

const ROWS: &[Case] =
    &[(&["--help"], 0, help, empty), (&["--version"], 0, version, empty), (&["--nope"], 2, empty, unknown)];

#[test]
fn the_binary_answers_its_table_and_refuses_other_shapes() {
    let scratch = skein_scratch::Scratch::new("twin-table");
    let mut failures = Vec::new();
    for (arguments, code, stdout, stderr) in ROWS {
        let output = Command::new(skein_scratch::paths::binary("temper-twin"))
            .args(*arguments)
            .env_clear()
            .current_dir(scratch.path())
            .stdin(Stdio::null())
            .output()
            .expect("run temper-twin");
        let out = String::from_utf8(output.stdout).expect("UTF-8 stdout");
        let err = String::from_utf8(output.stderr).expect("UTF-8 stderr");
        if output.status.code() != Some(*code) || !stdout(&out) || !stderr(&err) {
            failures.push(format!(
                "{arguments:?}: exit {:?}, stdout {out:?}, stderr {err:?}; expected exit {code}",
                output.status.code()
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
