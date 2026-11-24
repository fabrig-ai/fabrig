//! Native preparation and trace replay use unissued credentials and immutable fixtures.

use crate::common;

use std::path::Path;

use skein_lib::Duration;
use skein_shell::Clock;
use skein_world::end_to_end::Command;
use smith_bench::agent::smith::{FaceSince, Observing, Prepared};
use smith_bench::process::{Bounds, Capture, execute};
use smith_bench::{End, Forced};

/// No test here proves these bounds; the answer and natural exit each expects come first.
const HANG_GUARD: Duration = Duration::from_secs(30);

// Its peer reads the hangup's disposition from /proc/self/status.
#[cfg(target_os = "linux")]
#[test]
fn a_terminal_peer_can_close_its_stdio_before_the_leader_exits_without_hangup() {
    let scratch = root("terminal-before-exit");
    let directory = scratch.path().join("root");
    std::fs::create_dir(&directory).expect("private fixture directory");
    let trace = directory.join("trace.jsonl");
    std::fs::write(&trace, []).expect("empty append file");
    let recording = directory.join("recording.jsonl");
    std::fs::write(&recording, common::events::bytes(&common::events::balanced())).expect("event recording");
    let command = Command {
        program: skein_scratch::paths::binary("test-peer"),
        arguments: vec!["terminal".into(), recording.into_os_string(), trace.clone().into_os_string()],
        environment: vec![("PATH".into(), "/usr/bin:/bin".into())],
        directory: directory.clone(),
    };
    let result = execute(
        &Clock::new(),
        command,
        Box::from(&b"finish the task\n"[..]),
        Capture::Agent(Box::new(smith_bench::agent::Observing::Smith(observing(&trace)))),
        bounds(),
    )
    .expect("tree settles");

    assert!(result.error.is_none(), "{:?}", result.error);
    assert_eq!(result.forced, Forced::No);
    assert!(result.answered.is_some());
    assert_eq!(result.tree.live_pids.len(), 0);
    assert_eq!(result.exit, smith_bench::Exit::Code(0));
}

fn root(name: &str) -> skein_scratch::Scratch {
    skein_scratch::Scratch::new(name)
}

fn observing(trace: &Path) -> Observing {
    let face = smith_bench::agent::smith::events::Shape::Chat;
    Observing::new(Prepared {
        face,
        settings: trace.with_extension("settings"),
        trace: Some(trace.into()),
        registry: FaceSince {
            commit: "30259d89b88834ed3edaaa08b3f320d2252bbb9a".into(),
            shape: face,
            receiving: serde_json::from_value(serde_json::json!({"string":524_288,"content":4096,"items":32,"encoded_line":16384})).expect("fixture bounds"),
        },
        rendered: serde_json::json!({"models":[{"input_price":10,"cached_price":1,"output_price":80,"price_unit":8_000_000}]}),
        program: "/bin/sh".into(),
        state: trace.with_extension("state"),
        serial: None,
        teardown: HANG_GUARD,
    }).expect("event observing")
}

fn bounds() -> Bounds {
    Bounds {
        hard: HANG_GUARD,
        teardown: HANG_GUARD,
        grace: Duration::from_millis(20),
        settlement: HANG_GUARD,
        stdout: 16384,
        stderr: 4096,
        force_walk: true,
    }
}

#[test]
fn a_terminal_person_closes_after_main_acceptance_and_drains_the_final_file_record() {
    let scratch = root("terminal");
    let directory = scratch.path().join("root");
    std::fs::create_dir(&directory).expect("fixture directory");
    let trace = directory.join("trace.jsonl");
    std::fs::write(&trace, []).expect("empty append file");
    let recording = directory.join("recording.jsonl");
    std::fs::write(&recording, common::events::bytes(&common::events::balanced())).expect("event recording");
    let command = Command { program:"/bin/sh".into(),arguments:vec!["-c".into(),
        "IFS= read -r prompt; sed '$d' \"$1\" >> \"$2\"; if IFS= read -r extra; then exit 9; fi; tail -n 1 \"$1\" >> \"$2\"".into(),
        "replay".into(),recording.into_os_string(),trace.clone().into_os_string()],
        environment:vec![("PATH".into(),"/usr/bin:/bin".into())],directory:directory.clone() };
    let result = execute(
        &Clock::new(),
        command,
        Box::from(&b"finish the task\n"[..]),
        Capture::Agent(Box::new(smith_bench::agent::Observing::Smith(observing(&trace)))),
        bounds(),
    )
    .expect("tree settles");
    assert!(result.error.is_none(), "{:?}", result.error);
    assert_eq!(result.forced, Forced::No);
    assert!(result.answered.is_some());
    assert!(String::from_utf8_lossy(&result.trace).contains("session.ended"));
    assert_eq!(result.observing.expect("paired observer").classify(result.exit, result.forced).end, End::Completed);
    assert_eq!(result.tree.live_pids.len(), 0);
}

// It reads source history; macOS's checkout holds the tested commit alone.
#[cfg(target_os = "linux")]
#[test]
fn the_current_source_arm_selects_the_chat_face() {
    let project = Path::new(env!("CARGO_MANIFEST_DIR"));
    let repository = project.parent().expect("repository");
    let output = std::process::Command::new("git")
        .args(["-C", repository.to_str().expect("path"), "rev-parse", "HEAD"])
        .output()
        .expect("current source identity");
    assert!(output.status.success());
    let commit = String::from_utf8(output.stdout).expect("commit").trim().to_owned();
    let arm = smith_bench::arm::BuiltArm {
        version: 1,
        commit,
        sha256: "0".repeat(64),
        toolchain: "selection only".into(),
        binaries: vec![],
    };
    let entries = smith_bench::agent::smith::read_faces(&project.join("agents/smith/faces.toml")).expect("registry");
    let selected = smith_bench::agent::smith::face(&arm, &entries, repository).expect("source face");
    assert_eq!(selected.shape, smith_bench::agent::smith::events::Shape::Chat);
}
