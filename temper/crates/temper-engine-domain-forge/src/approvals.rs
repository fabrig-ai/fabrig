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
    /// Its enforcing approvals still short, in its panel's order, each with its people counted.
    pub short: Box<[ShortApproval]>,
}

/// A short approval and the sets of its rule touched at this head
/// (domain/forge/change/questions.md, 2).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ShortApproval {
    pub scope: u32,
    pub rule: u32,
    pub role: u32,
    pub required: u32,
    pub counted: Box<[crate::Reviewer]>,
    pub sets: u8,
}

impl Domain {
    /// Kept facts at this head show an enforcing approval short and none refused
    /// (domain/forge/queues.md, Out of the queue while a sign-off waits).
    pub(crate) fn short_approvals(&self, row: &crate::ChangeRow) -> Box<[PanelApproval]> {
        let mut short = List::uncapped();
        for approval in self.panel_approvals(row) {
            if approval.refused {
                return Box::new([]);
            }
            if let Some(remaining) = approval.remaining
                && remaining > 0
            {
                short.push(approval).expect("admitted approvals");
            }
        }
        short.into_boxed()
    }

    pub(crate) fn approval_short(&self, row: &crate::ChangeRow) -> bool {
        !self.short_approvals(row).is_empty()
    }

    /// Every live change in state `Queued`, `First` or `Landing`, with a pull request, whose review facts
    /// are at its last head, with an approval still short and none refused; oldest `since`
    /// first, then by task. Allocates; changes nothing.
    #[must_use]
    pub fn approval_waits(&self, _now: Wall, limits: &Limits) -> Box<[ApprovalWait]> {
        let mut waits = Map::with_capacity(limits.changes);
        for (task, row) in &self.changes {
            if row.terminal() {
                continue;
            }
            match row.change.state {
                State::Queued { .. } | State::First { .. } | State::Landing { .. } => {}
                State::Producing { .. }
                | State::Opening { .. }
                | State::Recreating { .. }
                | State::Reopening { .. }
                | State::Checking { .. }
                | State::Judging { .. }
                | State::Gating { .. }
                | State::Ready { .. }
                | State::Preparing { .. }
                | State::CancellingGates { .. }
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
            let short = self.short_approvals(row);
            if short.is_empty() {
                continue;
            }
            let since = self.review_since(*task).expect("kept review facts have their first time");
            let mut approvals = List::uncapped();
            let rules = self.landing_rules(repository.project);
            let touched = self.touched_at(row);
            for approval in short {
                let mut sets = 0;
                for held in &rules {
                    if held.scope == approval.scope && held.number == approval.rule {
                        for number in &held.rule.sets {
                            sets |= crate::gates::set_bit(*number) & touched.sets;
                        }
                    }
                }
                approvals
                    .push(ShortApproval {
                        scope: approval.scope,
                        rule: approval.rule,
                        role: approval.role,
                        required: approval.required,
                        counted: approval.counted,
                        sets,
                    })
                    .expect("admitted approvals");
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
                        short: approvals.into_boxed(),
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
