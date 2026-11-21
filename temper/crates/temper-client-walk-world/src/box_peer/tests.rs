//! Pipe requests and the single owned script (client/journey-walks.md, 7).
use super::{Answers, LINE_BYTES, OWNED};
use skein_lib::{Duration, Time};

#[test]
fn starts_are_owned_timed_single_and_idempotent() {
    let now = Time::from_nanos(1_000_000_000);
    let mut answers = Answers::default();
    assert_eq!(answers.ask(b"start 9", false, false, now), None);
    assert!(answers.deadline().is_none(), "the fake group's start stays open");
    assert_eq!(answers.ask(b"start 2", true, false, now), None);
    assert_eq!(answers.deadline(), Some(now));
    assert_eq!(answers.ask(b"start 2", true, false, now), None, "same forge shares its script");
    assert_eq!(answers.ask(b"start 3", true, false, now).as_deref(), Some(b"failed 3 busy\n".as_slice()));
    for line in OWNED {
        let due = now.saturating_add(line.after);
        if line.after != Duration::ZERO {
            assert!(answers.due(Time::from_nanos(due.as_nanos() - 1)).is_none(), "not before the line's time");
        }
        let (forge, drawn, _) = answers.due(due).expect("line due");
        assert_eq!(forge, 2);
        assert_eq!(drawn.bytes(forge), line.bytes(2));
    }
    assert_eq!(OWNED.last().expect("handover").after, Duration::from_secs(14));
    assert_eq!(answers.deadline(), None);
    assert_eq!(answers.ask(b"start 2", true, false, now).as_deref(), Some(b"started 2\n".as_slice()));
}

#[test]
fn a_sign_in_adds_the_source_step_before_handover_and_replays_its_result() {
    let began = Time::from_nanos(1_000_000_000);
    let mut answers = Answers::default();
    assert_eq!(answers.ask(b"start 1", true, true, began), None);
    for line in OWNED.iter().take(6) {
        answers.due(began.saturating_add(line.after)).expect("fetch and prepare");
    }
    for (seconds, bytes, handover) in [
        (12, "step 1 run\n", false),
        (14, "step 1 source\n", false),
        (16, "source 1 added\n", false),
        (18, "started 1\n", true),
    ] {
        let at = began.saturating_add(Duration::from_secs(seconds));
        assert_eq!(answers.deadline(), Some(at));
        assert!(answers.due(Time::from_nanos(at.as_nanos() - 1)).is_none());
        let (forge, line, last) = answers.due(at).expect("line due");
        assert_eq!(forge, 1);
        assert_eq!(line.bytes(forge).as_ref(), bytes.as_bytes());
        assert_eq!(last, handover, "handover is only laid at started");
    }
    assert_eq!(answers.deadline(), None);
    assert_eq!(
        answers.ask(b"start 1", true, false, began).as_deref(),
        Some(b"source 1 added\nstarted 1\n".as_slice()),
        "the ended group remembers its source, not the next ask's setup"
    );
    assert_eq!(answers.ask(b"stop 1", false, false, began).as_deref(), Some(b"stopped 1\n".as_slice()));
    assert_eq!(answers.ask(b"start 1", true, false, began), None);
    assert_eq!(answers.deadline(), Some(began), "stop clears the ended source");
}

#[test]
fn stop_and_remove_answer_at_once_and_other_lines_are_ignored() {
    let mut answers = Answers::default();
    for (line, reply) in [(b"stop 4".as_slice(), b"stopped 4\n".as_slice()), (b"remove 4", b"removed 4\n")] {
        assert_eq!(answers.ask(line, false, false, Time::ZERO).as_deref(), Some(reply));
    }
    for line in [b"unknown 4".as_slice(), b"start x", b"start 1 extra", b"start"] {
        assert_eq!(answers.ask(line, true, false, Time::ZERO), None);
    }
    assert!(answers.deadline().is_none());
}

#[test]
fn whole_lines_fit_the_bound_and_overlong_lines_do_not_poison_the_next() {
    let mut answers = Answers::default();
    let max = usize::try_from(LINE_BYTES).expect("bound");
    for _ in 0..max - 1 {
        assert!(answers.byte(b'x').is_none());
    }
    assert_eq!(answers.byte(b'\n').expect("at bound").len(), max - 1);
    for byte in std::iter::repeat_n(b'x', max).chain(b"\nstart 7\n".iter().copied()) {
        if let Some(line) = answers.byte(byte) {
            assert_eq!(line, b"start 7");
            answers.ask(&line, true, false, Time::ZERO);
        }
    }
    assert_eq!(answers.deadline(), Some(Time::ZERO), "the next whole ask is read");
}
