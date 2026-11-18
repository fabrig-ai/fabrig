//! Native saves and walker decisions across placeholders (client/journey-placeholders.md, 4.2).
use super::*;
use jig_client_protocol_wire as w;
use jig_client_shell_web::adapter::{Asked, Heard};
use skein_lib::{
    FixedClock, FixedStart, List, Overflow, Token, Wall, Writer,
    base64::{self, Alphabet},
};
use skein_ui_web::{mapping::Primary, page::Page};
use temper_client_shell_web::{
    host::{Host, Report},
    journey::{Facts, Recorder},
};

// Keep native reports as text lines, with no browser or box.
struct Capture {
    host: Host,
    page: Page<'static>,
    recorder: Recorder,
    now: Time,
    clock: Option<FixedClock>,
}
impl Capture {
    fn new() -> Self {
        Self::starting(None)
    }
    fn fixed() -> Self {
        Self::starting(Some(FixedStart::read(FIXED_START.as_bytes()).expect("walk start")))
    }
    fn starting(start: Option<FixedStart>) -> Self {
        let facts = Facts {
            fixed: start.is_some(),
            seed: start.map_or(11, |start| start.seed),
            now: start.map_or(Time::ZERO, |_| FixedStart::NOW),
            wall: start.map_or(Wall::EPOCH, |start| start.wall),
            primary: Primary::Control,
            zone_minutes: 0,
            ways: None,
            forges: vec![],
            path: b"/forges/connect?forge=managed".as_slice().into(),
            secret: None,
            stored: vec![],
        };
        let (host, page) = facts.start();
        Self {
            host,
            page,
            recorder: Recorder::new(&facts),
            now: Time::ZERO,
            clock: start.map(|start| FixedClock::new(&start, Time::ZERO)),
        }
    }
    fn report(&mut self, report: Report) -> Vec<Asked> {
        self.now = self.now.saturating_add(Duration::from_millis(10));
        let (now, wall) = self.clock.as_mut().map_or((self.now, Wall::EPOCH), |clock| clock.read(self.now));
        self.host.clocks(now, wall);
        self.recorder.turn(now, wall);
        let duties = self.recorder.report(&mut self.host, &self.page, report);
        self.host.draw(&mut self.page, &mut Vec::new());
        self.recorder.deadline(self.host.deadline());
        if let Some(clock) = &mut self.clock {
            let _next = clock.until(self.host.deadline(), self.now);
        }
        duties.asked
    }
    fn stream(&mut self, event: &w::StreamEvent) -> Vec<Asked> {
        let bytes = encoded(event.measure(), |writer| event.encode(writer));
        let size = base64::encoded_len(Alphabet::Standard, u32::try_from(bytes.len()).expect("fixture size"))
            .expect("base64 size");
        let mut writer = Writer::new(usize::try_from(size).expect("fixture size"));
        base64::encode(Alphabet::Standard, &bytes, &mut writer).expect("sized base64");
        self.report(Report::Heard(Heard::Piece {
            bytes: [b"data: ".as_slice(), &writer.finish(), b"\n\n"].concat().into(),
        }))
    }
    fn act(&mut self, role: Role, name: &[u8], act: Act) {
        let node = self
            .host
            .tree()
            .nodes()
            .iter()
            .find(|node| node.role == role && node.name.as_deref() == Some(name))
            .unwrap_or_else(|| panic!("missing {role:?} {}: {}", String::from_utf8_lossy(name), self.page.html()))
            .id;
        self.report(Report::Act { node, act });
    }
    fn save(&self) -> Box<[u8]> {
        self.recorder.save(&self.page)
    }
}
fn encoded(size: u32, write: impl FnOnce(&mut Writer) -> Result<(), Overflow>) -> Box<[u8]> {
    let mut writer = Writer::new(usize::try_from(size).expect("fixture size"));
    write(&mut writer).expect("sized fixture");
    writer.finish()
}
fn managed() -> Capture {
    managed_capture(Capture::new())
}
fn managed_capture(mut capture: Capture) -> Capture {
    capture.report(Report::Start);
    capture.report(Report::Heard(Heard::StreamOpened));
    let mut served = List::with_capacity(1);
    served
        .push(
            w::Served::new(&w::CEILINGS, w::ServedParts { owner: w::Owner::Jig, first: w::VERSION, last: w::VERSION })
                .expect("served jig"),
        )
        .expect("one owner");
    let mut me = jig_client_test_records::frame_two_projects_record().into_parts();
    me.deployment_role = Some(w::HeldRole::Owner);
    capture.stream(&jig_client_test_records::opened(
        w::Hello::new(&w::CEILINGS, w::HelloParts { stream: [7; 16], served, heartbeat: Duration::from_secs(30) })
            .expect("hello"),
        w::Me::new(&w::CEILINGS, me).expect("deployment owner"),
    ));
    capture.report(Report::Place { path: b"/forges/connect?forge=managed".as_slice().into() });
    capture.act(Role::TextField, b"Its address", Act::Input { value: b"https://forge.example".as_slice().into() });
    capture.act(
        Role::TextField,
        b"Its web address",
        Act::Input { value: b"http://127.0.0.1:41234/web".as_slice().into() },
    );
    capture
}

#[test]
fn a_still_save_finds_a_control_across_placeholders() {
    let mut capture = managed();
    let save = capture.save();
    capture.act(Role::Button, b"Connect as its admin", Act::Press);
    let committed = normalised(&capture.save()).expect("normalised source");
    let step = walked("managed", &committed)
        .expect("source walk")
        .into_iter()
        .find(|step| matches!(step, Step::Act { act: Act::Press, .. }))
        .expect("press step");
    let Step::Act { act, event } = &step else { panic!("press step") };
    let placeholder = format!("{:?}", b"http://127.0.0.1:10001/web");
    assert!(
        event.windows(placeholder.len()).any(|part| part == placeholder.as_bytes()),
        "the recorded event holds the placeholder's decimal elements"
    );
    assert_eq!(journey::control(&save, act, event), Ok(None), "raw save cannot give the placeholder event");
    let mut walker = Walker::new("managed", vec![step], None);
    walker.phase = Phase::Still { save: save.into_vec() };
    let mut face = BrowserFace::new(Token::new(1), "http://127.0.0.1:41234");
    walker.progress(&mut face, Time::ZERO).expect("normalised still save");
    assert!(
        matches!(&walker.phase, Phase::Find { control } if *control == Control { role: Role::Button, name: b"Connect as its admin".as_slice().into(), order: 0 })
    );
}

fn walk_save(port: &str, act: &str, marked: bool) -> Vec<u8> {
    format!("journey 2\nseed 11\nclocks 0 0\nprimary control\nways \"\" \"http://127.0.0.1:{port}\"\npath \"/\"\nturn 0 0\nact {act} \"Copy http://127.0.0.1:{port}/web\" \"Copy\"\ndeadline none\nturn 1000000000 0\n{}end\n", if marked { "mark\ndeadline none\n" } else { "" }).into_bytes()
}
#[test]
fn the_walk_check_normalises_both_recordings_and_keeps_real_differences() {
    let walker = Walker::new("ports", vec![Step::End], None);
    let live = walk_save("41234", "press", true);
    let committed = normalised(&live).expect("committed placeholders");
    assert!(walker.check(&committed, &live).is_ok());
    // The old source is normalised too, even when it predates placeholders.
    assert!(walker.check(&walk_save("42345", "press", true), &live).is_ok());
    for (new, step) in [(walk_save("42345", "toggle", true), 1), (walk_save("42345", "press", false), 2)] {
        assert!(
            matches!(walker.check(&committed, &new), Err(Refusal::WalkDiffers { name, step: found }) if name == "ports" && found == step)
        );
    }
}

#[test]
fn a_refused_normalisation_is_an_unreadable_save_at_its_line() {
    let mut walker =
        Walker::new("ports", vec![Step::Mark, Step::Act { act: Act::Press, event: b"Event".as_slice().into() }], None);
    walker.at = 1;
    for (save, line) in [(b"unknown\n".as_slice(), 1), (b"journey 2\nheard piece \"data: !bad\\x0a\"\n", 2)] {
        walker.phase = Phase::Still { save: save.into() };
        let mut face = BrowserFace::new(Token::new(1), "http://127.0.0.1:41234");
        let refusal = walker.progress(&mut face, Time::ZERO).expect_err("refused save");
        assert!(
            matches!(refusal, Refusal::Save { name, step: 2, refusal: journey::Refusal::Unreadable(journey::Unreadable { line: found, .. }) } if name == "ports" && found == line)
        );
    }
}

#[test]
fn a_typed_placeholder_is_given_as_committed() {
    use skein_browser::boundary::{Event, Request, Seen, States};
    let value = b"http://127.0.0.1:10001/web";
    let mut walker = Walker::new(
        "ports",
        vec![
            Step::Act { act: Act::Input { value: value.as_slice().into() }, event: b"Event".as_slice().into() },
            Step::End,
        ],
        None,
    );
    walker.phase =
        Phase::Find { control: Control { role: Role::TextField, name: b"Address".as_slice().into(), order: 0 } };
    let mut face = BrowserFace::new(Token::new(1), "http://127.0.0.1:41234");
    walker.progress(&mut face, Time::ZERO).expect("find field");
    let Request::Find { op, .. } = face.requests().next().expect("field query") else { panic!("find request") };
    let mut seen = List::with_capacity(1);
    seen.push(Seen {
        node: 42,
        role: b"textbox".as_slice().into(),
        name: b"Address".as_slice().into(),
        value: b"http://127.0.0.1:41234/web".as_slice().into(),
        states: States {
            focused: false,
            disabled: false,
            checked: false,
            expanded: false,
            selected: false,
            pressed: false,
        },
        rect: None,
    })
    .expect("one field");
    face.event(&Event::Found { op, seen, more: 0 });
    let next = walker.progress(&mut face, Time::ZERO).expect("type committed value");
    assert!(matches!(next, Next::Do(Doing::Act { act: Act::Input { value: found }, .. }) if found.as_ref() == value));
}

#[test]
fn an_end_save_is_kept_raw_for_add() {
    let save = walk_save("41234", "press", true);
    let mut walker = Walker::new("ports", vec![Step::End], None);
    walker.phase = Phase::Still { save: save.clone() };
    let mut face = BrowserFace::new(Token::new(1), "http://127.0.0.1:41234");
    assert_eq!(walker.progress(&mut face, Time::ZERO).expect("end save"), Next::Done);
    assert_eq!(walker.recording, Some(save));
}

fn fixed_walk() -> Vec<u8> {
    let mut capture = managed_capture(Capture::fixed());
    let before = capture.save();
    capture.act(Role::Button, b"Connect as its admin", Act::Press);
    let after = capture.save();
    let committed = normalised(&after).expect("normalised walk");
    let press = walked("fixed", &committed)
        .expect("walk")
        .into_iter()
        .find(|step| matches!(step, Step::Act { act: Act::Press, .. }))
        .expect("press");
    let mut walker = Walker::new("fixed", vec![press, Step::End], None);
    let mut face = BrowserFace::new(Token::new(1), "http://127.0.0.1:41234");
    walker.phase = Phase::Still { save: before.into_vec() };
    walker.progress(&mut face, Time::ZERO).expect("find press in still save");
    assert!(matches!(&walker.phase, Phase::Find { control } if control.name.as_ref() == b"Connect as its admin"));
    walker.advance();
    walker.phase = Phase::Still { save: after.into_vec() };
    assert_eq!(walker.progress(&mut face, Time::ZERO).expect("end walk"), Next::Done);
    let recording = walker.recording.take().expect("end recording");
    walker.check(&committed, &recording).expect("same walk");
    recording
}

#[test]
fn a_fixed_capture_keeps_its_start_through_add() {
    let recording = fixed_walk();
    let scratch = skein_scratch::Scratch::new("journeys-fixed-add");
    crate::add(scratch.path(), "fixed", &recording, &[]).expect("redact, prove, normalise and scan");
    let added = fs::read(scratch.path().join("fixed/recording")).expect("added recording");
    let header = b"journey 2\nseed 7\nclocks 3600000000000 1767690000000000000\nfixed\n";
    assert!(recording.starts_with(header), "capture keeps the fixed facts");
    assert!(added.starts_with(header), "add keeps the fixed facts");
    let mut replay = journey::Replay::new(&added).expect("added fixed recording");
    let mut marks = 0;
    while replay.next().expect("added fixed recording replays").is_some() {
        marks += 1;
    }
    assert_eq!(marks, 2, "the press mark and the end are replayed");
}

#[test]
fn two_fixed_walks_and_their_added_recordings_are_identical() {
    let first = fixed_walk();
    let second = fixed_walk();
    assert!(first == second, "fresh captures of the same scripted walk have identical bytes");
    let scratch = skein_scratch::Scratch::new("journeys-fixed-twice");
    for (name, recording) in [("first", first), ("second", second)] {
        crate::add(scratch.path(), name, &recording, &[]).expect("add fixed walk");
    }
    let first = fs::read(scratch.path().join("first/recording")).expect("first added recording");
    let second = fs::read(scratch.path().join("second/recording")).expect("second added recording");
    assert!(first == second, "add writes identical recording bytes");
}
