//! Deployment control stays within its project's task cancellation (domain/tasks.md, 5.5).
use super::revisions::{Harness, task, to};
use crate::{Cancel, Ending, Event, Key, Operation, Party, Phase, Refusal, Release, Request, Stored};

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
