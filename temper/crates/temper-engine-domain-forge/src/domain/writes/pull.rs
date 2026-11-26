//! A pull's state and base are judged on its head branch's record (writes.md, 4).
use super::{Operation, WriteState, Writes, judge::drift, matches_pull, pull_number};
use skein_forge::api::{Pull, State};
use temper_engine_domain_forge_vocabulary::writes::Seen;

pub(super) fn judge(row: &Writes, pull: &Pull, target: Option<&[u8]>) -> Seen {
    let mut state = None;
    let mut base = None;
    let mut was_closed = false;
    let mut opening = None;
    for entry in &row.entries {
        if entry.state == WriteState::Refused {
            continue;
        }
        if let Operation::Open { pull: None, .. } = &entry.operation
            && entry.state.unsettled()
            && matches_pull(&entry.operation, pull)
        {
            opening = Some(entry.order);
        }
        if pull_number(&entry.operation) != Some(pull.number) {
            continue;
        }
        match &entry.operation {
            Operation::Open { base: path, .. } => {
                base = Some((entry, None, path.as_ref()));
                state = Some(entry);
            }
            Operation::Retarget { from, to, .. } => base = Some((entry, Some(from.as_ref()), to.as_ref())),
            Operation::Close { .. } => {
                was_closed = true;
                state = Some(entry);
            }
            Operation::Merge { .. } | Operation::Reopen { .. } => state = Some(entry),
            Operation::Update { .. }
            | Operation::MergedInto { .. }
            | Operation::Push { .. }
            | Operation::Create { .. }
            | Operation::Delete { .. }
            | Operation::Writer { .. }
            | Operation::Compare { .. } => {}
        }
    }
    let state = state_seen(row, pull, state, was_closed, opening);
    let base = if let Some((entry, from, to)) = base {
        if pull.base.as_ref() == to {
            if entry.state.unsettled()
                && let Operation::Retarget { .. } = entry.operation
            {
                Seen::Explained { order: entry.order }
            } else {
                Seen::Current
            }
        } else if from == Some(pull.base.as_ref()) {
            if entry.state == WriteState::Confirmed { Seen::Behind { order: entry.order } } else { Seen::Current }
        } else {
            drift(row)
        }
    } else if target_changed(pull, target) {
        drift(row)
    } else {
        Seen::Current
    };
    combined(state, base)
}

fn state_seen(
    row: &Writes,
    pull: &Pull,
    state: Option<&super::Written>,
    was_closed: bool,
    opening: Option<u64>,
) -> Seen {
    if let Some(entry) = state {
        if matches_pull(&entry.operation, pull) {
            if entry.state.unsettled() { Seen::Explained { order: entry.order } } else { Seen::Current }
        } else if pull.merged.is_some() {
            drift(row)
        } else {
            let before = match (&entry.operation, pull.state) {
                (Operation::Merge { .. } | Operation::Close { .. } | Operation::Open { .. }, State::Open) => true,
                (Operation::Reopen { .. }, State::Closed) => was_closed,
                (Operation::Merge { .. } | Operation::Close { .. } | Operation::Open { .. }, State::Closed)
                | (Operation::Reopen { .. }, State::Open)
                | (
                    Operation::Update { .. }
                    | Operation::MergedInto { .. }
                    | Operation::Retarget { .. }
                    | Operation::Push { .. }
                    | Operation::Create { .. }
                    | Operation::Delete { .. }
                    | Operation::Writer { .. }
                    | Operation::Compare { .. },
                    _,
                ) => false,
            };
            if before {
                if entry.state == WriteState::Confirmed
                    && let Operation::Open { .. } = entry.operation
                {
                    // The head can change after an open, even with no further
                    // pull-state write. It is judged by its branch's record.
                    Seen::Current
                } else if entry.state == WriteState::Confirmed {
                    Seen::Behind { order: entry.order }
                } else {
                    Seen::Current
                }
            } else {
                drift(row)
            }
        }
    } else if let Some(order) = opening {
        Seen::Explained { order }
    } else if pull.state == State::Open && pull.merged.is_none() {
        Seen::Current
    } else {
        drift(row)
    }
}

pub(super) fn target_changed(pull: &Pull, target: Option<&[u8]>) -> bool {
    if let Some(target) = target { pull.base.as_ref() != target } else { false }
}

pub(super) fn combined(a: Seen, b: Seen) -> Seen {
    match (a, b) {
        (drift @ Seen::Drift { .. }, _) | (_, drift @ Seen::Drift { .. }) => drift,
        (behind @ Seen::Behind { .. }, _) | (_, behind @ Seen::Behind { .. }) => behind,
        (Seen::Owed, _) | (_, Seen::Owed) => Seen::Owed,
        (Seen::Explained { order: a }, Seen::Explained { order: b }) => Seen::Explained { order: a.max(b) },
        (seen, Seen::Current) | (Seen::Current, seen) => seen,
    }
}
