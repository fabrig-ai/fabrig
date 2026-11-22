use super::*;
use crate::drive::{Action as DriveAction, Event, Machine, Output, Plan, Process};
use skein_lib::Time;
use skein_ui::NodeId;
use std::ffi::OsStr;

fn fixture() -> (skein_scratch::Scratch, Roots, drive::Options) {
    let scratch = skein_scratch::Scratch::new("journey-walk");
    let root = scratch.path();
    let roots = Roots { web: root.join("web"), tui: root.join("tui") };
    for path in [&roots.web, &roots.tui, &root.join("box")] {
        fs::create_dir(path).expect("directory");
    }
    for (name, lines) in [
        ("a", "walk 2\nweb see \"Ancestor\"\nweb see \"Ready\"\nweb mark\n"),
        ("b", "walk 2\nafter a\nweb see \"Child\"\nweb mark\n"),
    ] {
        let directory = roots.web.join(name);
        fs::create_dir(&directory).expect("journey directory");
        fs::write(directory.join("walk"), lines).expect("walk");
        fs::write(directory.join("scripts.json"), "[]").expect("scripts");
    }
    let arguments = [
        "b",
        "--bin",
        "/build",
        "--bundle",
        "/bundle",
        "--scratch",
        root.join("box").to_str().expect("path"),
        "--strict",
        "--ports",
        "1,2,3,4,5,6,7,8,9",
        "--widths",
        "1440,420",
    ]
    .iter()
    .map(|word| word.as_bytes().into())
    .collect::<Vec<_>>();
    let (_, options) = arguments_for_walk(&arguments).expect("walk options");
    (scratch, roots, options)
}
fn ready(options: drive::Options) -> Machine {
    let plan = Plan::new(options, "/chromium".into(), OsStr::new("/bin"));
    let (mut machine, _) = Machine::start(plan, Time::ZERO);
    for event in [
        Event::BoxLine { output: Output::Stdout, line: b"http://127.0.0.1:9/sign-in?secret=first\n".as_slice().into() },
        Event::Exited { process: Process::SignIn, code: 0, stdout: Box::new([]), stderr: Box::new([]) },
        Event::FaceDone(Ok(())),
        Event::BoxLine { output: Output::Stdout, line: b"http://127.0.0.1:9/sign-in?secret=fresh\n".as_slice().into() },
        Event::FaceDone(Ok(())),
        Event::Found(Some(NodeId(0))),
    ] {
        machine.step(event, Time::ZERO);
    }
    assert!(machine.ready());
    machine
}
fn end() -> drive::Exit {
    drive::Exit { status: 0, shown: vec!["ended".into()], error: None }
}

#[test]
fn chain_walks_one_journey_session_root_first_with_scripts_and_only_its_own_marks() {
    let (_scratch, roots, options) = fixture();
    let (options, mut walker, first) = prepare(&roots, "b", options).expect("prepared chain");
    assert!(options.journey && !options.ctl && options.strict);
    assert_eq!(options.ports.as_deref(), Some("1,2,3,4,5,6,7,8,9"));
    assert_eq!(options.widths, [1440, 420]);
    assert_eq!(options.scripts, [roots.web.join("a/scripts.json"), roots.web.join("b/scripts.json")]);
    let mut machine = ready(options);
    let mut actions = VecDeque::from([first]);
    let mut given = vec![];
    let mut printed = vec![];
    let mut ended = false;
    while let Some(action) = actions.pop_front() {
        match action {
            Action::Line(line) => {
                given.push((line.source, String::from_utf8(line.bytes.to_vec()).expect("UTF-8 line")));
                let mut effects: VecDeque<_> = machine.step(Event::Line(line.bytes), Time::ZERO).into();
                while let Some(effect) = effects.pop_front() {
                    let event = match effect {
                        DriveAction::Find(_) => Some(Event::Found(Some(NodeId(1)))),
                        DriveAction::Screenshot { .. } => Some(Event::FaceDone(Ok(()))),
                        DriveAction::Snapshot => Some(Event::Snapshot(b"main Child".as_slice().into())),
                        DriveAction::Keep(_) => None,
                        DriveAction::Answer(answer) => {
                            actions.extend(walker.answer(answer));
                            None
                        }
                        other => panic!("unexpected session effect: {other:?}"),
                    };
                    if let Some(event) = event {
                        effects.extend(machine.step(event, Time::ZERO));
                    }
                }
            }
            Action::Print(text) => printed.push(text),
            Action::End => {
                ended = true;
                break;
            }
        }
    }
    assert!(ended);
    assert_eq!(
        given,
        [
            ("a:2".into(), "web see \"Ancestor\"".into()),
            ("a:3".into(), "web see \"Ready\"".into()),
            ("b:3".into(), "web see \"Child\"".into()),
            ("b:4".into(), "web mark".into())
        ]
    );
    assert_eq!(&printed[..3], ["a:2 ok", "a:3 ok", "b:3 ok"]);
    assert!(printed.iter().any(|line| line.starts_with("b:4 | ") && line.ends_with("session/looks/0004-1440.png")));
    assert!(printed.iter().any(|line| line == "b:4 | main Child"));
    assert_eq!(printed.last().expect("last answer"), "b:4 ok");
    let exit = walker.ended(end());
    assert_eq!(exit.status, 0);
    assert_eq!(exit.shown.last().expect("summary"), "walked b");
    assert_eq!(
        fs::read_to_string(roots.web.join("a/walk")).expect("ancestor unchanged"),
        "walk 2\nweb see \"Ancestor\"\nweb see \"Ready\"\nweb mark\n"
    );
}

#[test]
fn failed_line_stops_before_the_next_and_preserves_session_exits() {
    let (_scratch, roots, options) = fixture();
    let (_, mut walker, _) = prepare(&roots, "b", options).expect("chain");
    let actions = walker.answer(drive::Answer { shown: vec![], failed: None });
    assert!(matches!(actions.last(), Some(Action::Line(line)) if line.source == "a:3"));
    let actions =
        walker.answer(drive::Answer { shown: vec!["shown instead".into()], failed: Some("line 2: WHY".into()) });
    assert!(
        matches!(actions.as_slice(), [Action::Print(first), Action::Print(second), Action::End] if first == "a:3 | shown instead" && second == "a:3 failed: line 2: WHY")
    );
    assert!(walker.lines.is_empty());
    let exit = walker.ended(end());
    assert_eq!(exit.status, 5);
    assert_eq!(exit.shown.last().expect("stop"), "stopped at a:3: WHY");
    for status in [1, 3, 4] {
        let exit = drive::Exit { status, shown: vec!["session failure".into()], error: Some("error".into()) };
        assert_eq!(walker.ended(exit.clone()), exit);
    }
}

#[test]
fn missing_journey_and_held_scratch_refuse_without_starting_a_session() {
    let (_scratch, roots, options) = fixture();
    fs::create_dir(options.scratch.join("session")).expect("previous session");
    let kept = options.scratch.join("session/lines");
    fs::write(&kept, "previous").expect("previous lines");
    // Executables and Chromium do not exist: these refusals precede drive preflight.
    let mut output = vec![];
    let exit = walk(&roots, "missing", options.clone(), &[], &mut output);
    assert_eq!(exit.status, 1);
    let error = exit.error.expect("missing refusal");
    assert!(error.contains("missing:0") && error.contains("missing journey"), "{error}");
    for directory in ["data", "config"] {
        let path = options.scratch.join(directory);
        fs::create_dir(&path).expect("held directory");
        let exit = walk(&roots, "b", options.clone(), &[], &mut output);
        assert_eq!(exit.status, 1);
        let error = exit.error.expect("held refusal");
        assert!(error.contains("config/") && error.contains("data/"), "{error}");
        fs::remove_dir(path).expect("remove held directory");
        assert_eq!(fs::read_to_string(&kept).expect("untouched session"), "previous");
    }
    let mut overlap = options;
    overlap.scratch = roots.web.join("a");
    let exit = walk(&roots, "b", overlap, &[], &mut output);
    assert_eq!(exit.status, 1);
    assert!(exit.error.expect("root overlap").contains("overlap journey root"));
    assert_eq!(output, Vec::<u8>::new());
}
