//! Replaying controls (smith/docs/design/testing-tiers.md).

use skein_fake_machine::Item;
use smith_bench::files::{Files, Kind};
use smith_bench::testing::task::validate_suite_with;
use smith_bench::{self as bench, check_task};
use smith_bench_fixtures::files::Tree;
use std::os::unix::{ffi::OsStrExt, fs::PermissionsExt};
use std::path::{Path, PathBuf};

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

#[test]
fn a_reference_that_changes_a_protected_path_fails_validation() {
    use bench::grade::CommandReceipt;
    use bench::guard::Guard;
    use bench::isolate::Machine;
    use bench::testing::validate::{ReplayPaths, validate_catalogue_with};

    let tree = Tree::new(&[
        Item::directory(b"tasks"),
        Item::directory(b"tasks/seed"),
        Item::file(b"tasks/seed/Cargo.toml", b"seed"),
        Item::directory(b"tasks/reference"),
        Item::file(b"tasks/reference/Cargo.toml", b"changed"),
        Item::directory(b"tools"),
        Item::file(b"tools/true", b"").mode(0o700),
        Item::directory(b"scratch"),
    ]);
    let digest = bench::seed_digest_with(&tree, Path::new("/tasks/seed")).expect("seed digest");
    let manifest = format!(
        r#"id = "protected-reference"
version = 1
kind = "fixture"
tier = "working"
deadline_seconds = 10
prompt = "Fix the crate.\n"
seed_sha256 = "{digest}"
protected = ["Cargo.toml"]
deciding = ["end", "grade", "protected-unchanged", "tree-ended"]
[[grade]]
command = ["true"]
deadline_seconds = 1
max_output_bytes = 1024
"#,
    );
    tree.write(Path::new("/tasks/task.toml"), manifest.as_bytes()).expect("manifest");
    let tasks = bench::catalogue_with(&tree, Path::new("/tasks"), Path::new("/design")).expect("valid fixture");
    let paths = ReplayPaths {
        guard: &Guard::new_with(&tree, Path::new("/home"), Path::new("/config"), Path::new("/state"), &[])
            .expect("guard"),
        machine: &Machine { path: "/tools".into(), rustup_home: None },
        scratch: Path::new("/scratch"),
        bundles: Path::new("/bundles"),
    };
    let mut candidates = Vec::new();
    let error = validate_catalogue_with(&tree, &tasks, &[], &paths, |command, _bounds| {
        assert_eq!(command.program, Path::new("/tools/true"));
        candidates.push(tree.read(&command.directory.join("Cargo.toml")).expect("candidate's protected bytes"));
        Ok(CommandReceipt {
            exit: bench::Exit::Code(0),
            forced: bench::Forced::No,
            stdout: Vec::new(),
            stderr: Vec::new(),
            error: None,
        })
    })
    .expect_err("changed protection refuses");
    assert!(error.detail.contains("reference changes Cargo.toml"), "{error}");
    assert_eq!(candidates, [b"seed".to_vec(), b"changed".to_vec()]);
}

const COMPARISON_DOCUMENT: &str = include_str!("../../tasks/fixtures/01-backoff/task.toml");

fn check_comparison_document(document: &str) -> bench::Refusal {
    let file = Path::new("/task.toml");
    let tree = Tree::new(&[Item::file(b"task.toml", document.as_bytes())]);
    bench::check_task_with(&tree, file, Path::new("/design")).expect_err("comparison document refuses")
}

#[test]
fn a_comparison_task_with_a_budget_is_refused() {
    for kind in ["fixture", "repository"] {
        for budget in ["[budget]\n", "[budget]\nseconds = 1\n"] {
            let document = COMPARISON_DOCUMENT.replace("kind = \"fixture\"", &format!("kind = \"{kind}\""));
            let error = check_comparison_document(&format!("{document}\n{budget}"));
            assert_eq!(error.key, "budget", "{kind}: {budget}");
            assert_eq!(error.reason, "comparison task must not name a budget", "{kind}: {budget}");
        }
    }
}

#[test]
fn a_comparison_variant_with_a_budget_is_refused() {
    for kind in ["fixture", "repository"] {
        let document = COMPARISON_DOCUMENT.replace("kind = \"fixture\"", &format!("kind = \"{kind}\""));
        let error = check_comparison_document(&format!("{document}\n[[variant]]\nname = 'budget'\n[variant.budget]\n"));
        assert_eq!(error.key, "variant[0].budget", "{kind}");
        assert_eq!(error.reason, "comparison task must not name a budget", "{kind}");
    }
}

#[test]
fn a_comparison_deadline_at_the_run_defaults_is_refused() {
    for kind in ["fixture", "repository"] {
        for deadline in [3_600, 3_601, u64::MAX] {
            let document = COMPARISON_DOCUMENT
                .replace("kind = \"fixture\"", &format!("kind = \"{kind}\""))
                .replace("deadline_seconds = 1800", &format!("deadline_seconds = {deadline}"));
            let error = check_comparison_document(&document);
            assert_eq!(error.key, "deadline_seconds", "{kind}: {deadline}");
            assert_eq!(
                error.reason, "comparison deadline must be below the run default of 3600 seconds",
                "{kind}: {deadline}"
            );
        }
    }
}

pub(super) const BENCHMARK_TREE: &str = "/benchmarks";
pub(super) const DESIGN_TREE: &str = "/docs/design";

/// Lay the authored inputs, not a saved inventory that could omit a new manifest.
/// All harness reads below run against the resulting tree, never the checkout.
pub(super) fn catalogue_tree(paths: &[&str]) -> Tree {
    let smith = crate::common::repository().parent().expect("smith root").to_path_buf();
    let mut parents = std::collections::BTreeSet::new();
    for path in paths.iter().map(Path::new) {
        parents.extend(path.ancestors().skip(1).filter(|parent| !parent.as_os_str().is_empty()));
    }
    let mut items: Vec<_> = parents.into_iter().map(|path| Item::directory(path.as_os_str().as_bytes())).collect();
    for path in paths.iter().map(Path::new) {
        lay_input(&smith, path, &mut items);
    }
    Tree::new(&items)
}

fn lay_input(smith: &Path, path: &Path, items: &mut Vec<Item>) {
    let source = smith.join(path);
    let metadata = std::fs::symlink_metadata(&source).unwrap_or_else(|error| panic!("{}: {error}", source.display()));
    let name = path.as_os_str().as_bytes();
    let item = if metadata.is_dir() {
        Item::directory(name)
    } else if metadata.is_symlink() {
        let target = std::fs::read_link(&source).expect("fixture link");
        Item::link(name, target.as_os_str().as_bytes())
    } else if metadata.is_file() {
        let bytes = if fixture_has_bytes(path) {
            std::fs::read(&source).expect("authored fixture bytes")
        } else if path.starts_with("benchmarks/agents") {
            // An unlisted configuration must be absent, not a digest of empty bytes.
            return;
        } else {
            Vec::new()
        };
        Item::file(name, &bytes)
    } else {
        Item::fifo(name)
    };
    items.push(item.mode(metadata.permissions().mode() & 0o7777));
    if metadata.is_dir() && descend_fixture_directory(path) {
        for child in std::fs::read_dir(&source).expect("authored input directory") {
            lay_input(smith, &path.join(child.expect("directory entry").file_name()), items);
        }
    }
}

fn descend_fixture_directory(path: &Path) -> bool {
    if let Ok(agent) = path.strip_prefix("benchmarks/agents") {
        // Pins live one directory below agents; never traverse login directories.
        return agent.components().count() <= 1 && !matches!(agent.to_str(), Some(".codex" | ".claude" | "logins"));
    }
    !path.components().any(|part| matches!(part.as_os_str().to_str(), Some(".git" | ".codex" | ".claude")))
}

fn fixture_has_bytes(path: &Path) -> bool {
    let name = path.file_name().and_then(std::ffi::OsStr::to_str);
    if matches!(name, Some("auth.json" | ".credentials.json" | ".claude.json" | ".env")) {
        return false;
    }
    if let Ok(agent) = path.strip_prefix("benchmarks/agents") {
        // Only authored model, pin and pinned configuration bytes, never accounts or login files.
        return [
            "models.toml",
            "codex/overrides.toml",
            "smith/everything.toml",
            "smith/standard.toml",
            "smith/smith-claude-code.toml",
        ]
        .iter()
        .any(|fixture| agent == Path::new(fixture))
            || (agent.components().count() == 2 && name.is_some_and(|name| name.ends_with(".pin.toml")));
    }
    if path.starts_with("benchmarks/tasks") {
        // The catalogue only opens task documents and frozen seeds; grader and reference layout needs just names.
        return name == Some("task.toml") || path.components().any(|part| part.as_os_str() == "seed");
    }
    path.extension()
        .and_then(std::ffi::OsStr::to_str)
        .is_some_and(|extension| matches!(extension, "md" | "toml" | "json"))
        || path.starts_with("benchmarks/tests/recorded/bundles")
}

#[test]
fn recorded_catalogues_and_results() {
    let tree = catalogue_tree(&[
        "benchmarks/tasks",
        "benchmarks/agents",
        "benchmarks/suites",
        "benchmarks/summaries",
        "benchmarks/baselines",
        "benchmarks/tests/recorded/bundles",
        "benchmarks/tests/recorded/tasks/repository-seed",
        "benchmarks/tests/recorded/tasks/valid",
        "benchmarks/tests/recorded/suites/sample.toml",
        "benchmarks/tests/recorded/results/attempt.json",
        "docs/design",
    ]);
    let root = Path::new(BENCHMARK_TREE);
    let design = Path::new(DESIGN_TREE);
    let recorded = root.join("tests/recorded");
    let bundles = recorded.join("bundles");
    let task_directory = recorded.join("tasks/repository-seed");
    let mut manifest =
        bench::read_bundle_manifest_with(&tree, &task_directory.join("seed/bundle.toml")).expect("recorded manifest");

    assert_eq!(bench::check_bundle_with(&tree, &manifest, &bundles.join("none")), Ok(bench::BundleCheck::Absent));
    assert_eq!(bench::check_bundle_with(&tree, &manifest, &bundles), Ok(bench::BundleCheck::Matches));
    let task = bench::check_task_with(&tree, &task_directory.join("task.toml"), design).expect("recorded task");
    let (source, digest) =
        bench::seed_source_with(&tree, &task, &task_directory, &bundles).expect("present repository seed");
    assert_eq!(source, bundles.join("tiny"));
    assert_eq!(digest, bench::seed_digest_with(&tree, &source).expect("recorded bytes"));
    assert_ne!(digest, task.seed_sha256);
    assert!(
        bench::seed_source_with(&tree, &task, &task_directory, &bundles.join("none"))
            .expect_err("absent bundles refuse attempts")
            .detail
            .contains("absent")
    );
    manifest.sha256 = "0".repeat(64);
    assert!(
        bench::check_bundle_with(&tree, &manifest, &bundles).expect_err("digest differs").reason.contains("digest")
    );

    let planner = root.join("tasks/fixtures/03-planner");
    for path in ["grader/tests/hidden.rs", "grader/tests/regression.rs", "reference/src/planner.rs"] {
        assert!(tree.metadata(&planner.join(path)).is_ok_and(|facts| facts.kind == Kind::File), "{path}");
    }
    assert!(!tree.try_exists(&planner.join("seed/tests/hidden.rs")).expect("seed path"));
    let counts =
        bench::check_benchmark_tree_with(&tree, root, design, &bundles).expect("all benchmark manifests validate");
    assert!(counts.configurations > 0, "committed pins are checked");

    let suite_file = recorded.join("suites/sample.toml");
    let suite = bench::read_suite_with(&tree, &suite_file).expect("recorded suite");
    let valid = recorded.join("tasks/valid");
    let tasks = bench::catalogue_with(&tree, &valid, design).expect("recorded catalogue");
    let agents = root.join("agents");
    let models = bench::read_model_tiers_with(&tree, &agents.join("models.toml")).expect("committed models");
    assert_eq!(
        validate_suite_with(&suite_file, &suite, &tasks, &agents, &models, |pin| {
            bench::read_configuration_with(&tree, pin)
        })
        .expect("references resolve"),
        ["probes/recorded"]
    );
    assert_eq!(bench::check_tree_with(&tree, &valid, design).expect("valid frozen task"), 1);

    recorded_result_round_trips(&tree, &recorded.join("results/attempt.json"));
}

fn recorded_result_round_trips(tree: &Tree, file: &Path) {
    let result = bench::read_result_with(tree, file).expect("recorded result");
    let rendered = bench::render_result(&result).expect("versioned JSON");
    assert_eq!(bench::parse_result(file, &rendered).expect("round-trip"), result);
    assert!(matches!(result.spend.usd, bench::Measure::Unavailable { .. }));
    assert!(matches!(result.resources.cpu_ms, bench::Measure::LowerBound { .. }));
    assert_eq!(result.outcome.end, bench::End::Completed);
    assert_eq!(result.outcome.forced, bench::Forced::Killed);

    let results = bench::read_results_with(tree, file).expect("recorded attempts");
    let summary =
        bench::summarise(&results, "recorded", "small", bench::Design::Single, 123, None, None).expect("summary");
    assert_eq!(summary.groups[0].counts_by_end["completed"], 1);
    assert_eq!(summary.groups[0].metrics["task_wall_ms"].observed, 1);
    let baseline = bench::baseline(&summary).expect("single configuration baseline");
    let summary_file = Path::new("/summary.json");
    let baseline_file = Path::new("/baseline.json");
    bench::write_summary_with(tree, summary_file, &summary).expect("summary output");
    bench::write_summary_with(tree, baseline_file, &baseline).expect("baseline output");
    assert_eq!(bench::read_summary_with(tree, summary_file).expect("round-trip"), summary);
    assert_eq!(bench::read_baseline_with(tree, baseline_file).expect("round-trip"), baseline);
    assert!(bench::write_summary_with(tree, summary_file, &summary).is_err(), "committed evidence is not overwritten");
}
