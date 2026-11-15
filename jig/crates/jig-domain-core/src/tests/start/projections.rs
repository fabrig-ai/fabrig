//! Stored projection fits and links (jig/docs/design/domain/engine/restart.md).
#![expect(clippy::disallowed_macros, reason = "dynamic stored-row fixtures exercise byte bounds")]
use crate::{
    CoreKey, CoreRecord, Limits, MilestoneId, ProjectionGoalHot, ProjectionMilestone, ProjectionRecord,
    ProjectionTaskHot, Restored,
    start::{Linked, LiveSet},
    tests::start::{Core, Failure, Key, Wall, config, limits, tasks},
};

fn loaded() -> (Core, Limits) {
    loaded_policy(true)
}

fn loaded_policy(policy: bool) -> (Core, Limits) {
    let limits = limits();
    let configured = config(401);
    let project = &config(401).seed_projects[0];
    let mut core = Core::new(configured, &limits).expect("configured core");
    {
        let _begin = core.start_begin(false, Wall::EPOCH);
    }
    let mut header = core.counters.deployment();
    header.tasks = 100;
    header.projects = 100;
    header.commits = 1;
    assert_eq!(core.restore_core(CoreRecord::Deployment(header), &limits), Restored::Deployment { commits: 1 });
    if policy {
        assert!(core.restore_policy(
            &limits,
            project.number,
            project.name.clone(),
            crate::admission::policy::snapshot(&project.policy.policy, &[], &[]).expect("bounded policy")
        ));
    }
    (core, limits)
}

fn ending() -> tasks::Phase {
    tasks::Phase::Ended(tasks::Ending::Failed { reason: Box::new([]) })
}
fn goal(number: u64) -> ProjectionRecord {
    ProjectionRecord::GoalHot {
        goal: ProjectionGoalHot { number, project: 1, priority: 1, phase: ending() },
        pending: Box::new([]),
        changed: false,
        fed_at: Some(1),
    }
}
fn task(number: u64) -> ProjectionRecord {
    ProjectionRecord::TaskHot {
        goal: 1,
        task: ProjectionTaskHot { number, requester: tasks::Party::Task(1), phase: ending() },
    }
}
fn milestone(task: u64) -> ProjectionRecord {
    ProjectionRecord::Milestone {
        goal: 1,
        milestone: ProjectionMilestone {
            identity: MilestoneId::Lifecycle { task, position: 1 },
            phase: Some(ending()),
            change: None,
            words: Box::new([]),
        },
    }
}
fn restore(core: &mut Core, limits: &Limits, row: ProjectionRecord) -> Restored {
    core.restore_core(CoreRecord::Projection(row), limits)
}

// The whole fixtures' words are put beside their hot rows. Parts tests use `restore` directly.
fn restore_fixture(core: &mut Core, limits: &Limits, row: ProjectionRecord) -> Restored {
    let words = match &row {
        ProjectionRecord::GoalHot { goal, .. } => Some(goal_words(goal.number)),
        ProjectionRecord::TaskHot { goal, task } => {
            Some(ProjectionRecord::TaskWords { goal: *goal, task: task.number, words: b"task".as_slice().into() })
        }
        ProjectionRecord::GoalWords { .. }
        | ProjectionRecord::TaskWords { .. }
        | ProjectionRecord::Milestone { .. } => None,
    };
    let result = restore(core, limits, row);
    if result == Restored::Live
        && let Some(words) = words
    {
        assert_eq!(restore(core, limits, words), Restored::Live);
    }
    result
}

fn goal_words(goal: u64) -> ProjectionRecord {
    ProjectionRecord::GoalWords { goal, task: goal, words: b"goal".as_slice().into() }
}
fn task_words(task: u64) -> ProjectionRecord {
    ProjectionRecord::TaskWords { goal: 1, task, words: b"task".as_slice().into() }
}

fn retained(row: ProjectionRecord, header: bool) {
    let (mut started, limits) = loaded();
    if header {
        assert_eq!(restore_fixture(&mut started, &limits, goal(1)), Restored::Live);
    }
    assert_eq!(restore_fixture(&mut started, &limits, row), Restored::Live);
    assert_eq!(started.start_failure(), None);
    started.start_projections_link();
    assert_eq!(started.projections.len(), 1);
}

#[test]
fn retained_numbers_and_feed_positions_are_not_admission_rules() {
    for number in [0, 101] {
        retained(goal(number), false);
        retained(task(number), true);
        retained(milestone(number), true);
    }
    let mut row = goal(1);
    let ProjectionRecord::GoalHot { fed_at, .. } = &mut row else { unreachable!() };
    *fed_at = Some(2);
    retained(row, false);
}

#[test]
fn milestone_shape_and_zero_position_are_not_admission_rules() {
    for position in [0, 1] {
        let row = ProjectionRecord::Milestone {
            goal: 1,
            milestone: ProjectionMilestone {
                identity: MilestoneId::Lifecycle { task: 1, position },
                phase: None,
                change: None,
                words: b"history".as_slice().into(),
            },
        };
        retained(row, true);
    }
    retained(
        ProjectionRecord::Milestone {
            goal: 1,
            milestone: ProjectionMilestone {
                identity: MilestoneId::Revision { task: 1, revision: 0 },
                phase: Some(ending()),
                change: None,
                words: Box::new([]),
            },
        },
        true,
    );
}

#[test]
fn policy_is_linked_after_loading_and_deployment_needs_none() {
    let (mut core, limits) = loaded_policy(false);
    let project = &config(401).seed_projects[0];
    assert_eq!(restore_fixture(&mut core, &limits, goal(1)), Restored::Live);
    assert!(core.restore_policy(
        &limits,
        1,
        project.name.clone(),
        crate::admission::policy::snapshot(&project.policy.policy, &[], &[]).expect("bounded policy")
    ));
    core.start_projections_link();
    assert_eq!(core.projections.get(&1).expect("linked goal").goal.project, 1);
    let mut row = goal(2);
    let ProjectionRecord::GoalHot { goal, .. } = &mut row else { unreachable!() };
    goal.project = crate::people::DEPLOYMENT;
    retained(row, false);
}

#[test]
fn pending_closing_projection_loads_without_its_live_task() {
    let mut row = goal(1);
    let ProjectionRecord::GoalHot { goal, pending, .. } = &mut row else { unreachable!() };
    goal.phase = tasks::Phase::Closing(tasks::Closing {
        stage: tasks::Stage::Effects,
        ending: tasks::Ending::Failed { reason: Box::new([]) },
    });
    *pending = Box::new([config(401).connectors[0]]);
    retained(row, false);
}

#[test]
fn pending_on_a_live_tracked_goal_is_retained() {
    let (mut source, limits) = crate::tests::core();
    crate::tests::make_task(&mut source, &limits, 1);
    let mut live = source.tasks.task(1).expect("live fixture").clone();
    live.tracked = Some(1);
    let mut row = goal(1);
    let ProjectionRecord::GoalHot { goal, pending, .. } = &mut row else { unreachable!() };
    goal.phase = live.phase.clone();
    *pending = Box::new([config(401).connectors[0]]);
    {
        let (mut core, limits) = loaded();
        let record = tasks::Stored::Live(Box::new(live.clone()));
        let mut out = super::Queue::uncapped();
        tasks::step(
            &mut core.tasks,
            &super::Env { now: super::Time::ZERO, wall: Wall::EPOCH, limits: limits.tasks },
            tasks::Event::Restore { record },
            &mut out,
        );
        assert_eq!(restore_fixture(&mut core, &limits, row.clone()), { Restored::Live });
        {
            core.start_projections_link();
            assert_eq!(
                core.projections.get(&1).expect("linked live projection").pending.as_ref(),
                config(401).connectors.as_ref().get(..1).expect("one connector")
            );
        }
    }
}

#[test]
fn pieces_in_any_order_finalize_to_the_default_projection() {
    let rows = [goal(1), goal_words(1), task(1), task_words(1), task(2), task_words(2), milestone(1), milestone(2)];
    let (mut expected, limits) = loaded();
    for row in &rows {
        assert_eq!(restore(&mut expected, &limits, row.clone()), Restored::Live);
    }
    expected.start_projections_link();
    assert_eq!(expected.projections.get(&1).unwrap().goal.words.as_ref(), b"goal");
    assert_eq!(expected.projections.get(&1).unwrap().plan[1].words.as_ref(), b"task");
    for order in [[7, 5, 3, 1, 6, 4, 2, 0], [1, 4, 3, 6, 0, 7, 5, 2]] {
        let (mut started, _) = loaded();
        for index in order {
            assert_eq!(restore(&mut started, &limits, rows[index].clone()), Restored::Live);
        }
        started.start_projections_link();
        assert_eq!(started.start_failure(), None);
        assert_eq!(started.projections.get(&1), expected.projections.get(&1));
        assert!(started.start.projections.is_none());
    }
}

#[test]
fn orphan_task_refuses_at_link_with_its_key() {
    let (mut core, limits) = loaded();
    assert_eq!(restore_fixture(&mut core, &limits, task(2)), Restored::Live);
    let assembly = core.start.projections.as_ref().expect("assembly").get(&1).expect("orphan");
    assert!(assembly.header.is_none());
    assert_eq!(
        assembly.tasks.get(&2),
        match task(2) {
            ProjectionRecord::TaskHot { ref task, .. } => Some(task),
            ProjectionRecord::GoalHot { .. }
            | ProjectionRecord::GoalWords { .. }
            | ProjectionRecord::TaskWords { .. }
            | ProjectionRecord::Milestone { .. } => unreachable!(),
        }
    );
    core.start_projections_link();
    assert_eq!(
        core.start_failure(),
        Some(Failure::Link {
            record: Key::Core(CoreKey::Projection(task(2).key())),
            linked: Linked::Record(Key::Core(CoreKey::Projection(crate::ProjectionKey::Goal(1))))
        })
    );
}

#[test]
fn orphan_milestone_refuses_at_link_with_its_key() {
    let (mut core, limits) = loaded();
    assert_eq!(restore_fixture(&mut core, &limits, milestone(2)), Restored::Live);
    core.start_projections_link();
    assert_eq!(
        core.start_failure(),
        Some(Failure::Link {
            record: Key::Core(CoreKey::Projection(milestone(2).key())),
            linked: Linked::Record(Key::Core(CoreKey::Projection(crate::ProjectionKey::Goal(1))))
        })
    );
}

fn refused(core: &mut Core, limits: &Limits, row: ProjectionRecord, held: u64, limit: u64) {
    let key = row.key();
    assert_eq!(restore_fixture(core, limits, row), Restored::Rejected);
    assert_eq!(
        core.start_failure(),
        Some(Failure::LiveSet { set: LiveSet::Projections, key: Key::Core(CoreKey::Projection(key)), held, limit })
    );
}

#[test]
fn table_capacity_counts_headerless_goals() {
    for headers in [false, true] {
        let (mut core, limits) = loaded();
        for number in 1..=core.projections.capacity() {
            let row = if headers {
                goal(u64::from(number))
            } else {
                ProjectionRecord::TaskHot {
                    goal: u64::from(number),
                    task: ProjectionTaskHot { number: 1, requester: tasks::Party::Task(1), phase: ending() },
                }
            };
            assert_eq!(restore_fixture(&mut core, &limits, row), Restored::Live);
        }
        let limit = u64::from(core.projections.capacity());
        refused(&mut core, &limits, goal(limit + 1), limit + 1, limit);
    }
}

#[test]
fn words_phase_and_pending_fits_keep_the_row_key() {
    let limits = limits();
    for row in [goal_words(1), task_words(2), milestone(2)] {
        let (mut core, _) = loaded();
        let mut row = row;
        let (words, bound) = match &mut row {
            ProjectionRecord::GoalWords { words, .. } | ProjectionRecord::TaskWords { words, .. } => {
                (words, limits.tasks.spec_bytes)
            }
            ProjectionRecord::GoalHot { .. } | ProjectionRecord::TaskHot { .. } => unreachable!(),
            ProjectionRecord::Milestone { milestone, .. } => {
                (&mut milestone.words, limits.tasks.message_bytes.max(limits.tasks.result_bytes))
            }
        };
        *words = vec![b'x'; skein_lib::length::usize_of(bound + 1)].into_boxed_slice();
        refused(&mut core, &limits, row, u64::from(bound) + 1, u64::from(bound));
    }
    let (mut core, _) = loaded();
    let mut row = goal(1);
    let ProjectionRecord::GoalHot { pending, .. } = &mut row else { unreachable!() };
    let bound = core.connectors.len();
    *pending = vec![1; bound + 1].into_boxed_slice();
    refused(
        &mut core,
        &limits,
        row,
        u64::try_from(bound + 1).expect("fixture count"),
        u64::try_from(bound).expect("fixture count"),
    );
    for mut row in [goal(1), task(2), milestone(2)] {
        let (mut core, _) = loaded();
        let phase = match &mut row {
            ProjectionRecord::GoalHot { goal, .. } => &mut goal.phase,
            ProjectionRecord::TaskHot { task, .. } => &mut task.phase,
            ProjectionRecord::Milestone { milestone, .. } => milestone.phase.as_mut().expect("fixture phase"),
            ProjectionRecord::GoalWords { .. } | ProjectionRecord::TaskWords { .. } => unreachable!(),
        };
        *phase = tasks::Phase::Ended(tasks::Ending::Failed {
            reason: vec![b'x'; skein_lib::length::usize_of(limits.tasks.result_bytes * 2 + 1)].into_boxed_slice(),
        });
        refused(
            &mut core,
            &limits,
            row,
            u64::from(limits.tasks.result_bytes) * 2 + 1,
            u64::from(limits.tasks.result_bytes) * 2,
        );
    }
}

#[test]
fn piece_limits_apply_before_and_after_headers() {
    for headers in [false, true] {
        for milestones in [false, true] {
            let (mut core, limits) = loaded();
            core.routes = 2;
            if headers {
                assert_eq!(restore_fixture(&mut core, &limits, goal(1)), Restored::Live);
            }
            let bound = if milestones { core.routes } else { limits.tasks.tree_tasks };
            for number in 1..=bound {
                assert_eq!(
                    restore_fixture(
                        &mut core,
                        &limits,
                        if milestones { milestone(u64::from(number)) } else { task(u64::from(number)) }
                    ),
                    Restored::Live
                );
            }
            refused(
                &mut core,
                &limits,
                if milestones { milestone(u64::from(bound) + 1) } else { task(u64::from(bound) + 1) },
                u64::from(bound) + 1,
                u64::from(bound),
            );
        }
    }
}

#[test]
fn a_missing_project_refuses_at_link() {
    let (mut core, limits) = loaded();
    let mut row = goal(1);
    let ProjectionRecord::GoalHot { goal, .. } = &mut row else { unreachable!() };
    goal.project = 2;
    assert_eq!(restore_fixture(&mut core, &limits, row), Restored::Live);
    core.start_projections_link();
    assert_eq!(
        core.start_failure(),
        Some(Failure::Link {
            record: Key::Core(CoreKey::Projection(crate::ProjectionKey::Goal(1))),
            linked: Linked::Project(2)
        })
    );
}

#[test]
fn a_missing_pending_connector_refuses_at_link() {
    let (mut core, limits) = loaded();
    let mut row = goal(1);
    let ProjectionRecord::GoalHot { pending, .. } = &mut row else { unreachable!() };
    *pending = Box::new([99]);
    assert_eq!(restore_fixture(&mut core, &limits, row), Restored::Live);
    core.start_projections_link();
    assert_eq!(
        core.start_failure(),
        Some(Failure::Link {
            record: Key::Core(CoreKey::Projection(crate::ProjectionKey::Goal(1))),
            linked: Linked::Connector(99)
        })
    );
}

#[test]
fn an_open_goal_requires_its_live_task_at_link() {
    let (mut core, limits) = loaded();
    let mut row = goal(1);
    let ProjectionRecord::GoalHot { goal, .. } = &mut row else { unreachable!() };
    goal.phase = tasks::Phase::Waiting;
    assert_eq!(restore_fixture(&mut core, &limits, row), Restored::Live);
    core.start_projections_link();
    assert_eq!(
        core.start_failure(),
        Some(Failure::Link {
            record: Key::Core(CoreKey::Projection(crate::ProjectionKey::Goal(1))),
            linked: Linked::Record(Key::Tasks(tasks::Key::Live(1)))
        })
    );
}

#[test]
fn duplicate_pending_numbers_are_stored_values_not_admission_rules() {
    let mut row = goal(1);
    let ProjectionRecord::GoalHot { pending, .. } = &mut row else { unreachable!() };
    *pending = Box::new([config(401).connectors[0]; 2]);
    retained(row, false);
}

#[test]
fn exact_word_and_phase_bounds_remain_loadable() {
    let (mut core, limits) = loaded();
    for mut row in [goal_words(1), task_words(2), milestone(2)] {
        let (words, bound) = match &mut row {
            ProjectionRecord::GoalWords { words, .. } | ProjectionRecord::TaskWords { words, .. } => {
                (words, limits.tasks.spec_bytes)
            }
            ProjectionRecord::Milestone { milestone, .. } => {
                (&mut milestone.words, limits.tasks.message_bytes.max(limits.tasks.result_bytes))
            }
            ProjectionRecord::GoalHot { .. } | ProjectionRecord::TaskHot { .. } => unreachable!(),
        };
        *words = vec![b'x'; skein_lib::length::usize_of(bound)].into_boxed_slice();
        if let ProjectionRecord::Milestone { milestone, .. } = &mut row {
            milestone.phase = Some(tasks::Phase::Ended(tasks::Ending::Failed {
                reason: vec![b'x'; skein_lib::length::usize_of(limits.tasks.result_bytes * 2)].into_boxed_slice(),
            }));
        }
        assert_eq!(restore(&mut core, &limits, row), Restored::Live);
    }
    for mut row in [goal(1), task(2)] {
        let phase = match &mut row {
            ProjectionRecord::GoalHot { goal, .. } => &mut goal.phase,
            ProjectionRecord::TaskHot { task, .. } => &mut task.phase,
            ProjectionRecord::GoalWords { .. }
            | ProjectionRecord::TaskWords { .. }
            | ProjectionRecord::Milestone { .. } => unreachable!(),
        };
        *phase = tasks::Phase::Ended(tasks::Ending::Failed {
            reason: vec![b'x'; skein_lib::length::usize_of(limits.tasks.result_bytes * 2)].into_boxed_slice(),
        });
        assert_eq!(restore(&mut core, &limits, row), Restored::Live);
    }
    core.start_projections_link();
    let projection = core.projections.get(&1).expect("projection at byte bounds");
    assert_eq!(projection.goal.words.len(), skein_lib::length::usize_of(limits.tasks.spec_bytes));
    assert_eq!(projection.plan.len(), 1);
    assert_eq!(projection.milestones.len(), 1);
}

#[test]
fn each_unjoined_part_refuses_at_link_with_its_missing_part() {
    use crate::ProjectionKey as K;
    for (rows, record, linked) in [
        (vec![goal_words(1)], K::GoalWords(1), K::Goal(1)),
        (vec![goal(1)], K::Goal(1), K::GoalWords(1)),
        (vec![goal(1), goal_words(1), task(2)], K::Task { goal: 1, task: 2 }, K::TaskWords { goal: 1, task: 2 }),
        (vec![goal(1), goal_words(1), task_words(2)], K::TaskWords { goal: 1, task: 2 }, K::Task { goal: 1, task: 2 }),
    ] {
        let (mut core, limits) = loaded();
        for row in rows {
            assert_eq!(restore(&mut core, &limits, row), Restored::Live);
        }
        assert_eq!(core.start_failure(), None);
        core.start_projections_link();
        assert_eq!(
            core.start_failure(),
            Some(Failure::Link {
                record: Key::Core(CoreKey::Projection(record)),
                linked: Linked::Record(Key::Core(CoreKey::Projection(linked))),
            })
        );
    }
}
