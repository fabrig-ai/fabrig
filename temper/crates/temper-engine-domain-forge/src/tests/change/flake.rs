//! A repair's flake regates without release at unchanged content (landing-regate.md, 3–5).
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

pub(super) fn change_step_at(d: &mut Domain, head: api::Commit) -> change::Decision {
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
        let clock = env();
        let out = fixtures::answer_at(d, &clock, call, Ok(answer));
        requests = collected(fixtures::ordinary_method_probes(d, &clock, out));
    }
    panic!("change decides")
}

pub(super) fn restart(d: &Domain, key: &ReceiptKey) -> Domain {
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
    outputs(&mut restored, Event::CatchUp);
    fixtures::settle_home_reads(&mut restored, &env(), fixtures::BASE);

    restored.landing.main_keys.insert((REPO, b"gate".as_slice().into()), (fixtures::BASE, key.key)).unwrap();
    restored
}

pub(super) fn next_landing(d: &mut Domain, key: &ReceiptKey) {
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

pub(super) fn release_legacy(d: &mut Domain, key: &ReceiptKey, flake: bool) {
    let row = d.changes.get_mut(&fixtures::CHANGE).unwrap();
    row.change.state = change::State::Held {
        was: Box::new(row.change.state.clone()),
        hold: change::Hold::new(change::Cause::RedStands, Wall::EPOCH),
    };
    row.delegate = None;
    row.delegate_status = change::Status::Passed;
    row.delegate_flake = flake;
    row.delegate_no_test = !flake;
    *d = restart(d, key);
    outputs(d, Event::ReleaseChange { task: fixtures::CHANGE, release: None });
    assert_eq!(change_step_at(d, fixtures::HEAD), change::Decision::Ready);
    let row = d.change(fixtures::CHANGE).unwrap();
    assert_eq!(row.regate, Some(Regate { receipt: key.clone(), run: 41, taken: None }));
    assert_eq!(row.change.last_head, Some(fixtures::HEAD));
    assert_eq!(row.change.repairs, 1);
    assert_eq!(row.repair_tasks.as_ref(), [52]);
    assert_eq!(row.delegate, None);
    next_landing(d, key);
}

fn regate(d: &mut Domain, key: &ReceiptKey) {
    verdict_terminal(d, 52, WORDS);
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
        skein_lib::bytes::find(&row.result.repairs.bytes, b"successful repair task 52: changed nothing (flake): ")
            .is_some()
    );
    assert!(row.result.repairs.bytes.ends_with(WORDS));
    next_landing(d, key);
}

#[test]
fn an_unchanged_flake_regates_across_restart_and_proves_green_without_a_commit() {
    let (mut d, key, run) = repairing_flake();
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
fn a_previous_flake_hold_releases_into_one_regate_whose_red_stands() {
    let (mut d, key, mut run) = repairing_flake();
    release_legacy(&mut d, &key, true);
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
    assert_eq!(held.producer_failure.as_deref(), Some(b"its red at 030303030303 stands: repair task 52 changed nothing, and the same content failed again: test a::t x; amend this change to ask a fresh repair, measure it again, or cancel it".as_slice()));
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
fn a_flake_without_its_receipt_holds_and_keeps_its_words() {
    let (mut d, key, _) = repairing_flake();
    d.landing.receipts.remove(&key);
    let words = "test a::t x passed again; ".repeat(100);
    verdict_terminal(&mut d, 52, words.as_bytes());
    assert_eq!(change_step_at(&mut d, fixtures::HEAD), change::Decision::Hold(change::Cause::RedStands));
    let row = d.change(fixtures::CHANGE).unwrap();
    assert_eq!(row.producer_failure.as_deref(), Some(b"its red at 030303030303 stands: repair task 52 changed nothing, and the same content can't be checked again: the landing no longer keeps its failed receipt; amend this change to ask a fresh repair, or cancel it".as_slice()));
    assert!(row.result.repairs.bytes.ends_with(words.as_bytes()));
    assert_eq!(row.regate, None);
    assert_eq!(row.repair_tasks.as_ref(), [52]);
}

#[test]
fn a_flake_at_a_moved_branch_is_a_push_and_clears_the_regate() {
    for already_read in [false, true] {
        let (mut d, key, _) = repairing_flake();
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
        verdict_terminal(&mut d, 52, WORDS);
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

#[test]
fn plain_no_change_proceeds_with_its_receipt_and_account() {
    let (mut d, key, _) = repairing_flake();
    outputs(
        &mut d,
        Event::DelegateResult {
            task: fixtures::CHANGE,
            child: 52,
            ended_at: None,
            status: change::Status::Passed,
            no_change: true,
            flake: false,
            no_test: false,
            failed: false,
            lost: false,
            cancelled: false,
            words: b"Nothing to change".as_slice().into(),
        },
    );
    assert_eq!(change_step_at(&mut d, fixtures::HEAD), change::Decision::Ready);
    let row = d.change(fixtures::CHANGE).unwrap();
    assert_eq!(row.regate, Some(Regate { receipt: key, run: 41, taken: None }));
    assert_eq!(
        row.result.repairs.bytes.as_ref(),
        b"\nsuccessful repair task 52: changed nothing (no change): Nothing to change"
    );
    assert_eq!(row.change.repairs, 1);
}

#[test]
fn another_base_key_can_ask_an_ordinary_repair() {
    let (mut d, key, run) = repairing_flake();
    regate(&mut d, &key);
    // Simulate the keyed proof made on a different base, at the same member head.
    let other = ReceiptKey { key: [22; 32], ..key.clone() };
    let older = d.landing.receipts.get(&key).unwrap().clone();
    d.landing
        .receipts
        .insert(
            other.clone(),
            Receipt { base: Some([21; 32]), run: ReceiptRun { id: 56, number: 56, ..older.run.clone() }, ..older },
        )
        .unwrap();
    d.landing.rows.get_mut(&61).unwrap().landing.members[0].gates[0].proof = landing::Proof::Keyed { key: other.key };
    assert_eq!(
        landing_routes::step_regate(&mut d, 61, 91, &run),
        landing::Decision::Leave { member: 0, why: landing::Leave::Failed { gate: 0, run: 56 } }
    );
    assert_eq!(change_step_at(&mut d, fixtures::HEAD), change::Decision::Delegate(change::Delegate::Repair(why())));
    let row = d.change(fixtures::CHANGE).unwrap();
    assert_eq!(row.change.repairs, 2);
    assert_eq!(row.regate, None);
}

#[test]
fn a_red_stands_account_cuts_whole_keys_before_its_way_out() {
    let keys = [b"test a::t x".as_slice().into(), Box::from([b'x'; 512]), b"test b::t x".as_slice().into()];
    let words = recheck::stands_again(fixtures::HEAD, 52, &keys);
    assert!(words.len() <= skein_lib::usize_of(jig_domain_tasks::HOLD_ACCOUNT_BYTES));
    assert_eq!(&*words, "its red at 030303030303 stands: repair task 52 changed nothing, and the same content failed again: test a::t x …; amend this change to ask a fresh repair, measure it again, or cancel it".as_bytes());
}
