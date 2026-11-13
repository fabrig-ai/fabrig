//! Decisions change membership before their public readers (jig/docs/design/domain/people/public-reading.md, 7).
use super::*;

#[test]
fn untracking_a_goal_takes_its_tree_out() {
    let (mut core, limits) = fixture();
    visibility(&mut core, &limits, true);
    make(&mut core, &limits, 1, None, Some(1), false);
    make(&mut core, &limits, 2, Some(1), None, false);
    assert_eq!(ready(&core).public_tasks().as_ref(), &[1, 2]);
    amend(&mut core, &limits, 1, Some(tasks::Tracking::Untracked), None);
    let project = ready(&core);
    assert_eq!((project.live()).len(), 0);
    assert!(project.public_tasks().is_empty());
    for task in [1, 2] {
        assert_eq!(project.goal_of(task), None);
        assert_eq!(project.lineage(task), None);
        assert_eq!(project.delegates(task), ([].as_slice(), 0));
        assert_eq!(project.counts(task), zero());
    }
}

#[test]
fn tracking_a_task_makes_it_public() {
    let (mut core, limits) = fixture();
    visibility(&mut core, &limits, true);
    make(&mut core, &limits, 1, None, None, true);
    make(&mut core, &limits, 2, Some(1), None, false);
    make(&mut core, &limits, 3, Some(2), None, false);
    assert!(ready(&core).public_tasks().is_empty());
    amend(&mut core, &limits, 2, Some(tasks::Tracking::Tracked { priority: 7 }), None);
    let project = ready(&core);
    assert_eq!(project.live(), &[2]);
    assert_eq!(project.public_tasks().as_ref(), &[2, 3]);
    assert_eq!(project.line(2).unwrap().parent, None);
    assert_eq!(project.lineage(3), Some([2].as_slice()));
}

#[test]
fn a_moved_task_leaves_the_tree_it_left() {
    let (mut core, limits) = fixture();
    visibility(&mut core, &limits, true);
    make(&mut core, &limits, 1, None, Some(1), false);
    make(&mut core, &limits, 2, Some(1), Some(2), false);
    make(&mut core, &limits, 3, Some(2), None, false);
    send(
        &mut core,
        &limits,
        tasks::Event::Move {
            operation: tasks::Move { task: 2, reason: Box::new([]) },
            reply_to: reply(),
            person: 1,
            period: 1,
            pool_budget: 500,
            period_budget: 1000,
        },
    );
    let project = ready(&core);
    assert_eq!(project.live(), &[1, 2]);
    assert_eq!(project.tree(1), Some(([].as_slice(), 0)));
    assert_eq!(project.delegates(1), ([].as_slice(), 0));
    assert_eq!(project.counts(1), zero());
    assert_eq!(project.tree(2), Some(([3].as_slice(), 0)));
    assert_eq!(project.goal_of(3), Some(2));
    assert_eq!(project.lineage(3), Some([2].as_slice()));
    assert_eq!(project.line(2).unwrap().parent, None);
}

#[test]
fn an_ended_goal_is_frozen_and_the_oldest_past_the_bound_leaves() {
    let (mut core, mut limits) = fixture();
    limits.public.ended = 1;
    visibility(&mut core, &limits, true);
    make(&mut core, &limits, 1, None, Some(1), false);
    make(&mut core, &limits, 2, Some(1), None, false);
    end(&mut core, &limits, 2);
    assert_eq!(ready(&core).counts(1), PublicCounts { ended: 1, ..zero() });
    assert_eq!(ready(&core).tree(1), Some(([2].as_slice(), 0)));
    end(&mut core, &limits, 1);
    let frozen = snapshot(ready(&core));
    let project = ready(&core);
    assert_eq!((project.live()).len(), 0);
    assert_eq!(project.ended().len(), 1);
    assert_eq!(project.ended()[0].line.task, 1);
    assert_eq!(project.ended()[0].tree[0].task, 2);
    assert_eq!(project.ended()[0].folded, 0);
    assert_eq!(project.goal_of(2), Some(1));
    assert_eq!(project.tree(1), Some(([2].as_slice(), 0)));
    assert_eq!(project.lineage(2), Some([1].as_slice()));
    make(&mut core, &limits, 3, None, Some(2), false);
    for line in &frozen {
        assert_eq!(ready(&core).line(line.task), Some(line));
    }
    end(&mut core, &limits, 3);
    let project = ready(&core);
    assert_eq!(project.ended().len(), 1);
    assert_eq!(project.ended()[0].line.task, 3);
    assert_eq!(project.public_tasks().as_ref(), &[3]);
    for task in [1, 2] {
        assert_eq!(project.goal_of(task), None);
        assert_eq!(project.line(task), None);
        assert_eq!(project.lineage(task), None);
    }
    visibility(&mut core, &limits, false);
    visibility(&mut core, &limits, true);
    assert!(ready(&core).ended().is_empty(), "publication begins with no ended history");
}

#[test]
fn a_private_a_deleting_and_an_unknown_project_are_the_same_miss() {
    let (mut core, limits) = fixture();
    make(&mut core, &limits, 1, None, Some(1), false);
    assert!(core.public_index.project(1).is_none());
    assert!(core.public_index.project(999).is_none());
    visibility(&mut core, &limits, true);
    assert_eq!(ready(&core).live(), &[1]);
    visibility(&mut core, &limits, false);
    assert!(core.public_index.project(1).is_none());
    visibility(&mut core, &limits, true);
    let mut work = Queue::uncapped();
    let mut out = Queue::uncapped();
    crate::admission::deletion::begin(&mut core, &environment(&limits), &mut work, &mut out, Token::new(902), 1, None)
        .unwrap();
    for project in [0, 1, 999] {
        assert!(core.public_index.project(project).is_none());
    }
}

#[test]
fn a_project_is_busy_until_its_entry_is_built() {
    let (_, limits) = fixture();
    let mut core = Core::new(crate::tests::config(41), &limits).unwrap();
    let action = core.start_begin(false, Wall::EPOCH);
    core.counters.deployment.projects = 1;
    assert_eq!(
        core.restore(
            &environment(&limits),
            crate::Record::Core(crate::CoreRecord::Deployment(core.counters.deployment()))
        ),
        crate::Restore::Deployment { commits: 0 }
    );
    let value =
        crate::admission::policy::snapshot(&crate::tests::config(41).seed_projects[0].policy.policy, &[], &[]).unwrap();
    assert_eq!(
        core.restore(
            &environment(&limits),
            crate::Record::People(people::Stored::Policy { project: 1, name: Box::from("Project"), value })
        ),
        crate::Restore::Kept
    );
    assert_eq!(core.start_phase(), Some(crate::start::Phase::LoadCore));
    assert_eq!(
        core.restore(&environment(&limits), crate::Record::People(people::Stored::Public { project: 1 })),
        crate::Restore::Kept
    );
    match core.public_index.project(1) {
        Some(Entry::Busy) => {}
        Some(Entry::Ready(_)) | None => panic!("restored entry waits for Open"),
    }
    // The start script builds each restored public project in its Open phase.
    let _out = crate::tests::finish_started(&mut core, &environment(&limits), action);
    assert_eq!((ready(&core).live()).len(), 0);
    assert!(ready(&core).ended().is_empty());
}

#[test]
fn an_ended_nested_goal_is_not_promoted_into_the_live_list() {
    let (mut core, limits) = fixture();
    visibility(&mut core, &limits, true);
    make(&mut core, &limits, 1, None, Some(1), false);
    make(&mut core, &limits, 2, Some(1), Some(2), false);
    end(&mut core, &limits, 2);
    assert_eq!(ready(&core).tree(1), Some(([2].as_slice(), 0)));
    amend(&mut core, &limits, 1, Some(tasks::Tracking::Untracked), None);
    assert_eq!((ready(&core).live()).len(), 0);
    assert!(ready(&core).public_tasks().is_empty());
}

#[test]
fn a_goal_folded_out_of_the_live_list_can_enter_the_ended_list() {
    let (mut core, mut limits) = fixture();
    limits.public.goals = 1;
    limits.public.ended = 1;
    visibility(&mut core, &limits, true);
    make(&mut core, &limits, 1, None, Some(1), false);
    make(&mut core, &limits, 2, None, Some(2), false);
    make(&mut core, &limits, 3, Some(2), None, false);
    assert_eq!(ready(&core).live(), &[1]);
    assert_eq!(ready(&core).live_folded(), 1);
    assert_eq!(ready(&core).goal_of(2), None);
    end(&mut core, &limits, 3);
    end(&mut core, &limits, 2);
    let project = ready(&core);
    assert_eq!(project.live(), &[1]);
    assert_eq!(project.live_folded(), 0);
    assert_eq!(project.ended().len(), 1);
    assert_eq!(project.ended()[0].line.task, 2);
    assert!(project.ended()[0].tree.is_empty());
    assert_eq!(project.ended()[0].folded, 0);
    assert_eq!(project.counts(2), zero());
    assert_eq!(project.goal_of(3), None);
    assert_eq!(project.goal_of(2), Some(2));
    assert_eq!(project.public_tasks().as_ref(), &[1, 2]);
    assert_eq!(project.lineage(2), Some([].as_slice()));
}

#[test]
fn ended_cards_follow_the_recent_list_when_its_positions_change() {
    let (mut core, limits) = fixture();
    visibility(&mut core, &limits, true);
    make(&mut core, &limits, 1, None, Some(1), false);
    end(&mut core, &limits, 1);
    let first = ready(&core).line(1).unwrap().clone();
    make(&mut core, &limits, 2, None, Some(2), false);
    end(&mut core, &limits, 2);
    assert_eq!(ready(&core).line(1), Some(&first));
    assert_eq!(ready(&core).ended()[0].line.task, 2);
    assert_eq!(ready(&core).ended()[1].line.task, 1);
    let second = ready(&core).line(2).unwrap().clone();
    make(&mut core, &limits, 3, None, Some(3), false);
    end(&mut core, &limits, 3);
    let project = ready(&core);
    assert_eq!(project.line(2), Some(&second));
    assert_eq!(project.line(1), None);
    assert_eq!(project.goal_of(1), None);
    assert_eq!(project.public_tasks().as_ref(), &[2, 3]);
}
