//! A landing's end wakes its train alone (domain/landing.md, 4.2 and 9).
use super::super::{Work, forge, landing, landing_route, tasks};
use alloc::boxed::Box;
use skein_lib::{List, Queue};

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
        forge::step(
            &mut domain.forge,
            &crate::translate::environment_forge(&env),
            forge::Event::Restore { record },
            &mut Queue::uncapped(),
        );
    }
    // Admit one main member without driving other trains or starting any reads.
    super::super::start_landing(&mut domain, &env, 7);
    let Some(Work::Core(resource)) = domain.work.pop() else { panic!("branch resource") };
    drop(jig_domain_core::step(&mut domain.core, &crate::limits::environment_core(&env), resource));
    let Some(Work::Core(event)) = domain.work.pop() else { panic!("landing task") };
    drop(jig_domain_core::step(&mut domain.core, &crate::limits::environment_core(&env), event));
    let Some(Work::LandingAdmission { mut row, .. }) = domain.work.pop() else { panic!("landing admission") };
    assert_eq!(row.landing.members.len(), 1);
    assert_eq!(row.landing.members[0].change, 7);
    let task = row.landing.task;
    row.landing.phase =
        if choice == landing::Decision::Finish { landing::Phase::Landed } else { landing::Phase::Cancelled };
    row.landing.members[0].left = Some(landing::Leave::Cancelled);
    forge::step(
        &mut domain.forge,
        &crate::translate::environment_forge(&env),
        forge::Event::Restore { record: forge::Stored::Landing(row) },
        &mut Queue::uncapped(),
    );
    // A member that left is unlinked and ready again when the end is routed.
    assert_eq!(domain.forge.change(7).unwrap().change.landing, None);
    landing_route::landing_decision(&mut domain, &env, task, choice);
    let mut wakes = List::uncapped();
    while let Some(work) = domain.work.pop() {
        if let Work::Tasks(tasks::Event::WakeProcedure { task }) = work {
            wakes.push(task).unwrap();
        }
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
