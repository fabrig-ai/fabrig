use crate::{
    Command, commands, configure_with,
    tui::{self, Refused},
};
use skein_ui_screen_terminal::Depth;
use temper_client_tui::Scheme;

const LINK: &str = "http://127.0.0.1:8367/jig/v1/sign-in/link?secret=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";

fn environment(entries: &[(&str, &str)]) -> Vec<crate::EnvironmentEntry> {
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
    for (value, expected) in [
        (Some("15;0"), Scheme::Dark),
        (Some("0;15"), Scheme::Light),
        (Some("0;default;7"), Scheme::Light),
        (Some("7"), Scheme::Light),
        (None, Scheme::Dark),
    ] {
        let entries = value.map(|value| ("COLORFGBG", value)).into_iter().collect::<Vec<_>>();
        assert_eq!(tui::scheme(&environment(&entries)), expected, "{value:?}");
    }
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
fn bare_temper_and_temper_address_parse_and_refuse() {
    for (arguments, expected) in [
        (vec![], Ok(Command::Tui { address: None })),
        (
            vec!["http://127.0.0.1:8367"],
            Ok(Command::Tui { address: Some(Box::from(b"http://127.0.0.1:8367".as_slice())) }),
        ),
        (
            vec!["http://127.0.0.1:8367", "http://127.0.0.1:9000"],
            Err("temper: invalid arguments\nusage: temper [ADDRESS]\n".into()),
        ),
        (vec!["bogus"], Err("temper: no command bogus; temper --help lists them\n".into())),
    ] {
        let bytes = arguments.iter().map(|word| Box::from(word.as_bytes())).collect::<Vec<_>>();
        assert_eq!(configure_with(&bytes, &[], b"/"), expected, "{arguments:?}");
    }
    for (why, line) in [
        (Refused::NotATerminal, "temper: the TUI needs a terminal on stdin and stdout\n"),
        (Refused::NotAnAddress(Box::from(b"ARG".as_slice())), "temper: not an address: ARG\n"),
        (Refused::Link, "temper: a sign-in link is never an argument: run temper http://HOST:PORT and paste it\n"),
        (Refused::NotLoopback, "temper: the engine's address must be on loopback\n"),
        (
            Refused::Directory("/s/temper/tui: cannot open its files".into()),
            "temper: /s/temper/tui: cannot open its files\n",
        ),
    ] {
        assert_eq!(why.line(), line, "{why:?}");
    }
    assert_eq!(tui::terminal_refusal(false), commands::unknown(&[]));
    assert_eq!(tui::terminal_refusal(false), "temper: no command; temper --help lists them\n");
    assert_eq!(tui::terminal_refusal(true), Refused::NotATerminal.line());
    let row = commands::find(&[]).expect("TUI row");
    assert!(commands::row(row).starts_with("usage: temper [ADDRESS]\n"));
    assert!(commands::reference().contains("## temper [ADDRESS]\n\n`temper [ADDRESS]`\n"));
}
