//! Change effects at their deciding steps (domain/landing-history.md, 2.2, 3.2 and 3.4).
use crate::{ChangeRow, Domain, Request, history};
use skein_forge::api::{Commit, Pull, State};
use skein_lib::{List, Queue, Wall};
use temper_engine_domain_forge_change as change;

#[expect(clippy::match_like_matches_macro, reason = "step code uses enum patterns, not matches macros")]
pub(crate) fn unfollowed(state: &change::State) -> bool {
    if let change::State::Landed { .. } = state { true } else { false }
}

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

pub(crate) fn base(d: &Domain, row: &ChangeRow) -> Option<Commit> {
    if let Some(base) = crate::domain::updating_base(&row.change.state) { return Some(base); }
    if let change::State::Resolving { base, .. } | change::State::WaitingProducer { base } = row.change.state { return Some(base); }
    for (name, head) in &d.heads {
        if name.forge == row.repository.forge && name.repository == row.repository.repository
            && crate::items::branch_matches(name, &row.base) { return Some(head.commit); }
    }
    None
}

pub(crate) fn step(d: &Domain, row: &ChangeRow, what: history::What) -> history::Step {
    history::Step { what, landing: landing(d, row), head: None, base: None, operation: history::Operation::Own, attempt: 0, source_at: None }
}

/// Keep the writer's cursor alongside a caller's changed row.
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
            && crate::items::branch_matches(name, &row.branch) && row.change.last_head != Some(head)
        { changes.push(row.clone()).expect("live changes"); }
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
                change::Repair::Landing { gate } | change::Repair::LandingAt { gate, .. } => Some(gate),
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
    if pull.merged.is_none() { return; }
    // Squashed members are recorded by their landing's own effect step.
    for (_, landing) in &d.landing.rows {
        for member in &landing.landing.members {
            if member.change == row.task && member.landed.is_some()
                && member.combined == Some(temper_engine_domain_forge_landing::Combine::Squash) { return; }
        }
    }
    let mut event = step(d, row, history::What::Recorded { state: history::RecordedState::Merged });
    event.operation = history::Operation::Of(pull.number);
    event.head = pull.merged;
    write(d, wall, row, event, out);
}

/// A gather's terminal pull observation is not a keep poll: it changes no CI,
/// wake or drift facts. Its saved pull state fences a later poll or restart.
pub(crate) fn observed_pull(d: &mut Domain, wall: Wall, task: u64, pull: &Pull, out: &mut Queue<Request>) {
    if pull.state == State::Open && pull.merged.is_none() { return; }
    let Some(mut row) = d.changes.get(&task).cloned() else { return };
    let name = crate::Name { forge: row.repository.forge, repository: row.repository.repository, what: crate::What::Pull(pull.number) };
    if let Some(prior) = d.pulls.get(&name) && prior.head == pull.commit && prior.state == pull.state { return; }
    let state = crate::PullState { name: name.clone(), head: pull.commit, ci: pull.ci, state: pull.state };
    if d.pulls.insert(name, state.clone()).is_err() { return; }
    out.push(Request::Save { record: crate::Stored::PullState(state), release: None, read_afresh: None });
    recorded(d, wall, &mut row, pull, out);
}

pub(crate) fn conflict(d: &mut Domain, wall: Wall, row: &mut ChangeRow, out: &mut Queue<Request>) {
    let mut event = step(d, row, history::What::Conflict { with: history::With::Branch, files: 0 });
    event.head = row.change.last_head;
    event.base = base(d, row);
    write(d, wall, row, event, out);
}

pub(crate) fn advanced(d: &mut Domain, wall: Wall, row: &mut ChangeRow, prior: &change::Change, out: &mut Queue<Request>) {
    if row.change.resolutions > prior.resolutions && row.effect != change::EffectResult::Conflict
        && row.change.way == change::Way::Queue { conflict(d, wall, row, out); }
    if crate::domain::landed_commit(&prior.state).is_none()
        && let Some(head) = crate::domain::landed_commit(&row.change.state)
    {
        let mut event = step(d, row, history::What::Landed);
        if event.landing.is_none() {
            let mut member = row.clone();
            member.change.landing = prior.landing;
            event.landing = landing(d, &member);
        }
        event.head = Some(head);
        write(d, wall, row, event, out);
    }
    if prior.state == row.change.state { return; }
    let verdict = match row.change.state {
        change::State::Red { .. } => history::AfterVerdict::Red,
        change::State::Cleared { .. } => history::AfterVerdict::Cleared,
        change::State::Uncleared { .. } => history::AfterVerdict::Uncleared,
        change::State::Producing { .. } | change::State::Opening { .. } | change::State::Recreating { .. }
        | change::State::Reopening { .. } | change::State::Checking { .. } | change::State::Judging { .. }
        | change::State::Gating { .. } | change::State::CancellingGates { .. } | change::State::Ready { .. }
        | change::State::Preparing { .. } | change::State::Queued { .. } | change::State::First { .. }
        | change::State::Updating { .. } | change::State::WaitingProducer { .. } | change::State::Resolving { .. }
        | change::State::Repairing { .. } | change::State::Landing { .. } | change::State::Landed { .. }
        | change::State::NoChange { .. } | change::State::Ended | change::State::Held { .. } => return,
    };
    let mut event = step(d, row, history::What::After { verdict });
    event.head = crate::domain::landed_commit(&row.change.state);
    write(d, wall, row, event, out);
}
