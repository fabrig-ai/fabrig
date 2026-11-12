use super::*;
use crate::host::Report;
use skein_ui_web::store::Record;

const LINK: &[u8] = b"0123456789abcdef-link";
const STORED: &[u8] = b"a-stored-credential";

fn signed_in() -> Facts {
    let mut facts = facts();
    facts.secret = Some(LINK.into());
    facts.stored.push(Record { key: b"other".as_slice().into(), value: STORED.into() });
    facts
}

#[test]
fn a_save_replaces_sign_in_but_keeps_typed_values_and_memory() {
    let facts = signed_in();
    let (_, page) = facts.start();
    let mut recorder = Recorder::new(&facts);
    let input = Act::Input { value: b"a-typed-key".as_slice().into() };
    recorder.turn(facts.now, facts.wall);
    recorder.act(&page, &input, b"Key { value: Secret { len: 11 } }".as_slice().into(), b"Key".as_slice().into());
    recorder.deadline(None);
    let kept = recorder.save(&page);
    let saved = saved(&kept);
    let mut expected = lines(&kept).expect("kept lines");
    for line in &mut expected {
        if let Line::Secret(value) = line {
            *value = vec![b'x'; 21].into();
        }
        if let Line::Stored { value, .. } = line {
            *value = vec![b'x'; STORED.len()].into();
        }
    }
    assert_eq!(lines(&saved).expect("saved lines"), expected, "every other line stands, including the typed act");
    assert!(skein_lib::bytes::find(&saved, LINK).is_none());
    assert!(skein_lib::bytes::find(&saved, STORED).is_none());
    let added = redact(&saved, &[]).expect("saved file can be added");
    assert_eq!(added.originals.count(Kind::Link), 0);
    assert_eq!(added.originals.count(Kind::Stored), 0);
    assert_eq!(added.originals.count(Kind::Typed), 1);
    assert_eq!(recorder.save(&page), kept, "save redaction does not change memory");
    assert_eq!(recorder.end(&page), kept, "end still gives originals");
}

#[test]
fn a_saved_link_and_stored_start_still_proves() {
    let facts = signed_in();
    let (mut host, mut page) = facts.start();
    let mut recorder = Recorder::new(&facts);
    recorder.turn(facts.now, facts.wall);
    recorder.report(&mut host, &page, Report::Start);
    host.draw(&mut page, &mut Vec::new());
    recorder.deadline(host.deadline());
    let kept = recorder.save(&page);
    prove(&kept).expect("kept start proves");
    let output = saved(&kept);
    prove(&output).expect("saved start proves");
    assert!(output != kept, "the sign-in was replaced");
}

#[test]
fn an_undecoded_save_hides_every_piece_but_keeps_its_length() {
    let facts = signed_in();
    let (_, page) = facts.start();
    let mut records = lines(&Recorder::new(&facts).save(&page)).expect("facts");
    let end = records.pop().expect("end");
    let extra = [
        Line::Place([LINK, b"/", STORED].concat().into()),
        Line::Heard(Heard::Piece { bytes: b"data: b2s=\n\n".as_slice().into() }),
        Line::Heard(Heard::StreamOpened),
        Line::Heard(Heard::Piece { bytes: b"data: not canonical!\n\n".as_slice().into() }),
        Line::Heard(Heard::Piece { bytes: [LINK, STORED].concat().into() }),
    ];
    records.extend(extra);
    records.push(end);
    assert!(Streams::read(&records).is_err(), "the fallback is exercised");
    let saved = saved(&written(&records));
    for line in &mut records {
        for (field, _) in fields_mut(line) {
            for original in [LINK, STORED] {
                while let Some(at) = skein_lib::bytes::find(field, original) {
                    field[at..at + original.len()].fill(b'x');
                }
            }
        }
        if let Line::Heard(Heard::Piece { bytes }) = line {
            bytes.fill(b'x');
        }
    }
    assert_eq!(lines(&saved).expect("saved lines"), records);
}

fn prefix(recording: &[u8]) -> Vec<Line> {
    let mut records = lines(recording).expect("recording lines");
    assert!(matches!(records.pop(), Some(Line::End(_))));
    records
}
fn size(records: &[Line]) -> u64 {
    u64::try_from(written(records).len()).expect("fixture size")
}

#[test]
fn a_report_and_its_turn_and_asks_fit_together_or_end_the_recording() {
    let facts = facts();
    let (mut host, page) = facts.start();
    let mut unbounded = Recorder::new(&facts);
    unbounded.turn(facts.now, facts.wall);
    let expected_duties = record::asked(&unbounded.report(&mut host, &page, Report::Start));
    assert!(!expected_duties.is_empty(), "the report asks for transport");
    let fitting = prefix(&unbounded.save(&page));
    let bound = size(&fitting);
    for bytes in [bound - 1, bound] {
        let (mut host, mut page) = facts.start();
        let mut recorder = Recorder::new(&facts).with_bound(bytes);
        recorder.turn(facts.now, facts.wall);
        let duties = recorder.report(&mut host, &page, Report::Start);
        assert_eq!(record::asked(&duties), expected_duties, "even the overflowing report reaches the host");
        let file = recorder.save(&page);
        if bytes == bound {
            assert_eq!(prefix(&file), fitting, "the exact bound admits the whole call");
        } else {
            assert_eq!(file, Recorder::new(&facts).save(&page), "no turn, report or ask is kept on overflow");
            prove(&file).expect("the ended recording proves");
            host.draw(&mut page, &mut Vec::new());
            assert_ne!(page.html().as_bytes(), b"", "the live page still draws");
            recorder.deadline(host.deadline());
            recorder.turn(Time::from_nanos(1), facts.wall);
            recorder.mark(&page);
            let (mut comparison, mut other_page) = facts.start();
            comparison.report(Report::Start);
            comparison.draw(&mut other_page, &mut Vec::new());
            let report = || Report::Place { path: b"/?kind=reply&project=3".as_slice().into() };
            let later = recorder.report(&mut host, &page, report());
            assert_eq!(later.show, Some((Box::from(b"/?project=3".as_slice()), true)), "the live host returns duties");
            assert_eq!(record::asked(&later), record::asked(&comparison.report(report())));
            host.draw(&mut page, &mut Vec::new());
            comparison.draw(&mut other_page, &mut Vec::new());
            assert_eq!(page.html(), other_page.html(), "later reports reach the live host");
            recorder.deadline(Some(Time::from_nanos(9)));
            assert_eq!(recorder.save(&page), file, "later drawing, reports, marks and deadlines cannot grow it");
            assert_eq!(recorder.end(&page), file, "no second end, even with a changed page");
        }
    }
}

#[test]
fn reports_keep_the_prefix_until_the_next_would_pass_the_bound() {
    let facts = facts();
    let (mut host, mut page) = facts.start();
    let mut recorder = Recorder::new(&facts);
    recorder.turn(facts.now, facts.wall);
    recorder.report(&mut host, &page, Report::Start);
    host.draw(&mut page, &mut Vec::new());
    recorder.deadline(host.deadline());
    let initial = prefix(&recorder.save(&page));
    // Enough for one more turn and Due, but not two in that turn.
    let mut fitting = initial.clone();
    fitting.extend([turn(1), Line::Due]);
    recorder = recorder.with_bound(size(&fitting));
    let now = Time::from_nanos(1_000_000);
    host.clocks(now, facts.wall);
    recorder.turn(now, facts.wall);
    recorder.report(&mut host, &page, Report::Due);
    assert_eq!(prefix(&recorder.save(&page)), fitting);
    recorder.report(&mut host, &page, Report::Due);
    let ended = recorder.save(&page);
    assert_eq!(prefix(&ended), fitting, "only the second report overflows");
    prove(&ended).expect("the kept prefix ends with the page as drawn");
    recorder.deadline(host.deadline());
    assert_eq!(recorder.end(&page), ended);
}

#[test]
fn a_mark_and_an_acts_mark_are_admitted_as_whole_calls() {
    let facts = facts();
    let (_, page) = facts.start();
    for is_act in [false, true] {
        let mut recorder = Recorder::new(&facts);
        let baseline = recorder.save(&page);
        let give = |recorder: &mut Recorder| {
            recorder.turn(facts.now, facts.wall);
            if is_act {
                recorder.act(&page, &Act::Press, b"Event".as_slice().into(), b"Name".as_slice().into());
            } else {
                recorder.mark(&page);
            }
        };
        give(&mut recorder);
        let fitting = recorder.save(&page);
        let bound = size(&prefix(&fitting));
        for bytes in [bound - 1, bound] {
            let mut recorder = Recorder::new(&facts).with_bound(bytes);
            give(&mut recorder);
            assert_eq!(recorder.save(&page), if bytes == bound { &fitting } else { &baseline }.clone());
        }
    }
}

#[test]
fn the_start_is_whole_even_when_it_is_larger_than_the_bound() {
    let facts = signed_in();
    let (mut host, page) = facts.start();
    let mut recorder = Recorder::new(&facts).with_bound(0);
    let whole = Recorder::new(&facts).save(&page);
    assert_eq!(recorder.save(&page), whole, "the bound does not cut facts");
    recorder.turn(facts.now, facts.wall);
    recorder.report(&mut host, &page, Report::Start);
    assert_eq!(recorder.save(&page), whole, "the first report ends without cutting facts");
    prove(&recorder.end(&page)).expect("facts with an end prove");
    assert_eq!(crate::limits::RECORDING_BYTES, 67_108_864);
}
