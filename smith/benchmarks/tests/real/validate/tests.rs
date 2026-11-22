use smith_bench::{
    CatalogueTask, grade::GradeEnd, guard::Guard, isolate::Machine, testing::validate::validate_catalogue,
    validate::Validation,
};
use std::{fs, path::Path};

/// No test here proves this bound; the grader verdict each expects comes first.
const HANG_GUARD_SECONDS: u64 = 30;

fn fixture(name: &str, solved: bool) -> Validation {
    let scratch = skein_scratch::Scratch::new("bench-validate");
    let root = scratch.path().to_path_buf().join(format!("smith-bench-validate-{name}-{}", std::process::id()));
    fs::create_dir(&root).expect("fresh root");
    let task_root = root.join("task");
    fs::create_dir_all(task_root.join("seed")).expect("seed");
    fs::create_dir(task_root.join("reference")).expect("reference");
    if solved {
        fs::write(task_root.join("seed/solved"), "solution").expect("solution marker");
    }
    let mut task =
        crate::read_task(&Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/recorded/tasks/valid/task.toml"))
            .expect("recorded task");
    task.seed_sha256 = crate::seed_digest(&task_root.join("seed")).expect("seed digest");
    task.grade = vec![crate::Grade {
        command: vec!["sh".into(), "-c".into(), "test -f solved".into()],
        deadline_seconds: HANG_GUARD_SECONDS,
        max_output_bytes: 1024,
    }];
    let guard = Guard::new(&root.join("home"), &root.join("config"), &root.join("state"), &[]).expect("guard");
    let machine = Machine { path: "/usr/bin:/bin".into(), rustup_home: None };
    let entry = CatalogueTask { name: name.into(), manifest: task_root.join("task.toml"), task };
    let mut results = validate_catalogue(&[entry], &[], &guard, &machine, &root).expect("validation");

    results.pop().expect("one task")
}

#[test]
fn validate_reports_a_seed_its_grader_passes() {
    let result = fixture("passing-seed", true);
    assert_eq!(result.seed, GradeEnd::Passed);
    assert!(!result.passed());
}

#[test]
fn validate_reports_a_reference_its_grader_fails() {
    let result = fixture("failing-reference", false);
    assert_eq!(result.seed, GradeEnd::Failed);
    assert_eq!(result.reference, GradeEnd::Failed);
    assert!(!result.passed());
}
