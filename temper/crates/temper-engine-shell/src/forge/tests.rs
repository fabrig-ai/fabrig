use super::prepare_hint;
use temper_shell::directories;

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
