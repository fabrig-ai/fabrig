//! A repair's no-test verdict waits for release and regates at unchanged content (landing-regate.md, 3–5).
use super::super::landing_routes;
use super::flake::{change_step_at, restart};
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
            cancelled: false,
            words: words.into(),
        },
    );
}

fn hold_and_release(d: &mut Domain, key: &ReceiptKey) {
    no_test_terminal(d, 52, WORDS);
    // The successful terminal flag, not an inbox notice, survives the branch read's restart.
    *d = restart(d, key);
    assert!(d.change(fixtures::CHANGE).unwrap().delegate_no_test, "terminal survived restart");
    assert_eq!(
        change_step_at(d, fixtures::HEAD),
        change::Decision::Hold(change::Hold::NoTest),
        "{:?}",
        d.change(fixtures::CHANGE)
    );
    let row = d.change(fixtures::CHANGE).unwrap();
    assert_eq!(row.change.repairs, 1);
    assert_eq!(row.delegate_status, change::Status::Passed);
    assert_eq!(row.producer_failure.as_deref(), Some(b"its repair, task 52, found no failed test: the build failed before tests ran; release this change to run its gate once more, or cancel it".as_slice()));
    assert!(d.landing.flights.is_empty());
    outputs(d, Event::ReleaseChange { task: fixtures::CHANGE, release: None });
    let expected = Some(Regate { receipt: key.clone(), run: 41 });
    assert_eq!(d.change(fixtures::CHANGE).unwrap().regate, expected);
    *d = restart(d, key);
    assert_eq!(d.change(fixtures::CHANGE).unwrap().regate, expected);
    assert_eq!(change_step_at(d, fixtures::HEAD), change::Decision::Ready);
    let row = d.change(fixtures::CHANGE).unwrap();
    assert_eq!(row.change.last_head, Some(fixtures::HEAD));
    assert_eq!(row.change.repairs, 0);
    assert_eq!(row.repair_tasks.as_ref(), [52]);
    assert_eq!(row.delegate, None);
    // Form the released ready member into the fixture's new landing.
    let mut row = landing_routes::erase_landing(d, 61);
    row.entries = Box::new([]);

    row.landing.phase = landing::Phase::Proving;
    row.landing.last_left = None;
    row.landing.round = landing::Round::new(b"temper/1/l61".as_slice().into());
    row.landing.round.pull = Some(2);
    row.landing.round.combined = Some(landing::Combine::Merge);
    let member = &mut row.landing.members[0];
    member.left = None;
    member.leave_told = false;
    member.gates[0].proof = landing::Proof::Keyed { key: key.key };
    member.gates[0].posted = landing::Posted::Nothing;
    outputs(d, Event::Landing { row });
}

#[test]
fn a_released_no_test_regates_across_restart_and_proves_green_without_a_commit() {
    let (mut d, key, run) = repairing_no_test();
    hold_and_release(&mut d, &key);
    landing_routes::dispatch_regate(&mut d, &key, &run);
    d = restart(&d, &key);
    assert_eq!(
        landing_routes::step_regate(&mut d, 61, 93, &run),
        landing::Decision::Post { member: 0, gate: 0, posting: landing::Posting::Ran { passed: true } }
    );
    let receipt = d.receipt(&key).unwrap();
    assert!(receipt.passed);
    assert_eq!((receipt.run.id, receipt.candidate), (57, fixtures::HEAD));
    landing_routes::commit_gate_effect(&mut d, 93);
    assert_eq!(landing_routes::step_regate(&mut d, 61, 94, &run), landing::Decision::Proved);
    assert_eq!(d.change(fixtures::CHANGE).unwrap().change.last_head, Some(fixtures::HEAD));
}

#[test]
fn a_red_regate_counts_another_repair_and_its_no_test_waits_for_another_release() {
    let (mut d, key, mut run) = repairing_no_test();
    hold_and_release(&mut d, &key);
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
    assert!(!d.receipt(&key).unwrap().passed);
    assert_eq!(d.receipt(&key).unwrap().run.id, 57);
    assert_eq!(d.receipt(&key).unwrap().report, ReceiptReport::Unset);
    assert_eq!(change_step_at(&mut d, fixtures::HEAD), change::Decision::Delegate(change::Delegate::Repair(why())));
    assert_eq!(d.change(fixtures::CHANGE).unwrap().regate, None);
    outputs(&mut d, Event::Delegated { task: fixtures::CHANGE, child: 53, kind: change::Delegate::Repair(why()) });
    no_test_terminal(&mut d, 53, WORDS);
    assert_eq!(change_step_at(&mut d, fixtures::HEAD), change::Decision::Hold(change::Hold::NoTest));
    let held = d.change(fixtures::CHANGE).unwrap();
    assert_eq!(held.change.repairs, 1);
    assert_eq!(held.repair_tasks.as_ref(), [52, 53]);
    assert_eq!(held.regate, None);
    d = restart(&d, &key);
    let held = d.change(fixtures::CHANGE).unwrap().change.state.clone();
    assert_ne!(change_step_at(&mut d, fixtures::HEAD), change::Decision::Delegate(change::Delegate::Repair(why())));
    assert_eq!(d.change(fixtures::CHANGE).unwrap().change.state, held);
    assert!(d.landing.flights.is_empty(), "another no-test verdict never dispatches without a release");
    assert_eq!(d.receipt(&key).unwrap().run.id, 57);
}

#[test]
fn a_no_test_account_marks_its_cut_and_a_missing_receipt_does_not_ask_a_repair() {
    let (mut d, key, _) = repairing_no_test();
    d.landing.receipts.remove(&key);
    let words = "test a::t x passed again; ".repeat(100);
    no_test_terminal(&mut d, 52, words.as_bytes());
    assert_eq!(change_step_at(&mut d, fixtures::HEAD), change::Decision::Hold(change::Hold::NoTest));
    let account = d.change(fixtures::CHANGE).unwrap().producer_failure.as_ref().unwrap();
    assert_eq!(account.len(), usize::try_from(jig_domain_tasks::HOLD_ACCOUNT_BYTES).unwrap());
    assert!(account.starts_with(b"its repair, task 52, found no failed test: test a::t x passed again; "));
    assert!(account.ends_with(b"...; release this change to run its gate once more, or cancel it"));
    outputs(&mut d, Event::ReleaseChange { task: fixtures::CHANGE, release: None });
    assert_eq!(d.change(fixtures::CHANGE).unwrap().regate, None);
    assert_eq!(change_step_at(&mut d, fixtures::HEAD), change::Decision::Ready);
    assert_eq!(d.change(fixtures::CHANGE).unwrap().repair_tasks.as_ref(), [52]);
}

#[test]
fn a_no_test_at_a_moved_branch_is_a_push_and_clears_the_regate() {
    for already_read in [false, true] {
        let (mut d, key, _) = repairing_no_test();
        let advanced = [9; 32];
        let row = d.changes.get_mut(&fixtures::CHANGE).unwrap();
        if already_read {
            row.change.last_head = Some(advanced);
        } else {
            row.regate = Some(Regate { receipt: key, run: 41 });
        }
        no_test_terminal(&mut d, 52, WORDS);
        assert_ne!(change_step_at(&mut d, advanced), change::Decision::Hold(change::Hold::NoTest));
        let row = d.change(fixtures::CHANGE).unwrap();
        assert_eq!(row.change.last_head, Some(advanced));
        assert_eq!(row.regate, None);
        assert_eq!(row.change.repairs, 1);
    }
}
