//! Effects tests.
pub(in crate::tests) mod deployment_connectors;
pub(in crate::tests) mod hold_origin;

#[test]
fn saved_lifecycle_projections_take_holds_and_ended_positions_but_not_attempts() {
    use crate::{CoreRecord, MilestoneId, ProjectionRecord, Record, Request, Write, tasks};
    use skein_lib::{Env, List, Queue, Time, Wall};
    let (mut core, limits) = super::make_tree();
    let mut ended = core.tasks.task(2).unwrap().clone();
    ended.phase = tasks::Phase::Ended(tasks::Ending::Failed { reason: Box::new([]) });
    ended.milestone = 29;
    ended.ended_at = Some(Wall::from_nanos(17));
    let held = tasks::Held {
        was: tasks::Was::Active(tasks::Active::Running { attempt: 3 }),
        why: tasks::Hold::Stopped,
        source: None,
    };
    let phase = tasks::Phase::Held { was: held.was.clone(), why: held.why, source: None };
    let rows = [
        tasks::Stored::Attempt(tasks::AttemptRow {
            task: 2,
            attempt: 3,
            claimed: None,
            started: None,
            ended: None,
            end: None,
            phases: None,
        }),
        tasks::Stored::Hold(tasks::HoldRow { task: 2, position: 23, at: Wall::from_nanos(11), held }),
        tasks::Stored::Ended(Box::new(ended.clone())),
    ];
    let before = core.projections.get(&1).unwrap().milestones.len();
    let mut feeds = List::with_capacity(2);
    for (index, row) in rows.into_iter().enumerate() {
        let mut saved = Queue::uncapped();
        saved.push(tasks::Request::Save { record: row });
        let out = crate::routing::task::tag_task_outputs(
            &mut core,
            &Env { now: Time::ZERO, wall: Wall::from_nanos(19), limits },
            saved,
        );
        for request in &out {
            if let Request::Write(Write::Save(Record::Core(CoreRecord::Projection(ProjectionRecord::Milestone {
                milestone,
                ..
            })))) = request
            {
                feeds.push(milestone.clone()).unwrap();
            }
        }
        if index == 0 {
            assert!(feeds.is_empty());
            assert_eq!(core.projections.get(&1).unwrap().milestones.len(), before);
        }
    }
    assert_eq!(feeds.len(), 2);
    assert_eq!(feeds.get(0).unwrap().identity, MilestoneId::Lifecycle { task: 2, position: 23 });
    assert_eq!(feeds.get(0).unwrap().phase, Some(phase));
    assert_eq!(feeds.get(1).unwrap().identity, MilestoneId::Lifecycle { task: 2, position: ended.milestone });
    assert_eq!(feeds.get(1).unwrap().phase, Some(ended.phase));
}

#[test]
fn saving_a_projection_preserves_neighbours_when_updating_or_appending() {
    use crate::{CoreRecord, ProjectionRecord, ProjectionTask, Record, Request, Write, effects::projections, tasks};
    use skein_lib::{List, Queue};

    let (mut core, limits) = super::make_tree();
    let mut row = core.tasks.task(2).unwrap().clone();
    row.spec.words = b"Changed task".as_slice().into();
    row.phase = tasks::Phase::Waiting;
    for number in [2, 5] {
        row.number = number;
        let next = ProjectionTask {
            number,
            requester: row.requester,
            words: row.spec.words.clone(),
            phase: row.phase.clone(),
        };
        let state = core.projections.get_mut(&1).unwrap();
        state.changed = false;
        let mut expected = List::with_capacity(limits.tasks.tree_tasks);
        let mut retained = List::with_capacity(limits.tasks.tree_tasks);
        let mut found = false;
        for (index, old) in state.plan.iter().enumerate() {
            if old.number == number {
                expected.push(next.clone()).unwrap();
                found = true;
            } else {
                expected.push(old.clone()).unwrap();
                retained.push((index, old.words.as_ptr())).unwrap();
            }
        }
        if !found {
            expected.push(next.clone()).unwrap();
        }
        let mut out = Queue::uncapped();
        projections::save(&mut core, &limits, &row, &mut out);
        let state = core.projections.get(&1).unwrap();
        assert_eq!(state.plan.as_ref(), expected.as_slice(), "task {number}");
        assert!(state.changed, "task {number} changed its projection");
        for (index, words) in &retained {
            let neighbour = state.plan.get(*index).unwrap();
            assert_eq!(neighbour.words.as_ptr(), *words, "task {number} copied neighbour {}", neighbour.number);
        }
        let Some(Request::Write(Write::Save(Record::Core(CoreRecord::Projection(ProjectionRecord::Task {
            goal,
            task,
        }))))) = out.pop()
        else {
            panic!("saved projection task")
        };
        assert_eq!(goal, 1);
        assert_eq!(task, next);
        let Some(Request::Write(Write::Save(Record::Core(CoreRecord::Projection(ProjectionRecord::Goal {
            goal,
            changed,
            ..
        }))))) = out.pop()
        else {
            panic!("saved projection header")
        };
        assert_eq!(goal, state.goal);
        assert!(changed);
        assert!(out.is_empty());
        core.projections.get_mut(&1).unwrap().changed = false;
        projections::save(&mut core, &limits, &row, &mut out);
        assert!(!core.projections.get(&1).unwrap().changed, "identical save leaves task {number} unchanged");
    }
}
