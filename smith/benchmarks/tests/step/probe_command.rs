//! Replaying controls (smith/docs/design/testing.md, 2).

use std::path::Path;

use skein_fake_machine::Item;
use smith_bench::files::Files;
use smith_bench::testing::task::parse_task;
use smith_bench::{Calibration, Judgment, Measure, RunSummary, Task, read_summary_with};
use smith_bench_fixtures::files::{Operation, Tree, Write};

use crate::common::{self, recorded_passes};

const CALIBRATION_REPOSITORY: &str = "/repository";
const CALIBRATION_MANIFEST: &str = "/repository/task.toml";

fn calibration_files() -> Tree {
    Tree::new(&[
        Item::directory(b"repository"),
        Item::file(
            CALIBRATION_MANIFEST.trim_start_matches('/').as_bytes(),
            include_bytes!("../../tasks/probes/smoke-tools/task.toml"),
        ),
    ])
}

fn read_calibration_task(files: &Tree) -> Task {
    let file = Path::new(CALIBRATION_MANIFEST);
    let document = files.read_to_string(file).expect("read task");
    parse_task(file, &document).expect("parse task")
}

fn record_calibration(files: &Tree, name: &str, block: &Calibration) -> RunSummary {
    const SEED: u64 = 7;
    let repository = Path::new(CALIBRATION_REPOSITORY);
    let file = Path::new(CALIBRATION_MANIFEST);
    smith_bench::probe::record_calibration_with(files, repository, file, name, block, &recorded_passes(), SEED)
        .expect("record calibration");
    let summary = repository.join("benchmarks").join(&block.summary);
    assert_eq!(
        files.writes(),
        [
            (Operation::CreateDirAll, Path::new("/repository/benchmarks/summaries")),
            (Operation::CreateNew, Path::new("/repository/benchmarks/summaries/calibration.json")),
            (Operation::Write, file),
        ]
        .map(|(operation, path)| Write { operation, path: path.into(), source: None })
    );
    read_summary_with(files, &summary).expect("read calibration summary")
}

#[test]
fn a_calibration_block_names_its_summary_and_failing_control() {
    const CALIBRATION_ATTEMPTS: usize = 10; // Ten passes in common::recorded_passes.
    let files = calibration_files();
    let block = common::calibration(&read_calibration_task(&files));
    let summary = record_calibration(&files, "probes/smoke-tools", &block);
    let task = read_calibration_task(&files);

    assert_eq!(task.calibration, Some(block.clone()));
    assert_eq!(block.failed_before, ["control-run/01"]);
    assert_eq!(block.control, "c".repeat(40));
    assert_eq!(summary.groups[0].attempts.len(), CALIBRATION_ATTEMPTS);
    assert!(task.calibrated().expect("current inputs"));
}

#[test]
fn a_variant_carries_its_own_calibration() {
    let files = calibration_files();
    let child = read_calibration_task(&files).with_variant("child").expect("child");
    let block = common::calibration(&child);
    let _summary = record_calibration(&files, "probes/smoke-tools/child", &block);
    let task = read_calibration_task(&files);

    assert_eq!(task.calibration, None);
    let variant = task.with_variant("child").expect("child");
    assert_eq!(variant.calibration, Some(block));
    assert!(variant.calibrated().expect("current inputs"));
}

#[test]
fn a_line_prints_an_unavailable_spend_as_a_dash() {
    let mut result = crate::common::attempt();
    result.spend.usd = Measure::unavailable("no prices for this model");
    let text = smith_bench::probe::line(&result).to_string();
    assert!(text.starts_with(&format!("{}/{} ", result.identity.run, result.identity.attempt)), "{text}");
    assert!(text.ends_with(" spend=-"), "{text}");
}

#[test]
fn calibration_stops_at_ten_passes_or_once_ten_are_out_of_reach() {
    let result = crate::common::attempt();
    assert_eq!(smith_bench::judge(&result), Judgment::Pass);
    let results = vec![result; 10];
    let count = smith_bench::probe::count(&results, 10, 12);
    assert!(count.calibrated());
    let mut results = results[..3].to_vec();
    for result in &mut results {
        result.outcome.checks[0].passed = Measure::Observed { value: false };
    }
    let count = smith_bench::probe::count(&results, 10, 12);
    assert!(!count.calibrated());
    assert!(!count.reachable());
}

#[test]
fn ten_passes_within_twelve_attempts_is_calibrated() {
    let mut results = recorded_passes();
    let mut deviation = results[0].clone();
    deviation.outcome.deciding.push("answer".into());
    deviation.outcome.checks.push(smith_bench::CheckVerdict {
        check: "answer".into(),
        passed: Measure::Observed { value: false },
        evidence: vec!["wrong sum".into()],
    });
    results.extend([deviation.clone(), deviation]);
    let counted = smith_bench::probe::count(&results, 10, 12);
    assert_eq!((counted.passed, counted.failed, counted.deviations, counted.unjudged), (10, 0, 2, 0));
    assert!(counted.calibrated());
}

#[test]
fn a_model_deviation_is_counted_apart_from_smiths_failures() {
    let mut results = recorded_passes();
    results.truncate(2);
    results[0].outcome.deciding.push("answer".into());
    results[0].outcome.checks.push(smith_bench::CheckVerdict {
        check: "answer".into(),
        passed: Measure::Observed { value: false },
        evidence: vec!["wrong sum".into()],
    });
    results[1].outcome.end =
        smith_bench::End::Failed { reason: smith_bench::FailureReason::Model, detail: "malformed compaction".into() };
    let counted = smith_bench::probe::count(&results, 10, 12);
    assert_eq!((counted.passed, counted.failed, counted.deviations, counted.unjudged), (0, 1, 1, 0));
    assert!(!counted.calibrated());
}

#[test]
fn an_unavailable_advisory_check_leaves_a_model_deviation() {
    let mut result = recorded_passes().remove(0);
    result.outcome.deciding = vec!["file-content:first.txt".into()];
    result.outcome.checks = vec![smith_bench::CheckVerdict {
        check: "file-content:first.txt".into(),
        passed: Measure::Observed { value: false },
        evidence: vec!["No such file or directory (os error 2)".into()],
    }];
    for check in ["teardown", "headroom/request-bytes", "headroom/first-byte", "headroom/largest-gap"] {
        result.outcome.checks.push(smith_bench::CheckVerdict {
            check: check.into(),
            passed: Measure::unavailable("not yet: passive observation"),
            evidence: vec![],
        });
    }
    assert!(smith_bench::probe::model_deviation(&result));
}

#[test]
fn a_failed_deciding_contract_check_is_no_model_deviation() {
    let mut result = recorded_passes().remove(0);
    result.outcome.deciding = vec!["file-content:first.txt".into(), "run-completed".into()];
    result.outcome.checks = ["file-content:first.txt", "run-completed"]
        .into_iter()
        .map(|check| smith_bench::CheckVerdict {
            check: check.into(),
            passed: Measure::Observed { value: false },
            evidence: vec![],
        })
        .collect();
    assert!(!smith_bench::probe::model_deviation(&result));
}

#[test]
fn a_block_whose_prompt_digest_differs_gates_nothing() {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut task = smith_bench::read_task(&repository.join("tasks/probes/smoke-tools/task.toml")).expect("task");
    task.calibration = Some(common::calibration(&task));
    assert!(task.calibrated().expect("unchanged inputs"));
    task.prompt.push_str("Read another file.\n");
    assert!(!task.calibrated().expect("changed prompt"));
}

#[test]
fn a_failed_call_count_with_smiths_checks_passing_is_a_model_deviation() {
    let mut result = recorded_passes().remove(0);
    result.outcome.deciding = vec!["call-count".into(), "compaction-carried".into(), "tool-call".into()];
    result.outcome.checks = vec![
        smith_bench::CheckVerdict {
            check: "call-count".into(),
            passed: Measure::Observed { value: false },
            evidence: vec![],
        },
        smith_bench::CheckVerdict {
            check: "compaction-carried".into(),
            passed: Measure::Observed { value: true },
            evidence: vec![],
        },
        smith_bench::CheckVerdict {
            check: "tool-call".into(),
            passed: Measure::Observed { value: true },
            evidence: vec![],
        },
    ];
    let counted = smith_bench::probe::count(&[result], 10, 12);
    assert_eq!((counted.passed, counted.failed, counted.deviations, counted.unjudged), (0, 0, 1, 0));
}

#[test]
fn a_model_failure_beside_an_unavailable_contract_check_is_unjudged() {
    let mut result = recorded_passes().remove(0);
    result.outcome.deciding = vec!["answer".into(), "compaction-carried".into()];
    result.outcome.checks = vec![
        smith_bench::CheckVerdict {
            check: "answer".into(),
            passed: Measure::Observed { value: false },
            evidence: vec![],
        },
        smith_bench::CheckVerdict {
            check: "compaction-carried".into(),
            passed: Measure::unavailable("prompt was not captured"),
            evidence: vec![],
        },
    ];
    let counted = smith_bench::probe::count(&[result], 10, 12);
    assert_eq!((counted.passed, counted.failed, counted.deviations, counted.unjudged), (0, 0, 0, 1));
}

#[test]
fn an_early_finish_before_the_threshold_is_a_model_deviation() {
    let mut result = recorded_passes().remove(0);
    result.outcome.deciding =
        vec!["answer".into(), "call-count".into(), "conversation-opened".into(), "compaction-choice".into()];
    result.outcome.checks = ["answer", "call-count"]
        .into_iter()
        .map(|check| smith_bench::CheckVerdict {
            check: check.into(),
            passed: Measure::Observed { value: false },
            evidence: vec![],
        })
        .collect();
    for check in ["conversation-opened", "compaction-choice"] {
        result.outcome.checks.push(smith_bench::CheckVerdict {
            check: check.into(),
            passed: Measure::unavailable("not exercised: no request passed the compaction line"),
            evidence: vec![],
        });
    }
    let counted = smith_bench::probe::count(&[result], 10, 12);
    assert_eq!((counted.passed, counted.failed, counted.deviations, counted.unjudged), (0, 0, 1, 0));
}

#[test]
fn only_the_probe_command_names_another_shared_temporary() {
    for (command, name) in [("suite", "smoke"), ("calibrate", "probes/smoke-tools"), ("fix", "probes/smoke-tools")] {
        let fault = smith_bench::probe::command(
            command,
            &[name.into(), "--shared-temporary".into(), "/owned/shared-tmp".into()],
        )
        .expect_err("shared temporary is a probe option alone");
        assert_eq!(fault.kind, smith_bench::FaultKind::Input, "{command}: {fault}");
        assert_eq!(fault.detail, "unknown or repeated command option", "{command}");
    }
}
