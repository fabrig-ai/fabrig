//! Ordered start facts without constructing a host (client/tui/journeys.md, 3.2).
use super::*;
use crate::journey::tests::fixture;
use crate::journey::{Replay, lines, replay::facts};
use fixture::written;

#[test]
fn setup_and_log_fields_are_strict() {
    for bad in [b"set-up extra\n".as_slice(), b"box-log\n", b"box-log bare\n", b"box-log \"path\" extra\n"] {
        assert!(lines(bad).is_err(), "strict start-fact fields: {bad:?}");
    }
}

#[test]
fn setup_and_log_facts_are_ordered_and_optional() {
    // Journey, seed, clocks, screen and address precede the optional facts.
    const FACTS_AT: usize = 5;
    // With box, front, setup and log present, these lie after front, log and start.
    const AFTER_FRONT: usize = 7;
    const AFTER_LOG: usize = 9;
    const AFTER_START: usize = 11;
    for (set_up, log) in fixture::SETUPS {
        let given = fixture::setup(set_up, log);
        let recorder = Recorder::new(&given);
        let mut records = lines(&recorder.file).expect("start records");
        let mut expected = vec![Line::Box, Line::Front];
        if set_up {
            expected.push(Line::SetUp);
        }
        if !log.is_empty() {
            expected.push(Line::BoxLog(log.into()));
        }
        assert!(
            records.get(FACTS_AT..FACTS_AT + expected.len()) == Some(expected.as_slice()),
            "facts follow address in order"
        );
        let (read, _) = facts(&records).expect("ordered facts");
        assert_eq!(read.set_up, set_up);
        assert_eq!(read.box_log.as_ref(), log);
        // An otherwise well-formed envelope lets the reader reject facts misplaced in the body too.
        records.extend([
            Line::Turn { now: Time::from_nanos(1), wall: Wall::from_nanos(1) },
            Line::Start,
            Line::Read,
            Line::Deadline(None),
            Line::Mark(None),
            Line::End(None),
        ]);
        if set_up && !log.is_empty() {
            for (at, insert) in
                [(AFTER_FRONT, Line::SetUp), (AFTER_LOG, Line::BoxLog(log.into())), (AFTER_START, Line::SetUp)]
            {
                let mut bad = records.clone();
                bad.insert(at, insert);
                assert!(Replay::new(&written(&bad)).is_err(), "repeated or misplaced fact");
            }
            let mut bad = records.clone();
            bad.swap(AFTER_FRONT, AFTER_FRONT + 1);
            assert!(Replay::new(&written(&bad)).is_err(), "box-log cannot precede set-up");
        }
    }
}
