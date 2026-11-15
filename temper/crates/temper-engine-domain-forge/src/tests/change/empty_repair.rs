//! Fresh reads and forge capability for empty repairs (change/repairs.md).
use super::*;
use skein_forge::api::{Answer, Check, Ci, CommitMessage, JobAttempt, Op, Read, Status, Write};
use temper_engine_domain_forge_change as change;

fn checked(kind: Kind, late: bool, no_change: bool) -> (Domain, change::Decision, u32) {
    checked_repair(kind, late, no_change, change::Repair::Ci, None)
}

fn checked_repair(
    kind: Kind,
    late: bool,
    no_change: bool,
    why: change::Repair,
    advanced: Option<[u8; 32]>,
) -> (Domain, change::Decision, u32) {
    let mut d = fixtures::connected_domain();
    d.configured.get_mut(&1).expect("forge configuration").kind = kind;
    let mut repository = adopted();
    repository.protection = ci_protection(false);
    outputs(&mut d, Event::Restore { record: Stored::Repository(repository) });
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut d, Event::ReadAfresh);
    outputs(&mut d, Event::SettleOutbox);
    let mut row = fixtures::opening_row();
    row.pending = None;
    row.pull = Some(1);
    row.effect = change::EffectResult::None;
    row.change.state = change::State::Repairing { requested: true, why };
    row.change.last_head = Some(fixtures::HEAD);
    row.change.repairs = 1;
    row.delegate = Some((52, change::Delegate::Repair(why)));
    row.delegate_status = change::Status::Pending;
    outputs(&mut d, Event::Change { row });
    if !late {
        producer_terminal(&mut d, no_change);
    }
    let attempt = JobAttempt { head: fixtures::HEAD, run: 7, job: 41, attempt: 1 };
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
            Op::Read(Read::Checks { commit }) if commit == fixtures::HEAD => Answer::Checks(Box::new([Status {
                at: Time::ZERO,
                context: b"ci/test".as_slice().into(),
                check: Check::Failed,
                description: Box::new([]),
                url: Box::new([]),
                job: Some(attempt),
            }])),
            Op::Read(Read::Checks { .. }) => fixtures::checks(Check::Passed),
            Op::Read(Read::Run { .. }) => Answer::Run(fixtures::failed_run(attempt, Some(1))),
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
            producer_terminal(&mut d, no_change);
        }
        let answered = fixtures::answer(&mut d, call, Ok(answer));
        for request in answered {
            if let Request::ChangeDecision { decision, .. } = request {
                return (d, decision, branches);
            }
        }
    }
    panic!("empty repair decided")
}

#[test]
fn empty_repairs_rerun_read_jobs_only_on_a_forge_that_writes_reruns() {
    for (kind, no_change) in
        [(Kind::Forgejo, true), (Kind::GitHub, true), (Kind::Forgejo, false), (Kind::GitHub, false)]
    {
        let (d, decision, _) = checked(kind, false, no_change);
        let row = d.change(fixtures::CHANGE).expect("change");
        if kind == Kind::Forgejo {
            assert_eq!(decision, change::Decision::Effect(change::Effect::Rerun { head: fixtures::HEAD }));
            assert_eq!(row.change.state, change::State::Checking { head: fixtures::HEAD });
            assert_eq!(row.change.repairs, 1);
            assert_eq!(
                d.entries.get(&row.pending.expect("entry")).expect("entry").entry.effect.write,
                Write::Rerun { attempt: JobAttempt { head: fixtures::HEAD, run: 7, job: 41, attempt: 1 } }
            );
            assert_eq!(
                row.effect_account.as_ref().expect("account").words.as_ref(),
                b"ran job 41 of ci/test again at 030303030303: a repair changed nothing"
            );
        } else {
            assert_eq!(decision, change::Decision::Delegate(change::Delegate::Repair(change::Repair::Ci)));
            assert_eq!(row.change.repairs, 2);
            assert!(row.pending.is_none());
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
fn a_review_repair_delivery_during_gathering_reads_its_branch_after_the_end() {
    for why in [change::Repair::Gate(1), change::Repair::Review { head: fixtures::HEAD }] {
        let advanced = [9; 32];
        let (d, decision, branches) = checked_repair(Kind::Forgejo, true, false, why, Some(advanced));
        assert_eq!(branches, 2, "{why:?}: a delivery needs a branch read after its terminal");
        assert_eq!(d.change(fixtures::CHANGE).expect("delivered change").change.last_head, Some(advanced));
        assert_ne!(decision, change::Decision::Hold(change::Hold::Failed));
    }
}
