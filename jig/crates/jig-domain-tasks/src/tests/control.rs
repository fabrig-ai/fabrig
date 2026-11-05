//! Deployment control cancels its project's tasks and releases procedures' own holds
//! (jig/docs/design/domain/tasks.md, sections 5.5 and 7).
use super::revisions::{Harness, task, to};
use crate::{
    Active, Cancel, Change, Ending, Event, Executor, Hold, HoldSource, Key, Operation, Party, Phase, Refusal, Release,
    Request, Stored, Why,
};

const PROCEDURE: Executor = Executor::Procedure { connector: 1, code: 1 };
const CONTROLLED: u64 = 1;
const PARENT: u64 = 2;
const PERSON: u64 = 9;
const PROJECT: u32 = 1;
const OTHER_PROJECT: u32 = 2;
const PLACED: u64 = 3;

fn control_task(executor: Executor) -> Harness {
    let mut test = Harness::new();
    let mut member = task(CONTROLLED, &[]);
    member.executor = executor;
    let creator = match executor {
        Executor::Job { .. } => {
            let mut parent = task(PARENT, &[]);
            parent.executor = PROCEDURE;
            test.send(Event::Make { reply_to: to(), creator: Party::Person(PERSON), batch: Box::new([parent]) });
            member.spec.job =
                Some(Box::new(crate::JobSpec { bytes: Box::new([]), deadline: skein_lib::Duration::from_secs(1) }));
            member.spec.words = Box::new([]);
            member.contract = crate::Contract::Job { bytes: 0 };
            member.numbers.budget = 0;
            member.authority.budget.spend = 0;
            member.funder = crate::Funder::Task(PARENT);
            Party::Task(PARENT)
        }
        Executor::Agent { .. } | Executor::Procedure { .. } | Executor::Person(_) => Party::Person(PERSON),
    };
    test.send(Event::Make { reply_to: to(), creator, batch: Box::new([member]) });
    test
}

fn place_control_hold(test: &mut Harness) {
    test.send(Event::Place {
        reply_to: to(),
        hold: crate::PlacedHold {
            number: PLACED,
            project: PROJECT,
            owner: crate::Owner::Party { person: PERSON },
            mode: crate::Mode::Stop,
            scope: crate::Scope::Subtree { task: CONTROLLED },
            reason: Box::new([]),
            placed_at: 0,
            revision: 1,
            exempt: Box::new([]),
            lifted: false,
        },
    });
}

#[test]
fn a_deployment_releases_a_procedure_from_its_own_hold() {
    for why in [Hold::Procedure, Hold::EffectFailed, Hold::Stalled, Hold::Budget] {
        let mut test = control_task(PROCEDURE);
        test.send(Event::Hold { task: CONTROLLED, why });
        let by = Party::Deployment { project: PROJECT };
        assert!(!crate::control::allowed(&test.domain, CONTROLLED, by, test.env.limits.tasks));
        let out = test.send(Event::Release {
            reply_to: to(),
            by,
            operation: Release { task: CONTROLLED, again: None, message: None },
        });
        let Some(Request::Done { .. }) = out.last() else { panic!("{why:?}: {out:?}") };
        let row = test.domain.task(CONTROLLED).expect("released procedure");
        assert_eq!(row.phase, Phase::Active(Active::Due), "{why:?}");
        assert_eq!(row.due_reason, Why::Released, "{why:?}");
        let Stored::History(history) =
            test.stored.get(&Key::History { task: CONTROLLED, revision: row.revision }).expect("release history")
        else {
            panic!("history record")
        };
        assert_eq!(history.change, Change::Released, "{why:?}");
        assert_eq!(history.by, by, "{why:?}");
    }
}

#[test]
fn a_deployment_releases_nothing_else() {
    for (executor, why, placed, project) in [
        (PROCEDURE, Some(Hold::Procedure), false, OTHER_PROJECT),
        (Executor::Agent { charter: 1 }, Some(Hold::Procedure), false, PROJECT),
        (Executor::Job { kind: 1 }, Some(Hold::Procedure), false, PROJECT),
        (Executor::Person(crate::PersonAddress::Person(PERSON)), Some(Hold::Procedure), false, PROJECT),
        (PROCEDURE, Some(Hold::Procedure), true, PROJECT),
        (PROCEDURE, Some(Hold::EffectFailed), true, PROJECT),
        (PROCEDURE, Some(Hold::Stalled), true, PROJECT),
        (PROCEDURE, Some(Hold::Budget), true, PROJECT),
        (PROCEDURE, None, true, PROJECT),
        (PROCEDURE, Some(Hold::Stopped), false, PROJECT),
        (PROCEDURE, Some(Hold::Drift), false, PROJECT),
        (PROCEDURE, Some(Hold::Effects), false, PROJECT),
        (PROCEDURE, Some(Hold::Failures(crate::Class::Lost)), false, PROJECT),
        (PROCEDURE, Some(Hold::Dependency(PARENT)), false, PROJECT),
        (PROCEDURE, Some(Hold::Exceeded(crate::Exceeded { bound: crate::Bound::Answer, value: 1 })), false, PROJECT),
        (PROCEDURE, Some(Hold::Uncertain { entry: 1 }), false, PROJECT),
        (PROCEDURE, Some(Hold::Deadline), false, PROJECT),
        (PROCEDURE, Some(Hold::HoldsWaited), false, PROJECT),
        (PROCEDURE, Some(Hold::ResourceUnavailable), false, PROJECT),
        (PROCEDURE, Some(Hold::ResourceAccess), false, PROJECT),
        (PROCEDURE, Some(Hold::PoolLost { pool: 1 }), false, PROJECT),
        (PROCEDURE, Some(Hold::Circle), false, PROJECT),
        (PROCEDURE, None, false, PROJECT),
    ] {
        let mut test = control_task(executor);
        if placed {
            place_control_hold(&mut test);
        }
        if let Some(why) = why {
            let source = if why == Hold::Circle {
                HoldSource::Circle(crate::Circle { tasks: Box::new([CONTROLLED]), cut: false })
            } else {
                HoldSource::Account { connector: 1, entry: Some(1), words: Box::new([1]) }
            };
            test.send(Event::HoldFrom { task: CONTROLLED, why, source: Box::new(source) });
        }
        let before = test.domain.task(CONTROLLED).expect("controlled task").clone();
        let stored = test.stored.clone();
        let by = Party::Deployment { project };
        assert!(!crate::control::allowed(&test.domain, CONTROLLED, by, test.env.limits.tasks));
        let out = test.send(Event::Release {
            reply_to: to(),
            by,
            operation: Release { task: CONTROLLED, again: None, message: None },
        });
        let [Request::Refused { problem, .. }] = out.as_slice() else {
            panic!("{executor:?}, {why:?}, placed={placed}, project={project}: {out:?}");
        };
        assert_eq!(problem.why, Refusal::Reference, "{executor:?}, {why:?}, placed={placed}, project={project}");
        assert_eq!(test.domain.task(CONTROLLED).expect("unchanged task"), &before);
        assert_eq!(test.stored, stored);
    }
}

#[test]
#[expect(clippy::wildcard_enum_match_arm, reason = "the table carries only cancel and release operations")]
fn a_deployment_cancels_a_task_of_its_project_and_no_other() {
    for (project, control, accepted) in [
        (1, Operation::Cancel(Cancel { task: 1, reason: Box::new([]), grace: None, message: None }), true),
        (2, Operation::Cancel(Cancel { task: 1, reason: Box::new([]), grace: None, message: None }), false),
        (1, Operation::Release(Release { task: 1, again: None, message: None }), false),
        (2, Operation::Release(Release { task: 1, again: None, message: None }), false),
    ] {
        let mut test = Harness::new();
        test.send(Event::Make { reply_to: to(), creator: Party::Person(9), batch: Box::new([task(1, &[])]) });
        let before = test.domain.task(1).expect("created task").clone();
        let by = Party::Deployment { project };
        assert!(!crate::control::allowed(&test.domain, 1, by, test.env.limits.tasks));
        let out = test.send(match control {
            Operation::Cancel(operation) => Event::Cancel { reply_to: to(), by, operation },
            Operation::Release(operation) => Event::Release { reply_to: to(), by, operation },
            _ => unreachable!("control operation"),
        });
        let mut done = false;
        let mut refused = false;
        for request in out {
            if let Request::Done { .. } = request {
                done = true;
            }
            if let Request::Refused { problem, .. } = request {
                assert_eq!(problem.why, Refusal::Reference);
                refused = true;
            }
        }
        assert_eq!(done, accepted);
        assert_eq!(refused, !accepted);
        if accepted {
            test.send(Event::EffectsSettled { task: 1 });
            test.send(Event::Settled { task: 1 });
            let Stored::Ended(row) = test.stored.get(&Key::Ended(1)).expect("cancelled task") else {
                panic!("ended record");
            };
            let Phase::Ended(Ending::Cancelled { .. }) = row.phase else { panic!("cancelled terminal") };
        } else {
            assert_eq!(test.domain.task(1).expect("task unchanged"), &before);
        }
    }
}

#[test]
fn party_sender_labels() {
    assert_eq!(Party::Task(123).sender().as_ref(), b"123");
    assert_eq!(Party::Person(456).sender().as_ref(), b"456");
    assert_eq!(Party::Deployment { project: 7 }.sender().as_ref(), b"deployment");
}

#[test]
fn cancel_permission_is_checked_before_its_shape() {
    let mut test = Harness::new();
    test.send(Event::Make { reply_to: to(), creator: Party::Person(9), batch: Box::new([task(1, &[])]) });
    let out = test.send(Event::Cancel {
        reply_to: to(),
        by: Party::Task(1),
        operation: Cancel { task: 1, reason: Box::new([]), grace: None, message: Some(1) },
    });
    let [Request::Refused { problem, .. }] = out.as_slice() else { panic!("one refusal") };
    assert_eq!(problem.why, Refusal::Reference);
}
