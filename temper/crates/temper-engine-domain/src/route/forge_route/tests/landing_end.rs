//! A landing's end wakes its train alone (domain/landing.md, 4.2, and domain/landing-turn.md, 2).
use super::super::{Work, forge, landing, landing_route, tasks};
use alloc::boxed::Box;
use skein_lib::List;

const STUCK: landing::Stuck = landing::Stuck::Train { landing: 41 };
const REASON: &[u8] = b"cannot move: landing 41 holds its train";

fn ending_wakes(choice: landing::Decision) {
    let (mut domain, env) = super::due_ready::ready_changes(&[7, 8, 9, 10], 1);
    let mut goal = domain.forge.change(9).unwrap().clone();
    goal.base = b"goal".as_slice().into();
    let mut other_repository = domain.forge.repository(super::adopted(1).provider).unwrap().clone();
    other_repository.provider.repository = 2;
    other_repository.name = b"other".as_slice().into();
    other_repository.home = false;
    let mut other = domain.forge.change(10).unwrap().clone();
    other.repository = other_repository.provider;
    for record in [
        forge::Stored::Repository(other_repository),
        forge::Stored::Change(Box::new(goal)),
        forge::Stored::Change(Box::new(other)),
    ] {
        super::restore_forge_with_engine_waits(&mut domain, &env, record);
    }
    // Admit one main member without driving other trains or starting any reads.
    super::super::start_landing(&mut domain, &env, 7);
    let (mut row, _) = super::take_landing_admission(&mut domain, &env);
    assert_eq!(row.landing.members.len(), 1);
    assert_eq!(row.landing.members[0].change, 7);
    let task = row.landing.task;
    row.landing.phase =
        if choice == landing::Decision::Finish { landing::Phase::Landed } else { landing::Phase::Cancelled };
    row.landing.members[0].left = Some(landing::Leave::Cancelled);
    super::restore_forge_with_engine_waits(&mut domain, &env, forge::Stored::Landing(row));
    // A member that left is unlinked and ready again when the end is routed.
    assert_eq!(domain.forge.landing_of(7), None);
    landing_route::landing_decision(&mut domain, &env, task, choice, &[]);
    let mut wakes = List::uncapped();
    let mut cancel = None;
    while let Some(work) = domain.work.pop() {
        if let Work::Tasks(tasks::Event::WakeProcedure { task }) = work {
            wakes.push(task).unwrap();
        } else if let Work::Tasks(event @ tasks::Event::Cancel { .. }) = work {
            assert!(cancel.replace(event).is_none(), "one cancellation");
        }
    }
    if choice == landing::Decision::End(STUCK) {
        let event = cancel.expect("asks jig for cancellation");
        let tasks::Event::Cancel { by, operation, .. } = &event else { unreachable!() };
        assert_eq!(*by, tasks::Party::Deployment { project: 1 });
        assert_eq!(operation.task, task);
        assert_eq!(operation.reason.as_ref(), REASON);
        super::task_event(&mut domain, &env, event);
        let tasks::Phase::Closing(closing) = &domain.engine.core().tasks.task(task).expect("landing task").phase else {
            panic!("jig accepted the cancel")
        };
        let tasks::Ending::Cancelled { reason, .. } = &closing.ending else { panic!("cancelled ending") };
        assert_eq!(reason.as_ref(), REASON);

        let mut change = domain.forge.change(7).expect("member").clone();
        change.goal = Some(9);
        super::restore_forge_with_engine_waits(&mut domain, &env, forge::Stored::Change(Box::new(change)));
        let step = forge::LandingNews::Cancelled(STUCK);
        super::forge_request(&mut domain, &env, forge::Request::LandingNews { task, member: 0, step });
        let Some(Work::Tasks(tasks::Event::Notice { task: goal, word })) = domain.work.pop() else {
            panic!("goal gets news")
        };
        assert_eq!(goal, 9);
        assert_eq!(word.kind, tasks::MessageKind::News { subscription: 0, class: tasks::NewsClass::Kept });
        let words =
            skein_lib::bytes::concat(&[b"landing ", skein_lib::Decimal::of(task).as_bytes(), b": cancelled: ", REASON]);
        assert_eq!(word.words, words);
        assert!(domain.work.pop().is_none());
    }
    assert!(wakes.as_slice().contains(&7), "former member wakes: {wakes:?}");
    assert!(wakes.as_slice().contains(&8), "own train's waiter wakes: {wakes:?}");
    assert!(!wakes.as_slice().contains(&9), "another branch's waiter stays asleep: {wakes:?}");
    assert!(!wakes.as_slice().contains(&10), "another repository's waiter stays asleep: {wakes:?}");
}

#[test]
fn a_finished_landing_wakes_only_its_own_train() {
    ending_wakes(landing::Decision::Finish);
}

#[test]
fn a_cancelled_landing_wakes_only_its_own_train() {
    ending_wakes(landing::Decision::Cancel);
}

#[test]
fn a_liveness_ending_cancels_with_its_reason_tells_the_goal_and_wakes_its_train() {
    ending_wakes(landing::Decision::End(STUCK));
}
