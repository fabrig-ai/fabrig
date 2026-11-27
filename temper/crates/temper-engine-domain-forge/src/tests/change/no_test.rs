//! A repair's no-test verdict regates without release at unchanged content (landing-regate.md, 3–5).
use super::super::landing_routes;
use super::flake::{change_step_at, next_landing, release_legacy, restart};
use super::*;
use skein_forge::api;
use temper_engine_domain_forge_change as change;
use temper_engine_domain_forge_landing as landing;

const WORDS: &[u8] = b"the build failed before tests ran";
fn why() -> change::Repair {
    change::Repair::LandingAt { gate: 0, candidate: fixtures::HEAD, below: None }
}
fn repairing_no_test() -> (Domain, ReceiptKey, api::Run) {
    let (mut d, key, mut run) = landing_routes::clocked_gate();
    run.id = 41;
    run.number = 42;
    run.url = b"https://forge/run/41".as_slice().into();
    run.state = api::RunState::Completed;
    run.conclusion = Some(api::Conclusion::Failure);
    run.jobs = Box::new([api::RunJob {
        id: 9,
        attempt: 1,
        name: b"build".as_slice().into(),
        state: api::RunState::Completed,
        conclusion: Some(api::Conclusion::Failure),
        steps: 3,
        step: Some(api::Step { number: 1, name: b"compile".as_slice().into() }),
    }]);
    assert_eq!(
        landing_routes::step_regate(&mut d, 61, 90, &run),
        landing::Decision::Leave { member: 0, why: landing::Leave::Failed { gate: 0, run: 42 } }
    );
    assert_eq!(d.receipt(&key).unwrap().report, ReceiptReport::Unset);
    // The first dispatch is finished; only the released landing's entry can be recovered.
    d.entries.remove(&9);
    d.landing.rows.get_mut(&61).unwrap().entries = Box::new([]);
    let row = d.changes.get_mut(&fixtures::CHANGE).unwrap();
    row.pending = None;
    row.effect = change::EffectResult::None;
    row.title = b"Small fix".as_slice().into();
    row.pull = Some(1);
    row.change.last_head = Some(fixtures::HEAD);
    row.change.way = change::Way::Turn;
    assert_eq!(change_step_at(&mut d, fixtures::HEAD), change::Decision::Delegate(change::Delegate::Repair(why())));
    let failure = d.landing_failure(fixtures::CHANGE, why()).unwrap();
    let row = d.change(fixtures::CHANGE).unwrap();
    let words = change_words::repair_words(row, why(), Some(&failure), &[], 4096).unwrap();
    assert!(skein_lib::bytes::find(&words, b"Failed tests: not known (the gate names no report)").is_some());
    assert!(skein_lib::bytes::find(&words, b"\nNo test: if no test failed in the run below").is_some());
    assert!(change_words::repair_no_test(why(), Some(&failure)));
    outputs(&mut d, Event::Delegated { task: fixtures::CHANGE, child: 52, kind: change::Delegate::Repair(why()) });
    run.id = 57;
    run.number = 58;
    run.created = env().now.saturating_add(Duration::from_secs(1));
    run.url = b"https://forge/run/57".as_slice().into();
    run.conclusion = Some(api::Conclusion::Success);
    run.jobs = Box::new([]);
    (d, key, run)
}

fn no_test_terminal(d: &mut Domain, child: u64, words: &[u8]) {
    outputs(
        d,
        Event::DelegateResult {
            task: fixtures::CHANGE,
            child,
            ended_at: None,
            status: change::Status::Passed,
            no_change: false,
            flake: false,
            no_test: true,
            failed: false,
            lost: false,
            cancelled: false,
            words: words.into(),
        },
    );
}

fn regate(d: &mut Domain, key: &ReceiptKey) {
    no_test_terminal(d, 52, WORDS);
    // Restart between the successful terminal and its fresh branch read.
    *d = restart(d, key);
    assert_eq!(change_step_at(d, fixtures::HEAD), change::Decision::Ready);
    let row = d.change(fixtures::CHANGE).unwrap();
    assert_eq!(row.regate, Some(Regate { receipt: key.clone(), run: 41, taken: None }));
    assert_eq!(row.change.last_head, Some(fixtures::HEAD));
    assert_eq!(row.change.repairs, 1);
    assert_eq!(row.repair_tasks.as_ref(), [52]);
    assert_eq!(row.delegate, None);
    assert_eq!(row.delegate_words, None);
    assert!(
        skein_lib::bytes::find(
            &row.result.repairs.bytes,
            b"successful repair task 52: changed nothing (no failed test): "
        )
        .is_some()
    );
    assert!(row.result.repairs.bytes.ends_with(WORDS));
    next_landing(d, key);
}

#[test]
fn an_unchanged_no_test_regates_across_restart_and_proves_green_without_a_commit() {
    let (mut d, key, run) = repairing_no_test();
    regate(&mut d, &key);
    landing_routes::dispatch_regate(&mut d, &key, &run);
    d = restart(&d, &key);
    assert_eq!(
        landing_routes::step_regate(&mut d, 61, 93, &run),
        landing::Decision::Post { member: 0, gate: 0, posting: landing::Posting::Ran { passed: true } }
    );
    let receipt = d.receipt(&key).unwrap();
    assert!(receipt.passed);
    assert_eq!((receipt.run.id, receipt.candidate), (57, fixtures::HEAD));
    let restored = restart(&d, &key);
    assert_eq!(restored.change(fixtures::CHANGE).unwrap().regate.as_ref().unwrap().taken, Some(57));
    landing_routes::commit_gate_effect(&mut d, 93);
    assert_eq!(landing_routes::step_regate(&mut d, 61, 94, &run), landing::Decision::Proved);
    assert_eq!(d.change(fixtures::CHANGE).unwrap().change.last_head, Some(fixtures::HEAD));
}

#[test]
fn a_previous_no_test_hold_releases_into_one_regate_whose_red_stands() {
    let (mut d, key, mut run) = repairing_no_test();
    release_legacy(&mut d, &key, false);
    run.conclusion = Some(api::Conclusion::Failure);
    run.jobs = Box::new([api::RunJob {
        id: 9,
        attempt: 1,
        name: b"test".as_slice().into(),
        state: api::RunState::Completed,
        conclusion: Some(api::Conclusion::Failure),
        steps: 3,
        step: Some(api::Step { number: 1, name: b"tests".as_slice().into() }),
    }]);
    landing_routes::dispatch_regate(&mut d, &key, &run);
    d = restart(&d, &key);
    assert_eq!(
        landing_routes::step_regate(&mut d, 61, 93, &run),
        landing::Decision::Leave { member: 0, why: landing::Leave::Failed { gate: 0, run: 58 } }
    );
    d = restart(&d, &key);
    assert_eq!(change_step_at(&mut d, fixtures::HEAD), change::Decision::Hold(change::Cause::RedStands));
    let held = d.change(fixtures::CHANGE).unwrap();
    assert_eq!(
        held.change.state,
        change::State::Held {
            was: Box::new(change::State::Repairing { requested: false, why: why() }),
            hold: change::Hold {
                judged: Some(change::Judged { head: fixtures::HEAD, spec: held.spec_revision }),
                ..change::Hold::new(change::Cause::RedStands, env().wall)
            },
        }
    );
    assert_eq!(held.producer_failure.as_deref(), Some(b"its red at 030303030303 stands: repair task 52 changed nothing, and the same content failed again: test; amend this change to ask a fresh repair, measure it again, or cancel it".as_slice()));
    assert_eq!(held.change.repairs, 1);
    assert_eq!(held.repair_tasks.as_ref(), [52]);
    assert_eq!(held.regate.as_ref().unwrap().taken, Some(57));
    assert!(d.landing.flights.is_empty(), "the taken regate never dispatches again");
    assert_eq!(d.receipt(&key).unwrap().run.id, 57);
    let revised = held.spec_revision + 1;
    d.refresh_spec_revision(fixtures::CHANGE, revised, &mut Queue::uncapped());
    outputs(&mut d, Event::ReleaseChange { task: fixtures::CHANGE, release: None });
    assert_eq!(change_step_at(&mut d, fixtures::HEAD), change::Decision::Delegate(change::Delegate::Repair(why())));
    let next = d.change(fixtures::CHANGE).unwrap();
    assert_eq!(next.regate, None);
    assert_eq!(next.change.repairs, 2);
}

#[test]
fn a_no_test_without_its_receipt_holds_and_keeps_its_words() {
    let (mut d, key, _) = repairing_no_test();
    d.landing.receipts.remove(&key);
    let words = "test a::t x passed again; ".repeat(100);
    no_test_terminal(&mut d, 52, words.as_bytes());
    assert_eq!(change_step_at(&mut d, fixtures::HEAD), change::Decision::Hold(change::Cause::RedStands));
    let row = d.change(fixtures::CHANGE).unwrap();
    assert_eq!(row.producer_failure.as_deref(), Some(b"its red at 030303030303 stands: repair task 52 changed nothing, and the same content can't be checked again: the landing no longer keeps its failed receipt; amend this change to ask a fresh repair, or cancel it".as_slice()));
    assert!(row.result.repairs.bytes.ends_with(words.as_bytes()));
    assert_eq!(row.regate, None);
    assert_eq!(row.repair_tasks.as_ref(), [52]);
}

#[test]
fn a_no_test_at_a_moved_branch_is_a_push_and_clears_the_regate() {
    for already_read in [false, true] {
        let (mut d, key, _) = repairing_no_test();
        let advanced = [9; 32];
        let name = run_branch(b"51");
        outputs(
            &mut d,
            Event::Restore {
                record: Stored::BranchHead(BranchHead { name: name.clone(), commit: fixtures::HEAD, owned: true }),
            },
        );
        fixtures::admit_push(&mut d, &name, 52, 1, Some(fixtures::HEAD), advanced);
        let row = d.changes.get_mut(&fixtures::CHANGE).unwrap();
        if already_read {
            row.change.last_head = Some(advanced);
        } else {
            row.regate = Some(Regate { receipt: key, run: 41, taken: None });
        }
        no_test_terminal(&mut d, 52, WORDS);
        assert_eq!(change_step_at(&mut d, advanced), change::Decision::Ready);
        let row = d.change(fixtures::CHANGE).unwrap();
        assert_eq!(row.change.last_head, Some(advanced));
        assert_eq!(row.regate, None);
        assert_eq!(row.change.repairs, 1);
        assert!(skein_lib::bytes::find(&row.result.repairs.bytes, b"successful repair task 52: ").is_some());
        assert!(row.result.repairs.bytes.ends_with(WORDS));
        assert!(skein_lib::bytes::find(&row.result.repairs.bytes, b"changed nothing").is_none());
    }
}
