//! Live room is a durable condition, with one oldest eligible procedure woken per ending.
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    clippy::wildcard_enum_match_arm,
    clippy::match_like_matches_macro,
    reason = "step tests script inputs and inspect selected outputs in ordinary Rust"
)]
use super::revisions::{Harness, task, to};
use crate::{
    Active, Cancel, Capacity, CapacityAction, CapacityCause, Domain, Event, Executor, Funder, Key, LiveRoom, New,
    Party, Phase, Problem, ProcedureDecision, ProcedureEnd, Refusal, Request, RoomWait, Stored,
};

fn procedure(number: u64) -> New {
    let mut row = task(number, &[]);
    row.executor = Executor::Procedure { connector: 1, code: 1 };
    row
}

fn child(parent: u64, number: u64) -> New {
    let mut row = task(number, &[]);
    row.funder = Funder::Task(parent);
    row
}

fn batch(h: &mut Harness, parent: u64, number: u64) -> Vec<Request> {
    let row = h.domain.task(parent).unwrap();
    let step = row.attempt.checked_add(1).expect("test step fits");
    let read = h.domain.procedure_offered(parent);
    h.send(Event::Procedure {
        reply_to: to(),
        task: parent,
        step,
        read,
        decision: ProcedureDecision {
            batch: Box::new([child(parent, number)]),
            ..ProcedureDecision::new(ProcedureEnd::Wait)
        },
    })
}

fn finish(h: &mut Harness, number: u64) -> Vec<Request> {
    h.send(Event::Cancel {
        reply_to: to(),
        by: Party::Person(9),
        operation: Cancel { task: number, reason: Box::new([]), grace: None, message: None },
    });
    h.send(Event::EffectsSettled { task: number });
    let out = h.send(Event::Settled { task: number });
    assert!(h.domain.task(number).is_none());
    out
}

fn waiting(h: &Harness, number: u64, room: LiveRoom, order: u64) {
    let row = h.domain.task(number).unwrap();
    assert_eq!(row.phase, Phase::Active(Active::Idle));
    assert_eq!(row.room_wait, Some(RoomWait { room, order }));
    let Stored::Live(saved) = h.stored.get(&Key::Live(number)).unwrap() else { panic!("live row") };
    assert_eq!(saved.room_wait, row.room_wait);
}

fn activations(out: &[Request]) -> Vec<u64> {
    out.iter()
        .filter_map(|request| match request {
            Request::Activate { context } => Some(context.task),
            _ => None,
        })
        .collect()
}

#[test]
fn deployment_refusal_waits_and_an_ending_wakes_it_to_make_its_batch() {
    let mut h = Harness::new();
    h.make(vec![procedure(1), task(2, &[])]);
    h.env.limits.tasks = 2;
    let out = batch(&mut h, 1, 3);
    assert!(out.iter().any(|request| match request {
        Request::ProcedureRefused { problem, .. } => problem.why == Refusal::Live,
        _ => false,
    }));
    waiting(&h, 1, LiveRoom::Deployment, 1);
    assert_eq!(activations(&finish(&mut h, 2)), [1]);
    assert_eq!(h.domain.task(1).unwrap().room_wait, None);
    let out = batch(&mut h, 1, 3);
    assert!(
        out.iter().any(|request| match request {
            Request::Made { tasks, .. } => tasks.as_ref() == [3],
            _ => false,
        }),
        "{out:?}"
    );
    assert!(h.domain.task(3).is_some());
    assert_eq!(h.domain.task(1).unwrap().room_wait, None);
}

#[test]
fn project_refusal_ignores_another_projects_ending_then_makes_its_batch() {
    let mut h = Harness::new();
    h.send(Event::OpenPeriod { reply_to: to(), project: 2, period: 0, budget: 1000 });
    let mut other = task(3, &[]);
    other.project = 2;
    other.funder = Funder::Period { project: 2, period: 0 };
    h.make(vec![procedure(1), task(2, &[]), other]);
    h.env.limits.project_tasks = 2;
    batch(&mut h, 1, 4);
    waiting(&h, 1, LiveRoom::Project(1), 1);
    assert_eq!(activations(&finish(&mut h, 3)).len(), 0);
    waiting(&h, 1, LiveRoom::Project(1), 1);
    assert_eq!(activations(&finish(&mut h, 2)), [1]);
    batch(&mut h, 1, 4);
    assert!(h.domain.task(4).is_some());
}

fn ordered_waiters() -> Harness {
    let mut h = Harness::new();
    h.make(vec![procedure(1), procedure(2), task(3, &[]), task(4, &[])]);
    h.env.limits.tasks = 4;
    batch(&mut h, 2, 5);
    batch(&mut h, 1, 6);
    waiting(&h, 2, LiveRoom::Deployment, 1);
    waiting(&h, 1, LiveRoom::Deployment, 2);
    h
}

#[test]
fn waiters_wake_one_per_ending_in_refusal_order() {
    let mut h = ordered_waiters();
    assert_eq!(activations(&finish(&mut h, 3)), [2]);
    waiting(&h, 1, LiveRoom::Deployment, 2);
    assert_eq!(activations(&finish(&mut h, 4)), [1]);
}

#[test]
fn a_wake_reserves_nothing_and_a_second_refusal_joins_the_back() {
    let mut h = ordered_waiters();
    finish(&mut h, 3);
    h.make(vec![task(7, &[])]);
    batch(&mut h, 2, 5);
    waiting(&h, 2, LiveRoom::Deployment, 3);
    assert_eq!(activations(&finish(&mut h, 4)), [1]);
    waiting(&h, 2, LiveRoom::Deployment, 3);
}

#[test]
fn restart_restores_the_waits_and_their_refusal_order() {
    let mut h = ordered_waiters();
    h.restart();
    waiting(&h, 2, LiveRoom::Deployment, 1);
    waiting(&h, 1, LiveRoom::Deployment, 2);
    assert_eq!(activations(&finish(&mut h, 3)), [2]);
    assert_eq!(activations(&finish(&mut h, 4)), [1]);
}

#[test]
fn tree_delegates_and_inbox_refusals_do_not_wait() {
    for (cause, why) in [(0_u8, Refusal::Tree), (1, Refusal::Delegates), (2, Refusal::Busy)] {
        let mut h = Harness::new();
        h.make(vec![procedure(1)]);
        match cause {
            0 => h.env.limits.tree_tasks = 1,
            1 => h.env.limits.delegates = 0,
            2 => h.env.limits.inbox_messages = 0,
            _ => unreachable!(),
        }
        let out = batch(&mut h, 1, 2);
        assert!(
            out.iter().any(|request| match request {
                Request::ProcedureRefused { problem, .. } =>
                    problem.why == why && problem.capacity.is_some() && problem.freeing_count().is_none(),
                _ => false,
            }),
            "{out:?}"
        );
        assert_eq!(h.domain.task(1).unwrap().room_wait, None);
        assert!(h.domain.task(2).is_none());
    }
}

#[test]
fn an_agents_delegation_refused_for_room_does_not_wait() {
    let mut h = Harness::new();
    h.make(vec![task(1, &[])]);
    h.env.limits.tasks = 1;
    let out = h.send(Event::Make { reply_to: to(), creator: Party::Task(1), batch: Box::new([child(1, 2)]) });
    assert!(out.iter().any(|request| match request {
        Request::Refused { problem, .. } => problem.why == Refusal::Live,
        _ => false,
    }));
    assert_eq!(h.domain.task(1).unwrap().room_wait, None);
}

#[test]
fn a_changed_fact_and_cancellation_drop_the_wait() {
    let mut h = ordered_waiters();
    let out = h.send(Event::WakeProcedure { task: 2 });
    assert_eq!(activations(&out), [2]);
    assert_eq!(h.domain.task(2).unwrap().room_wait, None);
    let out = h.send(Event::Cancel {
        reply_to: to(),
        by: Party::Person(9),
        operation: Cancel { task: 1, reason: Box::new([]), grace: None, message: None },
    });
    assert!(out.iter().any(|request| match request {
        Request::Done { .. } => true,
        _ => false,
    }));
    assert_eq!(h.domain.task(1).unwrap().room_wait, None);
}

#[test]
fn restore_refuses_nonprocedure_wrong_project_zero_and_duplicate_orders() {
    for invalid in 0_u8..4 {
        let mut h = ordered_waiters();
        let Stored::Live(row) = h.stored.get_mut(&Key::Live(1)).unwrap() else { panic!("live row") };
        match invalid {
            0 => row.executor = Executor::Agent { charter: 1 },
            1 => row.room_wait.as_mut().unwrap().room = LiveRoom::Project(2),
            2 => row.room_wait.as_mut().unwrap().order = 0,
            3 => row.room_wait.as_mut().unwrap().order = 1,
            _ => unreachable!(),
        }
        h.domain = Domain::new(&h.env.limits, crate::tests::KEPT, 91, 1, super::charters(), Box::new([0]));
        let rows: Vec<_> = h.stored.values().filter(|row| row.key().live()).cloned().collect();
        for record in rows {
            h.send(Event::Restore { record });
        }
        let out = h.send(Event::Check);
        assert!(
            out.iter().any(|request| match request {
                Request::RestoreRefused { .. } => true,
                _ => false,
            }),
            "{invalid}: {out:?}"
        );
        assert!(!h.domain.ready());
    }
}

#[test]
fn freeing_count_requires_the_batch_cause_and_matching_refusal() {
    for (cause, why, expected) in [
        (CapacityCause::Tasks { limit: 2 }, Refusal::Live, Some(LiveRoom::Deployment)),
        (CapacityCause::ProjectTasks { limit: 2 }, Refusal::Project, Some(LiveRoom::Project(1))),
        (CapacityCause::Tasks { limit: 2 }, Refusal::Project, None),
        (CapacityCause::ProjectTasks { limit: 2 }, Refusal::Live, None),
        (CapacityCause::Tree { limit: 2 }, Refusal::Tree, None),
    ] {
        let mut capacity = Capacity {
            project: Some(1),
            by: Party::Task(1),
            action: CapacityAction::Batch { first: Some(2), members: 1 },
            cause,
        };
        assert_eq!(Problem::capacity(None, why, capacity).freeing_count(), expected);
        capacity.action = CapacityAction::Message { task: 1, number: 2 };
        assert_eq!(Problem::capacity(None, why, capacity).freeing_count(), None);
    }
    assert_eq!(Problem::new(None, Refusal::Live).freeing_count(), None);
}
