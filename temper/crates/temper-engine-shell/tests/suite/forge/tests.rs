use temper_shell::directories;

fn setup_directories(root: &std::path::Path) -> directories::Directories {
    use std::os::unix::ffi::OsStrExt;
    let cache = root.join("cache");
    let args = [
        b"--config".as_slice(),
        root.as_os_str().as_bytes(),
        b"--data",
        root.as_os_str().as_bytes(),
        b"--cache",
        cache.as_os_str().as_bytes(),
    ]
    .map(Box::from);
    directories::resolve(&args, &[], b"/").unwrap().0
}

#[test]
fn prepare_flags_write_one_private_setup() {
    use super::setup_file;
    use std::os::unix::fs::PermissionsExt;
    let scratch = skein_scratch::Scratch::new("forge-prepare-file");
    let dirs = setup_directories(scratch.path());
    let flags =
        ["--listen", "127.0.0.1:3000", "--person", "ada", "ada@example.invalid", "--runner-label", "temper-host:host"]
            .map(Into::into);
    setup_file(&dirs, 1, "prepare", &flags).expect("first prepare writes its file");
    let path = scratch.path().join("forge-1/setup");
    assert_eq!(std::fs::metadata(&path).unwrap().permissions().mode() & 0o777, 0o600);
    assert_eq!(std::fs::metadata(path.parent().unwrap()).unwrap().permissions().mode() & 0o777, 0o700);
}
