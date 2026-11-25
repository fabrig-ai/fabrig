//! Application failure uses cancellation's tree lifecycle, but returns failure (domain/tasks/lifecycle.md, 13).
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    reason = "unit scripts retain harness outputs and build bounded reason bytes in ordinary Rust"
)]
use super::closer_funds::{answer, fixture, start};
use super::revisions::{Harness, task, to};
use crate::{Active, Cancel, End, Ending, Event, Fail, Funder, Hold, Key, Party, Phase, Refusal, Request, Stored};

const TASK: u64 = 1;
const CLOSER: u64 = 2;
const WORK: u64 = 3;
const BELOW: u64 = 4;
const DEPLOYMENT: Party = Party::Deployment { project: 1 };
const REASON: &[u8] = b"application deadline passed";

fn done(out: &[Request]) -> bool {
    for request in out {
        if let Request::Done { .. } = request {
            return true;
        }
    }
    false
}

fn fail(h: &mut Harness, number: u64, by: Party, reason: &[u8]) -> Vec<Request> {
    h.send(Event::Fail { reply_to: to(), by, operation: Fail { task: number, reason: reason.into() } })
}

fn child(h: &mut Harness, parent: u64, number: u64, budget: u64) {
    let mut member = task(number, &[]);
    member.funder = Funder::Task(parent);
    member.numbers.budget = budget;
    member.authority.budget.spend = budget;
    let out = h.send(Event::Make {
        reads_project: false,
        maker: None,
        places: Box::new([]),
        reply_to: to(),
        creator: Party::Task(parent),
        batch: Box::new([member]),
    });
    assert!(h.domain.task(number).is_some(), "{out:?}");
}

fn ending(h: &Harness, number: u64) -> &Ending {
    let Stored::Ended(row) = h.stored.get(&Key::Ended(number)).expect("ended record kept") else {
        panic!("ended record")
    };
    let Phase::Ended(ending) = &row.phase else { panic!("terminal phase") };
    ending
}

fn settle(h: &mut Harness, number: u64) -> Vec<Request> {
    // After a delegate's result the root gets a turn before its requester advances.
    let mut out = skein_lib::Queue::uncapped();
    crate::fire(&mut h.domain, &h.env, &mut out);
    while let Some(request) = out.pop() {
        if let Request::Save { record } = request {
            h.stored.insert(record.key(), record);
        }
    }
    h.send(Event::EffectsSettled { task: number });
    h.send(Event::Settled { task: number })
}

#[test]
fn a_deployment_fails_idle_or_running_agents_and_cancels_their_tree_and_closer() {
    for running in [false, true] {
        let mut h = fixture();
        start(&mut h, TASK, 300); // The closer yields funds to its goal's judgement.
        child(&mut h, TASK, WORK, 100);
        start(&mut h, WORK, 50);
        child(&mut h, WORK, BELOW, 20);
        if !running {
            answer(&mut h, TASK, 0, End::Parked);
            assert_eq!(h.domain.task(TASK).expect("idle goal").phase, Phase::Active(Active::Idle));
        }
        let attempt = h.domain.task(TASK).expect("goal").attempt;
        let work_attempt = h.domain.task(WORK).expect("work").attempt;
        let out = fail(&mut h, TASK, DEPLOYMENT, REASON);
        let mut task_stops = 0;
        let mut work_stopped = false;
        let mut replied = false;
        for request in out {
            if let Request::Stop { task, attempt: stopped } = request {
                if task == TASK {
                    assert_eq!(stopped, attempt);
                    task_stops += 1;
                }
                if task == WORK {
                    assert_eq!(stopped, work_attempt);
                    work_stopped = true;
                }
            }
            if let Request::Done { reply_to } = request {
                assert_eq!(reply_to.into_token(), to().into_token());
                replied = true;
            }
        }
        assert!(replied && work_stopped);
        assert_eq!(task_stops, usize::from(running));
        let Phase::Closing(closing) = &h.domain.task(TASK).expect("closing goal").phase else { panic!("closing") };
        assert_eq!(closing.ending, Ending::Failed { reason: REASON.into() });
        for number in [BELOW, WORK, CLOSER] {
            let Phase::Closing(closing) = &h.domain.task(number).expect("cancelled child").phase else {
                panic!("child closing")
            };
            let Ending::Cancelled { reason, by, .. } = &closing.ending else { panic!("child cancelled") };
            assert_eq!(reason.as_ref(), REASON);
            assert_eq!(*by, DEPLOYMENT);
        }
        if running {
            answer(&mut h, TASK, 0, End::Parked);
            assert_eq!(h.domain.task(TASK).expect("run stopped").attempt, attempt);
        }
        answer(&mut h, WORK, 0, End::Parked);
        for number in [BELOW, WORK, CLOSER] {
            settle(&mut h, number);
            let Ending::Cancelled { reason, .. } = ending(&h, number) else { panic!("descendant ended cancelled") };
            assert_eq!(reason.as_ref(), REASON);
        }
        let out = settle(&mut h, TASK);
        assert_eq!(ending(&h, TASK), &Ending::Failed { reason: REASON.into() });
        let mut heard = false;
        for request in out {
            if let Request::Ended { task: TASK, requester, ending } = request {
                assert_eq!(requester, Party::Person(9));
                assert_eq!(ending, Ending::Failed { reason: REASON.into() });
                heard = true;
            }
        }
        assert!(heard, "the requester receives the failure");
    }
}

#[test]
fn ancestors_may_fail_a_task_but_people_self_siblings_and_other_projects_may_not() {
    for (by, permitted) in [
        (Party::Task(1), true),
        (Party::Task(2), true),
        (Party::Person(9), false),
        (Party::Person(0), false),
        (Party::Task(3), false),
        (Party::Task(4), false),
        (Party::Deployment { project: 2 }, false),
    ] {
        let mut h = Harness::new();
        h.make([task(1, &[]), task(4, &[])].into());
        child(&mut h, 1, 2, 50);
        child(&mut h, 2, 3, 20);
        let before = h.domain.task(3).expect("target").clone();
        let stored = h.stored.clone();
        let out = fail(&mut h, 3, by, REASON);
        if permitted {
            assert!(done(&out), "ancestor admitted");
            let out = settle(&mut h, 3);
            assert_eq!(ending(&h, 3), &Ending::Failed { reason: REASON.into() });
            let mut heard = false;
            for request in out {
                if let Request::Ended { task: 3, requester, ending } = request {
                    assert_eq!(requester, Party::Task(2));
                    assert_eq!(ending, Ending::Failed { reason: REASON.into() });
                    heard = true;
                }
            }
            assert!(heard);
        } else {
            let [Request::Refused { problem, .. }] = out.as_slice() else { panic!("one refusal: {out:?}") };
            assert_eq!((problem.task, problem.why), (Some(3), Refusal::Reference));
            assert_eq!(h.domain.task(3).expect("unchanged"), &before);
            assert_eq!(h.stored, stored);
        }
    }
}

#[test]
fn failure_of_closing_or_ended_tasks_is_refused_without_writes_and_names_the_task() {
    let mut h = Harness::new();
    h.make([task(TASK, &[])].into());
    h.send(Event::Cancel {
        reply_to: to(),
        by: DEPLOYMENT,
        operation: Cancel { task: TASK, reason: REASON.into(), grace: None, message: None },
    });
    for held in [false, true] {
        if held {
            h.send(Event::Hold { task: TASK, why: Hold::Effects });
        }
        let before = h.domain.task(TASK).expect("closing target").clone();
        let stored = h.stored.clone();
        let out = fail(&mut h, TASK, DEPLOYMENT, b"too late");
        let [Request::Refused { problem, .. }] = out.as_slice() else { panic!("one refusal: {out:?}") };
        assert_eq!((problem.task, problem.why), (Some(TASK), Refusal::State));
        assert_eq!(h.domain.task(TASK).expect("unchanged"), &before);
        assert_eq!(h.stored, stored);
    }
    // A separate ended task is no longer in the live arena, as for cancellation.
    let mut h = Harness::new();
    h.make([task(TASK, &[])].into());
    fail(&mut h, TASK, DEPLOYMENT, REASON);
    settle(&mut h, TASK);
    let stored = h.stored.clone();
    let out = fail(&mut h, TASK, DEPLOYMENT, b"too late");
    let [Request::Refused { problem, .. }] = out.as_slice() else { panic!("one refusal: {out:?}") };
    assert_eq!((problem.task, problem.why), (Some(TASK), Refusal::Unknown));
    assert_eq!(h.stored, stored);
}

#[test]
fn an_application_failure_cuts_the_whole_trees_reason_before_a_split_utf8_character() {
    let mut h = Harness::new();
    h.make([task(TASK, &[])].into());
    child(&mut h, TASK, WORK, 20);
    let bound = usize::try_from(h.env.limits.result_bytes).expect("test bound");
    let prefix = b"x".repeat(bound - 1);
    let mut reason = prefix.clone();
    reason.extend_from_slice("é deadline".as_bytes());
    let out = fail(&mut h, TASK, DEPLOYMENT, &reason);
    assert!(done(&out), "bounded, not refused");
    settle(&mut h, WORK);
    let Ending::Cancelled { reason, .. } = ending(&h, WORK) else { panic!("child cancelled") };
    assert_eq!(reason.as_ref(), prefix);
    settle(&mut h, TASK);
    assert_eq!(ending(&h, TASK), &Ending::Failed { reason: prefix.into_boxed_slice() });
}

#[test]
fn make_checks_a_goals_closer_kind_before_any_writes() {
    let mut h = Harness::new();
    let mut goal = task(TASK, &[]);
    goal.flags.ended_by = crate::EndedBy::Closer(crate::Closer { connector: 1, code: 6 });
    let stored = h.stored.clone();
    let out = h.send(Event::Make {
        reads_project: false,
        maker: None,
        places: Box::new([]),
        reply_to: to(),
        creator: Party::Person(9),
        batch: Box::new([goal]),
    });
    let [Request::Refused { problem, .. }] = out.as_slice() else { panic!("one refusal: {out:?}") };
    assert_eq!((problem.task, problem.why), (Some(TASK), Refusal::Executor));
    assert!(h.domain.task(TASK).is_none());
    assert_eq!(h.stored, stored);
}
