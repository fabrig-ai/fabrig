//! Fresh, pinned reads for write intents and completed attempts (writes.md, sections 3–4 and 7).
use super::{
    Domain, Name, Operation, Queue, Request, Sender, Tip, WriteState, advance, after, before, filtered, save_changed,
};
use crate::Limits;
use skein_forge::api::{Answer, Commit, Error, Read, Repository};
use skein_forge_client::{Event, What};
use skein_lib::{Env, List, Map, Time, Token};

pub(super) mod previous;
use previous::WriterRead;

#[derive(Hash, Debug)]
pub(crate) struct State {
    pending: Map<(Name, u64), Job>,
    flight: Option<Flight>,
    checked: Map<(Name, u64), Checked>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Check {
    Branch { lost: bool },
    Pull { observed: Option<Commit> },
    Contains { tip: Commit, contains: Commit, then: Then },
    Order { head: Commit, observed: Commit },
    PreviousCompare { head: Commit, waiting: Commit },
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Then {
    Push { lost: bool },
    Writer { head: Option<Commit>, next: Option<Commit>, mode: WriterRead },
    Update { base: Commit, rewrite: bool },
    UpdateBase,
}

#[derive(Clone, Copy, Hash, Debug)]
struct Job {
    check: Check,
    at: Time,
}

#[derive(Hash, Debug)]
struct Flight {
    owner: Token,
    key: (Name, u64),
    check: Check,
    live: bool,
}

#[derive(Clone, Copy, Hash, Debug)]
struct Checked {
    head: Option<Commit>,
    observed: Commit,
}

impl State {
    pub(crate) fn new() -> Self {
        Self { pending: Map::uncapped(), flight: None, checked: Map::uncapped() }
    }

    pub(super) fn owed(&self, name: &Name, order: u64) -> bool {
        if let Some(flight) = &self.flight
            && flight.live
            && &flight.key.0 == name
            && flight.key.1 == order
        {
            return true;
        }
        self.pending.contains_key(&(name.clone(), order))
    }

    pub(crate) fn deadline(&self) -> Option<Time> {
        if self.flight.is_some() {
            return None;
        }
        let mut earliest = None;
        for (_, job) in &self.pending {
            earliest = Some(match earliest {
                Some(at) => job.at.min(at),
                None => job.at,
            });
        }
        earliest
    }
}

pub(super) fn checked(d: &Domain, name: &Name, order: u64, head: Option<Commit>, observed: Commit) -> bool {
    if let Some(result) = d.write_reads.checked.get(&(name.clone(), order)) {
        result.head == head && result.observed == observed
    } else {
        false
    }
}

pub(crate) fn forget(d: &mut Domain, name: &Name, order: u64) {
    let key = (name.clone(), order);
    d.write_reads.pending.remove(&key);
    d.write_reads.checked.remove(&key);
    if let Some(flight) = &mut d.write_reads.flight
        && flight.key == key
    {
        flight.live = false;
    }
}

fn schedule(d: &mut Domain, name: &Name, order: u64, check: Check) {
    let key = (name.clone(), order);
    if let Some(flight) = &d.write_reads.flight
        && flight.live
        && flight.key == key
        && flight.check == check
    {
        return;
    }
    if !d.write_reads.pending.contains_key(&key) {
        d.write_reads.pending.insert(key, Job { check, at: Time::from_nanos(0) }).expect("one check per write");
    }
}

pub(super) fn order(d: &mut Domain, name: &Name, order: u64, head: Commit, observed: Commit) {
    schedule(d, name, order, Check::Order { head, observed });
}

#[expect(clippy::manual_find, reason = "step code searches by order without a closure")]
fn entry<'a>(d: &'a Domain, name: &Name, order: u64) -> Option<&'a super::Written> {
    let row = d.writes.get(name)?;
    for entry in &row.entries {
        if entry.order == order {
            return Some(entry);
        }
    }
    None
}

pub(super) fn preparing_rebase(d: &Domain, name: &Name, order: u64) -> bool {
    let Some(written) = entry(d, name, order) else { return false };
    if let (Operation::Update { .. }, Sender::Entry { task, entry }) = (&written.operation, written.sender)
        && let Some((pending, _, _)) = super::super::rebase::pending(d, task)
    {
        return entry == pending;
    }
    false
}

pub(super) fn intent(d: &mut Domain, name: &Name, order: u64, observed: Commit, actor: super::Actor) {
    let Some(entry) = entry(d, name, order) else { return };
    let check = match entry.operation {
        Operation::Update { before, base, .. } => {
            let rewrite = actor == super::Actor::Writer && preparing_rebase(d, name, order);
            Check::Contains { tip: observed, contains: before, then: Then::Update { base, rewrite } }
        }
        Operation::MergedInto { .. } => Check::Pull { observed: Some(observed) },
        Operation::Writer { before } => {
            let head = super::known_head(d, name);
            let Some(check) = previous::writer_proof(before, head, observed, WriterRead::Observed) else { return };
            check
        }
        Operation::Compare { .. }
        | Operation::Push { .. }
        | Operation::Create { .. }
        | Operation::Delete { .. }
        | Operation::Merge { .. }
        | Operation::Open { .. }
        | Operation::Close { .. }
        | Operation::Reopen { .. }
        | Operation::Retarget { .. } => Check::Branch { lost: true },
    };
    schedule(d, name, order, check);
}

pub(super) fn push_read(d: &mut Domain, name: &Name, order: u64, tip: Option<Commit>, commit: Commit) {
    let check = match tip {
        Some(tip) => Check::Contains { tip, contains: commit, then: Then::Push { lost: true } },
        None => Check::Branch { lost: true },
    };
    schedule(d, name, order, check);
}

pub(super) fn finished(d: &mut Domain, name: &Name, order: u64, lost: bool) {
    if let Some(written) = entry(d, name, order)
        && let Operation::Writer { .. } = written.operation
    {
        // An observation that began while the attempt was live cannot be
        // its final readback, even if the answer is still out.
        forget(d, name, order);
    }
    schedule(d, name, order, Check::Branch { lost });
}

pub(crate) fn restart(d: &mut Domain, out: &mut Queue<Request>) {
    let mut needed = List::uncapped();
    let mut comparisons = List::uncapped();
    for (name, row) in &d.writes {
        for entry in &row.entries {
            if entry.state != WriteState::Uncertain {
                continue;
            }
            let check = match entry.operation {
                Operation::Merge { .. }
                | Operation::MergedInto { .. }
                | Operation::Open { .. }
                | Operation::Close { .. }
                | Operation::Reopen { .. }
                | Operation::Retarget { .. } => Check::Pull { observed: None },
                Operation::Push { .. }
                | Operation::Create { .. }
                | Operation::Delete { .. }
                | Operation::Update { .. }
                | Operation::Writer { .. } => Check::Branch { lost: true },
                Operation::Compare { after } => {
                    comparisons.push((name.clone(), entry.order, after)).expect("uncapped comparison queue");
                    continue;
                }
            };
            needed.push((name.clone(), entry.order, check)).expect("uncapped readback queue");
        }
    }
    for (name, order, waiting) in comparisons.into_boxed() {
        previous::compare(d, &name, order, waiting, out);
    }
    for (name, order, check) in needed.into_boxed() {
        schedule(d, &name, order, check);
    }
}

fn read(d: &Domain, l: &Limits, name: &Name, order: u64, check: Check) -> Option<Read> {
    let entry = entry(d, name, order)?;
    let resource = super::super::to_client(name, l)?;
    let What::Branch(branch) = resource.what else { return None };
    if let Check::Order { head, observed } = check {
        return Some(Read::Compare { before: head, after: observed });
    }
    if !entry.state.unsettled() {
        return None;
    }
    match check {
        Check::PreviousCompare { head, waiting } => Some(Read::Commits { before: head, after: waiting, page: 1 }),
        Check::Branch { .. } => Some(Read::Branch { branch }),
        Check::Pull { .. } => {
            if let Operation::Open { pull: None, base, .. } = &entry.operation {
                return Some(Read::PullFor { head: branch, base: base.clone() });
            }
            Some(Read::Pull { number: super::pull_number(&entry.operation)?, ci: false })
        }
        Check::Contains { tip, contains, .. } => Some(Read::Commits { before: tip, after: contains, page: 1 }),
        Check::Order { .. } => unreachable!("the comparison was handled above"),
    }
}

pub(crate) fn drive(d: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    if d.write_reads.flight.is_some() || !super::super::fresh_reads_in(d) || d.stopping {
        return;
    }
    let mut due = None;
    for (key, job) in &d.write_reads.pending {
        if job.at <= env.now {
            due = Some(key.clone());
            break;
        }
    }
    let Some(key) = due else { return };
    let job = d.write_reads.pending.remove(&key).expect("job selected");
    let repository = Repository { forge: key.0.forge, repository: key.0.repository };
    if !d.repositories.contains_key(&repository) || d.forge_ended(repository.forge) {
        return;
    }
    let Some(read) = read(d, &env.limits, &key.0, key.1, job.check) else { return };
    let Some(sequence) = d.sequence.checked_add(1) else { return };
    d.sequence = sequence;
    let owner = Token::new(sequence | (1_u64 << 63));
    d.write_reads.flight = Some(Flight { owner, key, check: job.check, live: true });
    super::super::child(d, env, Event::Read { owner, repository, read }, out);
}

pub(crate) fn reading(d: &Domain, owner: Token) -> bool {
    if let Some(flight) = &d.write_reads.flight { flight.owner == owner } else { false }
}

pub(crate) fn answered(d: &mut Domain, env: &Env<Limits>, result: Result<Answer, Error>, out: &mut Queue<Request>) {
    let Flight { key, check, live, .. } = d.write_reads.flight.take().expect("write read in flight");
    if !live {
        return;
    }
    let Some(entry) = entry(d, &key.0, key.1) else { return };
    let lost_push = if let Sender::Attempt { task, attempt } = entry.sender
        && let Operation::Push { after, .. } = &entry.operation
        && is_lost_push_read(check)
    {
        Some((task, attempt, *after))
    } else {
        None
    };
    let found = match check {
        Check::Order { head, observed } => {
            if let Ok(Answer::Compare { before, after, behind, .. }) = result
                && before == head
                && after == observed
            {
                Found::Ordered { head, observed, contains: behind == 0 }
            } else {
                Found::Retry
            }
        }
        Check::PreviousCompare { head, waiting } => previous::compared(head, waiting, result),
        Check::Branch { lost } if entry.state.unsettled() => found_branch(d, &key.0, &entry.operation, lost, result),
        Check::Pull { observed } if entry.state.unsettled() => {
            if let Ok(Answer::Pull(pull)) = result {
                if let Operation::MergedInto { .. } = entry.operation {
                    if crate::items::branch_matches(&key.0, &pull.base)
                        && let Some(commit) = pull.merged
                        && observed.unwrap_or(commit) == commit
                    {
                        Found::Confirm { after: Some(commit), pull: None }
                    } else {
                        Found::NoProof { observed }
                    }
                } else if super::matches_pull(&entry.operation, &pull)
                    && crate::items::branch_matches(&key.0, &pull.head)
                {
                    Found::Confirm { after: None, pull: Some(pull.number) }
                } else {
                    Found::NoProof { observed }
                }
            } else {
                Found::Retry
            }
        }
        Check::Contains { tip, then, .. } if entry.state.unsettled() => {
            use temper_engine_domain_forge_change::Status;
            match super::super::ancestry_status(result) {
                Status::Passed => match then {
                    Then::Push { .. } => Found::Confirm { after: None, pull: None },
                    Then::Writer { head, next, mode } => previous::writer_proved(head, next, tip, mode),
                    Then::UpdateBase => Found::Confirm { after: Some(tip), pull: None },
                    Then::Update { base, .. } => {
                        Found::Check(Check::Contains { tip, contains: base, then: Then::UpdateBase })
                    }
                },
                Status::Failed => match then {
                    Then::Push { lost } => branch_after_failed_push(&entry.operation, lost, Some(tip)),
                    Then::Update { base, rewrite: true } => {
                        Found::Check(Check::Contains { tip, contains: base, then: Then::UpdateBase })
                    }
                    Then::Writer { mode: WriterRead::Answered, .. } => Found::Refuse,
                    Then::Writer { .. } | Then::Update { rewrite: false, .. } | Then::UpdateBase => {
                        Found::NoProof { observed: Some(tip) }
                    }
                },
                Status::Unknown | Status::Pending => Found::Retry,
            }
        }
        Check::Branch { .. } | Check::Pull { .. } | Check::Contains { .. } => return,
    };
    let name = key.0.clone();
    let writer = if let Operation::Writer { .. } = entry.operation { Some(entry.sender) } else { None };
    apply(d, env, key, check, found, out);
    if let Some(sender) = writer {
        previous::end_rereading(d, env, &name, sender, check, found, out);
    }
    if let Found::Confirm { .. } = found
        && let Some((task, attempt, commit)) = lost_push
        && super::known_head(d, &name) == Some(commit)
        && pin_lost_delegate(d, &name, task, attempt)
    {
        super::kept::sync_heads(d, env, out);
        super::super::record_delegate_head(d, env, task, &name, commit, false, out);
    }
    if let Found::Confirm { .. }
    | Found::Refuse
    | Found::Ordered { .. }
    | Found::NoProof { .. }
    | Found::Compared { .. }
    | Found::WriterProgress { .. } = found
    {
        super::kept::rejudge(d, env, &name, out);
    }
}

fn is_lost_push_read(check: Check) -> bool {
    match check {
        Check::Branch { lost } | Check::Contains { then: Then::Push { lost }, .. } => lost,
        Check::Pull { .. }
        | Check::Contains { then: Then::Writer { .. } | Then::Update { .. } | Then::UpdateBase, .. }
        | Check::Order { .. }
        | Check::PreviousCompare { .. } => false,
    }
}

fn pin_lost_delegate(d: &Domain, name: &Name, task: u64, attempt: u64) -> bool {
    use crate::Writer;
    let Some(hold) = d.holds.get(name) else { return false };
    if hold.drift.is_some() {
        return false;
    }
    match hold.writer {
        None => true,
        Some(Writer::Lost { task: holder, attempt: run }) => holder == task && run == attempt,
        Some(Writer::Run { task: holder, attempt: run }) => holder == task && run > attempt,
        Some(Writer::Entry(_)) => false,
    }
}

#[derive(Clone, Copy)]
enum Found {
    Confirm { after: Option<Commit>, pull: Option<u64> },
    Refuse,
    Ordered { head: Commit, observed: Commit, contains: bool },
    Compared { head: Commit, waiting: Commit, older: bool },
    WriterProgress { head: Option<Commit>, tip: Commit, mode: WriterRead },
    NoProof { observed: Option<Commit> },
    Check(Check),
    Retry,
}

fn branch_tip(result: Result<Answer, Error>) -> Result<Option<Commit>, ()> {
    if let Ok(Answer::Commit(tip)) = result {
        return Ok(Some(tip));
    }
    if result == Err(Error::Missing) {
        return Ok(None);
    }
    Err(())
}

fn branch_after_failed_push(op: &Operation, lost: bool, tip: Option<Commit>) -> Found {
    if !lost || before(op) != Tip::Known(tip) { Found::Refuse } else { Found::NoProof { observed: tip } }
}

fn found_branch(d: &Domain, name: &Name, op: &Operation, lost: bool, result: Result<Answer, Error>) -> Found {
    let Ok(tip) = branch_tip(result) else { return Found::Retry };
    if let Operation::Writer { before } = op {
        return previous::writer_read(d, name, *before, tip, lost);
    }
    branch_answer(op, lost, tip)
}

fn branch_answer(op: &Operation, lost: bool, tip: Option<Commit>) -> Found {
    if after(op) == Tip::Known(tip) {
        return Found::Confirm { after: None, pull: None };
    }
    if let Some(tip) = tip {
        match op {
            Operation::Push { after, .. } => {
                return Found::Check(Check::Contains { tip, contains: *after, then: Then::Push { lost } });
            }
            Operation::Update { before, base, .. } => {
                return Found::Check(Check::Contains {
                    tip,
                    contains: *before,
                    then: Then::Update { base: *base, rewrite: false },
                });
            }
            Operation::Writer { .. }
            | Operation::Compare { .. }
            | Operation::Create { .. }
            | Operation::Delete { .. }
            | Operation::MergedInto { .. }
            | Operation::Merge { .. }
            | Operation::Open { .. }
            | Operation::Close { .. }
            | Operation::Reopen { .. }
            | Operation::Retarget { .. } => {}
        }
    }
    if let Operation::Push { .. } = op {
        branch_after_failed_push(op, lost, tip)
    } else {
        Found::NoProof { observed: tip }
    }
}

fn apply(d: &mut Domain, env: &Env<Limits>, key: (Name, u64), check: Check, found: Found, out: &mut Queue<Request>) {
    let (name, order) = &key;
    match found {
        Found::Check(check) => schedule(d, name, *order, check),
        Found::Retry => {
            let at = env.now.saturating_add(env.limits.client.poll);
            if at > env.now {
                d.write_reads.pending.insert(key, Job { check, at }).expect("one retry per write");
            }
        }
        Found::Ordered { head, observed, contains } => {
            let row = d.writes.get_mut(name).expect("record checked");
            if row.head != Some(head) {
                return;
            }
            if contains {
                row.head = Some(observed);
            }
            if contains {
                save_changed(row, Some(head), &mut d.write_heads, out);
            }
            if !contains {
                d.write_reads.checked.insert(key, Checked { head: Some(head), observed }).expect("one check per write");
            }
        }
        Found::Compared { head, waiting, older } => {
            if super::known_head(d, name) != Some(head) {
                previous::compare(d, name, *order, waiting, out);
            } else if older {
                previous::ordered(d, name, *order, head, waiting, out);
            } else {
                previous::ordered(d, name, *order, waiting, head, out);
            }
        }
        Found::WriterProgress { head, tip, mode } => previous::progress(d, name, *order, head, tip, mode, out),
        Found::NoProof { observed: Some(observed) } => {
            let head = super::known_head(d, name);
            d.write_reads.checked.insert(key, Checked { head, observed }).expect("one check per write");
        }
        Found::NoProof { observed: None } => {}
        Found::Confirm { after, pull } => {
            settle(d, name, *order, WriteState::Confirmed, after, pull, out);
        }
        Found::Refuse => settle(d, name, *order, WriteState::Refused, None, None, out),
    }
}

pub(super) fn settle(
    d: &mut Domain,
    name: &Name,
    order: u64,
    state: WriteState,
    head: Option<Commit>,
    number: Option<u64>,
    out: &mut Queue<Request>,
) {
    let Some(row) = d.writes.get_mut(name) else { return };
    let previous = row.head;
    for entry in &mut row.entries {
        if entry.order != order || !entry.state.unsettled() {
            continue;
        }
        entry.state = state;
        if state == WriteState::Confirmed {
            match &mut entry.operation {
                Operation::Update { after, .. } | Operation::MergedInto { after, .. } => {
                    if let Some(head) = head {
                        *after = Some(head);
                    }
                }
                Operation::Open { pull, .. } => {
                    if let Some(number) = number {
                        *pull = Some(number);
                    }
                }
                Operation::Writer { .. }
                | Operation::Push { .. }
                | Operation::Create { .. }
                | Operation::Delete { .. }
                | Operation::Merge { .. }
                | Operation::Close { .. }
                | Operation::Reopen { .. }
                | Operation::Retarget { .. }
                | Operation::Compare { .. } => {}
            }
        }
        break;
    }
    if state == WriteState::Refused {
        row.entries = filtered(row, None, true).into_boxed();
    } else {
        advance(row);
    }
    save_changed(row, previous, &mut d.write_heads, out);
    forget(d, name, order);
    if !d.heads.contains_key(name) && !d.holds.contains_key(name) {
        super::remove(d, name, out);
    }
}

pub(super) fn retryable_push(sender: Sender, op: &Operation, state: WriteState, tip: Option<Commit>) -> Option<Commit> {
    if state == WriteState::Uncertain
        && let Sender::Attempt { .. } = sender
        && let Operation::Push { before, after } = op
        && *before != tip
    {
        Some(*after)
    } else {
        None
    }
}
