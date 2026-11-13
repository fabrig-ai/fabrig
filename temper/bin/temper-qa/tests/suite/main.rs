//! The tool's front and real command boundary (testing/tools.md, 6).
mod builds;
mod commands;
mod hold;
mod passes;
mod place;
mod tune;
use std::process::{Command, Stdio};
use temper_qa_tool::TABLE;

fn paragraph(text: &str) -> String {
    text.split("\n\n").next().expect("paragraph").lines().collect::<Vec<_>>().join(" ")
}
#[test]
fn the_binary_answers_its_table_and_refuses_unknown_commands() {
    let scratch = skein_scratch::Scratch::new("qa-table");
    for arguments in [
        vec!["--help"],
        vec!["-h"],
        vec!["help"],
        vec!["--version"],
        vec!["run", "--help"],
        vec!["build", "keep", "--help"],
        vec!["triage", "none", "--help"],
        vec!["triage", "case", "--help"],
        vec!["pass", "setup", "--help"],
        vec!["pass", "recipe", "--help"],
        vec!["pass", "outcome", "--help"],
        vec!["pass", "finding", "--help"],
        vec!["pass", "check", "--help"],
        vec!["pass", "note", "--help"],
        vec!["show", "--help"],
        vec!["lines", "--help"],
        vec!["report", "--help"],
        vec!["pass", "open", "--help"],
        vec!["pass", "hold", "--help"],
        vec!["pass", "line", "--help"],
        vec!["pass", "close", "--help"],
        vec!["nope"],
    ] {
        let output = Command::new(skein_scratch::paths::binary("temper-qa"))
            .args(&arguments)
            .env_clear()
            .current_dir(scratch.path())
            .stdin(Stdio::null())
            .output()
            .expect("run tool");
        let stdout = String::from_utf8(output.stdout).expect("stdout");
        let stderr = String::from_utf8(output.stderr).expect("stderr");
        match arguments.as_slice() {
            ["--help" | "-h" | "help"] => {
                assert_eq!(output.status.code(), Some(0));
                assert_eq!(
                    paragraph(&stdout),
                    "temper-qa: QA after landing: main's landings triaged into cases, and passes run on builds kept for them."
                );
                let commands = stdout.split_once("\n\n").expect("help body").1;
                for internal in [
                    "run",
                    "build keep",
                    "triage none",
                    "triage case",
                    "pass setup",
                    "pass recipe",
                    "pass outcome",
                    "pass finding",
                    "pass check",
                    "pass note",
                    "show",
                    "lines",
                    "report",
                    "pass open",
                    "pass hold",
                    "pass line",
                    "pass close",
                    "TEMPER_INTERNAL_QA_HOME",
                    "TEMPER_INTERNAL_QA_SESSION",
                ] {
                    assert!(!commands.contains(internal), "internal entry: {internal}");
                }
                assert_eq!(stdout, TABLE.help());
            }
            ["--version"] => {
                assert_eq!(output.status.code(), Some(0));
                let prefix = format!("temper-qa {} (", env!("CARGO_PKG_VERSION"));
                let commit =
                    stdout.strip_prefix(&prefix).and_then(|text| text.strip_suffix(")\n")).expect("version line");
                let hash = commit.strip_suffix("-dirty").unwrap_or(commit);
                assert!(commit == "unknown" || (hash.len() == 12 && hash.bytes().all(|byte| byte.is_ascii_hexdigit())));
            }
            ["nope"] => {
                assert_eq!(output.status.code(), Some(2));
                assert_eq!(stdout, "");
                assert_eq!(stderr, "temper-qa: no command nope; temper-qa --help says more\n");
                continue;
            }
            _ => {
                assert_eq!(output.status.code(), Some(0));
                assert_eq!(
                    paragraph(&stdout),
                    "internal: not part of temper-qa's public command line; it may change without notice."
                );
                assert!(stdout.contains(&format!("temper-qa {}", arguments[..arguments.len() - 1].join(" "))));
            }
        }
        assert!(stderr.is_empty(), "{arguments:?}: {stderr}");
    }
}
