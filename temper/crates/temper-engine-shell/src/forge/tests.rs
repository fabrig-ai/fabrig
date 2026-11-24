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
    let lookup = |_| panic!("refused or absent counts do not look up a listener");
    assert_eq!(
        super::inherited_listener(Which::Forgejo, &environment, lookup),
        Err("LISTEN_FDS: forgejo requires exactly 1 listener at fd 3".into())
    );
    assert_eq!(
        super::inherited_listener(Which::Runner, &environment, lookup),
        Err("LISTEN_FDS: runner takes no listeners (fd 3)".into())
    );
    assert_eq!(super::inherited_listener(Which::Forgejo, &[], lookup), Ok(None));
    let empty = [(Box::from(b"LISTEN_FDS".as_slice()), Box::from(b"".as_slice()))];
    assert_eq!(super::inherited_listener(Which::Forgejo, &empty, lookup), Ok(None));
}

#[test]
fn forge_run_reports_bad_listener_counts_before_any_lookup_or_read() {
    use super::{Command, Inputs, Preparation, RUN, configure, report_exit, start_with_preparation};
    use temper_config_shell::Shape;

    let inputs = Inputs {
        reader: |_, _| panic!("no deployment document before listener admission"),
        listener_at: |_| panic!("counts are checked before descriptor lookup"),
        setup: &|_, _, _, _| panic!("counts are checked before the forge setup read"),
        configuration: &|_, _| panic!("counts are checked before the configuration read"),
        sign_in: &|_, _, _, _, _| panic!("run does not sign in"),
    };
    let preparation = Preparation {
        directory: &|_, _| panic!("counts are checked before directory preparation and launch"),
        fake: &|_| panic!("counts are checked before token preparation"),
    };
    for (which, count) in [("forgejo", "0"), ("forgejo", "2"), ("forgejo", "many"), ("runner", "1"), ("runner", "0")] {
        let environment = [(Box::from(b"LISTEN_FDS".as_slice()), Box::from(count.as_bytes()))];
        let args = ["--config", "/no-document", "--data", "/unused", "--cache", "/unused", "1", which]
            .map(|word| Box::from(word.as_bytes()));
        let Command::Forge { directories, arguments, ssl_cert_file, environment } =
            configure(&RUN, &args, &environment, b"/").expect("forge run arguments")
        else {
            panic!("expected the forge command");
        };
        let result = start_with_preparation(
            "temper-box",
            Shape::Standalone,
            &directories,
            &arguments,
            ssl_cert_file,
            &environment,
            (&inputs, &preparation),
        );
        let mut diagnostic = String::new();
        let status = report_exit("temper-box", &arguments, result, |_| panic!("no stdout"), |line| diagnostic = line);
        assert_eq!(status, 1, "{which} {count}");
        assert!(diagnostic.contains("LISTEN_FDS") && diagnostic.contains("fd 3"), "{which} {count}: {diagnostic}");
        assert!(!diagnostic.contains("no-document"), "refuses before document read: {diagnostic}");
    }
}
