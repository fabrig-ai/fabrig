//! Fresh, pinned reads for write intents and completed attempts (writes.md, sections 3–4 and 7).
use super::{Domain, Name, Operation, Queue, Request, Sender, Tip, WriteState, advance, after, before, filtered, save};
use crate::Limits;
use skein_forge::api::{Answer, Commit, Error, Read, Repository};
use skein_forge_client::{Event, What};
use skein_lib::{Env, List, Map, Time, Token};

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
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Then {
    Push { lost: bool },
    Writer,
    Update { base: Commit },
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

pub(super) fn intent(d: &mut Domain, name: &Name, order: u64, observed: Commit) {
    let Some(entry) = entry(d, name, order) else { return };
    let check = match entry.operation {
        Operation::Update { before, base, .. } => {
            Check::Contains { tip: observed, contains: before, then: Then::Update { base } }
        }
        Operation::MergedInto { .. } => Check::Pull { observed: Some(observed) },
        Operation::Writer { before: Some(before) } => {
            Check::Contains { tip: observed, contains: before, then: Then::Writer }
        }
        Operation::Compare { .. } => {
            let Some(head) = super::known_head(d, name) else { return };
            Check::Order { head, observed }
        }
        Operation::Writer { before: None }
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
    schedule(d, name, order, Check::Branch { lost });
}

pub(crate) fn restart(d: &mut Domain) {
    let mut needed = List::uncapped();
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
                | Operation::Writer { .. }
                | Operation::Compare { .. } => Check::Branch { lost: true },
            };
            needed.push((name.clone(), entry.order, check)).expect("uncapped readback queue");
        }
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
        Check::Branch { lost } if entry.state.unsettled() => match branch_tip(result) {
            Ok(tip) => branch_answer(&entry.operation, lost, tip),
            Err(()) => Found::Retry,
        },
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
                    Then::Writer | Then::UpdateBase => Found::Confirm { after: Some(tip), pull: None },
                    Then::Update { base } => {
                        Found::Check(Check::Contains { tip, contains: base, then: Then::UpdateBase })
                    }
                },
                Status::Failed => match then {
                    Then::Push { lost } => branch_after_failed_push(&entry.operation, lost, Some(tip)),
                    Then::Writer | Then::Update { .. } | Then::UpdateBase => Found::NoProof { observed: Some(tip) },
                },
                Status::Unknown | Status::Pending => Found::Retry,
            }
        }
        Check::Branch { .. } | Check::Pull { .. } | Check::Contains { .. } => return,
    };
    apply(d, env, key, check, found, out);
}

#[derive(Clone, Copy)]
enum Found {
    Confirm { after: Option<Commit>, pull: Option<u64> },
    Refuse,
    Ordered { head: Commit, observed: Commit, contains: bool },
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
                return Found::Check(Check::Contains { tip, contains: *before, then: Then::Update { base: *base } });
            }
            Operation::Writer { before: Some(before) } => {
                return Found::Check(Check::Contains { tip, contains: *before, then: Then::Writer });
            }
            Operation::Writer { before: None } => return Found::Confirm { after: Some(tip), pull: None },
            Operation::Compare { after } => return Found::Check(Check::Order { head: tip, observed: *after }),
            Operation::Create { .. }
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
            let mut settled = false;
            for entry in &mut row.entries {
                if entry.order == *order
                    && entry.state.unsettled()
                    && let Operation::Compare { .. } = entry.operation
                {
                    entry.state = WriteState::Confirmed;
                    settled = true;
                    break;
                }
            }
            if contains {
                row.head = Some(observed);
            }
            if contains || settled {
                save(row, out);
            }
            if !contains {
                d.write_reads.checked.insert(key, Checked { head: Some(head), observed }).expect("one check per write");
            }
        }
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
                Operation::Writer { before } if *before == row.head => {
                    if head.is_some() {
                        row.head = head;
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
    save(row, out);
    forget(d, name, order);
    if !d.heads.contains_key(name) {
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
