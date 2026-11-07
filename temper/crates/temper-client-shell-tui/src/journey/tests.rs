//! The host's world, with fake bytes and no terminal (temper/docs/design/client/tui/journeys.md, 7).
use super::*;
use crate::host::{Duties, Host, Report};
use jig_client::{Message, Request};
use skein_lib::{Secret, Time, Token, Wall};
use skein_ui_screen::Size;
use skein_ui_screen_terminal::Key;
use temper_client_tui::Scheme;

fn facts() -> Facts {
    Facts {
        seed: 7,
        now: Time::ZERO,
        wall: Wall::EPOCH,
        size: Size { rows: 12, columns: 48 },
        scheme: Scheme::Dark,
        address: b"127.0.0.1:8367".as_slice().into(),
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
        let facts = facts();
        let (host, start) = facts.start();
        let mut walk = Self { host, recorder: Recorder::new(&facts), now: 1, marks: Vec::new() };
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
    let exchange = duties.iter().flat_map(|duty| &duty.asked).find_map(|request| match request {
        Request::Exchange { exchange, message: Message::SignInLink(_), .. } => Some(*exchange),
        _ => None,
    }).expect("pasted link exchange");
    walk.reports(vec![Report::Heard(jig_client::Event::Lost { exchange })]);
    walk.mark();
    walk
}

fn written(records: &[Line]) -> Box<[u8]> {
    let mut bytes = Vec::new();
    for record in records { record.write(&mut bytes); }
    bytes.into()
}

fn replay_stale(recording: &[u8]) -> Stale {
    let mut replay = Replay::new(recording).expect("readable recording");
    loop {
        match replay.next() {
            Err(stale) => return stale,
            Ok(Some(_)) => {},
            Ok(None) => panic!("changed rule was accepted"),
        }
    }
}

#[test]
fn pasted_reports_replay_to_the_same_frames_and_records() {
    let mut walk = pasted();
    let file = walk.save();
    let records = lines(&file).expect("readable records");
    assert_eq!(written(&records), file);
    let mut replay = Replay::new(&file).expect("facts");
    walk.marks.push((walk.host.frame().print(), None));
    for (print, key) in walk.marks {
        let mark = replay.next().expect("same duties").expect("mark");
        assert_eq!(mark.frame.print(), print);
        assert_eq!(mark.live, Some(print));
        assert_eq!(mark.key, key);
    }
    assert!(replay.next().expect("end").is_none());
}

#[test]
fn each_enter_marks_its_frame_before_the_key() {
    let mut walk = pasted();
    let records = lines(&walk.save()).expect("records");
    let mut enters = 0;
    for pair in records.windows(2) {
        if matches!(pair.get(1), Some(Line::Key { key: Key::Enter, .. })) {
            enters += 1;
            assert!(matches!(pair.first(), Some(Line::Mark(Some(_)))), "Enter needs its preceding live mark");
        }
    }
    assert_eq!(enters, 1);
}

#[test]
fn changed_duties_deadlines_and_key_events_are_stale() {
    let mut walk = pasted();
    let original = lines(&walk.save()).expect("records");
    let send = original.iter().position(|line| matches!(line, Line::Send { message: Sent::SignInLink(_), .. })).expect("send");
    let deadline = original.iter().position(|line| matches!(line, Line::Deadline(_))).expect("deadline");
    let key = original.iter().position(|line| matches!(line, Line::Key { key: Key::Enter, .. })).expect("key");
    for (at, replacement) in [
        (send, Line::Send { exchange: 0, message: Sent::SignInLink(b"changed".as_slice().into()), credential: None }),
        (deadline, Line::Deadline(Some(Time::from_nanos(u64::MAX)))),
        (key, Line::Key { key: Key::Enter, event: Some(b"Quit".as_slice().into()) }),
    ] {
        let mut records = original.clone();
        records[at] = replacement.clone();
        let stale = replay_stale(&written(&records));
        assert_eq!(stale.line, u32::try_from(at + 1).expect("line"));
        let mut expected = Vec::new(); replacement.write(&mut expected); expected.pop();
        assert_eq!(stale.recorded.as_ref(), expected);
        assert_ne!(stale.recorded, stale.asked);
        assert!(stale.turn > 0);
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
    assert!(prove(&file).is_ok());
}

#[test]
fn proof_refuses_the_first_differing_print_line() {
    let mut walk = pasted();
    let original = lines(&walk.save()).expect("records");
    for (replacement, expected_line) in [(b"different\n".as_slice(), 1), (b"".as_slice(), 1)] {
        let mut records = original.clone();
        let mut changed = 0;
        for record in &mut records {
            if let Line::Mark(print) = record {
                if changed == 0 { *print = Some(replacement.into()); changed += 1; }
            }
        }
        assert_eq!(changed, 1);
        assert!(matches!(prove(&written(&records)), Err(Refusal::Differs { mark: 1, line }) if line == expected_line));
    }
}

#[test]
fn proof_drops_every_live_print_after_the_whole_walk() {
    let mut walk = pasted();
    let file = walk.save();
    let mut records = lines(&file).expect("records");
    for record in &mut records {
        if let Line::Mark(print) | Line::End(print) = record { *print = None; }
    }
    assert_eq!(prove(&file).expect("faithful prints"), written(&records));
}

#[test]
fn later_saves_extend_a_whole_recording() {
    let mut walk = pasted();
    let first = walk.save();
    walk.key(Key::Char('x'));
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
