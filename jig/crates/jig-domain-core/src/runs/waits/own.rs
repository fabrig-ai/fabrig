//! The facts and clocks of a task's own waits (jig/docs/design/domain/engine/waits.md, 3 and 5).

use crate::{AtBound, Bound, Core, Limits, Unbounded, Wait, WaitKind, WaitObject, admission::escalation::wait_holder};
use alloc::collections::BTreeMap;
use jig_domain_people::{Entry, EntryKind, Proposer, Whom};
use jig_domain_tasks::{self as tasks, Escalation, EscalationHolder as Holder, Party, Phase, TaskRecord};
use skein_lib::{Duration, List, Wall};

use crate::documents::{TaskRunWait, TaskSourceState, TaskWaitWhy};

// The last key preserves the inbox's order when a proposal projects to
// several roles at the same time. All insertions come from bounded owner rows.
pub(super) type Records = BTreeMap<(u8, Wall, u64, usize), Wait>;

pub(super) struct Own {
    records: Records,
    hears: Option<Holder>,
}

impl Own {
    pub(super) const fn new(hears: Option<Holder>) -> Self {
        Self { records: BTreeMap::new(), hears }
    }

    fn push(&mut self, task: u64, kind: WaitKind, object: WaitObject, since: Wall, bound: Bound) {
        self.insert(Wait { task, kind, object, since, bound, hears: self.hears });
    }

    pub(super) fn insert(&mut self, wait: Wait) {
        let group = match wait.kind {
            WaitKind::Party => 0,
            WaitKind::Question => 1,
            WaitKind::Dependency => 2,
            WaitKind::Resource => 3,
            WaitKind::Account | WaitKind::Host => 4,
            WaitKind::Delegate => 5,
            WaitKind::News | WaitKind::Step | WaitKind::Room | WaitKind::Turn | WaitKind::SignOff => 6,
        };
        let object = match wait.object {
            WaitObject::Entry { kind, .. } => match kind {
                EntryKind::Question { message } | EntryKind::Reply { message } | EntryKind::ChatReply { message } => {
                    message
                }
                EntryKind::Proposal { number, .. } => number,
                EntryKind::Escalation { revision } => revision,
                EntryKind::PersonTask => wait.task,
            },
            WaitObject::Question { number, .. } | WaitObject::Task(number) => number,
            WaitObject::Resource { .. }
            | WaitObject::Run(_)
            | WaitObject::News { .. }
            | WaitObject::Procedure { .. } => 0,
        };
        let key = (group, wait.since, object, self.records.len());
        assert!(self.records.insert(key, wait).is_none(), "distinct wait position");
    }

    pub(super) fn finish(
        mut self,
        core: &Core,
        limits: &Limits,
        row: &TaskRecord,
        working: bool,
        parked: Option<Wall>,
        due_wait: Option<TaskRunWait>,
    ) -> Records {
        match row.phase {
            Phase::Closing(_) | Phase::Ended(_) => return self.records,
            Phase::Active(_) | Phase::Waiting | Phase::Held { .. } => {}
        }
        self.task_entries(core, limits, row);
        self.questions(core, limits, row);
        if !working {
            self.dependencies(core, row);
            self.resource(core, limits, row);
            self.run(core, limits, row, due_wait);
            if row.delegates.is_empty() {
                if let Some(since) = parked {
                    self.news(core, limits, row, since);
                }
            } else {
                for number in &row.delegates {
                    let Some(delegate) = core.tasks.task(*number) else { continue };
                    self.push(
                        row.number,
                        WaitKind::Delegate,
                        WaitObject::Task(*number),
                        core.wait_since(WaitKind::Delegate, delegate.created_at),
                        deadline(row),
                    );
                }
            }
        }
        self.records
    }

    fn task_entries(&mut self, core: &Core, limits: &Limits, row: &TaskRecord) {
        if let Escalation::Waiting { holder: at @ Holder::Task(_), revision, since, .. } = row.escalation {
            self.insert(entry(core, limits, row, EntryKind::Escalation { revision }, at, since, self.hears));
        }
        if let Some(proposal) = &row.proposal
            && let tasks::ProposalState::Pending { holder: tasks::ProposalHolder::Task(task), since } = proposal.state
        {
            let kind = EntryKind::Proposal { proposer: Proposer::Task(row.number), number: proposal.number };
            self.insert(entry(core, limits, row, kind, Holder::Task(task), since, self.hears));
        }
    }

    fn questions(&mut self, core: &Core, limits: &Limits, row: &TaskRecord) {
        let Party::Task(answerer) = row.requester else { return };
        for credit in &row.questions {
            if credit.asked != row.requester {
                continue;
            }
            // Restored and newly committed questions have their activity fact.
            let since = match core.question_activity.get(&(row.number, credit.line)) {
                Some(line) => line.at,
                None => row.phase_since,
            };
            let since = core.wait_since(WaitKind::Question, since);
            self.push(
                row.number,
                WaitKind::Question,
                WaitObject::Question { number: credit.number, answerer },
                since,
                clock(since, limits.waits.question, row.authority.budget.deadline, AtBound::PassUp),
            );
        }
    }

    fn dependencies(&mut self, core: &Core, row: &TaskRecord) {
        if let Phase::Active(_) | Phase::Held { was: tasks::Was::Active(_), .. } = row.phase {
            for number in &row.waiting_on {
                let mut since = row.created_at;
                for added in &row.added {
                    if added.task == *number {
                        since = added.at;
                        break;
                    }
                }
                let since = core.wait_since(WaitKind::Dependency, since);
                self.push(row.number, WaitKind::Dependency, WaitObject::Task(*number), since, deadline(row));
            }
        }
    }

    fn resource(&mut self, core: &Core, limits: &Limits, row: &TaskRecord) {
        if let Some(joined) = row.hold_wait_since
            && let Some((name, holder)) = core.tasks.resource_wait(row)
        {
            let since = core.wait_since(WaitKind::Resource, row.hold_bound_since.unwrap_or(joined));
            self.push(
                row.number,
                WaitKind::Resource,
                WaitObject::Resource { name, holder },
                since,
                clock(since, limits.tasks.hold_wait, None, AtBound::Hold),
            );
        }
    }

    fn run(&mut self, core: &Core, limits: &Limits, row: &TaskRecord, due_wait: Option<TaskRunWait>) {
        let wait = if let Phase::Active(tasks::Active::BackingOff { until }) = row.phase {
            // Backoff has a reset but need not have a source to name.
            TaskRunWait::Account {
                account: row.kept_account,
                source: None,
                why: TaskWaitWhy::Source(TaskSourceState::Later { until: Some(until) }),
            }
        } else if let Some(wait) = due_wait {
            wait
        } else {
            return;
        };
        let (kind, figure, reset) = match wait {
            TaskRunWait::Worker { until, .. } => (WaitKind::Host, limits.waits.host, Some(until)),
            TaskRunWait::Host
            | TaskRunWait::Refused(_)
            | TaskRunWait::Account { why: TaskWaitWhy::Host | TaskWaitWhy::Untooled { .. }, .. } => {
                (WaitKind::Host, limits.waits.host, None)
            }
            TaskRunWait::Account {
                why: TaskWaitWhy::Source(TaskSourceState::Later { until } | TaskSourceState::Limited { until }),
                ..
            } => (WaitKind::Account, limits.waits.account, until),
            TaskRunWait::Account { .. } | TaskRunWait::Credential => (WaitKind::Account, limits.waits.account, None),
        };
        let since = core.wait_since(kind, row.phase_since);
        let from = since.max(reset.unwrap_or(since));
        self.push(
            row.number,
            kind,
            WaitObject::Run(wait),
            since,
            clock(from, figure, row.authority.budget.deadline, AtBound::Hold),
        );
    }

    fn news(&mut self, core: &Core, limits: &Limits, row: &TaskRecord, since: Wall) {
        let since = core.wait_since(WaitKind::News, since);
        let mut subscriptions = List::with_capacity(4);
        let mut timer = None;
        for subscription in &row.subscriptions {
            if subscriptions.room() > 0 {
                subscriptions.push(subscription.number).expect("four subscription numbers");
            }
            match subscription.kind {
                tasks::SubscriptionKind::Timer { at, .. } => {
                    timer = Some(timer.unwrap_or(at).min(at));
                }
                tasks::SubscriptionKind::Task { .. } | tasks::SubscriptionKind::Topic { .. } => {}
            }
        }
        let bound = if row.authority.budget.deadline.is_none()
            && (row.flags.conversation || row.flags.ended_by == tasks::EndedBy::Standing)
        {
            Bound::None(Unbounded::Standing)
        } else {
            clock(since, limits.waits.news, row.authority.budget.deadline, AtBound::Hold)
        };
        self.push(row.number, WaitKind::News, WaitObject::News { subscriptions, timer }, since, bound);
    }
}

pub(super) fn party(core: &Core, limits: &Limits, row: &TaskRecord, fact: &Entry, hears: Option<Holder>) -> Wait {
    let at = match fact.whom {
        Whom::Person(person) => Holder::Person(person),
        Whom::Role { project, role } => Holder::Role { project, role },
    };
    entry(core, limits, row, fact.kind, at, fact.at, hears)
}

fn entry(
    core: &Core,
    limits: &Limits,
    row: &TaskRecord,
    kind: EntryKind,
    at: Holder,
    since: Wall,
    hears: Option<Holder>,
) -> Wait {
    let since = core.wait_since(WaitKind::Party, since);
    let (figure, deadline) = match kind {
        EntryKind::Question { .. } => (limits.waits.party, row.authority.budget.deadline),
        EntryKind::Escalation { .. } => (limits.tasks.escalation_stall, None),
        EntryKind::Proposal { .. } => (limits.tasks.proposal_stall, None),
        EntryKind::PersonTask => (row.person_wait.expect("answerable person interval").stall, None),
        EntryKind::Reply { .. } | EntryKind::ChatReply { .. } => unreachable!("a reply is not waiting for a party"),
    };
    let passes = match kind {
        EntryKind::Proposal { .. } => match at {
            Holder::Role { .. } => false,
            Holder::Task(task) => proposal_passes(core, limits, row, tasks::ProposalHolder::Task(task)),
            Holder::Person(person) => proposal_passes(core, limits, row, tasks::ProposalHolder::Person(person)),
        },
        EntryKind::PersonTask if core.escalation_fallback(row.project) == Some(at) => false,
        EntryKind::PersonTask if hears != Some(at) => hears.is_some(),
        EntryKind::PersonTask | EntryKind::Question { .. } | EntryKind::Escalation { .. } => {
            match wait_holder(core, limits, row, Some(at)) {
                Some(next) => next != at,
                None => false,
            }
        }
        EntryKind::Reply { .. } | EntryKind::ChatReply { .. } => unreachable!("a reply is not a wait"),
    };
    let then = if passes { AtBound::PassUp } else { AtBound::Overdue };
    Wait {
        task: row.number,
        kind: WaitKind::Party,
        object: WaitObject::Entry { kind, at },
        since,
        bound: clock(since, figure, deadline, then),
        hears,
    }
}

fn proposal_passes(core: &Core, limits: &Limits, row: &TaskRecord, holder: tasks::ProposalHolder) -> bool {
    let Some(proposal) = &row.proposal else { return false };
    core.proposal_holder(limits, row.number, &proposal.action, Some(holder), core.run_wall).is_some()
}

fn clock(since: Wall, figure: Duration, deadline: Option<Wall>, then: AtBound) -> Bound {
    let due = Wall::from_nanos(since.as_nanos().saturating_add(figure.as_nanos()));
    Bound::At { at: due.min(deadline.unwrap_or(due)), then }
}

fn deadline(row: &TaskRecord) -> Bound {
    match row.authority.budget.deadline {
        Some(at) => Bound::At { at, then: AtBound::Hold },
        None => Bound::None(Unbounded::NoDeadline),
    }
}
