//! The forge's milestone row (temper/docs/design/domain/milestones.md, 8).
//! Landing targets use the adoption (temper/docs/design/domain/forge/adoption.md, 12).
use crate::{Domain, Key, MilestoneLanding, MilestoneRow, Request, Stored};
use skein_lib::{List, Map, Queue, Wall};
pub(crate) use temper_engine_domain_forge_vocabulary::milestones::{MILESTONE_NAME_BYTES, MILESTONES_PER_PROJECT};

impl Domain {
    #[must_use]
    pub fn milestone(&self, task: u64) -> Option<&MilestoneRow> {
        self.milestones.get(&task)
    }

    /// Ended since the given wall, newest end first.
    #[must_use]
    pub fn ended_milestones(&self, project: u32, since: Wall, count: u32) -> List<MilestoneRow> {
        let mut ordered = Map::uncapped();
        for (_, row) in &self.milestones {
            if row.project == project
                && let Some(end) = &row.ended
                && end.at >= since
            {
                ordered.insert((end.at, row.task), row).expect("one entry per milestone");
            }
        }
        let mut rows = List::with_capacity(count.min(ordered.len()));
        for (_, row) in ordered.iter().rev() {
            if rows.len() == count {
                break;
            }
            rows.push((*row).clone()).expect("requested count");
        }
        rows
    }
}

pub(crate) fn project_rows(d: &Domain, project: u32) -> u32 {
    let mut count = 0u32;
    for (_, row) in &d.milestones {
        if row.project == project {
            count = count.saturating_add(1);
        }
    }
    count
}

/// Projection supplies identity and end; a landing stays as it was recorded.
pub(crate) fn project(d: &mut Domain, mut row: MilestoneRow, out: &mut Queue<Request>) {
    if let Some(previous) = d.milestones.get(&row.task) {
        row.last_landing = previous.last_landing.clone();
        if row.ended.is_none() {
            row.ended = previous.ended.clone();
        }
        if previous == &row {
            return;
        }
    }
    d.milestones.insert(row.task, row.clone()).expect("milestones bounded per project");
    out.push(Request::Save { read_afresh: None, release: None, record: Stored::Milestone(row.clone()) });
    while project_rows(d, row.project) > MILESTONES_PER_PROJECT {
        let mut oldest = None;
        for (task, candidate) in &d.milestones {
            if candidate.project == row.project
                && let Some(end) = &candidate.ended
            {
                let key = (end.at, *task);
                let earlier = match oldest {
                    Some(prior) => key < prior,
                    None => true,
                };
                if earlier {
                    oldest = Some(key);
                }
            }
        }
        let Some((_, task)) = oldest else { break };
        d.milestones.remove(&task);
        out.push(Request::Erase { key: Key::Milestone(task) });
    }
}

/// Both landing paths use the retained landing log's decision wall.
pub(crate) fn landed(d: &mut Domain, at: Wall, task: u64, out: &mut Queue<Request>) {
    let Some(change) = d.changes.get(&task) else { return };
    let Some(repository) = d.repositories.get(&change.repository) else { return };
    if change.base != repository.landing {
        return;
    }
    let (Some(milestone), Some(goal)) = (change.milestone, change.goal) else { return };
    let Some(row) = d.milestones.get_mut(&milestone) else { return };
    if let Some(previous) = &row.last_landing
        && previous.at >= at
    {
        return;
    }
    row.last_landing = Some(MilestoneLanding { at, goal, change: task });
    out.push(Request::Save { read_afresh: None, release: None, record: Stored::Milestone(row.clone()) });
}
