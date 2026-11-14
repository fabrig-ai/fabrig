//! Effects tests.
pub(in crate::tests) mod deployment_connectors;
pub(in crate::tests) mod hold_origin;

#[test]
fn saving_a_projection_preserves_neighbours_when_updating_or_appending() {
    use crate::{ProjectionTask, effects::projections, tasks};
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
        while out.pop().is_some() {}
        core.projections.get_mut(&1).unwrap().changed = false;
        projections::save(&mut core, &limits, &row, &mut out);
        assert!(!core.projections.get(&1).unwrap().changed, "identical save leaves task {number} unchanged");
        assert!(out.is_empty());
    }
}

fn projection_puts(out: &mut skein_lib::Queue<crate::Request>) -> Box<[crate::ProjectionRecord]> {
    let mut rows = skein_lib::List::uncapped();
    while let Some(request) = out.pop() {
        let crate::Request::Write(crate::Write::Save(crate::Record::Core(crate::CoreRecord::Projection(row)))) =
            request
        else {
            panic!("only projection puts: {request:?}")
        };
        rows.push(row).expect("uncapped test rows");
    }
    rows.into_boxed()
}

fn projection_keys(rows: &[crate::ProjectionRecord]) -> Box<[crate::ProjectionKey]> {
    let mut keys = skein_lib::List::uncapped();
    for row in rows {
        keys.push(row.key()).expect("uncapped test keys");
    }
    keys.into_boxed()
}

#[test]
fn projection_saves_put_only_new_or_changed_parts() {
    use crate::{ProjectionKey as K, ProjectionRecord as R, effects::projections, tasks};
    use skein_lib::Queue;
    let (mut core, limits) = super::make_tree();
    let goal = core.tasks.task(1).unwrap().clone();
    core.projections.remove(&1).unwrap();
    let mut out = Queue::uncapped();
    projections::save(&mut core, &limits, &goal, &mut out);
    let rows = projection_puts(&mut out);
    assert_eq!(
        projection_keys(&rows).as_ref(),
        [K::GoalWords(1), K::Task { goal: 1, task: 1 }, K::TaskWords { goal: 1, task: 1 }, K::Goal(1)]
    );
    let R::GoalWords { words, .. } = &rows[0] else { panic!("goal words") };
    assert_eq!(words, &goal.spec.words);
    let R::GoalHot { goal: hot, changed, .. } = &rows[3] else { panic!("hot header") };
    assert!(*changed);
    assert_eq!(hot, &core.projections.get(&1).unwrap().goal.hot());

    let mut task = core.tasks.task(2).unwrap().clone();
    task.number = 9;
    core.projections.get_mut(&1).unwrap().changed = false;
    projections::save(&mut core, &limits, &task, &mut out);
    let rows = projection_puts(&mut out);
    assert_eq!(
        projection_keys(&rows).as_ref(),
        [K::Task { goal: 1, task: 9 }, K::TaskWords { goal: 1, task: 9 }, K::Goal(1)]
    );
    let R::TaskWords { words, .. } = &rows[1] else { panic!("task words") };
    assert_eq!(words, &task.spec.words);
    projections::save(&mut core, &limits, &task, &mut out);
    assert!(out.is_empty(), "a save with no projected change puts nothing");

    task.phase = tasks::Phase::Active(tasks::Active::Claimed { attempt: 7 });
    projections::save(&mut core, &limits, &task, &mut out);
    let rows = projection_puts(&mut out);
    assert_eq!(rows.len(), 1);
    let R::TaskHot { goal: 1, task: hot } = &rows[0] else { panic!("hot task") };
    assert_eq!(hot.number, 9);
    assert_eq!(hot.phase, task.phase);
    core.projections.get_mut(&1).unwrap().changed = false;
    task.phase = tasks::Phase::Waiting;
    projections::save(&mut core, &limits, &task, &mut out);
    assert_eq!(projection_keys(&projection_puts(&mut out)).as_ref(), [K::Task { goal: 1, task: 9 }, K::Goal(1)]);

    task.spec.words = b"amended words".as_slice().into();
    projections::save(&mut core, &limits, &task, &mut out);
    assert_eq!(
        projection_puts(&mut out).as_ref(),
        &[R::TaskWords { goal: 1, task: 9, words: task.spec.words.clone() }]
    );
    projections::save(&mut core, &limits, &task, &mut out);
    assert!(out.is_empty());
}

#[test]
fn projection_feed_writes_hot_header_and_last_settlement_erases_every_part() {
    use crate::{
        CoreKey, CoreRecord, Key, MilestoneId, ProjectionKey as K, ProjectionMilestone, ProjectionRecord as R, Record,
        Request, Write, effects::projections, tasks,
    };
    use skein_lib::{Env, Queue, Time, Wall};
    let (mut core, limits) = super::make_tree();
    let mut out = Queue::uncapped();
    let milestone = ProjectionMilestone {
        identity: MilestoneId::Lifecycle { task: 2, position: 900 },
        phase: None,
        change: None,
        words: b"history".as_slice().into(),
    };
    projections::history(&mut core, &limits, 2, milestone.clone(), &mut out);
    while out.pop().is_some() {}
    let state = core.projections.get_mut(&1).unwrap();
    state.goal.phase = tasks::Phase::Ended(tasks::Ending::Failed { reason: Box::new([]) });
    state.fed_at = None;
    let plan = state.plan.clone();
    let history_key = projections::milestone_key(1, milestone.identity);
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let mut out = projections::finish_decision(&mut core, &env);
    let mut keys = skein_lib::List::uncapped();
    let mut feeds = 0;
    while let Some(request) = out.pop() {
        match request {
            Request::Write(Write::Erase(Key::Core(CoreKey::Projection(key)))) => assert_eq!(key, history_key),
            Request::Write(Write::Save(Record::Core(CoreRecord::Projection(row)))) => {
                let R::GoalHot { changed: false, fed_at: Some(_), .. } = &row else { panic!("fed hot header") };
                keys.push(row.key()).expect("uncapped test keys");
            }
            Request::Ask { ask: crate::Ask::ProjectGoal { feed }, .. } => {
                assert_eq!(feed.plan, plan);
                assert_eq!(feed.milestones.as_ref(), core::slice::from_ref(&milestone));
                assert!(feed.closing);
                feeds += 1;
            }
            other @ (Request::Write(_)
            | Request::Ask { .. }
            | Request::ProjectMade { .. }
            | Request::Held(_)
            | Request::Now(_)) => panic!("feed decision: {other:?}"),
        }
    }
    assert_eq!(keys.as_slice(), &[K::Goal(1)]);
    assert_eq!(feeds, core.connectors.len());
    let connectors = core.connectors.clone();
    for connector in &connectors {
        projections::settled(&mut core, 1, *connector, &mut out);
        if core.projections.contains_key(&1) {
            assert_eq!(projection_keys(&projection_puts(&mut out)).as_ref(), [K::Goal(1)]);
        }
    }
    let mut erased = skein_lib::List::uncapped();
    while let Some(request) = out.pop() {
        match request {
            Request::Write(Write::Erase(Key::Core(CoreKey::Projection(key)))) => {
                erased.push(key).expect("uncapped test erasures");
            }
            Request::Ask { ask: crate::Ask::ForgetProjection { goal: 1 }, .. } => {}
            other @ (Request::Write(_)
            | Request::Ask { .. }
            | Request::ProjectMade { .. }
            | Request::Held(_)
            | Request::Now(_)) => panic!("last settlement: {other:?}"),
        }
    }
    let mut expected = skein_lib::List::uncapped();
    expected.push(K::Goal(1)).unwrap();
    expected.push(K::GoalWords(1)).unwrap();
    for task in &plan {
        expected.push(K::Task { goal: 1, task: task.number }).unwrap();
        expected.push(K::TaskWords { goal: 1, task: task.number }).unwrap();
    }
    let mut expected = expected.into_boxed();
    let mut erased = erased.into_boxed();
    expected.sort();
    erased.sort();
    assert_eq!(erased, expected);
    assert!(!core.projections.contains_key(&1));
}
