//! The public index (jig/docs/design/domain/people.md, 13.7 and 13.9).
//!
//! Publication and start build from live tasks and begin with no ended goals.
//! Decisions retain bounded public lines as delegates end, then freeze the goal's
//! tree when it ends. A goal folded out of the live list enters the ended list
//! with its line alone: none of its descendants was held public. No operation
//! reads the store. Person adoption is followed;
//! Move under another task (people.md, 5.1) is not built.
//!
//! Structural changes rebuild live membership in the decision, never in a read. This
//! also reselects the priority prefix when a goal is reprioritized. Temporary
//! walks are bounded by the tasks' limits; retained data by the public limits.

mod build;
mod hooks;
mod relations;

use super::{PublicCounts, PublicLine};
use crate::PublicIndexLimits;
use alloc::boxed::Box;
pub(crate) use hooks::{follow, publish, restored};
use jig_domain_tasks as tasks;
use skein_lib::{List, Map};

/// The public index (jig/docs/design/domain/people.md, 13.9).
#[derive(Debug)]
pub(crate) struct Index {
    projects: Map<u32, Option<Project>>,
    depth: u32,
}

/// One lookup treats excluded and absent projects alike.
#[derive(Debug)]
#[cfg_attr(not(test), expect(dead_code, reason = "public reads consume the index in the public-reads-core task"))]
pub(crate) enum Entry<'a> {
    Busy,
    Ready(&'a Project),
}

#[derive(Debug)]
pub(crate) struct Project {
    pub(crate) name: Box<str>,
    live: Box<[u64]>,
    live_folded: u32,
    trees: Map<u64, Tree>,
    ended: List<Frozen>,
    relations: Map<u64, Relations>,
}

#[derive(Debug)]
struct Tree {
    line: PublicLine,
    lines: Box<[PublicLine]>,
    ids: Box<[u64]>,
    folded: u32,
    omitted: Map<u64, u32>,
}

#[derive(Debug)]
#[expect(
    clippy::partial_pub_fields,
    reason = "the frozen record exposes its public lines; the index owns the identifier cache"
)]
pub(crate) struct Frozen {
    pub line: PublicLine,
    pub tree: Box<[PublicLine]>,
    pub folded: u32,
    ids: Box<[u64]>,
}

#[derive(Debug)]
struct Relations {
    goal: u64,
    location: Location,
    lineage: Box<[u64]>,
    delegates: Box<[u64]>,
    folded: u32,
}

#[derive(Clone, Copy, Debug)]
enum Location {
    Live { line: u32 },
    Ended { goal: u32, line: u32 },
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum Change {
    Made { task: u64 },
    Tracked { task: u64 },
    Untracked { task: u64 },
    Moved { task: u64 },
    Ended { task: u64 },
    Priority { task: u64 },
}

/// A decision's borrow, including the ending row after it leaves the live arena.
pub(crate) struct CoreTasks<'a> {
    tasks: &'a tasks::Domain,
    names: &'a Map<u32, Box<str>>,
    changed: Option<&'a tasks::TaskRecord>,
}

impl Index {
    pub(crate) fn new(projects: u32, depth: u32) -> Self {
        Self { projects: Map::with_capacity(projects), depth }
    }

    /// Mark a restored public project before its tasks have been restored.
    fn pending(&mut self, project: u32) {
        if !self.projects.contains_key(&project) {
            let _admitted = self.projects.insert(project, None);
        }
    }

    /// Build from live tasks at publication and after restoration.
    #[expect(clippy::trivially_copy_pass_by_ref, reason = "the index interface borrows its configured public limits")]
    pub(crate) fn publish(&mut self, core: &CoreTasks<'_>, limits: &PublicIndexLimits, project: u32) {
        self.pending(project);
        let Some(slot) = self.projects.get_mut(&project) else { return };
        if slot.is_none() {
            *slot = Some(Project::new(core.names.get(&project).expect("public project has a name"), *limits));
        }
        slot.as_mut().expect("built entry").rebuild(core, *limits, project, self.depth);
    }

    pub(crate) fn withdraw(&mut self, project: u32) {
        drop(self.projects.remove(&project));
    }

    /// Follow the row saved by the decision, including title and phase changes.
    #[expect(clippy::trivially_copy_pass_by_ref, reason = "the index interface borrows its configured public limits")]
    pub(crate) fn follow(&mut self, core: &CoreTasks<'_>, limits: &PublicIndexLimits, change: Change) {
        let task = match change {
            Change::Made { task }
            | Change::Tracked { task }
            | Change::Untracked { task }
            | Change::Moved { task }
            | Change::Ended { task }
            | Change::Priority { task } => task,
        };
        let row = match core.changed {
            Some(row) => Some(row),
            None => core.tasks.task(task),
        };
        let Some(row) = row else { return };
        let Some(Some(project)) = self.projects.get_mut(&row.project) else { return };
        match change {
            Change::Ended { .. } => {
                let goal = project.is_goal(core, row, self.depth);
                project.end(row, *limits, goal);
            }
            Change::Made { .. }
            | Change::Tracked { .. }
            | Change::Untracked { .. }
            | Change::Moved { .. }
            | Change::Priority { .. } => {
                if project.refresh(row) {
                    project.relink(*limits);
                    return;
                }
            }
        }
        project.rebuild(core, *limits, row.project, self.depth);
    }

    pub(crate) fn projects(&self) -> impl Iterator<Item = (u32, &Project)> {
        self.projects.iter().filter_map(|(&number, project)| match project {
            Some(project) => Some((number, project)),
            None => None,
        })
    }

    #[cfg_attr(not(test), expect(dead_code, reason = "public reads consume the index in the public-reads-core task"))]
    pub(crate) fn project(&self, project: u32) -> Option<Entry<'_>> {
        match self.projects.get(&project) {
            Some(Some(project)) => Some(Entry::Ready(project)),
            Some(None) => Some(Entry::Busy),
            None => None,
        }
    }
}

#[cfg_attr(
    not(test),
    expect(dead_code, reason = "public reads consume these bounded projections in the public-reads-core task")
)]
impl Project {
    fn new(name: &str, limits: PublicIndexLimits) -> Self {
        Self {
            name: Box::from(name),
            live: Box::new([]),
            live_folded: 0,
            trees: Map::with_capacity(u32::from(limits.goals)),
            ended: List::with_capacity(u32::from(limits.ended)),
            relations: Map::with_capacity(relations::capacity(limits)),
        }
    }

    pub(crate) fn live(&self) -> &[u64] {
        &self.live
    }
    pub(crate) fn live_folded(&self) -> u32 {
        self.live_folded
    }
    pub(crate) fn ended(&self) -> &[Frozen] {
        self.ended.as_slice()
    }
    pub(crate) fn goal_of(&self, task: u64) -> Option<u64> {
        match self.relations.get(&task) {
            Some(row) => Some(row.goal),
            None => None,
        }
    }
    pub(crate) fn tree(&self, goal: u64) -> Option<(&[u64], u32)> {
        if let Some(tree) = self.trees.get(&goal) {
            return Some((&tree.ids, tree.folded));
        }
        for frozen in &self.ended {
            if frozen.line.task == goal {
                return Some((&frozen.ids, frozen.folded));
            }
        }
        None
    }
    pub(crate) fn lineage(&self, task: u64) -> Option<&[u64]> {
        match self.relations.get(&task) {
            Some(row) => Some(&row.lineage),
            None => None,
        }
    }
    pub(crate) fn delegates(&self, task: u64) -> (&[u64], u32) {
        match self.relations.get(&task) {
            Some(row) => (&row.delegates, row.folded),
            None => (&[], 0),
        }
    }
    pub(crate) fn counts(&self, task: u64) -> PublicCounts {
        match self.line(task) {
            Some(line) => line.delegates,
            None => relations::ZERO,
        }
    }
    pub(crate) fn public_tasks(&self) -> Box<[u64]> {
        let mut tasks = List::with_capacity(self.relations.len());
        for (&number, _) in &self.relations {
            tasks.push(number).expect("bounded indexed tasks");
        }
        tasks.into_boxed()
    }
    pub(crate) fn line(&self, task: u64) -> Option<&PublicLine> {
        let relation = self.relations.get(&task)?;
        let (root, tree, position) = match relation.location {
            Location::Live { line } => {
                let goal = self.trees.get(&relation.goal)?;
                (&goal.line, &*goal.lines, line)
            }
            Location::Ended { goal, line } => {
                let goal = self.ended.get(goal)?;
                (&goal.line, &*goal.tree, line)
            }
        };
        if position == 0 {
            return Some(root);
        }
        tree.get(skein_lib::length::usize_of(position.checked_sub(1).expect("nonroot line position")))
    }

    /// A title or lifecycle change keeps membership and ordering as they were.
    fn refresh(&mut self, row: &tasks::TaskRecord) -> bool {
        let Some(goal) = self.goal_of(row.number) else { return false };
        let Some(previous) = self.line(row.number) else { return false };
        let parent = if goal == row.number {
            None
        } else {
            match row.requester {
                tasks::Party::Task(parent) => Some(parent),
                tasks::Party::Person(_) | tasks::Party::Deployment { .. } => None,
            }
        };
        if parent != previous.parent || row.tracked != previous.priority {
            return false;
        }
        let line = super::line::make(row, parent, previous.delegates);
        let Some(tree) = self.trees.get_mut(&goal) else { return false };
        if row.number == goal {
            tree.line = line;
            return true;
        }
        for stored in &mut tree.lines {
            if stored.task == row.number {
                *stored = line;
                return true;
            }
        }
        false
    }

    fn end(&mut self, row: &tasks::TaskRecord, limits: PublicIndexLimits, goal: bool) {
        if goal {
            let tree = self.trees.remove(&row.number);
            if limits.ended == 0 {
                return;
            }
            let frozen = match tree {
                Some(tree) => Frozen {
                    line: super::line::make(row, None, tree.line.delegates),
                    tree: tree.lines,
                    folded: tree.folded,
                    ids: tree.ids,
                },
                None => Frozen {
                    line: super::line::make(row, None, relations::ZERO),
                    tree: Box::new([]),
                    folded: 0,
                    ids: Box::new([]),
                },
            };
            let mut ended = List::with_capacity(u32::from(limits.ended));
            ended.push(frozen).expect("nonzero ended bound");
            let previous = core::mem::replace(&mut self.ended, List::with_capacity(0));
            for goal in previous.into_boxed() {
                if ended.room() == 0 {
                    break;
                }
                ended.push(goal).expect("ended prefix room");
            }
            self.ended = ended;
        } else {
            for &goal in &self.live {
                let Some(tree) = self.trees.get_mut(&goal) else { continue };
                for line in &mut tree.lines {
                    if line.task == row.number {
                        *line = super::line::make(row, line.parent, line.delegates);
                    }
                }
            }
        }
    }
}
