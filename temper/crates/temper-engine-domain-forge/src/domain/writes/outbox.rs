//! Admission and progress in the decisions that save the outbox row and its Flight.
use super::{
    Domain, Limits, Name, Operation, Queue, Request, Sender, WriteState, advance, branch, filtered, full, known_head,
    remove, save_changed,
};
use skein_forge::api::{Commit, MergeHow, Repository, Write};
use skein_forge_client::{Condition, Entry, Made, Outcome};
use skein_lib::List;

/// Branch identities and facts held by the procedure that decided the write.
#[derive(Clone, Copy)]
pub(crate) struct Pin {
    pub(crate) head: Option<Commit>,
    pub(crate) base: Option<Commit>,
}

impl Pin {
    pub(crate) const NONE: Self = Self { head: None, base: None };
}

pub(crate) struct Plan {
    head: Option<Target>,
    base: Option<Target>,
}

struct Target {
    name: Name,
    operation: Operation,
    initial: Option<Commit>,
}

fn target(name: Name, operation: Operation, initial: Option<Commit>) -> Target {
    Target { name, operation, initial }
}

pub(crate) fn plan(d: &Domain, l: &Limits, entry: &Entry, path: Option<&[u8]>, pin: Pin) -> Option<Plan> {
    let repository = entry.repository;
    let mut base = None;
    let head = match &entry.effect.write {
        Write::CreateBranch { branch: path, commit } => {
            Some(target(branch(l, repository, path)?, Operation::Create { after: *commit }, None))
        }
        Write::DeleteBranch { branch: path } => {
            let name = branch(l, repository, path)?;
            let before = known_head(d, &name);
            Some(target(name, Operation::Delete { before }, before))
        }
        Write::OpenPull { head, base, .. } => {
            let name = branch(l, repository, head)?;
            let head = pin.head.or(known_head(d, &name))?;
            Some(target(name, Operation::Open { pull: None, base: base.clone(), head }, Some(head)))
        }
        Write::Update { number, .. } => {
            let Condition::Update { head, base } = entry.effect.condition else { return None };
            Some(target(
                branch(l, repository, path?)?,
                Operation::Update { pull: *number, before: head, base, after: None },
                Some(head),
            ))
        }
        Write::Merge { number, head, how } => {
            let Condition::Merge { base: destination } = &entry.effect.condition else { return None };
            base = match how {
                // This records a pull's state at an already-contained prefix;
                // it does not write the base ref (landing-turn.md, section 1).
                MergeHow::Recorded { .. } => None,
                MergeHow::FastForwardOnly | MergeHow::Style => {
                    let name = branch(l, repository, destination)?;
                    let before = pin.base.or(known_head(d, &name));
                    let after = if *how == MergeHow::FastForwardOnly { Some(*head) } else { None };
                    Some(target(name, Operation::MergedInto { pull: *number, before, after }, before))
                }
            };
            Some(target(branch(l, repository, path?)?, Operation::Merge { pull: *number, head: *head }, Some(*head)))
        }
        Write::Close { number } => match path {
            Some(path) => Some(target(branch(l, repository, path)?, Operation::Close { pull: *number }, pin.head)),
            // A close in the issue projection has no head branch.
            None => None,
        },
        Write::Reopen { number } => match path {
            Some(path) => Some(target(branch(l, repository, path)?, Operation::Reopen { pull: *number }, pin.head)),
            None => None,
        },
        Write::CreateIssue { .. }
        | Write::Post { .. }
        | Write::Review { .. }
        | Write::Edit { .. }
        | Write::SetReviewers { .. }
        | Write::Status { .. }
        | Write::Dispatch { .. }
        | Write::Rerun { .. } => None,
    };
    Some(Plan { head, base })
}

impl Plan {
    pub(crate) fn blocked(&self, d: &Domain) -> Option<Name> {
        for target in [&self.head, &self.base].into_iter().flatten() {
            if full(d, &target.name) {
                return Some(target.name.clone());
            }
        }
        None
    }

    pub(crate) fn admit(self, d: &mut Domain, entry: &Entry, out: &mut Queue<Request>) {
        let sender = Sender::Entry { task: entry.task, entry: entry.number };
        for target in [self.head, self.base].into_iter().flatten() {
            d.admit_with_head(&target.name, target.operation, sender, target.initial, out).expect("write room checked");
        }
    }
}

fn changes_record(write: &Write) -> bool {
    match write {
        Write::CreateBranch { .. }
        | Write::DeleteBranch { .. }
        | Write::OpenPull { .. }
        | Write::Merge { .. }
        | Write::Update { .. }
        | Write::Close { .. }
        | Write::Reopen { .. } => true,
        Write::CreateIssue { .. }
        | Write::Post { .. }
        | Write::Review { .. }
        | Write::Edit { .. }
        | Write::SetReviewers { .. }
        | Write::Status { .. }
        | Write::Dispatch { .. }
        | Write::Rerun { .. } => false,
    }
}

pub(crate) fn progress(d: &mut Domain, entry: &Entry, out: &mut Queue<Request>) {
    if changes_record(&entry.effect.write)
        && let Some(previous) = d.entries.get(&entry.number)
        && entry.attempt.is_some()
        && entry.attempt != previous.entry.attempt
    {
        mark(d, Sender::Entry { task: entry.task, entry: entry.number }, Progress::Sent, out);
    }
}

pub(crate) fn outcome(d: &mut Domain, task: u64, entry: u64, outcome: Outcome, out: &mut Queue<Request>) {
    if let Some(row) = d.entries.get(&entry)
        && changes_record(&row.entry.effect.write)
    {
        mark(d, Sender::Entry { task, entry }, Progress::Outcome(outcome), out);
    }
}

/// Settle both records of a round merge proved by its tip or pull. A fresh
/// closed, unmerged pull at the round's base rules out its uncertain sends.
pub(crate) fn merge_readback(
    d: &mut Domain,
    train: &Name,
    task: u64,
    pull: u64,
    candidate: Commit,
    made: bool,
    out: &mut Queue<Request>,
) {
    let Some(row) = d.writes.get(train) else { return };
    let mut senders = List::with_capacity(super::write_limit());
    for entry in &row.entries {
        if let Operation::MergedInto { pull: number, after: Some(after), .. } = entry.operation
            && let Sender::Entry { task: owner, .. } = entry.sender
            && owner == task
            && number == pull
            && after == candidate
        {
            senders.push(entry.sender).expect("one sender per write");
        }
    }
    let outcome = if made {
        Outcome::Made { made: Made::Merged(candidate), found: true }
    } else {
        Outcome::Failed(skein_forge::api::Error::Closed)
    };
    for sender in senders.into_boxed() {
        mark(d, sender, Progress::Outcome(outcome), out);
    }
}

#[derive(Clone, Copy)]
enum Progress {
    Sent,
    Outcome(Outcome),
}

fn mark(d: &mut Domain, sender: Sender, progress: Progress, out: &mut Queue<Request>) {
    let mut names = List::uncapped();
    for (name, row) in &d.writes {
        for entry in &row.entries {
            if entry.sender == sender && entry.state.unsettled() {
                names.push(name.clone()).expect("one name per record");
                break;
            }
        }
    }
    for name in names.into_boxed() {
        let row = d.writes.get_mut(&name).expect("selected record");
        let mut changed = false;
        let mut settled = List::with_capacity(super::write_limit());
        for entry in &mut row.entries {
            if entry.sender != sender || !entry.state.unsettled() {
                continue;
            }
            let state = match progress {
                Progress::Sent => WriteState::Sent,
                Progress::Outcome(Outcome::Made { made, .. }) => {
                    complete(&mut entry.operation, made);
                    WriteState::Confirmed
                }
                Progress::Outcome(Outcome::Failed(_)) => WriteState::Refused,
                Progress::Outcome(Outcome::Withdrawn) if entry.state == WriteState::Admitted => WriteState::Refused,
                Progress::Outcome(
                    Outcome::Withdrawn | Outcome::Queued | Outcome::Uncertain | Outcome::Held | Outcome::Raced { .. },
                ) => WriteState::Uncertain,
            };
            changed |= entry.state != state;
            entry.state = state;
            if !state.unsettled() {
                settled.push(entry.order).expect("one settlement per entry");
            }
        }
        if changed {
            row.entries = filtered(row, None, true).into_boxed();
            let previous = row.head;
            advance(row);
            save_changed(row, previous, &mut d.write_heads, out);
            let empty = row.entries.is_empty();
            for order in settled.into_boxed() {
                super::readback::forget(d, &name, order);
            }
            // An unsettled row could have outlived its BranchHead. Its last
            // settlement lets it go, except while this entry owns a new head.
            if empty && !d.heads.contains_key(&name) {
                remove(d, &name, out);
            }
        }
    }
}

fn complete(op: &mut Operation, made: Made) {
    match op {
        Operation::Update { after, .. } => {
            if let Made::Updated(commit) = made {
                *after = Some(commit);
            }
        }
        Operation::MergedInto { after, .. } => {
            if let Made::Merged(commit) = made {
                *after = Some(commit);
            }
        }
        Operation::Open { pull, .. } => {
            if let Made::Created(number) = made {
                *pull = Some(number);
            }
        }
        Operation::Push { .. }
        | Operation::Create { .. }
        | Operation::Delete { .. }
        | Operation::Merge { .. }
        | Operation::Close { .. }
        | Operation::Reopen { .. }
        | Operation::Retarget { .. }
        | Operation::Writer { .. }
        | Operation::Compare { .. } => {}
    }
}

pub(crate) fn change_branch(d: &Domain, task: u64, repository: Repository) -> Option<&[u8]> {
    let row = d.changes.get(&task)?;
    if row.repository == repository { Some(&row.branch) } else { None }
}
