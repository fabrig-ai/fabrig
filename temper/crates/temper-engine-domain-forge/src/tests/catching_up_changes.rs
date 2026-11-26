//! Restart decisions wait for observations, not the start barrier (forge/catching-up.md, 4).
#![expect(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::wildcard_enum_match_arm,
    reason = "ordinary step stories inspect outputs and restored fixtures"
)]
use super::*;
use fixtures::{BASE, CHANGE, HEAD, OPEN};
use skein_forge::api::{self, Answer, Op, Read};
use temper_engine_domain_forge_change as c;
use temper_engine_domain_forge_tiers as p;

const FRESH: api::Commit = [9; 32];
const MARKER: &[u8] = b"temper/51";
const TIER_TASK: u64 = 61;
const DISPATCH: u64 = 21;

fn restore(d: &mut Domain, record: Stored) {
    outputs(d, Event::Restore { record });
}
fn watched(d: &mut Domain, branch: &[u8], commit: api::Commit) {
    let name = crate::tiers::name(REPO, branch);
    restore(d, Stored::BranchHead(BranchHead { name: name.clone(), commit, owned: false }));
    restore(d, Stored::Names { task: CHANGE, resources: Box::new([name]) });
}
fn begin(d: &mut Domain, e: &Env<Limits>) -> Queue<Request> {
    let mut out = Queue::uncapped();
    step(d, e, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic }, &mut out);
    step(d, e, Event::CatchUp, &mut out);
    out
}
fn requests(mut out: Queue<Request>) -> Box<[Request]> {
    let mut all = Vec::new();
    while let Some(request) = out.pop() {
        all.push(request);
    }
    all.into_boxed_slice()
}
fn decisions(out: &[Request]) -> Vec<c::Decision> {
    out.iter()
        .filter_map(|r| match r {
            Request::ChangeDecision { decision, .. } => Some(*decision),
            _ => None,
        })
        .collect()
}
fn held_branch(d: &mut Domain, e: &Env<Limits>, branch: &[u8]) -> Token {
    let (call, op) = fixtures::next_at(d, e);
    assert_eq!(op, Op::Read(Read::Branch { branch: branch.into() }));
    call
}
fn answered(d: &mut Domain, e: &Env<Limits>, call: Token, commit: api::Commit) -> Box<[Request]> {
    requests(fixtures::answer_at(d, e, call, Ok(Answer::Commit(commit))))
}
fn fresh_wake(out: &[Request], task: u64) {
    assert!(out.contains(&Request::RestartDone { stage: RestartStage::ReadAfresh }), "{out:?}");
    assert_eq!(out.iter().filter(|r| **r == Request::WakeProcedure { task }).count(), 1, "{out:?}");
}

#[test]
fn verified_changes_wait_for_fresh_reads_and_use_the_fresh_head() {
    let mut d = domain();
    restore(&mut d, Stored::Repository(adopted()));
    let mut row = fixtures::opening_row();
    row.pending = None;
    row.effect = c::EffectResult::None;
    row.change.state = c::State::Landed { merge: BASE, since: env().wall };
    row.change.gate_failures = Box::new([c::GateFailure { number: 7, count: 1 }]);
    restore(&mut d, Stored::Change(Box::new(row)));
    restore(
        &mut d,
        Stored::Verification(Verification {
            repository: REPO,
            commit: BASE,
            changes: Box::new([CHANGE]),
            tiers: Box::new([TierVerification { tier: b"browser".as_slice().into(), verdict: None, repaired: None }]),
        }),
    );
    watched(&mut d, MARKER, HEAD);
    begin(&mut d, &env());
    let call = held_branch(&mut d, &env(), MARKER);
    let before = outputs(&mut d, fixtures::change_step());
    assert!(decisions(&before).is_empty(), "{before:?}");
    assert!(d.reds.is_empty());
    assert_eq!(d.change(CHANGE).unwrap().change.gate_failures.as_ref(), &[c::GateFailure { number: 7, count: 1 }]);
    let fresh = answered(&mut d, &env(), call, FRESH);
    assert!(fresh.contains(&Request::RestartDone { stage: RestartStage::ReadAfresh }));
    assert!(domain::fresh_reads_in(&d));
    assert_eq!(
        decisions(&fresh),
        [c::Decision::Wait {
            until: Some(Wall::from_nanos(env().wall.as_nanos() + Duration::from_secs(7200).as_nanos()))
        }]
    );
    assert!(
        d.change(CHANGE).unwrap().change.gate_failures.is_empty(),
        "the decision used the newly read branch, not last_head"
    );
}

#[test]
fn taking_read_stalls_count_from_the_fresh_reads_not_the_restored_since() {
    for started_before in [true, false] {
        let mut e = env();
        e.wall = Wall::from_nanos(Duration::from_secs(3600).as_nanos());
        e.limits.change_policy.stall = Duration::from_secs(600);
        let mut d = domain_with_limits(&e.limits);
        fixtures::connect(&mut d);
        restore(&mut d, Stored::Repository(adopted()));
        let mut row = fixtures::opening_row();
        row.change.state = c::State::Taking { requested: true };
        row.change.since = Wall::from_nanos(e.wall.as_nanos() - Duration::from_secs(1800).as_nanos());
        row.pending = None;
        row.effect = c::EffectResult::None;
        restore(&mut d, Stored::Change(Box::new(row)));
        watched(&mut d, MARKER, HEAD);
        begin(&mut d, &e);
        let call = held_branch(&mut d, &e, MARKER);
        let mut out = Queue::uncapped();
        if started_before {
            step(&mut d, &e, fixtures::change_step(), &mut out);
            assert_eq!(d.steps.len(), 1);
        }
        // The change's own read remains unanswered throughout; only the restart's read finishes.
        let waiting = fixtures::advance(&mut d, &mut e, Duration::from_secs(1200));
        assert!(
            !requests(waiting)
                .iter()
                .any(|r| matches!(r, Request::ChangeDecision { decision: c::Decision::Hold(c::Hold::Stalled), .. }))
        );
        assert!(!matches!(d.change(CHANGE).unwrap().change.state, c::State::Held { .. }));
        let fresh = answered(&mut d, &e, call, HEAD);
        assert!(fresh.contains(&Request::RestartDone { stage: RestartStage::ReadAfresh }));
        assert!(!fresh.contains(&Request::WakeProcedure { task: CHANGE }), "the gather is already open");
        if started_before {
            assert_eq!(d.steps.len(), 1, "the in-flight gather keeps its entry and read");
        }
        if !started_before {
            step(&mut d, &e, fixtures::change_step(), &mut out);
            assert_eq!(d.steps.len(), 1);
        }
        let almost = fixtures::advance(&mut d, &mut e, Duration::from_secs(599));
        assert!(
            !requests(almost)
                .iter()
                .any(|r| matches!(r, Request::ChangeDecision { decision: c::Decision::Hold(c::Hold::Stalled), .. }))
        );
        let expired = requests(fixtures::advance(&mut d, &mut e, Duration::from_secs(1)));
        assert!(
            expired
                .iter()
                .any(|r| matches!(r, Request::ChangeDecision { decision: c::Decision::Hold(c::Hold::Stalled), .. })),
            "{expired:?}"
        );
    }
}

fn tier_runs(out: &[Request]) -> Vec<api::Commit> {
    out.iter()
        .filter_map(|r| match r {
            Request::TierRun { tip, .. } => Some(*tip),
            _ => None,
        })
        .collect()
}

#[test]
fn restored_tier_baselines_and_pending_landings_wait_for_their_fresh_tip() {
    for baseline in [false, true] {
        let live = tiers::dispatched();
        let mut saved = tiers::row(&live).clone();
        saved.record.run = None;
        saved.record.judged = Some(p::Judged { tip: BASE, key: BASE });
        saved.baseline = if baseline { Some(TierLanding { commit: HEAD, key: Some(HEAD) }) } else { None };
        saved.pending = if baseline { Box::new([]) } else { Box::new([TierLanding { commit: HEAD, key: Some(HEAD) }]) };
        saved.task = None;
        saved.requested = false;
        saved.flight = None;
        let mut d = tiers::restoring(None);
        restore(&mut d, Stored::Tier(saved));
        watched(&mut d, MARKER, HEAD);
        restore(
            &mut d,
            Stored::BranchHead(BranchHead { name: crate::tiers::name(REPO, b"main"), commit: HEAD, owned: false }),
        );
        let initial = requests(begin(&mut d, &env()));
        assert!(tier_runs(&initial).is_empty(), "{initial:?}");
        let mut held = Vec::new();
        for _ in 0_u8..2 {
            held.push(fixtures::next_at(&mut d, &env()));
        }
        let (main, _) =
            held.iter().find(|(_, op)| *op == Op::Read(Read::Branch { branch: b"main".as_slice().into() })).unwrap();
        let (marker, _) = held.iter().find(|(_, op)| *op == Op::Read(Read::Branch { branch: MARKER.into() })).unwrap();
        let mut before = Queue::uncapped();
        crate::tiers::start_due(&mut d, &env(), REPO, &mut before);
        let before = requests(before);
        assert!(tier_runs(&before).is_empty(), "{before:?}");
        let observed = answered(&mut d, &env(), *main, FRESH);
        assert!(tier_runs(&observed).is_empty(), "the new tip needs its content key");
        let (tree, op) = fixtures::next_at(&mut d, &env());
        assert_eq!(op, Op::Read(Read::Tree { object: FRESH, page: 1 }));
        let keyed = requests(fixtures::answer_at(
            &mut d,
            &env(),
            tree,
            Ok(Answer::Tree { entries: Box::new([]), more: false }),
        ));
        assert_eq!(tier_runs(&keyed), [FRESH]);
        assert!(!domain::fresh_reads_in(&d), "the branch's own read suffices while another fresh read is held");
        let all_fresh = answered(&mut d, &env(), *marker, HEAD);
        assert!(all_fresh.contains(&Request::RestartDone { stage: RestartStage::ReadAfresh }));
        assert!(tier_runs(&all_fresh).is_empty(), "the tier was requested once");
    }
}

fn tier_step(d: &mut Domain, entry: u64) -> Box<[Request]> {
    outputs(d, Event::StepTier { task: TIER_TASK, entry, heard: TierHeard { released: false, message_bytes: 4096 } })
}
fn no_tier_effects(out: &[Request]) {
    assert!(
        !out.iter().any(|r| matches!(
            r,
            Request::TierRun { .. }
                | Request::TierDecision { entry: Some(_), .. }
                | Request::TierDecision { decision: TierDecision::Done { .. } | TierDecision::Held { .. }, .. }
                | Request::Save { record: Stored::Entry { .. } | Stored::Red(_) | Stored::Verification(_), .. }
        )),
        "{out:?}"
    );
}

fn tier_look(d: &mut Domain, entry: u64, out: &mut Queue<Request>) {
    domain::scripted_outcome(d, &env(), entry, TIER_TASK, skein_forge_client::Outcome::Uncertain, out);
}

#[test]
fn tier_results_and_pending_reruns_wait_for_fresh_reads_and_each_restored_entry() {
    for result in [false, true] {
        for entries_first in [false, true] {
            let live = tiers::dispatched();
            let mut saved = tiers::row(&live).clone();
            saved.branch = if result { TierBranch::Absent } else { TierBranch::Made };
            saved.pending = Box::new([]);
            saved.baseline = None;
            let pending = api::JobAttempt { head: BASE, run: 71, job: 81, attempt: 1 };
            saved.flight.as_mut().unwrap().waiting = false;
            saved.flight.as_mut().unwrap().pending = Box::new([pending]);
            if result {
                saved.result = Some(Receipt {
                    report: ReceiptReport::Unset,
                    base: None,
                    jobs: Box::new([]),
                    logs: Box::new([]),
                    passed: true,
                    failures: None,
                    candidate: BASE,
                    ended: env().wall,
                    run: ReceiptRun { id: 71, number: 1, url: b"https://forge.example/run/71".as_slice().into() },
                });
            }
            let mut d = tiers::restoring(None);
            restore(&mut d, Stored::Tier(saved));
            watched(&mut d, MARKER, HEAD);
            // Both pending writes belong to the flight's task; looking at one is not enough.
            for number in [DISPATCH, DISPATCH + 1] {
                let mut entry = status(number, TIER_TASK);
                entry.effect.write = api::Write::Rerun { attempt: pending };
                entry.start = held_issue().start;
                entry.attempt = held_issue().attempt;
                restore(&mut d, Stored::Entry { entry, held: false });
            }
            begin(&mut d, &env());
            let mut fresh_calls = Vec::new();
            for _ in 0_u8..3 {
                if fresh_calls.len() == 2 {
                    break;
                }
                let (call, op) = fixtures::next_at(&mut d, &env());
                match op {
                    Op::Read(Read::Branch { branch }) => fresh_calls.push((call, branch)),
                    Op::Read(Read::Run { run: 71 }) => {
                        // The restored find runs beside the pass, and still owes its first look.
                        fixtures::answer_at(&mut d, &env(), call, Err(api::Error::Unavailable));
                    }
                    unexpected => panic!("branch watch or restored rerun find: {unexpected:?}"),
                }
            }
            assert_eq!(fresh_calls.len(), 2);
            let before = tier_step(&mut d, 40);
            no_tier_effects(&before);
            let mut all = Queue::uncapped();
            if entries_first {
                tier_look(&mut d, DISPATCH, &mut all);
                no_tier_effects(&tier_step(&mut d, 40));
                tier_look(&mut d, DISPATCH + 1, &mut all);
                no_tier_effects(&tier_step(&mut d, 40));
            }
            for (call, branch) in fresh_calls {
                let fresh = answered(&mut d, &env(), call, if branch.as_ref() == b"main" { BASE } else { HEAD });
                for request in fresh {
                    all.push(request);
                }
            }
            assert!(domain::fresh_reads_in(&d));
            if !entries_first {
                no_tier_effects(&tier_step(&mut d, 40));
                tier_look(&mut d, DISPATCH, &mut all);
                no_tier_effects(&tier_step(&mut d, 40));
                tier_look(&mut d, DISPATCH + 1, &mut all);
            }
            assert!(
                all.iter().any(|r| *r == Request::WakeProcedure { task: TIER_TASK }),
                "the last prerequisite wakes the task"
            );
            let after = tier_step(&mut d, 40);
            if result {
                assert!(tiers::row(&d).posting.is_some(), "the restored result is now judged: {after:?}");
            } else {
                assert!(
                    after.iter().any(|r| matches!(r, Request::Save { record: Stored::Entry { entry, .. }, .. }
                            if entry.number == 40 && matches!(entry.effect.write, api::Write::Rerun { .. }))),
                    "{after:?}"
                );
            }
        }
    }
}

#[test]
fn a_follow_checks_pending_writes_wait_for_each_entries_first_look_only() {
    use crate::follow_check::{self as check, Action};
    let (mut d, mut row) = follow_check::configured();
    follow_check::keyed(&mut d, &mut row);
    check::write(&d, &mut row, OPEN).unwrap();
    let pending = api::JobAttempt { head: HEAD, run: 71, job: 81, attempt: 1 };
    let flight = row.follow_check.as_mut().unwrap().flight.as_mut().unwrap();
    flight.waiting = false;
    flight.pending = Box::new([pending]);
    restore(&mut d, Stored::Change(Box::new(row.clone())));
    watched(&mut d, MARKER, HEAD);
    for number in [OPEN, OPEN + 1] {
        let mut entry = status(number, CHANGE);
        entry.effect.write = api::Write::Rerun { attempt: pending };
        restore(&mut d, Stored::Entry { entry, held: false });
    }
    begin(&mut d, &env());
    let (_call, _read) = fixtures::next_at(&mut d, &env());
    let mut out = Queue::uncapped();
    assert!(matches!(check::advance(&mut d, &env(), &mut row, HEAD, &mut out), Action::Wait));
    domain::scripted_outcome(&mut d, &env(), OPEN, CHANGE, skein_forge_client::Outcome::Uncertain, &mut out);
    assert!(matches!(check::advance(&mut d, &env(), &mut row, HEAD, &mut out), Action::Wait));
    out.clear();
    domain::scripted_outcome(
        &mut d,
        &env(),
        OPEN + 1,
        CHANGE,
        skein_forge_client::Outcome::Made { made: skein_forge_client::Made::Set, found: true },
        &mut out,
    );
    assert!(out.iter().any(|r| *r == Request::WakeProcedure { task: CHANGE }));
    assert!(!domain::fresh_reads_in(&d));
    assert!(
        matches!(check::advance(&mut d, &env(), &mut row, HEAD, &mut out), Action::Write),
        "only the flight's entries are needed"
    );
}

#[test]
fn restored_follows_wait_for_their_source_read_and_queue_only_the_fresh_tip() {
    let (mut d, _row) = follow_check::configured();
    let mut follow = d.follow_row(2, 1).unwrap().clone();
    follow.heard = Some(BASE);
    let source = follow.source;
    restore(&mut d, Stored::Follow(follow));
    restore(
        &mut d,
        Stored::BranchHead(BranchHead { name: crate::tiers::name(source, b"main"), commit: HEAD, owned: false }),
    );
    watched(&mut d, MARKER, HEAD);
    let started = requests(begin(&mut d, &env()));
    assert!(!started.iter().any(|r| matches!(r, Request::FollowRun { .. })), "{started:?}");
    assert!(!follows::waiting(&d), "the restored tip cannot start a range");
    let mut reads = Vec::new();
    for _ in 0_u8..2 {
        reads.push(fixtures::next_at(&mut d, &env()));
    }
    let (source_call, _) =
        reads.iter().find(|(_, op)| *op == Op::Read(Read::Branch { branch: b"main".as_slice().into() })).unwrap();
    let (marker_call, _) =
        reads.iter().find(|(_, op)| *op == Op::Read(Read::Branch { branch: MARKER.into() })).unwrap();
    let mut before = Queue::uncapped();
    follows::resume(&mut d, &env(), &mut before);
    assert!(before.is_empty());
    assert!(!follows::waiting(&d));
    assert!(d.follow_row(2, 1).unwrap().queue.is_empty());
    let fresh = answered(&mut d, &env(), *source_call, FRESH);
    assert!(fresh.contains(&Request::FollowRun { project: 2, follow: 1 }), "{fresh:?}");
    assert!(!domain::fresh_reads_in(&d));
    let mut queued = false;
    for _ in 0_u8..3 {
        let (call, op) = fixtures::next_at(&mut d, &env());
        let answer = match op {
            Op::Read(Read::Compare { before, after }) => {
                assert_eq!(after, FRESH);
                Answer::Compare {
                    before,
                    after,
                    files: Box::new([b"src/lib.rs".as_slice().into()]),
                    ahead: 1,
                    behind: 0,
                    changed: 1,
                }
            }
            Op::Read(Read::Commits { before, after, page: 1 }) => {
                assert_eq!((before, after), (BASE, FRESH));
                queued = true;
                Answer::Commits {
                    commits: Box::new([api::CommitMessage {
                        id: FRESH,
                        parents: Box::new([BASE]),
                        message: b"fresh".as_slice().into(),
                    }]),
                    more: false,
                }
            }
            _ => panic!("follow's range: {op:?}"),
        };
        fixtures::answer(&mut d, call, Ok(answer));
        if queued {
            break;
        }
    }
    assert!(queued);
    let follow = d.follow_row(2, 1).unwrap();
    assert_eq!(follow.heard, Some(FRESH));
    assert_eq!(follow.queue.len(), 1);
    assert_eq!(follow.queue[0].after, FRESH);
    let finished = answered(&mut d, &env(), *marker_call, HEAD);
    assert!(finished.contains(&Request::RestartDone { stage: RestartStage::ReadAfresh }));
}

const GOAL_BRANCH: &[u8] = b"temper/7/g7";
fn restored_goal(d: &mut Domain) {
    let name = crate::tiers::name(REPO, GOAL_BRANCH);
    restore(
        d,
        Stored::Hold(Hold {
            name: name.clone(),
            task: 7,
            writer: None,
            beside: None,
            design_writers: Box::new([]),
            drift: None,
        }),
    );
    restore(d, Stored::BranchHead(BranchHead { name, commit: HEAD, owned: true }));
}

fn change_at_head(d: &mut Domain, head: api::Commit) -> Box<[Request]> {
    outputs(d, fixtures::change_step());
    let mut all = Vec::new();
    for _ in 0_u8..12 {
        let (call, op) = fixtures::next_at(d, &env());
        let mut done = false;
        let answer = match op {
            Op::Read(Read::Branch { branch }) => {
                Answer::Commit(if branch.as_ref() == GOAL_BRANCH { head } else { BASE })
            }
            Op::Read(Read::Pull { .. } | Read::PullFor { .. }) => {
                let mut pull = fixtures::open_pull();
                pull.head = GOAL_BRANCH.into();
                pull.commit = head;
                pull.ci = api::Ci::Pending;
                Answer::Pull(pull)
            }
            Op::Read(Read::Checks { commit }) => {
                fixtures::checks(if commit == head { api::Check::Pending } else { api::Check::Passed })
            }
            Op::Read(Read::Commits { .. }) => Answer::Commits { commits: Box::new([]), more: false },
            Op::Read(Read::Reviews { .. }) => {
                done = true;
                Answer::Reviews { reviews: Box::new([]), more: false, cut: false }
            }
            _ => panic!("change gather: {op:?}"),
        };
        all.extend(fixtures::answer(d, call, Ok(answer)));
        if done {
            return all.into_boxed_slice();
        }
    }
    panic!("the coherent gather ends at its reviews")
}

#[test]
fn moved_branch_facts_wait_and_compare_against_the_fresh_confirmed_head() {
    for still_moved in [false, true] {
        let mut d = fixtures::connected_domain();
        restore(&mut d, Stored::Repository(adopted()));
        restored_goal(&mut d);
        let mut row = fixtures::opening_row();
        row.branch = GOAL_BRANCH.into();
        row.change.state = c::State::Checking { head: HEAD };
        row.pending = None;
        row.effect = c::EffectResult::None;
        restore(&mut d, Stored::Change(Box::new(row)));
        watched(&mut d, MARKER, HEAD);
        begin(&mut d, &env());
        let goal_read = held_branch(&mut d, &env(), GOAL_BRANCH);
        let fresh_call = held_branch(&mut d, &env(), MARKER);
        answered(&mut d, &env(), goal_read, if still_moved { HEAD } else { FRESH });
        let before = change_at_head(&mut d, FRESH);
        assert!(decisions(&before).is_empty(), "{before:?}");
        assert_eq!(d.change(CHANGE).unwrap().change.state, c::State::Checking { head: HEAD });
        let fresh = answered(&mut d, &env(), fresh_call, HEAD);
        assert!(fresh.contains(&Request::RestartDone { stage: RestartStage::ReadAfresh }));
        assert!(!fresh.contains(&Request::WakeProcedure { task: CHANGE }), "the completed gather resumes in place");
        assert_eq!(decisions(&fresh).contains(&c::Decision::Hold(c::Hold::BranchMoved)), still_moved, "{fresh:?}");
        if !still_moved {
            assert_eq!(d.change(CHANGE).unwrap().change.last_head, Some(FRESH));
        }
    }
}

#[test]
fn a_changes_goal_base_waits_for_all_fresh_reads_even_after_its_making_read() {
    let mut d = fixtures::connected_domain();
    restore(&mut d, Stored::Repository(adopted()));
    restored_goal(&mut d);
    let mut row = fixtures::opening_row();
    row.base = GOAL_BRANCH.into();
    row.pending = None;
    row.effect = c::EffectResult::None;
    restore(&mut d, Stored::Change(Box::new(row)));
    watched(&mut d, MARKER, HEAD);
    begin(&mut d, &env());
    let mut reads = Vec::new();
    for _ in 0_u8..2 {
        reads.push(fixtures::next_at(&mut d, &env()));
    }
    let (goal, _) = reads.iter().find(|(_, op)| *op == Op::Read(Read::Branch { branch: GOAL_BRANCH.into() })).unwrap();
    let (marker, _) = reads.iter().find(|(_, op)| *op == Op::Read(Read::Branch { branch: MARKER.into() })).unwrap();
    answered(&mut d, &env(), *goal, HEAD);
    outputs(&mut d, fixtures::change_step());
    let own = held_branch(&mut d, &env(), MARKER);
    let waiting = answered(&mut d, &env(), own, HEAD);
    assert!(decisions(&waiting).iter().any(|d| matches!(d, c::Decision::Wait { .. })), "{waiting:?}");
    assert!(d.steps.is_empty(), "no base read is made from the restored base fact");
    let fresh = answered(&mut d, &env(), *marker, HEAD);
    fresh_wake(&fresh, CHANGE);
    outputs(&mut d, fixtures::change_step());
    let own = held_branch(&mut d, &env(), MARKER);
    answered(&mut d, &env(), own, HEAD);
    held_branch(&mut d, &env(), GOAL_BRANCH);
}

#[test]
fn held_landing_judgements_wait_and_are_woken_by_fresh_reads() {
    let mut d = domain();
    restore(&mut d, Stored::Repository(adopted()));
    let mut row = fixtures::opening_row();
    row.change.state = c::State::Landing { head: HEAD };
    let evidence = ChangeEvidence {
        behind: None,
        readback_owed: false,
        nearest_write: None,
        touched: Touched::UNKNOWN,
        prior: row.change.clone(),
        head: Some(HEAD),
        base_tip: BASE,
        contains_base: c::Status::Passed,
        ci: c::Status::Passed,
        gates: Box::new([]),
        reviews: Box::new([]),
        reviews_complete: true,
        size: None,
    };
    restore(&mut d, Stored::Change(Box::new(row)));
    watched(&mut d, MARKER, HEAD);
    begin(&mut d, &env());
    let call = held_branch(&mut d, &env(), MARKER);
    assert_eq!(d.judge_landing(5, 5, 0, CHANGE, HEAD, &evidence, &[]), Some(JudgeVerdict::Wait));
    let fresh = answered(&mut d, &env(), call, HEAD);
    fresh_wake(&fresh, CHANGE);
    assert_eq!(d.judge_landing(5, 5, 0, CHANGE, HEAD, &evidence, &[]), Some(JudgeVerdict::Met));
}
