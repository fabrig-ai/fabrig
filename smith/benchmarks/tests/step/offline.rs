//! Replaying controls (smith/docs/design/testing.md, 2).

use smith_bench::check_task;
use std::path::PathBuf;

use crate::common::design;
use crate::common::offline_recorded as recorded;

#[test]
fn fixtures_freeze_their_seeds_and_keep_graders_outside_them() {
    let root = crate::common::repository().join("tasks/fixtures");
    for name in ["01-backoff", "02-line-decoder"] {
        let directory = root.join(name);
        let task = check_task(&directory.join("task.toml"), &design()).expect("frozen fixture");
        assert_eq!(task.kind, smith_bench::Kind::Fixture);
        assert_eq!(task.tier, "working");
        assert_eq!(task.setup, [vec!["cargo", "build", "--offline", "--tests"]]);
        assert_eq!(task.protected, ["Cargo.toml", "Cargo.lock", "AGENTS.md"].map(PathBuf::from));
        assert!(directory.join("grader/tests/hidden.rs").is_file());
        assert!(directory.join("grader/tests/regression.rs").is_file());
        assert!(directory.join("reference/src/lib.rs").is_file());
        assert!(!directory.join("seed/tests/hidden.rs").exists());
        assert!(!directory.join("seed/grader").exists());
        assert!(!directory.join("seed/reference").exists());
    }
    let hidden = std::fs::read_to_string(root.join("02-line-decoder/grader/tests/hidden.rs")).expect("hidden test");
    assert!(hidden.contains("1u32 << 18"));
    assert!(!hidden.contains("i < 16"));
}

#[test]
fn no_fixture_protects_a_path_its_prompt_asks_to_change() {
    for id in ["01-backoff", "02-line-decoder", "03-planner"] {
        let manifest = crate::common::repository().join("tasks/fixtures").join(id).join("task.toml");
        let task = smith_bench::read_task(&manifest).expect("fixture task");
        assert!(!task.protected.iter().any(|path| path == std::path::Path::new("tests")), "{id} protects tests");
    }
}

#[test]
fn every_fixture_grader_holds_the_seeds_test_functions() {
    let names = regex::Regex::new(r"#\[test\]\s*fn\s+(\w+)").expect("test function expression");
    let root = crate::common::repository().join("tasks/fixtures");
    for id in ["01-backoff", "02-line-decoder", "03-planner"] {
        let directory = root.join(id);
        let grader = std::fs::read_dir(directory.join("grader/tests"))
            .expect("grader tests")
            .map(|entry| std::fs::read_to_string(entry.expect("entry").path()).expect("grader source"))
            .collect::<String>();
        for entry in std::fs::read_dir(directory.join("seed/tests")).expect("seed tests") {
            let source = std::fs::read_to_string(entry.expect("entry").path()).expect("seed source");
            for name in names.captures_iter(&source) {
                assert!(names.captures_iter(&grader).any(|candidate| candidate[1] == name[1]), "{id}: {}", &name[1]);
            }
        }
    }
}

#[test]
fn the_compare_suite_names_both_fixtures_and_both_agents() {
    let root = crate::common::repository();
    let suite = smith_bench::read_suite(&root.join("suites/compare.toml")).expect("comparison");
    assert_eq!(suite.tasks, ["fixtures/01-backoff", "fixtures/02-line-decoder"]);
    assert_eq!(suite.tier, "working");
    assert_eq!(suite.design, smith_bench::Design::Interleaved);
    assert_eq!(suite.repetitions, 5);
    assert_eq!(
        suite.agents.iter().map(|agent| agent.agent).collect::<Vec<_>>(),
        [smith_bench::Agent::Smith, smith_bench::Agent::Codex]
    );
    assert_eq!(suite.agents.iter().map(|agent| agent.config.as_str()).collect::<Vec<_>>(), ["standard", "standard"]);
    let pin = smith_bench::read_configuration(&root.join("agents/smith/standard.pin.toml")).expect("comparison pin");
    assert_eq!(pin.pin.configuration, PathBuf::from("standard.toml"));
}

fn refuses(name: &str, key: &str, reason: &str) {
    let error = check_task(&recorded(name), &design()).expect_err("recorded task must refuse");
    assert!(error.key.contains(key), "{error}");
    assert!(error.reason.contains(reason), "{error}");
}

fn a_prompt_without_its_final_newline_is_refused() {
    refuses("missing-newline", "prompt", "end in a newline");
}

fn a_prompt_with_two_paragraphs_is_refused() {
    refuses("multiple-paragraphs", "prompt", "one nonempty paragraph");
}

fn a_prompt_at_the_terminal_line_bound_is_refused() {
    refuses("oversize-prompt", "prompt", "4,088 bytes");
}

fn changed_seed_bytes_are_refused() {
    refuses("wrong-digest", "seed_sha256", "seed digest differs");
}

fn a_missing_design_section_is_refused() {
    refuses("missing-guard", "guards[0]", "does not exist");
}

fn a_benchmark_tree_in_a_seed_is_refused() {
    refuses("benchmark-seed", "seed", "stay outside");
}

fn a_grader_file_in_a_seed_is_refused() {
    refuses("grader-seed", "seed", "stay outside");
}

fn a_reference_file_in_a_seed_is_refused() {
    refuses("reference-seed", "seed", "stay outside");
}

fn an_unknown_outcome_check_is_refused() {
    refuses("unknown-check", "outcome", "unknown variant");
}

fn an_unknown_nested_key_is_refused() {
    refuses("unknown-key", "budget.turnz", "unknown field");
}

fn a_check_cannot_escape_the_workspace() {
    refuses("outside-path", "outcome[0].path", "without traversal");
}

fn duplicate_variant_names_are_refused() {
    refuses("duplicate-variant", "variant[1].name", "unique");
}

#[test]
fn invalid_recorded_manifests() {
    a_prompt_without_its_final_newline_is_refused();
    a_prompt_with_two_paragraphs_is_refused();
    a_prompt_at_the_terminal_line_bound_is_refused();
    changed_seed_bytes_are_refused();
    a_missing_design_section_is_refused();
    a_benchmark_tree_in_a_seed_is_refused();
    a_grader_file_in_a_seed_is_refused();
    a_reference_file_in_a_seed_is_refused();
    an_unknown_outcome_check_is_refused();
    an_unknown_nested_key_is_refused();
    a_check_cannot_escape_the_workspace();
    duplicate_variant_names_are_refused();
}

#[test]
fn committed_summaries_and_baselines_parse_at_result_version_two() {
    let root = crate::common::repository();
    assert_eq!(smith_bench::RESULT_VERSION, 4);
    // The catalogue test checks the full inventory; sample each versioned reader here.
    for entry in std::fs::read_dir(root.join("summaries")).expect("summaries") {
        let path = entry.expect("summary entry").path();
        if path.extension().is_some_and(|extension| extension == "json") {
            assert_eq!(smith_bench::read_summary(&path).expect("committed summary").result_version, 4);
            break;
        }
    }
    for entry in std::fs::read_dir(root.join("baselines")).expect("baselines") {
        let path = entry.expect("baseline entry").path();
        if path.extension().is_some_and(|extension| extension == "json") {
            assert_eq!(smith_bench::read_baseline(&path).expect("committed baseline").result_version, 4);
            break;
        }
    }
}

fn multi_configuration_summary() -> smith_bench::RunSummary {
    let file = crate::common::repository().join("tests/recorded/results/attempt.json");
    let results = smith_bench::read_results(&file).expect("recorded attempts");
    let mut summary =
        smith_bench::summarise(&results, "recorded", "small", smith_bench::Design::Single, 123, None, None)
            .expect("whole summary");
    let first = summary.groups[0].clone();
    for change in 0..4 {
        let mut other = first.clone();
        match change {
            0 => other.agent = smith_bench::Agent::Smith,
            1 => other.configuration_sha256 = "other-configuration".into(),
            2 => other.model.requested_model = "other-model".into(),
            _ => other.model.requested_effort = "other-effort".into(),
        }
        summary.groups.push(other);
    }
    summary
}

#[test]
fn a_baseline_selector_keeps_one_configuration_model_and_effort() {
    let summary = multi_configuration_summary();
    let first = summary.groups[0].clone();
    let selected = smith_bench::baseline_group(
        &summary,
        first.agent,
        &first.configuration_sha256,
        &first.model.requested_model,
        &first.model.requested_effort,
    )
    .expect("one selected configuration");
    assert_eq!(selected.groups.as_slice(), std::slice::from_ref(&first));
    assert_eq!(summary.groups.len(), 5, "the whole suite summary stays intact");
    assert!(
        smith_bench::baseline_group(
            &summary,
            first.agent,
            "missing",
            &first.model.requested_model,
            &first.model.requested_effort,
        )
        .is_err()
    );
}
