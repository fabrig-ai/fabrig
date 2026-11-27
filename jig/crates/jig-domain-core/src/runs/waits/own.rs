//! The facts and clocks of a task's own waits (jig/docs/design/domain/engine/waits.md, 3 and 5).

use crate::{AtBound, Bound, Core, Limits, Unbounded, Wait, WaitKind, WaitObject, admission::escalation::wait_holder};
use alloc::{boxed::Box, collections::BTreeMap};
use jig_domain_people::{DEPLOYMENT, Entry, EntryKind, Proposer, Role, Whom};
use jig_domain_tasks::{self as tasks, Escalation, EscalationHolder as Holder, Party, Phase, TaskRecord};
use skein_lib::{Duration, List, Wall};

use crate::documents::{TaskRunWait, TaskSourceState, TaskWaitWhy};

pub(super) struct Records {
    // The common single-record case needs no tree or sort key.
    only: Option<Wait>,
    // The last key preserves the inbox order of proposals projected to several roles.
    ordered: BTreeMap<(u8, Wall, u64, usize), Wait>,
}

impl Records {
    const fn new() -> Self {
        Self { only: None, ordered: BTreeMap::new() }
    }

    fn insert(&mut self, wait: Wait) {
        if self.ordered.is_empty() {
            match self.only.take() {
                Some(first) => self.insert_ordered(first),
                None => {
                    self.only = Some(wait);
                    return;
                }
            }
        }
        self.insert_ordered(wait);
    }

    pub(super) fn len(&self) -> usize {
        if self.only.is_some() { 1 } else { self.ordered.len() }
    }

    pub(super) fn first(&self) -> Option<&Wait> {
        match &self.only {
            Some(wait) => Some(wait),
            None => match self.ordered.first_key_value() {
                Some((_, wait)) => Some(wait),
                None => None,
            },
        }
    }

    pub(super) fn values(&self) -> impl Iterator<Item = &Wait> + '_ {
        self.only.iter().chain(self.ordered.values())
    }

    pub(super) fn into_values(self) -> impl Iterator<Item = Wait> {
        self.only.into_iter().chain(self.ordered.into_values())
    }

    pub(super) fn into_boxed(self) -> Box<[Wait]> {
        if let Some(only) = self.only {
            return Box::new([only]);
        }
        let mut waits = List::allocated(u32::try_from(self.ordered.len()).expect("bounded waits"));
        for wait in self.ordered.into_values() {
            waits.push(wait).expect("one slot per wait");
        }
        waits.into_boxed()
    }

    fn address(&mut self, hears: Option<Holder>) {
        if let Some(wait) = &mut self.only {
            wait.hears = hears;
        }
        for wait in self.ordered.values_mut() {
            wait.hears = hears;
        }
    }

    fn insert_ordered(&mut self, wait: Wait) {
        let key = (group(wait.kind), wait.since, object_number(&wait), self.ordered.len());
        assert!(self.ordered.insert(key, wait).is_none(), "distinct wait position");
    }
}

pub(super) struct Own {
    records: Records,
    hears: Hears,
}

// A row with no records never needs its escalation route.
enum Hears {
    Unread,
    Read(Option<Holder>),
}

impl Own {
    pub(super) const fn new() -> Self {
        Self { records: Records::new(), hears: Hears::Unread }
    }

    pub(super) const fn unaddressed() -> Self {
        Self { records: Records::new(), hears: Hears::Read(None) }
    }

    pub(super) fn hears(&mut self, core: &Core, limits: &Limits, row: &TaskRecord) -> Option<Holder> {
        if let Hears::Read(hears) = self.hears {
            return hears;
        }
        let hears = wait_holder(core, limits, row, None);
        self.hears = Hears::Read(hears);
        hears
    }

    pub(super) fn party(&mut self, core: &Core, limits: &Limits, row: &TaskRecord, fact: &Entry) {
        let hears = self.hears(core, limits, row);
        self.records.insert(party(core, limits, row, fact, hears));
    }

    pub(super) fn into_records(self) -> Records {
        self.records
    }

    fn push(&mut self, task: u64, kind: WaitKind, object: WaitObject, since: Wall, bound: Bound) {
        self.records.insert(Wait { task, kind, object, since, bound: not_before(bound, since), hears: None });
    }

    pub(super) fn finish(
        &mut self,
        core: &Core,
        limits: &Limits,
        row: &TaskRecord,
        working: bool,
        parked: Option<Wall>,
        due_wait: Option<TaskRunWait>,
    ) {
        match row.phase {
            Phase::Closing(_) | Phase::Ended(_) => return,
            Phase::Active(_) | Phase::Waiting | Phase::Held { .. } => {}
        }
        self.task_entries(core, limits, row);
        self.questions(core, limits, row);
        self.dependencies(core, row);
        if !working {
            self.resource(core, limits, row);
            self.run(core, limits, row, due_wait);
            if row.delegates.is_empty() {
                if let Some(since) = parked {
                    if let Some(report) = core.reported_waits.get(&row.number) {
                        for wait in report {
                            let since = core.wait_since(wait.kind, wait.since);
                            let bound = super::reports::bound(limits, tasks::deadline(row, core.run_wall), wait, since);
                            self.push(row.number, wait.kind, wait.object.clone(), since, bound);
                        }
                    } else {
                        self.news(core, limits, row, since);
                    }
                }
            } else {
                for number in &row.delegates {
                    let Some(delegate) = core.tasks.task(*number) else { continue };
                    self.push(
                        row.number,
                        WaitKind::Delegate,
                        WaitObject::Task(*number),
                        core.wait_since(WaitKind::Delegate, after_release(row, delegate.created_at)),
                        deadline(core, row),
                    );
                }
            }
        }
        if self.records.len() != 0
            && let Some(hears) = self.hears(core, limits, row)
        {
            self.records.address(Some(hears));
        }
    }

    fn task_entries(&mut self, core: &Core, limits: &Limits, row: &TaskRecord) {
        if let Escalation::Waiting { holder: at @ Holder::Task(_), revision, since, .. } = row.escalation {
            let hears = self.hears(core, limits, row);
            self.records.insert(entry(core, limits, row, EntryKind::Escalation { revision }, at, since, hears));
        }
        if let Some(proposal) = &row.proposal
            && let tasks::ProposalState::Pending { holder: tasks::ProposalHolder::Task(task), since } = proposal.state
        {
            let kind = EntryKind::Proposal { proposer: Proposer::Task(row.number), number: proposal.number };
            let hears = self.hears(core, limits, row);
            self.records.insert(entry(core, limits, row, kind, Holder::Task(task), since, hears));
        }
    }

    fn questions(&mut self, core: &Core, limits: &Limits, row: &TaskRecord) {
        for credit in &row.questions {
            let Some((holder, at)) = question_place(core, row, credit) else { continue };
            let Holder::Task(answerer) = holder else { continue };
            if credit.offer.is_none() && credit.asked != row.requester {
                continue;
            }
            let since = core.wait_since(WaitKind::Question, at);
            self.push(
                row.number,
                WaitKind::Question,
                WaitObject::Question { number: credit.number, answerer },
                since,
                clock(
                    since,
                    limits.waits.question,
                    tasks::deadline(row, core.run_wall),
                    question_bound(core, limits, row, holder),
                ),
            );
        }
    }

    fn dependencies(&mut self, core: &Core, row: &TaskRecord) {
        if let Phase::Waiting | Phase::Active(_) | Phase::Held { was: tasks::Was::Active(_), .. } = row.phase {
            for number in &row.waiting_on {
                let mut since = row.created_at;
                for added in &row.added {
                    if added.task == *number {
                        since = added.at;
                        break;
                    }
                }
                let since = core.wait_since(WaitKind::Dependency, after_release(row, since));
                self.push(row.number, WaitKind::Dependency, WaitObject::Task(*number), since, deadline(core, row));
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
        let (kind, figure, reset) = run_clock(limits.waits, wait);
        let since = core.wait_since(kind, row.phase_since);
        let from = since.max(reset.unwrap_or(since));
        self.push(
            row.number,
            kind,
            WaitObject::Run(wait),
            since,
            clock(from, figure, tasks::deadline(row, core.run_wall), AtBound::Hold),
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
        let bound = if tasks::deadline(row, core.run_wall).is_none()
            && (row.flags.conversation || row.flags.ended_by == tasks::EndedBy::Standing)
        {
            Bound::None(Unbounded::Standing)
        } else {
            clock(since, limits.waits.news, tasks::deadline(row, core.run_wall), AtBound::Hold)
        };
        self.push(row.number, WaitKind::News, WaitObject::News { subscriptions, timer }, since, bound);
    }
}

// Release starts a new task wait. The original additions and delegate
// creations are still the facts for waits that haven't been released.
fn after_release(row: &TaskRecord, since: Wall) -> Wall {
    if (row.due_reason == tasks::Why::Released || row.due_reason == tasks::Why::Measured)
        && let Phase::Waiting | Phase::Active(tasks::Active::Idle) = row.phase
    {
        since.max(row.phase_since)
    } else {
        since
    }
}

pub(super) fn run_clock(figures: crate::Figures, wait: TaskRunWait) -> (WaitKind, Duration, Option<Wall>) {
    match wait {
        TaskRunWait::Worker { until, .. } => (WaitKind::Host, figures.host, Some(until)),
        TaskRunWait::Host
        | TaskRunWait::Refused(_)
        | TaskRunWait::Account { why: TaskWaitWhy::Host | TaskWaitWhy::Untooled { .. }, .. } => {
            (WaitKind::Host, figures.host, None)
        }
        TaskRunWait::Account {
            why: TaskWaitWhy::Source(TaskSourceState::Later { until } | TaskSourceState::Limited { until }),
            ..
        } => (WaitKind::Account, figures.account, until),
        TaskRunWait::Account { .. } | TaskRunWait::Credential => (WaitKind::Account, figures.account, None),
    }
}

fn group(kind: WaitKind) -> u8 {
    match kind {
        WaitKind::Party => 0,
        WaitKind::Question => 1,
        WaitKind::Dependency => 2,
        WaitKind::Resource => 3,
        WaitKind::Account | WaitKind::Host => 4,
        WaitKind::Delegate => 5,
        WaitKind::News | WaitKind::Step | WaitKind::Room | WaitKind::Turn | WaitKind::SignOff => 6,
    }
}

fn object_number(wait: &Wait) -> u64 {
    match wait.object {
        WaitObject::Entry { kind, .. } => match kind {
            EntryKind::Question { message } | EntryKind::Reply { message } | EntryKind::ChatReply { message } => {
                message
            }
            EntryKind::Proposal { number, .. } => number,
            EntryKind::Escalation { revision } => revision,
            EntryKind::PersonTask => wait.task,
        },
        WaitObject::Question { number, .. } | WaitObject::Task(number) => number,
        WaitObject::Resource { .. } | WaitObject::Run(_) | WaitObject::News { .. } => 0,
        WaitObject::Procedure { ref things, .. } => match things.get(0) {
            Some(thing) => thing.number,
            None => 0,
        },
    }
}

/// The owner can move a policy proposal that has no deciding role in the
/// task's inbox projection (jig/docs/design/domain/engine/waits.md, 3).
pub(super) fn policy_entry(row: &TaskRecord, inbox: &[Entry]) -> Option<Entry> {
    if let Phase::Closing(_) | Phase::Ended(_) = row.phase {
        return None;
    }
    let proposal = row.proposal.as_ref()?;
    let tasks::ProposalState::Pending { holder: tasks::ProposalHolder::Policy { project, kind }, since } =
        proposal.state
    else {
        return None;
    };
    for entry in inbox {
        if let EntryKind::Proposal { .. } = entry.kind {
            return None;
        }
    }
    let project = if kind == tasks::ProposalKind::Settings { DEPLOYMENT } else { project };
    Some(Entry {
        task: row.number,
        project: row.project,
        whom: Whom::Role { project, role: Role::Owner.number() },
        kind: EntryKind::Proposal { proposer: Proposer::Task(row.number), number: proposal.number },
        at: since,
    })
}

fn question_place(core: &Core, row: &TaskRecord, credit: &tasks::QuestionCredit) -> Option<(Holder, Wall)> {
    if let Some(offer) = &credit.offer {
        return Some((offer.holder, offer.since));
    }
    let holder = match credit.answerer {
        Party::Task(task) => Holder::Task(task),
        Party::Person(person) => Holder::Person(person),
        Party::Deployment { .. } => unreachable!("questions wait at a task or person"),
    };
    let question = tasks::OpenQuestion { asker: row.number, number: credit.number, line: credit.line };
    let since = match core.open_question_fact(question) {
        Some((at, _)) => at,
        None => match holder {
            Holder::Task(_) => row.phase_since,
            Holder::Person(_) | Holder::Role { .. } => return None,
        },
    };
    Some((holder, since))
}

fn question_bound(core: &Core, limits: &Limits, row: &TaskRecord, holder: Holder) -> AtBound {
    if crate::admission::escalation::next_question_holder(core, limits, row, holder).is_some() {
        AtBound::PassUp
    } else {
        AtBound::Overdue
    }
}

/// Original and offered inbox entries are two ways to answer, not two waits.
/// Put every party-held question's wait on its credit owner, even when its
/// first inbox link is the task the question was asked of.
pub(super) fn party_entries(core: &Core, limits: &Limits, row: &TaskRecord) -> List<Entry> {
    let inbox = core.waiting_entries(limits, row);
    let mut entries = List::uncapped();
    for fact in inbox.iter().chain(policy_entry(row, &inbox).iter()) {
        if let EntryKind::Question { message } = fact.kind
            && core.tasks.question_ticket(row.number, message).is_some()
        {
            continue;
        }
        if let Some(fact) = party_fact(row, *fact) {
            entries.push(fact).expect("bounded party facts");
        }
    }
    if crate::inbox::questions_live(&row.phase) {
        for credit in &row.questions {
            if let Some(entry) = party_question(core, row, credit) {
                entries.push(entry).expect("one fact per open credit");
            }
        }
    }
    entries
}

fn party_question(core: &Core, row: &TaskRecord, credit: &tasks::QuestionCredit) -> Option<Entry> {
    let (holder, at) = question_place(core, row, credit)?;
    crate::inbox::question_entry(row, credit.number, holder, at)
}

/// The second inbox entry is another way to answer, not another wait.
fn party_fact(row: &TaskRecord, fact: Entry) -> Option<Entry> {
    match fact.kind {
        EntryKind::PersonTask => {
            let current = person_place(row, fact)?;
            if row.taken_by.is_none() && fact.whom != current.whom { None } else { Some(current) }
        }
        EntryKind::Question { .. } | EntryKind::Escalation { .. } | EntryKind::Proposal { .. } => Some(fact),
        EntryKind::Reply { .. } | EntryKind::ChatReply { .. } => None,
    }
}

fn person_place(row: &TaskRecord, mut fact: Entry) -> Option<Entry> {
    if let Some(offer) = row.person_wait?.offer {
        fact.whom = crate::inbox::person_whom(row.project, offer.holder);
        fact.at = offer.since;
    }
    Some(fact)
}

pub(super) fn inbox_wait(core: &Core, limits: &Limits, row: &TaskRecord, fact: Entry) -> Option<Wait> {
    match fact.kind {
        EntryKind::Question { message } => {
            let asker = match question_asker(core, row, message) {
                Some(asker) => core.tasks.task(asker).unwrap_or(row),
                None => row,
            };
            for credit in &asker.questions {
                if credit.number == message {
                    let current = party_question(core, asker, credit)?;
                    return Some(party(core, limits, asker, &current, None));
                }
            }
            Some(party(core, limits, row, &fact, None))
        }
        EntryKind::PersonTask => Some(party(core, limits, row, &person_place(row, fact)?, None)),
        EntryKind::Escalation { .. } | EntryKind::Proposal { .. } => Some(party(core, limits, row, &fact, None)),
        EntryKind::Reply { .. } | EntryKind::ChatReply { .. } => None,
    }
}

// The first inbox entry links to the asked task. Its question line names the
// credit owner; the offered entry links straight to the credit owner instead.
fn question_asker(core: &Core, row: &TaskRecord, message: u64) -> Option<u64> {
    for position in &row.question_lines {
        if position.message == message {
            let line = core.question_activity.get(&(row.number, position.line))?;
            if let tasks::Line::Words { by: Party::Task(asker), .. } = line.line {
                return Some(asker);
            }
        }
    }
    None
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
    let passes = match kind {
        EntryKind::Proposal { .. } => match at {
            Holder::Role { .. } => false,
            Holder::Task(task) => proposal_passes(core, limits, row, tasks::ProposalHolder::Task(task)),
            Holder::Person(person) => proposal_passes(core, limits, row, tasks::ProposalHolder::Person(person)),
        },
        EntryKind::PersonTask => crate::admission::escalation::next_person_holder(core, limits, row).is_some(),
        EntryKind::Question { .. } => question_bound(core, limits, row, at) == AtBound::PassUp,
        EntryKind::Escalation { .. } => match wait_holder(core, limits, row, Some(at)) {
            Some(next) => next != at,
            None => false,
        },
        EntryKind::Reply { .. } | EntryKind::ChatReply { .. } => unreachable!("a reply is not a wait"),
    };
    let then = if passes { AtBound::PassUp } else { AtBound::Overdue };
    let (since, bound) = match kind {
        EntryKind::Question { .. } => {
            party_clock(core, since, limits.waits.party, tasks::deadline(row, core.run_wall), then)
        }
        EntryKind::Escalation { .. } => party_clock(core, since, limits.tasks.escalation_stall, None, then),
        EntryKind::Proposal { .. } => proposal_clock(core, limits, since, then),
        EntryKind::PersonTask => {
            party_clock(core, since, row.person_wait.expect("answerable person interval").stall, None, then)
        }
        EntryKind::Reply { .. } | EntryKind::ChatReply { .. } => unreachable!("a reply is not a wait"),
    };
    Wait { task: row.number, kind: WaitKind::Party, object: WaitObject::Entry { kind, at }, since, bound, hears }
}

pub(super) fn proposal_clock(core: &Core, limits: &Limits, since: Wall, then: AtBound) -> (Wall, Bound) {
    party_clock(core, since, limits.tasks.proposal_stall, None, then)
}

fn party_clock(core: &Core, since: Wall, figure: Duration, deadline: Option<Wall>, then: AtBound) -> (Wall, Bound) {
    let since = core.wait_since(WaitKind::Party, since);
    (since, not_before(clock(since, figure, deadline, then), since))
}

fn proposal_passes(core: &Core, limits: &Limits, row: &TaskRecord, holder: tasks::ProposalHolder) -> bool {
    let Some(proposal) = &row.proposal else { return false };
    core.proposal_holder(limits, row.number, &proposal.action, Some(holder), core.run_wall).is_some()
}

// A deadline already passed when the wait begins is due immediately. Keep the
// episode's since intact, including any upgrade floor.
fn not_before(bound: Bound, since: Wall) -> Bound {
    match bound {
        Bound::At { at, then } => Bound::At { at: at.max(since), then },
        Bound::None(why) => Bound::None(why),
    }
}

pub(super) fn clock(since: Wall, figure: Duration, deadline: Option<Wall>, then: AtBound) -> Bound {
    let due = figure_at(since, figure);
    Bound::At { at: due.min(deadline.unwrap_or(due)), then }
}

fn deadline(core: &Core, row: &TaskRecord) -> Bound {
    match tasks::deadline(row, core.run_wall) {
        Some(at) => Bound::At { at, then: AtBound::Hold },
        None => Bound::None(Unbounded::NoDeadline),
    }
}

pub(super) fn figure_at(since: Wall, figure: Duration) -> Wall {
    Wall::from_nanos(since.as_nanos().saturating_add(figure.as_nanos()))
}
