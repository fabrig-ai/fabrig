use super::*;
use crate::drive::{Action as DriveAction, Event, Machine, Output, Plan, Process};
use crate::tests::TuiWorld;
use skein_fake_person::{Doing, Find};
use skein_lib::Time;
use skein_ui::{Act, NodeId, Role};
use std::ffi::OsStr;

const PARENT: &str = "walk 2\nweb see \"Welcome to temper\"\nweb press link \"Set up temper\"\nweb see \"Run on this machine\"\nweb mark\n";
const CHILD: &str = "walk 2\nafter parent\nweb see \"Run on this machine\"\nweb mark\n";
const TERMINAL: &str = "walk 2\ntui see \"then press Approve.\"\ntui approve\ntui see \"Setting temper up makes your first project\"\ntui mark\n";

fn journey(root: &Path, name: &str, lines: &str) -> std::path::PathBuf {
    let directory = root.join(name);
    fs::create_dir(&directory).expect("journey directory");
    fs::write(directory.join("walk"), lines).expect("walk");
    directory
}

fn scratch_walk(name: &str) -> (skein_scratch::Scratch, Roots, drive::Options) {
    let scratch = skein_scratch::Scratch::new("journey-walk");
    let root = scratch.path();
    let roots = Roots { web: root.join("web"), tui: root.join("tui") };
    for path in [&roots.web, &roots.tui, &root.join("box")] {
        fs::create_dir(path).expect("directory");
    }
    let arguments = [
        name,
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
        "420",
    ]
    .iter()
    .map(|word| word.as_bytes().into())
    .collect::<Vec<_>>();
    let (_, options) = arguments_for_walk(&arguments).expect("walk options");
    (scratch, roots, options)
}
fn fixture() -> (skein_scratch::Scratch, Roots, drive::Options) {
    let (scratch, roots, options) = scratch_walk("child");
    for (name, lines) in [("parent", PARENT), ("child", CHILD)] {
        fs::write(journey(&roots.web, name, lines).join("scripts.json"), "[]").expect("scripts");
    }
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

struct Walked {
    given: Vec<(String, String)>,
    printed: Vec<String>,
    kept: Vec<String>,
    exit: drive::Exit,
}

fn walk_scripted(
    options: drive::Options,
    mut walker: Walker,
    first: Action,
    mut face: impl FnMut(DriveAction) -> Event,
) -> Walked {
    let mut machine = ready(options);
    let mut actions = VecDeque::from([first]);
    let mut given = vec![];
    let mut printed = vec![];
    let mut kept = vec![];
    while let Some(action) = actions.pop_front() {
        match action {
            Action::Line(line) => {
                given.push((line.source, String::from_utf8(line.bytes.to_vec()).expect("UTF-8 line")));
                let mut effects: VecDeque<_> = machine.step(Event::Line(line.bytes), Time::ZERO).into();
                while let Some(effect) = effects.pop_front() {
                    match effect {
                        DriveAction::Keep(bytes) => {
                            kept.push(String::from_utf8(bytes.into_vec()).expect("UTF-8 kept line"));
                        }
                        DriveAction::Answer(answer) => actions.extend(walker.answer(answer)),
                        effect => effects.extend(machine.step(face(effect), Time::ZERO)),
                    }
                }
            }
            Action::Print(text) => printed.push(text),
            Action::End => return Walked { given, printed, kept, exit: walker.ended(end()) },
        }
    }
    panic!("walk did not end: {printed:?}");
}

#[test]
fn chain_walks_one_journey_session_root_first_with_scripts_and_only_its_own_marks() {
    const WELCOME: &str = "Welcome to temper";
    const SET_UP: &str = "Run on this machine";
    const SET_UP_LINK: NodeId = NodeId(2);
    const LOOK_WIDTH: u32 = 420;
    let (_scratch, roots, options) = fixture();
    let (options, walker, first) = prepare(&roots, "child", options).expect("prepared chain");
    assert!(options.journey && !options.ctl && options.strict);
    assert_eq!(options.ports.as_deref(), Some("1,2,3,4,5,6,7,8,9"));
    assert_eq!(options.widths, [LOOK_WIDTH]);
    assert_eq!(options.scripts, [roots.web.join("parent/scripts.json"), roots.web.join("child/scripts.json")]);
    let look = options.scratch.join("session/looks/0005-420.png");
    let mut page = WELCOME;
    let mut looks = vec![];
    let result = walk_scripted(options, walker, first, |effect| match effect {
        DriveAction::Find(find) if find == Find::text(page.as_bytes()) => Event::Found(Some(NodeId(1))),
        DriveAction::Find(find) if page == WELCOME && find == Find::exactly_named(Role::Link, b"Set up temper") => {
            Event::Found(Some(SET_UP_LINK))
        }
        DriveAction::Perform(Doing::Act { node: SET_UP_LINK, act: Act::Press }) if page == WELCOME => {
            page = SET_UP;
            Event::FaceDone(Ok(()))
        }
        DriveAction::Screenshot { width, path } => {
            looks.push((width, path));
            Event::FaceDone(Ok(()))
        }
        DriveAction::Snapshot => Event::Snapshot(format!("main {page}").into_bytes().into()),
        other => panic!("unexpected session effect on {page:?}: {other:?}"),
    });
    assert_eq!(
        result.given,
        [
            ("parent:2".into(), "web see \"Welcome to temper\"".into()),
            ("parent:3".into(), "web press link \"Set up temper\"".into()),
            ("parent:4".into(), "web see \"Run on this machine\"".into()),
            ("child:3".into(), "web see \"Run on this machine\"".into()),
            ("child:4".into(), "web mark".into()),
        ]
    );
    assert_eq!(
        result.printed,
        [
            "parent:2 ok".to_owned(),
            "parent:3 ok".into(),
            "parent:4 ok".into(),
            "child:3 ok".into(),
            format!("child:4 | {}", look.display()),
            "child:4 | main Run on this machine".into(),
            "child:4 ok".into(),
        ]
    );
    assert_eq!(looks, [(LOOK_WIDTH, look)]);
    assert_eq!(result.exit.status, 0);
    assert_eq!(result.exit.shown.last().expect("summary"), "walked child");
    assert_eq!(fs::read_to_string(roots.web.join("parent/walk")).expect("ancestor unchanged"), PARENT);
}

fn launchpad_opened() -> temper_client_shell_tui::host::Report {
    use jig_client_protocol_wire::{self as w, Record as _};
    let limits = temper_client_shell_tui::limits::tui();
    let owners = limits.native.client.client.domain.owners;
    let mut served = skein_lib::List::with_capacity(u32::try_from(owners.len() + 1).expect("owner count"));
    served.push(w::Served { owner: w::Owner::Jig, first: w::VERSION, last: w::VERSION }).expect("Jig owner");
    for owner in owners {
        let family = w::Owner::Family(w::Family { number: owner.family });
        served.push(w::Served { owner: family, first: owner.version, last: owner.version }).expect("family owner");
    }
    let hello = w::Hello { stream: [7; 16], served, heartbeat: skein_lib::Duration::from_secs(30) };
    let event = jig_client_test_records::opened(hello, jig_client_test_records::frame_record());
    let message = jig_client::Streamed::Record(event.encode(&w::CEILINGS).expect("empty launchpad stream"));
    temper_client_shell_tui::host::Report::Heard(jig_client::Event::Streamed { message })
}

#[test]
fn a_tui_walk_answers_by_its_lines_and_keeps_approve_alone() {
    const HOME: &[u8] = b"http://127.0.0.1:9/";
    const APPROVAL: &[u8] = b"http://127.0.0.1:9/terminal?code=WDJB-MJHT";
    const APPROVE: NodeId = NodeId(5);
    let (_scratch, roots, options) = scratch_walk("terminal");
    journey(&roots.tui, "terminal", TERMINAL);
    let (options, walker, first) = prepare(&roots, "terminal", options).expect("prepared terminal walk");
    let mut world = None;
    let mut page = HOME;
    let mut approved = false;
    let result = walk_scripted(options, walker, first, |effect| match effect {
        DriveAction::StartTui { rows, columns, .. } if world.is_none() => {
            let mut tui = TuiWorld::sized(b"127.0.0.1:9", skein_ui_screen::Size { rows, columns });
            tui.mark();
            tui.offer_approval(true);
            world = Some(tui);
            Event::TuiReady
        }
        DriveAction::SaveTui => Event::TuiSaved(world.as_mut().expect("TUI started").save()),
        DriveAction::BrowserAddress => Event::BrowserAddress(page.into()),
        DriveAction::Open(url) if page == HOME && url.as_ref() == APPROVAL && !approved => {
            page = APPROVAL;
            Event::FaceDone(Ok(()))
        }
        DriveAction::Find(find) if page == APPROVAL && find == Find::exactly_named(Role::Button, b"Approve") => {
            Event::Found(Some(APPROVE))
        }
        DriveAction::Perform(Doing::Act { node: APPROVE, act: Act::Press }) if page == APPROVAL && !approved => {
            let tui = world.as_mut().expect("TUI started");
            tui.accept_approval();
            tui.report(launchpad_opened());
            approved = true;
            Event::FaceDone(Ok(()))
        }
        DriveAction::Open(url) if page == APPROVAL && url.as_ref() == HOME && approved => {
            page = HOME;
            Event::FaceDone(Ok(()))
        }
        other => panic!("unexpected TUI walk effect: {other:?}"),
    });
    assert_eq!(result.exit.status, 0, "{:?}", result.printed);
    assert_eq!(&result.printed[..3], ["terminal:2 ok", "terminal:3 ok", "terminal:4 ok"]);
    assert!(
        result.printed.iter().any(
            |line| line.starts_with("terminal:5 | ") && line.contains("Setting temper up makes your first project")
        ),
        "{:?}",
        result.printed
    );
    assert_eq!(result.printed.last().expect("mark answer"), "terminal:5 ok");
    assert_eq!(
        result.kept,
        ["tui see \"then press Approve.\"", "tui approve", "tui see \"Setting temper up makes your first project\""]
    );
}

#[test]
fn failed_line_stops_before_the_next_and_preserves_session_exits() {
    let (_scratch, roots, options) = fixture();
    let (_, mut walker, _) = prepare(&roots, "child", options).expect("chain");
    let actions = walker.answer(drive::Answer { shown: vec![], failed: None });
    assert!(matches!(actions.last(), Some(Action::Line(line)) if line.source == "parent:3"));
    let actions =
        walker.answer(drive::Answer { shown: vec!["shown instead".into()], failed: Some("line 2: WHY".into()) });
    assert!(
        matches!(actions.as_slice(), [Action::Print(first), Action::Print(second), Action::End] if first == "parent:3 | shown instead" && second == "parent:3 failed: line 2: WHY")
    );
    assert!(walker.lines.is_empty());
    let exit = walker.ended(end());
    assert_eq!(exit.status, 5);
    assert_eq!(exit.shown.last().expect("stop"), "stopped at parent:3: WHY");
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
        let exit = walk(&roots, "child", options.clone(), &[], &mut output);
        assert_eq!(exit.status, 1);
        let error = exit.error.expect("held refusal");
        assert!(error.contains("config/") && error.contains("data/"), "{error}");
        fs::remove_dir(path).expect("remove held directory");
        assert_eq!(fs::read_to_string(&kept).expect("untouched session"), "previous");
    }
    let mut overlap = options;
    overlap.scratch = roots.web.join("parent");
    let exit = walk(&roots, "child", overlap, &[], &mut output);
    assert_eq!(exit.status, 1);
    assert!(exit.error.expect("root overlap").contains("overlap journey root"));
    assert_eq!(output, Vec::<u8>::new());
}
