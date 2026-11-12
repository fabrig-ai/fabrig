use super::*;
use skein_fake_person::Target;
use std::{collections::VecDeque, ffi::OsStr, path::Path};
fn plan() -> Plan {
    Plan::new(
        Options {
            bin: "/build/bin".into(),
            bundle: "/build/bundle".into(),
            scratch: "/scratch".into(),
            scripts: vec!["/build/one.script".into(), "/build/two.script".into()],
            strict: true,
        },
        "/browser/chromium".into(),
        OsStr::new("/bin"),
    )
}
fn step(m: &mut Machine, event: Event) -> Vec<Action> {
    m.step(event, Time::ZERO)
}
fn exited(process: Process, code: i32) -> Event {
    Event::Exited { process, code, stdout: Box::new([]), stderr: Box::new([]) }
}
fn ready() -> Machine {
    let (mut m, _) = Machine::start(plan(), Time::ZERO);
    step(
        &mut m,
        Event::BoxLine { output: Output::Stdout, line: b"http://127.0.0.1:9/link?secret=first\n".as_slice().into() },
    );
    step(&mut m, exited(Process::SignIn, 0));
    step(&mut m, Event::FaceDone(Ok(())));
    step(
        &mut m,
        Event::BoxLine { output: Output::Stdout, line: b"http://127.0.0.1:9/link?secret=fresh\n".as_slice().into() },
    );
    step(&mut m, Event::FaceDone(Ok(())));
    step(&mut m, Event::FaceDone(Ok(())));
    assert!(matches!(step(&mut m, Event::Found(Some(NodeId(0)))).as_slice(), [Action::Ready]));
    m
}
fn line(m: &mut Machine, source: &str) -> Vec<Action> {
    step(m, Event::Line(source.as_bytes().into()))
}
fn answer(actions: &[Action]) -> &Answer {
    actions.iter().find_map(|a| if let Action::Answer(a) = a { Some(a) } else { None }).expect("answer")
}
fn finish(actions: &[Action]) -> &Exit {
    actions.iter().find_map(|a| if let Action::Finished(a) = a { Some(a) } else { None }).expect("exit")
}
#[test]
fn startup_orders_sign_ins_and_confines_programs() {
    let (mut m, actions) = Machine::start(plan(), Time::ZERO);
    let [Action::Start { process: Process::Box, program }] = actions.as_slice() else { panic!("{actions:?}") };
    assert_eq!(program.program, Path::new("/build/bin/temper-box"));
    assert_eq!(
        program.args,
        [
            "--fake-backends",
            "--bundle",
            "/build/bundle",
            "--config",
            "/scratch/config",
            "--data",
            "/scratch/data",
            "--cache",
            "/scratch/cache",
            "--scripts",
            "/build/one.script",
            "--scripts",
            "/build/two.script",
            "--strict"
        ]
    );
    assert_eq!(program.input, Input::Null);
    assert_eq!(program.directory, Path::new("/scratch"));
    for (key, value) in &program.environment {
        if key != "PATH" {
            assert!(Path::new(value).starts_with("/scratch/session"));
        }
    }
    let first = b"http://127.0.0.1:9/link?secret=first\n";
    let actions = step(&mut m, Event::BoxLine { output: Output::Stdout, line: first.as_slice().into() });
    let [Action::Log { .. }, Action::Start { process: Process::SignIn, program }] = actions.as_slice() else {
        panic!("{actions:?}")
    };
    assert_eq!(program.program, Path::new("/build/bin/temper"));
    assert_eq!(program.args, ["ctl", "/scratch/session/ctl", "sign-in", "-"]);
    assert_eq!(program.input, Input::Bytes(first.as_slice().into()));
    assert!(
        matches!(step(&mut m, exited(Process::SignIn, 0)).as_slice(), [Action::Open(url)] if url.as_ref() == first.strip_suffix(b"\n").expect("newline"))
    );
    let fresh = b"http://127.0.0.1:9/link?secret=fresh";
    assert!(matches!(
        step(&mut m, Event::BoxLine { output: Output::Stdout, line: fresh.as_slice().into() }).as_slice(),
        [Action::Log { .. }]
    ));
    assert!(matches!(step(&mut m, Event::FaceDone(Ok(()))).as_slice(), [Action::Open(url)] if url.as_ref() == fresh));
    assert!(
        matches!(step(&mut m, Event::FaceDone(Ok(()))).as_slice(), [Action::Open(url)] if url.as_ref() == b"http://127.0.0.1:9/")
    );
    assert!(
        matches!(step(&mut m, Event::FaceDone(Ok(()))).as_slice(), [Action::Find(find)] if *find == Find::named(Role::Main, b""))
    );
    assert!(matches!(step(&mut m, Event::Found(Some(NodeId(1)))).as_slice(), [Action::Ready]));
    for program in [plan().box_program(), plan().ctl(&[b"me".as_slice().into()], Input::Null)] {
        let text = format!("{program:?}");
        assert!(!text.contains("cargo") && !text.contains("scripts/bundle.sh"));
    }
    let browser = skein_browser::command::command(b"/browser/chromium", b"/scratch/session/browser");
    assert!(browser.args.iter().any(|a| a.as_ref() == b"--user-data-dir=/scratch/session/browser"));
}
#[test]
fn browser_and_ctl_lines_answer_their_effects() {
    let mut m = ready();
    assert!(
        matches!(line(&mut m, r#"web press button "Start""#).as_slice(), [Action::Find(find)] if *find == Find::exactly_named(Role::Button, b"Start").nth(0))
    );
    assert!(matches!(
        step(&mut m, Event::Found(Some(NodeId(7)))).as_slice(),
        [Action::Perform(Doing::Act { node: NodeId(7), act: Act::Press })]
    ));
    assert_eq!(answer(&step(&mut m, Event::FaceDone(Ok(())))).print(), "ok\n");
    assert!(
        matches!(line(&mut m, r#"web input textbox "Name" "Hello""#).as_slice(), [Action::Find(find)] if *find == Find::exactly_named(Role::TextField, b"Name"))
    );
    assert!(
        matches!(step(&mut m, Event::Found(Some(NodeId(8)))).as_slice(), [Action::Perform(Doing::Act { node: NodeId(8), act: Act::Input { value } })] if value.as_ref() == b"Hello")
    );
    step(&mut m, Event::FaceDone(Ok(())));
    assert!(
        matches!(line(&mut m, r#"web see "Hello""#).as_slice(), [Action::Find(find)] if *find == Find::text(b"Hello"))
    );
    assert_eq!(answer(&step(&mut m, Event::Found(Some(NodeId(9))))).print(), "ok\n");
    assert!(
        matches!(line(&mut m, r#"web reload "/projects/1?tab=home""#).as_slice(), [Action::Open(url)] if url.as_ref() == b"http://127.0.0.1:9/projects/1?tab=home")
    );
    step(&mut m, Event::FaceDone(Ok(())));
    step(&mut m, Event::Found(Some(NodeId(1))));
    let actions = line(&mut m, "ctl task show 3 --until ended");
    let [Action::Start { process: Process::Ctl, program }] = actions.as_slice() else { panic!("{actions:?}") };
    assert_eq!(program.program, Path::new("/build/bin/temper"));
    assert_eq!(program.args, ["ctl", "/scratch/session/ctl", "task", "show", "3", "--until", "ended"]);
    assert_eq!(program.input, Input::Null);
    assert_eq!(m.next_deadline(), Some(Time::ZERO.saturating_add(Duration::from_secs(310))));
    let actions = step(
        &mut m,
        Event::Exited {
            process: Process::Ctl,
            code: 2,
            stdout: b"task 3\nended\n".as_slice().into(),
            stderr: b"warning\n".as_slice().into(),
        },
    );
    assert_eq!(answer(&actions).print(), "| warning\n| task 3\n| ended\nfailed: line 5: exit 2\n");
    let mut m = ready();
    line(&mut m, "");
    line(&mut m, "");
    assert!(
        matches!(line(&mut m, "web look 1000").as_slice(), [Action::Screenshot { width: 1000, path }] if path == Path::new("/scratch/session/looks/0003-1000.png"))
    );
    assert!(matches!(step(&mut m, Event::FaceDone(Ok(()))).as_slice(), [Action::Snapshot]));
    assert_eq!(
        answer(&step(&mut m, Event::Snapshot(b"main\n  text Hello\n".as_slice().into()))).print(),
        "| /scratch/session/looks/0003-1000.png\n| main\n|   text Hello\nok\n"
    );
}
#[test]
fn failure_names_sought_controls_keeps_line_and_continues() {
    let mut m = ready();
    line(&mut m, r#"web press button "Missing""#);
    step(&mut m, Event::Found(None));
    let actions = m.step(Event::Deadline, Time::ZERO.saturating_add(STEP));
    assert!(matches!(actions.as_slice(), [Action::CancelFace, Action::Describe { role: Some(Role::Button) }]));
    let actions = step(&mut m, Event::Description(b"button \"Start\"\nbutton \"Cancel\"".as_slice().into()));
    let why = answer(&actions).failed.as_deref().expect("failed");
    for word in ["line 1", "Missing", "Start", "Cancel"] {
        assert!(why.contains(word), "{why}");
    }
    assert!(matches!(actions.first(), Some(Action::Keep(bytes)) if bytes.as_ref() == br#"web press button "Missing""#));
    assert!(m.ready());
    let actions = line(&mut m, "web press button");
    assert!(answer(&actions).failed.as_ref().is_some_and(|why| why.contains("line 2") && why.contains("name")));
    assert!(!actions.iter().any(|a| matches!(a, Action::Keep(_))));
    assert_eq!(answer(&line(&mut m, "")).print(), "ok\n");
}
struct Pages {
    page: u32,
    value: Box<[u8]>,
}
impl Pages {
    fn walk(&mut self, m: &mut Machine, source: &[u8], kept: &mut Vec<Box<[u8]>>) {
        let mut actions: VecDeque<_> = step(m, Event::Line(source.into())).into();
        while let Some(action) = actions.pop_front() {
            let event = match action {
                Action::Find(find) => {
                    let name = match find.target {
                        Target::Role { name, .. } | Target::ExactRole { name, .. } => name,
                        Target::Text { text } => text,
                    };
                    let id = if self.page == 0 && name.as_ref() == b"Start" {
                        1
                    } else if self.page == 1 && name.as_ref() == b"Name" {
                        2
                    } else if self.page == 1 && name.as_ref() == b"Continue" {
                        3
                    } else {
                        panic!("no control {:?} on {}", name, self.page)
                    };
                    Some(Event::Found(Some(NodeId(id))))
                }
                Action::Perform(Doing::Act { node: NodeId(1), act: Act::Press }) => {
                    self.page = 1;
                    Some(Event::FaceDone(Ok(())))
                }
                Action::Perform(Doing::Act { node: NodeId(2), act: Act::Input { value } }) => {
                    self.value = value;
                    Some(Event::FaceDone(Ok(())))
                }
                Action::Perform(Doing::Act { node: NodeId(3), act: Act::Press }) => {
                    self.page = 2;
                    Some(Event::FaceDone(Ok(())))
                }
                Action::Snapshot => Some(Event::Snapshot(format!("page {}", self.page).into_bytes().into())),
                Action::Screenshot { .. } => Some(Event::FaceDone(Ok(()))),
                Action::Keep(bytes) => {
                    kept.push(bytes);
                    None
                }
                Action::Answer(_) => None,
                other => panic!("{other:?}"),
            };
            if let Some(event) = event {
                actions.extend(step(m, event));
            }
        }
    }
}
#[test]
fn kept_steps_replay_on_a_fresh_face() {
    let mut m = ready();
    let mut pages = Pages { page: 0, value: Box::new([]) };
    let mut kept = vec![];
    for source in [
        "",
        "web look",
        r#"web press button "Start""#,
        "web look 1000",
        r#"web input textbox "Name" "Hello""#,
        r#"web press button "Continue""#,
        "web look",
    ] {
        pages.walk(&mut m, source.as_bytes(), &mut kept);
    }
    assert_eq!(kept.len(), 3);
    let mut replay = Pages { page: 0, value: Box::new([]) };
    let mut fresh = ready();
    let mut replay_kept = vec![];
    for source in &kept {
        replay.walk(&mut fresh, source, &mut replay_kept);
    }
    assert_eq!((replay.page, replay.value.as_ref()), (pages.page, pages.value.as_ref()));
    assert_eq!((pages.page, pages.value.as_ref()), (2, b"Hello".as_slice()));
    assert_eq!(kept, replay_kept);
    assert!(matches!(line(&mut m, "end").as_slice(), [Action::CancelFace, Action::StopCtl, Action::CloseBrowser]));
}
#[test]
fn exits_classify_store_refusal_tails_and_clean_sigterm_stop() {
    for (serve, expected) in [
        (vec![b"temper serve: The store in /scratch/data holds an unreadable row".as_slice().into()], 3),
        (vec![b"2026-11-12T01:00:00Z temper serve: The store in /scratch/data cannot open".as_slice().into()], 3),
        (vec![b"temper serve: The store in /scratch-other/data cannot open".as_slice().into()], 4),
        (vec![b"temper serve: The store in /other/scratch/data cannot open".as_slice().into()], 4),
        (vec![], 4),
    ] {
        let (mut m, _) = Machine::start(plan(), Time::ZERO);
        assert!(step(&mut m, exited(Process::Box, 1)).iter().any(|a| matches!(a, Action::CloseBrowser)));
        assert!(matches!(step(&mut m, Event::FaceDone(Ok(()))).as_slice(), [Action::Classify]));
        let box_tail = (0..30).map(|n| format!("box {n}").into_bytes().into()).collect();
        let actions = step(&mut m, Event::Logs { serve, box_tail });
        let exit = finish(&actions);
        assert_eq!(exit.status, expected);
        if expected == 3 {
            assert!(exit.shown[0].starts_with("refused: temper serve: "));
        } else {
            assert!(exit.shown[0].starts_with("box failed: "));
            assert_eq!(exit.shown[1], "| box 10");
            assert_eq!(exit.shown[20], "| box 29");
        }
    }
    let mut m = ready();
    line(&mut m, "end");
    assert!(matches!(step(&mut m, Event::FaceDone(Ok(()))).as_slice(), [Action::TerminateBox]));
    assert_eq!(
        *finish(&step(&mut m, exited(Process::Box, 0))),
        Exit { status: 0, shown: vec!["ended".into()], error: None }
    );
    let mut m = ready();
    step(&mut m, exited(Process::Box, 5));
    assert_eq!(answer(&line(&mut m, "web look")).print(), "failed: line 1: the box exited\n");
    step(&mut m, Event::End);
    step(&mut m, Event::FaceDone(Ok(())));
    assert_eq!(finish(&step(&mut m, Event::Logs { serve: vec![], box_tail: vec![] })).status, 4);
    let mut m = ready();
    step(&mut m, Event::End);
    step(&mut m, Event::FaceDone(Ok(())));
    assert!(matches!(
        m.step(Event::Deadline, Time::ZERO.saturating_add(STEP)).as_slice(),
        [Action::KillBox, Action::Classify]
    ));
}
#[test]
fn ctl_destinations_cannot_escape_scratch() {
    use skein_scratch::Scratch;
    use std::{fs, os::unix::fs::symlink};
    let mut m = ready();
    let actions = line(&mut m, "ctl party make --out /outside/token");
    assert!(answer(&actions).failed.as_ref().is_some_and(|why| why.contains("outside scratch")));
    assert!(!actions.iter().any(|a| matches!(a, Action::Start { .. })));
    let s = Scratch::new("drive-output");
    let root = s.path();
    fs::create_dir(root.join("scratch")).expect("scratch");
    symlink(root, root.join("scratch/escape")).expect("outgoing link");
    let mut p = plan();
    p.options.scratch = root.join("scratch");
    let args = [
        b"party".as_slice().into(),
        b"make".as_slice().into(),
        b"--out".as_slice().into(),
        b"escape/token".as_slice().into(),
    ];
    assert!(super::options::check_destinations(&p.ctl(&args, Input::Null), &p.options.scratch).is_err());
    let args =
        [b"party".as_slice().into(), b"make".as_slice().into(), b"--out".as_slice().into(), b"token".as_slice().into()];
    super::options::check_destinations(&p.ctl(&args, Input::Null), &p.options.scratch).expect("inside scratch");
    assert!(!root.join("token").exists());
}

#[test]
fn preflight_validates_before_reset_and_preserves_box_directories() {
    use skein_scratch::Scratch;
    use std::{fs, os::unix::fs::PermissionsExt};
    let s = Scratch::new("drive-preflight");
    let root = s.path();
    for dir in ["bin", "bundle", "session", "config", "data", "cache"] {
        fs::create_dir(root.join(dir)).expect("fixture directory");
    }
    fs::write(root.join("bundle/index.html"), b"index").expect("index");
    fs::write(root.join("session/lines"), b"previous").expect("old session");
    for dir in ["config", "data", "cache"] {
        fs::write(root.join(dir).join("kept"), dir).expect("box bytes");
    }
    let options = Options {
        bin: root.join("bin"),
        bundle: root.join("bundle"),
        scratch: root.into(),
        scripts: vec![],
        strict: false,
    };
    let why = super::options::prepare(options.clone(), &[]).expect_err("missing binary before any process");
    assert!(why.contains("temper-box"));
    assert_eq!(Exit::own(why).status, 1);
    assert_eq!(fs::read(root.join("session/lines")).expect("old session"), b"previous");
    for name in ["temper-box", "fake-claude", "temper", "chromium"] {
        let file = root.join("bin").join(name);
        fs::write(&file, b"not executed").expect("fixture");
        fs::set_permissions(file, fs::Permissions::from_mode(0o700)).expect("executable");
    }
    let environment = vec![
        (b"SKEIN_TEST_CHROMIUM".as_slice().into(), root.join("bin/chromium").as_os_str().as_encoded_bytes().into()),
        (b"DISPLAY".as_slice().into(), b":7".as_slice().into()),
    ];
    let plan = super::options::prepare(options, &environment).expect("preflight only");
    let tui = plan.tui(b"http://127.0.0.1:9").expect("TUI plan");
    assert!(tui.environment.iter().any(|(name, value)| name == "DISPLAY" && value == ":7"));
    for opener in ["xdg-open", "open"] {
        assert_eq!(
            fs::read_link(root.join("session/bin").join(opener)).expect("opener link"),
            Path::new("/usr/bin/true")
        );
    }
    assert_eq!(fs::read(root.join("session/lines")).expect("fresh lines"), b"");
    for dir in ["config", "data", "cache"] {
        assert_eq!(fs::read(root.join(dir).join("kept")).expect("unchanged"), dir.as_bytes());
    }
}

mod tui;
