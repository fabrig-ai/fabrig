//! Native preparation and trace replay use unissued credentials and immutable fixtures.

use crate::common;

use std::path::{Path, PathBuf};

use skein_lib::Duration;
use skein_shell::Clock;
use skein_world::end_to_end::Command;
use smith_bench::agent::smith::{FaceSince, Observing, Prepared};
use smith_bench::process::{Bounds, Capture, execute};
use smith_bench::{End, Forced, Measure};

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
        program: env!("CARGO_BIN_EXE_test-peer").into(),
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
fn the_selected_event_face_observes_the_answer_and_drains_its_native_boundaries() {
    let scratch = root("events");
    let directory = scratch.path().join("root");
    std::fs::create_dir(&directory).expect("private fixture directory");
    let trace = directory.join("trace.jsonl");
    let replay = directory.join("codec-control.jsonl");
    let records = common::events::bytes(&common::events::balanced());
    std::fs::write(&trace, []).expect("empty append stream");
    std::fs::write(&replay, &records).expect("synthetic codec records");
    let face = smith_bench::agent::smith::events::Shape::Chat;
    let prepared = Prepared {
        face,
        settings: directory.join("settings.json"),
        trace: Some(trace.clone()),
        registry: FaceSince {
            commit: "30259d89b88834ed3edaaa08b3f320d2252bbb9a".into(),
            shape: face,
            receiving: serde_json::from_value(serde_json::json!({
                "string":524_288,"content":4096,"items":32,
                "encoded_line":records.split_inclusive(|byte| *byte == b'\n').map(<[u8]>::len).max().expect("record lines")
            })).expect("writer-derived receiving ceiling"),
        },
        rendered: serde_json::json!({}),
        program: "/bin/sh".into(),
        state: directory.join("state"),
        serial: None,
        teardown: bounds().teardown,
    };
    let observer = Observing::new(prepared).expect("checked event receiving bounds");
    let command = Command {
        program: "/bin/sh".into(),
        arguments: vec![
            "-c".into(),
            "IFS= read -r prompt; cat \"$1\" >> \"$2\"; if IFS= read -r extra; then exit 9; fi".into(),
            "codec-replay".into(),
            replay.into_os_string(),
            trace.into_os_string(),
        ],
        environment: vec![("PATH".into(), "/usr/bin:/bin".into())],
        directory: directory.clone(),
    };
    let result = execute(
        &Clock::new(),
        command,
        Box::from(&b"task\n"[..]),
        Capture::Agent(Box::new(smith_bench::agent::Observing::Smith(observer))),
        bounds(),
    )
    .expect("real replay tree settles");
    assert_eq!(result.forced, Forced::No);
    assert_eq!(result.trace, records);
    assert!(result.answered.is_some());
    assert_eq!(result.error, None);
    let Some(smith_bench::agent::Observing::Smith(native)) = result.observing.as_deref() else {
        panic!("selected native face retained");
    };
    assert_eq!(native.observer.classify(result.exit, result.forced).end, End::Completed);
    assert_eq!(native.counts().responses, Measure::Observed { value: 1 });
    let health = native.health(result.exit, result.forced).expect("typed passive evidence");
    for check in &health.passive_checks {
        if check.check == "teardown"
            || (check.check.starts_with("headroom/")
                && !matches!(check.check.as_str(), "headroom/prompt-tokens" | "headroom/spend"))
        {
            assert!(matches!(check.passed, Measure::Unavailable { .. }), "{}", check.check);
        } else {
            assert_eq!(check.passed, Measure::Observed { value: true }, "{}", check.check);
        }
    }
    assert_eq!(health.headroom.len(), 7);
    let spend = native.spend(&native.tokens().expect("exact response usage")).expect("reported notional spend");
    assert_eq!(spend.basis, smith_bench::SpendBasis::Notional);
    assert_eq!(spend.usd, Measure::Observed { value: 0.000_003 });
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

#[test]
fn an_incomplete_file_record_faults_after_observed_tree_settlement_and_retains_its_prefix() {
    let scratch = root("partial");
    let directory = scratch.path().join("root");
    std::fs::create_dir(&directory).expect("fixture directory");
    let trace = directory.join("trace.jsonl");
    std::fs::write(&trace, b"{\"type\":").expect("partial fixture");
    let command = Command {
        program: "/bin/sh".into(),
        arguments: vec!["-c".into(), "exit 0".into()],
        environment: vec![],
        directory: directory.clone(),
    };
    let result = execute(
        &Clock::new(),
        command,
        Box::new([]),
        Capture::Agent(Box::new(smith_bench::agent::Observing::Smith(observing(&trace)))),
        bounds(),
    )
    .expect("settled tree");
    assert!(result.error.as_ref().is_some_and(|error| error.contains("partial record")));
    assert_eq!(result.trace, b"{\"type\":");
    assert_eq!(result.tree.live_pids.len(), 0);
}

mod exec;

#[test]
fn an_exec_arm_reads_its_prompt_to_eof_without_a_trace_file() {
    let fixture = exec::ExecFixture::new("exec-eof", 0);
    let context = fixture.context();
    let prepared = smith_bench::agent::smith::prepare(&context).expect("checked Exec");
    assert!(prepared.trace.is_none());
    let invocation = prepared.command();
    let result = execute(
        &Clock::new(),
        Command {
            program: invocation.program,
            arguments: invocation.arguments,
            environment: context.environment.to_vec(),
            directory: context.private.workspace.clone(),
        },
        b"read this prompt to EOF\n".as_slice().into(),
        Capture::Agent(Box::new(smith_bench::agent::Observing::Smith(Observing::new(prepared).expect("observer")))),
        bounds(),
    )
    .expect("Exec pipes settle");
    assert_eq!(result.exit, smith_bench::Exit::Code(0));
    assert_eq!(result.forced, Forced::No);
    assert!(result.error.is_none() && result.tree.live_pids.is_empty());
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
