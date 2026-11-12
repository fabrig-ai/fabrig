//! Volatile evidence of Forgejo settling, beside the change's persisted push.
use skein_lib::{Duration, Env, Wall};
use temper_engine_domain_forge_change as change;

use crate::{ChangeRow, Domain, Limits};
type Commit = skein_forge::api::Commit;

/// Hints and fresh reads, retained only for this incarnation of a task.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) struct Settle {
    since: Option<Wall>,
    base: Option<Commit>,
    read_base: Option<Commit>,
    open: Option<(Wall, Wall)>,
    pub(crate) branch: crate::domain::BranchRead,
    pub(crate) contains_head: Option<HeadContainment>,
}

/// One reverse ancestry answer per unchanged head and tip (domain/forge/change/README.md, 2).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) struct HeadContainment {
    pub(crate) head: Commit,
    pub(crate) tip: Commit,
    pub(crate) status: change::Status,
}

impl Settle {
    pub(crate) const NEW: Settle = Settle {
        since: None,
        base: None,
        read_base: None,
        open: None,
        branch: crate::domain::BranchRead::NEW,
        contains_head: None,
    };

    /// The branch tip used for the last decision's base CI and comparison.
    pub(crate) fn read_base(self) -> Option<Commit> {
        self.read_base
    }

    /// The pull and branch can disagree while the forge settles.
    pub(crate) fn uses_base(self, head: Commit) -> bool {
        self.base == Some(head) || self.read_base == Some(head)
    }

    /// First reads open no window: only a prior base read detects a move.
    pub(crate) fn fold(mut self, now: Wall, own: Wall, settle: Duration, moving: bool, base: Option<Commit>) -> Settle {
        if self.base.is_some() && self.base != base {
            self.since = Some(now);
        }
        self.base = base;
        let unsettled = moving;
        if !unsettled {
            self.since = None;
        }
        let start = own.max(self.since.unwrap_or(own));
        let end = Wall::from_nanos(start.as_nanos().saturating_add(settle.as_nanos()));
        self.open = if unsettled && now < end { Some((start, end)) } else { None };
        self
    }
}

/// Only a named branch move advances mergeability's start.
pub(crate) fn hinted(d: &mut Domain, env: &Env<Limits>, hint: &skein_forge::api::Hint, tasks: &[u64]) {
    for task in tasks {
        let row = d.changes.get(task).expect("a change the hint wakes");
        match &hint.change {
            skein_forge::api::Change::Branch(branch) | skein_forge::api::Change::BranchMoved { branch, .. } => {
                if *row.branch != **branch && *row.base != **branch {
                    continue;
                }
            }
            skein_forge::api::Change::Item(_) | skein_forge::api::Change::Commit(_) => continue,
        }
        let mut settle = d.settling.get(task).copied().unwrap_or(Settle::NEW);
        settle.since = Some(env.wall);
        d.settling.insert(*task, settle).expect("one settle row per change");
    }
}

/// Fold the fresh pull before constructing this step's mergeability facts.
pub(crate) fn observe(
    d: &mut Domain,
    env: &Env<Limits>,
    row: &ChangeRow,
    pull: Option<&skein_forge::api::Pull>,
    base_tip: Commit,
) {
    let (moving, base) = match pull {
        Some(pull) => {
            (pull.state == skein_forge::api::State::Open && pull.merged.is_none() && !pull.mergeable, pull.base_commit)
        }
        None => (false, None),
    };
    let mut settle = d.settling.get(&row.task).copied().unwrap_or(Settle::NEW).fold(
        env.wall,
        row.change.since,
        env.limits.client.settle,
        moving,
        base,
    );
    settle.read_base = Some(base_tip);
    d.settling.insert(row.task, settle).expect("one settle row per change");
}

pub(crate) fn window(d: &Domain, task: u64) -> Option<(Wall, Wall)> {
    d.settling.get(&task)?.open
}

/// A watched pull wakes the first change when mergeability settles; its
/// deadline bounds a persistent conflict. Other waits still retry their reads.
pub(crate) fn cap(
    d: &Domain,
    env: &Env<Limits>,
    row: &ChangeRow,
    facts: &change::Facts,
    decision: change::Decision,
) -> change::Decision {
    let Some((start, end)) = window(d, row.task) else { return decision };
    match decision {
        change::Decision::Wait { until: Some(until) } => {
            if let change::State::First { .. } = row.change.state
                && facts.contains_base == change::Status::Passed
                && facts.mergeable == change::Status::Pending
                && !facts.writer_taken
                && watches_pull(d, row)
            {
                return change::Decision::Wait { until: Some(end.min(until)) };
            }
            let hinted = env.limits.client.hinted.as_nanos();
            let interval = env
                .wall
                .as_nanos()
                .saturating_sub(start.as_nanos())
                .max(hinted)
                .min(env.limits.client.poll.as_nanos().max(hinted));
            let next = Wall::from_nanos(env.wall.as_nanos().saturating_add(interval));
            change::Decision::Wait { until: Some(next.min(end).min(until)) }
        }
        change::Decision::Wait { until: None }
        | change::Decision::None
        | change::Decision::CancelProducer
        | change::Decision::Delegate(_)
        | change::Decision::CancelGates { .. }
        | change::Decision::CancelRepair { .. }
        | change::Decision::CancelResolution { .. }
        | change::Decision::EndHold
        | change::Decision::RepairEnded { .. }
        | change::Decision::Effect(_)
        | change::Decision::Ready
        | change::Decision::QueueRepair
        | change::Decision::Finish { .. }
        | change::Decision::Cancel
        | change::Decision::NoChange
        | change::Decision::Hold(_) => decision,
    }
}

fn watches_pull(d: &Domain, row: &ChangeRow) -> bool {
    let Some(pull) = row.pull else { return false };
    let Some(names) = d.names(row.task) else { return false };
    for name in names {
        if name.forge == row.repository.forge
            && name.repository == row.repository.repository
            && name.what == crate::What::Pull(pull)
        {
            return true;
        }
    }
    false
}
