use super::*;
use skein_lib::Queue;
use skein_scratch::Scratch;
use skein_ui_screen_terminal::Decoder;

#[test]
fn a_tui_walk_is_read_before_any_build_or_child() {
    let scratch = Scratch::new("tui-again-starting");
    let root = scratch.path();
    fs::create_dir(root.join("first")).expect("journey directory");
    let path = root.join("first/walk");
    for (bytes, line, why) in [
        (b"walk 1\nsee \"Sign in\"\n".as_slice(), 3, "missing end"),
        (b"walk 1\nkey unknown\nend\n".as_slice(), 2, "unknown key"),
        (b"walk 2\nend\n".as_slice(), 1, "expected walk 1"),
    ] {
        fs::write(&path, bytes).expect("write invalid walk");
        let refusal = starting(root, "first").expect_err("invalid walk refused without building");
        let Refusal::TuiWalk { name, unreadable } = refusal else { panic!("walk refusal: {refusal:?}") };
        assert_eq!(name, "first");
        assert_eq!(unreadable.line, line);
        assert!(unreadable.why.contains(why), "{}: {why}", unreadable.why);
    }
    fs::write(path, b"walk 1\nsee \"Sign in\"\nend\n").expect("write first walk");
    assert_eq!(
        starting(root, "first").expect("no earlier recording needed"),
        vec![WalkStep::See(b"Sign in".as_slice().into()), WalkStep::End]
    );
    assert!(!root.join("first/recording").exists());
}

#[test]
fn walk_keys_are_terminal_bytes() {
    let cases: &[(Key, &[u8])] = &[
        (Key::Enter, b"\r"),
        (Key::Tab, b"\t"),
        (Key::BackTab, b"\x1b[Z"),
        (Key::Backspace, b"\x7f"),
        (Key::Delete, b"\x1b[3~"),
        (Key::Up, b"\x1b[A"),
        (Key::Down, b"\x1b[B"),
        (Key::Right, b"\x1b[C"),
        (Key::Left, b"\x1b[D"),
        (Key::Home, b"\x1b[H"),
        (Key::End, b"\x1b[F"),
        (Key::PageUp, b"\x1b[5~"),
        (Key::PageDown, b"\x1b[6~"),
        (Key::Escape, b"\x1b"),
        (Key::Char('a'), b"a"),
        (Key::Char('λ'), "λ".as_bytes()),
        (Key::Control(b'c'), b"\x03"),
        (Key::Alt('M'), b"\x1bM"),
        (Key::Alt('S'), b"\x1bS"),
        (Key::Paste(b"one\ntwo".as_slice().into()), b"\x1b[200~one\ntwo\x1b[201~"),
    ];
    for (key, expected) in cases {
        let bytes = key_bytes(key);
        assert_eq!(bytes, *expected, "{key:?}");
        let mut decoder = Decoder::new(100);
        let mut keys = Queue::with_capacity(2);
        decoder.read(&bytes, &mut keys);
        assert_eq!(keys.pop().as_ref(), Some(key), "decoder sees {key:?}");
        assert!(keys.is_empty());
    }
}

#[test]
fn tui_environment_keeps_the_opener_inside_the_scratch_directory() {
    let scratch = Path::new("/tmp/journey-environment");
    for display in [None, Some(":7")] {
        let mut given: Vec<(OsString, OsString)> = [
            ("PATH", "/usr/bin:/bin"),
            ("HOME", "/old-home"),
            ("XDG_STATE_HOME", "/old-state"),
            ("TERM", "dumb"),
            ("SSH_CONNECTION", "remote"),
            ("SSH_TTY", "tty"),
            ("SSH_AUTH_SOCK", "agent"),
            ("LANG", "C.UTF-8"),
        ]
        .into_iter()
        .map(|(name, value)| (name.into(), value.into()))
        .collect();
        if let Some(display) = display {
            given.push(("DISPLAY".into(), display.into()));
        }
        let environment = environment(&given, scratch).expect("scratch PATH");
        let lookup = |name: &str| environment.iter().find(|(key, _)| key == name).map(|(_, value)| value.as_os_str());
        for (name, value) in [
            ("PATH", "/tmp/journey-environment/bin:/usr/bin:/bin"),
            ("HOME", "/tmp/journey-environment/home"),
            ("XDG_STATE_HOME", "/tmp/journey-environment/state"),
            ("TERM", "xterm-256color"),
            ("DISPLAY", display.unwrap_or(":0")),
            ("LANG", "C.UTF-8"),
        ] {
            assert_eq!(lookup(name), Some(std::ffi::OsStr::new(value)), "{name}");
        }
        assert!(environment.iter().all(|(name, _)| !name.as_bytes().starts_with(b"SSH_")));
    }
}

#[test]
fn a_save_is_a_replacement_even_when_its_bytes_are_unchanged() {
    let scratch = Scratch::new("tui-again-save");
    let path = scratch.path().join("recording");
    assert!(replaced(&path, None).expect("no first save").is_none());
    fs::write(&path, b"same save").expect("first save");
    let earlier = File::open(&path).expect("hold earlier inode");
    assert!(replaced(&path, Some(&earlier)).expect("unchanged file").is_none());
    let next = scratch.path().join("next");
    fs::write(&next, b"same save").expect("next save");
    fs::rename(next, &path).expect("replace save");
    assert_eq!(replaced(&path, Some(&earlier)).expect("replacement"), Some(b"same save".to_vec()));
}

#[test]
fn a_step_has_thirty_seconds_and_names_its_position() {
    let paths = Paths {
        temper: "temper".into(),
        bundle: "bundle".into(),
        chromium: "chromium".into(),
        scratch: "/tmp/journey".into(),
    };
    let mut walker = Walker {
        name: "first".into(),
        paths: &paths,
        environment: vec![],
        steps: vec![WalkStep::End],
        at: 0,
        phase: Phase::Step,
        since: Some(Time::ZERO),
        terminal: None,
        refusal: None,
        recording: None,
    };
    for at in [0, 4] {
        walker.at = at;
        assert!(walker.deadline(Time::from_nanos(29_999_999_999)).is_ok());
        let refusal = walker.deadline(Time::from_nanos(30_000_000_000)).expect_err("step deadline");
        let Refusal::TuiAgain { name, step, report } = refusal else { panic!("deadline refusal: {refusal:?}") };
        assert_eq!(name, "first");
        assert_eq!(step, u32::try_from(at + 1).expect("step"));
        assert!(report.contains("30 s"));
    }
}
