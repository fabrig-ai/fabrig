//! Prefixes, folded counts and zero bounds (jig/docs/design/domain/people/public-reading.md, 9).
use super::*;

#[test]
fn a_task_past_the_depth_or_the_lines_is_not_public() {
    for (lines, depth, expected_tree, folded, excluded) in
        [(8, 1, [2, 4].as_slice(), 1, [3].as_slice()), (1, 4, [2].as_slice(), 2, [3, 4].as_slice())]
    {
        let (mut core, mut limits) = fixture();
        limits.public.lines = lines;
        limits.public.depth = depth;
        visibility(&mut core, &limits, true);
        make(&mut core, &limits, 1, None, Some(1), false);
        make(&mut core, &limits, 2, Some(1), None, false);
        make(&mut core, &limits, 3, Some(2), None, false);
        make(&mut core, &limits, 4, Some(1), None, false);
        let project = ready(&core);
        assert_eq!(project.tree(1), Some((expected_tree, folded)));
        for &task in excluded {
            assert_eq!(project.goal_of(task), None);
            assert_eq!(project.lineage(task), None);
            assert_eq!(project.delegates(task), ([].as_slice(), 0));
            assert_eq!(project.counts(task), zero());
            assert!(!project.public_tasks().contains(&task));
        }
        assert_eq!(project.delegates(2), ([].as_slice(), 1));
        assert_eq!(project.counts(2), zero(), "folded delegates never enter phase counts");
        let direct = if lines == 1 { 1 } else { 2 };
        assert_eq!(project.counts(1), PublicCounts { working: direct, ..zero() });
        assert_eq!(project.delegates(1).1, u32::from(lines == 1));
    }
}

#[test]
fn live_goals_keep_their_first_by_priority_and_fold_the_rest() {
    let (mut core, mut limits) = fixture();
    limits.public.goals = 2;
    visibility(&mut core, &limits, true);
    for (task, priority) in [(1, 3), (2, 1), (3, 2)] {
        make(&mut core, &limits, task, None, Some(priority), false);
    }
    let project = ready(&core);
    assert_eq!(project.live(), &[2, 3]);
    assert_eq!(project.live_folded(), 1);
    assert_eq!(project.goal_of(1), None);
    assert_eq!(project.public_tasks().as_ref(), &[2, 3]);
    amend(&mut core, &limits, 1, Some(tasks::Tracking::Tracked { priority: 0 }), None);
    let project = ready(&core);
    assert_eq!(project.live(), &[1, 2]);
    assert_eq!(project.live_folded(), 1);
    assert_eq!(project.goal_of(3), None);
    assert_eq!(project.public_tasks().as_ref(), &[1, 2]);
}

#[test]
fn with_every_bound_zero_nothing_is_public() {
    let (mut core, mut limits) = fixture();
    limits.public = PublicIndexLimits::OFF;
    visibility(&mut core, &limits, true);
    make(&mut core, &limits, 1, None, Some(1), false);
    make(&mut core, &limits, 2, Some(1), None, false);
    assert_eq!((ready(&core).live()).len(), 0);
    assert!(ready(&core).ended().is_empty());
    assert!(ready(&core).public_tasks().is_empty());
    for task in [1, 2] {
        assert_eq!(ready(&core).goal_of(task), None);
        assert_eq!(ready(&core).counts(task), zero());
    }
    end(&mut core, &limits, 2);
    end(&mut core, &limits, 1);
    assert!(ready(&core).ended().is_empty());
    assert!(ready(&core).public_tasks().is_empty());
}

#[test]
fn the_index_stays_within_its_bounds() {
    let (mut core, mut limits) = fixture();
    limits.public = PublicIndexLimits { goals: 1, ended: 1, lines: 1, depth: 1 };
    visibility(&mut core, &limits, true);
    make(&mut core, &limits, 1, None, Some(1), false);
    make(&mut core, &limits, 2, Some(1), None, false);
    make(&mut core, &limits, 3, Some(1), None, false);
    make(&mut core, &limits, 4, None, Some(2), false);
    assert_eq!(ready(&core).live(), &[1]);
    assert_eq!(ready(&core).live_folded(), 1);
    assert_eq!(ready(&core).tree(1), Some(([2].as_slice(), 1)));
    assert_eq!(ready(&core).public_tasks().len(), 2);
    end(&mut core, &limits, 3);
    end(&mut core, &limits, 2);
    end(&mut core, &limits, 1);
    assert_eq!(ready(&core).live(), &[4]);
    assert_eq!(ready(&core).ended().len(), 1);
    assert_eq!(ready(&core).ended()[0].tree.len(), 1);
    assert_eq!(ready(&core).public_tasks().len(), 3);
    end(&mut core, &limits, 4);
    assert_eq!(ready(&core).ended().len(), 1);
    assert_eq!(ready(&core).public_tasks().as_ref(), &[4]);
    assert!(core.public_index.project(2).is_none());
}

#[test]
fn public_entries_stop_at_the_projects_bound_even_during_restore() {
    let (_, mut limits) = fixture();
    limits.authority.projects = 3;
    assert_eq!(limits.people.projects, 2);
    let configured = crate::tests::config(41);
    let value = crate::admission::policy::snapshot(&configured.seed_projects[0].policy.policy, &[], &[]).unwrap();
    let mut core = Core::new(configured, &limits).unwrap();
    let _action = core.start_begin(false, Wall::EPOCH);
    for (project, name) in [(1, "First"), (2, "Second"), (3, "Third")] {
        assert_eq!(
            core.restore(
                &environment(&limits),
                crate::Record::People(people::Stored::Policy { project, name: Box::from(name), value: value.clone() })
            ),
            crate::Restore::Kept
        );
        assert_eq!(
            core.restore(&environment(&limits), crate::Record::People(people::Stored::Public { project })),
            crate::Restore::Kept
        );
    }
    for project in [1, 2] {
        match core.public_index.project(project) {
            Some(Entry::Busy) => {}
            Some(Entry::Ready(_)) | None => panic!("restored public project within entry bound"),
        }
    }
    assert!(core.public_index.project(3).is_none());
    assert!(core.public_index.project(999).is_none());
}
