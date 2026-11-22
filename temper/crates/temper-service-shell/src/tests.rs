fn scripted_serve<T>(
    binary: &str,
    shape: temper_config_shell::Shape,
    text: &[u8],
    is_executable: impl Fn(&std::path::Path) -> bool,
    prepare: impl FnOnce(&[u8], &temper_config_shell::Programs) -> Result<T, String>,
) -> Result<T, String> {
    let facts = temper_config_shell::Facts { login: "tester".into(), hostname: "fixture".into(), provider_key: false };
    let temper_config_shell::Read { file, document, keys } =
        temper_config_shell::parse(text, shape, &facts).expect("fixture document");
    let read = super::configuration::Read { file, document, keys };
    let programs = read.programs_with(is_executable).expect("fixture programs");
    super::prepare_serve(&read, &programs, prepare).map_err(|why| super::serve_error(binary, &why))
}

#[test]
fn a_refused_start_is_its_binarys_serve_line() {
    assert_eq!(
        scripted_serve(
            "temper-worker",
            temper_config_shell::Shape::WorkerAlone,
            b"engine = \"127.0.0.1:7200\"\n",
            |_| false,
            |_, _| Err::<(), _>("refused".into()),
        ),
        Err("temper-worker serve: refused\n".into())
    );
}

#[test]
fn toml_serve_maps_member_lines_and_keeps_other_problems() {
    assert_eq!(
        scripted_serve(
            "temper-box",
            temper_config_shell::Shape::Standalone,
            b"# box\n\nlisten = \"127.0.0.1:7100\"\npublic = \"http://example.com\"\n",
            |_| false,
            |_, _| {
                Err::<(), _>("application.clients.public: start's words\napplication.webhooks.listener: listener words\nagents.setting: unchanged\n".into())
            },
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
    let program = std::path::Path::new("/fixture/program");
    let text = format!(
        "listen = \"127.0.0.1:7100\"\n[programs]\ngit = {program:?}\nclaude = {program:?}\nforgejo = {program:?}\nrunner = {program:?}\n"
    );
    assert_eq!(
        scripted_serve(
            "temper-box",
            temper_config_shell::Shape::Standalone,
            text.as_bytes(),
            |path| path == program,
            |bytes, programs| {
                assert_eq!(programs.git.as_deref(), Some(program));
                assert_eq!(programs.claude.as_deref(), Some(program));
                assert_eq!(programs.forgejo.as_deref(), Some(program));
                assert_eq!(programs.runner.as_deref(), Some(program));
                assert!(
                    !bytes.windows(b"\"programs\"".len()).any(|word| word == b"\"programs\""),
                    "file-only paths stay out of the document"
                );
                Err::<(), _>("checked programs reached preparation".into())
            },
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
