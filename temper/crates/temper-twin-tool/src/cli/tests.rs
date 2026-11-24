//! The twin's front, parsing and effect boundary (temper/docs/design/testing/tools.md, 6).
use super::{TABLE, TWIN, line, run};
use crate::Refusal;
use skein_scratch::Scratch;
use skein_shell::{Front, Table};
use std::{io, path::Path};

const TEST_TABLE: Table = Table { version: "0.0.0", commit: "0123456789ab", ..TABLE };

fn arguments(words: &[&str]) -> Vec<Box<[u8]>> {
    words.iter().map(|word| Box::from(word.as_bytes())).collect()
}

fn no_write(_: &Path, _: &[u8]) -> io::Result<()> {
    panic!("a refusal must not write the document")
}

#[test]
fn help_aliases_share_the_public_page() {
    let page = TABLE.help();
    let opening = page.lines().take_while(|line| !line.is_empty()).collect::<Vec<_>>().join(" ");
    assert_eq!(opening, "temper-twin: a live run's twin: a fake LLM document that replays its agents.");
    assert!(!page.contains("--data"));
    for flag in ["--help", "-h", "help"] {
        assert_eq!(TEST_TABLE.front(&arguments(&[flag])), Front::Print(page.clone()), "{flag}");
    }
}

#[test]
fn bare_command_refuses_missing_data() {
    let answer = run(&[], &TEST_TABLE, no_write);
    assert_eq!(answer.code, 2);
    assert_eq!(answer.stdout, "");
    assert!(answer.stderr.starts_with("temper-twin: --data is not given\nusage:\n"));
    assert_eq!(answer.stderr, TABLE.refusal(&TWIN, "--data is not given"));
}

#[test]
fn from_refuses_transcript_options_before_opening_output() {
    for (option, value) in [("--data", "directory"), ("--base", "base.json"), ("--cue", "1=brief")] {
        // No input need exist: parsing must refuse before any file is read.
        let args = arguments(&["--from", "source.json", "--out", "out.json", option, value]);
        let answer = run(&args, &TEST_TABLE, no_write);
        assert_eq!(answer.code, 2, "{option}");
        assert_eq!(
            answer.stderr,
            TABLE.refusal(&TWIN, "--from cannot be combined with --data, --base or --cue"),
            "{option}"
        );
    }
}

#[test]
fn a_read_refusal_names_the_transcript() {
    let refusal = Refusal::Read { path: "data/transcripts/task".into() };
    assert_eq!(line(&refusal), "read transcript: data/transcripts/task");
}

#[test]
fn a_conversion_refusal_exits_two_without_opening_the_output() {
    let scratch = Scratch::new("twin-refusal");
    let base = scratch.path().join("base.json");
    std::fs::write(
        &base,
        br#"{"listen":{"socket":"127.0.0.1:0"},"key":"/twin/key","tls":null,
            "models":[{"name":"fake","window":65536,"output":16384}],"cued":[],"asked":[],"otherwise":["off script"]}"#,
    )
    .expect("base file");
    let args = arguments(&[
        "--data",
        scratch.path().to_str().expect("scratch path"),
        "--base",
        base.to_str().expect("base path"),
        "--out",
        "out.json",
        "--cue",
        "9=missing task",
    ]);
    let answer = run(&args, &TEST_TABLE, no_write);
    assert_eq!(answer.code, 2);
    assert_eq!(answer.stdout, "");
    assert!(answer.stderr.starts_with("read transcript:"), "{}", answer.stderr);
    assert_eq!(answer.stderr.lines().count(), 1);
    assert!(answer.stderr.ends_with('\n'));
}
