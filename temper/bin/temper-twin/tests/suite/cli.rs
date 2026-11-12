//! The binary writes only validated twins (temper/docs/design/testing/fakes.md, 1).
use serde_json::json;
use skein_scratch::Scratch;
use std::{path::Path, process::Command};

fn base(scratch: &Scratch) -> std::path::PathBuf {
    let file = scratch.path().join("base.json");
    std::fs::write(
        &file,
        serde_json::to_vec(&json!({
            "listen":{"socket":"127.0.0.1:0"}, "key":"/twin/key", "tls":null,
            "models":[{"name":"fake","window":65536,"output":16384}],
            "cued":[],"asked":[],"otherwise":["off script"]
        }))
        .expect("base JSON"),
    )
    .expect("base file");
    file
}
#[test]
fn a_conversion_refusal_exits_two_without_opening_the_output() {
    let scratch = Scratch::new("twin-cli-refusal");
    let base = base(&scratch);
    let out = scratch.path().join("out.json");
    let result = Command::new(env!("CARGO_BIN_EXE_temper-twin"))
        .arg("--data")
        .arg(scratch.path())
        .arg("--base")
        .arg(base)
        .arg("--out")
        .arg(&out)
        .args(["--cue", "9=missing task"])
        .output()
        .expect("run temper-twin");
    assert_eq!(result.status.code(), Some(2));
    assert!(!out.exists());
    assert!(result.stderr.starts_with(b"read transcript:"));
    assert_eq!(String::from_utf8(result.stderr).expect("stderr").lines().count(), 1);
    assert!(result.stdout.is_empty());
}
#[test]
fn the_cli_reports_tasks_and_replacement_counts() {
    let scratch = Scratch::new("twin-cli-report");
    let base = base(&scratch);
    let directory = scratch.path().join("transcripts/0000000000000000");
    std::fs::create_dir_all(&directory).expect("task directory");
    std::fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../crates/temper-twin-tool/tests/fixtures/0000000000000005"),
        directory.join("0000000000000005"),
    )
    .expect("reviewer frames");
    let out = scratch.path().join("out.json");
    let result = Command::new(env!("CARGO_BIN_EXE_temper-twin"))
        .arg("--data")
        .arg(scratch.path())
        .arg("--base")
        .arg(base)
        .arg("--out")
        .arg(&out)
        .args(["--cue", "5=Review this change at the named head", "--replace", "Checked=Reviewed"])
        .output()
        .expect("run temper-twin");
    assert_eq!(result.status.code(), Some(0), "stderr: {}", String::from_utf8_lossy(&result.stderr));
    assert_eq!(result.stdout, b"task 5: 1 turns\ntask 5: 0 opaque blocks dropped\nChecked: 1\n");
    assert!(temper_fake::llm::Document::decode(&std::fs::read(out).expect("twin document")).is_ok());
}

#[test]
fn from_writes_a_twin_with_replacements_takes_and_a_use_report() {
    let scratch = Scratch::new("twin-cli-from");
    let source = base(&scratch);
    let takes = scratch.path().join("takes.json");
    std::fs::write(&takes, br#"[{"name":"WORD","value":"new","before":"<","after":">"}]"#).expect("takes file");
    let out = scratch.path().join("out.json");
    let result = Command::new(env!("CARGO_BIN_EXE_temper-twin"))
        .arg("--from")
        .arg(source)
        .arg("--out")
        .arg(&out)
        .args(["--replace", "off=new"])
        .arg("--takes")
        .arg(&takes)
        .output()
        .expect("run temper-twin");
    assert_eq!(result.status.code(), Some(0), "{}", String::from_utf8_lossy(&result.stderr));
    assert_eq!(result.stdout, b"off: 1\ntake WORD: 1 uses\n");
    let bytes = std::fs::read(out).expect("output written");
    temper_fake::llm::Document::decode(&bytes).expect("fake document");
    let document: serde_json::Value = serde_json::from_slice(&bytes).expect("JSON");
    assert_eq!(document["otherwise"][0]["lines"][1]["text"], "{{WORD}} script");
}

#[test]
fn from_refuses_transcript_options_before_opening_output() {
    let scratch = Scratch::new("twin-cli-from-conflict");
    let source = base(&scratch);
    let out = scratch.path().join("out.json");
    for (option, value) in [("--data", "directory"), ("--base", "base.json"), ("--cue", "1=brief")] {
        let result = Command::new(env!("CARGO_BIN_EXE_temper-twin"))
            .arg("--from")
            .arg(&source)
            .arg("--out")
            .arg(&out)
            .args([option, value])
            .output()
            .expect("run temper-twin");
        assert_eq!(result.status.code(), Some(2));
        assert!(
            String::from_utf8_lossy(&result.stderr).contains("--from cannot be combined"),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert!(!out.exists());
    }
}
