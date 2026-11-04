//! Membership and tree order share one requester walk (jig/docs/design/domain/people.md, 13.7).
use super::{CoreTasks, Project, Tree, relations};
use crate::{PublicIndexLimits, PublicLine};
use alloc::boxed::Box;
use jig_domain_tasks as tasks;
use skein_lib::{List, Map};

struct Source<'a> {
    core: &'a CoreTasks<'a>,
    previous: &'a Project,
    project: u32,
    depth: u32,
}

struct Place {
    parent: Option<u64>,
    priority: Option<u32>,
    excluded: bool,
    ended: bool,
}

impl Source<'_> {
    fn row(&self, task: u64) -> Option<&tasks::TaskRecord> {
        if let Some(row) = self.core.tasks.task(task) {
            return Some(row);
        }
        match self.core.changed {
            Some(row) if row.number == task => Some(row),
            Some(_) | None => None,
        }
    }

    fn place(&self, task: u64) -> Option<Place> {
        if let Some(row) = self.row(task) {
            if row.project != self.project {
                return None;
            }
            return Some(Place {
                parent: match row.requester {
                    tasks::Party::Task(parent) => Some(parent),
                    tasks::Party::Person(_) | tasks::Party::Deployment { .. } => None,
                },
                priority: row.tracked,
                excluded: row.flags.conversation || row.recurring.is_some(),
                ended: row.ended_at.is_some(),
            });
        }
        let line = self.previous.line(task)?;
        Some(Place { parent: line.parent, priority: line.priority, excluded: false, ended: line.ended.is_some() })
    }

    /// The rule: the outermost tracked ancestor, with ancestry cut at that goal.
    /// Excluded ancestors above the goal (such as its conversation) never cross.
    fn path(&self, task: u64) -> Option<Box<[u64]>> {
        let mut reversed = List::with_capacity(self.depth.checked_add(1).expect("bounded task depth"));
        let mut next = Some(task);
        let mut goal = None;
        let mut excluded = false;
        let mut goal_excluded = false;
        for _ in 0..=self.depth {
            let Some(number) = next else { break };
            let place = self.place(number)?;
            reversed.push(number).expect("requester walk bounded by task depth");
            excluded |= place.excluded;
            if place.priority.is_some() {
                goal = Some(reversed.len());
                goal_excluded = excluded;
            }
            next = place.parent;
        }
        if next.is_some() {
            return None;
        }
        if goal_excluded {
            return None;
        }
        let length = goal?;
        let mut path = List::with_capacity(length);
        for index in (0..length).rev() {
            path.push(*reversed.get(index).expect("walk prefix")).expect("same prefix bound");
        }
        Some(path.into_boxed())
    }

    fn line(&self, task: u64, parent: Option<u64>) -> PublicLine {
        match self.row(task) {
            Some(row) => crate::public::line::make(row, parent, relations::ZERO),
            None => {
                let mut line = self.previous.line(task).expect("retained public task").clone();
                line.parent = parent;
                line.delegates = relations::ZERO;
                line
            }
        }
    }

    fn candidates(&self) -> Box<[u64]> {
        // The live snapshot is bounded by tasks.tasks; retained lines add at most
        // goals × (lines + 1). Only decision work allocates this temporary list.
        let live = self.core.tasks.view_tasks();
        let mut capacity = u32::try_from(live.len()).expect("bounded live tasks");
        for (_, tree) in &self.previous.trees {
            capacity = capacity
                .checked_add(1)
                .expect("bounded candidates")
                .checked_add(u32::try_from(tree.ids.len()).expect("bounded tree"))
                .expect("bounded candidates");
        }
        let mut numbers = List::with_capacity(capacity);
        for row in &live {
            numbers.push(row.number).expect("live snapshot bound");
        }
        for (_, tree) in &self.previous.trees {
            if self.core.tasks.task(tree.line.task).is_none() {
                numbers.push(tree.line.task).expect("retained candidates bound");
            }
            for &task in &tree.ids {
                if self.core.tasks.task(task).is_none() {
                    numbers.push(task).expect("retained candidates bound");
                }
            }
        }
        numbers.into_boxed()
    }
}

impl Project {
    pub(super) fn is_goal(&self, core: &CoreTasks<'_>, row: &tasks::TaskRecord, depth: u32) -> bool {
        let source = Source { core, previous: self, project: row.project, depth };
        match source.path(row.number) {
            Some(path) => path.len() == 1,
            None => false,
        }
    }

    pub(super) fn rebuild(&mut self, core: &CoreTasks<'_>, limits: PublicIndexLimits, project: u32, depth: u32) {
        let source = Source { core, previous: self, project, depth };
        let candidates = source.candidates();
        let mut roots = Map::with_capacity(u32::from(limits.goals));
        let mut total = 0_u32;
        for &task in &candidates {
            let Some(path) = source.path(task) else { continue };
            if path.len() != 1 {
                continue;
            }
            let place = source.place(task).expect("path has a place");
            if place.ended {
                continue;
            }
            let priority = place.priority.expect("public goal tracked");
            total = total.checked_add(1).expect("bounded goal candidates");
            let key = (priority, task);
            if roots.len() == u32::from(limits.goals) {
                let Some((&last, _)) = roots.last() else { continue };
                if key >= last {
                    continue;
                }
                let _removed = roots.remove(&last);
            }
            roots.insert(key, task).expect("priority prefix room checked");
        }
        let mut live = List::with_capacity(u32::from(limits.goals));
        let mut trees = Map::with_capacity(u32::from(limits.goals));
        for (_, &goal) in &roots {
            live.push(goal).expect("selected goals bound");
            let tree = build_tree(&source, limits, goal, &candidates);
            trees.insert(goal, tree).expect("one tree per selected goal");
        }
        self.live_folded = total.checked_sub(live.len()).expect("selected subset of goals");
        self.live = live.into_boxed();
        self.trees = trees;
        self.name = core.names.get(&project).expect("public project's name").clone();
        self.relink(limits);
    }
}

fn build_tree(source: &Source<'_>, limits: PublicIndexLimits, goal: u64, candidates: &[u64]) -> Tree {
    let mut selected = Map::<Box<[u64]>, u64>::with_capacity(u32::from(limits.lines));
    let mut total = 0_u32;
    for &task in candidates {
        let Some(path) = source.path(task) else { continue };
        if path.first() != Some(&goal) || task == goal {
            continue;
        }
        total = total.checked_add(1).expect("bounded tree candidates");
        if path.len().saturating_sub(1) > usize::from(limits.depth) {
            continue;
        }
        if selected.len() == u32::from(limits.lines) {
            let Some((last, _)) = selected.last() else { continue };
            if &path >= last {
                continue;
            }
            let last = last.clone();
            let _removed = selected.remove(&last);
        }
        selected.insert(path, task).expect("tree prefix room checked");
    }
    let mut lines = List::with_capacity(u32::from(limits.lines));
    let mut ids = List::with_capacity(u32::from(limits.lines));
    let mut omitted = Map::<u64, u32>::with_capacity(u32::from(limits.lines).checked_add(1).expect("goal and lines"));
    omitted.insert(goal, 0).expect("goal slot");
    for (path, &task) in &selected {
        let parent = path.get(path.len().checked_sub(2).expect("nonroot public line")).copied();
        lines.push(source.line(task, parent)).expect("selected lines bound");
        ids.push(task).expect("same selected lines bound");
        omitted.insert(task, 0).expect("one counter per selected line");
    }
    for &task in candidates {
        let Some(path) = source.path(task) else { continue };
        if path.first() != Some(&goal) || task == goal || ids.as_slice().contains(&task) {
            continue;
        }
        let parent = path.get(path.len().checked_sub(2).expect("nonroot candidate")).expect("parent in path");
        if let Some(count) = omitted.get_mut(parent) {
            *count = count.checked_add(1).expect("bounded omitted delegates");
        }
    }
    Tree {
        line: source.line(goal, None),
        folded: total.checked_sub(lines.len()).expect("selected subset"),
        lines: lines.into_boxed(),
        ids: ids.into_boxed(),
        omitted,
    }
}
