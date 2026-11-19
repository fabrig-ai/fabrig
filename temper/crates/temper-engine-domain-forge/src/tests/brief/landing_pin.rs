//! A landing repair keeps its round's comparison base (forge/change/repairs.md).
use super::*;
use skein_forge::api;

const CHILD: u64 = 90;
const CANDIDATE: api::Commit = [7; 32];

fn reasons() -> [change::Repair; 2] {
    [change::Repair::LandingAt { gate: 0, candidate: CANDIDATE, below: None }, change::Repair::Landing { gate: 0 }]
}

fn failed_round(why: change::Repair) -> Domain {
    let mut d = fixtures::connected_domain();
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut d, Event::ReadAfresh);
    outputs(&mut d, Event::SettleOutbox);
    let mut row = fixtures::opening_row();
    row.pending = None;
    row.effect = change::EffectResult::None;
    row.pull = Some(1);
    row.change.last_head = Some(fixtures::HEAD);
    row.change.state = change::State::Repairing { requested: true, why };
    outputs(&mut d, Event::Change { row });
    kept_candidate_failure(&mut d);
    kept_landing_failure(&mut d);
    let key = d.landing.receipts.iter().next().expect("failed receipt").0.clone();
    d.landing.receipts.get_mut(&key).expect("failed receipt").base = Some(fixtures::BASE);
    d
}

fn delegate(d: &mut Domain, kind: change::Delegate) {
    outputs(d, Event::Delegated { task: fixtures::CHANGE, child: CHILD, kind });
}

fn seed(saved: bool) -> items::Seed {
    items::Seed {
        project: 5,
        modify: false,
        task: CHILD,
        parent: Some(fixtures::CHANGE),
        repository: REPO,
        run_branch: b"run".as_slice().into(),
        saved_branch: saved.then_some(b"saved".as_slice().into()),
    }
}

#[test]
fn landing_repair_items_pin_the_receipts_base_without_changing_the_checkout() {
    for why in reasons() {
        let mut d = failed_round(why);
        delegate(&mut d, change::Delegate::Repair(why));
        let row = d.changes.get_mut(&fixtures::CHANGE).expect("change");
        row.change.last_checked_by = Some(CHILD);
        row.change.last_checked = Some(fixtures::HEAD);
        for saved in [false, true] {
            let item = d.workspace_item(seed(saved)).expect("repair checkout");
            assert_eq!(item.pin, Some(fixtures::BASE));
            let key = d.landing.receipts.iter().next().expect("receipt").0.clone();
            d.landing.receipts.get_mut(&key).expect("receipt").base = None;
            let unpinned = d.workspace_item(seed(saved)).expect("unpinned checkout");
            assert_eq!(unpinned.pin, None);
            assert_eq!(item.start, unpinned.start);
            assert_eq!(item.push, unpinned.push);
            assert_eq!(item.landing, unpinned.landing);
            assert_eq!(item.delivers, unpinned.delivers);
            assert_eq!(item.delivered, unpinned.delivered);
            assert_eq!(item.holder, unpinned.holder);
            assert_eq!(item.bytes, unpinned.bytes);
            assert_eq!(item.delivered, Some(fixtures::HEAD));
            assert_eq!(item.landing, None);
            if saved {
                assert_eq!(item.start, items::Start::Saved(b"saved".as_slice().into()));
            }
            d.landing.receipts.get_mut(&key).expect("receipt").base = Some(fixtures::BASE);
        }
        let key = d.landing.receipts.iter().next().expect("receipt").0.clone();
        d.landing.receipts.remove(&key).expect("receipt");
        assert_eq!(d.workspace_item(seed(false)).expect("unresolved repair").pin, None);
    }
}

#[test]
fn other_delegates_never_name_a_pin_even_with_a_failed_landing() {
    for kind in [
        change::Delegate::Repair(change::Repair::Ci),
        change::Delegate::Repair(change::Repair::Gate(1)),
        change::Delegate::Repair(change::Repair::Review { head: fixtures::HEAD }),
        change::Delegate::Repair(change::Repair::Semantic),
        change::Delegate::Produce,
        change::Delegate::Close,
        change::Delegate::Resolve { base: fixtures::BASE },
        change::Delegate::Merge { base: fixtures::BASE, from: Some(fixtures::HEAD) },
        change::Delegate::Gate { number: 1, head: fixtures::HEAD },
    ] {
        let mut d = failed_round(reasons()[0]);
        delegate(&mut d, kind);
        assert_eq!(d.workspace_item(seed(false)).expect("delegate checkout").pin, None, "{kind:?}");
    }
}

#[test]
fn read_only_repairs_goal_runs_and_landing_prepares_name_no_pin() {
    let why = reasons()[0];
    let mut d = failed_round(why);
    delegate(&mut d, change::Delegate::Repair(why));
    for context in [false, true] {
        let mut repository = adopted();
        if context {
            repository.role = Role::Context;
        } else {
            repository.kinds.push = false;
        }
        outputs(&mut d, Event::Restore { record: Stored::Repository(repository) });
        let item = d.workspace_item(seed(false)).expect("read-only checkout");
        assert_eq!(item.push, None);
        assert_eq!(item.pin, None);
    }
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    let goal_branch = Name {
        forge: REPO.forge,
        repository: REPO.repository,
        what: What::Branch(Box::new([b"temper".as_slice().into(), b"90".as_slice().into(), b"g90".as_slice().into()])),
    };
    outputs(
        &mut d,
        Event::Restore {
            record: Stored::Hold(Hold {
                name: goal_branch.clone(),
                task: CHILD,
                writer: None,
                beside: None,
                drift: None,
                design_writers: Box::new([]),
            }),
        },
    );
    outputs(
        &mut d,
        Event::Restore {
            record: Stored::BranchHead(BranchHead { name: goal_branch, commit: fixtures::BASE, owned: true }),
        },
    );
    let mut goal = seed(false);
    goal.parent = None;
    goal.modify = true;
    let item = d.workspace_item(goal).expect("goal checkout");
    assert_eq!(item.start, items::Start::Base(b"temper/90/g90".as_slice().into()));
    assert!(item.delivers);
    assert_eq!(item.pin, None);
    assert_eq!(d.prepare_item(REPO, b"prepare".as_slice().into(), fixtures::BASE, 61).expect("prepare").pin, None);
}

#[expect(clippy::fn_params_excessive_bools, reason = "the delegate event distinguishes each terminal account")]
fn terminal(d: &mut Domain, failed: bool, no_change: bool, flake: bool, cancelled: bool, words: &[u8]) {
    outputs(
        d,
        Event::DelegateResult {
            task: fixtures::CHANGE,
            child: CHILD,
            ended_at: None,
            status: if failed || cancelled { change::Status::Failed } else { change::Status::Passed },
            no_change,
            flake,
            failed,
            cancelled,
            words: words.into(),
        },
    );
}

#[test]
fn a_delegated_landing_repairs_base_survives_into_the_landed_result_words() {
    for why in reasons() {
        let mut d = failed_round(why);
        delegate(&mut d, change::Delegate::Repair(why));
        let expected =
            skein_lib::bytes::concat(&[b"landing gate 0 failed, judged against ", &commit_text(&fixtures::BASE)]);
        assert_eq!(d.change(fixtures::CHANGE).expect("delegated repair").result.repairs.bytes, expected);
        terminal(&mut d, false, false, false, false, b"fixed the gate");
        fixtures::step_change(&mut d);
        fixtures::answer_next(&mut d, Ok(Answer::Commit(fixtures::HEAD)));
        fixtures::answer_next(&mut d, Ok(Answer::Commit(fixtures::BASE)));
        let mut pull = fixtures::open_pull();
        pull.state = api::State::Closed;
        pull.merged = Some(fixtures::BASE);
        fixtures::answer_next(&mut d, Ok(Answer::Pull(pull)));
        fixtures::finish_open(&mut d);
        let (call, op) = fixtures::next_call(&mut d);
        assert_eq!(op, api::Op::Read(Read::Reviews { number: 1, page: 1, skip: 0 }));
        fixtures::answer(&mut d, call, Ok(Answer::Reviews { reviews: Box::new([]), more: false, cut: false }));
        let row = d.change(fixtures::CHANGE).expect("landed repair");
        assert_eq!(row.change.state, change::State::Landed { merge: fixtures::BASE, since: env().wall });
        let words = row.result.reported(&row.report, &fixtures::BASE, LIMITS.client.answer_bytes);
        assert!(skein_lib::bytes::find(&words.bytes, &expected).is_some());
    }
}

#[test]
fn a_landing_repair_that_fails_for_good_names_its_base_in_the_hold_account() {
    for why in reasons() {
        let mut d = failed_round(why);
        delegate(&mut d, change::Delegate::Repair(why));
        terminal(&mut d, true, false, false, false, b"cannot fix the gate");
        let expected = skein_lib::bytes::concat(&[
            b"its repair, task 90, judged against ", &commit_text(&fixtures::BASE),
            b", failed for good: cannot fix the gate; resolve this failure, then release this change to ask for a fresh repair; or cancel it",
        ]);
        assert_eq!(
            d.change(fixtures::CHANGE).expect("failed repair").producer_failure.as_deref(),
            Some(expected.as_ref())
        );
        fixtures::step_change(&mut d);
        fixtures::answer_next(&mut d, Ok(Answer::Commit(fixtures::HEAD)));
        fixtures::answer_next(&mut d, Ok(Answer::Commit(fixtures::BASE)));
        fixtures::answer_next(&mut d, Ok(Answer::Pull(fixtures::open_pull())));
        fixtures::finish_open(&mut d);
        let (call, op) = fixtures::next_call(&mut d);
        assert_eq!(op, api::Op::Read(Read::Reviews { number: 1, page: 1, skip: 0 }));
        fixtures::answer(&mut d, call, Ok(Answer::Reviews { reviews: Box::new([]), more: false, cut: false }));
        let row = d.change(fixtures::CHANGE).expect("held repair");
        assert!(matches!(row.change.state, change::State::Held { why: change::Hold::Failed, .. }));
        assert_eq!(row.producer_failure.as_deref(), Some(expected.as_ref()));
    }
}

#[test]
fn an_unpinned_landing_repair_keeps_its_existing_account_and_result_words() {
    for keep_receipt in [false, true] {
        let why = reasons()[0];
        let mut d = failed_round(why);
        if keep_receipt {
            let key = d.landing.receipts.iter().next().expect("receipt").0.clone();
            d.landing.receipts.get_mut(&key).expect("receipt").base = None;
        } else {
            let key = d.landing.receipts.iter().next().expect("receipt").0.clone();
            d.landing.receipts.remove(&key).expect("receipt");
        }
        delegate(&mut d, change::Delegate::Repair(why));
        assert_eq!(d.change(fixtures::CHANGE).expect("repair").result.repairs.bytes.as_ref(), b"landing gate 0 failed");
        terminal(&mut d, true, false, false, false, b"cannot fix the gate");
        assert_eq!(d.change(fixtures::CHANGE).expect("failed repair").producer_failure.as_deref(), Some(
            b"its repair, task 90, failed for good: cannot fix the gate; resolve this failure, then release this change to ask for a fresh repair; or cancel it".as_slice()
        ));
    }
}

#[test]
fn a_named_base_does_not_change_no_change_flake_or_cancellation_accounts() {
    for (no_change, flake, cancelled, expected) in [
        (true, false, false, b"its repair, task 90, ended without making the required change: unchanged; resolve this failure, then release this change to ask for a fresh repair; or cancel it".as_slice()),
        (false, true, false, b"its repair, task 90, found only flakes: unchanged; release this change to run its gate once more, or cancel it".as_slice()),
        (false, false, true, b"its repair, task 90, was cancelled: unchanged".as_slice()),
    ] {
        let why = reasons()[0];
        let mut d = failed_round(why);
        delegate(&mut d, change::Delegate::Repair(why));
        terminal(&mut d, false, no_change, flake, cancelled, b"unchanged");
        assert_eq!(d.change(fixtures::CHANGE).expect("repair account").producer_failure.as_deref(), Some(expected));
    }
}
