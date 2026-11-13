//! Change intervals, decided with their policy step (domain/landing-history.md, 2.4 and 3.2).
use alloc::boxed::Box;
use skein_lib::{Env, List, Queue};
use temper_engine_domain_forge_change as c;
use temper_engine_domain_forge_landing::Phase;

use crate::{ChangeRow, Domain, Limits, Request, history as h};

fn landing(d: &Domain, row: &ChangeRow) -> Option<h::InLanding> {
    let task = row.change.landing?;
    let landing = &d.landing.rows.get(&task)?.landing;
    if landing.preparation == 0 {
        return None;
    }
    match landing.phase {
        Phase::Landed | Phase::Cancelled => return None,
        Phase::Waiting
        | Phase::Preparing
        | Phase::Proving
        | Phase::Proved { .. }
        | Phase::Turn { .. }
        | Phase::Held { .. } => {}
    }
    for member in &landing.members {
        if member.change == row.task && member.left.is_none() && member.landed.is_none() {
            return Some(h::InLanding { task, round: landing.preparation, unit: member.unit });
        }
    }
    None
}

fn write(d: &mut Domain, env: &Env<Limits>, row: &mut ChangeRow, step: h::Step, out: &mut Queue<Request>) -> u64 {
    d.changes.insert(row.task, row.clone()).expect("existing change");
    let number = d.write_event(env.wall, row.task, step, out).expect("adopted change has a history");
    row.history = d.changes.get(&row.task).expect("written change").history;
    number
}

fn step(d: &Domain, row: &ChangeRow, what: h::What, operation: h::Operation) -> h::Step {
    h::Step { what, landing: landing(d, row), head: None, base: None, operation, attempt: 0, source_at: None }
}

fn ended(state: &c::State) -> Option<h::Ending> {
    match state {
        c::State::Ended => Some(h::Ending::Cancelled),
        c::State::NoChange { .. } => Some(h::Ending::NoChange),
        c::State::Producing { .. }
        | c::State::Opening { .. }
        | c::State::Recreating { .. }
        | c::State::Reopening { .. }
        | c::State::Checking { .. }
        | c::State::Gating { .. }
        | c::State::CancellingGates { .. }
        | c::State::Ready { .. }
        | c::State::Preparing { .. }
        | c::State::Judging { .. }
        | c::State::Queued { .. }
        | c::State::First { .. }
        | c::State::Updating { .. }
        | c::State::WaitingProducer { .. }
        | c::State::Resolving { .. }
        | c::State::Repairing { .. }
        | c::State::Landing { .. }
        | c::State::Landed { .. }
        | c::State::Red { .. }
        | c::State::Cleared { .. }
        | c::State::Uncleared { .. }
        | c::State::Held { .. } => None,
    }
}

/// A terminal state or a root ending records its first decision, not cleanup's repetition.
pub(crate) fn ending(
    d: &mut Domain,
    env: &Env<Limits>,
    row: &mut ChangeRow,
    ending: h::Ending,
    out: &mut Queue<Request>,
) {
    if d.repository(row.repository).is_none() {
        return;
    }
    let event = step(d, row, h::What::Ended { ending }, h::Operation::Own);
    write(d, env, row, event, out);
    clear(row);
    persist(d, row, out);
}

fn persist(d: &mut Domain, row: &ChangeRow, out: &mut Queue<Request>) {
    d.changes.insert(row.task, row.clone()).expect("existing change");
    crate::domain::save_change(d, out, row.clone());
}

pub(crate) fn ending_kind(ending: &crate::Ending) -> Option<h::Ending> {
    match ending {
        crate::Ending::Landed => None,
        crate::Ending::Cancelled => Some(h::Ending::Cancelled),
        crate::Ending::NoChange => Some(h::Ending::NoChange),
        crate::Ending::Failed { .. } => Some(h::Ending::Failed),
    }
}

pub(crate) fn unavailable(
    d: &mut Domain,
    env: &Env<Limits>,
    row: &mut ChangeRow,
    why: h::Unready,
    out: &mut Queue<Request>,
) {
    if d.repository(row.repository).is_none() {
        return;
    }
    if let Some(number) = match row.history {
        Some(history) => history.ready,
        None => None,
    } {
        let mut event = step(d, row, h::What::Unready { why }, h::Operation::Of(number));
        event.head = row.change.last_head;
        write(d, env, row, event, out);
        row.history.as_mut().expect("written unready").ready = None;
        persist(d, row, out);
    }
}

fn clear(row: &mut ChangeRow) {
    if let Some(history) = &mut row.history {
        history.ready = None;
        history.queued = None;
        history.paused = None;
        history.held = None;
    }
}

fn line(row: &ChangeRow) -> bool {
    match row.change.state {
        c::State::Queued { .. }
        | c::State::First { .. }
        | c::State::Ready { .. }
        | c::State::Preparing { .. }
        | c::State::Judging { .. } => true,
        c::State::Updating { .. } | c::State::Landing { .. } | c::State::Checking { .. } | c::State::Gating { .. } => {
            row.change.owns_turn
        }
        c::State::Producing { .. }
        | c::State::Opening { .. }
        | c::State::Recreating { .. }
        | c::State::Reopening { .. }
        | c::State::CancellingGates { .. }
        | c::State::WaitingProducer { .. }
        | c::State::Resolving { .. }
        | c::State::Repairing { .. }
        | c::State::Landed { .. }
        | c::State::NoChange { .. }
        | c::State::Ended
        | c::State::Red { .. }
        | c::State::Cleared { .. }
        | c::State::Uncleared { .. }
        | c::State::Held { .. } => false,
    }
}

fn hold(state: &c::State) -> Option<c::Hold> {
    match state {
        c::State::Held { why, .. } => Some(*why),
        c::State::Producing { .. }
        | c::State::Opening { .. }
        | c::State::Recreating { .. }
        | c::State::Reopening { .. }
        | c::State::Checking { .. }
        | c::State::Gating { .. }
        | c::State::CancellingGates { .. }
        | c::State::Ready { .. }
        | c::State::Preparing { .. }
        | c::State::Judging { .. }
        | c::State::Queued { .. }
        | c::State::First { .. }
        | c::State::Updating { .. }
        | c::State::WaitingProducer { .. }
        | c::State::Resolving { .. }
        | c::State::Repairing { .. }
        | c::State::Landing { .. }
        | c::State::Landed { .. }
        | c::State::NoChange { .. }
        | c::State::Ended
        | c::State::Red { .. }
        | c::State::Cleared { .. }
        | c::State::Uncleared { .. } => None,
    }
}

fn first(state: &c::State) -> bool {
    match state {
        c::State::First { .. } => true,
        c::State::Producing { .. }
        | c::State::Opening { .. }
        | c::State::Recreating { .. }
        | c::State::Reopening { .. }
        | c::State::Checking { .. }
        | c::State::Gating { .. }
        | c::State::CancellingGates { .. }
        | c::State::Ready { .. }
        | c::State::Preparing { .. }
        | c::State::Judging { .. }
        | c::State::Queued { .. }
        | c::State::Held { .. }
        | c::State::Updating { .. }
        | c::State::WaitingProducer { .. }
        | c::State::Resolving { .. }
        | c::State::Repairing { .. }
        | c::State::Landing { .. }
        | c::State::Landed { .. }
        | c::State::NoChange { .. }
        | c::State::Ended
        | c::State::Red { .. }
        | c::State::Cleared { .. }
        | c::State::Uncleared { .. } => false,
    }
}

fn reason(why: c::Hold) -> Box<[u8]> {
    Box::from(match why {
        c::Hold::BranchMoved => b"branch_moved".as_slice(),
        c::Hold::BranchMissing => b"branch_missing".as_slice(),
        c::Hold::PullClosed => b"pull_closed".as_slice(),
        c::Hold::Retargeted => b"retargeted".as_slice(),
        c::Hold::Repairs => b"repairs".as_slice(),
        c::Hold::Resolutions => b"resolutions".as_slice(),
        c::Hold::Updates => b"updates".as_slice(),
        c::Hold::Rejected => b"rejected".as_slice(),
        c::Hold::Stalled => b"stalled".as_slice(),
        c::Hold::Failed => b"failed".as_slice(),
        c::Hold::Questions => b"questions".as_slice(),
        c::Hold::Authority(_) => b"authority".as_slice(),
    })
}

fn why(row: &ChangeRow, facts: Option<&c::Facts>, approval: bool) -> h::Unready {
    match hold(&row.change.state) {
        Some(c::Hold::BranchMoved | c::Hold::BranchMissing) => return h::Unready::HeadMoved,
        Some(c::Hold::PullClosed) => return h::Unready::Closed,
        Some(c::Hold::Rejected) => return h::Unready::Refused,
        Some(_) => return h::Unready::Held,
        None => {}
    }
    if let Some(facts) = facts {
        if facts.kept {
            return h::Unready::Dependency;
        }
        if facts.branch != row.change.last_head {
            return h::Unready::HeadMoved;
        }
    }
    if approval {
        return h::Unready::Approval;
    }
    match row.change.state {
        c::State::Resolving { .. } | c::State::WaitingProducer { .. } => h::Unready::Conflict,
        c::State::Producing { .. }
        | c::State::Opening { .. }
        | c::State::Recreating { .. }
        | c::State::Reopening { .. }
        | c::State::Checking { .. }
        | c::State::Gating { .. }
        | c::State::CancellingGates { .. }
        | c::State::Ready { .. }
        | c::State::Preparing { .. }
        | c::State::Judging { .. }
        | c::State::Queued { .. }
        | c::State::First { .. }
        | c::State::Updating { .. }
        | c::State::Repairing { .. }
        | c::State::Landing { .. }
        | c::State::Landed { .. }
        | c::State::NoChange { .. }
        | c::State::Ended
        | c::State::Red { .. }
        | c::State::Cleared { .. }
        | c::State::Uncleared { .. }
        | c::State::Held { .. } => h::Unready::Refused,
    }
}

/// Compare inside the policy loop: `First` need not be the decision's final state.
pub(crate) fn advanced(
    d: &mut Domain,
    env: &Env<Limits>,
    row: &mut ChangeRow,
    prior: &c::Change,
    facts: &c::Facts,
    out: &mut Queue<Request>,
) {
    if d.repository(row.repository).is_none() {
        return;
    }
    if row.ending.is_none()
        && ended(&prior.state).is_none()
        && let Some(ending_kind) = ended(&row.change.state)
    {
        ending(d, env, row, ending_kind, out);
    }
    if prior.last_head != row.change.last_head && !crate::domain::matches_updating(&prior.state) {
        unavailable(d, env, row, h::Unready::HeadMoved, out);
    }
    if let Some(why) = hold(&prior.state)
        && hold(&row.change.state).is_none()
    {
        released(d, env, row, why, out);
    }
    reconcile(d, env, row, Some(facts), out);
    let selected = first(&row.change.state) && !first(&prior.state) && !prior.owns_turn;
    if row.change.way == c::Way::Queue && selected {
        let mut event = step(d, row, h::What::Selected, h::Operation::Own);
        event.head = row.change.last_head;
        event.base = Some(facts.base_tip);
        write(d, env, row, event, out);
    }
    if row.change.way == c::Way::Queue && line(row) && facts.base_ci.status == c::Status::Failed {
        base_ci(d, env, row.task, facts.base_tip, facts.base_ci.status, out);
        row.history = d.changes.get(&row.task).expect("stepped change").history;
    }
}

pub(crate) fn released(d: &mut Domain, env: &Env<Limits>, row: &mut ChangeRow, why: c::Hold, out: &mut Queue<Request>) {
    if d.repository(row.repository).is_none() {
        return;
    }
    let held = match row.history {
        Some(history) => history.held,
        None => None,
    };
    if let Some(number) = held {
        let event = step(
            d,
            row,
            h::What::Released { on: h::On::Change, reason: reason(why), by: None },
            h::Operation::Of(number),
        );
        write(d, env, row, event, out);
        row.history.as_mut().expect("written release").held = None;
        persist(d, row, out);
    }
}

/// Review facts decide availability even when the policy state has not moved.
pub(crate) fn reviews(d: &mut Domain, env: &Env<Limits>, row: &mut ChangeRow, out: &mut Queue<Request>) {
    intervals(d, env, row, None, true, out);
}

/// Stored interval starts survive a restart, including availability not represented by a state.
pub(crate) fn reconcile(
    d: &mut Domain,
    env: &Env<Limits>,
    row: &mut ChangeRow,
    facts: Option<&c::Facts>,
    out: &mut Queue<Request>,
) {
    intervals(d, env, row, facts, false, out);
}

fn intervals(
    d: &mut Domain,
    env: &Env<Limits>,
    row: &mut ChangeRow,
    facts: Option<&c::Facts>,
    refresh: bool,
    out: &mut Queue<Request>,
) {
    if d.repository(row.repository).is_none() {
        return;
    }
    let before = row.history;
    if row.ending.is_some()
        || ended(&row.change.state).is_some()
        || crate::domain::landed_commit(&row.change.state).is_some()
    {
        clear(row);
        if before != row.history {
            persist(d, row, out);
        }
        return;
    }
    let approval = d.approval_short(row);
    let kept = match facts {
        Some(facts) => facts.kept,
        None => false,
    };
    let in_line = line(row);
    let ready = match row.history {
        Some(history) => history.ready,
        None => None,
    };
    let mut unread = false;
    for approval in &d.panel_approvals(row) {
        unread |= approval.read == crate::ReviewRead::Unread;
    }
    let known = (refresh || facts.is_some() || row.history.is_none() || ready.is_some())
        && (!unread || row.history.is_none() || ready.is_some());
    let open = in_line && !approval && !kept && known;
    if open && ready.is_none() {
        let mut event = step(d, row, h::What::Ready, h::Operation::Own);
        event.head = row.change.last_head;
        let number = write(d, env, row, event, out);
        row.history.as_mut().expect("written ready").ready = Some(number);
    } else if !open && let Some(number) = ready {
        let why = why(row, facts, approval);
        let mut event = step(d, row, h::What::Unready { why }, h::Operation::Of(number));
        event.head = row.change.last_head;
        write(d, env, row, event, out);
        row.history.as_mut().expect("written unready").ready = None;
    }
    if row.change.way == c::Way::Queue {
        queue(d, env, row, in_line, out);
    }
    let held = match row.history {
        Some(history) => history.held,
        None => None,
    };
    if let Some(why) = hold(&row.change.state) {
        if held.is_none() {
            let event =
                step(d, row, h::What::Held { on: h::On::Change, reason: reason(why), by: None }, h::Operation::Own);
            let number = write(d, env, row, event, out);
            row.history.as_mut().expect("written hold").held = Some(number);
        }
    } else if held.is_some()
        && let Some(why) = row.change.released
    {
        released(d, env, row, why, out);
    }
    if before != row.history {
        persist(d, row, out);
    }
}

fn queue(d: &mut Domain, env: &Env<Limits>, row: &mut ChangeRow, in_line: bool, out: &mut Queue<Request>) {
    let joined = match row.history {
        Some(history) => history.queued,
        None => None,
    };
    if in_line && joined.is_none() {
        let (ready, repair) = super::domain::queue_ready(d, row, &env.limits);
        let mut place = 1_u32;
        if let Some(own) = crate::panel::ready(row) {
            for other in &ready {
                if other.task != row.task && crate::panel::queue_before(other, &own, repair, &env.limits, env.wall) {
                    place = place.saturating_add(1);
                }
            }
        }
        let mut event = step(
            d,
            row,
            h::What::Joined { line: h::Line::Queue, place: Some(place), priority: Some(row.priority) },
            h::Operation::Own,
        );
        event.head = row.change.last_head;
        let number = write(d, env, row, event, out);
        row.history.as_mut().expect("written joined").queued = Some(number);
    } else if !in_line && let Some(number) = joined {
        let cause = match row.change.state {
            c::State::Resolving { .. } | c::State::WaitingProducer { .. } => h::Departure::Conflict,
            c::State::Repairing { .. } => h::Departure::Repair,
            c::State::Producing { .. }
            | c::State::Opening { .. }
            | c::State::Recreating { .. }
            | c::State::Reopening { .. }
            | c::State::Checking { .. }
            | c::State::Gating { .. }
            | c::State::CancellingGates { .. }
            | c::State::Ready { .. }
            | c::State::Preparing { .. }
            | c::State::Judging { .. }
            | c::State::Queued { .. }
            | c::State::First { .. }
            | c::State::Updating { .. }
            | c::State::Landing { .. }
            | c::State::Landed { .. }
            | c::State::NoChange { .. }
            | c::State::Ended
            | c::State::Red { .. }
            | c::State::Cleared { .. }
            | c::State::Uncleared { .. }
            | c::State::Held { .. } => h::Departure::Unready,
        };
        let mut event = step(d, row, h::What::Departed { line: h::Line::Queue, cause }, h::Operation::Of(number));
        event.head = row.change.last_head;
        write(d, env, row, event, out);
        row.history.as_mut().expect("written departed").queued = None;
        row.history.as_mut().expect("queue interval").paused = None;
    }
}

/// Repair admission is also a decision that the branch's queue is paused.
pub(crate) fn repair_started(d: &mut Domain, env: &Env<Limits>, owner: u64, out: &mut Queue<Request>) {
    let Some(row) = d.changes.get(&owner) else { return };
    let mut tip = None;
    for (name, head) in &d.heads {
        if name.forge == row.repository.forge
            && name.repository == row.repository.repository
            && crate::items::branch_matches(name, &row.base)
        {
            tip = Some(head.commit);
            break;
        }
    }
    if let Some(tip) = tip {
        base_ci(d, env, owner, tip, c::Status::Failed, out);
    }
}

/// One fresh verdict of the landing branch's tip applies to every waiting change.
pub(crate) fn base_ci(
    d: &mut Domain,
    env: &Env<Limits>,
    owner: u64,
    tip: skein_forge::api::Commit,
    status: c::Status,
    out: &mut Queue<Request>,
) {
    let Some(owner) = d.changes.get(&owner) else { return };
    if owner.change.way != c::Way::Queue || d.repository(owner.repository).is_none() {
        return;
    }
    let mut tasks = List::with_capacity(env.limits.changes);
    for (task, row) in &d.changes {
        if row.repository == owner.repository
            && row.base == owner.base
            && !row.base_repair
            && !row.terminal()
            && line(row)
        {
            tasks.push(*task).expect("bounded changes");
        }
    }
    for task in &tasks {
        let mut row = d.changes.get(task).expect("waiting change").clone();
        let paused = match row.history {
            Some(history) => history.paused,
            None => None,
        };
        let mut event = match status {
            c::Status::Failed if paused.is_none() => {
                step(d, &row, h::What::Paused { why: h::Pause::BranchRed }, h::Operation::Own)
            }
            c::Status::Passed if paused.is_some() => {
                step(d, &row, h::What::Resumed, h::Operation::Of(paused.expect("open pause")))
            }
            c::Status::Failed | c::Status::Passed | c::Status::Pending | c::Status::Unknown => continue,
        };
        event.base = Some(tip);
        let number = write(d, env, &mut row, event, out);
        row.history.as_mut().expect("written pause").paused =
            if status == c::Status::Failed { Some(number) } else { None };
        d.changes.insert(*task, row.clone()).expect("existing change");
        super::domain::save_change(d, out, row);
    }
}
