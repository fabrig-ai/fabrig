//! A repair's flake waits for release and regates at unchanged content (landing-regate.md, 3–5).
use super::super::landing_routes;
use super::*;
use skein_forge::api::{self, Answer, Check, Op, Read};
use temper_engine_domain_forge_change as change;
use temper_engine_domain_forge_landing as landing;

const WORDS: &[u8] = b"test a::t x passed again at this head";
fn why() -> change::Repair {
    change::Repair::LandingAt { gate: 0, candidate: fixtures::HEAD, below: None }
}
fn repairing_flake() -> (Domain, ReceiptKey, api::Run) {
    let (mut d, key, run) = landing_routes::regate_landing();
    d.landing.projects.get_mut(&5).unwrap().1[0].gates[0].report = Some(GateReport {
        artifact: b"report".as_slice().into(),
        format: temper_engine_domain_forge_tiers::Format::Lines,
        fuzzy: None,
    });
    let waiting =
        delegate_waiting(change::State::Repairing { requested: true, why: why() }, change::Delegate::Repair(why()));
    let mut row = waiting.change(fixtures::CHANGE).unwrap().clone();
    row.title = b"Small fix".as_slice().into();
    row.pull = Some(1);
    row.change.repairs = 1;
    row.change.way = change::Way::Turn;
    row.repair_tasks = Box::new([52]);
    outputs(&mut d, Event::Restore { record: Stored::Change(Box::new(row)) });
    d.landing.receipts.get_mut(&key).unwrap().report =
        ReceiptReport::Tests(Box::new([ReceiptTest { key: b"test a::t x".as_slice().into(), fuzzy: false }]));
    (d, key, run)
}

fn change_step_at(d: &mut Domain, head: api::Commit) -> change::Decision {
    let mut requests = outputs(d, waiting_step(false, false));
    for _ in 0..30_u32 {
        for request in &requests {
            if let Request::ChangeDecision { decision, .. } = request {
                return *decision;
            }
        }
        let (call, op) = fixtures::next_call(d);
        let answer = match op {
            Op::Read(Read::Branch { branch }) => {
                Answer::Commit(if branch.as_ref() == b"main" { fixtures::BASE } else { head })
            }
            Op::Read(Read::Pull { .. }) => Answer::Pull(api::Pull { commit: head, ..fixtures::open_pull() }),
            Op::Read(Read::Checks { .. }) => fixtures::checks(Check::Passed),
            Op::Read(Read::Commits { .. }) => Answer::Commits { commits: Box::new([]), more: false },
            Op::Read(Read::Reviews { .. }) => Answer::Reviews { reviews: Box::new([]), more: false, cut: false },
            Op::Read(Read::Compare { before, after }) => {
                Answer::Compare { before, after, ahead: 1, behind: 0, files: Box::new([]), changed: 0 }
            }
            Op::Read(_) | Op::Write(_) => panic!("unexpected change read {op:?}"),
        };
        requests = fixtures::answer(d, call, Ok(answer));
    }
    panic!("change decides")
}

fn restart(d: &Domain, key: &ReceiptKey) -> Domain {
    let mut restored = landing_routes::landing_fixture();
    let (cold, hot) = d.change(fixtures::CHANGE).unwrap().clone().split();
    outputs(&mut restored, Event::Restore { record: Stored::ChangeCold(Box::new(cold)) });
    outputs(&mut restored, Event::Restore { record: Stored::ChangeHot(Box::new(hot)) });
    let (way, gates, _, _, _, settings, _) = d.landing.projects.get(&5).unwrap();
    assert!(restored.project_landing(5, *way, gates.clone(), *settings, d.after_landing(5).unwrap()));
    outputs(&mut restored, Event::Restore { record: Stored::Landing(d.landing.rows.get(&61).unwrap().clone()) });
    outputs(
        &mut restored,
        Event::Restore { record: Stored::Receipt { key: key.clone(), receipt: d.receipt(key).unwrap().clone() } },
    );
    for (_, entry) in &d.entries {
        outputs(&mut restored, Event::Restore { record: Stored::Entry { entry: entry.entry.clone(), held: false } });
    }
    fixtures::connect(&mut restored);
    outputs(&mut restored, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut restored, Event::ReadAfresh);
    outputs(&mut restored, Event::SettleOutbox);
    restored.landing.main_keys.insert((REPO, b"gate".as_slice().into()), (fixtures::BASE, key.key)).unwrap();
    restored
}

fn hold_and_release(d: &mut Domain, key: &ReceiptKey) {
    verdict_terminal(d, 52, WORDS);
    // The successful terminal flag, not an inbox notice, survives the branch read's restart.
    *d = restart(d, key);
    assert!(d.change(fixtures::CHANGE).unwrap().delegate_flake, "terminal survived restart");
    assert_eq!(
        change_step_at(d, fixtures::HEAD),
        change::Decision::Hold(change::Hold::Flake),
        "{:?}",
        d.change(fixtures::CHANGE)
    );
    let row = d.change(fixtures::CHANGE).unwrap();
    assert_eq!(row.change.repairs, 1);
    assert_eq!(row.delegate_status, change::Status::Passed);
    assert_eq!(row.producer_failure.as_deref(), Some(b"its repair, task 52, found only flakes: test a::t x passed again at this head; release this change to run its gate once more, or cancel it".as_slice()));
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
    let row = d.landing.rows.remove(&61).unwrap();
    outputs(d, Event::Landing { row });
}

#[test]
fn a_released_flake_regates_across_restart_and_proves_green_without_a_commit() {
    let (mut d, key, run) = repairing_flake();
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
fn a_red_regate_counts_another_repair_and_its_flake_waits_for_another_release() {
    let (mut d, key, mut run) = repairing_flake();
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
    assert_eq!(
        d.receipt(&key).unwrap().report,
        ReceiptReport::Tests(Box::new([ReceiptTest { key: b"test a::t x".as_slice().into(), fuzzy: false },]))
    );
    assert_eq!(change_step_at(&mut d, fixtures::HEAD), change::Decision::Delegate(change::Delegate::Repair(why())));
    assert_eq!(d.change(fixtures::CHANGE).unwrap().regate, None);
    outputs(&mut d, Event::Delegated { task: fixtures::CHANGE, child: 53, kind: change::Delegate::Repair(why()) });
    verdict_terminal(&mut d, 53, WORDS);
    assert_eq!(change_step_at(&mut d, fixtures::HEAD), change::Decision::Hold(change::Hold::Flake));
    let held = d.change(fixtures::CHANGE).unwrap();
    assert_eq!(held.change.repairs, 1);
    assert_eq!(held.repair_tasks.as_ref(), [52, 53]);
    assert_eq!(held.regate, None);
    d = restart(&d, &key);
    let held = d.change(fixtures::CHANGE).unwrap().change.state.clone();
    assert_ne!(change_step_at(&mut d, fixtures::HEAD), change::Decision::Delegate(change::Delegate::Repair(why())));
    assert_eq!(d.change(fixtures::CHANGE).unwrap().change.state, held);
    assert!(d.landing.flights.is_empty(), "another flake never dispatches without a release");
    assert_eq!(d.receipt(&key).unwrap().run.id, 57);
}

#[test]
fn a_flake_account_marks_its_cut_and_a_missing_receipt_does_not_ask_a_repair() {
    let (mut d, key, _) = repairing_flake();
    d.landing.receipts.remove(&key);
    let words = "test a::t x passed again; ".repeat(100);
    verdict_terminal(&mut d, 52, words.as_bytes());
    assert_eq!(change_step_at(&mut d, fixtures::HEAD), change::Decision::Hold(change::Hold::Flake));
    let account = d.change(fixtures::CHANGE).unwrap().producer_failure.as_ref().unwrap();
    assert_eq!(account.len(), usize::try_from(jig_domain_tasks::HOLD_ACCOUNT_BYTES).unwrap());
    assert!(account.starts_with(b"its repair, task 52, found only flakes: test a::t x passed again; "));
    assert!(account.ends_with(b"...; release this change to run its gate once more, or cancel it"));
    outputs(&mut d, Event::ReleaseChange { task: fixtures::CHANGE, release: None });
    assert_eq!(d.change(fixtures::CHANGE).unwrap().regate, None);
    assert_eq!(change_step_at(&mut d, fixtures::HEAD), change::Decision::Ready);
    assert_eq!(d.change(fixtures::CHANGE).unwrap().repair_tasks.as_ref(), [52]);
}

#[test]
fn a_flake_at_a_moved_branch_is_a_push_and_clears_the_regate() {
    for already_read in [false, true] {
        let (mut d, key, _) = repairing_flake();
        let advanced = [9; 32];
        let row = d.changes.get_mut(&fixtures::CHANGE).unwrap();
        if already_read {
            row.change.last_head = Some(advanced);
        } else {
            row.regate = Some(Regate { receipt: key, run: 41 });
        }
        verdict_terminal(&mut d, 52, WORDS);
        assert_ne!(change_step_at(&mut d, advanced), change::Decision::Hold(change::Hold::Flake));
        let row = d.change(fixtures::CHANGE).unwrap();
        assert_eq!(row.change.last_head, Some(advanced));
        assert_eq!(row.regate, None);
        assert_eq!(row.change.repairs, 1);
    }
}
