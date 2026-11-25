use super::words::prepare_hint;
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

#[test]
fn prepare_flags_rows() {
    use super::{Command, PREPARE, SetupFacts, configure, setup_file_with};
    use std::{
        cell::{Cell, RefCell},
        collections::BTreeMap,
        ffi::OsString,
        io,
        path::{Path, PathBuf},
    };
    use temper_forge_setup::{
        machine::{self, BoxKind},
        setup_file::{self, SetupFile},
    };
    use temper_forge_setup_shell::setup_file as files;
    use temper_service_shell::jig_directories;

    let flags = ["--config", "/config", "--data", "/data", "--cache", "/cache"].map(|word| word.as_bytes().into());
    let (dirs, _) = directories::resolve(&flags, &[], b"/").unwrap();
    let jig = jig_directories(&dirs);
    let owned_path = Path::new("/data/forge-1/setup");
    let fake_path = Path::new("/data/playground-7/setup");
    let fake = setup_file::chosen(BoxKind::Fake, 3007, b"ada", false, &jig).unwrap();
    let bytes = RefCell::new(BTreeMap::from([(fake_path.to_path_buf(), setup_file::encode(&fake))]));
    let writes = Cell::new(0);
    let facts = SetupFacts {
        group_read: &|path, bound| {
            assert_eq!(bound, 16);
            if path == Path::new("/data/box/groups/7") {
                Ok(machine::group_file(BoxKind::Fake))
            } else {
                Err(skein_shell::Unread::Absent)
            }
        },
        group_stat: &|_| Err(io::ErrorKind::NotFound.into()),
        setup_present: &|path| Ok(bytes.borrow().contains_key(path)),
        setup_read: &|dirs, number, kind| {
            files::read_with(dirs, number, kind, |path, bound| {
                assert_eq!(bound, 65536);
                let bytes = bytes.borrow();
                Ok(skein_lib::Secret::new(bytes.get(path).ok_or(skein_shell::Unread::Absent)?.clone()))
            })
        },
        setup_write: &|dirs, number, setup| {
            let path: PathBuf = files::path(dirs, number, setup.kind());
            assert!(bytes.borrow_mut().insert(path, setup_file::encode(setup)).is_none(), "write once");
            writes.set(writes.get() + 1);
            Ok(())
        },
    };
    let prepare = |number, verb, words: &[&str]| {
        let arguments = words.iter().copied().map(OsString::from).collect::<Vec<_>>();
        setup_file_with(&dirs, number, verb, &arguments, &facts)
    };
    let flags =
        ["--listen", "127.0.0.1:3000", "--person", "ada", "ada@example.invalid", "--runner-label", "temper-host:host"];
    let file = prepare(1, "prepare", &flags).expect("first prepare writes its file");
    let SetupFile::Owned(owned) = &file else { panic!("prepare makes owned") };
    assert_eq!(owned.remote.as_ref(), b"http://127.0.0.1:3000");
    assert!(owned.fetch.forgejo && owned.fetch.runner);
    let runner = owned.runner.as_ref().unwrap();
    assert_eq!(runner.capacity, 1);
    assert_eq!(runner.path.as_ref(), b"/usr/local/bin:/usr/bin:/bin");
    let kept = bytes.borrow()[owned_path].clone();
    assert_eq!(writes.get(), 1);
    for words in [&flags[..], &[]] {
        assert_eq!(prepare(1, "prepare", words).unwrap(), file);
    }
    assert_eq!(
        prepare(1, "prepare", &["--listen", "127.0.0.1:3001"]).unwrap_err(),
        "setup differs at listen: the file has 127.0.0.1:3000, the flag 127.0.0.1:3001"
    );
    assert_eq!(bytes.borrow()[owned_path], kept);
    assert_eq!(writes.get(), 1, "repeat and differing flags must not write");

    let local = ["--listen", "127.0.0.1:3002", "--person", "ada", "ada@example.invalid", "--forgejo", "bin/forgejo"];
    let resolved = prepare(2, "prepare", &local).unwrap();
    assert_eq!(prepare(2, "prepare", &local).expect("the same relative path compares resolved"), resolved);
    assert!(prepare(3, "run", &[]).unwrap_err().contains("forge-3/setup"));
    bytes.borrow_mut().insert(owned_path.to_owned(), Box::from(b"{\"version\":2,\"kind\":\"owned\"}".as_slice()));
    let why = prepare(1, "prepare", &[]).unwrap_err();
    assert!(why.contains("forge-1/setup") && why.contains("Version(2)"), "{why}");
    assert_eq!(prepare(7, "run", &[]).unwrap(), fake);
    assert_eq!(writes.get(), 2, "only the two initial preparations write");

    let mut arguments = dirs.flags().into_vec();
    arguments.push(b"1".as_slice().into());
    arguments.extend(flags.map(|flag| flag.as_bytes().into()));
    let command = configure(&PREPARE, &arguments, &[], b"/").expect("prepare accepts its setup flags");
    let Command::Forge { directories, arguments, .. } = command else { panic!("forge prepare command") };
    assert_eq!(directories, dirs);
    assert_eq!(arguments, ["prepare", "1"].into_iter().chain(flags).map(OsString::from).collect::<Vec<_>>());
}
