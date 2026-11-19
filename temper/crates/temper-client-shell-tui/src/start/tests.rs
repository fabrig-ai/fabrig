use super::{self as tui, Refused};
use skein_ui_screen_terminal::Depth;
use std::{fs, os::unix::fs::PermissionsExt};
use temper_client_tui::{Note, Origin, Scheme};

const LINK: &str = "http://127.0.0.1:8367/jig/v1/sign-in/link?secret=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";

fn environment(entries: &[(&str, &str)]) -> Vec<super::EnvironmentEntry> {
    entries.iter().map(|(name, value)| (Box::from(name.as_bytes()), Box::from(value.as_bytes()))).collect()
}

#[test]
fn an_address_is_an_engine_on_loopback_and_never_a_link() {
    for (argument, expected) in [
        ("http://127.0.0.1:8367", Ok(Origin::Http("127.0.0.1:8367".parse().expect("IPv4")))),
        ("http://127.0.0.1:8367/", Ok(Origin::Http("127.0.0.1:8367".parse().expect("IPv4")))),
        ("http://[::1]:9000", Ok(Origin::Http("[::1]:9000".parse().expect("IPv6")))),
        ("http://10.0.0.1:8367", Err(Refused::NotLoopback)),
        (LINK, Err(Refused::Link)),
        ("http://127.0.0.1:8367/sign-in?secret=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA", Err(Refused::Link)),
        ("http://127.0.0.1:8367/sign-in?secret=S", Err(Refused::Link)),
        ("http://127.0.0.1:8367/jig/v1/sign-in/link?secret=S", Err(Refused::Link)),
        ("127.0.0.1:8367", Err(Refused::NotAnAddress(Box::from(b"127.0.0.1:8367".as_slice())))),
        ("http://localhost:8367", Err(Refused::HttpName)),
    ] {
        assert_eq!(tui::address(argument.as_bytes()), expected, "{argument}");
        if expected == Err(Refused::Link) {
            assert_eq!(
                expected.expect_err("link refused").line("temper"),
                "temper: a sign-in link is never an argument: run temper http://HOST:PORT and paste it\n"
            );
        }
    }
}

#[test]
fn origins_refuse_with_the_arguments_words() {
    for (argument, reason) in [
        (
            "http://dev.fabrig.ai".to_owned(),
            "http:// reaches an engine on this machine alone: use https://NAME".to_owned(),
        ),
        ("https://127.0.0.1".to_owned(), "https:// needs the deployment's name".to_owned()),
        ("http://10.0.0.1:8367".to_owned(), "the engine's address must be on loopback".to_owned()),
    ]
    .into_iter()
    .chain(
        [
            "https://dev.fabrig.ai/x".to_owned(),
            "https://box.123".to_owned(),
            "https://-a.fabrig.ai".to_owned(),
            format!("https://{}.ai", "a".repeat(64)),
        ]
        .into_iter()
        .map(|argument| {
            let reason = format!("not an address: {argument}");
            (argument, reason)
        }),
    ) {
        let refused = tui::address(argument.as_bytes()).expect_err("refused argument");
        assert_eq!(refused.line("temper"), format!("temper: {reason}\n"));
    }
}

#[test]
fn named_origins_resolve_once_and_use_machine_roots_for_tls() {
    use jig_client_shell_native::Engine;
    use skein_lib::{
        Writer,
        base64::{self, Alphabet},
    };
    use std::cell::Cell;
    let scratch = skein_scratch::Scratch::new("tui-roots");
    let root = include_bytes!("../../../../../skein/tests/tls/fixtures/root.der");
    let mut writer = Writer::new(
        usize::try_from(
            base64::encoded_len(Alphabet::Standard, u32::try_from(root.len()).expect("root length"))
                .expect("base64 length"),
        )
        .expect("writer length"),
    );
    base64::encode(Alphabet::Standard, root, &mut writer).expect("root base64");
    let pem = format!(
        "-----BEGIN CERTIFICATE-----\n{}\n-----END CERTIFICATE-----\n",
        String::from_utf8_lossy(&writer.finish())
    );
    let bundle = scratch.path().join("roots.pem");
    fs::write(&bundle, pem).expect("fixture roots");
    let entries = environment(&[("SSL_CERT_FILE", bundle.to_str().expect("fixture path"))]);
    for (argument, port, host) in
        [("https://dev.fabrig.ai", 443, "dev.fabrig.ai"), ("https://dev.fabrig.ai:8443", 8443, "dev.fabrig.ai:8443")]
    {
        let calls = Cell::new(0_u32);
        let addresses =
            vec![std::net::SocketAddr::from(([192, 0, 2, 1], port)), "[2001:db8::1]:443".parse().expect("IPv6")];
        let resolve = |name: &str, selected_port| {
            assert_eq!(name, "dev.fabrig.ai");
            assert_eq!(selected_port, port);
            calls.set(calls.get().checked_add(1).expect("one resolution"));
            let mut addresses = addresses.clone();
            for address in &mut addresses {
                address.set_port(selected_port);
            }
            Ok(addresses)
        };
        let origin = tui::address(argument.as_bytes()).expect("named origin");
        let Engine::Tls(tls) = tui::engine(&origin, &entries, &resolve).expect("TLS settings") else {
            panic!("TLS engine")
        };
        assert_eq!(calls.get(), 1);
        assert_eq!(tls.name.server(), skein_tls::Server::Dns("dev.fabrig.ai"));
        assert_eq!(tls.host.as_ref(), host.as_bytes());
        let mut expected = addresses.clone();
        for address in &mut expected {
            address.set_port(port);
        }
        assert_eq!(tls.addresses.as_ref(), expected);
    }
    let origin = tui::address(b"http://127.0.0.1:8367").expect("plain origin");
    let Engine::Plain(skein_io::Addr::Ip(socket)) =
        tui::engine(&origin, &[], &|_, _| panic!("HTTP never resolves")).expect("plain engine")
    else {
        panic!("plain IP engine")
    };
    assert_eq!(Origin::Http(socket), origin);
}

#[test]
fn named_start_refusals_keep_the_resolver_and_roots_words() {
    let origin = tui::address(b"https://dev.fabrig.ai").expect("origin");
    for (addresses, why) in
        [(Ok(Vec::new()), "no address"), (Err(std::io::Error::other("resolver refused")), "resolver refused")]
    {
        let result = tui::engine(&origin, &[], &|_, _| match &addresses {
            Ok(addresses) => Ok(addresses.clone()),
            Err(why) => Err(std::io::Error::other(why.to_string())),
        });
        let refusal = result.expect_err("resolution refused");
        assert_eq!(refusal.line("temper"), format!("temper: can't find dev.fabrig.ai: {why}\n"));
    }
    let scratch = skein_scratch::Scratch::new("tui-missing-roots");
    let missing = scratch.path().join("missing.pem");
    let entries = environment(&[("SSL_CERT_FILE", missing.to_str().expect("fixture path"))]);
    let refusal =
        tui::engine(&origin, &entries, &|_, port| Ok(vec![std::net::SocketAddr::from(([192, 0, 2, 1], port))]))
            .expect_err("missing roots");
    assert_eq!(refusal.line("temper"), format!("temper: the machine's trust roots: {}: Absent\n", missing.display()));
}

#[test]
fn an_explicit_name_never_reads_the_local_link_and_a_bare_start_prefers_it_to_a_kept_name() {
    use std::path::PathBuf;
    let origin = tui::address(b"https://dev.fabrig.ai").expect("origin");
    let local = tui::address(b"http://127.0.0.1:8367").expect("local");
    let file = Some(PathBuf::from("/engine/link"));
    let (selected, link, boxed) =
        tui::select(Some(origin.clone()), None, file.clone(), &|_| panic!("a name reads no engine link"));
    assert_eq!(selected, origin);
    assert!(link.is_none());
    assert!(!boxed);
    let (selected, link, boxed) = tui::select(None, Some(origin.clone()), file.clone(), &|_| None);
    assert_eq!(selected, origin);
    assert!(link.is_none());
    assert!(!boxed);
    let (selected, link, boxed) =
        tui::select(None, Some(origin), file.clone(), &|_| Some(skein_lib::Secret::new(LINK.as_bytes().into())));
    assert_eq!(selected, local);
    assert_eq!(link, file);
    assert!(boxed);
}

#[test]
fn the_recording_line_names_the_whole_path_and_last_result() {
    let name = b"recording-20261115T090032Z".as_slice();
    for (saved, expected) in [
        (None, ""),
        (Some((name, true)), "temper: recorded to /state/temper/tui/recording-20261115T090032Z\n"),
        (Some((name, false)), "temper: couldn't save the recording to /state/temper/tui/recording-20261115T090032Z\n"),
    ] {
        let mut stderr = Vec::new();
        super::say_recording(std::path::Path::new("/state/temper/tui"), saved, &mut stderr);
        assert_eq!(stderr, expected.as_bytes());
    }
}

#[test]
fn recording_requires_a_nonempty_captured_variable() {
    for (entries, expected) in [
        (vec![], false),
        (vec![("TEMPER_INTERNAL_RECORD", "")], false),
        (vec![("TEMPER_INTERNAL_RECORD", "1")], true),
        (vec![("TEMPER_INTERNAL_RECORD", "no")], true),
        (vec![("OTHER", "1")], false),
    ] {
        assert_eq!(tui::recording(&environment(&entries)), expected, "{entries:?}");
    }
}

#[test]
fn fixed_start_is_a_nonempty_captured_variable_on_loopback_only() {
    let start = skein_shell::FixedStart { wall: skein_lib::Wall::from_nanos(1_767_690_000_000_000_000), seed: 7 };
    for (host, value, expected) in [
        ("127.0.0.1:8367", Some("1767690000,7"), Ok(Some(start))),
        ("[::1]:9000", Some("1767690000,7"), Ok(Some(start))),
        ("10.0.0.1:8367", Some("1767690000,7"), Ok(None)),
        ("127.0.0.2:8367", Some("1767690000,7"), Ok(None)),
        ("10.0.0.1:8367", Some("bad"), Ok(None)),
        ("127.0.0.1:8367", None, Ok(None)),
        ("127.0.0.1:8367", Some(""), Ok(None)),
        ("127.0.0.1:8367", Some("bad"), Err(Refused::FixedStart)),
        ("127.0.0.1:8367", Some("01767690000,7"), Err(Refused::FixedStart)),
        ("127.0.0.1:8367", Some("18446744073709551615,7"), Err(Refused::FixedStart)),
    ] {
        let entries = value.map(|value| ("TEMPER_INTERNAL_FIXED_START", value)).into_iter().collect::<Vec<_>>();
        assert_eq!(
            tui::fixed_start(&environment(&entries), host.parse().expect("engine")),
            expected,
            "{host}: {value:?}"
        );
    }
    assert_eq!(Refused::FixedStart.line("temper"), "temper: TEMPER_INTERNAL_FIXED_START: expected SECONDS,SEED\n");
}

#[test]
fn a_fixed_run_draws_its_seed_on_the_tuis_stream_and_keeps_the_starts_first_reading() {
    let start = skein_shell::FixedStart::read(b"1767690000,7").expect("fixed start");
    let (seed, clock) = super::fixed_run(&start);
    let mut bytes = [0u8; 8];
    start.draw(4, &mut bytes);
    assert_eq!(seed, u64::from_le_bytes(bytes), "the seed given to local_run");
    assert_ne!(seed, start.seed);
    assert_eq!(clock.now(), skein_shell::Now { now: skein_shell::FixedStart::NOW, wall: start.wall });
}

#[test]
fn the_fixed_tui_and_page_share_no_request_keys() {
    fn keys(limits: &jig_client::DomainLimits, seed: u64) -> Vec<skein_lib::RequestKey> {
        // Domain::new gives this seed to RequestTable::new (jig-client-domain/src/domain.rs).
        let mut table = skein_lib::RequestTable::new(&limits.requests, seed);
        let mut keys = Vec::new();
        for _ in 0..64 {
            let admitted = table.ask((), 1, limits.ask_bytes, ()).expect("request fits the client's table");
            keys.push(admitted.key);
            table.not_kept(admitted.key, admitted.number).expect("retire the request to release its capacity");
            let mut outputs = table.take(skein_lib::Time::ZERO, skein_lib::Wall::EPOCH);
            while outputs.next_out().is_some() {}
            table.reclaim();
        }
        keys
    }

    let start = skein_shell::FixedStart::read(b"1767690000,7").expect("fixed start");
    let (seed, _) = super::fixed_run(&start);
    let tui = keys(&crate::limits::tui().native.client.client.domain, seed);
    let page = keys(&temper_client_shell_web::limits::client().client.domain, start.seed);
    let shared = tui.iter().filter(|key| page.contains(key)).count();
    assert_eq!(shared, 0, "shared request keys among each client's first 64");
}

#[test]
fn the_directory_is_the_state_directorys() {
    for (entries, expected) in [
        (vec![("XDG_STATE_HOME", "/s"), ("HOME", "/h")], Some("/s/temper/tui")),
        (vec![("XDG_STATE_HOME", "rel"), ("HOME", "/h")], Some("/h/.local/state/temper/tui")),
        (vec![("HOME", "/h")], Some("/h/.local/state/temper/tui")),
        (vec![], None),
        (vec![("HOME", "relative")], None),
    ] {
        let expected = expected
            .map(std::path::PathBuf::from)
            .ok_or_else(|| Refused::Directory("TUI directory: XDG_STATE_HOME or HOME must be an absolute path".into()));
        assert_eq!(tui::directory(&environment(&entries)), expected, "{entries:?}");
    }
}

#[test]
fn the_kept_address_reads_or_is_none() {
    for (bytes, expected) in [
        ("http://127.0.0.1:9000", Some(Origin::Http("127.0.0.1:9000".parse().expect("address")))),
        ("", None),
        ("garbage", None),
        ("https://dev.fabrig.ai", Some(Origin::parse(b"https://dev.fabrig.ai").expect("origin"))),
        ("https://dev.fabrig.ai:8443/", Some(Origin::parse(b"https://dev.fabrig.ai:8443").expect("origin"))),
        ("http://127.0.0.1:9000/", Some(Origin::Http("127.0.0.1:9000".parse().expect("address")))),
        ("http://10.0.0.1:9000", None),
        (LINK, None),
        ("http://127.0.0.1:9000\n", None),
    ] {
        assert_eq!(tui::kept(bytes.as_bytes()), expected, "{bytes}");
    }
}

#[test]
fn the_scheme_and_the_depth_follow_the_environment() {
    let mut failures = Vec::new();
    for (value, expected) in [
        (Some("0;15"), Some(Scheme::Light)),
        (Some("15;0"), Some(Scheme::Dark)),
        (Some("7"), Some(Scheme::Light)),
        (Some("0;default;8"), Some(Scheme::Dark)),
        (Some("default;default"), None),
        (Some(""), None),
        (None, None),
    ] {
        let entries = value.map(|value| ("COLORFGBG", value)).into_iter().collect::<Vec<_>>();
        if tui::scheme(&environment(&entries)) != expected {
            failures.push(value);
        }
    }
    assert!(failures.is_empty(), "COLORFGBG rows: {failures:?}");
    for (value, expected) in [
        (Some("truecolor"), Depth::True),
        (Some("24bit"), Depth::True),
        (None, Depth::Indexed),
        (Some("256color"), Depth::Indexed),
    ] {
        let entries = value.map(|value| ("COLORTERM", value)).into_iter().collect::<Vec<_>>();
        assert_eq!(tui::depth(&environment(&entries)), expected, "{value:?}");
    }
}

#[test]
fn the_address_prefers_given_then_file_then_kept_then_default() {
    let given = Origin::Http("[::1]:9000".parse().expect("given address"));
    let saved = Origin::Http("127.0.0.1:9001".parse().expect("kept address"));
    let default = Origin::Http("127.0.0.1:8367".parse().expect("default address"));
    let file_engine = Origin::Http("127.0.0.1:9002".parse().expect("file address"));
    let link = LINK.replacen(":8367", ":9002", 1);
    let off_loopback = link.replacen("127.0.0.1", "10.0.0.1", 1);
    let mut failures = Vec::new();
    for (row, argument, file, kept, expected) in [
        ("given", Some(given.clone()), Some(link.as_str()), Some(saved.clone()), given),
        ("file", None, Some(link.as_str()), Some(saved.clone()), file_engine),
        ("kept", None, None, Some(saved.clone()), saved.clone()),
        ("default", None, None, None, default),
        ("file address", None, Some("http://127.0.0.1:9002"), Some(saved.clone()), saved.clone()),
        ("off loopback", None, Some(off_loopback.as_str()), Some(saved.clone()), saved.clone()),
        ("garbage", None, Some("garbage"), Some(saved.clone()), saved.clone()),
    ] {
        let engine = tui::selected_address(argument, file.map(str::as_bytes), kept);
        if engine != expected {
            failures.push(row);
        }
        let (_, secret) = LINK.split_once("secret=").expect("fixture secret");
        assert!(!String::from_utf8_lossy(&engine.print()).contains(secret), "only the engine address leaves selection");
    }
    assert!(failures.is_empty(), "address rows: {failures:?}");
}

#[test]
fn on_a_box_requires_a_link_for_the_selected_engine() {
    let engine = "127.0.0.1:8367".parse().expect("engine address");
    let other = LINK.replacen(":8367", ":9000", 1);
    for (link, expected) in [
        (Some(LINK.as_bytes()), true),
        (Some(other.as_bytes()), false),
        (None, false),
        (Some(b"not a link".as_slice()), false),
        (Some(b"http://127.0.0.1:8367".as_slice()), false),
    ] {
        assert_eq!(tui::on_box(link, &Origin::Http(engine)), expected);
    }
}

#[test]
fn the_box_configuration_is_read_once_before_fetching() {
    use std::{cell::Cell, io::ErrorKind, path::Path};
    for (name, has_entry, error, expected) in [
        ("absent", false, Some(ErrorKind::NotFound), false),
        ("empty", false, None, false),
        ("an entry", true, None, true),
        ("unreadable", false, Some(ErrorKind::PermissionDenied), true),
        ("not a directory", false, Some(ErrorKind::NotADirectory), true),
    ] {
        let reads = Cell::new(0_u32);
        let read = |directory: &Path| {
            assert_eq!(directory, Path::new("/resolved/config"));
            reads.set(reads.get().checked_add(1).expect("one read"));
            error.map_or(Ok(has_entry), |kind| Err(std::io::Error::from(kind)))
        };
        assert_eq!(tui::box_set_up(b"/resolved/config", &read), expected, "{name}");
        assert_eq!(reads.get(), 1, "{name}");
    }
}

#[test]
fn the_box_log_uses_the_deployment_directories_resolved_for_a_lend() {
    for (arguments, entries, data) in [
        (vec![], vec![("HOME", "/home/person")], "/home/person/.local/share/temper"),
        (vec![], vec![("HOME", "/home/person"), ("TEMPER_DATA", "deployment")], "/working/deployment"),
        (vec!["--data", "chosen"], vec![("HOME", "/home/person"), ("TEMPER_DATA", "/ignored")], "/working/chosen"),
    ] {
        let arguments: Vec<Box<[u8]>> = arguments.iter().map(|arg| arg.as_bytes().into()).collect();
        let (directories, _) = temper_shell::directories::resolve(&arguments, &environment(&entries), b"/working")
            .expect("box directories");
        assert_eq!(directories.data.as_ref(), data.as_bytes());
        assert_eq!(tui::box_log(&directories.data).as_ref(), format!("{data}/box/logs/box.log").as_bytes());
    }
}

#[test]
fn the_link_path_is_the_same_deployments_as_the_box() {
    let expected = temper_shell::link_file::path(b"/working/deployment", Some(b"/runtime"), cfg!(target_os = "linux"));
    for (state, extra) in [
        ("/state-auto", vec![]),
        ("/state-retry", vec![]),
        ("/state-bothenv", vec![("TEMPER_CONFIG", "/box/config"), ("TEMPER_CACHE", "/box/cache")]),
    ] {
        let mut entries = vec![
            ("HOME", "/home/test"),
            ("TEMPER_DATA", "deployment"),
            ("XDG_RUNTIME_DIR", "/runtime"),
            ("XDG_STATE_HOME", state),
        ];
        entries.extend(extra);
        let entries = environment(&entries);
        let (directories, _) = temper_shell::directories::resolve(&[], &entries, b"/working").expect("box directories");
        assert_eq!(tui::link_path(&entries, b"/working"), expected, "{state}");
        assert_eq!(
            tui::link_path(&entries, b"/working"),
            temper_shell::link_file::path(&directories.data, Some(b"/runtime"), cfg!(target_os = "linux")),
        );
        assert_eq!(tui::directory(&entries).expect("TUI state"), std::path::Path::new(state).join("temper/tui"));
    }
}

#[test]
fn the_note_is_private_and_only_for_its_address() {
    let scratch = skein_scratch::Scratch::new("tui-note");
    let (directory, _) = prepare(&scratch);
    let path = directory.join("sign-in");
    let engine = "127.0.0.1:8367".parse().expect("note address");
    let other = "[::1]:9000".parse().expect("other address");
    let user = skein_shell::effective_user();
    let until = Note::Until(skein_lib::Wall::from_nanos(123));
    let mut failures = Vec::new();
    for (row, bytes, mode, owner, address, expected) in [
        ("own address", Some(until.print(&temper_client_tui::Origin::Http(engine))), 0o600, user, engine, Some(until)),
        ("other address", Some(until.print(&temper_client_tui::Origin::Http(engine))), 0o600, user, other, None),
        (
            "signed out",
            Some(Note::SignedOut.print(&temper_client_tui::Origin::Http(engine))),
            0o600,
            user,
            engine,
            Some(Note::SignedOut),
        ),
        ("nonprivate", Some(until.print(&temper_client_tui::Origin::Http(engine))), 0o640, user, engine, None),
        (
            "foreign",
            Some(until.print(&temper_client_tui::Origin::Http(engine))),
            0o600,
            user.wrapping_add(1),
            engine,
            None,
        ),
        ("garbage", Some(Box::from(b"garbage".as_slice())), 0o600, user, engine, None),
        ("absent", None, 0o600, user, engine, None),
    ] {
        if let Some(bytes) = bytes {
            fs::write(&path, bytes).expect("fixture note");
            fs::set_permissions(&path, fs::Permissions::from_mode(mode)).expect("note permissions");
        } else {
            fs::remove_file(&path).expect("remove fixture note");
        }
        if tui::read_note(&directory, &Origin::Http(address), owner) != expected {
            failures.push(row);
        }
    }
    assert!(failures.is_empty(), "note rows: {failures:?}");
}

#[test]
fn an_opener_needs_a_local_display_and_an_executable_in_an_absolute_path_directory() {
    for (entries, linux, found, expected) in [
        (vec![("SSH_CONNECTION", "set"), ("DISPLAY", ":0"), ("PATH", "/bin")], true, vec!["/bin/xdg-open"], None),
        (vec![("SSH_TTY", "set"), ("PATH", "/bin")], false, vec!["/bin/open"], None),
        (vec![("SSH_TTY", ""), ("PATH", "/bin")], false, vec!["/bin/open"], Some("/bin/open")),
        (vec![("PATH", "/bin")], true, vec!["/bin/xdg-open"], None),
        (vec![("DISPLAY", ":0"), ("PATH", "/bin")], true, vec!["/bin/xdg-open"], Some("/bin/xdg-open")),
        (vec![("WAYLAND_DISPLAY", "wayland-0"), ("PATH", "/bin")], true, vec!["/bin/xdg-open"], Some("/bin/xdg-open")),
        (vec![("PATH", "/bin")], false, vec!["/bin/open"], Some("/bin/open")),
        (
            vec![("DISPLAY", ":0"), ("PATH", "relative:/bin")],
            true,
            vec!["relative/xdg-open", "/bin/xdg-open"],
            Some("/bin/xdg-open"),
        ),
        (vec![("DISPLAY", ":0"), ("PATH", "relative:/empty")], true, vec!["relative/xdg-open"], None),
        (
            vec![("DISPLAY", ":0"), ("PATH", "/empty:/found:/later")],
            true,
            vec!["/found/xdg-open", "/later/xdg-open"],
            Some("/found/xdg-open"),
        ),
        (vec![("DISPLAY", ":0")], true, vec!["/bin/xdg-open"], None),
    ] {
        let env = environment(&entries);
        let opener = tui::opener(&env, linux, &|path| found.iter().any(|found| path == std::path::Path::new(found)));
        assert_eq!(
            opener.as_ref().map(|opener| opener.program.as_ref()),
            expected.map(str::as_bytes),
            "{entries:?} {linux}"
        );
        if let Some(opener) = opener {
            let expected = entries
                .iter()
                .map(|(name, value)| format!("{name}={value}").into_bytes().into_boxed_slice())
                .collect::<Vec<_>>();
            assert_eq!(opener.env.as_ref(), expected.as_slice());
        }
    }
}

#[test]
fn the_lines_name_the_binary() {
    for (name, lines) in [
        (
            "temper",
            [
                "temper: the TUI needs a terminal on stdin and stdout\n",
                "temper: not an address: ARG\n",
                "temper: a sign-in link is never an argument: run temper http://HOST:PORT and paste it\n",
                "temper: the engine's address must be on loopback\n",
                "temper: /s/temper/tui: cannot open its files\n",
                "temper: no command; temper --help lists them\n",
            ],
        ),
        (
            "temper-box",
            [
                "temper-box: the TUI needs a terminal on stdin and stdout\n",
                "temper-box: not an address: ARG\n",
                "temper-box: a sign-in link is never an argument: run temper-box http://HOST:PORT and paste it\n",
                "temper-box: the engine's address must be on loopback\n",
                "temper-box: /s/temper/tui: cannot open its files\n",
                "temper-box: no command; temper-box --help lists them\n",
            ],
        ),
    ] {
        let [terminal, address, link, loopback, directory, bare] = lines;
        for (why, expected) in [
            (Refused::NotATerminal, terminal),
            (Refused::NotAnAddress(Box::from(b"ARG".as_slice())), address),
            (Refused::Link, link),
            (Refused::NotLoopback, loopback),
            (Refused::Directory("/s/temper/tui: cannot open its files".into()), directory),
        ] {
            assert_eq!(why.line(name), expected, "{name}: {why:?}");
        }
        for (given, expected) in [(true, terminal), (false, bare)] {
            assert_eq!(super::terminal_refusal(name, given), expected, "{name}: given {given}");
        }
    }
}

fn origin() -> temper_client_tui::Origin {
    temper_client_tui::Origin::Http("127.0.0.1:8367".parse().expect("origin"))
}

fn private_file(path: &std::path::Path, bytes: &[u8]) {
    fs::write(path, bytes).expect("fixture bytes");
    fs::set_permissions(path, fs::Permissions::from_mode(0o600)).expect("private fixture");
}

fn flat(scratch: &skein_scratch::Scratch) {
    for (file, bytes) in
        [("token", b"fake-token".as_slice()), ("store", b"fake-store"), ("sign-in", b"127.0.0.1:8367 until 123")]
    {
        private_file(&scratch.path().join(file), bytes);
    }
    private_file(&scratch.path().join("address"), b"http://127.0.0.1:8367");
}

fn prepare(scratch: &skein_scratch::Scratch) -> (std::path::PathBuf, bool) {
    super::deployment::prepare(scratch.path(), &origin(), skein_shell::effective_user()).expect("origin directory")
}

fn no_flat(scratch: &skein_scratch::Scratch) {
    for file in ["token", "store", "sign-in"] {
        assert!(!scratch.path().join(file).exists(), "{file} removed from the flat directory");
    }
}

#[test]
fn origin_directories_keep_components_private_and_unshortened() {
    use temper_client_tui::Origin;
    let longest = ["a".repeat(63), "b".repeat(63), "c".repeat(63), "d".repeat(61)].join(".");
    for (origin, relative) in [
        (origin(), "at/127.0.0.1/8367".to_owned()),
        (Origin::Http("[::1]:8367".parse().expect("IPv6")), "at/::1/8367".to_owned()),
        (Origin::Https { name: b"dev.fabrig.ai".as_slice().into(), port: 443 }, "at/dev.fabrig.ai/443".to_owned()),
        (Origin::Https { name: longest.as_bytes().into(), port: 443 }, format!("at/{longest}/443")),
    ] {
        let scratch = skein_scratch::Scratch::new("tui-origin");
        let user = skein_shell::effective_user();
        let (directory, made) = super::deployment::prepare(scratch.path(), &origin, user).expect("prepare");
        assert!(made);
        assert_eq!(directory, scratch.path().join(relative));
        for path in [directory.clone(), directory.parent().expect("host").into(), scratch.path().join("at")] {
            assert_eq!(fs::metadata(&path).expect("metadata").permissions().mode() & 0o777, 0o700);
        }
        assert!(!super::deployment::prepare(scratch.path(), &origin, user).expect("again").1);
    }
}

#[test]
fn origin_directories_refuse_a_public_or_linked_parent() {
    for linked in [false, true] {
        let scratch = skein_scratch::Scratch::new("tui-parent");
        let at = scratch.path().join("at");
        if linked {
            std::os::unix::fs::symlink(scratch.path(), &at).expect("link");
        } else {
            fs::create_dir(&at).expect("public parent");
            fs::set_permissions(&at, fs::Permissions::from_mode(0o755)).expect("mode");
        }
        assert!(super::deployment::prepare(scratch.path(), &origin(), skein_shell::effective_user()).is_err());
    }
}

#[test]
fn flat_files_move_only_with_both_matching_witnesses() {
    let scratch = skein_scratch::Scratch::new("tui-move");
    flat(&scratch);
    let (destination, made) = prepare(&scratch);
    assert!(!made, "migration made the directory, not this origin's first start");
    for (file, bytes) in
        [("token", b"fake-token".as_slice()), ("store", b"fake-store"), ("sign-in", b"127.0.0.1:8367 until 123")]
    {
        assert_eq!(fs::read(destination.join(file)).expect("moved"), bytes);
    }
    no_flat(&scratch);
    assert_eq!(fs::read(scratch.path().join("address")).expect("address kept"), b"http://127.0.0.1:8367");
}

#[test]
fn flat_files_with_a_different_address_are_removed() {
    let scratch = skein_scratch::Scratch::new("tui-mismatch");
    flat(&scratch);
    private_file(&scratch.path().join("address"), b"http://127.0.0.1:9000");
    let (destination, made) = prepare(&scratch);
    assert!(made);
    assert!(!destination.join("token").exists());
    no_flat(&scratch);
}

#[test]
fn flat_files_without_an_address_are_removed() {
    let scratch = skein_scratch::Scratch::new("tui-no-address");
    flat(&scratch);
    fs::remove_file(scratch.path().join("address")).expect("no address");
    prepare(&scratch);
    no_flat(&scratch);
}

#[test]
fn flat_files_without_a_note_are_removed() {
    let scratch = skein_scratch::Scratch::new("tui-no-note");
    flat(&scratch);
    fs::remove_file(scratch.path().join("sign-in")).expect("no note");
    prepare(&scratch);
    no_flat(&scratch);
}

#[test]
fn an_interrupted_move_finishes_before_opening_the_origin() {
    let scratch = skein_scratch::Scratch::new("tui-resume");
    flat(&scratch);
    let staging = scratch.path().join("at/127.0.0.1/8367.new");
    temper_shell::directories::make_private(&staging, skein_shell::effective_user()).expect("staging");
    for file in ["token", "store", "sign-in"] {
        fs::rename(scratch.path().join(file), staging.join(file)).expect("staged");
    }
    let (destination, made) = prepare(&scratch);
    assert!(!made);
    assert!(!staging.exists());
    assert_eq!(fs::read(destination.join("token")).expect("finished token"), b"fake-token");
    assert_eq!(fs::read(destination.join("store")).expect("finished store"), b"fake-store");
    assert_eq!(fs::read(destination.join("sign-in")).expect("finished note"), b"127.0.0.1:8367 until 123");
    no_flat(&scratch);
}

#[test]
fn a_partial_move_finishes_with_the_remaining_flat_files() {
    let scratch = skein_scratch::Scratch::new("tui-partial");
    flat(&scratch);
    let staging = scratch.path().join("at/127.0.0.1/8367.new");
    temper_shell::directories::make_private(&staging, skein_shell::effective_user()).expect("staging");
    fs::rename(scratch.path().join("token"), staging.join("token")).expect("first rename");
    let (destination, _) = prepare(&scratch);
    for file in ["token", "store", "sign-in"] {
        assert!(destination.join(file).is_file());
    }
    assert!(!staging.exists());
    no_flat(&scratch);
}

#[test]
fn an_existing_destination_keeps_its_files_and_discards_the_flat_files() {
    let scratch = skein_scratch::Scratch::new("tui-existing");
    let (destination, _) = prepare(&scratch);
    private_file(&destination.join("token"), b"destination-token");
    flat(&scratch);
    prepare(&scratch);
    assert_eq!(fs::read(destination.join("token")).expect("kept"), b"destination-token");
    assert!(!destination.join("store").exists());
    no_flat(&scratch);
}

#[test]
fn a_new_origin_takes_only_the_last_origins_theme() {
    let scratch = skein_scratch::Scratch::new("tui-theme");
    let user = skein_shell::effective_user();
    let (previous, _) = prepare(&scratch);
    let store = b"\0\0\0\x02\0\0\0\x05theme\0\0\0\x04dark\0\0\0\x05other\0\0\0\x03bad";
    private_file(&previous.join("store"), store);
    private_file(&scratch.path().join("address"), b"http://127.0.0.1:8367");
    let other = temper_client_tui::Origin::Http("[::1]:9000".parse().expect("other"));
    let (_, made) = super::deployment::prepare(scratch.path(), &other, user).expect("new origin");
    let record = super::deployment::theme(scratch.path(), &other, user, made).expect("theme carried");
    assert_eq!(record.key.as_ref(), temper_client_tui::theme::THEME_KEY);
    assert_eq!(record.bytes.as_ref(), b"dark");
    assert!(super::deployment::theme(scratch.path(), &other, user, false).is_none(), "only on first start");
    fs::remove_file(previous.join("store")).expect("no store");
    assert!(super::deployment::theme(scratch.path(), &other, user, true).is_none(), "Auto without a store");
}

#[test]
fn only_apple_terminal_leaves_the_mark_out() {
    for (value, plain) in [
        (Some("Apple_Terminal"), true),
        (Some("iTerm.app"), false),
        (Some("apple_terminal"), false),
        (Some(""), false),
        (None, false),
    ] {
        let entries = value.map(|value| ("TERM_PROGRAM", value)).into_iter().collect::<Vec<_>>();
        assert_eq!(tui::plain(&environment(&entries)), plain, "{value:?}");
    }
}
