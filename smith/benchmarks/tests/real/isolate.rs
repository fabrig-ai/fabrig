use smith_bench::{guard::Guard, isolate::remove_owned_tree};
use std::{fs, os::unix::fs::PermissionsExt};

#[test]
fn readonly_owned_cleanup_restores_directory_permissions_and_never_follows_links() {
    let scratch = skein_scratch::Scratch::new("bench-isolate");
    let root = scratch.path().to_path_buf().join(format!("smith-readonly-cleanup-{}", std::process::id()));
    let guard = Guard::new(&root.join("user"), &root.join("config"), &root.join("state"), &[]).expect("fixture guard");
    let owned = root.join("owned");
    let readonly = owned.join("spec");
    let outside = root.join("outside");
    fs::create_dir_all(&readonly).expect("owned layout");
    fs::create_dir_all(&outside).expect("owned outside control");
    fs::write(readonly.join("rules.txt"), "frozen").expect("readonly file");
    fs::write(outside.join("sentinel.txt"), "unchanged").expect("outside sentinel");
    std::os::unix::fs::symlink(&outside, owned.join("link")).expect("outside link");
    fs::set_permissions(&outside, fs::Permissions::from_mode(0o500)).expect("outside mode control");
    fs::set_permissions(&readonly, fs::Permissions::from_mode(0o500)).expect("readonly mode");
    assert!(fs::remove_file(readonly.join("rules.txt")).is_err(), "ordinary unlink cannot delete readonly layout");
    remove_owned_tree(&guard, &owned).expect("settled cleanup");
    assert!(!owned.exists());
    assert_eq!(fs::metadata(&outside).expect("outside unchanged").permissions().mode() & 0o777, 0o500);
    assert_eq!(fs::read_to_string(outside.join("sentinel.txt")).expect("outside unchanged"), "unchanged");
    let root_link = root.join("root-link");
    std::os::unix::fs::symlink(&outside, &root_link).expect("nonordinary root");
    assert!(remove_owned_tree(&guard, &root_link).is_err());
    assert_eq!(fs::read_to_string(outside.join("sentinel.txt")).expect("outside still unchanged"), "unchanged");
    fs::remove_file(root_link).expect("unlink fixture root link");
    remove_owned_tree(&guard, &root).expect("owned fixture cleanup");
}
