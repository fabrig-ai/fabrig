use smith_bench::{
    arm::resolve_commit,
    guard::Guard,
    testing::arm::{build_lock, git, output, outside_repositories},
};
use std::fs::{self, OpenOptions};

#[test]
fn a_build_waits_while_another_holds_the_build_lock() {
    let scratch = skein_scratch::Scratch::new("bench-arm");
    let root = scratch.path().to_path_buf().join(format!("smith-build-lock-{}", std::process::id()));
    fs::create_dir(&root).expect("isolated fixture");
    let guard = Guard::new(&root.join("home"), &root.join("config"), &root.join("state"), &[]).expect("guard");
    let owner = build_lock(&guard, &root).expect("first build owns its lock");
    let contender = OpenOptions::new().write(true).open(root.join("build.lock")).expect("second build's file");
    assert!(matches!(contender.try_lock(), Err(std::fs::TryLockError::WouldBlock)));
    assert!(!root.join("clone").exists());
    owner.unlock().expect("first build ends");
    contender.try_lock().expect("second build can now start");
    contender.unlock().expect("second build ends");
}

#[test]
fn dirty_and_unknown_commits_are_refused_before_any_build_write() {
    let scratch = skein_scratch::Scratch::new("bench-arm");
    let root = scratch.path().to_path_buf().join(format!("smith-bench-arm-source-{}", std::process::id()));
    fs::create_dir(&root).expect("isolated source");
    output(git(&root).args(["init", "--quiet"]), "test git init").expect("init");
    output(
        git(&root).args([
            "-c",
            "user.name=Benchmark test",
            "-c",
            "user.email=benchmark@example.invalid",
            "commit",
            "--quiet",
            "--allow-empty",
            "-m",
            "synthetic seed",
        ]),
        "test commit",
    )
    .expect("commit");
    let commit = resolve_commit(&root, "HEAD").expect("clean commit");
    assert_eq!(commit.len(), 40);
    assert!(resolve_commit(&root, "unknown-commit").is_err());
    assert!(resolve_commit(&root, "--help").is_err());
    fs::write(root.join("dirty"), "synthetic untracked change").expect("dirty source");
    assert!(resolve_commit(&root, "HEAD").expect_err("dirty source").detail.contains("dirty"));
    assert!(outside_repositories(&root.join("private/arms")).is_err());
}
