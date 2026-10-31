//! Bounded projection assembly and links (jig/docs/design/domain/engine.md, 6).
use super::{Failure, LiveSet};
use crate::projections::phase_within;
use crate::{
    Core, CoreKey, Key, Limits, Projection, ProjectionKey, ProjectionMilestone, ProjectionRecord, ProjectionTask,
    people, tasks,
};
use alloc::boxed::Box;
use skein_lib::{List, Map};

#[derive(Debug)]
pub(crate) struct Assembly {
    pub(crate) header: Option<Projection>,
    pub(crate) tasks: Map<u64, ProjectionTask>,
    pub(crate) milestones: Map<ProjectionKey, ProjectionMilestone>,
}

fn fit(core: &mut Core, key: ProjectionKey, held: u64, limit: u64) -> bool {
    if held <= limit {
        return true;
    }
    let _refused = core.start_refuse(Failure::LiveSet {
        set: LiveSet::Projections,
        key: Key::Core(CoreKey::Projection(key)),
        held,
        limit,
    });
    false
}

fn fit_row(core: &mut Core, limits: &Limits, row: &ProjectionRecord) -> Option<u64> {
    let key = row.key();
    let (goal, words, word_limit, phase) = match row {
        ProjectionRecord::Goal { goal, pending, .. } => {
            if !fit(
                core,
                key,
                u64::try_from(pending.len()).expect("connector count"),
                u64::try_from(core.connectors.len()).expect("connector count"),
            ) {
                return None;
            }
            (goal.number, goal.words.len(), limits.tasks.spec_bytes, Some(&goal.phase))
        }
        ProjectionRecord::Task { goal, task } => (*goal, task.words.len(), limits.tasks.spec_bytes, Some(&task.phase)),
        ProjectionRecord::Milestone { goal, milestone } => (
            *goal,
            milestone.words.len(),
            limits.tasks.message_bytes.max(limits.tasks.result_bytes),
            milestone.phase.as_ref(),
        ),
    };
    if !fit(core, key, u64::try_from(words).expect("owned words fit u64"), u64::from(word_limit)) {
        return None;
    }
    if let Some(phase) = phase
        && !phase_within(phase, limits)
    {
        let _fits = fit(
            core,
            key,
            tasks::phase_bytes(phase).unwrap_or(u64::MAX),
            u64::from(limits.tasks.result_bytes).checked_mul(2).expect("phase byte bound"),
        );
        return None;
    }
    Some(goal)
}

pub(crate) fn start_restore(core: &mut Core, limits: &Limits, row: ProjectionRecord) -> bool {
    let key = row.key();
    let Some(goal) = fit_row(core, limits, &row) else { return false };
    let assemblies = core.start.projections.get_or_insert(Map::with_capacity(core.projections.capacity()));
    if !assemblies.contains_key(&goal) {
        let held = assemblies.len();
        if !core.start_room(
            LiveSet::Projections,
            Key::Core(CoreKey::Projection(key)),
            held,
            core.projections.capacity(),
        ) {
            return false;
        }
        core.start
            .projections
            .as_mut()
            .expect("assembly table")
            .insert(
                goal,
                Assembly {
                    header: None,
                    tasks: Map::with_capacity(limits.tasks.tree_tasks),
                    milestones: Map::with_capacity(core.routes),
                },
            )
            .expect("projection assembly room");
    }
    let state = core.start.projections.as_mut().expect("assembly table").get_mut(&goal).expect("assembly admitted");
    match row {
        ProjectionRecord::Goal { goal, pending, changed, fed_at } => {
            assert!(state.header.is_none(), "duplicate projection {key:?}");
            state.header =
                Some(Projection { goal, pending, changed, fed_at, plan: Box::new([]), milestones: Box::new([]) });
        }
        ProjectionRecord::Task { task, .. } => {
            assert!(!state.tasks.contains_key(&task.number), "duplicate projection {key:?}");
            let held = state.tasks.len();
            let limit = state.tasks.capacity();
            if held == limit {
                return fit(core, key, u64::from(held).checked_add(1).expect("one more piece"), u64::from(limit));
            }
            state.tasks.insert(task.number, task).expect("bounded projection tasks");
        }
        ProjectionRecord::Milestone { milestone, .. } => {
            assert!(!state.milestones.contains_key(&key), "duplicate projection {key:?}");
            let held = state.milestones.len();
            let limit = state.milestones.capacity();
            if held == limit {
                return fit(core, key, u64::from(held).checked_add(1).expect("one more piece"), u64::from(limit));
            }
            state.milestones.insert(key, milestone).expect("bounded projection milestones");
        }
    }
    true
}

impl Core {
    pub(crate) fn start_projections_link(&mut self) {
        let Some(mut assemblies) = self.start.projections.take() else { return };
        for _ in 0..assemblies.len() {
            let (&goal, _) = assemblies.first().expect("bounded assembly walk");
            let mut state = assemblies.remove(&goal).expect("assembled projection");
            if state.header.is_none() {
                let key = match state.tasks.first() {
                    Some((&task, _)) => ProjectionKey::Task { goal, task },
                    None => *state.milestones.first().expect("assembly has a piece").0,
                };
                assert!(
                    state.header.is_some(),
                    "{:?} links missing {:?}",
                    Key::Core(CoreKey::Projection(key)),
                    Key::Core(CoreKey::Projection(ProjectionKey::Goal(goal)))
                );
            }
            let mut projection = state.header.expect("assembled header linked");
            let source = Key::Core(CoreKey::Projection(ProjectionKey::Goal(goal)));
            assert!(
                projection.goal.project == people::DEPLOYMENT
                    || self.authority.policy(projection.goal.project).is_some(),
                "{source:?} links missing project {}",
                projection.goal.project
            );
            for connector in &projection.pending {
                assert!(self.connectors.contains(connector), "{source:?} links missing connector {connector}");
            }
            match &projection.goal.phase {
                tasks::Phase::Ended(_)
                | tasks::Phase::Closing(_)
                | tasks::Phase::Held { was: tasks::Was::Closing(_), .. } => {}
                tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Held { .. } => {
                    assert!(
                        self.tasks.task(goal).is_some(),
                        "{source:?} links missing {:?}",
                        Key::Tasks(tasks::Key::Live(goal))
                    );
                }
            }
            let mut plan = List::with_capacity(state.tasks.len());
            for _ in 0..state.tasks.len() {
                let (&number, _) = state.tasks.first().expect("bounded assembled plan");
                let task = state.tasks.remove(&number).expect("assembled task");
                plan.push(task).expect("bounded assembled plan");
            }
            let mut milestones = List::with_capacity(state.milestones.len());
            for _ in 0..state.milestones.len() {
                let (&key, _) = state.milestones.first().expect("bounded assembled history");
                let milestone = state.milestones.remove(&key).expect("assembled milestone");
                milestones.push(milestone).expect("bounded assembled history");
            }
            projection.plan = plan.into_boxed();
            projection.milestones = milestones.into_boxed();
            self.projections.insert(goal, Box::new(projection)).expect("bounded assembled projections");
        }
    }
}
