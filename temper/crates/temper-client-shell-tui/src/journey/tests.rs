//! The host's world, with fake bytes and no terminal (temper/docs/design/client/tui/journeys.md, 7).
use super::*;
use crate::host::{Duties, Host, Report};
use jig_client::{Message, Request};
use skein_lib::{Secret, Time, Token, Wall};
use skein_ui_screen::{Rgb, Size};
use skein_ui_screen_terminal::Key;
use temper_client_tui::Scheme;

fn facts() -> Facts {
    Facts {
        seed: 7,
        now: Time::ZERO,
        wall: Wall::EPOCH,
        size: Size { rows: 12, columns: 48 },
        scheme: Some(Scheme::Dark),
        background: None,
        boxed: false,
        address: b"127.0.0.1:8367".as_slice().into(),
        front: false,
        local: false,
        note: None,
        kept: Vec::new(),
        credential: None,
    }
}

struct Walk {
    host: Host,
    recorder: Recorder,
    now: u64,
    marks: Vec<(Box<[u8]>, Option<Key>)>,
}
impl Walk {
    fn new() -> Self {
        Self::with(&facts())
    }
    fn with(facts: &Facts) -> Self {
        let (host, start) = facts.start();
        let mut walk = Self { host, recorder: Recorder::new(facts), now: 1, marks: Vec::new() };
        walk.reports(vec![start]);
        walk
    }
    fn begin(&mut self) {
        let now = Time::from_nanos(self.now);
        let wall = Wall::from_nanos(self.now);
        self.host.clocks(now, wall);
        self.recorder.turn(now, wall);
        self.now += 1;
    }
    fn finish(&mut self) {
        self.recorder.deadline(self.host.deadline());
        self.host.frame();
    }
    fn reports(&mut self, reports: Vec<Report>) -> Vec<Duties> {
        self.begin();
        let duties = reports.into_iter().map(|report| self.recorder.report(&mut self.host, report)).collect();
        self.finish();
        duties
    }
    fn key(&mut self, key: Key) -> Vec<Duties> {
        if self.host.event_of(&key).is_some_and(|event| !matches!(event, temper_client_tui::Event::Typed { .. })) {
            self.marks.push((self.host.frame().print(), Some(key.clone())));
        }
        self.reports(vec![Report::Key(key)])
    }
    fn mark(&mut self) {
        self.begin();
        self.marks.push((self.host.frame().print(), None));
        self.recorder.mark(&mut self.host);
        self.finish();
    }
    fn save(&mut self) -> Box<[u8]> {
        self.recorder.save(&mut self.host)
    }
}

fn pasted() -> Walk {
    let mut walk = Walk::new();
    let link = format!("http://127.0.0.1:8367/jig/v1/sign-in/link?secret={}", "A".repeat(43));
    walk.key(Key::Paste(link.into_bytes().into()));
    let duties = walk.key(Key::Enter);
    let exchange = duties
        .iter()
        .flat_map(|duty| &duty.asked)
        .find_map(|request| match request {
            Request::Exchange { exchange, message: Message::SignInLink(_), .. } => Some(*exchange),
            Request::Exchange { .. }
            | Request::OpenStream { .. }
            | Request::CloseStream
            | Request::Save { .. }
            | Request::Erase { .. }
            | Request::Told { .. }
            | Request::Authorized(_)
            | Request::Approval(_)
            | Request::ServiceCredential { .. } => None,
        })
        .expect("pasted link exchange");
    walk.reports(vec![Report::Heard(jig_client::Event::Reply { exchange, message: signed_in() })]);
    walk.reports(vec![Report::Kept { written: true }]);
    walk.mark();
    walk
}

fn written(records: &[Line]) -> Box<[u8]> {
    let mut bytes = Vec::new();
    for record in records {
        record.write(&mut bytes);
    }
    bytes.into()
}

fn replay_stale(recording: &[u8]) -> Stale {
    let mut replay = Replay::new(recording).expect("readable recording");
    loop {
        match replay.next() {
            Err(stale) => return stale,
            Ok(Some(_)) => {}
            Ok(None) => panic!("changed rule was accepted"),
        }
    }
}

#[test]
fn pasted_reports_replay_to_the_same_frames_and_records() {
    let mut walk = pasted();
    let records = lines(&walk.save()).expect("records");
    assert_eq!(records.iter().filter(|line| **line == Line::Keep(Some(credential().into_exposed()))).count(), 1);
    assert!(records.iter().any(|line| *line == Line::Stream(Some(credential().into_exposed()))));
    same_frames(walk);
}

fn same_frames(mut walk: Walk) {
    let file = walk.save();
    let records = lines(&file).expect("readable records");
    assert!(written(&records) == file, "line codec preserves the recording");
    assert!(prove(&file).is_ok(), "the whole walk proves its live frames");
    let mut replay = Replay::new(&file).expect("facts");
    walk.marks.push((walk.host.frame().print(), None));
    for (print, key) in walk.marks {
        let mark = replay.next().expect("same duties").expect("mark");
        assert!(mark.frame.print() == print, "replay yields the live frame");
        assert!(mark.live == Some(print), "mark retains its live print");
        assert_eq!(mark.key, key);
    }
    assert!(replay.next().expect("end").is_none());
}

#[test]
fn each_enter_marks_its_frame_before_the_key() {
    let mut walk = pasted();
    let records = lines(&walk.save()).expect("records");
    let mut enters = 0;
    let mut typed = 0;
    for pair in records.windows(2) {
        if matches!(pair.get(1), Some(Line::Key { key: Key::Enter, .. })) {
            enters += 1;
            assert!(matches!(pair.first(), Some(Line::Mark(Some(_)))), "Enter needs its preceding live mark");
        }
        if matches!(pair.get(1), Some(Line::Key { key: Key::Paste(_), .. })) {
            typed += 1;
            assert!(!matches!(pair.first(), Some(Line::Mark(_))), "typing marks no frame");
        }
    }
    assert_eq!(enters, 1);
    assert_eq!(typed, 1);
}

#[test]
fn changed_duties_deadlines_and_key_events_are_stale() {
    let mut walk = pasted();
    let original = lines(&walk.save()).expect("records");
    let send =
        original.iter().position(|line| matches!(line, Line::Send { message: Sent::SignInLink(_), .. })).expect("send");
    let deadline = original.iter().position(|line| matches!(line, Line::Deadline(_))).expect("deadline");
    let key = original.iter().position(|line| matches!(line, Line::Key { key: Key::Enter, .. })).expect("key");
    let mut changed_send = original[send].clone();
    if let Line::Send { message, .. } = &mut changed_send {
        *message = Sent::SignInLink(b"changed".as_slice().into());
    }
    for (at, replacement) in [
        (send, changed_send),
        (deadline, Line::Deadline(Some(Time::from_nanos(u64::MAX)))),
        (key, Line::Key { key: Key::Enter, event: Some(b"Quit".as_slice().into()) }),
    ] {
        let mut records = original.clone();
        records[at] = replacement.clone();
        assert_eq!(records.iter().zip(&original).filter(|(a, b)| a != b).count(), 1);
        let stale = replay_stale(&written(&records));
        assert_eq!(stale.line, u32::try_from(at + 1).expect("line"));
        let mut expected = Vec::new();
        replacement.write(&mut expected);
        expected.pop();
        assert_eq!(stale.recorded.as_ref(), expected);
        assert_ne!(stale.recorded, stale.asked);
        let turn = original.iter().take(at).filter(|line| matches!(line, Line::Turn { .. })).count();
        assert_eq!(stale.turn, u32::try_from(turn).expect("turn"));
        let mut expected = Vec::new();
        original[at].write(&mut expected);
        expected.pop();
        assert_eq!(stale.asked.as_ref(), expected);
    }
}

#[test]
fn tab_with_no_event_is_kept_and_replayed() {
    let mut walk = Walk::new();
    walk.key(Key::Tab);
    walk.key(Key::BackTab);
    walk.key(Key::Enter);
    let file = walk.save();
    let records = lines(&file).expect("records");
    assert!(records.iter().any(|line| matches!(line, Line::Key { key: Key::Tab, event: None })));
    assert_eq!(
        records.iter().filter(|line| matches!(line, Line::Mark(_))).count(),
        1,
        "only Enter marks a frame; focus-only keys mark nothing"
    );
    assert!(prove(&file).is_ok());
}

#[test]
fn proof_refuses_the_first_differing_print_line() {
    let mut walk = pasted();
    let original = lines(&walk.save()).expect("records");
    for (wanted_mark, expected_line) in [(1, 1), (2, 2), (3, 3)] {
        let mut records = original.clone();
        let mut changed = 0;
        let mut mark_number = 0;
        for record in &mut records {
            if let Line::Mark(print) | Line::End(print) = record {
                mark_number += 1;
                if mark_number == wanted_mark {
                    let live = print.as_ref().expect("live print");
                    let mut replacement = live.to_vec();
                    let offset = if expected_line == 1 {
                        0
                    } else {
                        live.iter()
                            .enumerate()
                            .filter(|(_, byte)| **byte == b'\n')
                            .nth(usize::try_from(expected_line - 2).expect("line offset"))
                            .expect("print line")
                            .0
                            + 1
                    };
                    replacement.insert(offset, b'!');
                    *print = Some(replacement.into());
                    changed += 1;
                }
            }
        }
        assert_eq!(changed, 1);
        assert!(
            matches!(prove(&written(&records)), Err(Refusal::Differs { mark, line }) if mark == wanted_mark && line == expected_line)
        );
    }
}

#[test]
fn proof_drops_every_live_print_after_the_whole_walk() {
    let mut walk = pasted();
    let file = walk.save();
    let mut records = lines(&file).expect("records");
    for record in &mut records {
        if let Line::Mark(print) | Line::End(print) = record {
            *print = None;
        }
    }
    assert!(prove(&file).expect("faithful prints") == written(&records), "proof drops every print");
}

#[test]
fn later_saves_extend_a_whole_recording() {
    let mut walk = pasted();
    let first = walk.save();
    walk.reports(vec![Report::Resize(Size { rows: 14, columns: 60 })]);
    walk.mark();
    let second = walk.save();
    let mut first_records = lines(&first).expect("first");
    assert!(matches!(first_records.pop(), Some(Line::End(Some(_)))));
    let second_records = lines(&second).expect("second");
    assert!(second_records.starts_with(&first_records));
    assert!(second_records.len() > first_records.len());
    assert!(prove(&first).is_ok());
    assert!(prove(&second).is_ok());
}

// Jig's v19 SignedIn wire: answer tag, present token, length, token, party, expiry.
// The credential's first eight bytes are its sign-in number, followed by 32 fake bytes.
fn credential() -> Secret {
    let mut bytes = 7_u64.to_be_bytes().to_vec();
    bytes.extend_from_slice(&[7; 32]);
    Secret::new(bytes.into())
}
fn signed_in() -> Box<[u8]> {
    let credential = credential();
    let mut bytes = vec![0, 1];
    bytes.extend_from_slice(&u32::try_from(credential.len()).expect("credential size").to_be_bytes());
    bytes.extend_from_slice(credential.expose());
    bytes.extend_from_slice(&7_u64.to_be_bytes());
    bytes.extend_from_slice(&300_000_000_000_u64.to_be_bytes());
    bytes.into()
}

#[test]
fn a_start_with_no_terminal_scheme_records_and_replays() {
    let mut facts = facts();
    facts.scheme = None;
    let mut walk = Walk::with(&facts);
    walk.mark();
    assert!(lines(&walk.save()).expect("recording").contains(&Line::Screen { size: facts.size, scheme: None }));
    same_frames(walk);
}

#[test]
fn facts_and_only_kept_turns_have_ordered_lines() {
    for (background, boxed, local) in [
        (None, false, false),
        (None, true, false),
        (Some(Rgb { red: 0, green: 43, blue: 54 }), false, false),
        (Some(Rgb { red: 0, green: 43, blue: 54 }), true, true),
        (Some(Rgb { red: 255, green: 246, blue: 227 }), true, false),
    ] {
        let mut facts = facts();
        facts.background = background;
        facts.boxed = boxed;
        facts.local = local;
        facts.scheme = Some(Scheme::Light);
        facts
            .kept
            .push(jig_client::Record { key: b"test-store".as_slice().into(), bytes: b"test-value".as_slice().into() });
        facts.credential = Some(credential());
        let (mut host, _) = facts.start();
        let mut recorder = Recorder::new(&facts);
        recorder.turn(Time::from_nanos(10), Wall::from_nanos(20));
        recorder.deadline(None);
        let file = recorder.save(&mut host);
        let records = lines(&file).expect("facts");
        let mut expected = vec![
            Line::Journey { version: 2 },
            Line::Seed(7),
            Line::Clocks { now: Time::ZERO, wall: Wall::EPOCH },
            Line::Screen { size: Size { rows: 12, columns: 48 }, scheme: Some(Scheme::Light) },
            Line::Address(b"127.0.0.1:8367".as_slice().into()),
            Line::Stored { key: b"test-store".as_slice().into(), bytes: b"test-value".as_slice().into() },
            Line::Credential(credential().into_exposed()),
            Line::End(Some(host.frame().print())),
        ];
        if let Some(rgb) = background {
            expected.insert(4, Line::Background(rgb));
        }
        let mut at = if background.is_some() { 6 } else { 5 };
        if local {
            expected.insert(at, Line::Local);
            at += 1;
        }
        if boxed {
            expected.insert(at, Line::Box);
        }
        assert!(records == expected, "facts ordered, idle turn omitted");
        let (read, _) = replay::facts(&records).expect("read start facts");
        let (_, Report::Start(start)) = read.start() else { panic!("start report") };
        assert_eq!(start.background, background);
        assert_eq!(start.boxed, boxed);
        assert_eq!(start.local, local);
        assert!(prove(&file).is_ok());
    }
}

#[test]
fn misplaced_records_are_unreadable_before_replay() {
    let mut walk = pasted();
    let original = lines(&walk.save()).expect("records");
    let start = original.iter().position(|line| matches!(line, Line::Start)).expect("start");
    let deadline = original.iter().position(|line| matches!(line, Line::Deadline(_))).expect("deadline");
    let end = original.len() - 1;
    for (at, replacement) in [
        (0, Line::Journey { version: 1 }),
        (1, Line::Address(Box::new([]))),
        (start, Line::Due),
        (deadline, Line::Start),
        (end, Line::Mark(None)),
        (end, Line::Turn { now: Time::ZERO, wall: Wall::EPOCH }),
    ] {
        let mut records = original.clone();
        records[at] = replacement;
        assert_eq!(records.iter().zip(&original).filter(|(a, b)| a != b).count(), 1);
        assert!(Replay::new(&written(&records)).is_err(), "misplaced line {at}");
    }
}

#[test]
fn native_transport_reports_round_trip_through_the_host() {
    let mut walk = Walk::new();
    walk.key(Key::Control(b'c'));
    let reports = vec![
        Report::Heard(jig_client::Event::Lost { exchange: Token::new(500) }),
        Report::Heard(jig_client::Event::Reply { exchange: Token::new(501), message: b"unused".as_slice().into() }),
        Report::Heard(jig_client::Event::Streamed { message: jig_client::Streamed::Record(Box::new([])) }),
        Report::Heard(jig_client::Event::Streamed { message: jig_client::Streamed::Heartbeat }),
        Report::Heard(jig_client::Event::Streamed {
            message: jig_client::Streamed::Full(jig_client::StreamsFull {
                bound: jig_client::StreamBound::Component,
                size: 11,
            }),
        }),
        Report::Heard(jig_client::Event::Streamed {
            message: jig_client::Streamed::Full(jig_client::StreamsFull {
                bound: jig_client::StreamBound::Transport,
                size: 12,
            }),
        }),
        Report::Heard(jig_client::Event::Streamed { message: jig_client::Streamed::SignedOut }),
        Report::Heard(jig_client::Event::StreamGone),
        Report::Heard(jig_client::Event::Stored),
        Report::Heard(jig_client::Event::NotStored),
        Report::Kept { written: false },
        Report::Opened { opened: true },
        Report::Opened { opened: false },
    ];
    walk.reports(reports);
    let file = walk.save();
    let records = lines(&file).expect("records");
    let begin = records.iter().position(|line| matches!(line, Line::Unanswered(500))).expect("transport reports");
    let expected = [
        Line::Unanswered(500),
        Line::Reply { exchange: 501, bytes: b"unused".as_slice().into() },
        Line::Streamed(Streamed::Record(Box::new([]))),
        Line::Streamed(Streamed::Heartbeat),
        Line::Streamed(Streamed::Full { transport: false, size: 11 }),
        Line::Streamed(Streamed::Full { transport: true, size: 12 }),
        Line::Streamed(Streamed::SignedOut),
        Line::Gone,
        Line::Saved,
        Line::Unsaved,
        Line::Kept { written: false },
        Line::Opened { opened: true },
        Line::Opened { opened: false },
        Line::Deadline(None),
    ];
    assert!(records[begin..].starts_with(&expected), "native reports are retained in order");
    assert!(prove(&file).is_ok());
}

fn sent(walk: &mut Walk, kind: fn(&Sent) -> bool) -> Token {
    let records = lines(&walk.save()).expect("records");
    records
        .iter()
        .rev()
        .find_map(|line| {
            if let Line::Send { exchange, message, .. } = line
                && kind(message)
            {
                return Some(Token::new(*exchange));
            }
            None
        })
        .expect("walk's exchange")
}

fn approved() -> Walk {
    let mut walk = Walk::new();
    let exchange = sent(&mut walk, |message| matches!(message, Sent::ApprovalAsk(_)));
    // v19 ApprovalAskAnswer::Asked: tag, secret length and bytes, code, expiry.
    let mut asked = vec![0];
    asked.extend_from_slice(&32_u32.to_be_bytes());
    asked.extend_from_slice(&[7; 32]);
    asked.extend_from_slice(b"BCDFGHJK");
    asked.extend_from_slice(&300_000_000_000_u64.to_be_bytes());
    walk.reports(vec![Report::Heard(jig_client::Event::Reply { exchange, message: asked.into() })]);
    walk.reports(vec![Report::Opened { opened: true }]);
    walk.mark();
    let exchange = sent(&mut walk, |message| matches!(message, Sent::SignInApproval(_)));
    // ApprovalSignInAnswer::Waiting, then its shipped one-second poll.
    walk.reports(vec![Report::Heard(jig_client::Event::Reply { exchange, message: Box::new([1]) })]);
    walk.now = 2_000_000_000;
    walk.reports(vec![Report::Due]);
    let exchange = sent(&mut walk, |message| matches!(message, Sent::SignInApproval(_)));
    walk.reports(vec![Report::Heard(jig_client::Event::Reply { exchange, message: signed_in() })]);
    walk.reports(vec![Report::Kept { written: true }]);
    walk.mark();
    walk.key(Key::Char('q'));
    walk
}

#[test]
fn approval_reports_replay_to_the_same_frames_and_records() {
    same_frames(approved());
}

#[test]
fn browse_asked_or_recorded_alone_is_stale() {
    let mut walk = approved();
    let original = lines(&walk.save()).expect("records");
    let at = original.iter().position(|line| matches!(line, Line::Browse(_))).expect("browse");
    let mut missing = original.clone();
    let removed = missing.remove(at);
    let stale = replay_stale(&written(&missing));
    assert_eq!(stale.line, u32::try_from(at + 1).expect("line"));
    assert_eq!(stale.recorded, line_text(&missing[at]));
    assert_eq!(stale.asked, line_text(&removed));
    let mut extra = original.clone();
    extra.insert(at + 1, removed.clone());
    let stale = replay_stale(&written(&extra));
    assert_eq!(stale.line, u32::try_from(at + 2).expect("line"));
    assert_eq!(stale.recorded, line_text(&removed));
    assert_eq!(stale.asked.as_ref(), b"nothing");
}

fn line_text(line: &Line) -> Box<[u8]> {
    let mut bytes = Vec::new();
    line.write(&mut bytes);
    bytes.pop();
    bytes.into()
}

#[test]
fn duties_keep_the_seams_order_and_fields() {
    // Each native file duty in this fixture carries one record.
    const RECORDS: u32 = 1;
    let mut saved = skein_lib::List::with_capacity(RECORDS);
    saved
        .push(jig_client::Record { key: b"key".as_slice().into(), bytes: b"value".as_slice().into() })
        .expect("one saved record");
    let mut erased = skein_lib::List::with_capacity(RECORDS);
    erased.push(b"old".as_slice().into()).expect("one erased record");
    let duties = Duties {
        lend: None,
        unlend: false,
        stop: None,
        local: true,
        note: Some(temper_client_tui::Note::SignedOut),
        asked: vec![
            Request::Exchange {
                exchange: Token::new(11),
                message: Message::ApprovalAsk(b"ask".as_slice().into()),
                credential: Some(credential()),
            },
            Request::Save { records: saved },
            Request::Erase { records: erased },
            Request::OpenStream { credential: None },
            Request::CloseStream,
        ],
        keep: Some(Some(credential())),
        address: Some(b"signed".as_slice().into()),
        open: Some(b"browse".as_slice().into()),
        close: true,
    };
    let expected = vec![
        Line::Send {
            exchange: 11,
            message: Sent::ApprovalAsk(b"ask".as_slice().into()),
            credential: Some(credential().into_exposed()),
        },
        Line::Save { key: b"key".as_slice().into(), bytes: b"value".as_slice().into() },
        Line::Erase(b"old".as_slice().into()),
        Line::Stream(None),
        Line::Unstream,
        Line::Keep(Some(credential().into_exposed())),
        Line::Note(temper_client_tui::Note::SignedOut),
        Line::Signed(b"signed".as_slice().into()),
        Line::Read,
        Line::Browse(b"browse".as_slice().into()),
        Line::Quit,
    ];
    assert!(record::asked(&duties) == expected, "requests, keep, signed, browse, quit retain their fields and order");
}

fn local_walk() -> Walk {
    linked_walk(false)
}

fn linked_walk(front: bool) -> Walk {
    linked_walk_duties(front).0
}

fn linked_walk_duties(front: bool) -> (Walk, Duties) {
    let mut facts = facts();
    facts.local = true;
    facts.front = front;
    facts.note = Some(temper_client_tui::Note::Until(Wall::from_nanos(1)));
    let mut walk = Walk::with(&facts);
    let link = format!("http://127.0.0.1:8367/jig/v1/sign-in/link?secret={}", "A".repeat(43));
    let duties = walk.reports(vec![Report::Local { link: Some(Secret::new(link.into_bytes().into())) }]);
    let exchange = duties
        .iter()
        .flat_map(|duty| &duty.asked)
        .find_map(|request| {
            if let Request::Exchange { exchange, message: Message::SignInLink(_), .. } = request {
                Some(*exchange)
            } else {
                None
            }
        })
        .expect("engine link exchange");
    walk.reports(vec![Report::Heard(jig_client::Event::Reply { exchange, message: signed_in() })]);
    let duties = walk.reports(vec![Report::Heard(jig_client::Event::Streamed {
        message: jig_client::Streamed::Record(crate::host::tests::hello_bytes()),
    })]);
    let mut duties = duties.into_iter().next().expect("hello's duties");
    let next = walk.reports(vec![Report::Heard(jig_client::Event::Streamed {
        message: jig_client::Streamed::Record(crate::host::tests::me_bytes(&duties)),
    })]);
    duties.asked.extend(next.into_iter().flat_map(|duties| duties.asked));
    walk.mark();
    (walk, duties)
}

#[test]
fn local_reports_replay_and_compare_the_new_duties() {
    let mut walk = local_walk();
    let original = lines(&walk.save()).expect("local records");
    for kind in ["local", "noted", "found", "read", "note"] {
        assert!(original.iter().any(|line| line.kind() == kind), "missing {kind}");
    }
    for (kind, replacement) in
        [("read", Line::Browse(b"changed".as_slice().into())), ("note", Line::Note(temper_client_tui::Note::SignedOut))]
    {
        let mut records = original.clone();
        let at = records.iter().position(|line| line.kind() == kind).expect("local duty");
        records[at] = replacement;
        assert_eq!(records.iter().zip(&original).filter(|(a, b)| a != b).count(), 1);
        let stale = replay_stale(&written(&records));
        assert_eq!(stale.line, u32::try_from(at + 1).expect("line"), "changed {kind}");
        let mut expected = Vec::new();
        original[at].write(&mut expected);
        expected.pop();
        assert_eq!(stale.asked.as_ref(), expected);
    }
    same_frames(walk);
}

#[test]
fn local_records_are_refused_outside_their_places() {
    use temper_client_tui::Note;
    let head = vec![
        Line::Journey { version: 2 },
        Line::Seed(7),
        Line::Clocks { now: Time::ZERO, wall: Wall::EPOCH },
        Line::Screen { size: Size { rows: 12, columns: 48 }, scheme: Some(Scheme::Dark) },
        Line::Address(b"127.0.0.1:8367".as_slice().into()),
    ];
    let tail = vec![
        Line::Turn { now: Time::ZERO, wall: Wall::EPOCH },
        Line::Start,
        Line::Keep(None),
        Line::Note(Note::SignedOut),
        Line::Signed(b"address".as_slice().into()),
        Line::Read,
        Line::Browse(b"address".as_slice().into()),
        Line::Deadline(None),
        Line::End(None),
    ];
    let canonical =
        [head, vec![Line::Local, Line::Credential(Box::new([])), Line::Noted(Note::SignedOut)], tail].concat();
    assert!(Replay::new(&written(&canonical)).is_ok(), "ordered optional facts and duties");
    let rows = [
        ("background after address", 5, Line::Background(Rgb { red: 0, green: 43, blue: 54 })),
        ("background among reports", 10, Line::Background(Rgb { red: 0, green: 43, blue: 54 })),
        ("box before local", 5, Line::Box),
        ("box among reports", 10, Line::Box),
        ("local before address", 4, Line::Local),
        ("local after credential", 7, Line::Local),
        ("noted before credential", 6, Line::Noted(Note::SignedOut)),
        ("local among reports", 10, Line::Local),
        ("noted among duties", 12, Line::Noted(Note::SignedOut)),
        ("front among reports", 10, Line::Front),
        ("box-line among facts", 5, Line::BoxLine(b"log recent".as_slice().into())),
        ("box-stage among facts", 5, Line::BoxStage(b"finding".as_slice().into())),
        ("stop-box among facts", 5, Line::StopBox),
        ("found among facts", 5, Line::Found(None)),
        ("found outside turn", canonical.len() - 1, Line::Found(None)),
        ("note before keep", 10, Line::Note(Note::SignedOut)),
        ("note after signed", 13, Line::Note(Note::SignedOut)),
        ("read before signed", 12, Line::Read),
        ("read after browse", 14, Line::Read),
        ("note outside turn", canonical.len() - 1, Line::Note(Note::SignedOut)),
        ("read outside turn", canonical.len() - 1, Line::Read),
    ];
    let mut failures = Vec::new();
    for (name, at, line) in rows {
        let mut records = canonical.clone();
        if matches!(line, Line::Note(_) | Line::Read) {
            let original = records.iter().position(|record| record.kind() == line.kind()).expect("duty to move");
            records.remove(original);
        }
        records.insert(at.min(records.len() - 1), line);
        if Replay::new(&written(&records)).is_ok() {
            failures.push(name);
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    for bytes in [b"unknown field".as_slice(), b"log ", b""] {
        let mut records = canonical.clone();
        records.insert(10, Line::BoxLine(bytes.into()));
        let Err(error) = Replay::new(&written(&records)) else { panic!("unreadable box line accepted") };
        assert_eq!(error, Unreadable { line: 11, why: "unreadable box line" });
    }
    for bytes in [b"unknown".as_slice(), b"ended", b"fetching x -"] {
        let mut records = canonical.clone();
        records.insert(10, Line::BoxStage(bytes.into()));
        let Err(error) = Replay::new(&written(&records)) else { panic!("unreadable box stage accepted") };
        assert_eq!(error, Unreadable { line: 11, why: "unreadable box stage" });
    }
}

#[test]
fn front_reports_replay_through_box_line_read() {
    use crate::host::tests::{delivery_report, home_subject, home_tasks, launchpad_me, watch};
    use jig_client_protocol_wire::v19 as w;
    use temper_client_tui::{BoxEnd, BoxLine, BoxStage};
    let (mut walk, duties) = linked_walk_duties(true);
    let stages =
        [BoxStage::Finding, BoxStage::Fetching { done: 7, total: Some(10) }, BoxStage::Starting, BoxStage::Running];
    for stage in &stages {
        walk.reports(vec![Report::Stage(stage.clone())]);
    }
    let duties = walk.admit(duties);
    let me = watch(&duties, &w::Subject::Me);
    let next = walk.reports(vec![delivery_report(me, &w::DeliveryValue::Me(launchpad_me(true)))]);
    let duties = walk.admit(next.into_iter().next().expect("me duties"));
    let tasks = watch(&duties, &home_subject(&duties));
    walk.reports(vec![delivery_report(tasks, &w::DeliveryValue::Tasks(home_tasks(true)))]);
    for bytes in [b"restarting forge".as_slice(), b"log recent line"] {
        walk.reports(vec![Report::Front(BoxLine::read(bytes).expect("box line"))]);
    }
    walk.mark();
    let duties = walk.key(Key::Control(b'c'));
    assert!(!duties.iter().any(|duties| duties.close || duties.stop.is_some()), "work running asks before quitting");
    assert!(
        walk.host.frame().text().windows(b"Stop the box?".len()).any(|bytes| bytes == b"Stop the box?"),
        "the quit question stands"
    );
    let first = walk.key(Key::Control(b'c'));
    assert!(first.iter().any(|duties| duties.stop == Some(crate::host::Stop::First)));
    let second = walk.key(Key::Control(b'c'));
    assert!(second.iter().any(|duties| duties.stop == Some(crate::host::Stop::Second)));
    let ended = BoxStage::Ended { end: BoxEnd::Killed { signal: 15 } };
    walk.reports(vec![Report::Stage(ended.clone())]);
    let records = lines(&walk.save()).expect("front records");
    let recorded: Vec<_> = records
        .iter()
        .filter_map(|line| if let Line::BoxStage(bytes) = line { Some(bytes.as_ref()) } else { None })
        .collect();
    let expected: Vec<_> = stages.iter().chain(core::iter::once(&ended)).map(BoxStage::print).collect();
    assert_eq!(recorded, expected.iter().map(AsRef::as_ref).collect::<Vec<_>>());
    assert!(records.contains(&Line::StopBox));
    assert!(records.contains(&Line::StopBoxAgain));
    for (recorded, changed) in [(Line::StopBox, Line::StopBoxAgain), (Line::StopBoxAgain, Line::Quit)] {
        let mut changed_records = records.clone();
        let at = changed_records.iter().position(|line| *line == recorded).expect("stop duty");
        changed_records[at] = changed;
        let stale = replay_stale(&written(&changed_records));
        let mut expected = Vec::new();
        recorded.write(&mut expected);
        expected.pop();
        assert_eq!(stale.asked.as_ref(), expected);
    }
    assert!(records.contains(&Line::Front), "front start fact");
    assert_eq!(records.iter().filter(|line| matches!(line, Line::BoxLine(_))).count(), 2);
    same_frames(walk);
}

#[test]
fn lending_reports_and_duties_replay_through_the_host() {
    use crate::host::tests::{delivery_report, terminal_document, watch};
    use jig_client::contract::LoginExit;
    use jig_client_protocol_wire::v19 as w;
    let (mut walk, duties) = linked_walk_duties(true);
    let duties = walk.admit(duties);
    let subject = w::Subject::Terminal(
        w::SubjectTerminal::new(&w::CEILINGS, w::SubjectTerminalParts { terminal: skein_lib::Rng::new(7).next_u64() })
            .unwrap(),
    );
    let terminal = watch(&duties, &subject);
    let duties = walk.reports(vec![delivery_report(terminal, &terminal_document(Some((1, false))))]);
    assert!(duties.iter().any(|duties| duties.lend.as_ref().is_some_and(|lend| lend.number == 1)));
    walk.mark();
    let duties = walk.reports(vec![delivery_report(terminal, &terminal_document(Some((1, true))))]);
    assert!(duties.iter().any(|duties| duties.unlend));
    let duties = walk.reports(vec![Report::Lent { number: 1, exit: LoginExit::Killed { signal: 15 } }]);
    walk.admit(duties.into_iter().next().unwrap());
    walk.mark();
    let records = lines(&walk.save()).unwrap();
    assert!(records.contains(&Line::Lend { number: 1, name: b"work".as_slice().into() }));
    assert!(records.contains(&Line::Unlend));
    assert!(records.contains(&Line::Lent { number: 1, exit: LoginExit::Killed { signal: 15 } }));
    same_frames(walk);
}

impl Walk {
    fn admit(&mut self, mut duties: Duties) -> Duties {
        let mut pending = std::collections::VecDeque::from(core::mem::take(&mut duties.asked));
        while let Some(request) = pending.pop_front() {
            if let Some(report) = crate::host::tests::background_reply(&request) {
                for next in self.reports(vec![report]) {
                    pending.extend(next.asked);
                }
            }
            duties.asked.push(request);
        }
        duties
    }
}
