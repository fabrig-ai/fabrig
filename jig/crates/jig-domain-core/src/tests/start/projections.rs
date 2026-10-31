//! Stored projection fits and links (jig/docs/design/domain/engine.md, 6).
#![expect(clippy::disallowed_macros, reason = "dynamic stored-row fixtures exercise byte bounds")]
use super::{Core, Failure, Key, Wall, config, limits, tasks};
use crate::start::LiveSet;
use crate::{
    CoreKey, CoreRecord, Limits, MilestoneId, ProjectionGoal, ProjectionMilestone, ProjectionRecord, ProjectionTask,
    Restored,
};

fn loaded(selected: bool) -> (Core, Limits) {
    loaded_policy(selected, true)
}

fn loaded_policy(selected: bool, policy: bool) -> (Core, Limits) {
    let limits = limits();
    let configured = config(401);
    let project = &config(401).seed_projects[0];
    let mut core = Core::new(configured, &limits).expect("configured core");
    if selected {
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
            crate::policy::snapshot(&project.policy.policy, &[], &[]).expect("bounded policy")
        ));
    }
    (core, limits)
}

fn ending() -> tasks::Phase {
    tasks::Phase::Ended(tasks::Ending::Failed { reason: Box::new([]) })
}
fn goal(number: u64) -> ProjectionRecord {
    ProjectionRecord::Goal {
        goal: ProjectionGoal { number, project: 1, priority: 1, words: b"goal".as_slice().into(), phase: ending() },
        pending: Box::new([]),
        changed: false,
        fed_at: Some(1),
    }
}
fn task(number: u64) -> ProjectionRecord {
    ProjectionRecord::Task {
        goal: 1,
        task: ProjectionTask {
            number,
            requester: tasks::Party::Task(1),
            words: b"task".as_slice().into(),
            phase: ending(),
        },
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
fn retained(row: ProjectionRecord, header: bool) {
    let (mut default, limits) = loaded(false);
    let (mut selected, _) = loaded(true);
    if header {
        assert_eq!(restore(&mut default, &limits, goal(1)), Restored::Live);
        assert_eq!(restore(&mut selected, &limits, goal(1)), Restored::Live);
    }
    assert_eq!(restore(&mut default, &limits, row.clone()), Restored::Rejected);
    assert_eq!(restore(&mut selected, &limits, row), Restored::Live);
    assert_eq!(selected.start_failure(), None);
    selected.start_projections_link();
    assert_eq!(selected.projections.len(), 1);
}

#[test]
fn historical_numbers_and_feed_positions_are_not_admission_rules() {
    for number in [0, 101] {
        retained(goal(number), false);
        retained(task(number), true);
        retained(milestone(number), true);
    }
    let mut row = goal(1);
    let ProjectionRecord::Goal { fed_at, .. } = &mut row else { unreachable!() };
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
    let (mut default, limits) = loaded_policy(false, false);
    assert_eq!(restore(&mut default, &limits, goal(1)), Restored::Rejected);
    let (mut core, _) = loaded_policy(true, false);
    let project = &config(401).seed_projects[0];
    assert_eq!(restore(&mut core, &limits, goal(1)), Restored::Live);
    assert!(core.restore_policy(
        &limits,
        1,
        project.name.clone(),
        crate::policy::snapshot(&project.policy.policy, &[], &[]).expect("bounded policy")
    ));
    core.start_projections_link();
    assert_eq!(core.projections.get(&1).expect("linked goal").goal.project, 1);
    let mut row = goal(2);
    let ProjectionRecord::Goal { goal, .. } = &mut row else { unreachable!() };
    goal.project = crate::people::DEPLOYMENT;
    retained(row, false);
}

#[test]
fn pending_closing_projection_loads_without_its_live_task() {
    let mut row = goal(1);
    let ProjectionRecord::Goal { goal, pending, .. } = &mut row else { unreachable!() };
    goal.phase = tasks::Phase::Closing(tasks::Closing {
        stage: tasks::Stage::Effects,
        ending: tasks::Ending::Failed { reason: Box::new([]) },
    });
    *pending = Box::new([config(401).connectors[0]]);
    retained(row, false);
}

#[test]
fn pending_on_a_live_tracked_goal_is_retained() {
    let (mut source, limits) = super::super::core();
    super::super::make_task(&mut source, &limits, 1);
    let mut live = source.tasks.task(1).expect("live fixture").clone();
    live.tracked = Some(1);
    let mut row = goal(1);
    let ProjectionRecord::Goal { goal, pending, .. } = &mut row else { unreachable!() };
    goal.phase = live.phase.clone();
    *pending = Box::new([config(401).connectors[0]]);
    for selected in [false, true] {
        let (mut core, limits) = loaded(selected);
        let record = tasks::Stored::Live(Box::new(live.clone()));
        let mut out = super::Queue::uncapped();
        tasks::step(
            &mut core.tasks,
            &super::Env { now: super::Time::ZERO, wall: Wall::EPOCH, limits: limits.tasks },
            tasks::Event::Restore { record },
            &mut out,
        );
        assert_eq!(
            restore(&mut core, &limits, row.clone()),
            if selected { Restored::Live } else { Restored::Rejected }
        );
        if selected {
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
    let rows = [goal(1), task(1), task(2), milestone(1), milestone(2)];
    let (mut default, limits) = loaded(false);
    for row in &rows {
        assert_eq!(restore(&mut default, &limits, row.clone()), Restored::Live);
    }
    for order in [[1, 4, 2, 3, 0], [4, 0, 3, 2, 1], [0, 1, 2, 3, 4]] {
        let (mut selected, _) = loaded(true);
        for index in order {
            assert_eq!(restore(&mut selected, &limits, rows[index].clone()), Restored::Live);
        }
        let assembly = selected.start.projections.as_ref().expect("assembly").get(&1).expect("goal pieces");
        assert_eq!(
            assembly.tasks.get(&2),
            match &rows[2] {
                ProjectionRecord::Task { task, .. } => Some(task),
                ProjectionRecord::Goal { .. } | ProjectionRecord::Milestone { .. } => unreachable!(),
            }
        );
        assert_eq!(assembly.milestones.len(), 2);
        selected.start_projections_link();
        assert_eq!(selected.projections.get(&1), default.projections.get(&1));
        assert!(selected.start.projections.is_none());
    }
}

#[test]
#[should_panic(expected = "Core(Projection(Task { goal: 1, task: 2 })) links missing Core(Projection(Goal(1)))")]
fn orphan_task_panics_at_link_with_its_key() {
    let (mut core, limits) = loaded(true);
    assert_eq!(restore(&mut core, &limits, task(2)), Restored::Live);
    let assembly = core.start.projections.as_ref().expect("assembly").get(&1).expect("orphan");
    assert_eq!(assembly.header, None);
    assert_eq!(
        assembly.tasks.get(&2),
        match task(2) {
            ProjectionRecord::Task { ref task, .. } => Some(task),
            ProjectionRecord::Goal { .. } | ProjectionRecord::Milestone { .. } => unreachable!(),
        }
    );
    core.start_projections_link();
}

#[test]
#[should_panic(expected = "Projection(Lifecycle { goal: 1, task: 2, position: 1 })")]
fn orphan_milestone_panics_at_link_with_its_key() {
    let (mut core, limits) = loaded(true);
    assert_eq!(restore(&mut core, &limits, milestone(2)), Restored::Live);
    core.start_projections_link();
}

fn refused(core: &mut Core, limits: &Limits, row: ProjectionRecord, held: u64, limit: u64) {
    let key = row.key();
    assert_eq!(restore(core, limits, row), Restored::Rejected);
    assert_eq!(
        core.start_failure(),
        Some(Failure::LiveSet { set: LiveSet::Projections, key: Key::Core(CoreKey::Projection(key)), held, limit })
    );
}

#[test]
fn table_capacity_counts_headerless_goals() {
    for headers in [false, true] {
        let (mut core, limits) = loaded(true);
        for number in 1..=core.projections.capacity() {
            let row = if headers {
                goal(u64::from(number))
            } else {
                ProjectionRecord::Task {
                    goal: u64::from(number),
                    task: ProjectionTask {
                        number: 1,
                        requester: tasks::Party::Task(1),
                        words: Box::new([]),
                        phase: ending(),
                    },
                }
            };
            assert_eq!(restore(&mut core, &limits, row), Restored::Live);
        }
        let limit = u64::from(core.projections.capacity());
        refused(&mut core, &limits, goal(limit + 1), limit + 1, limit);
    }
}

#[test]
fn words_phase_and_pending_fits_keep_the_row_key() {
    let limits = limits();
    for row in [goal(1), task(2), milestone(2)] {
        let (mut core, _) = loaded(true);
        let mut row = row;
        let (words, bound) = match &mut row {
            ProjectionRecord::Goal { goal, .. } => (&mut goal.words, limits.tasks.spec_bytes),
            ProjectionRecord::Task { task, .. } => (&mut task.words, limits.tasks.spec_bytes),
            ProjectionRecord::Milestone { milestone, .. } => {
                (&mut milestone.words, limits.tasks.message_bytes.max(limits.tasks.result_bytes))
            }
        };
        *words = vec![b'x'; usize::try_from(bound + 1).expect("fixture bytes")].into_boxed_slice();
        refused(&mut core, &limits, row, u64::from(bound) + 1, u64::from(bound));
    }
    let (mut core, _) = loaded(true);
    let mut row = goal(1);
    let ProjectionRecord::Goal { pending, .. } = &mut row else { unreachable!() };
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
        let (mut core, _) = loaded(true);
        let phase = match &mut row {
            ProjectionRecord::Goal { goal, .. } => &mut goal.phase,
            ProjectionRecord::Task { task, .. } => &mut task.phase,
            ProjectionRecord::Milestone { milestone, .. } => milestone.phase.as_mut().expect("fixture phase"),
        };
        *phase = tasks::Phase::Ended(tasks::Ending::Failed {
            reason: vec![b'x'; usize::try_from(limits.tasks.result_bytes * 2 + 1).expect("fixture bytes")]
                .into_boxed_slice(),
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
            let (mut core, limits) = loaded(true);
            core.routes = 2;
            if headers {
                assert_eq!(restore(&mut core, &limits, goal(1)), Restored::Live);
            }
            let bound = if milestones { core.routes } else { limits.tasks.tree_tasks };
            for number in 1..=bound {
                assert_eq!(
                    restore(
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
fn duplicate_rows_assert_without_replacing_the_assembled_value() {
    for row in [goal(1), task(2), milestone(2)] {
        let (mut core, limits) = loaded(true);
        assert_eq!(restore(&mut core, &limits, row.clone()), Restored::Live);
        let duplicate =
            std::panic::catch_unwind(core::panic::AssertUnwindSafe(|| restore(&mut core, &limits, row.clone())));
        let _panic = duplicate.expect_err("duplicate row asserts");
        let assembly = core.start.projections.as_ref().expect("assembly").get(&1).expect("assembled goal");
        match &row {
            ProjectionRecord::Goal { goal, .. } => assert_eq!(&assembly.header.as_ref().expect("header").goal, goal),
            ProjectionRecord::Task { task, .. } => assert_eq!(assembly.tasks.get(&task.number), Some(task)),
            ProjectionRecord::Milestone { milestone, .. } => {
                assert_eq!(assembly.milestones.get(&row.key()), Some(milestone));
            }
        }
        assert_eq!(core.start_failure(), None);
    }
}

#[test]
#[should_panic(expected = "Core(Projection(Goal(1))) links missing project 2")]
fn a_missing_project_panics_at_link() {
    let (mut core, limits) = loaded(true);
    let mut row = goal(1);
    let ProjectionRecord::Goal { goal, .. } = &mut row else { unreachable!() };
    goal.project = 2;
    assert_eq!(restore(&mut core, &limits, row), Restored::Live);
    core.start_projections_link();
}

#[test]
#[should_panic(expected = "Core(Projection(Goal(1))) links missing connector 99")]
fn a_missing_pending_connector_panics_at_link() {
    let (mut core, limits) = loaded(true);
    let mut row = goal(1);
    let ProjectionRecord::Goal { pending, .. } = &mut row else { unreachable!() };
    *pending = Box::new([99]);
    assert_eq!(restore(&mut core, &limits, row), Restored::Live);
    core.start_projections_link();
}

#[test]
#[should_panic(expected = "Core(Projection(Goal(1))) links missing Tasks(Live(1))")]
fn an_open_goal_requires_its_live_task_at_link() {
    let (mut core, limits) = loaded(true);
    let mut row = goal(1);
    let ProjectionRecord::Goal { goal, .. } = &mut row else { unreachable!() };
    goal.phase = tasks::Phase::Waiting;
    assert_eq!(restore(&mut core, &limits, row), Restored::Live);
    core.start_projections_link();
}

#[test]
fn duplicate_pending_numbers_are_stored_values_not_admission_rules() {
    let mut row = goal(1);
    let ProjectionRecord::Goal { pending, .. } = &mut row else { unreachable!() };
    *pending = Box::new([config(401).connectors[0]; 2]);
    retained(row, false);
}

#[test]
fn exact_word_and_phase_bounds_remain_loadable() {
    let (mut core, limits) = loaded(true);
    for mut row in [goal(1), task(2), milestone(2)] {
        let (words, bound, phase) = match &mut row {
            ProjectionRecord::Goal { goal, .. } => (&mut goal.words, limits.tasks.spec_bytes, &mut goal.phase),
            ProjectionRecord::Task { task, .. } => (&mut task.words, limits.tasks.spec_bytes, &mut task.phase),
            ProjectionRecord::Milestone { milestone, .. } => (
                &mut milestone.words,
                limits.tasks.message_bytes.max(limits.tasks.result_bytes),
                milestone.phase.as_mut().expect("fixture phase"),
            ),
        };
        *words = vec![b'x'; usize::try_from(bound).expect("fixture bytes")].into_boxed_slice();
        *phase = tasks::Phase::Ended(tasks::Ending::Failed {
            reason: vec![b'x'; usize::try_from(limits.tasks.result_bytes * 2).expect("fixture phase bytes")]
                .into_boxed_slice(),
        });
        assert_eq!(restore(&mut core, &limits, row), Restored::Live);
    }
    core.start_projections_link();
    let projection = core.projections.get(&1).expect("projection at byte bounds");
    assert_eq!(projection.goal.words.len(), usize::try_from(limits.tasks.spec_bytes).expect("fixture bytes"));
    assert_eq!(projection.plan.len(), 1);
    assert_eq!(projection.milestones.len(), 1);
}
