//! A project's whole queues and retained titles (temper/docs/design/domain/forge/queues.md;
//! temper/docs/design/client/forge/records.md; temper/docs/design/client/forge/README.md, 3).
use crate::{ChangeRow, Domain, LandingRed, Limits};
use alloc::boxed::Box;
use core::cmp::Reverse;
use skein_lib::{List, Map, Wall};

const QUEUES: u32 = 32;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ChangesView {
    pub project: u32,
    pub landing: Box<[u8]>,
    pub queues: Box<[ChangesQueue]>,
    pub more: u32,
    pub landed: Box<[ChangesLanded]>,
    pub landed_more: u32,
    pub landed_default: Box<[ChangesLanded]>,
    pub landed_default_more: u32,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ChangesQueue {
    pub repository: Box<[u8]>,
    pub branch: Box<[u8]>,
    pub red: bool,
    pub changes: Box<[ChangesQueued]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ChangesQueued {
    pub task: u64,
    pub title: Box<[u8]>,
    pub ready_since: Wall,
    pub checks_since: Option<Wall>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ChangesLanded {
    pub task: u64,
    pub title: Box<[u8]>,
    pub at: Wall,
    pub commit: [u8; 32],
    pub default_branch: bool,
}

impl Domain {
    pub(crate) fn changes_view(&self, project: u32, now: Wall, limits: &Limits) -> Option<ChangesView> {
        let mut branches = Map::with_capacity(limits.changes);
        for (_, row) in &self.changes {
            if self.repositories.get(&row.repository)?.project != project {
                continue;
            }
            let red = self.branch_red(row.repository, &row.base);
            if crate::panel::ready(row).is_some() || owns(row) || red {
                branches.insert((self.repository_address(row)?, row.base.clone(), row.repository), row.task).ok()?;
            }
        }
        let mut queues = List::with_capacity(QUEUES);
        let mut left = limits.changes_rows;
        let mut more = 0_u32;
        for ((repository, branch, _), task) in &branches {
            let row = self.changes.get(task)?;
            let ordered = self.changes_order(row, now, limits)?;
            let mut changes = List::with_capacity(left);
            for task in &ordered {
                if left == 0 || queues.room() == 0 {
                    more = more.checked_add(1)?;
                    continue;
                }
                let row = self.changes.get(task)?;
                let ready_since = match crate::panel::ready(row) {
                    Some(ready) => ready.since,
                    None => ready_since(row),
                };
                let checks_since = if changes.is_empty() { Some(row.change.since) } else { None };
                changes
                    .push(ChangesQueued {
                        task: *task,
                        title: crate::text::fit(&row.title, crate::LOGGED_TITLE_BYTES, "…".as_bytes()),
                        ready_since,
                        checks_since,
                    })
                    .ok()?;
                left = left.checked_sub(1)?;
            }
            if queues.room() > 0 {
                queues
                    .push(ChangesQueue {
                        repository: repository.clone(),
                        branch: branch.clone(),
                        red: self.branch_red(row.repository, &row.base),
                        changes: changes.into_boxed(),
                    })
                    .ok()?;
            }
        }
        let mut view = ChangesView {
            project,
            landing: Box::default(),
            queues: queues.into_boxed(),
            more,
            landed: Box::new([]),
            landed_more: 0,
            landed_default: Box::new([]),
            landed_default_more: 0,
        };
        self.changes_history(&mut view, limits)?;
        Some(view)
    }

    fn changes_history(&self, view: &mut ChangesView, limits: &Limits) -> Option<()> {
        let mut recent = Map::with_capacity(
            limits.repositories.checked_mul(limits.landings_kept)?.checked_mul(limits.landing_members)?,
        );
        let mut newest = None;
        let mut landing: Box<[u8]> = Box::default();
        for (repository, log) in &self.landing_logs {
            let adopted = self.repositories.get(repository)?;
            if adopted.project != view.project {
                continue;
            }
            for logged in &log.landings {
                let key = (Reverse(logged.at), *repository, logged.commit);
                let first = match newest {
                    Some(previous) => key < previous,
                    None => true,
                };
                if first {
                    newest = Some(key);
                    landing.clone_from(&logged.branch);
                }
                for change in &logged.changes {
                    recent
                        .insert(
                            (key, change.task),
                            ChangesLanded {
                                task: change.task,
                                title: change.title.clone(),
                                at: logged.at,
                                commit: logged.commit,
                                default_branch: logged.branch == adopted.settings.default_branch,
                            },
                        )
                        .ok()?;
                }
            }
        }
        if newest.is_none() {
            for (_, repository) in &self.repositories {
                if repository.project == view.project && (landing.is_empty() || repository.home) {
                    landing.clone_from(&repository.settings.default_branch);
                    if repository.home {
                        break;
                    }
                }
            }
        }
        let mut landed = List::with_capacity(limits.changes_landed);
        let mut landed_more = 0_u32;
        let mut landed_default = List::with_capacity(limits.changes_landed);
        let mut landed_default_more = 0_u32;
        for (_, change) in &recent {
            if change.default_branch {
                if landed_default.room() == 0 {
                    landed_default_more = landed_default_more.checked_add(1)?;
                } else {
                    landed_default.push(change.clone()).ok()?;
                }
            }
            if landed.room() == 0 {
                landed_more = landed_more.checked_add(1)?;
            } else {
                landed.push(change.clone()).ok()?;
            }
        }
        view.landing = landing;
        view.landed = landed.into_boxed();
        view.landed_more = landed_more;
        view.landed_default = landed_default.into_boxed();
        view.landed_default_more = landed_default_more;
        Some(())
    }

    pub(crate) fn changes_order(&self, row: &ChangeRow, now: Wall, limits: &Limits) -> Option<List<u64>> {
        let (candidates, repair) = crate::domain::queue_ready(self, row, limits);
        let mut active = Map::with_capacity(limits.changes);
        for (_, other) in &self.changes {
            if other.repository == row.repository
                && other.base == row.base
                && owns(other)
                && (crate::panel::ready(other).is_none() || repair.is_none())
                && (!crate::domain::in_queue(&other.change.state) || !self.approval_short(other))
            {
                active.insert((ready_since(other), other.task), other.task).ok()?;
            }
        }
        let mut ordered = List::with_capacity(limits.changes);
        if let Some(repair) = repair {
            ordered.push(repair).ok()?;
        }
        for (_, task) in &active {
            if Some(*task) != repair {
                ordered.push(*task).ok()?;
            }
        }
        let candidates = crate::public::order_ready(candidates, repair, limits, now)?;
        for candidate in &candidates {
            if !ordered.as_slice().contains(&candidate.task) {
                ordered.push(candidate.task).ok()?;
            }
        }
        Some(ordered)
    }

    pub(crate) fn branch_red(&self, repository: skein_forge::api::Repository, base: &[u8]) -> bool {
        for (_, change) in &self.changes {
            if change.repository == repository
                && change.base.as_ref() == base
                && change.base_repair
                && !change.terminal()
            {
                return true;
            }
        }
        for ((held, commit), verification) in &self.verifications {
            if *held != repository {
                continue;
            }
            for task in &verification.changes {
                if let Some(change) = self.changes.get(task)
                    && change.base.as_ref() == base
                    && self.landing_red(*held, commit, base) == LandingRed::Red
                {
                    return true;
                }
            }
        }
        false
    }
}
fn owns(row: &ChangeRow) -> bool {
    crate::domain::owns_turn(row)
}

fn ready_since(row: &ChangeRow) -> Wall {
    use temper_engine_domain_forge_change::State;
    match row.change.state {
        State::Queued { ready_since, .. }
        | State::First { ready_since, .. }
        | State::Updating { ready_since, .. }
        | State::Ready { ready_since, .. }
        | State::Preparing { ready_since, .. } => ready_since,
        State::Producing { .. }
        | State::Opening { .. }
        | State::Recreating { .. }
        | State::Reopening { .. }
        | State::Checking { .. }
        | State::Gating { .. }
        | State::CancellingGates { .. }
        | State::WaitingProducer { .. }
        | State::Resolving { .. }
        | State::Repairing { .. }
        | State::Landing { .. }
        | State::Landed { .. }
        | State::NoChange { .. }
        | State::Ended
        | State::Held { .. }
        | State::Red { .. }
        | State::Cleared { .. }
        | State::Uncleared { .. } => row.change.ready_since.unwrap_or(row.change.since),
    }
}
