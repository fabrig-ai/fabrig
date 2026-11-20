use super::*;
use jig_client_shell_web::adapter::Heard;
use skein_ui::Act;

mod fixed;
mod pieces;
mod saving;

#[test]
fn every_record_reads_back_as_written() {
    let bytes = || Box::from(b"bytes".as_slice());
    let headers = || vec![(Box::from(b"name".as_slice()), Box::from(b"value".as_slice()))];
    let event = || Box::from(b"Event \"quoted\"\\\n".as_slice());
    let name = || Box::from(b"Name".as_slice());
    let rows = [
        Line::Journey { version: 2 },
        Line::Seed(u64::MAX),
        Line::Clocks { now: Time::ZERO, wall: Wall::EPOCH },
        Line::Fixed,
        Line::Primary(Primary::Control),
        Line::Primary(Primary::Meta),
        Line::Zone(120),
        Line::Zone(-300),
        Line::Zone(i32::MIN),
        Line::Zone(i32::MAX),
        Line::Ways { content: bytes(), origin: bytes() },
        Line::Forge(bytes()),
        Line::Path(bytes()),
        Line::Secret(bytes()),
        Line::Stored { key: bytes(), value: bytes() },
        Line::Turn { now: Time::from_nanos(u64::MAX), wall: Wall::from_nanos(u64::MAX) },
        Line::Start,
        Line::Due,
        Line::TabOpen(bytes()),
        Line::TabPoint(bytes()),
        Line::TabClose,
        Line::TabRefused,
        Line::Place(bytes()),
        Line::Heard(Heard::Answered { exchange: u32::MAX, status: 200, body: bytes(), headers: Vec::new() }),
        Line::Heard(Heard::Failed { exchange: 2 }),
        Line::Heard(Heard::StreamOpened),
        Line::Heard(Heard::StreamRefused { status: 403 }),
        Line::Heard(Heard::Piece { bytes: bytes() }),
        Line::Heard(Heard::StreamEnded),
        Line::Carried { writes: 2 },
        Line::Written(bytes()),
        Line::Unwritten(bytes()),
        Line::Lost(3),
        Line::Act { event: event(), name: name(), act: Act::Press },
        Line::Act { event: event(), name: name(), act: Act::Submit },
        Line::Act { event: event(), name: name(), act: Act::Toggle },
        Line::Act { event: event(), name: name(), act: Act::Input { value: b"ab".as_slice().into() } },
        Line::Act { event: event(), name: name(), act: Act::Drawing { data: bytes() } },
        Line::Exchange {
            id: 4,
            bound: 100,
            pieces: None,
            method: b"POST".as_slice().into(),
            path: bytes(),
            body: bytes(),
            headers: headers(),
        },
        Line::Open { piece: 100, path: bytes(), headers: headers() },
        Line::Close,
        Line::Save { key: bytes(), value: bytes() },
        Line::Erase(bytes()),
        Line::Writes(2),
        Line::Navigate(bytes()),
        Line::Replace(bytes()),
        Line::Show { path: b"/accounts".as_slice().into(), replace: false },
        Line::Show { path: bytes(), replace: true },
        Line::Deadline(None),
        Line::Deadline(Some(Time::ZERO)),
        Line::Mark(Some(bytes())),
        Line::Mark(None),
        Line::End(Some(bytes())),
        Line::End(None),
    ];
    for line in rows {
        let mut out = Vec::new();
        line.write(&mut out);
        assert_eq!(Line::read(&out[..out.len() - 1]), Ok(line.clone()));
        if matches!(&line, Line::Act { act: Act::Input { .. }, .. }) {
            let mut expected = br#"act input "ab" "Event \"quoted\"\\\x0a" "Name""#.to_vec();
            expected.push(b'\n');
            assert_eq!(out, expected);
        }
        if matches!(&line, Line::Show { replace: false, .. }) {
            assert_eq!(out, b"show \"/accounts\" push\n");
        }
        if line == Line::Deadline(None) {
            assert_eq!(out, b"deadline none\n");
        }
    }
}
#[test]
fn bytes_outside_printable_ascii_are_escaped() {
    let line =
        Line::Path("a\"b\\c\0\u{1f}\u{7f}".as_bytes().iter().copied().chain([0xff]).chain(" é".bytes()).collect());
    let mut out = Vec::new();
    line.write(&mut out);
    assert_eq!(out, b"path \"a\\\"b\\\\c\\x00\\x1f\\x7f\\xff \\xc3\\xa9\"\n");
    assert_eq!(Line::read(&out[..out.len() - 1]), Ok(line));
}
#[test]
fn a_malformed_line_is_unreadable() {
    for bad in [
        "unknown",
        "start extra",
        "start ",
        "path \"open",
        "path \"\\x4\"",
        "path \"\\q\"",
        "zone +120",
        "zone -0",
        "zone 0120",
        "zone 2147483648",
        "zone -2147483649",
        "seed -1",
        "seed 01",
        "",
        "seed",
        "seed +1",
        "seed 18446744073709551616",
        "heard unknown",
        "primary other",
        "show \"/\" other",
        "act alien \"event\" \"Name\"",
        "act press \"event\"",
        "exchange 1 1 OTHER \"/\" \"\"",
        "open 1 \"/\" \"name\"",
        "path \"\\xAF\"",
        "path \"\\x41\"",
        "path \"a\"extra",
        "start  extra",
        "seed\t1",
        "path \"é\"",
        "path \"\r\"",
    ] {
        let mut recording = b"journey 2\n".to_vec();
        recording.extend_from_slice(bad.as_bytes());
        recording.push(b'\n');
        let error = lines(&recording).expect_err(bad);
        assert_eq!(error.line, 2, "{bad}");
        assert_ne!(error.why, "");
        assert_eq!(Line::read(bad.as_bytes()).expect_err(bad).line, 1);
    }
    assert_eq!(lines(b"journey 2\nseed 1").expect_err("no final newline").line, 2);
    assert_eq!(lines(b"").expect_err("empty recording").line, 1);
}

pub(super) fn facts() -> Facts {
    Facts {
        fixed: false,
        seed: 11,
        now: Time::ZERO,
        wall: Wall::EPOCH,
        primary: Primary::Control,
        zone_minutes: 0,
        ways: None,
        forges: Vec::new(),
        path: b"/accounts/connect".as_slice().into(),
        secret: None,
        stored: Vec::new(),
    }
}
fn written(records: &[Line]) -> Box<[u8]> {
    let mut out = Vec::new();
    for record in records {
        record.write(&mut out);
    }
    out.into()
}
// Use the real start's facts, then edit records without replay: old duties may be stale.
fn recording(records: &[Line]) -> Box<[u8]> {
    let facts = facts();
    let (_, page) = facts.start();
    let mut lines = lines(&Recorder::new(&facts).save(&page)).expect("facts");
    lines.pop();
    lines.extend(records.iter().cloned());
    lines.push(Line::End(None));
    written(&lines)
}
fn turn(millis: u64) -> Line {
    Line::Turn { now: Time::from_nanos(millis * 1_000_000), wall: Wall::EPOCH }
}
fn act(act: Act) -> Line {
    Line::Act { act, event: b"Event".as_slice().into(), name: b"Name".as_slice().into() }
}
#[test]
fn a_saves_turn_keeps_nothing() {
    let facts = facts();
    let (mut host, mut page) = facts.start();
    let mut recorder = Recorder::new(&facts);
    recorder.turn(facts.now, facts.wall);
    recorder.report(&mut host, &page, crate::host::Report::Start);
    host.draw(&mut page, &mut Vec::new());
    recorder.deadline(host.deadline());
    let first = recorder.save(&page);
    recorder.turn(Time::from_nanos(1), facts.wall);
    assert_eq!(recorder.save(&page), first);
    recorder.deadline(host.deadline());
    assert_eq!(recorder.save(&page), first);
    let now = Time::from_nanos(2);
    host.clocks(now, facts.wall);
    recorder.turn(now, facts.wall);
    recorder.report(&mut host, &page, crate::host::Report::Due);
    host.draw(&mut page, &mut Vec::new());
    recorder.deadline(host.deadline());
    let later = lines(&recorder.save(&page)).expect("later records");
    let mut expected = lines(&first).expect("first records");
    expected.pop();
    expected.extend([
        Line::Turn { now, wall: facts.wall },
        Line::Due,
        Line::Deadline(host.deadline()),
        Line::End(Some(page.html().into_bytes().into())),
    ]);
    assert_eq!(later, expected);
    prove(&recorder.save(&page)).expect("save's empty turn needs no replay");
}
#[test]
fn a_kept_pages_return_is_recorded_and_replayed_as_its_own_report() {
    let mut facts = facts();
    facts.path = b"/accounts".as_slice().into();
    let (mut host, mut page) = facts.start();
    let mut recorder = Recorder::new(&facts);
    recorder.turn(facts.now, facts.wall);
    recorder.report(&mut host, &page, crate::host::Report::Start);
    host.draw(&mut page, &mut Vec::new());
    recorder.deadline(host.deadline());
    recorder.turn(facts.now, facts.wall);
    let duties = recorder.report(&mut host, &page, crate::host::Report::Returned { path: facts.path.clone() });
    assert_eq!(duties.show, None);
    host.draw(&mut page, &mut Vec::new());
    recorder.deadline(host.deadline());
    let saved = recorder.save(&page);
    let records = lines(&saved).expect("return records");
    assert!(records.contains(&Line::Returned(facts.path)));
    prove(&saved).expect("return replays and draws the recorded page");
}
#[test]
fn a_page_parts_report_keeps_no_print_mark() {
    let facts = facts();
    let (_, page) = facts.start();
    let mut recorder = Recorder::new(&facts);
    recorder.turn(facts.now, facts.wall);
    let act = Act::Near { edge: skein_ui::LogEdge::Head };
    recorder.act(&page, &act, b"Event".as_slice().into(), b"Name".as_slice().into());
    recorder.deadline(None);
    let mut expected = lines(&Recorder::new(&facts).save(&page)).expect("facts");
    let end = expected.pop().expect("end mark");
    expected.extend([turn(0), self::act(act), Line::Deadline(None), end]);
    assert_eq!(lines(&recorder.save(&page)).expect("page report"), expected);
}
#[test]
fn a_marks_lead_is_its_next_act() {
    for (act, lead) in [
        (Act::Press, Lead::Press),
        (Act::Submit, Lead::Submit),
        (Act::Toggle, Lead::Toggle),
        (Act::Drawing { data: Box::default() }, Lead::Press),
    ] {
        let recording = recording(&[turn(0), Line::Mark(None), self::act(act), Line::Deadline(None)]);
        let mut replay = Replay::new(&recording).expect("read mark");
        assert_eq!(replay.next().expect("mark").expect("act's mark").lead, lead);
    }
}

#[test]
fn a_journey_keeps_the_shells_zone_and_omits_zero() {
    for zone in [0_i32, 120_i32, -300_i32] {
        let mut facts = facts();
        facts.zone_minutes = zone;
        let (_, page) = facts.start();
        let save = Recorder::new(&facts).save(&page);
        let read = lines(&save).expect("read zone facts");
        assert_eq!(read.iter().any(|line| matches!(line, Line::Zone(value) if *value == zone)), zone != 0);
        let (replayed, _) = replay::facts(&read).expect("replay zone facts");
        assert_eq!(replayed.zone_minutes, zone);
    }
}

#[test]
fn a_changed_tab_open_is_stale_at_its_turn_and_line() {
    use crate::host::{Duties, Report, TabAct};
    let facts = facts();
    let (mut host, mut page) = facts.start();
    let mut recorder = Recorder::new(&facts);
    recorder.turn(facts.now, facts.wall);
    recorder.report(&mut host, &page, Report::Start);
    host.draw(&mut page, &mut Vec::new());
    recorder.deadline(host.deadline());
    recorder.turn(Time::from_nanos(1), facts.wall);
    recorder.report(&mut host, &page, Report::TabRefused);
    host.draw(&mut page, &mut Vec::new());
    recorder.deadline(host.deadline());
    let save = recorder.save(&page);
    prove(&save).expect("a refusal replays with no tab duty");
    let mut records = lines(&save).expect("recorded refusal");
    let at = records.iter().position(|line| *line == Line::TabRefused).expect("refusal kept") + 1;
    let duties =
        Duties { tab: vec![TabAct::Open { words: b"Other opening words".as_slice().into() }], ..Duties::default() };
    records.splice(at..at, record::asked(&duties));
    let mut replay = Replay::new(&written(&records)).expect("changed recording reads");
    let stale = replay.next().err().expect("a tab not asked is stale");
    assert_eq!(stale.turn, 2);
    assert_eq!(stale.line, u32::try_from(at + 1).expect("line fits"));
    assert_eq!(stale.recorded, "tab-open \"Other opening words\"");
    assert_eq!(stale.asked, "nothing");
}

#[test]
fn tab_duties_precede_transport_in_their_order() {
    use crate::host::{Duties, TabAct};
    use jig_client_shell_web::adapter::Asked;
    let duties = Duties {
        tab: vec![
            TabAct::Open { words: b"Opening".as_slice().into() },
            TabAct::Point { to: b"https://issuer.example/sign-in".as_slice().into() },
            TabAct::Close,
        ],
        asked: vec![Asked::CloseStream],
        ..Duties::default()
    };
    assert!(!duties.departs());
    assert_eq!(
        record::asked(&duties),
        vec![
            Line::TabOpen(b"Opening".as_slice().into()),
            Line::TabPoint(b"https://issuer.example/sign-in".as_slice().into()),
            Line::TabClose,
            Line::Close,
        ]
    );
}
