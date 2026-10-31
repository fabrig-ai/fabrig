//! Neutral settled evidence supplies no inferred agent diagnostics or answers.

use std::path::Path;

use smith_bench::grade::{Ended, Snapshot};
use smith_bench::{End, Exit, Measure, OutcomeCheck};

fn graded_attempt() -> smith_bench::AttemptResult {
    let recorded = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/recorded/results/attempt.json");
    let mut result = smith_bench::read_result(&recorded).expect("recorded attempt");
    result.identity.task = "fixtures/01-backoff".into();
    result.outcome.end = End::Completed;
    result.outcome.deciding = ["end", "grade", "protected-unchanged", "tree-ended"].map(str::to_owned).into();
    result.outcome.grade = Some(Measure::Observed { value: true });
    result.outcome.checks = result
        .outcome
        .deciding
        .iter()
        .map(|name| smith_bench::CheckVerdict {
            check: name.clone(),
            passed: Measure::Observed { value: true },
            evidence: Vec::new(),
        })
        .collect();
    result
}

#[test]
fn an_unavailable_passive_check_outside_deciding_decides_nothing() {
    let mut result = graded_attempt();
    for passed in [Measure::unavailable("calls are not observed"), Measure::Observed { value: false }] {
        result.outcome.checks.push(smith_bench::CheckVerdict {
            check: "nothing-outside".into(),
            passed: passed.clone(),
            evidence: vec!["scope listing".into()],
        });
        assert_eq!(smith_bench::judge(&result), smith_bench::Judgment::Pass);
        assert_eq!(result.outcome.checks.last().expect("advisory evidence").passed, passed);
    }
}

#[test]
fn a_graded_attempt_passes_on_end_grade_protected_and_tree() {
    let mut result = graded_attempt();
    assert_eq!(smith_bench::judge(&result), smith_bench::Judgment::Pass);
    for index in 0..result.outcome.checks.len() {
        result.outcome.checks[index].passed = Measure::Observed { value: false };
        assert_eq!(smith_bench::judge(&result), smith_bench::Judgment::Fail, "{}", result.outcome.checks[index].check);
        result.outcome.checks[index].passed = Measure::Observed { value: true };
    }
}

#[test]
fn a_task_without_deciding_checks_is_refused() {
    let root = std::env::temp_dir().join(format!("smith-bench-no-deciding-{}", std::process::id()));
    std::fs::create_dir(&root).expect("fixture root");
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/recorded/tasks/valid");
    std::fs::create_dir(root.join("seed")).expect("seed");
    std::fs::copy(fixture.join("seed/README.md"), root.join("seed/README.md")).expect("seed bytes");
    let manifest = std::fs::read_to_string(fixture.join("task.toml"))
        .expect("manifest")
        .replace("deciding = [\"tree-ended\"]\n", "");
    std::fs::write(root.join("task.toml"), manifest).expect("empty deciding");
    let error =
        smith_bench::check_task(&root.join("task.toml"), &Path::new(env!("CARGO_MANIFEST_DIR")).join("../docs/design"))
            .expect_err("every task names deciding checks");
    assert_eq!(error.key, "deciding");
    assert_eq!(error.reason, "task requires deciding checks");
    std::fs::remove_dir_all(root).expect("remove fixture");
}

fn checks(ended: &Ended, checks: &[OutcomeCheck]) -> Vec<smith_bench::CheckVerdict> {
    smith_bench::grade::checks(Path::new("."), &Snapshot::new(), &Snapshot::new(), checks, ended)
}

fn neutral_outcomes_read_only_the_settled_end_exit_and_retained_text() {
    let mut ended = Ended {
        end: End::Completed,
        exit: Measure::Observed { value: Exit::Code(0) },
        last_line: Some("smith: completion failed: protocol".into()),
        answer: Some("survey complete: marker-42".into()),
        credential_refresh: Measure::Observed { value: true },
    };
    let expected = [
        OutcomeCheck::End { end: "completed".into() },
        OutcomeCheck::ExitCode { code: 0 },
        OutcomeCheck::LastLine { pattern: "^smith: completion failed: protocol$".into() },
        OutcomeCheck::Answer { pattern: "marker-42$".into() },
        OutcomeCheck::CredentialRefresh,
    ];
    assert!(checks(&ended, &expected).iter().all(|check| check.passed == Measure::Observed { value: true }));
    ended.end = End::Timeout;
    ended.exit = Measure::Observed { value: Exit::Signal(9) };
    ended.credential_refresh = Measure::Observed { value: false };
    let judged = checks(&ended, &expected);
    for index in [0, 1, 4] {
        assert_eq!(judged[index].passed, Measure::Observed { value: false });
    }
    assert_eq!(judged[2].passed, Measure::Observed { value: true });
    assert_eq!(judged[3].passed, Measure::Observed { value: true });
    ended.exit = Measure::unavailable("setup never spawned");
    ended.last_line = None;
    ended.answer = None;
    ended.credential_refresh = Measure::unavailable("no source login was borrowed");
    let judged = checks(&ended, &expected);
    for index in [1, 2, 3, 4] {
        assert!(matches!(judged[index].passed, Measure::Unavailable { .. }));
    }
}

fn every_terminal_name_is_distinct_from_answer_content_and_exit_code() {
    let cases = [
        (End::Completed, "completed"),
        (End::Timeout, "timeout"),
        (End::Budget { which: smith_bench::BudgetLimit::Turns }, "budget"),
        (End::Refused { setup: "no login".into() }, "refused"),
        (End::Failed { reason: smith_bench::FailureReason::Model, detail: "wire".into() }, "failed"),
        (End::HarnessError { what: "known shape changed".into() }, "harness-error"),
    ];
    for (end, name) in cases {
        let ended = Ended {
            end,
            exit: Measure::unavailable("no exit"),
            last_line: None,
            answer: None,
            credential_refresh: Measure::unavailable("no source"),
        };
        let judged = checks(&ended, &[OutcomeCheck::End { end: name.into() }]);
        assert_eq!(judged[0].passed, Measure::Observed { value: true });
    }
}

#[test]
fn settled_agent_neutral_outcome_contracts() {
    neutral_outcomes_read_only_the_settled_end_exit_and_retained_text();
    every_terminal_name_is_distinct_from_answer_content_and_exit_code();
}

fn protected_snapshot() -> Snapshot {
    use smith_bench::grade::Entry;
    Snapshot::from([
        ("README".into(), Entry::File { sha256: "a".repeat(64), executable: false }),
        ("Cargo.toml".into(), Entry::File { sha256: "b".repeat(64), executable: false }),
        ("src".into(), Entry::Directory),
        ("src/lib.rs".into(), Entry::File { sha256: "c".repeat(64), executable: false }),
    ])
}

fn protected_verdicts(before: &Snapshot, after: &Snapshot) -> Vec<smith_bench::CheckVerdict> {
    let ended = Ended {
        end: End::Completed,
        exit: Measure::Observed { value: Exit::Code(0) },
        last_line: None,
        answer: None,
        credential_refresh: Measure::unavailable("no credential attempt"),
    };
    let checks = ["README", "Cargo.toml"].map(|path| OutcomeCheck::Protected { path: path.into() });
    // Protected checks consume the supplied snapshots and never open this path.
    smith_bench::grade::checks(Path::new("/unopened-workspace"), before, after, &checks, &ended)
}

#[test]
fn prepared_protected_snapshot_ignores_changes_outside_its_paths() {
    let before = protected_snapshot();
    let mut after = before.clone();
    after.insert("src/lib.rs".into(), smith_bench::grade::Entry::File { sha256: "d".repeat(64), executable: false });
    assert!(!smith_bench::grade::changed(&before, &after, Path::new("README")));
    assert!(!smith_bench::grade::changed(&before, &after, Path::new("Cargo.toml")));
    let verdicts = protected_verdicts(&before, &after);
    assert_eq!(
        verdicts.iter().map(|verdict| verdict.passed.clone()).collect::<Vec<_>>(),
        [Measure::Observed { value: true }, Measure::Observed { value: true }]
    );
}

#[test]
fn changed_protected_file_fails_against_its_prepared_snapshot() {
    let before = protected_snapshot();
    let mut after = before.clone();
    after.insert("Cargo.toml".into(), smith_bench::grade::Entry::File { sha256: "d".repeat(64), executable: false });
    let changed: Vec<_> = ["README", "Cargo.toml"]
        .into_iter()
        .filter(|path| smith_bench::grade::changed(&before, &after, Path::new(path)))
        .collect();
    assert_eq!(changed, ["Cargo.toml"]);
    let verdicts = protected_verdicts(&before, &after);
    assert_eq!(
        verdicts.iter().map(|verdict| verdict.passed.clone()).collect::<Vec<_>>(),
        [Measure::Observed { value: true }, Measure::Observed { value: false }]
    );
}

#[test]
fn protected_directory_detects_missing_added_and_changed_descendants() {
    use smith_bench::grade::Entry;
    let before = protected_snapshot();
    let mut missing = before.clone();
    missing.remove(Path::new("src/lib.rs"));
    let mut added = before.clone();
    added.insert("src/new.rs".into(), Entry::File { sha256: "d".repeat(64), executable: false });
    let mut executable = before.clone();
    executable.insert("src/lib.rs".into(), Entry::File { sha256: "c".repeat(64), executable: true });
    let mut linked = before.clone();
    linked.insert("src/lib.rs".into(), Entry::Link { target: "elsewhere".into() });
    for after in [missing, added, executable, linked] {
        assert!(smith_bench::grade::changed(&before, &after, Path::new("src")));
    }
}
