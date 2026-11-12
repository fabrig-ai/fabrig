//! Preparation failures keep their class and reason through backoff and restart
//! (`jig/docs/design/domain/engine/briefs.md`, 1.1).
use super::*;
use jig_domain_brief::{GatherFailure, GatherFate};
use skein_lib::Duration;

fn permanent() -> GatherFailure {
    GatherFailure {
        fate: GatherFate::Permanent,
        section: b"task",
        words: Box::from(b"could not be read within its bounds".as_slice()),
    }
}
fn capacity() -> GatherFailure {
    GatherFailure {
        fate: GatherFate::Capacity,
        section: b"brief",
        words: Box::from(b"no brief slot is free".as_slice()),
    }
}

fn failing_brief(failure: GatherFailure, seed: u64) -> (World, u64) {
    let mut world = plan_world(seed);
    world.domain.script_brief_failure(Some(failure));
    let parent = world.assignments[0];
    let child = member(&world, tasks::Executor::Agent { charter: 1 }, b"prepare my brief");
    let task = delegate(&mut world, parent, 1, child);
    (world, task)
}

fn logged(world: &mut World, task: u64, failure: GatherFailure) {
    assert_eq!(world.domain.take_brief_failure(), Some((task, failure)));
    assert_eq!(world.domain.take_brief_failure(), None, "each failure reaches the root once");
}

fn held(world: &mut World, task: u64) {
    let allowance = world.env().limits.core.tasks.retries.permanent.retries;
    for _ in 0..allowance {
        logged(world, task, permanent());
        world.advance(Duration::from_secs(2));
    }
    logged(world, task, permanent());
}

#[test]
fn a_brief_that_cannot_be_read_counts_to_a_hold_naming_it() {
    let (mut world, task) = failing_brief(permanent(), 924);
    let allowance = world.env().limits.core.tasks.retries.permanent.retries;
    assert_eq!(
        live_row(&world, task).retry_notice,
        Some(tasks::RetryNotice::Failed {
            class: tasks::Class::Permanent,
            count: 1,
            allowance,
            reason: Some(permanent().reason()),
        })
    );
    held(&mut world, task);
    assert!(matches!(&live_row(&world, task).phase, tasks::Phase::Held {
        why: tasks::Hold::Failures(tasks::Class::Permanent), source: Some(source), ..
    } if **source == tasks::HoldSource::Preparation { words: permanent().reason() }));
    assert!(!world.assignments.iter().any(|(number, _)| *number == task));
    world.reads = Some(jig_core_world::reads::Parties::new(world.store.applied));
    jig_core_world::reads::ask(&mut world, 91, core::reads::Read::Task { task });
    let core::reads::Answer::Document(document) = jig_core_world::reads::latest(&world) else {
        panic!("task document")
    };
    let core::documents::Document::Task(document) = document.as_ref() else { panic!("task document") };
    let hold = document.hold.as_ref().expect("failure hold on document");
    assert_eq!(hold.reason, tasks::Hold::Failures(tasks::Class::Permanent));
    assert_eq!(hold.words.as_ref().expect("preparation words").bytes.as_ref(), permanent().reason().as_ref());
    world.close();
}

#[test]
fn no_room_for_a_brief_pauses_past_any_allowance_and_then_runs() {
    let (mut world, task) = failing_brief(capacity(), 925);
    let retries = world.env().limits.core.tasks.retries;
    let allowance = [retries.transient, retries.permanent, retries.run, retries.agent, retries.lost, retries.invalid, retries.infrastructure]
        .iter()
        .map(|retry| retry.retries)
        .max()
        .expect("failure classes");
    for count in 1..=allowance + 2 {
        logged(&mut world, task, capacity());
        assert_eq!(
            live_row(&world, task).retry_notice,
            Some(tasks::RetryNotice::Refused { count, reason: Some(capacity().reason()) })
        );
        assert_eq!(live_row(&world, task).tries, tasks::Tries::NONE);
        assert!(matches!(live_row(&world, task).phase, tasks::Phase::Active(tasks::Active::BackingOff { .. })));
        assert!(!world.assignments.iter().any(|(number, _)| *number == task));
        world.advance(Duration::from_secs(2));
    }
    logged(&mut world, task, capacity());
    world.domain.script_brief_failure(None);
    world.advance(Duration::from_secs(2));
    let run = running(&world, task);
    finish(&mut world, run, b"brief ready");
    world.close();
}

#[test]
fn a_release_counts_again_and_runs() {
    let (mut world, task) = failing_brief(permanent(), 926);
    held(&mut world, task);
    // A release starts the class's allowance afresh even if the system still refuses.
    let revision = live_row(&world, task).revisions.hold;
    ask(&mut world, 92, people::Ask::Release { project: 1, task, revision, again: None });
    logged(&mut world, task, permanent());
    assert!(matches!(
        live_row(&world, task).retry_notice,
        Some(tasks::RetryNotice::Failed { class: tasks::Class::Permanent, count: 1, .. })
    ));
    world.domain.script_brief_failure(None);
    world.advance(Duration::from_secs(2));
    let run = running(&world, task);
    finish(&mut world, run, b"released brief ready");
    world.close();
}

#[test]
fn a_late_section_counts_as_infrastructure() {
    let failure = GatherFailure {
        fate: GatherFate::Infrastructure,
        section: b"context",
        words: Box::from(b"not gathered by its deadline".as_slice()),
    };
    let (mut world, task) = failing_brief(failure.clone(), 927);
    let allowance = world.env().limits.core.tasks.retries.infrastructure.retries;
    assert_eq!(
        live_row(&world, task).retry_notice,
        Some(tasks::RetryNotice::Failed {
            class: tasks::Class::Infrastructure,
            count: 1,
            allowance,
            reason: Some(failure.reason()),
        })
    );
    logged(&mut world, task, failure);
    assert_eq!(live_row(&world, task).tries.permanent, 0);
    assert_eq!(live_row(&world, task).tries.transient, 0);
    assert_eq!(live_row(&world, task).tries.infrastructure, 1);
    assert!(!world.assignments.iter().any(|(number, _)| *number == task));
    world.domain.script_brief_failure(None);
    world.advance(Duration::from_secs(10));
    let run = running(&world, task);
    finish(&mut world, run, b"late section arrived");
    world.close();
}
