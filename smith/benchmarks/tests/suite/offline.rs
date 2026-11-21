//! Recorded task acceptance and each independent manifest refusal.

#![forbid(unsafe_code)]

use std::path::PathBuf;

use smith_bench::{check_task, check_tree};

#[test]
fn a_reference_that_changes_a_protected_path_fails_validation() {
    let scratch = skein_scratch::Scratch::new("bench-offline");
    let root = scratch.path().to_path_buf().join(format!("smith-bench-protected-reference-{}", std::process::id()));
    std::fs::create_dir_all(root.join("seed")).expect("seed");
    std::fs::create_dir(root.join("reference")).expect("reference");
    std::fs::write(root.join("seed/Cargo.toml"), "seed").expect("protected seed");
    std::fs::write(root.join("reference/Cargo.toml"), "changed").expect("changed reference");
    let digest = smith_bench::seed_digest(&root.join("seed")).expect("seed digest");
    let manifest = format!(
        "id = 'protected-reference'\nversion = 1\nkind = 'fixture'\ntier = 'working'\n\
         deadline_seconds = 10\nprompt = 'Fix the crate.\\n'\nseed_sha256 = '{digest}'\n\
         protected = ['Cargo.toml']\ndeciding = ['end', 'grade', 'protected-unchanged', 'tree-ended']\n\
         [[grade]]\ncommand = ['true']\ndeadline_seconds = 1\nmax_output_bytes = 1024\n"
    )
    .replace("'Fix the crate.\\n'", "\"Fix the crate.\\n\"");
    std::fs::write(root.join("task.toml"), manifest).expect("manifest");
    let error = smith_bench::validate::validate(&root, &[]).expect_err("changed protection refuses");
    assert!(error.detail.contains("reference changes Cargo.toml"), "{error}");
}

fn comparison_document(name: &str) -> (skein_scratch::Scratch, PathBuf, String) {
    let scratch = skein_scratch::Scratch::new("bench-offline");
    let directory = scratch.path().to_path_buf().join(format!("smith-bench-{name}-{}", std::process::id()));
    std::fs::create_dir(&directory).expect("temporary task");
    let manifest = crate::common::repository().join("tasks/fixtures/01-backoff/task.toml");
    (scratch, directory.join("task.toml"), std::fs::read_to_string(manifest).expect("fixture document"))
}

#[test]
fn a_comparison_task_with_a_budget_is_refused() {
    let (_scratch, file, document) = comparison_document("comparison-budget");
    for kind in ["fixture", "repository"] {
        for budget in ["[budget]\n", "[budget]\nseconds = 1\n"] {
            let document = document.replace("kind = \"fixture\"", &format!("kind = \"{kind}\""));
            std::fs::write(&file, format!("{document}\n{budget}")).expect("budget document");
            let error = check_task(&file, &design()).expect_err("comparison budget refuses");
            assert_eq!(error.key, "budget", "{kind}: {budget}");
            assert_eq!(error.reason, "comparison task must not name a budget", "{kind}: {budget}");
        }
    }
}

#[test]
fn a_comparison_variant_with_a_budget_is_refused() {
    let (_scratch, file, document) = comparison_document("comparison-variant-budget");
    for kind in ["fixture", "repository"] {
        let document = document.replace("kind = \"fixture\"", &format!("kind = \"{kind}\""));
        std::fs::write(&file, format!("{document}\n[[variant]]\nname = 'budget'\n[variant.budget]\n"))
            .expect("variant budget document");
        let error = check_task(&file, &design()).expect_err("comparison variant budget refuses");
        assert_eq!(error.key, "variant[0].budget", "{kind}");
        assert_eq!(error.reason, "comparison task must not name a budget", "{kind}");
    }
}

#[test]
fn a_comparison_deadline_at_the_run_defaults_is_refused() {
    let (_scratch, file, document) = comparison_document("comparison-deadline");
    for kind in ["fixture", "repository"] {
        for deadline in [3_600, 3_601, u64::MAX] {
            let document = document
                .replace("kind = \"fixture\"", &format!("kind = \"{kind}\""))
                .replace("deadline_seconds = 1800", &format!("deadline_seconds = {deadline}"));
            std::fs::write(&file, document).expect("deadline document");
            let error = check_task(&file, &design()).expect_err("comparison deadline refuses");
            assert_eq!(error.key, "deadline_seconds", "{kind}: {deadline}");
            assert_eq!(
                error.reason, "comparison deadline must be below the run default of 3600 seconds",
                "{kind}: {deadline}"
            );
        }
    }
}

fn every_catalogue_manifest_is_checked_offline() {
    let tasks = crate::common::repository().join("tasks");
    // The complete benchmark-tree check validates this same catalogue once.
    // Keep the fixture layout checks here; exercise check_tree on the small
    // recorded catalogue below instead of walking every frozen seed twice.
    let planner = tasks.join("fixtures/03-planner");
    for path in ["grader/tests/hidden.rs", "grader/tests/regression.rs", "reference/src/planner.rs"] {
        assert!(planner.join(path).is_file());
    }
    assert!(!planner.join("seed/tests/hidden.rs").exists());
}

fn every_committed_suite_and_configuration_pin_is_checked_offline() {
    let root = crate::common::repository();
    let counts = smith_bench::check_benchmark_tree_with_bundles(&root, &design(), &root.join("tests/recorded/bundles"))
        .expect("all benchmark manifests validate");
    assert!(counts.configurations > 0, "committed pins are checked");
}

fn the_recorded_suite_resolves_its_frozen_task_and_pin() {
    let root = crate::common::repository();
    let suite_file = root.join("tests/recorded/suites/sample.toml");
    let suite = smith_bench::read_suite(&suite_file).expect("recorded suite");
    let tasks =
        smith_bench::catalogue(&root.join("tests/recorded/tasks/valid"), &design()).expect("recorded catalogue");
    let models = smith_bench::read_model_tiers(&root.join("agents/models.toml")).expect("committed models");
    assert_eq!(
        smith_bench::validate_suite(&suite_file, &suite, &tasks, &root.join("agents"), &models)
            .expect("references resolve"),
        ["probes/recorded"]
    );
}

fn a_complete_recorded_task_is_accepted() {
    let file = recorded("valid");
    assert_eq!(check_tree(file.parent().expect("recorded catalogue"), &design()).expect("valid frozen task"), 1);
}

fn a_recorded_attempt_round_trips_without_erasing_measurement_gaps() {
    let file = crate::common::repository().join("tests/recorded/results/attempt.json");
    let result = smith_bench::read_result(&file).expect("recorded result");
    let rendered = smith_bench::render_result(&result).expect("versioned JSON");
    assert_eq!(smith_bench::parse_result(&file, &rendered).expect("round-trip"), result);
    assert!(matches!(result.spend.usd, smith_bench::Measure::Unavailable { .. }));
    assert!(matches!(result.resources.cpu_ms, smith_bench::Measure::LowerBound { .. }));
    assert_eq!(result.outcome.end, smith_bench::End::Completed);
    assert_eq!(result.outcome.forced, smith_bench::Forced::Killed);
}

fn a_summary_and_baseline_from_recorded_results_round_trip_offline() {
    let file = crate::common::repository().join("tests/recorded/results/attempt.json");
    let results = smith_bench::read_results(&file).expect("recorded attempts");
    let summary = smith_bench::summarise(&results, "recorded", "small", smith_bench::Design::Single, 123, None, None)
        .expect("summary");
    assert_eq!(summary.groups[0].counts_by_end["completed"], 1);
    assert_eq!(summary.groups[0].metrics["task_wall_ms"].observed, 1);
    let baseline = smith_bench::baseline(&summary).expect("single configuration baseline");
    let scratch = skein_scratch::Scratch::new("bench-offline");
    let root = scratch.path().to_path_buf().join(format!("smith-bench-summary-{}", std::process::id()));
    std::fs::create_dir(&root).expect("fresh test root");
    let summary_file = root.join("summary.json");
    let baseline_file = root.join("baseline.json");
    smith_bench::write_summary(&summary_file, &summary).expect("summary output");
    smith_bench::write_summary(&baseline_file, &baseline).expect("baseline output");
    assert_eq!(smith_bench::read_summary(&summary_file).expect("round-trip"), summary);
    assert_eq!(smith_bench::read_baseline(&baseline_file).expect("round-trip"), baseline);
    assert!(smith_bench::write_summary(&summary_file, &summary).is_err(), "committed evidence is not overwritten");
}

// Related offline assertions share a test process to keep startup overhead
// inside the one-second harness budget; each named story keeps its assertions.

#[test]
fn recorded_catalogues_and_results() {
    an_absent_bundle_is_skipped_and_said();
    a_bundle_whose_digest_differs_is_refused();
    a_present_bundle_matches_its_manifest();
    a_repository_seed_comes_from_its_bundle();
    every_catalogue_manifest_is_checked_offline();
    every_committed_suite_and_configuration_pin_is_checked_offline();
    the_recorded_suite_resolves_its_frozen_task_and_pin();
    a_complete_recorded_task_is_accepted();
    a_recorded_attempt_round_trips_without_erasing_measurement_gaps();
    a_summary_and_baseline_from_recorded_results_round_trip_offline();
}

fn an_absent_bundle_is_skipped_and_said() {
    let root = crate::common::repository().join("tests/recorded");
    let manifest = smith_bench::read_bundle_manifest(&root.join("tasks/repository-seed/seed/bundle.toml"))
        .expect("recorded manifest");
    assert_eq!(smith_bench::check_bundle(&manifest, &root.join("bundles/none")), Ok(smith_bench::BundleCheck::Absent));
}

fn a_bundle_whose_digest_differs_is_refused() {
    let root = crate::common::repository().join("tests/recorded");
    let mut manifest = smith_bench::read_bundle_manifest(&root.join("tasks/repository-seed/seed/bundle.toml"))
        .expect("recorded manifest");
    manifest.sha256 = "0".repeat(64);
    assert!(
        smith_bench::check_bundle(&manifest, &root.join("bundles"))
            .expect_err("digest differs")
            .reason
            .contains("digest")
    );
}

fn a_present_bundle_matches_its_manifest() {
    let root = crate::common::repository().join("tests/recorded");
    let manifest = smith_bench::read_bundle_manifest(&root.join("tasks/repository-seed/seed/bundle.toml"))
        .expect("recorded manifest");
    assert_eq!(smith_bench::check_bundle(&manifest, &root.join("bundles")), Ok(smith_bench::BundleCheck::Matches));
}

fn a_repository_seed_comes_from_its_bundle() {
    let root = crate::common::repository().join("tests/recorded");
    let task_directory = root.join("tasks/repository-seed");
    let task = check_task(&task_directory.join("task.toml"), &design()).expect("recorded task");
    let (source, digest) =
        smith_bench::seed_source(&task, &task_directory, &root.join("bundles")).expect("present repository seed");
    assert_eq!(source, root.join("bundles/tiny"));
    assert_eq!(digest, smith_bench::seed_digest(&source).expect("recorded bytes"));
    assert_ne!(digest, task.seed_sha256);
    assert!(
        smith_bench::seed_source(&task, &task_directory, &root.join("bundles/none"))
            .expect_err("absent bundles refuse attempts")
            .detail
            .contains("absent")
    );
}

use crate::common::design;
use crate::common::offline_recorded as recorded;
