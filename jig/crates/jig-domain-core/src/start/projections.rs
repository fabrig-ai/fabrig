//! Bounded projection assembly and links (jig/docs/design/domain/engine/restart.md).
use crate::{
    Core, CoreKey, CoreRecord, Key, Limits, Projection, ProjectionGoalHot, ProjectionKey, ProjectionMilestone,
    ProjectionRecord, ProjectionTaskHot, Record, Request, Write,
    effects::projections::{header, phase_within},
    people,
    start::{Failure, Linked, LiveSet},
    tasks,
};
use alloc::boxed::Box;
use skein_lib::{List, Map, length::len_u64};

#[derive(Hash, Debug)]
pub(crate) struct Assembly {
    pub(crate) header: Option<Header>,
    pub(crate) goal_words: Option<Box<[u8]>>,
    pub(crate) tasks: Map<u64, ProjectionTaskHot>,
    pub(crate) task_words: Map<u64, Box<[u8]>>,
    pub(crate) milestones: Map<ProjectionKey, ProjectionMilestone>,
}

#[derive(Hash, Debug)]
pub(crate) struct Header {
    goal: ProjectionGoalHot,
    pending: Box<[u16]>,
    changed: bool,
    fed_at: Option<u64>,
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
        ProjectionRecord::GoalHot { goal, pending, .. } => {
            if !fit(core, key, len_u64(pending.len()), len_u64(core.connectors.len())) {
                return None;
            }
            (goal.number, 0, limits.tasks.spec_bytes, Some(&goal.phase))
        }
        ProjectionRecord::TaskHot { goal, task } => (*goal, 0, limits.tasks.spec_bytes, Some(&task.phase)),
        ProjectionRecord::GoalWords { goal, words, .. } | ProjectionRecord::TaskWords { goal, words, .. } => {
            (*goal, words.len(), limits.tasks.spec_bytes, None)
        }
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
                    goal_words: None,
                    tasks: Map::with_capacity(limits.tasks.tree_tasks),
                    task_words: Map::with_capacity(limits.tasks.tree_tasks),
                    milestones: Map::with_capacity(core.routes),
                },
            )
            .expect("projection assembly room");
    }
    let state = core.start.projections.as_mut().expect("assembly table").get_mut(&goal).expect("assembly admitted");
    match row {
        ProjectionRecord::GoalHot { goal, pending, changed, fed_at } => {
            assert!(state.header.is_none(), "duplicate projection {key:?}");
            state.header = Some(Header { goal, pending, changed, fed_at });
        }
        ProjectionRecord::TaskHot { task, .. } => {
            assert!(!state.tasks.contains_key(&task.number), "duplicate projection {key:?}");
            let held = state.tasks.len();
            let limit = state.tasks.capacity();
            if held == limit {
                return fit(core, key, u64::from(held).checked_add(1).expect("one more piece"), u64::from(limit));
            }
            state.tasks.insert(task.number, task).expect("bounded projection tasks");
        }
        ProjectionRecord::GoalWords { words, .. } => {
            assert!(state.goal_words.is_none(), "duplicate projection {key:?}");
            state.goal_words = Some(words);
        }
        ProjectionRecord::TaskWords { task, words, .. } => {
            assert!(!state.task_words.contains_key(&task), "duplicate projection {key:?}");
            let held = state.task_words.len();
            let limit = state.task_words.capacity();
            if held == limit {
                return fit(core, key, u64::from(held).checked_add(1).expect("one more piece"), u64::from(limit));
            }
            state.task_words.insert(task, words).expect("bounded projection words");
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
    pub(crate) fn start_projections_repair(&mut self, out: &mut skein_lib::Queue<Request>) {
        let mut goals = List::with_capacity(self.projections.len());
        for (goal, _) in &self.projections {
            goals.push(*goal).expect("bounded projection repair");
        }
        for goal in &goals {
            let state = self.projections.get(goal).expect("assembled projection");
            let mut plan = List::with_capacity(u32::try_from(state.plan.len()).expect("bounded plan"));
            let mut repaired = false;
            for row in &state.plan {
                let missing = match row.phase {
                    tasks::Phase::Closing(_)
                    | tasks::Phase::Ended(_)
                    | tasks::Phase::Held { was: tasks::Was::Closing(_), .. } => false,
                    tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Held { .. } => {
                        self.tasks.task(row.number).is_none()
                    }
                };
                if !missing {
                    plan.push(row.clone()).expect("assembled plan room");
                    continue;
                }
                repaired = true;
                match self.projections.get(&row.number) {
                    Some(own) => {
                        let mut row = row.clone();
                        row.phase.clone_from(&own.goal.phase);
                        out.push(Request::Write(Write::Save(Record::Core(CoreRecord::Projection(
                            ProjectionRecord::TaskHot { goal: *goal, task: row.hot() },
                        )))));
                        plan.push(row).expect("assembled plan room");
                    }
                    None => {
                        for key in [
                            ProjectionKey::Task { goal: *goal, task: row.number },
                            ProjectionKey::TaskWords { goal: *goal, task: row.number },
                        ] {
                            out.push(Request::Write(Write::Erase(Key::Core(CoreKey::Projection(key)))));
                        }
                    }
                }
            }
            if repaired {
                let state = self.projections.get_mut(goal).expect("assembled projection");
                state.plan = plan.into_boxed();
                state.changed = true;
                out.push(Request::Write(Write::Save(Record::Core(CoreRecord::Projection(header(state))))));
            }
        }
    }

    pub(crate) fn start_projections_link(&mut self) {
        let Some(mut assemblies) = self.start.projections.take() else { return };
        for _ in 0..assemblies.len() {
            let (&goal, _) = assemblies.first().expect("bounded assembly walk");
            let mut state = assemblies.remove(&goal).expect("assembled projection");
            if state.header.is_none() {
                let key = match state.tasks.first() {
                    Some((&task, _)) => ProjectionKey::Task { goal, task },
                    None => match state.task_words.first() {
                        Some((&task, _)) => ProjectionKey::TaskWords { goal, task },
                        None => match state.goal_words {
                            Some(_) => ProjectionKey::GoalWords(goal),
                            None => *state.milestones.first().expect("assembly has a piece").0,
                        },
                    },
                };
                let _refused = self.start_refuse(Failure::Link {
                    record: Key::Core(CoreKey::Projection(key)),
                    linked: Linked::Record(Key::Core(CoreKey::Projection(ProjectionKey::Goal(goal)))),
                });
                return;
            }
            let source = Key::Core(CoreKey::Projection(ProjectionKey::Goal(goal)));
            let Some(words) = state.goal_words else {
                let _refused = self.start_refuse(Failure::Link {
                    record: source,
                    linked: Linked::Record(Key::Core(CoreKey::Projection(ProjectionKey::GoalWords(goal)))),
                });
                return;
            };
            let Header { goal: hot, pending, changed, fed_at } = state.header.expect("assembled header linked");
            let mut projection = Projection {
                goal: hot.join(words),
                pending,
                changed,
                fed_at,
                plan: Box::new([]),
                milestones: Box::new([]),
            };
            if projection.goal.project != people::DEPLOYMENT && self.authority.policy(projection.goal.project).is_none()
            {
                let _refused = self
                    .start_refuse(Failure::Link { record: source, linked: Linked::Project(projection.goal.project) });
                return;
            }
            for connector in &projection.pending {
                if !self.connectors.contains(connector) {
                    let _refused =
                        self.start_refuse(Failure::Link { record: source, linked: Linked::Connector(*connector) });
                    return;
                }
            }
            match &projection.goal.phase {
                tasks::Phase::Ended(_)
                | tasks::Phase::Closing(_)
                | tasks::Phase::Held { was: tasks::Was::Closing(_), .. } => {}
                tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Held { .. } => {
                    if self.tasks.task(goal).is_none() {
                        let _refused = self.start_refuse(Failure::Link {
                            record: source,
                            linked: Linked::Record(Key::Tasks(tasks::Key::Live(goal))),
                        });
                        return;
                    }
                }
            }

            let mut plan = List::with_capacity(state.tasks.len());
            for _ in 0..state.tasks.len() {
                let (&number, _) = state.tasks.first().expect("bounded assembled plan");
                let task = state.tasks.remove(&number).expect("assembled task");
                let Some(words) = state.task_words.remove(&number) else {
                    let _refused = self.start_refuse(Failure::Link {
                        record: Key::Core(CoreKey::Projection(ProjectionKey::Task { goal, task: number })),
                        linked: Linked::Record(Key::Core(CoreKey::Projection(ProjectionKey::TaskWords {
                            goal,
                            task: number,
                        }))),
                    });
                    return;
                };
                plan.push(task.join(words)).expect("bounded assembled plan");
            }
            if let Some((&task, _)) = state.task_words.first() {
                let _refused = self.start_refuse(Failure::Link {
                    record: Key::Core(CoreKey::Projection(ProjectionKey::TaskWords { goal, task })),
                    linked: Linked::Record(Key::Core(CoreKey::Projection(ProjectionKey::Task { goal, task }))),
                });
                return;
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
