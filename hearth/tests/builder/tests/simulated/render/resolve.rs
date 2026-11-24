use fixtures::{File, RecordedFailure, SIMULATION_SEED, TEST_REPOSITORY};
use hearth_builder_world::{fixtures, protocol};

const ABSENT_TASK: File =
    File { path: "tasks/absent/task.json", body: br#"{"packages":{"hearth-absent":{}}}"#, mode: 0o644 };
const MIN_WITH_ABSENT: File = File { path: "hosts/test/test-min.json", body: br#"{"absent":{}}"#, mode: 0o644 };

#[test]
fn an_unknown_package_fails_its_resolution_naming_it_and_exits_2() {
    let mut files = TEST_REPOSITORY.to_vec();
    files.retain(|file| file.path != MIN_WITH_ABSENT.path);
    files.extend([ABSENT_TASK, MIN_WITH_ABSENT]);
    let scenario = fixtures::recorded_resolution_failure(&files, RecordedFailure::UnknownPackage);
    let run = protocol::run(&scenario, SIMULATION_SEED);
    let text = String::from_utf8_lossy(&run.stdout);
    assert!(run.fault_fell, "the fake replayed the unknown-package recording");
    assert!(run.vectors.iter().flatten().any(|arg| arg.as_ref() == b"--include=hearth-absent"));
    assert!(text.contains("test-min: apt cannot locate package \"hearth-absent\""), "5.5: {text}");
    assert_eq!(run.code, Some(2));
}

#[test]
fn an_unreachable_mirror_fails_the_resolution_as_mmdebstraps_and_exits_75() {
    let scenario = fixtures::recorded_resolution_failure(TEST_REPOSITORY, RecordedFailure::UnreachableMirror);
    let run = protocol::run(&scenario, SIMULATION_SEED);
    let text = String::from_utf8_lossy(&run.stdout);
    assert!(run.fault_fell, "the fake replayed the unreachable-mirror recording");
    assert!(text.contains("test-min: mmdebstrap exited 25"), "7.2: {text}");
    assert!(text.contains("Connection refused"), "7.2: {text}");
    assert!(text.contains(" needed\n"), "7.2: {text}");
    assert_eq!(run.code, Some(75));
    assert!(fixtures::staging_is_empty(&run.shared), "5.2: no staging entry survives");
    assert_eq!(run.stderr, b"resolving 21e9a02feb65\nresolution 21e9a02feb65 failed\n");
    assert!(!text.lines().any(|line| line.starts_with("resolving ") || line.starts_with("resolution ")));
}
