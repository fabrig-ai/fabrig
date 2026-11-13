//! A change's effects at their deciding steps
//! (temper/docs/design/domain/landing-history.md, 2.2, 3.2 and 3.4).
use crate::{ChangeRow, Domain, Request, history};
use skein_forge::api::{Commit, Pull, State};
use skein_lib::{List, Queue, Wall};
use temper_engine_domain_forge_change as change;

pub(crate) fn landing(d: &Domain, row: &ChangeRow) -> Option<history::InLanding> {
    let task = row.change.landing?;
    let landing = d.landing.rows.get(&task)?;
    for member in &landing.landing.members {
        if member.change == row.task {
            return Some(history::InLanding { task, round: landing.landing.preparation, unit: member.unit });
        }
    }
    None
}

pub(crate) fn step(d: &Domain, row: &ChangeRow, what: history::What) -> history::Step {
    history::Step {
        what,
        landing: landing(d, row),
        head: None,
        base: None,
        operation: history::Operation::Own,
        attempt: 0,
        source_at: None,
    }
}

/// Keep the writer's history cursor alongside a caller's changed row.
pub(crate) fn write(d: &mut Domain, wall: Wall, row: &mut ChangeRow, step: history::Step, out: &mut Queue<Request>) {
    row.history = d.changes.get(&row.task).expect("retained change").history;
    d.changes.insert(row.task, row.clone()).expect("retained change");
    d.write_event(wall, row.task, step, out);
    row.history = d.changes.get(&row.task).expect("retained change").history;
}

pub(crate) fn pushed(d: &mut Domain, wall: Wall, name: &crate::Name, head: Commit, out: &mut Queue<Request>) {
    let mut changes = List::uncapped();
    for (_, row) in &d.changes {
        if row.repository.forge == name.forge && row.repository.repository == name.repository
            && crate::items::branch_matches(name, &row.branch)
        {
            changes.push(row.clone()).expect("live changes");
        }
    }
    for row in changes.into_boxed() {
        let mut event = step(d, &row, history::What::Pushed);
        event.head = Some(head);
        d.write_event(wall, row.task, event, out);
    }
}

pub(crate) fn asked(kind: change::Delegate) -> Option<(history::Ask, Option<u16>)> {
    match kind {
        change::Delegate::Resolve { .. } => Some((history::Ask::Resolution, None)),
        change::Delegate::Repair(why) => {
            let gate = match why {
                change::Repair::Gate(gate) => u16::try_from(gate).ok(),
                change::Repair::Landing { gate, .. } | change::Repair::LandingAt { gate, .. } => Some(gate),
                change::Repair::Ci | change::Repair::Semantic | change::Repair::Review { .. } => None,
            };
            Some((history::Ask::Repair, gate))
        }
        change::Delegate::Gate { number, .. } => Some((history::Ask::Judgement, u16::try_from(number).ok())),
        change::Delegate::Produce | change::Delegate::Close | change::Delegate::Merge { .. } => None,
    }
}

pub(crate) fn delegated(d: &mut Domain, wall: Wall, row: &mut ChangeRow, child: u64, kind: change::Delegate, out: &mut Queue<Request>) {
    let Some((ask, gate)) = asked(kind) else { return };
    let mut event = step(d, row, history::What::Asked { ask, gate, task: child });
    event.operation = history::Operation::Of(child);
    event.head = match kind {
        change::Delegate::Gate { head, .. } => Some(head),
        change::Delegate::Resolve { base } => { event.base = Some(base); row.change.last_head }
        change::Delegate::Repair(_) | change::Delegate::Produce | change::Delegate::Close | change::Delegate::Merge { .. } => row.change.last_head,
    };
    write(d, wall, row, event, out);
}

pub(crate) fn answered(d: &mut Domain, wall: Wall, row: &mut ChangeRow, child: u64, kind: change::Delegate, outcome: history::Answer, out: &mut Queue<Request>) {
    let Some((ask, _)) = asked(kind) else { return };
    let mut event = step(d, row, history::What::Answered { ask, outcome });
    event.operation = history::Operation::Of(child);
    if outcome == history::Answer::Pushed { event.head = row.change.last_head; }
    write(d, wall, row, event, out);
}

pub(crate) fn recorded(d: &mut Domain, wall: Wall, row: &mut ChangeRow, pull: &Pull, out: &mut Queue<Request>) {
    let state = match pull.state {
        State::Merged => history::RecordedState::Merged,
        State::Closed => history::RecordedState::Closed,
        State::Open => return,
    };
    let mut event = step(d, row, history::What::Recorded { state });
    event.operation = history::Operation::Of(pull.number);
    event.head = pull.merged;
    write(d, wall, row, event, out);
}

pub(crate) fn advanced(d: &mut Domain, wall: Wall, row: &mut ChangeRow, prior: &change::Change, out: &mut Queue<Request>) {
    if crate::domain::landed_commit(&prior.state).is_none()
        && let Some(head) = crate::domain::landed_commit(&row.change.state)
    {
        let mut event = step(d, row, history::What::Landed);
        event.head = Some(head);
        write(d, wall, row, event, out);
    }
    if prior.state == row.change.state { return; }
    let verdict = match row.change.state {
        change::State::Red { .. } => history::AfterVerdict::Red,
        change::State::Cleared { .. } => history::AfterVerdict::Cleared,
        change::State::Uncleared { .. } => history::AfterVerdict::Uncleared,
        _ => return,
    };
    let mut event = step(d, row, history::What::After { verdict });
    event.head = crate::domain::landed_commit(&row.change.state);
    write(d, wall, row, event, out);
}
