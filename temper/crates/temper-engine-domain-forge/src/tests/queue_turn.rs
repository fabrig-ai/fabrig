//! A queue's exclusive turn and same-step handoff (temper/docs/design/domain/forge/queues.md).
use super::*;
use crate::tests::{adopted, domain, env, fixtures};
use skein_lib::Wall;

const OWNER: u64 = fixtures::CHANGE;
const SECOND: u64 = OWNER + 1;
const HEAD: [u8; 32] = fixtures::HEAD;
const BASE: [u8; 32] = fixtures::BASE;
const SINCE: Wall = Wall::from_nanos(95_000_000_000);
const REVIEW: u64 = 7;
const REVIEWER: u64 = 60;

fn gate() -> change::Gate {
    change::Gate {
        number: REVIEW,
        kind: change::GateKind::Agent,
        blocking: true,
        freshness: change::Freshness::Exact,
        eager: false,
    }
}

fn first() -> change::State {
    change::State::First { head: HEAD, ready_since: SINCE }
}
fn updating() -> change::State {
    change::State::Updating { from: HEAD, base: BASE, ready_since: SINCE }
}
fn gating() -> change::State {
    change::State::Gating { head: HEAD, asked: Box::new([]) }
}
fn changes_setup(state: change::State, second_since: Wall) -> Domain {
    let mut d = domain();
    d.repositories.insert(fixtures::opening_row().repository, Box::new(adopted())).unwrap();
    for (task, priority, since) in [(OWNER, 9_i32, SINCE), (SECOND, 1_i32, second_since)] {
        let mut row = fixtures::opening_row();
        row.task = task;
        row.change.task = task;
        row.priority = priority;
        row.pending = None;
        row.effect = change::EffectResult::None;
        row.delegate_status = change::Status::Unknown;
        row.pull = Some(1);
        row.change.ready_since = Some(since);
        row.change.owns_turn = task == OWNER;
        row.change.state =
            if task == OWNER { state.clone() } else { change::State::Queued { head: HEAD, ready_since: since } };
        d.changes.insert(task, row).unwrap();
    }
    d
}
fn pending(task: u64) -> PendingStep {
    PendingStep {
        task,
        entry: 100,
        follow_tree: List::with_capacity(env().limits.key_entries),
        stage: StepStage::Reviews(1),
        heard: change::Heard {
            delegate_running: false,
            delegate_live: false,
            review_request: None,
            start_conflict: None,
            lacking: None,
            spend: None,
            delegate: change::Status::Unknown,
            effect: change::EffectResult::None,
            released: false,
            cancelled: false,
        },
        gates: Box::new([]),
        queue_repair_active: false,
        kept: false,
        merge_job: false,
        pending: Box::new([]),
        branch: Some(HEAD),
        branch_fresh: true,
        base_tip: Some(BASE),
        pull: Some(fixtures::open_pull()),
        pull_ci: true,
        ci: change::Status::Passed,
        failed: crate::change_ci::Failed::new(Box::new([])),
        base_ci: change::Status::Passed,
        contains_base: change::Status::Passed,
        contains_head: change::Status::Failed,
        contains_catch_up: change::Status::Unknown,
        reviews: Box::new([]),
        reviews_complete: true,
        paths: Box::new([]),
        paths_whole: false,
        woken: false,
    }
}
fn follower_facts(d: &Domain) -> change::Facts {
    change_facts(d, &env(), d.changes.get(&SECOND).unwrap(), &pending(SECOND), BASE)
}

#[test]
fn a_turn_is_exclusive_through_update_checks_gates_and_merge() {
    for (state, second_since) in [
        (first(), SINCE),
        (updating(), SINCE),
        (change::State::Checking { head: HEAD }, SINCE),
        (gating(), SINCE),
        (change::State::Landing { head: HEAD }, SINCE),
        (updating(), Wall::EPOCH),
    ] {
        let mut d = changes_setup(state.clone(), second_since);
        assert!(!follower_facts(&d).first, "{state:?}, follower since {second_since:?}");
        decide_step(&mut d, &env(), pending(SECOND), &mut Queue::uncapped());
        let follower = d.changes.get(&SECOND).unwrap();
        assert_eq!(follower.change.state, change::State::Queued { head: HEAD, ready_since: second_since });
        assert!(!follower.change.owns_turn);
        let view = d.changes_view(5, env().wall, &env().limits).unwrap();
        assert_eq!(view.queues[0].changes[0].task, OWNER, "the owner is shown first at {state:?}");
    }
}

#[test]
fn stale_turn_flags_do_not_block_the_queue_and_base_repairs_go_first() {
    for (state, ending) in [
        (change::State::Held { was: Box::new(updating()), why: change::Hold::Failed }, None),
        (change::State::Ended, None),
        (change::State::NoChange { words: Box::new([]) }, None),
        (change::State::CancellingGates { head: HEAD }, None),
        (change::State::Resolving { requested: true, base: BASE }, None),
        (updating(), Some(crate::Ending::Cancelled)),
    ] {
        let mut d = changes_setup(state.clone(), SINCE);
        d.changes.get_mut(&OWNER).unwrap().ending = ending;
        assert!(follower_facts(&d).first, "stale flag at {state:?}");
    }
    let mut d = changes_setup(updating(), SINCE);
    d.changes.get_mut(&SECOND).unwrap().base_repair = true;
    assert!(follower_facts(&d).first, "a ready repair of the branch's CI precedes its owner");
    assert!(d.changes.get(&OWNER).unwrap().change.owns_turn, "the owner keeps its turn for after the repair");
    let view = d.changes_view(5, env().wall, &env().limits).unwrap();
    assert_eq!([view.queues[0].changes[0].task, view.queues[0].changes[1].task], [SECOND, OWNER]);
}

#[derive(Clone, Copy, Debug)]
enum Leave {
    PolicyHold,
    DelegateRefused,
    CiCancelsGates,
    FailedCheck(change::Status),
    ReviewRequest(change::Status),
    ReviewFailed,
    Kept,
    Conflict(bool),
    LandedRead,
    LandedOutcome,
    Cancelled,
    NoChange,
    Ended,
    TaskEnding,
    Veto,
    ReviewVeto,
    AddedGate,
}

fn leave(d: &mut Domain, how: Leave) -> Queue<Request> {
    let mut p = pending(OWNER);
    let mut out = Queue::uncapped();
    match how {
        Leave::DelegateRefused | Leave::ReviewFailed => {
            let row = d.changes.get_mut(&OWNER).unwrap();
            row.delegate = Some((REVIEWER, change::Delegate::Gate { number: REVIEW, head: HEAD }));
            row.change.gates = Box::new([gate()]);
            row.change.state = change::State::Gating { head: HEAD, asked: Box::new([REVIEW]) };
            let event = if matches!(how, Leave::DelegateRefused) {
                Event::DelegateRefused { task: OWNER, child: REVIEWER }
            } else {
                Event::DelegateResult {
                    task: OWNER,
                    child: REVIEWER,
                    status: change::Status::Failed,
                    no_change: false,
                    failed: true,
                    cancelled: false,
                    words: Box::from(b"review failed".as_slice()),
                }
            };
            step(d, &env(), event, &mut out);
            return out;
        }
        Leave::Ended => {
            end_change(d, &env(), OWNER, &mut out);
            return out;
        }
        Leave::TaskEnding => {
            step(d, &env(), Event::ChangeEnding { task: OWNER, ending: crate::Ending::Cancelled }, &mut out);
            return out;
        }
        Leave::LandedOutcome => {
            outcome_facts(
                d,
                &env(),
                p.entry,
                OWNER,
                skein_forge_client::Outcome::Made { made: skein_forge_client::Made::Merged(BASE), found: false },
                &mut out,
            );
            return out;
        }
        Leave::Veto | Leave::ReviewVeto => {
            let prior = d.changes.get(&OWNER).unwrap().change.clone();
            let entry = p.entry;
            decide_step(d, &env(), p, &mut out);
            veto_change(d, &env(), OWNER, entry, prior, matches!(how, Leave::Veto), &mut out);
            if matches!(how, Leave::ReviewVeto) {
                step(d, &env(), Event::ReviewRequested { task: OWNER, head: HEAD }, &mut out);
            }
            return out;
        }
        Leave::PolicyHold => d.changes.get_mut(&OWNER).unwrap().drift = Some(change::Hold::BranchMoved),
        Leave::CiCancelsGates => {
            p.ci = change::Status::Failed;
            d.changes.get_mut(&OWNER).unwrap().delegate =
                Some((REVIEWER, change::Delegate::Gate { number: REVIEW, head: HEAD }));
        }
        Leave::FailedCheck(contains) => {
            p.ci = change::Status::Failed;
            p.contains_base = contains;
        }
        Leave::ReviewRequest(contains) => {
            p.contains_base = contains;
            d.review_requests.insert(OWNER, HEAD).unwrap();
        }
        Leave::Kept => p.kept = true,
        Leave::Conflict(merge_job) => {
            d.changes.get_mut(&OWNER).unwrap().effect = change::EffectResult::Conflict;
            if merge_job {
                d.merge_kind = Some((4096, 4096));
                assert!(d.project_landing(
                    5,
                    change::Way::Queue,
                    Box::new([]),
                    temper_engine_domain_forge_landing::Project::DEFAULT,
                    crate::AfterLanding {
                        verdict: temper_engine_domain_forge_tiers::Settings {
                            retries: 2,
                            suspects: temper_engine_domain_forge_tiers::Suspects::Bisect
                        },
                        bound: Duration::from_secs(7200),
                    }
                ));
                assert!(d.project_merge_job(5, Box::from(b".temper/merge".as_slice())));
                p.merge_job = true;
            }
        }
        Leave::LandedRead => p.pull.as_mut().unwrap().merged = Some(BASE),
        Leave::Cancelled => p.heard.cancelled = true,
        Leave::NoChange => p.contains_head = change::Status::Passed,
        Leave::AddedGate => {
            d.changes.get_mut(&OWNER).unwrap().change.gates = Box::new([gate()]);
        }
    }
    decide_step(d, &env(), p, &mut out);
    out
}

#[test]
fn leaving_the_queue_ends_the_turn_and_wakes_the_next() {
    for (state, how) in [
        (first(), Leave::PolicyHold),
        (gating(), Leave::DelegateRefused),
        (gating(), Leave::CiCancelsGates),
        (change::State::Checking { head: HEAD }, Leave::FailedCheck(change::Status::Passed)),
        (change::State::Checking { head: HEAD }, Leave::FailedCheck(change::Status::Failed)),
        (change::State::Checking { head: HEAD }, Leave::FailedCheck(change::Status::Pending)),
        (first(), Leave::ReviewRequest(change::Status::Passed)),
        (first(), Leave::ReviewRequest(change::Status::Failed)),
        (first(), Leave::ReviewRequest(change::Status::Pending)),
        (gating(), Leave::ReviewFailed),
        (updating(), Leave::Kept),
        (change::State::Checking { head: HEAD }, Leave::Kept),
        (gating(), Leave::Kept),
        (updating(), Leave::Conflict(false)),
        (updating(), Leave::Conflict(true)),
        (change::State::Landing { head: HEAD }, Leave::LandedRead),
        (change::State::Landing { head: HEAD }, Leave::LandedOutcome),
        (first(), Leave::Cancelled),
        (first(), Leave::NoChange),
        (updating(), Leave::Ended),
        (updating(), Leave::TaskEnding),
        (first(), Leave::Veto),
        (first(), Leave::ReviewVeto),
        (first(), Leave::AddedGate),
        (gating(), Leave::AddedGate),
    ] {
        let mut d = changes_setup(state.clone(), SINCE);
        let out = leave(&mut d, how);
        assert_queue_marks(&out, how);
        let owner = d.changes.get(&OWNER).unwrap();
        assert!(!owner.change.owns_turn, "{state:?} / {how:?}: {:?}", owner.change.state);
        assert!(follower_facts(&d).first, "{state:?} / {how:?}: the next is elected");
        assert!(d.procedure_wakes.contains_key(&SECOND), "{state:?} / {how:?}: same-step wake");
    }
}

#[test]
fn a_settled_turn_marks_the_project_queue_and_status() {
    for how in [Leave::LandedRead, Leave::LandedOutcome, Leave::Ended] {
        let mut d = changes_setup(change::State::Landing { head: HEAD }, SINCE);
        let out = leave(&mut d, how);
        assert_queue_marks(&out, how);
        // Verification and later saves still affect the project's status after ownership is cleared.
        let row = d.changes.get(&OWNER).unwrap().clone();
        let mut saved = Queue::uncapped();
        save_change(&mut d, &mut saved, row);
        assert_queue_marks(&saved, how);
    }
}

fn assert_queue_marks(out: &Queue<Request>, how: Leave) {
    let subjects = [crate::changes_subject(5), crate::status_subject(5), crate::APPROVALS_SUBJECT];
    for subject in subjects {
        let mut marked = false;
        for request in out {
            if let Request::Marked { tasks } = request {
                marked |= tasks.contains(&subject);
            }
        }
        assert!(marked, "{how:?}: turn settlement invalidates {subject}");
    }
}

#[test]
fn changes_document_keeps_the_first_while_it_updates() {
    let mut d = changes_setup(updating(), Wall::from_nanos(96_000_000_000));
    let owner = d.changes.get_mut(&OWNER).unwrap();
    owner.change.ready_since = None;
    owner.change.since = Wall::from_nanos(99_000_000_000);
    assert!(!follower_facts(&d).first);
    decide_step(&mut d, &env(), pending(SECOND), &mut Queue::uncapped());
    assert!(matches!(d.changes.get(&SECOND).unwrap().change.state, change::State::Queued { .. }));
    let got = d.changes_view(5, env().wall, &env().limits).unwrap();
    let queue = &got.queues[0];
    assert_eq!(queue.changes.len(), 2);
    assert_eq!([queue.changes[0].task, queue.changes[1].task], [OWNER, SECOND]);
    assert_eq!(queue.changes[0].ready_since, SINCE);
    assert_eq!(queue.changes[0].checks_since, Some(Wall::from_nanos(99_000_000_000)));
    assert_eq!(queue.changes[1].checks_since, None);
}
