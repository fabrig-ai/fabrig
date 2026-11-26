//! Judgement reads the record without choosing whether a caller waits or holds.
use super::{Operation, Tip, WRITES, WriteState, Writes, after, before};
use skein_forge::api::Commit;
use temper_engine_domain_forge_vocabulary::writes::{Actor, Seen};

pub(super) enum Judgement {
    Known(Seen),
    Explained { order: u64, path: [bool; WRITES] },
    Order { order: u64, head: Commit, observed: Commit },
    Readback { order: u64, observed: Commit },
}

pub(super) fn drift(row: &Writes) -> Seen {
    let nearest = match row.entries.last() {
        Some(entry) => Some(entry.order),
        None => None,
    };
    Seen::Drift { nearest }
}

pub(super) fn judge(row: &Writes, observed: Option<Commit>, actor: Actor) -> Judgement {
    if actor == Actor::Other {
        return Judgement::Known(drift(row));
    }
    if observed == row.head {
        return Judgement::Known(Seen::Current);
    }
    if let Some(explanation) = explain(row, observed) {
        return explanation;
    }
    if let Some(ordered) = ordered(row, observed) {
        return ordered;
    }
    // An absent read with no delete cannot be put down to an old creation.
    if observed.is_some() {
        for entry in &row.entries {
            if before(&entry.operation) == Tip::Known(observed) && entry.state == WriteState::Confirmed {
                return Judgement::Known(Seen::Behind { order: entry.order });
            }
        }
    }
    if let Some(observed) = observed {
        for entry in &row.entries {
            if entry.state.unsettled() && intent(&entry.operation, observed) {
                return Judgement::Readback { order: entry.order, observed };
            }
        }
    }
    Judgement::Known(drift(row))
}

fn explain(row: &Writes, observed: Option<Commit>) -> Option<Judgement> {
    // The parent is either the current head (WRITES) or an earlier reachable
    // entry. Keep a path instead of greedily following the first push: two
    // admitted pushes can have leased the same head, but only one can land.
    let mut parent = [None; WRITES];
    let mut end = None;
    for (index, entry) in row.entries.iter().enumerate() {
        if !entry.state.unsettled() {
            continue;
        }
        let (Tip::Known(from), Tip::Known(to)) = (before(&entry.operation), after(&entry.operation)) else { continue };
        let mut link = if from == row.head { Some(WRITES) } else { None };
        for (pos, prior) in row.entries.iter().take(index).enumerate() {
            if parent.get(pos).expect("stored bound").is_some() && after(&prior.operation) == Tip::Known(from) {
                link = Some(pos);
            }
        }
        *parent.get_mut(index).expect("stored bound") = link;
        if link.is_some() && to == observed {
            end = Some((index, entry.order));
        }
    }
    let (mut index, order) = end?;
    let mut path = [false; WRITES];
    while index != WRITES {
        *path.get_mut(index).expect("reachable entry") = true;
        index = parent.get(index).expect("reachable entry").expect("the explanation has a path to its head");
    }
    Some(Judgement::Explained { order, path })
}

fn ordered(row: &Writes, observed: Option<Commit>) -> Option<Judgement> {
    let mut order = None;
    for entry in &row.entries {
        if entry.state != WriteState::Confirmed {
            continue;
        }
        if order.is_some()
            && before(&entry.operation) == Tip::Known(observed)
            && after(&entry.operation) != Tip::Known(observed)
        {
            return Some(Judgement::Known(Seen::Behind { order: entry.order }));
        }
        if after(&entry.operation) == Tip::Known(observed) {
            order = Some(entry.order);
        }
    }
    let order = order?;
    Some(match (row.head, observed) {
        (Some(head), Some(observed)) => Judgement::Order { order, head, observed },
        // The compare API orders commits, not absence. A known removal or
        // a known earlier absence must precede the settled current state.
        (None, _) | (Some(_), None) => Judgement::Known(Seen::Behind { order }),
    })
}

fn intent(operation: &Operation, observed: Commit) -> bool {
    match operation {
        Operation::Update { after, .. } | Operation::MergedInto { after, .. } => after.is_none(),
        Operation::Writer { .. } => true,
        Operation::Compare { after } => *after == observed,
        Operation::Push { .. }
        | Operation::Create { .. }
        | Operation::Delete { .. }
        | Operation::Merge { .. }
        | Operation::Open { .. }
        | Operation::Close { .. }
        | Operation::Reopen { .. }
        | Operation::Retarget { .. } => false,
    }
}
