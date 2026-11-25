//! Parked tasks and the wakes that can still move them (jig/docs/design/domain/tasks/stuck.md, 2–6).
use crate::domain::{activate, publish, record, task_mut};
use crate::{
    Active, Class, Domain, EndedBy, EscalationHolder, Executor, Limits, OpenQuestion, Party, Phase, ProposalState,
    Request, ResultsWake, SubscriptionKind, TaskRecord, Tries,
};
use alloc::boxed::Box;
use skein_lib::{Env, List, Map, Queue, Set};

/// A task reached by a stuck walk and its oldest question edge, if it has one.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct StuckLink {
    pub task: u64,
    pub question: Option<u64>,
}

#[derive(Hash, Debug)]
struct LeftAlone {
    order: u64,
    visited: Box<[u64]>,
}

#[derive(Hash, Debug)]
struct Exhausted {
    task: u64,
    question: OpenQuestion,
    tries: Tries,
    chain: Box<[StuckLink]>,
}

/// The reverse index contains only the bounded prefixes that left a task idle.
#[derive(Hash, Debug)]
pub(crate) struct Index {
    left: Map<u64, LeftAlone>,
    readers: Map<(u64, u64, u64), ()>, // (visited, order left alone, reader)
    dirty: Set<(u64, u64)>,
    persons: Set<u64>,
    order: u64,
    queued: bool,
    exhausted: Option<Exhausted>,
}

impl Index {
    pub(crate) fn new(limits: &Limits) -> Self {
        Self {
            left: Map::with_capacity(limits.tasks),
            readers: Map::uncapped(),
            dirty: Set::with_capacity(limits.tasks),
            persons: Set::with_capacity(limits.tasks),
            order: 0,
            queued: false,
            exhausted: None,
        }
    }

    fn next_order(&mut self) -> u64 {
        self.order = self.order.checked_add(1).expect("parked order fits");
        self.order
    }

    fn remove(&mut self, task: u64) -> Option<u64> {
        let left = self.left.remove(&task)?;
        self.dirty.remove(&(left.order, task));
        for visited in left.visited {
            assert!(self.readers.remove(&(visited, left.order, task)).is_some(), "walk indexed at every visit");
        }
        Some(left.order)
    }

    fn keep(&mut self, task: u64, walk: Walk) {
        let order = match self.remove(task) {
            Some(order) => order,
            None => self.next_order(),
        };
        for visited in &walk.visited {
            self.readers.insert((*visited, order, task), ()).expect("one index entry per visit");
        }
        self.left
            .insert(task, LeftAlone { order, visited: walk.visited.into_boxed() })
            .expect("one walk per live task");
    }

    fn changed(&mut self, task: u64, include_task: bool) {
        for (&(_, order, reader), ()) in self.readers.range((task, 0, 0)..=(task, u64::MAX, u64::MAX)) {
            if reader != task || include_task {
                self.dirty.insert((order, reader)).expect("one review per left-alone task");
            }
        }
    }
}

/// Person delegates are live party entries even if the requester doesn't wake
/// on delegate results. Keep their kind without reading an extra task in a walk.
pub(crate) fn index_executor(domain: &mut Domain, number: u64, executor: Executor) {
    match executor {
        Executor::Person(_) => {
            domain.stuck.persons.insert(number).expect("one place per live person task");
        }
        Executor::Agent { .. } | Executor::Procedure { .. } | Executor::Job { .. } => {
            domain.stuck.persons.remove(&number);
        }
    }
}

/// Publication compares only what a walk can read. Its own root is also indexed.
pub(crate) fn refresh(domain: &mut Domain, number: u64) {
    let id = *domain.names.get(&number).expect("published name");
    let task = domain.tasks.get(id).expect("published task");
    let row = &task.record;
    let executor = row.executor;
    let executor_changed = match task.published.as_deref() {
        Some(old) => old.executor != executor,
        None => true,
    };
    let (changed, idle_before) = match task.published.as_deref() {
        None => (true, false),
        Some(old) => (
            old.phase != row.phase
                || executor_changed
                || old.questions != row.questions
                || old.question_lines != row.question_lines
                || old.subscriptions != row.subscriptions
                || old.delegates != row.delegates
                || old.wake != row.wake
                || old.proposal != row.proposal
                || old.waiting_on.is_empty() != row.waiting_on.is_empty()
                || old.flags != row.flags
                || old.settings.task != row.settings.task
                || old.requester != row.requester
                || old.asked_requester != row.asked_requester,
            old.phase == Phase::Active(Active::Idle),
        ),
    };
    if changed {
        if domain.stuck.left.contains_key(&number) && oldest(domain, row).is_none() {
            domain.stuck.remove(number);
        }
        // A task that just parked already took its own judgment.
        domain.stuck.changed(number, idle_before);
    }
    if executor_changed {
        index_executor(domain, number, executor);
    }
}

pub(crate) fn ended(domain: &mut Domain, task: u64) {
    domain.stuck.remove(task);
    domain.stuck.persons.remove(&task);
    domain.stuck.changed(task, false);
}

/// Wait until the change's own notices and result have been routed, so an
/// ordinary wake wins over a stuck wake. An exhausted walk takes its hold first.
pub(crate) fn request_review(domain: &mut Domain, out: &mut Queue<Request>) {
    if !domain.stuck.queued && domain.stuck.exhausted.is_none() && !domain.stuck.dirty.is_empty() {
        domain.stuck.queued = true;
        out.push(Request::RecheckStuck);
    }
}

pub(crate) fn restored(domain: &mut Domain, limits: &Limits) {
    let mut parked = Set::with_capacity(limits.tasks);
    for (&number, _) in &domain.names {
        let row = record(domain, number).expect("restored name");
        if oldest(domain, row).is_some() {
            parked.insert((row.phase_since, number)).expect("one review per restored candidate");
        }
    }
    for &(_, task) in &parked {
        let order = domain.stuck.next_order();
        domain.stuck.dirty.insert((order, task)).expect("one review per restored candidate");
    }
}

pub(crate) fn review(domain: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    domain.stuck.queued = false;
    if domain.stuck.exhausted.is_some() {
        return;
    }
    for _ in 0..env.limits.tasks {
        let Some((_, task)) = domain.stuck.dirty.pop_first() else { break };
        let Some(row) = record(domain, task) else { continue };
        if park(domain, env, task, row.tries, out) {
            if record(domain, task).expect("reviewed task live").phase == Phase::Active(Active::Due) {
                publish(domain, env, task, out);
                activate(domain, task, out);
            }
            // A subsequent judgment sees this outcome and its routed messages.
            break;
        }
    }
}

fn oldest(domain: &Domain, row: &TaskRecord) -> Option<OpenQuestion> {
    if row.phase != Phase::Active(Active::Idle)
        || !row.wake.questions
        || !crate::words::keeping(domain, row)
        || !crate::run::ends::for_record(row).finishes_alone()
    {
        return None;
    }
    domain.oldest_open_question(row)
}

/// An idle park is judged before its changes can cause other walks to run.
/// Returns whether the question took the park. The caller publishes a due park.
pub(crate) fn park(domain: &mut Domain, env: &Env<Limits>, task: u64, tries: Tries, out: &mut Queue<Request>) -> bool {
    let row = record(domain, task).expect("park names live task");
    let Some(question) = oldest(domain, row) else {
        domain.stuck.remove(task);
        return false;
    };
    let walk = walk(domain, row, env.limits.tree_tasks);
    if walk.live {
        domain.stuck.keep(task, walk);
        return false;
    }
    if tries.of(Class::Run) < env.limits.retries.run.retries {
        let row = &mut task_mut(domain, task).expect("stuck task live").record;
        row.tries = tries;
        row.tries.add(Class::Run);
        row.phase = Phase::Active(Active::Due);
    } else {
        let chain = chain(domain, &walk, task, question.asker);
        assert!(domain.stuck.exhausted.is_none(), "one hold reason routed at a time");
        out.push(Request::QuestionParked { task, question, chain: chain.clone() });
        domain.stuck.exhausted = Some(Exhausted { task, question, tries, chain });
    }
    true
}

pub(crate) fn reasoned(
    domain: &mut Domain,
    env: &Env<Limits>,
    task: u64,
    question: u64,
    reason: Option<Box<[u8]>>,
    out: &mut Queue<Request>,
) {
    let Some(pending) = &domain.stuck.exhausted else { return };
    if pending.task != task || pending.question.number != question {
        return;
    }
    let pending = domain.stuck.exhausted.take().expect("answered hold reason");
    let Some(row) = record(domain, task) else { return };
    let Some(open) = oldest(domain, row) else { return };
    // A routed notice or decision may have changed the graph before the words arrived.
    let walk = walk(domain, row, env.limits.tree_tasks);
    if walk.live {
        domain.stuck.keep(task, walk);
        return;
    }
    let chain = chain(domain, &walk, task, open.asker);
    if open != pending.question || chain != pending.chain {
        out.push(Request::QuestionParked { task, question: open, chain: chain.clone() });
        domain.stuck.exhausted = Some(Exhausted { task, question: open, tries: pending.tries, chain });
        return;
    }
    task_mut(domain, task).expect("stuck task live").record.tries = pending.tries;
    crate::run::hold_stuck(domain, env, task, reason.expect("open credit retains its question activity"), out);
}

struct Walk {
    live: bool,
    seen: Set<u64>,
    visited: List<u64>,
}

impl Walk {
    // False means the next new vertex would exceed the bound, not that it is dead.
    fn add(&mut self, task: u64) -> bool {
        if self.seen.contains(&task) {
            return true;
        }
        if self.visited.push(task).is_err() {
            return false;
        }
        self.seen.insert(task).expect("one seen entry per visit");
        true
    }
}

fn walk(domain: &Domain, root: &TaskRecord, limit: u32) -> Walk {
    let mut walk = Walk { live: true, seen: Set::with_capacity(limit), visited: List::with_capacity(limit) };
    if !walk.add(root.number) || live_wake(domain, root, &mut walk) {
        return walk;
    }
    for at in 1..limit {
        let Some(&task) = walk.visited.get(at) else { break };
        let Some(row) = record(domain, task) else { continue };
        match row.phase {
            Phase::Ended(_) => continue,
            Phase::Waiting
            | Phase::Held { .. }
            | Phase::Closing(_)
            | Phase::Active(
                Active::Due
                | Active::Preparing
                | Active::Claimed { .. }
                | Active::Running { .. }
                | Active::BackingOff { .. },
            ) => return walk,
            Phase::Active(Active::Idle) => {}
        }
        if independent(row) {
            return walk;
        }
        let live = match row.executor {
            Executor::Job { .. } | Executor::Person(_) => true,
            Executor::Procedure { .. } => row.questions.is_empty() || question_wakes(row, &mut walk),
            Executor::Agent { .. } => live_wake(domain, row, &mut walk),
        };
        if live {
            return walk;
        }
    }
    walk.live = false;
    walk
}

fn independent(row: &TaskRecord) -> bool {
    // Dependencies have their own bounds. A question to its requester, on the
    // other hand, is an edge even when it also keeps the asker from claiming.
    row.flags.conversation
        || !row.waiting_on.is_empty()
        || match row.flags.ended_by {
            EndedBy::Requester | EndedBy::Standing => true,
            EndedBy::Result | EndedBy::Closer(_) => false,
        }
}

fn question_task(credit: &crate::QuestionCredit) -> Option<u64> {
    match &credit.offer {
        Some(offer) => match offer.holder {
            EscalationHolder::Task(task) => Some(task),
            EscalationHolder::Person(_) | EscalationHolder::Role { .. } => None,
        },
        None => match credit.answerer {
            Party::Task(task) => Some(task),
            Party::Person(_) | Party::Deployment { .. } => None,
        },
    }
}

fn question_wakes(row: &TaskRecord, walk: &mut Walk) -> bool {
    let mut ordered = Map::with_capacity(u32::try_from(row.questions.len()).expect("bounded credits"));
    for credit in &row.questions {
        let Some(task) = question_task(credit) else { return true };
        ordered.insert(credit.number, task).expect("one edge per credit");
    }
    for (_, task) in &ordered {
        if !walk.add(*task) {
            return true;
        }
    }
    false
}

fn live_wake(domain: &Domain, row: &TaskRecord, walk: &mut Walk) -> bool {
    if let Some(proposal) = &row.proposal {
        match proposal.state {
            ProposalState::Pending { .. } => return true,
            ProposalState::Accepted { .. } | ProposalState::Rejected { .. } | ProposalState::Withdrawn => {}
        }
    }
    if question_wakes(row, walk) {
        return true;
    }
    for subscription in &row.subscriptions {
        match subscription.kind {
            SubscriptionKind::Timer { .. } => return true,
            SubscriptionKind::Task { target, .. } => {
                if domain.names.contains_key(&target) && !walk.add(target) {
                    return true;
                }
            }
            SubscriptionKind::Topic { .. } => {}
        }
    }
    for child in &row.delegates {
        if domain.stuck.persons.contains(child) {
            return true;
        }
    }
    match row.wake.results {
        ResultsWake::Never => {}
        ResultsWake::Each => {
            let mut ordered = Set::with_capacity(u32::try_from(row.delegates.len()).expect("bounded delegates"));
            for &child in &row.delegates {
                ordered.insert(child).expect("one edge per delegate");
            }
            for &child in &ordered {
                if !walk.add(child) {
                    return true;
                }
            }
        }
        ResultsWake::LastOrFailure => {
            if let [child] = row.delegates.as_ref()
                && !walk.add(*child)
            {
                return true;
            }
        }
    }
    false
}

fn chain(domain: &Domain, walk: &Walk, root: u64, asker: u64) -> Box<[StuckLink]> {
    let mut links = List::with_capacity(walk.visited.len());
    for &task in &walk.visited {
        if task == root || task == asker {
            continue;
        }
        let mut question = None;
        if let Some(row) = record(domain, task) {
            for credit in &row.questions {
                if question_task(credit).is_some() {
                    question = Some(match question {
                        None => credit.number,
                        Some(first) => core::cmp::min(first, credit.number),
                    });
                }
            }
        }
        links.push(StuckLink { task, question }).expect("chain is a subset of the walk");
    }
    links.into_boxed()
}
