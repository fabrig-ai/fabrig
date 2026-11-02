//! Tier families and receipt failures (temper/docs/design/protocol/store.md, 2–4).
use super::fixtures;
use crate::{Undecodable, keys, records};
use temper_engine_domain_forge as d;
use temper_engine_domain_forge_tiers as p;

fn round_trip(row: d::Stored, family: u8) {
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).unwrap();
    let meter = skein_world::domain::heap::Meter::new();
    meter.start();
    let cloned = row.clone();
    let held = meter.end().held();
    assert_eq!(d::stored_bytes(&cloned), Some(held));
    assert_eq!(cloned, row);
    drop(cloned);
    let key = d::stored_key(&row);
    let bytes = keys::encode(&key, u32::MAX).unwrap();
    assert_eq!(bytes[0], family);
    assert_eq!(keys::decode(&bytes, &fixtures::LIMITS), Some(key));
    assert_eq!(records::row(&bytes, &records::encode(&row), &bounds), Ok(row));
}
#[test]
fn tier_records_keep_the_whole_policy_and_all_parts() {
    let full = fixtures::tier_row();
    let run = full.record.run.as_ref().unwrap();
    let finding = match &run.part {
        p::Part::Probe { finding, .. } => finding.clone(),
        p::Part::Whole | p::Part::Rerun { .. } => panic!("full probe fixture"),
    };
    for part in [
        run.part.clone(),
        p::Part::Whole,
        p::Part::Rerun { failed: Box::new([Box::from(b"failed".as_slice())]) },
        p::Part::Probe { landing: 0, tests: p::Selection::All, finding },
    ] {
        let mut row = full.clone();
        row.record.run.as_mut().unwrap().part = part;
        round_trip(d::Stored::Tier(row), 28);
    }
    let mut empty = full;
    empty.task = None;
    empty.requested = false;
    empty.cancelled = false;
    empty.purpose = None;
    empty.pending = Box::new([]);
    empty.baseline = None;
    empty.branch = d::TierBranch::Absent;
    empty.probe = None;
    empty.flight = None;
    empty.held = None;
    empty.result = None;
    empty.received = false;
    empty.record = p::Record { sequence: 0, judged: None, failing: p::Failures::Many, flakes: Box::new([]), run: None };
    round_trip(d::Stored::Tier(empty), 28);
}
#[test]
fn verification_keeps_every_verdict_and_repair_and_receipts_keep_failures() {
    for failures in [
        p::Failures::Many,
        p::Failures::Tests(Box::new([])),
        p::Failures::Tests(Box::new([Box::from(b"failed".as_slice())])),
    ] {
        for blame in [p::Blame::Cleared, p::Blame::Owns(failures.clone()), p::Blame::Suspect(failures.clone())] {
            round_trip(
                d::Stored::Verification(d::Verification {
                    repository: fixtures::REPO,
                    commit: [9; 32],
                    changes: Box::new([51, 52]),
                    tiers: Box::new([
                        d::TierVerification { tier: Box::from(b"one".as_slice()), verdict: None, repaired: None },
                        d::TierVerification {
                            tier: Box::from(b"two".as_slice()),
                            verdict: Some(d::TierVerdict { run: 7, blame }),
                            repaired: Some(d::TierRepair { run: 2, tip: [8; 32] }),
                        },
                    ]),
                }),
                29,
            );
        }
        for value in [None, Some(failures)] {
            round_trip(
                d::Stored::Receipt {
                    key: d::ReceiptKey {
                        project: 5,
                        repository: fixtures::REPO,
                        check: d::Check::Tier(Box::from(b"tier".as_slice())),
                        key: [3; 32],
                    },
                    receipt: d::Receipt {
                        failures: value,
                        passed: false,
                        run: d::ReceiptRun { id: 7, number: 8, url: Box::new([]) },
                        candidate: [4; 32],
                        ended: skein_lib::Wall::EPOCH,
                    },
                },
                24,
            );
        }
    }
}
#[test]
fn tier_decoding_checks_names_policy_invariants_and_configured_bounds() {
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).unwrap();
    for name in [Box::<[u8]>::default(), Box::from(b"bad/name".as_slice()), Box::from([b'a'; 33])] {
        let key = d::Key::Tier { repository: fixtures::REPO, tier: name.clone() };
        assert_eq!(keys::decode(&keys::encode(&key, u32::MAX).unwrap(), &fixtures::LIMITS), None);
        if name.len() <= 32 {
            let mut row = fixtures::tier_row();
            row.tier = name;
            assert_eq!(records::decode(&records::encode(&d::Stored::Tier(row)), &bounds), Err(Undecodable));
        }
    }
    for case in 0_u8..5 {
        let mut row = fixtures::tier_row();
        match case {
            0 => {
                row.record.failing =
                    p::Failures::Tests(Box::new([Box::from(b"b".as_slice()), Box::from(b"a".as_slice())]));
            }
            1 => {
                row.record.flakes = Box::new([
                    p::Flake { test: Box::from(b"b".as_slice()), count: 1 },
                    p::Flake { test: Box::from(b"a".as_slice()), count: 1 },
                ]);
            }
            2 => row.record.run.as_mut().unwrap().part = p::Part::Rerun { failed: Box::new([Box::new([])]) },
            3 => match &mut row.record.run.as_mut().unwrap().part {
                p::Part::Probe { landing, .. } => *landing = 17,
                p::Part::Whole | p::Part::Rerun { .. } => unreachable!(),
            },
            4 => match &mut row.record.run.as_mut().unwrap().part {
                p::Part::Probe { finding, .. } => finding.blamed = Box::new([]),
                p::Part::Whole | p::Part::Rerun { .. } => unreachable!(),
            },
            _ => unreachable!(),
        }
        assert_eq!(records::decode(&records::encode(&d::Stored::Tier(row)), &bounds), Err(Undecodable), "case {case}");
    }
    let full = records::encode(&d::Stored::Tier(fixtures::tier_row()));
    for field in 0_u8..3 {
        let mut limits = fixtures::LIMITS;
        match field {
            0 => limits.changes -= 1,
            1 => limits.tiers.failing_tests -= 1,
            2 => limits.tiers.test_id_bytes -= 1,
            _ => unreachable!(),
        }
        let bounds = crate::bounds(&limits, limits.client.op_bytes).unwrap();
        assert_eq!(records::decode(&full, &bounds), Err(Undecodable));
    }
    let row = d::Stored::Tier(fixtures::tier_row());
    let bytes = records::encode(&row);
    for len in [0, 1, 2, bytes.len() - 1] {
        assert_eq!(records::decode(&bytes[..len], &bounds), Err(Undecodable));
    }
}
#[test]
fn tier_family_bounds_follow_the_forge_and_full_row_measure() {
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).unwrap();
    let l = bounds.limits();
    assert_eq!(l.tier_run_landings, 2 * fixtures::LIMITS.changes + 1);
    assert_eq!(l.tier_finding_blamed, l.tier_run_landings);
    assert_eq!(l.verification_changes, fixtures::LIMITS.changes);
    assert_eq!(l.verification_tiers, fixtures::LIMITS.tiers.tiers);
    assert_eq!(l.failed_tests_tests, fixtures::LIMITS.tiers.failing_tests);
    assert_eq!(l.failing_test_test_id, fixtures::LIMITS.tiers.test_id_bytes);
    let row = records::encode(&d::Stored::Tier(fixtures::tier_row()));
    assert_eq!(u64::try_from(row.len()).unwrap(), crate::v11::TierRow::encoded_bound(l).unwrap() + 3);
    assert_eq!(row.len(), 103_061);
    assert!(row.len() < 118_676);
}

#[test]
fn tier_operations_keep_branch_states_faults_and_pending_tips() {
    use temper_engine_domain_forge_landing::Fault;
    for (branch, fault) in [
        (d::TierBranch::Absent, Fault::SetUp),
        (d::TierBranch::Made, Fault::Completion),
        (d::TierBranch::Pending { entry: 31 }, Fault::NoStep),
        (d::TierBranch::Made, Fault::Cancelled),
        (d::TierBranch::Made, Fault::OtherCommit),
        (d::TierBranch::Made, Fault::Lost),
    ] {
        let mut row = fixtures::tier_row();
        row.branch = branch;
        row.held = Some(fault);
        row.flight.as_mut().unwrap().fault = Some(fault);
        row.pending[0].key = None;
        round_trip(d::Stored::Tier(row), 28);
    }
}
#[test]
fn tier_operation_lists_refuse_one_past_their_configured_bounds() {
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).unwrap();
    for pending_tips in [true, false] {
        let mut row = fixtures::tier_row();
        if pending_tips {
            let mut tips = skein_lib::List::with_capacity(u32::try_from(row.pending.len()).unwrap() + 1);
            for tip in &row.pending {
                tips.push(*tip).unwrap();
            }
            tips.push(d::TierLanding { commit: [99; 32], key: None }).unwrap();
            row.pending = tips.into_boxed();
        } else {
            let flight = row.flight.as_mut().unwrap();
            let mut jobs = skein_lib::List::with_capacity(u32::try_from(flight.pending.len()).unwrap() + 1);
            for job in &flight.pending {
                jobs.push(*job).unwrap();
            }
            jobs.push(skein_forge::api::JobAttempt { head: [4; 32], run: 71, job: 99, attempt: 1 }).unwrap();
            flight.pending = jobs.into_boxed();
        }
        assert_eq!(records::decode(&records::encode(&d::Stored::Tier(row)), &bounds), Err(Undecodable));
    }
}
