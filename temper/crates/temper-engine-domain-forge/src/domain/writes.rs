//! The connector's own writes, independent of the forge client's kept view
//! (temper/docs/design/domain/forge/writes.md, sections 1–7).
use super::{Domain, Limits, Name, Queue, Request, Stored, What, emit, from_client};
use alloc::boxed::Box;
use skein_forge::api::{Commit, Repository};
use skein_forge_client::{Resource, What as ClientWhat};
use skein_lib::List;
use temper_engine_domain_forge_vocabulary::writes::{Full, Operation, Sender, WRITES, WriteState, Writes, Written};

pub(crate) mod outbox;

pub(crate) fn branch(limits: &Limits, repository: Repository, branch: &[u8]) -> Option<Name> {
    from_client(&Resource { repository, what: ClientWhat::Branch(Box::from(branch)) }, limits)
}

pub(super) fn known_head(d: &Domain, name: &Name) -> Option<Commit> {
    match d.writes.get(name) {
        Some(row) => row.head,
        None => match d.heads.get(name) {
            Some(row) => Some(row.commit),
            None => None,
        },
    }
}

/// Absence is a branch state. A pull-only operation, or an unfilled
/// intent, names no branch state at all.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Tip {
    Unknown,
    Known(Option<Commit>),
}

const fn before(op: &Operation) -> Tip {
    match op {
        Operation::Push { before, .. }
        | Operation::Delete { before }
        | Operation::MergedInto { before, .. }
        | Operation::Writer { before } => Tip::Known(*before),
        Operation::Create { .. } => Tip::Known(None),
        Operation::Update { before, .. } => Tip::Known(Some(*before)),
        Operation::Merge { .. }
        | Operation::Open { .. }
        | Operation::Close { .. }
        | Operation::Reopen { .. }
        | Operation::Retarget { .. }
        | Operation::Compare { .. } => Tip::Unknown,
    }
}

const fn after(op: &Operation) -> Tip {
    match op {
        Operation::Push { after, .. } | Operation::Create { after } | Operation::Compare { after } => {
            Tip::Known(Some(*after))
        }
        Operation::Delete { .. } => Tip::Known(None),
        Operation::Update { after, .. } | Operation::MergedInto { after, .. } => match after {
            Some(after) => Tip::Known(Some(*after)),
            None => Tip::Unknown,
        },
        Operation::Merge { .. }
        | Operation::Open { .. }
        | Operation::Close { .. }
        | Operation::Reopen { .. }
        | Operation::Retarget { .. }
        | Operation::Writer { .. } => Tip::Unknown,
    }
}

fn evict(row: &Writes) -> Option<u64> {
    for entry in &row.entries {
        if entry.state == WriteState::Confirmed && after(&entry.operation) != Tip::Known(row.head) {
            return Some(entry.order);
        }
    }
    None
}

pub(crate) fn full(d: &Domain, name: &Name) -> bool {
    if let Some(row) = d.writes.get(name) {
        row.next == u64::MAX || (row.entries.len() >= WRITES && evict(row).is_none())
    } else {
        false
    }
}

fn senders(row: &Writes) -> Box<[Sender]> {
    let mut senders = List::with_capacity(write_limit());
    for entry in &row.entries {
        if entry.state.unsettled() {
            senders.push(entry.sender).expect("one sender per unsettled entry");
        }
    }
    senders.into_boxed()
}

fn write_limit() -> u32 {
    u32::try_from(WRITES).expect("the stored write bound fits")
}

pub(super) fn save(row: &Writes, out: &mut Queue<Request>) {
    emit(out, Request::Save { read_afresh: None, release: None, record: Stored::Writes(row.clone()) });
}

pub(crate) fn remove(d: &mut Domain, name: &Name, out: &mut Queue<Request>) {
    if let Some(row) = d.writes.get(name) {
        for entry in &row.entries {
            if entry.state.unsettled() {
                return;
            }
        }
        let row = d.writes.remove(name).expect("retained record");
        for entry in &row.entries {
            readback::forget(d, name, entry.order);
        }
        emit(out, Request::Erase { key: crate::Key::Writes(name.clone()) });
    }
}

fn filtered(row: &mut Writes, erase: Option<u64>, drop_refused: bool) -> List<Written> {
    let mut entries = List::with_capacity(write_limit());
    for entry in core::mem::replace(&mut row.entries, Box::new([])) {
        if Some(entry.order) != erase && !(drop_refused && entry.state == WriteState::Refused) {
            entries.push(entry).expect("the record's stored list bound");
        }
    }
    entries
}

/// Move through the known, settled writes in admission order, without ever
/// placing a late answer in front of the write that built on it.
fn advance(row: &mut Writes) {
    let mut at = 0;
    for entry in &row.entries {
        if entry.state == WriteState::Confirmed && after(&entry.operation) == Tip::Known(row.head) {
            at = entry.order;
        }
    }
    for entry in &row.entries {
        if entry.state == WriteState::Confirmed
            && entry.order > at
            && before(&entry.operation) == Tip::Known(row.head)
            && let Tip::Known(after) = after(&entry.operation)
        {
            row.head = after;
        }
    }
}

impl Domain {
    /// Admit a push to a destination already authorized by the assignment.
    pub fn admit_push(
        &mut self,
        limits: &Limits,
        name: &Name,
        expected: Option<Commit>,
        commit: Commit,
        sender: Sender,
        out: &mut Queue<Request>,
    ) -> Result<(), Box<[u8]>> {
        match self.admit(name, Operation::Push { before: expected, after: commit }, sender, out) {
            Ok(_) => Ok(()),
            Err(_) => Err(room_account(self, limits, name)),
        }
    }

    /// Admit one branch write in the caller's decision, before it can be sent.
    pub fn admit(
        &mut self,
        name: &Name,
        operation: Operation,
        sender: Sender,
        out: &mut Queue<Request>,
    ) -> Result<u64, Full> {
        self.admit_with_head(name, operation, sender, None, out)
    }

    fn admit_with_head(
        &mut self,
        name: &Name,
        operation: Operation,
        sender: Sender,
        initial: Option<Commit>,
        out: &mut Queue<Request>,
    ) -> Result<u64, Full> {
        let What::Branch(_) = &name.what else { unreachable!("a write record is keyed by a branch") };
        let head = known_head(self, name).or(initial);
        if !self.writes.contains_key(name) {
            self.writes
                .insert(name.clone(), Writes { name: name.clone(), head, next: 1, entries: Box::new([]) })
                .expect("uncapped branch write records");
        }
        let row = self.writes.get_mut(name).expect("the branch has its record");
        let order = row.next;
        let Some(next) = order.checked_add(1) else {
            return Err(Full { name: name.clone(), senders: senders(row) });
        };
        // Another worker admission vouches for the head it read, including
        // when the record has no settled link to that worker's first lease.
        let mut leased = None;
        if let (Sender::Attempt { .. }, Operation::Push { before: Some(expected), .. }) = (sender, &operation) {
            for prior in &mut row.entries {
                if prior.state.unsettled()
                    && let Operation::Push { after, .. } = prior.operation
                    && after == *expected
                {
                    prior.state = WriteState::Confirmed;
                    leased = Some(prior.order);
                }
            }
            if leased.is_some() {
                advance(row);
            }
        }
        let erase = if row.entries.len() < WRITES {
            None
        } else if let Some(evict) = evict(row) {
            Some(evict)
        } else {
            if leased.is_some() {
                save(row, out);
            }
            return Err(Full { name: name.clone(), senders: senders(row) });
        };
        let mut entries = filtered(row, erase, false);
        entries.push(Written { order, sender, state: WriteState::Admitted, operation }).expect("room checked");
        row.entries = entries.into_boxed();
        row.next = next;
        save(row, out);
        if let Some(erase) = erase {
            readback::forget(self, name, erase);
        }
        Ok(order)
    }
}

/// A full record's bounded hold words, oldest senders first.
pub(crate) fn room_account(d: &Domain, l: &Limits, name: &Name) -> Box<[u8]> {
    use skein_lib::{Decimal, bytes};
    let row = d.writes.get(name).expect("full record");
    let senders = senders(row);
    let resource = super::to_client(name, l).expect("stored branch name fits");
    let ClientWhat::Branch(path) = resource.what else { unreachable!("record belongs to a branch") };
    let bound = skein_lib::usize_of(crate::EFFECT_ACCOUNT_BYTES);
    let mut words = bytes::concat(&[
        b"waiting for room in the write record of ",
        &path,
        b" (",
        Decimal::of(u64::try_from(WRITES).expect("write limit")).as_bytes(),
        b" entries), held by ",
    ]);
    for (at, sender) in senders.iter().enumerate() {
        let sender = match sender {
            Sender::Attempt { task, attempt } => bytes::concat(&[
                b"task ",
                Decimal::of(*task).as_bytes(),
                b" attempt ",
                Decimal::of(*attempt).as_bytes(),
            ]),
            Sender::Entry { task, entry } => {
                bytes::concat(&[b"task ", Decimal::of(*task).as_bytes(), b" entry ", Decimal::of(*entry).as_bytes()])
            }
            Sender::Previous => Box::from(b"the previous build".as_slice()),
        };
        let rest = senders.len().saturating_sub(at);
        let suffix = bytes::concat(&[b"and ", Decimal::of(skein_lib::len_u64(rest)).as_bytes(), b" more"]);
        let separator = if at == 0 { b"".as_slice() } else { b", " };
        // Save space for the omitted count; a sender is either named whole
        // or not named at all.
        if words.len().saturating_add(separator.len()).saturating_add(sender.len()).saturating_add(if rest > 1 {
            suffix.len().saturating_add(2)
        } else {
            0
        }) > bound
        {
            let prefix = words
                .get(..words.len().min(bound.saturating_sub(separator.len().saturating_add(suffix.len()))))
                .expect("bounded prefix");
            return bytes::concat(&[prefix, separator, &suffix]);
        }
        words = bytes::concat(&[&words, separator, &sender]);
    }
    words
}

mod judge;
mod pull;
pub(crate) mod readback;

use skein_forge::api::{Pull, State};
use temper_engine_domain_forge_vocabulary::writes::{Actor, Seen};

pub(super) const fn pull_number(op: &Operation) -> Option<u64> {
    match op {
        Operation::Update { pull, .. }
        | Operation::MergedInto { pull, .. }
        | Operation::Merge { pull, .. }
        | Operation::Close { pull }
        | Operation::Reopen { pull }
        | Operation::Retarget { pull, .. } => Some(*pull),
        Operation::Open { pull, .. } => *pull,
        Operation::Push { .. }
        | Operation::Create { .. }
        | Operation::Delete { .. }
        | Operation::Writer { .. }
        | Operation::Compare { .. } => None,
    }
}

fn matches_pull(op: &Operation, pull: &Pull) -> bool {
    if let Some(number) = pull_number(op)
        && number != pull.number
    {
        return false;
    }
    match op {
        Operation::Merge { head, .. } => pull.merged.is_some() && pull.commit == *head,
        Operation::Close { .. } => pull.state == State::Closed && pull.merged.is_none(),
        Operation::Reopen { .. } => pull.state == State::Open && pull.merged.is_none(),
        Operation::Open { base, head, .. } => {
            pull.state == State::Open && pull.merged.is_none() && pull.base == *base && pull.commit == *head
        }
        Operation::Retarget { to, .. } => pull.base == *to,
        Operation::Update { .. }
        | Operation::MergedInto { .. }
        | Operation::Push { .. }
        | Operation::Create { .. }
        | Operation::Delete { .. }
        | Operation::Writer { .. }
        | Operation::Compare { .. } => false,
    }
}

impl Domain {
    /// Judge a branch observation without deciding its caller's policy.
    pub fn observe(&mut self, name: &Name, observed: Option<Commit>, actor: Actor, out: &mut Queue<Request>) -> Seen {
        use judge::Judgement;
        let Some(row) = self.writes.get(name) else { return Seen::Current };
        match judge::judge(row, observed, actor) {
            Judgement::Known(seen) => {
                if let Seen::Drift { .. } = seen
                    && actor != Actor::Other
                {
                    for entry in &row.entries {
                        if let Some(commit) =
                            readback::retryable_push(entry.sender, &entry.operation, entry.state, observed)
                        {
                            let order = entry.order;
                            readback::push_read(self, name, order, observed, commit);
                            return Seen::Owed;
                        }
                    }
                }
                seen
            }
            Judgement::Explained { order, path } => {
                let row = self.writes.get_mut(name).expect("judged record");
                let mut finished = List::with_capacity(write_limit());
                for (entry, proved) in row.entries.iter_mut().zip(path) {
                    if proved {
                        entry.state = WriteState::Confirmed;
                        finished.push(entry.order).expect("one order per entry");
                    }
                }
                row.head = observed;
                save(row, out);
                for order in finished.into_boxed() {
                    readback::forget(self, name, order);
                }
                Seen::Explained { order }
            }
            Judgement::Order { order, head, observed } => {
                if readback::checked(self, name, order, Some(head), observed) {
                    Seen::Behind { order }
                } else {
                    readback::order(self, name, order, head, observed);
                    Seen::Owed
                }
            }
            Judgement::Readback { order, observed } => {
                if readback::checked(self, name, order, row.head, observed) {
                    judge::drift(row)
                } else {
                    readback::intent(self, name, order, observed);
                    Seen::Owed
                }
            }
        }
    }

    /// Judge the pull's own state and base, and the two commits in its view.
    pub fn observe_pull(&mut self, head_branch: &Name, number: u64, read: &Pull, out: &mut Queue<Request>) -> Seen {
        if read.number != number || !crate::items::branch_matches(head_branch, &read.head) {
            return self.pull_drift(head_branch);
        }
        let head = self.observe(head_branch, Some(read.commit), Actor::Unknown, out);
        // The provider's names have already passed the read bound. Use the
        // tracked Name itself, rather than making an untracked base record.
        let mut base = None;
        if read.base_commit.is_some() {
            for (name, _) in &self.writes {
                if name.forge == head_branch.forge
                    && name.repository == head_branch.repository
                    && crate::items::branch_matches(name, &read.base)
                {
                    base = Some(name.clone());
                    break;
                }
            }
        }
        let base = match base {
            Some(base) => self.observe(&base, read.base_commit, Actor::Unknown, out),
            None => Seen::Current,
        };
        let state = match self.writes.get(head_branch) {
            Some(row) => pull::judge(row, read),
            None if read.state == State::Closed || read.merged.is_some() => self.pull_drift(head_branch),
            None => Seen::Current,
        };
        let seen = pull::combined(pull::combined(head, base), state);
        if let Seen::Explained { order } = state
            && let Seen::Current | Seen::Explained { .. } = seen
        {
            readback::settle(self, head_branch, order, WriteState::Confirmed, None, Some(number), out);
        }
        seen
    }

    fn pull_drift(&self, name: &Name) -> Seen {
        match self.writes.get(name) {
            Some(row) => judge::drift(row),
            None => Seen::Drift { nearest: None },
        }
    }

    /// A completed attempt cannot still push. A lost one might, so only a
    /// read showing it landed or that its lease can no longer take settles it.
    pub fn finished(&mut self, task: u64, attempt: u64, lost: bool, out: &mut Queue<Request>) {
        let sender = Sender::Attempt { task, attempt };
        let mut work = List::uncapped();
        let mut changed = List::uncapped();
        for (name, row) in &self.writes {
            for entry in &row.entries {
                if entry.sender == sender && entry.state.unsettled() {
                    work.push((name.clone(), entry.order)).expect("uncapped readbacks");
                    if lost && entry.state != WriteState::Uncertain && changed.last() != Some(name) {
                        changed.push(name.clone()).expect("one change per record");
                    }
                }
            }
        }
        for name in changed.into_boxed() {
            let row = self.writes.get_mut(&name).expect("selected write record");
            for entry in &mut row.entries {
                if entry.sender == sender && entry.state.unsettled() {
                    entry.state = WriteState::Uncertain;
                }
            }
            save(row, out);
        }
        for (name, order) in work.into_boxed() {
            readback::finished(self, &name, order, lost);
        }
    }
}
