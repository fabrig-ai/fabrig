//! Family 42 keeps follow state and obeys configured decode bounds.
use super::fixtures;
use crate::{Undecodable, keys, records};
use temper_engine_domain_forge_vocabulary as d;
fn entry(state: d::FollowState) -> d::FollowEntry {
    d::FollowEntry {
        before: [1; 32],
        after: [2; 32],
        pull: Some(19),
        commits: Box::new([d::FollowCommit { id: [2; 32], message: b"Fix handoff\n\nThe body.".as_slice().into() }]),
        more_commits: 3,
        files: Some(Box::new([b"src/a.c".as_slice().into()])),
        more_files: 2,
        change: Some(51),
        state,
        word: false,
        remake: false,
    }
}
fn row() -> d::FollowRow {
    d::FollowRow {
        project: 5,
        follow: 2,
        source: skein_forge::api::Repository { forge: 1, repository: 2 },
        branch: b"main".as_slice().into(),
        heard: Some([2; 32]),
        task: Some(71),
        stopped: Some(b"owner stopped".as_slice().into()),
        questions: Box::new([]),
        pending: None,
        queue: Box::new([entry(d::FollowState::NotMade)]),
        skips: Box::new([d::FollowSkip { after: [3; 32], reason: b"already done".as_slice().into() }]),
        parity: Some(d::FollowParity {
            source: [1; 32],
            head: [2; 32],
            verdict: d::FollowVerdict::Passed,
            run: b"https://forge.example/run/1".as_slice().into(),
        }),
    }
}
fn round_trip(row: d::FollowRow) {
    let row = d::Stored::Follow(row);
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("bounds");
    let meter = skein_world::domain::heap::Meter::new();
    meter.start();
    let cloned = row.clone();
    let held = meter.end().held();
    assert_eq!(d::stored_bytes(&cloned), Some(held));
    drop(cloned);
    let key = d::stored_key(&row);
    let encoded = keys::encode(&key, u32::MAX).expect("key");
    assert_eq!(encoded.first(), Some(&42));
    assert_eq!(keys::decode(&encoded, &fixtures::LIMITS), Some(key));
    assert_eq!(records::row(&encoded, &records::encode(&row), &bounds), Ok(row));
}
#[test]
fn pending_owner_acts_and_their_keyed_outcomes_survive_storage() {
    use d::follows::FollowAct as A;
    for act in [
        A::Start { after: [2; 32] },
        A::Hold { after: [2; 32], reason: b"waits".as_slice().into() },
        A::Skip { after: [2; 32], reason: b"build only".as_slice().into() },
        A::Merge { afters: Box::new([[2; 32], [3; 32]]) },
    ] {
        let mut row = row();
        row.pending = Some(act);
        round_trip(row);
    }
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).unwrap();
    for refusal in [None, Some(b"entry: change is producing".as_slice().into())] {
        let row = d::Stored::Application {
            token: 31,
            until: skein_lib::Wall::from_nanos(400),
            outcome: d::ApplicationOutcome::Follow(refusal),
        };
        assert_eq!(records::decode(&records::encode(&row), &bounds), Ok(row));
    }
}
#[test]
fn follow_records_round_trip_every_state_parity_and_optional_field() {
    for state in [
        d::FollowState::NotMade,
        d::FollowState::WaitsWord,
        d::FollowState::HeldByOwner(b"not yet".as_slice().into()),
        d::FollowState::WaitsFor(52),
        d::FollowState::Producing,
        d::FollowState::Held(b"check".as_slice().into()),
        d::FollowState::Ended(b"ended".as_slice().into()),
    ] {
        let mut row = row();
        row.queue = Box::new([entry(state)]);
        round_trip(row);
    }
    for verdict in [d::FollowVerdict::Passed, d::FollowVerdict::Failing(3), d::FollowVerdict::Many] {
        let mut row = row();
        row.parity.as_mut().expect("parity").verdict = verdict;
        round_trip(row);
    }
    let mut empty = row();
    empty.heard = None;
    empty.task = None;
    empty.stopped = None;
    empty.parity = None;
    empty.skips = Box::new([]);
    let mut unknown = entry(d::FollowState::NotMade);
    unknown.files = None;
    unknown.pull = None;
    unknown.change = None;
    unknown.commits = Box::new([]);
    empty.queue = Box::new([unknown]);
    round_trip(empty.clone());
    empty.queue = Box::new([]);
    round_trip(empty);
}
#[test]
fn follow_decode_refuses_one_past_each_configured_bound() {
    let row = row();
    for field in 0_u8..8 {
        let mut limits = fixtures::LIMITS;
        limits.follow_queue = 1;
        limits.follow_commits = 1;
        limits.follow_files = 1;
        limits.follow_skips = 1;
        limits.follow_message_bytes = 22;
        limits.follow_reason_bytes = 13;
        limits.name_bytes = 4;
        let exact = crate::bounds(&limits, limits.client.op_bytes).expect("bounds");
        let encoded = records::encode(&d::Stored::Follow(row.clone()));
        assert!(records::decode(&encoded, &exact).is_ok(), "at bound {field}");
        match field {
            0 => limits.follow_queue = 0,
            1 => limits.follow_commits = 0,
            2 => limits.follow_files = 0,
            3 => limits.follow_skips = 0,
            4 => limits.follow_message_bytes = 21,
            5 => limits.follow_reason_bytes = 12,
            6 => limits.name_bytes = 3,
            7 => {
                let mut held = row.clone();
                held.stopped = None;
                held.queue.first_mut().expect("entry").state =
                    d::FollowState::HeldByOwner(b"held by owner".as_slice().into());
                let encoded = records::encode(&d::Stored::Follow(held));
                assert!(records::decode(&encoded, &exact).is_ok(), "owner hold at bound");
                limits.follow_reason_bytes = 12;
                let smaller = crate::bounds(&limits, limits.client.op_bytes).expect("smaller bounds");
                assert_eq!(records::decode(&encoded, &smaller), Err(Undecodable::Malformed));
                continue;
            }
            _ => unreachable!("eight configured fields"),
        }
        let smaller = crate::bounds(&limits, limits.client.op_bytes).expect("smaller bounds");
        assert_eq!(records::decode(&encoded, &smaller), Err(Undecodable::Malformed), "one past {field}");
    }
}

#[test]
fn a_changes_follow_workflow_progress_round_trips_with_its_allocated_bytes() {
    let mut row = fixtures::change_row(temper_engine_domain_forge_change::State::Gating {
        head: [3; 32],
        asked: Box::new([u64::MAX]),
    });
    row.follow_check = Some(d::FollowCheck {
        follow: 1,
        source: [2; 32],
        head: [3; 32],
        definition: b"definition".as_slice().into(),
        key: Some([4; 32]),
        page: 3,
        faults: 2,
        held: Some(b"runner fault".as_slice().into()),
        flight: Some(d::TierFlight {
            entry: 11,
            start: Some(skein_lib::Time::from_nanos(10)),
            since: Some(skein_lib::Time::from_nanos(11)),
            waiting: true,
            next: None,
            run: Some(71),
            fault: None,
            pending: Box::new([skein_forge::api::JobAttempt { head: [3; 32], run: 71, job: 81, attempt: 2 }]),
            report: Some(d::ReceiptRun { id: 71, number: 7, url: b"https://forge.example/run/71".as_slice().into() }),
        }),
    });
    let record = d::Stored::Change(Box::new(row));
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("bounds");
    assert_eq!(records::decode(&records::encode(&record), &bounds), Ok(record.clone()));
    let meter = skein_world::domain::heap::Meter::new();
    meter.start();
    let cloned = record.clone();
    let held = meter.end().held();
    assert_eq!(d::stored_bytes(&cloned), Some(held));
    drop(cloned);
}

#[test]
fn follow_question_relays_round_trip_and_older_rows_have_no_relay() {
    for answer in [None, Some(b"A loop, as C does.".as_slice().into())] {
        let mut row = row();
        row.questions = Box::new([d::follows::FollowQuestion {
            change: 70,
            number: 11,
            words: b"Port vsr_tick?".as_slice().into(),
            person: None,
            relayed: Some(19),
            answer,
        }]);
        round_trip(row);
    }
    let old = super::catch_up::kept_golden(16, "record_follow_question_full.bin");
    let bytes = super::catch_up::patched(16, 5, b"FollowQuestion", old);
    let question = crate::v17::FollowQuestion::decode(&crate::v17::CEILINGS, &mut skein_lib::Reader::new(&bytes))
        .expect("old question");
    assert_eq!(question.relayed(), &None);
    assert_eq!(question.number(), u64::MAX);
}
