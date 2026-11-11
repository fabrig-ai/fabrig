//! Changes waiting on a person's approval (domain/forge/change/questions.md, 2).
use alloc::boxed::Box;
use skein_lib::{List, Map, Wall};
use temper_engine_domain_forge_change::State;

use crate::{Domain, Limits, PanelApproval};

/// A live change whose merge waits on a person's approval, before the people are joined
/// (domain/forge/change/questions.md, 2).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ApprovalWait {
    pub task: u64,
    pub project: u32,
    pub forge: u16,
    pub owner: Box<[u8]>,
    pub name: Box<[u8]>,
    pub pull: u64,
    pub title: Box<[u8]>,
    /// When its review facts at its last head were first kept.
    pub since: Wall,
    /// Whether its reviews were read whole.
    pub whole: bool,
    pub size: Option<skein_forge::api::Size>,
    /// The changes after it in its landing branch's order (domain/forge/queues.md).
    pub behind: u32,
    /// Its enforcing approvals still short, in its panel's order, each with its people counted.
    pub short: Box<[PanelApproval]>,
}

impl Domain {
    /// Every live change in state `First` or `Landing`, with a pull request, whose review facts
    /// are at its last head, with an approval still short and none refused; oldest `since`
    /// first, then by task. Allocates; changes nothing.
    #[must_use]
    pub fn approval_waits(&self, now: Wall, limits: &Limits) -> Box<[ApprovalWait]> {
        let mut waits = Map::with_capacity(limits.changes);
        for (task, row) in &self.changes {
            if row.terminal() {
                continue;
            }
            match row.change.state {
                State::First { .. } | State::Landing { .. } => {}
                State::Producing { .. }
                | State::Opening { .. }
                | State::Recreating { .. }
                | State::Reopening { .. }
                | State::Checking { .. }
                | State::Gating { .. }
                | State::Ready { .. }
                | State::Preparing { .. }
                | State::CancellingGates { .. }
                | State::Queued { .. }
                | State::Updating { .. }
                | State::WaitingProducer { .. }
                | State::Resolving { .. }
                | State::Repairing { .. }
                | State::Landed { .. }
                | State::NoChange { .. }
                | State::Ended
                | State::Held { .. }
                | State::Red { .. }
                | State::Cleared { .. }
                | State::Uncleared { .. } => continue,
            }
            let Some(pull) = row.pull else { continue };
            let Some(repository) = self.repositories.get(&row.repository) else { continue };
            let Some(facts) = self.review_facts.get(task) else { continue };
            if facts.head != row.change.last_head {
                continue;
            }
            let mut short = List::uncapped();
            let mut refused = false;
            for approval in self.panel_approvals(row) {
                refused |= approval.refused;
                if let Some(remaining) = approval.remaining
                    && remaining > 0
                {
                    short.push(approval).expect("admitted approvals");
                }
            }
            if refused || short.is_empty() {
                continue;
            }
            let since = *self.review_since.get(task).expect("kept review facts have their first time");
            let mut behind = 0_u32;
            let mut found = false;
            if let Some(ordered) = self.changes_order(row, now, limits) {
                for other in &ordered {
                    if found {
                        behind = behind.saturating_add(1);
                    }
                    found |= other == task;
                }
            }
            waits
                .insert(
                    (since, *task),
                    ApprovalWait {
                        task: *task,
                        project: repository.project,
                        forge: row.repository.forge,
                        owner: repository.owner.clone(),
                        name: repository.name.clone(),
                        pull,
                        title: crate::text::fit(&row.title, crate::LOGGED_TITLE_BYTES, "…".as_bytes()),
                        since,
                        whole: facts.reviews_complete,
                        size: facts.size,
                        behind,
                        short: short.into_boxed(),
                    },
                )
                .expect("one wait per bounded change");
        }
        let mut rows = List::with_capacity(limits.changes);
        for (_, wait) in &waits {
            rows.push(wait.clone()).expect("one wait per bounded change");
        }
        rows.into_boxed()
    }
}
