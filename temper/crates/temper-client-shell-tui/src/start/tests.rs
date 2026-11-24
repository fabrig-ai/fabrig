mod disk;

use super::{self as tui, Refused, deployment::Files};
use crate::testing::origin;
use disk::{Disk, USER};
use skein_fake_machine::Item;
use skein_ui_screen_terminal::Depth;
use std::{fs, os::unix::fs::PermissionsExt, path::Path};
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

fn prepare(scratch: &skein_scratch::Scratch) -> (std::path::PathBuf, bool) {
    super::deployment::prepare(scratch.path(), &origin(), skein_shell::effective_user()).expect("origin directory")
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

#[test]
fn a_bare_start_without_a_terminal_fails() {
    const HTTPS: &[u8] = b"https://dev.fabrig.ai";
    for (argument, words) in [
        (None, "temper: no command; temper --help lists them\n"),
        (Some(HTTPS), "temper: the TUI needs a terminal on stdin and stdout\n"),
    ] {
        assert_eq!(super::require_terminal("temper", argument, false), Err((1, words.into())), "{argument:?}");
    }
}

const FLAT: [(&str, &[u8]); 3] =
    [("token", b"fake-token"), ("store", b"fake-store"), ("sign-in", b"127.0.0.1:8367 until 123")];
const STAGING: &str = "tui/at/127.0.0.1/8367.new";
const SAVED: &[u8] = b"http://127.0.0.1:8367";

fn lay_flat(disk: &mut Disk, address: Option<&[u8]>, note: bool) {
    for (file, bytes) in FLAT {
        if file != "sign-in" || note {
            disk.write(&Path::new("tui").join(file), bytes);
        }
    }
    if let Some(address) = address {
        disk.write(Path::new("tui/address"), address);
    }
}

fn flat(address: Option<&[u8]>, note: bool) -> Disk {
    let mut disk = Disk::new(&[]);
    lay_flat(&mut disk, address, note);
    disk
}

fn no_flat(disk: &mut Disk) {
    for (file, _) in FLAT {
        assert!(!disk.exists(&Path::new("tui").join(file)), "{file} removed from the flat directory");
    }
}

fn staged(moved: &[&str]) -> Disk {
    let mut disk = Disk::new(&[
        Item::directory(b"tui/at").mode(0o700),
        Item::directory(b"tui/at/127.0.0.1").mode(0o700),
        Item::directory(STAGING.as_bytes()).mode(0o700),
    ]);
    lay_flat(&mut disk, Some(SAVED), true);
    for file in moved {
        disk.rename(&Path::new("tui").join(file), &Path::new(STAGING).join(file)).expect("staged");
    }
    disk
}

#[test]
fn origin_directories_keep_components_private_and_unshortened() {
    // The longest DNS name, including the dots between its labels.
    let longest = ["a".repeat(63), "b".repeat(63), "c".repeat(63), "d".repeat(61)].join(".");
    for (origin, relative) in [
        (origin(), "at/127.0.0.1/8367".to_owned()),
        (Origin::Http("[::1]:8367".parse().expect("IPv6")), "at/::1/8367".to_owned()),
        (Origin::Https { name: b"dev.fabrig.ai".as_slice().into(), port: 443 }, "at/dev.fabrig.ai/443".to_owned()),
        (Origin::Https { name: longest.as_bytes().into(), port: 443 }, format!("at/{longest}/443")),
    ] {
        let mut disk = Disk::new(&[]);
        let root = Path::new("tui");
        let expected = root.join(relative);
        assert_eq!(disk.prepare(&origin), (expected.clone(), true));
        for path in [expected.as_path(), expected.parent().expect("host"), &root.join("at")] {
            assert_eq!(disk.mode(path) & 0o777, 0o700, "{}", path.display());
        }
        assert!(!disk.prepare(&origin).1);
    }
}

#[test]
fn origin_directories_refuse_a_public_or_linked_parent() {
    for parent in [Item::directory(b"tui/at").mode(0o755), Item::link(b"tui/at", b".")] {
        let mut disk = Disk::new(&[parent]);
        let result = super::deployment::prepare_with(Path::new("tui"), &origin(), USER, &mut disk);
        assert!(result.is_err());
    }
}

#[test]
fn flat_files_move_only_with_both_matching_witnesses() {
    let mut disk = flat(Some(SAVED), true);
    let (destination, made) = disk.prepare(&origin());
    assert!(!made, "migration made the directory, not this origin's first start");
    for (file, bytes) in FLAT {
        assert_eq!(disk.read(&destination.join(file)), bytes);
    }
    no_flat(&mut disk);
    assert_eq!(disk.read(Path::new("tui/address")), SAVED);
}

#[test]
fn flat_files_with_a_different_address_are_removed() {
    let mut disk = flat(Some(b"http://127.0.0.1:9000"), true);
    let (destination, made) = disk.prepare(&origin());
    assert!(made);
    assert!(!disk.exists(&destination.join("token")));
    no_flat(&mut disk);
}

#[test]
fn flat_files_without_an_address_are_removed() {
    let mut disk = flat(None, true);
    disk.prepare(&origin());
    no_flat(&mut disk);
}

#[test]
fn flat_files_without_a_note_are_removed() {
    let mut disk = flat(Some(SAVED), false);
    disk.prepare(&origin());
    no_flat(&mut disk);
}

#[test]
fn an_interrupted_move_finishes_before_opening_the_origin() {
    let mut disk = staged(&["token", "store", "sign-in"]);
    let (destination, made) = disk.prepare(&origin());
    assert!(!made);
    assert!(!disk.exists(Path::new(STAGING)));
    for (file, bytes) in FLAT {
        assert_eq!(disk.read(&destination.join(file)), bytes);
    }
    no_flat(&mut disk);
}

#[test]
fn a_partial_move_finishes_with_the_remaining_flat_files() {
    let mut disk = staged(&["token"]);
    let (destination, _) = disk.prepare(&origin());
    for (file, _) in FLAT {
        assert!(disk.is_file(&destination.join(file)));
    }
    assert!(!disk.exists(Path::new(STAGING)));
    no_flat(&mut disk);
}

#[test]
fn an_existing_destination_keeps_its_files_and_discards_the_flat_files() {
    let mut disk = Disk::new(&[]);
    let (destination, _) = disk.prepare(&origin());
    disk.write(&destination.join("token"), b"destination-token");
    lay_flat(&mut disk, Some(SAVED), true);
    disk.prepare(&origin());
    assert_eq!(disk.read(&destination.join("token")), b"destination-token");
    assert!(!disk.exists(&destination.join("store")));
    no_flat(&mut disk);
}

#[test]
fn a_new_origin_takes_only_the_last_origins_theme() {
    let mut disk = Disk::new(&[]);
    let (previous, _) = disk.prepare(&origin());
    let store = b"\0\0\0\x02\0\0\0\x05theme\0\0\0\x04dark\0\0\0\x05other\0\0\0\x03bad";
    disk.write(&previous.join("store"), store);
    disk.write(Path::new("tui/address"), SAVED);
    let other = Origin::Http("[::1]:9000".parse().expect("other"));
    let (_, made) = disk.prepare(&other);
    let theme = |disk: &mut Disk, made| super::deployment::theme_with(Path::new("tui"), &other, USER, made, disk);
    let record = theme(&mut disk, made).expect("theme carried");
    assert_eq!(record.key.as_ref(), temper_client_tui::theme::THEME_KEY);
    assert_eq!(record.bytes.as_ref(), b"dark");
    assert!(theme(&mut disk, false).is_none(), "only on first start");
    disk.remove_file(&previous.join("store")).expect("no store");
    assert!(theme(&mut disk, true).is_none(), "Auto without a store");
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
