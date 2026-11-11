use super::*;
use skein_lib::Queue;
use skein_scratch::Scratch;
use skein_ui_screen_terminal::Decoder;

fn setup_journeys(root: &Path, walk: &[u8]) -> (std::path::PathBuf, std::path::PathBuf) {
    let tui = root.join("tui");
    let browser = root.join("browser");
    fs::create_dir_all(tui.join("first")).expect("TUI journey");
    fs::create_dir(&browser).expect("browser journeys");
    fs::write(tui.join("first/walk"), walk).expect("walk");
    (tui, browser)
}

fn walker_paths(scratch: &Path) -> Paths {
    Paths { temper: "temper".into(), bundle: "bundle".into(), chromium: "chromium".into(), scratch: scratch.into() }
}

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
        let refusal = starting(root, root, "first").expect_err("invalid walk refused without building");
        let Refusal::TuiWalk { name, unreadable } = refusal else { panic!("walk refusal: {refusal:?}") };
        assert_eq!(name, "first");
        assert_eq!(unreadable.line, line);
        assert!(unreadable.why.contains(why), "{}: {why}", unreadable.why);
    }
    fs::write(path, b"walk 1\nsee \"Sign in\"\nend\n").expect("write first walk");
    let read = starting(root, root, "first").expect("no earlier recording needed");
    assert_eq!(read.steps, vec![WalkStep::See(b"Sign in".as_slice().into()), WalkStep::End]);
    assert!(read.setup.is_none());
    let paths = walker_paths(root);
    let walker = Walker::new("first", &paths, vec![], &read);
    assert!(matches!(walker.phase, Phase::Opening));
    assert!(!root.join("first/recording").exists());
}

#[test]
fn a_tui_setup_is_read_whole_before_any_build_or_child() {
    use temper_client_shell_web::journey::SetupStep;
    let scratch = Scratch::new("tui-again-setup");
    let (tui, browser) = setup_journeys(scratch.path(), b"walk 1\nsee \"Sign in\"\nend\n");
    for directory in [tui.join("base"), browser.join("base")] {
        fs::create_dir(directory).expect("parent journey directory");
    }
    // A same-named TUI setup must not supply the parent's steps.
    fs::write(tui.join("base/setup"), b"setup 1\nsee \"Wrong parent\"\nreload \"/wrong\"\n").expect("TUI parent");
    fs::write(
        browser.join("base/setup"),
        b"setup 1\npress button \"Create\" 2\nreload \"/base\"\nmark 1 \"Browser mark\"\n",
    )
    .expect("browser parent");
    let path = tui.join("first/setup");
    fs::write(&path, b"setup 1\nafter \"base\"\nsee \"Ready\"\nreload \"/child\"\n").expect("TUI setup");
    let read = starting(&tui, &browser, "first").expect("complete setup read before building");
    assert_eq!(read.steps, vec![WalkStep::See(b"Sign in".as_slice().into()), WalkStep::End]);
    let prepared = read.setup.as_ref().expect("setup retained before building");
    assert!(matches!(&prepared.setup.steps[..],
        [SetupStep::Act { role: Role::Button, name, order: 2, act: Act::Press }, SetupStep::See { text }]
        if name.as_ref() == b"Create" && text.as_ref() == b"Ready"));
    assert_eq!(prepared.setup.reload.as_ref(), b"/child");
    assert!(prepared.setup.marks.is_empty(), "parent's marks are not inherited");
    assert_eq!(prepared.sources, [("base".into(), 2), ("first".into(), 3)]);
    assert!(!tui.join("first/recording").exists());
    let paths = walker_paths(scratch.path());
    let mut walker = Walker::new("first", &paths, vec![], &read);
    assert!(matches!(walker.phase, Phase::SetupStep { at: 0 }));
    let mut face = BrowserFace::new(skein_lib::Token::new(1), "127.0.0.1:1");
    for (phase, source, line, sought) in
        [(Phase::SetupStep { at: 0 }, "base", 2, "Create"), (Phase::SetupSee { at: 1 }, "first", 3, "Ready")]
    {
        walker.phase = phase;
        walker.since = None;
        assert_eq!(
            walker.progress(&mut face, Time::ZERO).expect("pending setup query"),
            Next::Wait { until: Some(Time::from_nanos(50_000_000)) }
        );
        assert!(walker.terminal.is_none(), "setup runs before the TUI");
        assert!(walker.deadline(Time::from_nanos(59_999_999_999)).is_ok());
        walker.check_deadline(&mut face, Time::from_nanos(60_000_000_000));
        let refusal = walker.refusal.take().expect("setup deadline while a query is pending");
        assert!(matches!(refusal,
            Refusal::SetupTimeout { name, setup, line: found, sought: query }
            if name == "first" && setup == source && found == line && query.contains(sought)));
    }
    walker.phase = Phase::SetupStep { at: 2 };
    walker.since = None;
    walker.progress(&mut face, Time::ZERO).expect("setup complete");
    assert!(matches!(walker.phase, Phase::Load));
    assert_eq!(
        walker.progress(&mut face, Time::ZERO).expect("reload after setup"),
        Next::Do(Doing::Go { address: b"/child".as_slice().into() })
    );
    assert!(matches!(walker.phase, Phase::Opening));
    assert!(walker.terminal.is_none(), "reload precedes the TUI");
}

#[test]
fn a_tui_setup_refuses_marks_before_building() {
    let scratch = Scratch::new("tui-again-setup-refusal");
    let (tui, browser) = setup_journeys(scratch.path(), b"walk 1\nend\n");
    for (bytes, line) in [
        (b"setup 1\nreload \"/\"\nmark 1 \"Ready\"\n".as_slice(), 3),
        (b"setup 1\nsee \"Home\"\nreload \"/\"\nmark 2 \"Ready\"\n".as_slice(), 4),
        (b"setup 1\nafter \"base\"\nsee \"Home\"\nreload \"/\"\nmark 3 \"Ready\"\n".as_slice(), 5),
    ] {
        fs::write(tui.join("first/setup"), bytes).expect("setup with mark");
        let refusal = starting(&tui, &browser, "first").expect_err("TUI setup mark refused");
        let Refusal::SetupUnreadable { name, unreadable } = refusal else { panic!("setup refusal: {refusal:?}") };
        assert_eq!(name, "first");
        assert_eq!(unreadable.line, line);
        assert!(unreadable.why.contains("mark"), "{}", unreadable.why);
    }
}

#[test]
fn a_tui_setup_refuses_a_missing_browser_setup_before_building() {
    let scratch = Scratch::new("tui-again-missing-setup");
    let (tui, browser) = setup_journeys(scratch.path(), b"walk 1\nend\n");
    fs::create_dir(browser.join("missing")).expect("browser journey without setup");
    fs::create_dir(tui.join("missing")).expect("same-named TUI journey");
    fs::write(tui.join("missing/setup"), b"setup 1\nreload \"/\"\n").expect("TUI setup cannot be the parent");
    fs::write(tui.join("first/setup"), b"setup 1\nafter \"missing\"\nreload \"/\"\n").expect("after setup");
    assert!(matches!(starting(&tui, &browser, "first"),
        Err(Refusal::SetupAfter { name, after, why: "has no setup" }) if name == "first" && after == "missing"));
    fs::remove_file(tui.join("first/setup")).expect("remove optional setup");
    let read = starting(&tui, &browser, "first").expect("no setup reads as before");
    assert_eq!(read.steps, vec![WalkStep::End]);
    assert!(read.setup.is_none());
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
        let mut given: Vec<crate::EnvironmentEntry> = [
            ("PATH", "/usr/bin:/bin"),
            ("HOME", "/old-home"),
            ("XDG_STATE_HOME", "/old-state"),
            ("XDG_RUNTIME_DIR", "/tmp/journey-environment/runtime"),
            ("COLORFGBG", "0;15"),
            ("TERM", "dumb"),
            ("SSH_CONNECTION", "remote"),
            ("SSH_TTY", "tty"),
            ("SSH_AUTH_SOCK", "agent"),
            ("LANG", "C.UTF-8"),
        ]
        .into_iter()
        .map(|(name, value)| (Box::from(name.as_bytes()), Box::from(value.as_bytes())))
        .collect();
        if let Some(display) = display {
            given.push((Box::from(b"DISPLAY".as_slice()), Box::from(display.as_bytes())));
        }
        let environment = environment(&given, scratch).expect("scratch PATH");
        let lookup = |name: &str| environment.iter().find(|(key, _)| key == name).map(|(_, value)| value.as_os_str());
        for (name, value) in [
            ("PATH", "/tmp/journey-environment/bin:/usr/bin:/bin"),
            ("HOME", "/tmp/journey-environment/home"),
            ("XDG_STATE_HOME", "/tmp/journey-environment/state"),
            ("XDG_RUNTIME_DIR", "/tmp/journey-environment/tui-runtime"),
            ("TERM", "xterm-256color"),
            ("DISPLAY", display.unwrap_or(":0")),
            ("LANG", "C.UTF-8"),
        ] {
            assert_eq!(lookup(name), Some(std::ffi::OsStr::new(value)), "{name}");
        }
        assert!(environment.iter().all(|(name, _)| !name.as_bytes().starts_with(b"SSH_")));
        assert_eq!(lookup("COLORFGBG"), None);
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
    let paths = walker_paths(Path::new("/tmp/journey"));
    let mut walker = Walker {
        name: "first".into(),
        paths: &paths,
        environment: vec![],
        steps: vec![WalkStep::End],
        setup: None,
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
