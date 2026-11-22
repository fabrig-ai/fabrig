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
fn prepare_flags_write_one_private_setup_and_later_flags_must_agree() {
    use super::setup_file;
    use std::os::unix::{ffi::OsStrExt, fs::PermissionsExt};
    use temper_forge_setup::{machine::BoxKind, setup_file::SetupFile};
    let scratch = skein_scratch::Scratch::new("forge-prepare-file");
    let dirs = setup_directories(scratch.path());
    let flags =
        ["--listen", "127.0.0.1:3000", "--person", "ada", "ada@example.invalid", "--runner-label", "temper-host:host"]
            .map(Into::into);
    let file = setup_file(&dirs, 1, "prepare", &flags).expect("first prepare writes its file");
    let SetupFile::Owned(owned) = &file else { panic!("prepare makes owned") };
    assert_eq!(owned.remote.as_ref(), b"http://127.0.0.1:3000");
    assert!(owned.fetch.forgejo && owned.fetch.runner);
    let runner = owned.runner.as_ref().unwrap();
    assert_eq!(runner.capacity, 1);
    assert_eq!(runner.path.as_ref(), b"/usr/local/bin:/usr/bin:/bin");
    let path = scratch.path().join("forge-1/setup");
    let kept = std::fs::read(&path).unwrap();
    assert_eq!(std::fs::metadata(&path).unwrap().permissions().mode() & 0o777, 0o600);
    assert_eq!(std::fs::metadata(path.parent().unwrap()).unwrap().permissions().mode() & 0o777, 0o700);
    assert_eq!(setup_file(&dirs, 1, "prepare", &flags).unwrap(), file);
    assert_eq!(setup_file(&dirs, 1, "prepare", &[]).unwrap(), file);
    assert_eq!(
        setup_file(&dirs, 1, "prepare", &["--listen".into(), "127.0.0.1:3001".into()]).unwrap_err(),
        "setup differs at listen: the file has 127.0.0.1:3000, the flag 127.0.0.1:3001"
    );
    assert_eq!(std::fs::read(&path).unwrap(), kept);
    let local = ["--listen", "127.0.0.1:3002", "--person", "ada", "ada@example.invalid", "--forgejo", "bin/forgejo"]
        .map(Into::into);
    setup_file(&dirs, 2, "prepare", &local).unwrap();
    setup_file(&dirs, 2, "prepare", &local).expect("the same relative path compares resolved");
    assert!(setup_file(&dirs, 3, "run", &[]).unwrap_err().contains("forge-3/setup"));
    std::fs::write(&path, b"{\"version\":2,\"kind\":\"owned\"}").unwrap();
    let why = setup_file(&dirs, 1, "prepare", &[]).unwrap_err();
    assert!(why.contains("forge-1/setup") && why.contains("Version(2)"));
    let fake = temper_forge_setup::setup_file::chosen(
        BoxKind::Fake,
        3007,
        b"ada",
        false,
        &temper_service_shell::jig_directories(&dirs),
    )
    .unwrap();
    temper_forge_setup_shell::setup_file::write(&temper_service_shell::jig_directories(&dirs), 7, &fake).unwrap();
    assert_eq!(
        temper_forge_setup_shell::setup_file::read(&temper_service_shell::jig_directories(&dirs), 7, BoxKind::Fake)
            .unwrap(),
        fake
    );
    let mut arguments = dirs.flags().into_vec();
    arguments.push(b"1".as_slice().into());
    for flag in &flags {
        arguments.push(flag.as_bytes().into());
    }
    let command = super::configure(&super::PREPARE, &arguments, &[], b"/")
        .expect("prepare accepts its setup flags through the command grammar");
    let super::Command::Forge { arguments, .. } = command else { panic!("forge prepare command") };
    assert_eq!(arguments[0], "prepare");
}
