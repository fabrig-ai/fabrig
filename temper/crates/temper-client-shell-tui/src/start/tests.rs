use super::{self as tui, Refused};
use skein_ui_screen_terminal::Depth;
use std::{fs, os::unix::fs::PermissionsExt};
use temper_client_tui::{Note, Scheme};

const LINK: &str = "http://127.0.0.1:8367/jig/v1/sign-in/link?secret=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";

fn environment(entries: &[(&str, &str)]) -> Vec<super::EnvironmentEntry> {
    entries.iter().map(|(name, value)| (Box::from(name.as_bytes()), Box::from(value.as_bytes()))).collect()
}

#[test]
fn an_address_is_an_engine_on_loopback_and_never_a_link() {
    for (argument, expected) in [
        ("http://127.0.0.1:8367", Ok("127.0.0.1:8367".parse().expect("IPv4"))),
        ("http://127.0.0.1:8367/", Ok("127.0.0.1:8367".parse().expect("IPv4"))),
        ("http://[::1]:9000", Ok("[::1]:9000".parse().expect("IPv6"))),
        ("http://10.0.0.1:8367", Err(Refused::NotLoopback)),
        (LINK, Err(Refused::Link)),
        ("127.0.0.1:8367", Err(Refused::NotAnAddress(Box::from(b"127.0.0.1:8367".as_slice())))),
        ("http://localhost:8367", Err(Refused::NotAnAddress(Box::from(b"http://localhost:8367".as_slice())))),
    ] {
        assert_eq!(tui::address(argument.as_bytes()), expected, "{argument}");
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
        ("http://127.0.0.1:9000", Some("127.0.0.1:9000".parse().expect("address"))),
        ("", None),
        ("garbage", None),
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
    let given = "[::1]:9000".parse().expect("given address");
    let saved = "127.0.0.1:9001".parse().expect("kept address");
    let default = "127.0.0.1:8367".parse().expect("default address");
    let file_engine = "127.0.0.1:9002".parse().expect("file address");
    let link = LINK.replacen(":8367", ":9002", 1);
    let off_loopback = link.replacen("127.0.0.1", "10.0.0.1", 1);
    let mut failures = Vec::new();
    for (row, argument, file, kept, expected) in [
        ("given", Some(given), Some(link.as_str()), Some(saved), given),
        ("file", None, Some(link.as_str()), Some(saved), file_engine),
        ("kept", None, None, Some(saved), saved),
        ("default", None, None, None, default),
        ("file address", None, Some("http://127.0.0.1:9002"), Some(saved), saved),
        ("off loopback", None, Some(off_loopback.as_str()), Some(saved), saved),
        ("garbage", None, Some("garbage"), Some(saved), saved),
    ] {
        let engine = tui::selected_address(argument, file.map(str::as_bytes), kept);
        if engine != expected {
            failures.push(row);
        }
        let (_, secret) = LINK.split_once("secret=").expect("fixture secret");
        assert!(!engine.to_string().contains(secret), "only the engine address leaves selection");
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
        assert_eq!(tui::on_box(link, engine), expected);
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
    let path = scratch.path().join("sign-in");
    let engine = "127.0.0.1:8367".parse().expect("note address");
    let other = "[::1]:9000".parse().expect("other address");
    let user = skein_shell::effective_user();
    let until = Note::Until(skein_lib::Wall::from_nanos(123));
    let mut failures = Vec::new();
    for (row, bytes, mode, owner, address, expected) in [
        ("own address", Some(until.print(b"127.0.0.1:8367")), 0o600, user, engine, Some(until)),
        ("other address", Some(until.print(b"127.0.0.1:8367")), 0o600, user, other, None),
        ("signed out", Some(Note::SignedOut.print(b"127.0.0.1:8367")), 0o600, user, engine, Some(Note::SignedOut)),
        ("nonprivate", Some(until.print(b"127.0.0.1:8367")), 0o640, user, engine, None),
        ("foreign", Some(until.print(b"127.0.0.1:8367")), 0o600, user.wrapping_add(1), engine, None),
        ("garbage", Some(Box::from(b"garbage".as_slice())), 0o600, user, engine, None),
        ("absent", None, 0o600, user, engine, None),
    ] {
        if let Some(bytes) = bytes {
            fs::write(&path, bytes).expect("fixture note");
            fs::set_permissions(&path, fs::Permissions::from_mode(mode)).expect("note permissions");
        } else {
            fs::remove_file(&path).expect("remove fixture note");
        }
        if tui::read_note(scratch.path(), address, owner) != expected {
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
            "temper-client",
            [
                "temper-client: the TUI needs a terminal on stdin and stdout\n",
                "temper-client: not an address: ARG\n",
                "temper-client: a sign-in link is never an argument: run temper-client http://HOST:PORT and paste it\n",
                "temper-client: the engine's address must be on loopback\n",
                "temper-client: /s/temper/tui: cannot open its files\n",
                "temper-client: no command; temper-client --help lists them\n",
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
