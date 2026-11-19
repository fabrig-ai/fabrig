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
        super::run(
            "temper-worker",
            &directories,
            temper_config_shell::Shape::WorkerAlone,
            &[],
            super::host::adapter_tests::SERVED,
            &skein_shell::Clock::new(),
            |_| Ok(()),
            |_, _, _, ()| Err::<super::Prepared<_, ()>, _>("refused".into()),
            |_, _, ()| Ok(())
        ),
        Err("temper-worker serve: refused\n".into())
    );
}

#[test]
fn toml_serve_maps_member_lines_and_keeps_other_problems() {
    use std::os::unix::ffi::OsStrExt;
    let scratch = skein_scratch::Scratch::new("serve-key-lines");
    let args = [
        b"--config".as_slice(),
        scratch.path().as_os_str().as_bytes(),
        b"--data",
        scratch.path().as_os_str().as_bytes(),
        b"--cache",
        scratch.path().as_os_str().as_bytes(),
    ]
    .map(Box::from);
    let (directories, _) = temper_shell::directories::resolve(&args, &[], b"/").expect("directories");
    std::fs::write(
        scratch.path().join(temper_config_shell::FILE),
        "# box\n\nlisten = \"127.0.0.1:7100\"\npublic = \"http://example.com\"\n",
    )
    .expect("box file");
    assert_eq!(
        super::run(
            "temper-box",
            &directories,
            temper_config_shell::Shape::Standalone,
            &[],
            super::host::adapter_tests::SERVED,
            &skein_shell::Clock::new(),
            |_| Ok(()),
            |_, _, _, ()| {
                Err::<super::Prepared<_, ()>, _>("application.clients.public: start's words\napplication.webhooks.listener: listener words\nagents.setting: unchanged\n".into())
            },
            |_, _, ()| Ok(())
        ),
        Err(format!(
            "temper-box serve: {}:4: public: start's words\ntemper-box serve: {}: webhooks (its default): listener words\ntemper-box serve: agents.setting: unchanged\n",
            temper_config_shell::FILE,
            temper_config_shell::FILE
        ))
    );
}

#[test]
fn toml_serve_passes_checked_programs_beside_the_document() {
    use std::os::unix::ffi::OsStrExt;
    let scratch = skein_scratch::Scratch::new("serve-file-programs");
    let args = [
        b"--config".as_slice(),
        scratch.path().as_os_str().as_bytes(),
        b"--data",
        scratch.path().as_os_str().as_bytes(),
        b"--cache",
        scratch.path().as_os_str().as_bytes(),
    ]
    .map(Box::from);
    let (directories, _) = temper_shell::directories::resolve(&args, &[], b"/").expect("directories");
    let program = scratch.path().join("program");
    std::fs::copy("/bin/sh", &program).expect("scratch executable");
    std::fs::write(scratch.path().join(temper_config_shell::FILE), format!(
        "listen = \"127.0.0.1:7100\"\n[programs]\ngit = {program:?}\nclaude = {program:?}\nforgejo = {program:?}\nrunner = {program:?}\n"
    )).expect("box file");
    assert_eq!(
        super::run(
            "temper-box",
            &directories,
            temper_config_shell::Shape::Standalone,
            &[],
            super::host::adapter_tests::SERVED,
            &skein_shell::Clock::new(),
            |_| Ok(()),
            |bytes, programs, _, ()| {
                assert_eq!(programs.git.as_deref(), Some(program.as_path()));
                assert_eq!(programs.claude.as_deref(), Some(program.as_path()));
                assert_eq!(programs.forgejo.as_deref(), Some(program.as_path()));
                assert_eq!(programs.runner.as_deref(), Some(program.as_path()));
                assert!(
                    !bytes.windows(b"\"programs\"".len()).any(|word| word == b"\"programs\""),
                    "file-only paths stay out of the document"
                );
                Err::<super::Prepared<_, ()>, _>("checked programs reached preparation".into())
            },
            |_, _, ()| Ok(()),
        ),
        Err("temper-box serve: checked programs reached preparation\n".into())
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
