use super::prepare_hint;
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

#[test]
fn each_prepare_flag_is_checked_by_its_setup_member() {
    use super::flags::Flags;
    let dirs = jig_protocol_config::Directories::given(b"/config", b"/data", b"/cache").unwrap();
    for (flag, values) in [
        ("--listen", &["192.0.2.1:3000"][..]),
        ("--remote", &["http://public.example/repo"][..]),
        ("--public", &["http://git.example.invalid"][..]),
        ("--person", &["bad/name", "ada@example.invalid"][..]),
        ("--person", &["ada", "not-an-address"][..]),
        ("--forgejo", &[""][..]),
        ("--runner-label", &[""][..]),
        ("--runner-binary", &[""][..]),
        ("--runner-capacity", &["0"][..]),
        ("--runner-capacity", &["65"][..]),
        ("--runner-path", &[""][..]),
        ("--sign-in", &["bad client"][..]),
    ] {
        let mut arguments = vec![
            "--listen",
            "127.0.0.1:3000",
            "--person",
            "ada",
            "ada@example.invalid",
            "--runner-label",
            "temper-host:host",
        ];
        if let Some(at) = arguments.iter().position(|value| *value == flag) {
            arguments.drain(at..at + if flag == "--person" { 3 } else { 2 });
        }
        arguments.push(flag);
        arguments.extend_from_slice(values);
        let arguments: Vec<_> = arguments.into_iter().map(Into::into).collect();
        let why = Flags::parse(&arguments).unwrap().make(&dirs).expect_err("bad flag is refused");
        assert!(why.contains(flag), "{flag}: {why}");
    }
}

#[test]
fn prepare_hint_names_the_running_binary() {
    let arguments =
        ["--config", "/config", "--data", "/data", "--cache", "/cache"].map(|word| Box::from(word.as_bytes()));
    let (directories, _) = directories::resolve(&arguments, &[], b"/").expect("explicit directories");
    for (binary, expected) in [
        ("temper", "not prepared: run temper forge prepare --config /config --data /data --cache /cache 17"),
        (
            "temper-engine",
            "not prepared: run temper-engine forge prepare --config /config --data /data --cache /cache 17",
        ),
    ] {
        assert_eq!(prepare_hint(binary, "not prepared", 17, &directories), expected, "{binary}");
    }
}

#[test]
fn inherited_listener_uses_the_given_descriptor_count() {
    use temper_forge_setup::spec::Which;
    let environment = [(Box::from(b"LISTEN_FDS".as_slice()), Box::from(b"2".as_slice()))];
    assert_eq!(
        super::inherited_listener(Which::Forgejo, &environment),
        Err("LISTEN_FDS: forgejo requires exactly 1 listener at fd 3".into())
    );
    assert_eq!(
        super::inherited_listener(Which::Runner, &environment),
        Err("LISTEN_FDS: runner takes no listeners (fd 3)".into())
    );
    assert_eq!(super::inherited_listener(Which::Forgejo, &[]), Ok(None));
    let empty = [(Box::from(b"LISTEN_FDS".as_slice()), Box::from(b"".as_slice()))];
    assert_eq!(super::inherited_listener(Which::Forgejo, &empty), Ok(None));
}
