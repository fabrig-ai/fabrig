//! Batch storage (domain/landing.md, 12).
use super::fixtures;
use crate::{records, v25};
use skein_codec::Record;
use skein_lib::Reader;
use temper_engine_domain_forge as forge;
use temper_engine_domain_forge_landing as landing;

fn seed() -> forge::LandingRow {
    forge::LandingRow {
        last_event: None,
        turn_since: None,
        waiting: None,
        flake_reruns: Box::new([]),
        room: None,
        job: None,
        landing: landing::Landing {
            task: 61,
            project: 5,
            phase: landing::Phase::Preparing,
            members: Box::new([landing::Member {
                round: Some(1),
                unit: 0,
                branch: b"temper/1/l61".as_slice().into(),
                combined: Some(landing::Combine::Squash),
                recorded: true,
                told: true,
                judging: true,
                leave_told: true,
                left: None,
                change: 51,
                repository: landing::Repository { forge: 1, repository: 2 },
                pull: 7,
                head: Some([1; 32]),
                base: Some([2; 32]),
                candidate: Some([3; 32]),
                gates: Box::new([landing::GateProof {
                    proof: landing::Proof::Passed { key: [3; 32] },
                    posted: landing::Posted::Pending,
                    faults: 2,
                }]),
                asked: Some(1),
                landed: None,
            }]),
            preparation: 1,
            stale_proofs: 3,
            branch: b"temper/g1".as_slice().into(),
            round: landing::Round::new(b"temper/1/l61".as_slice().into()),
            last_left: Some(0),
            searched: None,
        },
        asks: Box::new([None]),
        entries: Box::new([]),
    }
}

fn stored(row: v25::LandingRow) -> forge::Stored {
    let row = v25::Row { value: v25::Value::Landing(Box::new(row)) };
    let bytes = row.encode(&v25::CEILINGS).unwrap();
    records::decode(&bytes, &crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).unwrap()).unwrap()
}

#[test]
fn raised_codec_bounds_read_a_landing_past_the_old_entry_count() {
    let mut limits = fixtures::LIMITS;
    limits.entries = 8;
    let mut bounds = crate::bounds(&limits, limits.client.op_bytes).unwrap();
    let mut row = seed();
    let mut entries = skein_lib::List::with_capacity(9);
    for entry in 1..=9 {
        entries
            .push(forge::LandingEntry {
                entry,
                member: 0,
                gate: 0,
                operation: forge::Operation::Dispatch,
                preparation: Some(1),
                base: None,
                run: None,
            })
            .unwrap();
    }
    row.entries = entries.into_boxed();
    let stored = forge::Stored::Landing(row);
    let bytes = records::encode(&stored);
    assert_eq!(records::decode(&bytes, &bounds), Err(crate::Undecodable::Malformed));
    limits.entries = 16;
    limits.client.entries = limits.client.entries.max(16);
    limits.tasks += 1;
    limits.holds += 1;
    bounds.raise(&limits);
    assert_eq!(records::decode(&bytes, &bounds), Ok(stored));
    assert_eq!(bounds.limits().closer_selections, limits.tasks);
    assert_eq!(bounds.limits().hold_design_writers, limits.holds);
    assert_eq!(bounds.limits().tier_row_goals, limits.holds);
}

#[test]
fn batch_members_and_leave_causes_round_trip() {
    let row = seed();
    for why in [
        None,
        Some(landing::Leave::EarlierConflict),
        Some(landing::Leave::Unit { change: 13 }),
        Some(landing::Leave::Dependency { change: 14 }),
        Some(landing::Leave::Blamed { gate: 1, run: 9, red: [5; 32], green: [4; 32] }),
    ] {
        let mut row = row.clone();
        row.landing.phase = landing::Phase::Preparing;
        let mut members = skein_lib::List::with_capacity(3);
        for index in 0..3 {
            let mut member = row.landing.members[0].clone();
            member.change = index + 10;
            member.unit = u16::try_from(index).unwrap();
            member.branch = skein_lib::bytes::concat(&[b"temper/1/l61-", skein_lib::Decimal::of(index + 1).as_bytes()]);
            member.candidate = Some([u8::try_from(index).unwrap(); 32]);
            member.combined = if index == 1 { Some(landing::Combine::Merge) } else { Some(landing::Combine::Squash) };
            member.recorded = index == 0;
            member.told = index == 1;
            member.judging = index != 1;
            member.leave_told = index == 2;
            member.left = why;
            members.push(member).unwrap();
        }
        row.landing.members = members.into_boxed();
        row.landing.round.changed = Some(1);
        row.landing.round.search = Some(landing::Search { tests: landing::SearchTests::Whole, green: 1, red: 2 });
        row.asks = Box::new([None, None, None]);
        let row = forge::Stored::Landing(row);
        let limits = forge::Limits { landing_members: 3, ..fixtures::LIMITS };
        let bounds = crate::bounds(&limits, limits.client.op_bytes).unwrap();
        assert_eq!(records::decode(&records::encode(&row), &bounds), Ok(row));
    }
}

#[test]
fn landing_flake_reruns_round_trip_with_the_pending_jobs() {
    let mut row = seed();
    row.flake_reruns = Box::new([forge::FlakeRerun {
        key: forge::ReceiptKey {
            project: 5,
            repository: skein_forge::api::Repository { forge: 1, repository: 2 },
            check: forge::Check::Gate(b"gate".as_slice().into()),
            key: [7; 32],
        },
        run: 8,
        findings: Box::new([3, 7]),
        pending: Box::new([skein_forge::api::JobAttempt { head: [3; 32], run: 8, job: 9, attempt: 2 }]),
        redispatch: true,
        ended: true,
    }]);
    let row = forge::Stored::Landing(row);
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).unwrap();
    assert_eq!(records::decode(&records::encode(&row), &bounds), Ok(row));
}

#[test]
fn landing_repair_context_round_trips() {
    use temper_engine_domain_forge_change::{Ask, Below, Landed, Repair, State};
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).unwrap();
    let seed = seed();
    let mut changes = [None; 4];
    for (index, slot) in changes.iter_mut().enumerate() {
        *slot = Some(Landed { change: 2341 + u64::try_from(index).unwrap(), pull: 7 + u64::try_from(index).unwrap() });
    }
    for below in [
        None,
        Some(Below { by_test: false, top: false, green: [0xbb; 32], changes: [None; 4] }),
        Some(Below { by_test: false, top: false, green: [0xbb; 32], changes }),
    ] {
        let mut row = seed.clone();
        row.landing.phase = landing::Phase::Preparing;
        row.asks = Box::new([Some(Ask::Repair { gate: 1, candidate: [0xaa; 32], below })]);
        let row = forge::Stored::Landing(row);
        assert_eq!(records::decode(&records::encode(&row), &bounds), Ok(row));
        let row = fixtures::change_hot(fixtures::change_row(State::Repairing {
            requested: true,
            why: Repair::LandingAt { gate: 1, candidate: [0xaa; 32], below },
        }));
        assert_eq!(records::decode(&records::encode(&row), &bounds), Ok(row));
    }
}

#[test]
fn a_stored_fallback_leaves_as_held() {
    let bytes = records::encode(&forge::Stored::Landing(seed()));
    let row = v25::Row::decode(&v25::CEILINGS, &mut Reader::new(&bytes)).unwrap();
    let v25::Value::Landing(row) = row.value else { panic!("landing") };
    let mut row = row;
    let mut state = row.landing;
    let member = state.members.get_mut(0).unwrap();
    let mut parts = member.clone();
    parts.left = Some(v25::LandingLeave::Fallback(v25::LandingFallback { units: 3, top: [7; 32] }));
    *member = parts;
    row.landing = state;
    let forge::Stored::Landing(after) = stored(*row) else { panic!("landing") };
    assert_eq!(after.landing.members[0].left, Some(landing::Leave::Held));
}

#[test]
fn searches_by_test_round_trip() {
    let mut row = seed();
    row.landing.phase = landing::Phase::Proving;
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).unwrap();
    for tests in [
        landing::SearchTests::ByTest {
            gates: Box::new([Some(Box::new([Box::from(b"a".as_slice()), Box::from(b"b".as_slice())])), None]),
            probed: Box::new([
                landing::Probed { member: 1, gate: 0, run: 72, passed: true },
                landing::Probed { member: 2, gate: 0, run: 73, passed: false },
            ]),
        },
        landing::SearchTests::WholeAgain,
    ] {
        row.landing.round.search = Some(landing::Search { green: 1, red: 3, tests });
        row.landing.searched =
            Some(landing::Searched { red: [8; 32], green: Some([7; 32]), changes: Box::new([11, 12]) });
        row.landing.members[0].gates[0].posted = landing::Posted::ByTest;
        let stored = forge::Stored::Landing(row.clone());
        assert_eq!(records::decode(&records::encode(&stored), &bounds), Ok(stored));
    }
    assert_eq!(bounds.limits.landing_by_test_gates, fixtures::LIMITS.gates);
    assert_eq!(
        bounds.limits.landing_by_test_probed,
        fixtures::LIMITS.gates.checked_mul(fixtures::LIMITS.landing_members).unwrap()
    );
    assert_eq!(bounds.limits.landing_selection_tests, fixtures::LIMITS.tiers.failing_tests);
    assert_eq!(bounds.limits.landing_searched_changes, fixtures::LIMITS.landing_members);
}

#[test]
fn a_by_test_repair_keeps_its_top_fact() {
    use temper_engine_domain_forge_change::{Ask, Below, Repair, State};
    let seed = seed();
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).unwrap();
    for (by_test, top) in [(true, false), (true, true), (false, true)] {
        let below = Some(Below { green: [0xbb; 32], changes: [None; 4], by_test, top });
        let row = fixtures::change_hot(fixtures::change_row(State::Repairing {
            requested: true,
            why: Repair::LandingAt { gate: 1, candidate: [0xaa; 32], below },
        }));
        assert_eq!(records::decode(&records::encode(&row), &bounds), Ok(row));
        let mut row = seed.clone();
        row.landing.phase = landing::Phase::Preparing;
        row.asks = Box::new([Some(Ask::Repair { gate: 1, candidate: [0xaa; 32], below })]);
        let row = forge::Stored::Landing(row);
        assert_eq!(records::decode(&records::encode(&row), &bounds), Ok(row));
    }
}

#[test]
#[expect(clippy::disallowed_methods, reason = "ordinary Rust registered layout contract test")]
fn landing_waits_and_clocks_round_trip_and_default_on_older_rows() {
    use skein_codec::{
        layout::{Plan, Workspace},
        schema,
    };
    use skein_lib::{Buffer, Wall};
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).unwrap();
    let mut row = seed();
    row.last_event = Some(Wall::from_nanos(17));
    row.turn_since = Some(Wall::from_nanos(11));
    for wait in [
        None,
        Some(landing::Awaited::Release),
        Some(landing::Awaited::Ready { member: 3 }),
        Some(landing::Awaited::Turn),
        Some(landing::Awaited::Prepare),
        Some(landing::Awaited::Prefixes),
        Some(landing::Awaited::Open),
        Some(landing::Awaited::Close),
        Some(landing::Awaited::Delete { member: 2 }),
        Some(landing::Awaited::Post { member: 1, gate: 2 }),
        Some(landing::Awaited::Run { member: 2, gate: 3 }),
        Some(landing::Awaited::Receipt { member: 3, gate: 4 }),
        Some(landing::Awaited::Judgement { member: 4 }),
        Some(landing::Awaited::Merge),
        Some(landing::Awaited::Record { member: 5 }),
        Some(landing::Awaited::Ancestry { member: 6 }),
        Some(landing::Awaited::Read),
    ] {
        row.waiting = wait;
        let stored = forge::Stored::Landing(row.clone());
        assert_eq!(records::decode(&records::encode(&stored), &bounds), Ok(stored));
    }
    let family = crate::OWNER_SET.families.iter().find(|family| family.name == "temper-store").unwrap();
    let mut before = family.current.to_owned();
    for field in
        ["last_event: option u64 = none ", "turn_since: option u64 = none ", "waiting: option LandingAwaited = none "]
    {
        before = before.replacen(field, "", 1);
    }
    let old = schema::parse(before.as_bytes()).unwrap();
    let new = schema::parse(family.current.as_bytes()).unwrap();
    let old_root = old.root(b"Row", Some(b"landing")).unwrap();
    let new_root = new.root(b"Row", Some(b"landing")).unwrap();
    let down = Plan::derive_dropping(&new, new_root, &[], &old, old_root, &[]).unwrap();
    let up = Plan::derive(&old, old_root, &[], &new, new_root, &[]).expect("landing's added fields default to none");
    let mut work = Workspace::new();
    assert!(work.fit(&down));
    let mut earlier = Buffer::with_capacity(4096);
    down.convert(&records::encode(&forge::Stored::Landing(row.clone())), &mut earlier, &mut work).unwrap();
    assert!(work.fit(&up));
    let mut current = Buffer::with_capacity(4096);
    up.convert(earlier.as_slice(), &mut current, &mut work).unwrap();
    row.last_event = None;
    row.turn_since = None;
    row.waiting = None;
    assert_eq!(records::decode(current.as_slice(), &bounds), Ok(forge::Stored::Landing(row)));
}

#[test]
fn a_held_round_keeps_its_merge_count_and_last_read_across_storage() {
    let mut row = seed();
    row.landing.round.merges = 2;
    row.landing.round.failure = b"merge of #7: withdrawn before completion; read #7 open, temper/g1 at 0000000000000000000000000000000000000000".as_slice().into();
    row.landing.phase = landing::Phase::Held {
        was: Box::new(landing::Phase::Turn { at: skein_lib::Wall::EPOCH }),
        why: landing::Hold::Turn,
    };
    let row = forge::Stored::Landing(row);
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).unwrap();
    assert_eq!(records::decode(&records::encode(&row), &bounds), Ok(row));
}
