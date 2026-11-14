//! Changed connector facts stay level-triggered through a pending step
//! (jig/docs/design/domain/tasks/lifecycle.md, 3).
use super::revisions::{Harness, task, to};
use crate::{Active, Event, Executor, Phase, ProcedureDecision, ProcedureEnd};

fn procedure() -> Harness {
    let mut h = Harness::new();
    let mut row = task(1, &[]);
    row.executor = Executor::Procedure { connector: 1, code: 1 };
    h.make([row].into());
    h
}

fn wait(h: &mut Harness) -> Vec<crate::Request> {
    let (_, _, step) = h.domain.procedure_due(1).expect("due procedure");
    let read = h.domain.procedure_offered(1);
    h.send(Event::Procedure {
        reply_to: to(),
        task: 1,
        step,
        read,
        decision: ProcedureDecision::new(ProcedureEnd::Wait),
    })
}

#[test]
fn changed_facts_during_a_step_wake_once_after_its_wait() {
    let mut h = procedure();
    for _ in 0_u8..3 {
        h.send(Event::WakeProcedure { task: 1 });
    }
    wait(&mut h);
    assert_eq!(h.domain.task(1).expect("procedure").phase, Phase::Active(Active::Due));
    wait(&mut h);
    assert_eq!(h.domain.task(1).expect("procedure").phase, Phase::Active(Active::Idle));
}

#[test]
fn restart_reoffers_the_due_steps_changed_facts() {
    let mut h = procedure();
    h.send(Event::WakeProcedure { task: 1 });
    h.restart();
    assert_eq!(h.domain.task(1).expect("restored procedure").phase, Phase::Active(Active::Due));
    wait(&mut h);
    assert_eq!(h.domain.task(1).expect("procedure").phase, Phase::Active(Active::Idle));
}

#[test]
fn a_pending_wake_follows_the_owners_step_acknowledgment() {
    let mut h = procedure();
    h.send(Event::WakeProcedure { task: 1 });
    let out = wait(&mut h);
    let done = out.iter().position(|request| matches!(request, crate::Request::Done { .. })).expect("step done");
    let wake = out
        .iter()
        .position(|request| matches!(request, crate::Request::Activate { context } if context.task == 1))
        .expect("next step");
    assert!(done < wake, "the owner records its decision before another activation: {out:?}");
}
