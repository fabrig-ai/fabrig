use smith_bench::credential::logins::serial;
use std::os::unix::fs::DirBuilderExt;

#[test]
fn one_claude_login_holds_one_serial_lock() {
    let scratch = skein_scratch::Scratch::new("bench-logins");
    let root = scratch.path().to_path_buf().join(format!("smith-bench-serial-{}", std::process::id()));
    std::fs::DirBuilder::new().mode(0o700).create(&root).expect("private root");
    let guard =
        crate::guard::Guard::new(&root.join("home"), &root.join("config"), &root.join("state"), &[]).expect("guard");
    let login = root.join("state/smith-bench/logins/smith");
    let first = serial(&guard, &login).expect("first program attempt");
    let second = std::fs::File::open(root.join("state/smith-bench/locks/claude.lock")).expect("same lock");
    assert!(second.try_lock().is_err());
    drop(first);
    second.try_lock().expect("next program attempt after settlement");
    drop(second);
}
