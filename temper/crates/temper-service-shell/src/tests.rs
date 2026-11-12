#[test]
fn each_box_adoption_refusal_renders_one_line_naming_what_it_lacks() {
    use temper_service::system::BoxRefusal;
    for (why, expected) in [
        (BoxRefusal::Sockets { limit: 1 }, "the box's pipes need two free socket slots; the service socket limit is 1"),
        (BoxRefusal::Adopted, "the box's pipes were already adopted"),
        (BoxRefusal::NoEngine, "the box's pipes need an engine"),
        (BoxRefusal::ZeroLine, "the box's pipes need a nonzero application.limits.overrides.box.line_bytes"),
    ] {
        assert_eq!(
            super::prefixed("temper serve", &super::box_refusal(why)),
            format!("temper serve: --box: {expected}\n")
        );
    }
}

#[test]
fn a_push_key_refuses_nonprivate_files_naming_its_member() {
    use std::os::unix::{ffi::OsStrExt, fs::PermissionsExt};
    let scratch = skein_scratch::Scratch::new("push-private-file");
    let path = scratch.path().join("push.pem");
    let name = || jig_protocol_config::File {
        at: jig_protocol_config::Path::root().member(b"application").member(b"push").member(b"key"),
        path: path.as_os_str().as_bytes().into(),
        max: 1024,
        private: true,
    };
    std::fs::write(&path, b"fixture").expect("key fixture file");
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).expect("nonprivate fixture");
    let why = super::read_files(Box::new([name()]), skein_shell::effective_user()).expect_err("nonprivate key refused");
    assert!(why.starts_with("application.push.key:"));
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).expect("private fixture");
    assert!(super::read_files(Box::new([name()]), skein_shell::effective_user()).is_ok());
}

#[test]
fn a_boxed_serve_reads_none_or_a_whole_secret() {
    use std::io::Cursor;
    use temper_service::clients::LINK_SECRET_BYTES;

    let mut empty = Cursor::new([]);
    let (printed, local) = super::given_secret(&mut empty).expect("empty input");
    assert!(printed.is_none() && local.is_none());
    let bytes = [23; LINK_SECRET_BYTES];
    let mut whole = Cursor::new(bytes);
    let (printed, local) = super::given_secret(&mut whole).expect("whole secret");
    assert!(printed.expect("given secret").expose() == bytes, "the secret is taken whole");
    assert!(local.is_none());
    let mut both = Cursor::new([vec![23; LINK_SECRET_BYTES], vec![37; LINK_SECRET_BYTES]].concat());
    let (printed, local) = super::given_secret(&mut both).expect("both given secrets");
    assert!(printed.expect("printed secret").expose() == bytes, "the printed secret comes first");
    assert!(local.expect("local secret").expose() == [37; LINK_SECRET_BYTES], "the local secret comes second");
    for length in [LINK_SECRET_BYTES - 1, LINK_SECRET_BYTES + 1, 63, 65] {
        let mut input = Cursor::new(vec![23; length]);
        assert_eq!(
            super::given_secret(&mut input).expect_err("partial or oversized secret"),
            "--box: stdin: expected the first link's secret"
        );
    }
    let mut oversized = Cursor::new(vec![23; LINK_SECRET_BYTES * 3]);
    assert!(super::given_secret(&mut oversized).is_err(), "oversized input is refused");
    assert_eq!(oversized.position(), 65);
}

#[test]
fn a_refused_start_is_its_binarys_serve_line() {
    use std::os::unix::ffi::OsStrExt;
    let scratch = skein_scratch::Scratch::new("serve-refused-start");
    let arguments = [
        Box::from(b"--config".as_slice()),
        scratch.path().join("config").as_os_str().as_bytes().into(),
        Box::from(b"--data".as_slice()),
        scratch.path().join("data").as_os_str().as_bytes().into(),
        Box::from(b"--cache".as_slice()),
        scratch.path().join("cache").as_os_str().as_bytes().into(),
    ];
    let (directories, _) = temper_shell::directories::resolve(&arguments, &[], b"/").expect("fixture directories");
    std::fs::create_dir(scratch.path().join("config")).expect("configuration directory");
    std::fs::write(directories.document(), b"{}").expect("document");
    assert_eq!(
        super::run("temper-worker", &directories, &[], false, |_, _, _| Err("refused".into())),
        Err("temper-worker serve: refused\n".into())
    );
}

#[test]
fn wait_check_uses_the_given_switch_outside_debug_builds() {
    let environment = [(Box::from(b"TEMPER_INTERNAL_WAIT_CHECK".as_slice()), Box::from(b"1".as_slice()))];
    assert!(super::host::wait_check(&environment, false));
    assert!(!super::host::wait_check(&[], false));
    let empty = [(Box::from(b"TEMPER_INTERNAL_WAIT_CHECK".as_slice()), Box::from(b"".as_slice()))];
    assert!(!super::host::wait_check(&empty, false));
    assert!(super::host::wait_check(&[], true));
}

#[test]
fn restart_cut_keeps_the_given_marker_only_with_the_wait_switch() {
    let cut = (Box::from(b"TEMPER_INTERNAL_CUT_AT_LISTEN".as_slice()), Box::from(b"/fixture/cut".as_slice()));
    let wait = (Box::from(b"TEMPER_INTERNAL_WAIT_CHECK".as_slice()), Box::from(b"1".as_slice()));
    assert_eq!(
        super::restart_fault::marker(&[wait.clone(), cut.clone()]),
        Some(std::path::PathBuf::from("/fixture/cut"))
    );
    assert_eq!(super::restart_fault::marker(&[cut]), None);
    assert_eq!(super::restart_fault::marker(&[wait]), None);
    assert_eq!(super::restart_fault::marker(&[]), None);
}

#[test]
fn local_link_uses_the_given_runtime_directory() {
    let arguments = ["--config", "/c", "--data", "/d", "--cache", "/k"].map(|word| Box::from(word.as_bytes()));
    let (directories, _) = temper_shell::directories::resolve(&arguments, &[], b"/").expect("directories");
    let environment = [(Box::from(b"XDG_RUNTIME_DIR".as_slice()), Box::from(b"/fixture/runtime".as_slice()))];
    let link = super::local_link(&directories, &environment).expect("local link");
    if cfg!(target_os = "linux") {
        assert_eq!(link.parent(), Some(std::path::Path::new("/fixture/runtime/temper")));
        assert_eq!(super::local_link(&directories, &[]), None);
    } else {
        assert_eq!(link, std::path::Path::new("/d/printed-link"));
    }
}
