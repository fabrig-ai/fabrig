//! Deployment control stays within its project's task cancellation (domain/tasks.md, 5.5).
use super::revisions::{Harness, task, to};
use crate::{Control, Ending, Event, Key, Party, Phase, Refusal, Request, Stored};

#[test]
fn a_deployment_cancels_a_task_of_its_project_and_no_other() {
    for (project, control, accepted) in [
        (1, Control::Cancel { reason: Box::new([]), grace: None, message: None }, true),
        (2, Control::Cancel { reason: Box::new([]), grace: None, message: None }, false),
        (1, Control::Release { again: None, message: None }, false),
        (2, Control::Release { again: None, message: None }, false),
    ] {
        let mut test = Harness::new();
        test.send(Event::Make { reply_to: to(), creator: Party::Person(9), batch: Box::new([task(1, &[])]) });
        let before = test.domain.task(1).expect("created task").clone();
        let by = Party::Deployment { project };
        assert!(!crate::control::allowed(&test.domain, 1, by, test.env.limits.tasks));
        let out = test.send(Event::Control { reply_to: to(), by, task: 1, control });
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
