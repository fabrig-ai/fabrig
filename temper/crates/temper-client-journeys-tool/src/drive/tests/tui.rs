use super::*;
use crate::tests::{TuiWorld, tui_approval_reply};
use temper_client_shell_tui::{host::Report, journey};
fn saves() -> (Box<[u8]>, Box<[u8]>) {
    let mut world = TuiWorld::sized(b"127.0.0.1:9", skein_ui_screen::Size { rows: 24, columns: 80 });
    let first = world.save();
    world.mark();
    let exchange = world.exchange(false);
    world.report(Report::Heard(jig_client::Event::Reply { exchange, message: tui_approval_reply() }));
    world.report(Report::Opened { opened: true });
    (first, world.save())
}
fn approve(m: &mut Machine, source: &str) -> Vec<Action> {
    assert!(matches!(line(m, source).as_slice(), [Action::StartTui { .. }]));
    step(m, Event::TuiReady);
    step(m, Event::TuiSaved(saves().1));
    step(m, Event::BrowserAddress(b"http://127.0.0.1:9/projects/3?tab=home".as_slice().into()));
    step(m, Event::FaceDone(Ok(())));
    step(m, Event::Found(Some(NodeId(5))));
    step(m, Event::FaceDone(Ok(())));
    step(m, Event::FaceDone(Ok(())))
}
fn running() -> Machine {
    let mut m = ready();
    approve(&mut m, "tui key enter");
    step(&mut m, Event::TuiWritten);
    m
}
#[test]
fn first_tui_line_starts_confined_and_approves_before_acting() {
    let mut m = ready();
    m.plan.environment.extend([("SSH_TTY".into(), "outside".into()), ("COLORFGBG".into(), "0;15".into())]);
    let actions = line(&mut m, r#"tui see "Ask temper""#);
    let [Action::StartTui { program, rows: 24, columns: 80 }] = actions.as_slice() else { panic!("{actions:?}") };
    assert_eq!(program.program, Path::new("/build/bin/temper-client"));
    assert_eq!(program.args, ["http://127.0.0.1:9"]);
    assert_eq!(program.directory, Path::new("/scratch"));
    let lookup = |key: &str| program.environment.iter().find(|(name, _)| name == key).map(|(_, v)| v.as_os_str());
    for (key, value) in [("HOME", "/scratch/session/home"), ("XDG_STATE_HOME", "/scratch/session/state"),
        ("XDG_RUNTIME_DIR", "/scratch/session/tui-runtime"), ("TERM", "xterm-256color"),
        ("XDG_CONFIG_HOME", "/scratch/session/home/config"), ("XDG_CACHE_HOME", "/scratch/session/home/cache"),
        ("XDG_DATA_HOME", "/scratch/session/home/data"),
        ("PATH", "/scratch/session/bin:/bin"), ("DISPLAY", ":0")] {
        assert_eq!(lookup(key), Some(OsStr::new(value)), "{key}");
    }
    for (key, value) in &program.environment {
        assert!(!key.as_encoded_bytes().starts_with(b"SSH_"));
        assert_ne!(key, "COLORFGBG");
        if key == "HOME" || key == "TMPDIR" || key.as_encoded_bytes().starts_with(b"XDG_") {
            assert!(Path::new(value).starts_with("/scratch/session"));
        }
    }
    assert!(matches!(step(&mut m, Event::TuiReady).as_slice(), [Action::SaveTui]));
    assert!(step(&mut m, Event::TuiSaved(saves().0)).is_empty(), "no browse yet");
    assert!(matches!(m.step(Event::Deadline, Time::from_nanos(50_000_000)).as_slice(), [Action::SaveTui]));
    assert!(matches!(step(&mut m, Event::TuiSaved(saves().1)).as_slice(), [Action::BrowserAddress]));
    let back = b"http://127.0.0.1:9/projects/3?tab=home";
    assert!(matches!(step(&mut m, Event::BrowserAddress(back.as_slice().into())).as_slice(), [Action::Open(url)]
        if url.as_ref() == b"http://127.0.0.1:9/terminal?code=WDJB-MJHT"));
    assert!(matches!(step(&mut m, Event::FaceDone(Ok(()))).as_slice(), [Action::Find(find)]
        if *find == Find::exactly_named(Role::Button, b"Approve")));
    step(&mut m, Event::Found(None));
    assert!(matches!(m.step(Event::Deadline, Time::from_nanos(50_000_000)).as_slice(), [Action::Find(_)]));
    assert!(matches!(step(&mut m, Event::Found(Some(NodeId(5)))).as_slice(),
        [Action::Perform(Doing::Act { node: NodeId(5), act: Act::Press })]));
    assert!(matches!(step(&mut m, Event::FaceDone(Ok(()))).as_slice(), [Action::Open(url)] if url.as_ref() == back));
    assert!(matches!(step(&mut m, Event::FaceDone(Ok(()))).as_slice(), [Action::SaveTui]));
    assert!(step(&mut m, Event::TuiSaved(saves().1)).is_empty(), "Ask temper not shown yet");
}
#[test]
fn keys_use_walk_bytes_and_look_replays_rows() {
    let mut m = running();
    assert!(matches!(line(&mut m, r#"tui key char "/""#).as_slice(), [Action::WriteTui(bytes)] if bytes == b"/"));
    assert_eq!(answer(&step(&mut m, Event::TuiWritten)).print(), "ok\n");
    assert!(matches!(line(&mut m, "tui look").as_slice(), [Action::SaveTui]));
    let mut records = journey::lines(&saves().1).expect("save records");
    for record in &mut records {
        if let journey::Line::End(text) = record { *text = Some(b"not the native frame  ".as_slice().into()); }
    }
    let mut bytes = vec![];
    for record in records { record.write(&mut bytes); }
    let actions = step(&mut m, Event::TuiSaved(bytes.into()));
    assert!(!actions.iter().any(|a| matches!(a, Action::Keep(_))), "looks are not kept");
    let a = answer(&actions);
    assert!(a.failed.is_none());
    assert_eq!(a.shown.len(), 24);
    assert!(a.shown.iter().any(|row| row.contains("then press Approve.")));
    assert!(a.shown.iter().all(|row| !row.ends_with(' ') && !row.contains("not the native frame")));
    assert!(a.print().lines().take(24).all(|row| row.starts_with("| ")));
    assert!(a.print().ends_with("ok\n"));
}
#[test]
fn see_polls_native_final_frame_until_thirty_seconds() {
    let mut m = running();
    line(&mut m, r#"tui see "then press Approve.""#);
    assert!(step(&mut m, Event::TuiSaved(saves().0)).is_empty());
    assert!(matches!(m.step(Event::Deadline, Time::from_nanos(50_000_000)).as_slice(), [Action::SaveTui]));
    assert_eq!(answer(&step(&mut m, Event::TuiSaved(saves().1))).print(), "ok\n");
    line(&mut m, r#"tui see "Missing""#);
    step(&mut m, Event::TuiSaved(saves().1));
    assert!(!m.step(Event::Deadline, Time::from_nanos(29_999_999_999)).iter().any(|a| matches!(a, Action::Answer(_))));
    m.step(Event::TuiSaved(saves().1), Time::from_nanos(29_999_999_999));
    let actions = m.step(Event::Deadline, Time::from_nanos(30_000_000_000));
    let why = answer(&actions).failed.as_deref().expect("30 s expired");
    assert!(why.contains("Missing") && why.contains("then press Approve.") && why.contains("line 3"), "{why}");
    assert!(m.ready());
    line(&mut m, r#"tui see "not the native frame""#);
    let mut records = journey::lines(&saves().1).expect("save");
    let deadline = records.iter_mut().rev().find(|r| matches!(r, journey::Line::Deadline(_))).expect("deadline");
    *deadline = journey::Line::Deadline(Some(Time::from_nanos(u64::MAX)));
    let mut bad = vec![];
    for record in records { record.write(&mut bad); }
    assert!(answer(&step(&mut m, Event::TuiSaved(bad.into()))).failed.as_ref().expect("stale").contains("stale TUI save"));
}
#[test]
fn missing_recording_fails_after_ten_seconds_and_continues() {
    let mut m = ready();
    line(&mut m, "tui look");
    step(&mut m, Event::TuiReady);
    assert!(m.step(Event::Deadline, Time::from_nanos(9_999_999_999)).is_empty());
    let actions = m.step(Event::Deadline, Time::from_nanos(10_000_000_000));
    let why = answer(&actions).failed.as_ref().expect("save bound");
    assert!(why.contains("line 1") && why.contains("10 s") && why.contains("client's recording build"), "{why}");
    assert!(matches!(line(&mut m, "tui look").as_slice(), [Action::StartTui { .. }]), "failed approval restarts");
    step(&mut m, Event::TuiFailure("TUI exited".into()));
    assert_eq!(answer(&line(&mut m, "")).print(), "ok\n");
}
#[test]
fn stop_is_idempotent_restarts_and_end_stops_tui_first() {
    let mut m = ready();
    assert_eq!(answer(&line(&mut m, "tui stop")).print(), "ok\n");
    approve(&mut m, "tui key enter");
    step(&mut m, Event::TuiWritten);
    assert!(matches!(line(&mut m, "tui stop").as_slice(), [Action::StopTui]));
    assert_eq!(answer(&step(&mut m, Event::TuiStopped)).print(), "ok\n");
    assert_eq!(answer(&line(&mut m, "tui stop")).print(), "ok\n");
    assert!(matches!(approve(&mut m, "tui key enter").as_slice(), [Action::WriteTui(_)]));
    step(&mut m, Event::TuiWritten);
    assert!(matches!(step(&mut m, Event::End).as_slice(), [Action::CancelFace, Action::StopCtl, Action::StopTui]));
    assert!(matches!(step(&mut m, Event::TuiStopped).as_slice(), [Action::CloseBrowser]));
    assert!(matches!(step(&mut m, Event::FaceDone(Ok(()))).as_slice(), [Action::TerminateBox]));
}
#[test]
fn exit_and_failed_approval_leave_session_usable() {
    let mut m = running();
    line(&mut m, "tui look");
    assert!(answer(&step(&mut m, Event::TuiFailure("TUI exited".into()))).failed.as_ref().expect("exit").contains("TUI exited"));
    line(&mut m, "tui key enter");
    step(&mut m, Event::TuiReady);
    step(&mut m, Event::TuiSaved(saves().1));
    let back = b"http://127.0.0.1:9/projects/3";
    step(&mut m, Event::BrowserAddress(back.as_slice().into()));
    let actions = step(&mut m, Event::FaceDone(Err("approval page failed".into())));
    assert!(actions.iter().any(|a| matches!(a, Action::Open(url) if url.as_ref() == back)));
    assert!(actions.iter().any(|a| matches!(a, Action::StopTui)));
    assert!(answer(&step(&mut m, Event::FaceDone(Ok(())))).failed.as_ref().expect("approval failed").contains("approval page failed"));
    assert!(m.ready());
}
