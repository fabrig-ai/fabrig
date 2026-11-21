//! Smoke task manifests and decision boundaries use recorded and typed outside evidence.

fn smoke_catalogue_selects_five_active_probes_and_retains_deferred_one_turn_with_exact_frozen_inputs() {
    let root = crate::common::repository();
    let design = root.join("../docs/design");
    let tasks: Vec<_> = ["first-request", "smoke-tools", "large-write", "budget", "failures"]
        .into_iter()
        .map(|name| {
            let manifest = root.join("tasks/probes").join(name).join("task.toml");
            let task = smith_bench::check_task(&manifest, &design).expect("frozen seed and manifest");
            smith_bench::CatalogueTask { name: format!("probes/{name}"), manifest, task }
        })
        .collect();
    let models = smith_bench::read_model_tiers(&root.join("agents/models.toml")).expect("model tiers");
    for name in ["smoke", "calibrate"] {
        let path = root.join("suites").join(format!("{name}.toml"));
        let suite = smith_bench::read_suite(&path).expect("suite format");
        assert_eq!(
            smith_bench::validate_suite(&path, &suite, &tasks, &root.join("agents"), &models)
                .expect("pins/selectors")
                .iter()
                .filter(|task| *task != "probes/failures/one-turn")
                .count(),
            5
        );
    }
    let failures = tasks.iter().find(|task| task.name == "probes/failures").expect("authored deferred probe");
    let one_turn = failures.task.with_variant("one-turn").expect("offline only until10.5");
    assert_eq!(one_turn.budget.turns, Some(1));
    assert!(one_turn.deciding.iter().any(|name| name == "no-request"));
    let write = tasks.iter().find(|task| task.name == "probes/large-write").expect("large-write");
    let oversized = write.task.with_variant("oversized").expect("named replacement");
    assert_eq!(write.task.event.len(), 4, "original three assertions plus captured-file comparison");
    assert!(matches!(
        write.task.event[1],
        smith_bench::EventCheck::ToolCall { count: Some(smith_bench::Count::Exactly(1)), .. }
    ));
    assert!(oversized.event.iter().any(|check| matches!(
        check,
        smith_bench::EventCheck::Blocks { oversized: Some(smith_bench::Count::Exactly(1)), .. }
    )));
    assert!(
        !oversized.event.iter().any(|check| matches!(
            check,
            smith_bench::EventCheck::ToolCall { count: Some(smith_bench::Count::Exactly(1)), .. }
        )),
        "piecewise repair does not inherit the one-call predicate"
    );
    let smoke = tasks.iter().find(|task| task.name == "probes/smoke-tools").expect("smoke-tools");
    let child = smoke.task.with_variant("child").expect("historical additive behavior");
    assert!(smoke.task.event.iter().all(|check| child.event.contains(check)));
    let scratch = skein_scratch::Scratch::new("bench-probes");
    let directory = scratch.path().to_path_buf().join(format!("smith-probe-name-{}", std::process::id()));
    std::fs::create_dir(&directory).expect("isolated author typo");
    let manifest = directory.join("task.toml");
    let source = std::fs::read_to_string(&smoke.manifest).expect("authored manifest");
    std::fs::write(&manifest, source.replace("tree-ended", "tree-edned")).expect("one changed check name");
    assert!(
        smith_bench::check_task(&manifest, &design)
            .expect_err("misspelled deciding check refused")
            .key
            .starts_with("deciding")
    );

    let serialized = smith_bench::render_result(&result()).expect("historical result remains valid");
    assert_eq!(smith_bench::parse_result(&root, &serialized).expect("historical round trip"), result());
}

#[test]
fn the_smoke_retains_each_probes_frozen_checks_and_inputs() {
    smoke_catalogue_selects_five_active_probes_and_retains_deferred_one_turn_with_exact_frozen_inputs();
}

use crate::common::probe_result as result;
