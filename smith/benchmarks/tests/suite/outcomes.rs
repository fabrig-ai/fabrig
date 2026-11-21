//! Neutral settled evidence supplies no inferred agent diagnostics or answers.

use std::path::Path;

#[test]
fn a_task_without_deciding_checks_is_refused() {
    let scratch = skein_scratch::Scratch::new("bench-outcomes");
    let root = scratch.path().to_path_buf().join(format!("smith-bench-no-deciding-{}", std::process::id()));
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
}
