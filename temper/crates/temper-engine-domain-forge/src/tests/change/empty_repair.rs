//! Fresh reads and forge capability for empty repairs (change/repairs.md).
use super::*;
use skein_forge::api::{
    Answer, Check, Ci, CommitMessage, Conclusion, Error, JobAttempt, Op, Read, Run, RunState, Status, Write,
};
use temper_engine_domain_forge_change as change;

const REPAIR: u64 = 52;
const ATTEMPT: JobAttempt = JobAttempt { head: fixtures::HEAD, run: 7, job: 41, attempt: 1 };
const UNCHECKED: &[u8] = b"its red at 030303030303 stands: repair task 52 changed nothing, and the same content can't be checked again: no failed check names a job the forge can run again; release this change to ask a fresh repair, or cancel it";

fn checked(kind: Kind, late: bool, no_change: bool) -> (Domain, change::Decision, u32) {
    checked_repair(kind, late, no_change, false, change::Repair::Ci, None, Some(ATTEMPT))
}

fn ci_check(context: &[u8], check: Check, job: Option<JobAttempt>) -> Status {
    Status { at: Time::ZERO, context: context.into(), check, description: Box::new([]), url: Box::new([]), job }
}

fn connected(kind: Kind) -> Domain {
    let mut d = fixtures::connected_domain();
    d.configured.get_mut(&1).expect("forge configuration").kind = kind;
    let mut repository = adopted();
    repository.protection = ci_protection(false);
    outputs(&mut d, Event::Restore { record: Stored::Repository(repository) });
    d
}

fn checked_repair(
    kind: Kind,
    late: bool,
    no_change: bool,
    flake: bool,
    why: change::Repair,
    advanced: Option<[u8; 32]>,
    job: Option<JobAttempt>,
) -> (Domain, change::Decision, u32) {
    let mut d = connected(kind);
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut d, Event::CatchUp);

    let mut row = fixtures::opening_row();
    row.pending = None;
    row.pull = Some(1);
    row.effect = change::EffectResult::None;
    row.change.state = change::State::Repairing { requested: true, why };
    row.change.last_head = Some(fixtures::HEAD);
    row.change.repairs = 1;
    row.repair_tasks = Box::new([REPAIR]);
    row.delegate = Some((REPAIR, change::Delegate::Repair(why)));
    row.delegate_head = row.change.last_head;
    row.delegate_status = change::Status::Pending;
    outputs(&mut d, Event::Change { row });
    if !late {
        if flake {
            verdict_terminal(&mut d, 52, b"test a::t x passed again");
        } else {
            producer_terminal(&mut d, no_change);
        }
    }
    fixtures::step_change(&mut d);
    let mut branches = 0_u32;
    for _ in 0_u32..30 {
        let (call, op) = fixtures::next_call(&mut d);
        let terminal =
            late && matches!(op, Op::Read(Read::Checks { commit }) if commit == fixtures::HEAD) && branches == 1;
        let answer = match op {
            Op::Read(Read::Branch { branch }) => {
                if branch.as_ref() == b"main" {
                    Answer::Commit(fixtures::BASE)
                } else {
                    branches = branches.saturating_add(1);
                    Answer::Commit(if branches > 1 { advanced.unwrap_or(fixtures::HEAD) } else { fixtures::HEAD })
                }
            }
            Op::Read(Read::Pull { .. }) => Answer::Pull(skein_forge::api::Pull {
                commit: if branches > 1 { advanced.unwrap_or(fixtures::HEAD) } else { fixtures::HEAD },
                ci: Ci::Failed,
                ..fixtures::open_pull()
            }),
            Op::Read(Read::Checks { commit }) if commit == fixtures::HEAD => {
                Answer::Checks(Box::new([ci_check(b"ci/test", Check::Failed, job)]))
            }
            Op::Read(Read::Checks { .. }) => fixtures::checks(Check::Passed),
            Op::Read(Read::Run { .. }) => Answer::Run(fixtures::failed_run(job.expect("a named job"), Some(1))),
            Op::Read(Read::Commits { before, .. }) => Answer::Commits {
                commits: if before == fixtures::HEAD {
                    Box::new([])
                } else {
                    Box::new([CommitMessage { id: fixtures::HEAD, parents: Box::new([]), message: Box::new([]) }])
                },
                more: false,
            },
            Op::Read(Read::Reviews { .. }) => Answer::Reviews { reviews: Box::new([]), more: false, cut: false },
            Op::Read(Read::Compare { before, after }) => {
                Answer::Compare { before, after, ahead: 1, behind: 0, files: Box::new([]), changed: 0 }
            }
            other @ (Op::Read(_) | Op::Write(_)) => panic!("unexpected {other:?}"),
        };
        if terminal {
            if flake {
                verdict_terminal(&mut d, 52, b"test a::t x passed again");
            } else {
                producer_terminal(&mut d, no_change);
            }
        }
        let answered = fixtures::answer(&mut d, call, Ok(answer));
        if let Some(decision) = decision(&d, &answered) {
            return (d, decision, branches);
        }
    }
    panic!("empty repair decided")
}

#[test]
fn empty_repairs_rerun_read_jobs_only_on_a_forge_that_writes_reruns() {
    for (kind, no_change, job) in [
        (Kind::Forgejo, true, Some(ATTEMPT)),
        (Kind::GitHub, true, Some(ATTEMPT)),
        (Kind::Forgejo, false, Some(ATTEMPT)),
        (Kind::GitHub, false, Some(ATTEMPT)),
        (Kind::Forgejo, true, None),
    ] {
        let (d, decision, _) = checked_repair(kind, false, no_change, false, change::Repair::Ci, None, job);
        let row = d.change(fixtures::CHANGE).expect("change");
        assert!(row.delegate_words.is_none());
        assert!(
            row.result
                .repairs
                .bytes
                .ends_with(b"successful repair task 52: changed nothing (no change): Nothing applies")
        );
        if kind == Kind::Forgejo && job.is_some() {
            assert_eq!(decision, change::Decision::Effect(change::Effect::Rerun { head: fixtures::HEAD }));
            assert_eq!(row.change.state, change::State::Checking { head: fixtures::HEAD });
            assert_eq!(row.change.repairs, 1);
            assert_eq!(
                d.entries.get(&row.pending.expect("entry")).expect("entry").entry.effect.write,
                Write::Rerun { attempt: ATTEMPT }
            );
            assert_eq!(
                row.effect_account.as_ref().expect("account").words.as_ref(),
                b"ran job 41 of ci/test again at 030303030303: a repair changed nothing"
            );
            assert_eq!(row.ci_recheck, Some(CiRecheck { job: ATTEMPT, taken: None }));
        } else {
            assert_eq!(decision, change::Decision::Hold(change::Hold::RedStands));
            assert_eq!(row.change.repairs, 1);
            assert_eq!(row.change.updates, 0);
            assert!(row.pending.is_none());
            assert!(row.ci_recheck.is_none());
            assert_eq!(row.producer_failure.as_deref(), Some(UNCHECKED));
        }
    }
}

#[test]
fn rerun_job_prefers_runner_then_requires_a_read_job_at_this_head() {
    let attempt = JobAttempt { head: fixtures::HEAD, run: 7, job: 41, attempt: 1 };
    let check = Status {
        at: Time::ZERO,
        context: b"ci/test".as_slice().into(),
        check: Check::Failed,
        description: Box::new([]),
        url: Box::new([]),
        job: Some(attempt),
    };
    let mut failed = change_ci::Failed::new(Box::new([
        check.clone(),
        Status { job: Some(JobAttempt { job: 42, ..attempt }), ..check.clone() },
    ]));
    assert_eq!(failed.next(fixtures::HEAD), Some(Read::Run { run: 7 }));
    assert_eq!(failed.job(fixtures::HEAD), None, "an unread job cannot run again");
    let mut run = fixtures::failed_run(attempt, Some(1));
    let runner = fixtures::failed_run(JobAttempt { job: 42, ..attempt }, Some(0));
    run.jobs =
        Box::new([run.jobs.first().expect("workflow job").clone(), runner.jobs.first().expect("runner job").clone()]);
    failed.read(7, Ok(Answer::Run(run)));
    assert_eq!(failed.job(fixtures::HEAD), Some(JobAttempt { job: 42, ..attempt }));
    for mismatch in 0_u32..5 {
        let mut failed = change_ci::Failed::new(Box::new([check.clone()]));
        assert_eq!(failed.next(fixtures::HEAD), Some(Read::Run { run: 7 }));
        let mut run = fixtures::failed_run(attempt, Some(1));
        match mismatch {
            0 => {}
            1 => run.commit = fixtures::BASE,
            2 => run.id = 8,
            3 => run.jobs.first_mut().expect("job").id = 42,
            4 => run.jobs.first_mut().expect("job").attempt = 2,
            _ => unreachable!("five identity cases"),
        }
        failed.read(7, Ok(Answer::Run(run)));
        assert_eq!(failed.job(fixtures::HEAD), if mismatch == 0 { Some(attempt) } else { None });
        assert_eq!(failed.job(fixtures::BASE), None, "a pin at another head cannot run again");
    }
}

#[test]
fn a_repair_terminal_during_ci_gathering_reads_its_branch_after_the_end() {
    let (_, decision, branches) = checked(Kind::Forgejo, true, true);
    assert_eq!(decision, change::Decision::Effect(change::Effect::Rerun { head: fixtures::HEAD }));
    assert_eq!(branches, 2, "the empty-repair decision uses a branch read after its terminal");
}

#[test]
fn a_flake_terminal_during_gathering_uses_the_post_terminal_branch() {
    let advanced = [9; 32];
    let why = change::Repair::LandingAt { gate: 0, candidate: fixtures::HEAD, below: None };
    let (d, decision, branches) = checked_repair(Kind::Forgejo, true, false, true, why, Some(advanced), Some(ATTEMPT));
    assert_eq!(branches, 2, "flake verdicts require a branch read sent after the end");
    assert_eq!(d.change(fixtures::CHANGE).unwrap().change.last_head, Some(advanced));
    assert_ne!(decision, change::Decision::Hold(change::Hold::Flake));
}

#[test]
fn a_review_repair_delivery_during_gathering_reads_its_branch_after_the_end() {
    for why in [change::Repair::Gate(1), change::Repair::Review { head: fixtures::HEAD }] {
        let advanced = [9; 32];
        let (d, decision, branches) =
            checked_repair(Kind::Forgejo, true, false, false, why, Some(advanced), Some(ATTEMPT));
        assert_eq!(branches, 2, "{why:?}: a delivery needs a branch read after its terminal");
        assert_eq!(d.change(fixtures::CHANGE).expect("delivered change").change.last_head, Some(advanced));
        assert_ne!(decision, change::Decision::Hold(change::Hold::Failed));
    }
}

// Read the hot row written by the deciding step, not just the live projection.
fn decision(d: &Domain, requests: &[Request]) -> Option<change::Decision> {
    let mut decided = None;
    let mut kept = None;
    for request in requests {
        if let Request::ChangeDecision { decision, .. } = request {
            decided = Some(*decision);
        }
        if let Request::Save { record: Stored::ChangeHot(row), .. } = request
            && row.task == fixtures::CHANGE
        {
            kept = Some(row);
        }
    }
    let decision = decided?;
    if let Some(saved) = kept {
        assert_eq!(saved.ci_recheck, d.change(fixtures::CHANGE).unwrap().ci_recheck);
    } else {
        assert!(d.change(fixtures::CHANGE).unwrap().ci_recheck.is_none(), "the deciding step saves the recheck");
    }
    Some(decision)
}

fn restored(d: &Domain) -> Domain {
    let mut next = connected(Kind::Forgejo);
    let (cold, hot) = d.change(fixtures::CHANGE).unwrap().clone().split();
    outputs(&mut next, Event::Restore { record: Stored::ChangeCold(Box::new(cold)) });
    outputs(&mut next, Event::Restore { record: Stored::ChangeHot(Box::new(hot)) });
    outputs(&mut next, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut next, Event::CatchUp);
    next
}

fn settle_rerun(d: &mut Domain, attempt: JobAttempt, result: Result<Answer, Error>) {
    let entry = d.change(fixtures::CHANGE).unwrap().pending.expect("kept rerun");
    outputs(d, Event::Committed { entry });
    let (mut call, mut op) = fixtures::next_call(d);
    if let Op::Read(Read::Items { .. }) = op {
        fixtures::answer(d, call, Ok(Answer::Items { cut: false, items: Box::new([]), more: false, now: env().now }));
        (call, op) = fixtures::next_call(d);
    }
    assert_eq!(op, Op::Write(Write::Rerun { attempt }));
    fixtures::answer(d, call, result);
    assert!(d.change(fixtures::CHANGE).unwrap().pending.is_none());
}

fn read_ci(d: &mut Domain, head: [u8; 32], ci: Ci, checks: &[Status], run: &Run) -> change::Decision {
    read_ci_step(d, fixtures::change_step(), head, ci, checks, run)
}

fn read_ci_step(
    d: &mut Domain,
    event: Event,
    head: [u8; 32],
    ci: Ci,
    checks: &[Status],
    run: &Run,
) -> change::Decision {
    let mut answered = outputs(d, event);
    for _ in 0_u32..24 {
        if let Some(decision) = decision(d, &answered) {
            return decision;
        }
        let (call, op) = fixtures::next_call(d);
        let answer = match op {
            Op::Read(Read::Branch { branch }) => {
                Answer::Commit(if branch.as_ref() == b"main" { fixtures::BASE } else { head })
            }
            Op::Read(Read::Pull { .. }) => {
                Answer::Pull(skein_forge::api::Pull { commit: head, ci, ..fixtures::open_pull() })
            }
            Op::Read(Read::Checks { commit }) if commit == head => Answer::Checks(checks.into()),
            Op::Read(Read::Checks { .. }) => fixtures::checks(Check::Passed),
            Op::Read(Read::Run { run: id }) if id == ATTEMPT.run => Answer::Run(run.clone()),
            Op::Read(Read::Commits { before, .. }) => Answer::Commits {
                commits: if before == head {
                    Box::new([])
                } else {
                    Box::new([CommitMessage { id: head, parents: Box::new([]), message: Box::new([]) }])
                },
                more: false,
            },
            Op::Read(Read::Reviews { .. }) => Answer::Reviews { reviews: Box::new([]), more: false, cut: false },
            Op::Read(Read::Compare { before, after }) => {
                Answer::Compare { before, after, ahead: 1, behind: 0, files: Box::new([]), changed: 0 }
            }
            other @ (Op::Read(_) | Op::Write(_)) => panic!("unexpected {other:?}"),
        };
        answered = fixtures::answer(d, call, Ok(answer));
    }
    panic!("CI recheck decided")
}

#[test]
fn a_restarted_recheck_waits_for_its_later_ended_attempt_then_holds_and_releases() {
    let (mut d, _, _) = checked(Kind::Forgejo, false, true);
    settle_rerun(&mut d, ATTEMPT, Ok(Answer::Done));
    let mut d = restored(&d);
    let later = JobAttempt { attempt: 2, ..ATTEMPT };
    let old = fixtures::failed_run(ATTEMPT, Some(1));
    let ended = fixtures::failed_run(later, Some(1));
    let mut running = ended.clone();
    running.jobs[0].state = RunState::Running;
    let wrong = fixtures::failed_run(JobAttempt { job: 42, ..later }, Some(1));
    for (link, run) in [(ATTEMPT, &old), (ATTEMPT, &ended), (later, &running), (later, &wrong)] {
        let checks = [ci_check(b"ci/test", Check::Failed, Some(link))];
        let decision = read_ci(&mut d, fixtures::HEAD, Ci::Failed, &checks, run);
        assert!(matches!(decision, change::Decision::Wait { until: Some(_) }), "{link:?}: {decision:?}");
        let row = d.change(fixtures::CHANGE).unwrap();
        assert_eq!(row.change.state, change::State::Checking { head: fixtures::HEAD });
        assert_eq!(row.ci_recheck, Some(CiRecheck { job: ATTEMPT, taken: None }));
        assert_eq!((row.change.repairs, row.change.updates), (1, 0));
        assert!(row.pending.is_none(), "no second write after restart");
    }
    let checks = [
        ci_check(b"ci/test", Check::Failed, Some(later)),
        ci_check(b"ci/lint", Check::Failed, None),
        ci_check(b"ci/build", Check::Passed, None),
    ];
    assert_eq!(
        read_ci(&mut d, fixtures::HEAD, Ci::Failed, &checks, &ended),
        change::Decision::Hold(change::Hold::RedStands)
    );
    let row = d.change(fixtures::CHANGE).unwrap();
    assert_eq!(row.ci_recheck, Some(CiRecheck { job: ATTEMPT, taken: Some(2) }));
    assert_eq!(
        row.change.state,
        change::State::Held {
            was: Box::new(change::State::Repairing { requested: false, why: change::Repair::Ci }),
            why: change::Hold::RedStands,
        }
    );
    assert_eq!(row.change.repairs, 1);
    assert_eq!(row.producer_failure.as_deref(), Some(b"its red at 030303030303 stands: repair task 52 changed nothing, and the same content failed again: ci/test, ci/lint; release this change to ask a fresh repair, or cancel it".as_slice()));

    let mut released = restored(&d);
    outputs(&mut released, Event::ReleaseChange { task: fixtures::CHANGE, release: None });
    assert_eq!(released.change(fixtures::CHANGE).unwrap().change.repairs, 0);
    assert_eq!(
        read_ci(&mut released, fixtures::HEAD, Ci::Failed, &checks, &ended),
        change::Decision::Delegate(change::Delegate::Repair(change::Repair::Ci))
    );
    let row = released.change(fixtures::CHANGE).unwrap();
    assert_eq!(row.change.repairs, 1);
    assert!(row.ci_recheck.is_none());

    let passed = [ci_check(b"ci/test", Check::Passed, Some(later))];
    let mut green = ended;
    green.conclusion = Some(Conclusion::Success);
    green.jobs[0].conclusion = Some(Conclusion::Success);
    assert_eq!(read_ci(&mut d, fixtures::HEAD, Ci::Passed, &passed, &green), change::Decision::EndHold);
    let row = d.change(fixtures::CHANGE).unwrap();
    assert_eq!(row.change.repairs, 1);
    assert!(row.producer_failure.is_none());
    assert!(!matches!(row.change.state, change::State::Held { .. }));
}

#[test]
fn a_green_measurement_goes_on_and_ending_or_moving_clears_the_record() {
    let (mut d, _, _) = checked(Kind::Forgejo, false, true);
    settle_rerun(&mut d, ATTEMPT, Ok(Answer::Done));
    let later = JobAttempt { attempt: 2, ..ATTEMPT };
    let mut run = fixtures::failed_run(later, None);
    run.conclusion = Some(Conclusion::Success);
    run.jobs[0].conclusion = Some(Conclusion::Success);
    let passed = [ci_check(b"ci/test", Check::Passed, Some(later))];
    assert_eq!(read_ci(&mut d, fixtures::HEAD, Ci::Passed, &passed, &run), change::Decision::Ready);
    let row = d.change(fixtures::CHANGE).unwrap();
    assert_eq!(row.change.last_head, Some(fixtures::HEAD));
    assert_eq!(row.change.repairs, 1);
    assert_eq!(row.ci_recheck, Some(CiRecheck { job: ATTEMPT, taken: Some(2) }));

    let mut moved = restored(&d);
    assert_eq!(read_ci(&mut moved, [9; 32], Ci::Passed, &[], &run), change::Decision::Hold(change::Hold::BranchMoved));
    assert!(moved.change(fixtures::CHANGE).unwrap().ci_recheck.is_none());

    let mut repaired = restored(&d);
    outputs(
        &mut repaired,
        Event::Delegated {
            task: fixtures::CHANGE,
            child: REPAIR + 1,
            kind: change::Delegate::Repair(change::Repair::Ci),
        },
    );
    assert!(repaired.change(fixtures::CHANGE).unwrap().ci_recheck.is_none());

    // The cancellation path also clears the kept measurement, with no head move.
    let mut cancel = fixtures::change_step();
    let Event::StepChange { heard, .. } = &mut cancel else { unreachable!("change step") };
    heard.cancelled = true;
    assert_eq!(read_ci_step(&mut d, cancel, fixtures::HEAD, Ci::Passed, &passed, &run), change::Decision::Cancel);
    assert_eq!(d.change(fixtures::CHANGE).unwrap().change.state, change::State::Ended);
    assert!(d.change(fixtures::CHANGE).unwrap().ci_recheck.is_none());
}

#[test]
fn runner_failure_after_a_recheck_keeps_the_runner_path() {
    let (mut d, _, _) = checked(Kind::Forgejo, false, true);
    settle_rerun(&mut d, ATTEMPT, Ok(Answer::Done));
    let later = JobAttempt { attempt: 2, ..ATTEMPT };
    d.changes.get_mut(&fixtures::CHANGE).unwrap().runner_ci = Some(RunnerCi::Counting { attempts: Box::new([later]) });
    let checking = [ci_check(b"ci/test", Check::Failed, Some(later))];
    assert_eq!(
        read_ci(&mut d, fixtures::HEAD, Ci::Failed, &checking, &fixtures::failed_run(later, Some(0))),
        change::Decision::Effect(change::Effect::Rerun { head: fixtures::HEAD })
    );
    assert_eq!(d.change(fixtures::CHANGE).unwrap().ci_recheck, Some(CiRecheck { job: ATTEMPT, taken: Some(2) }));
    settle_rerun(&mut d, later, Ok(Answer::Done));
    let last = JobAttempt { attempt: 3, ..ATTEMPT };
    let checking = [ci_check(b"ci/test", Check::Failed, Some(last))];
    assert_eq!(
        read_ci(&mut d, fixtures::HEAD, Ci::Failed, &checking, &fixtures::failed_run(last, Some(0))),
        change::Decision::Hold(change::Hold::Runner)
    );
    let row = d.change(fixtures::CHANGE).unwrap();
    assert_eq!(row.change.repairs, 1);
    assert!(matches!(row.runner_ci, Some(RunnerCi::Held { .. })));
}

#[test]
fn a_refused_recheck_write_holds_instead_of_waiting_for_an_attempt() {
    let (mut d, _, _) = checked(Kind::Forgejo, false, true);
    settle_rerun(&mut d, ATTEMPT, Err(Error::Refused));
    let checks = [ci_check(b"ci/test", Check::Failed, Some(ATTEMPT))];
    assert_eq!(
        read_ci(&mut d, fixtures::HEAD, Ci::Failed, &checks, &fixtures::failed_run(ATTEMPT, Some(1))),
        change::Decision::Hold(change::Hold::RedStands)
    );
    let row = d.change(fixtures::CHANGE).unwrap();
    assert_eq!(row.change.repairs, 1);
    assert_eq!(row.producer_failure.as_deref(), Some(UNCHECKED));

    let passed = [ci_check(b"ci/test", Check::Passed, None)];
    let old = fixtures::failed_run(ATTEMPT, Some(1));
    assert_eq!(read_ci(&mut d, fixtures::HEAD, Ci::Passed, &passed, &old), change::Decision::EndHold);
    assert_eq!(read_ci(&mut d, fixtures::HEAD, Ci::Passed, &passed, &old), change::Decision::Ready);
}
