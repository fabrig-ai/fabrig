//! Party routes and run writes to their requester's preferences.
//! Admission holds the requester and asking family before a note is decided
//! and recorded (`jig/docs/design/domain/engine.md`, 10.1).

pub(crate) mod account_requests;
pub(crate) mod amendment_reads;
pub(crate) mod controls;
pub(crate) mod create;
pub(crate) mod decisions;
pub(crate) mod deployment_acts;
pub(crate) mod dispatch;
pub(crate) mod ended;
pub(crate) mod holdings;
pub(crate) mod inputs;
pub(crate) mod key_routes;
pub(crate) mod people;
pub(crate) mod projects;
pub(crate) mod proposal_revisions;
pub(crate) mod propose;

use crate::{
    CallKey, CallPart, Core, Event, Request,
    routing::{
        NoteRoute, Work,
        run::{Call, calls::call_decision},
    },
};
use alloc::boxed::Box;
use jig_domain_authority as authority;
use jig_domain_notes as notes;
use skein_lib::Queue;

/// Admission of a run's write to a party's scope, including its asking family.
pub(crate) fn authorize_note(core: &Core, key: CallKey, party: u64) -> Option<CallPart> {
    let Some(task) = core.tasks.task(key.task) else { return Some(scope_denied()) };
    if task.requester != jig_domain_tasks::Party::Person(party)
        || core.authorize_tool(key, crate::ToolKind::Ask).is_some()
    {
        return Some(scope_denied());
    }
    None
}

/// Whether a run may recall a party's scope.
pub(crate) fn note_readable(_core: &Core, _task: u64, _party: u64) -> bool {
    false
}

fn scope_denied() -> CallPart {
    CallPart::ToolDenied {
        answer: authority::Answer::Refuse,
        findings: Box::new([authority::Finding::Scope { source: authority::Source::Task }]),
    }
}

/// Route and admit a named deletion after checking the call's replay.
pub(crate) fn delete(
    core: &mut Core,
    work: &mut Queue<Work>,
    call: Call,
    scope: notes::Scope,
    name: u64,
    recalled: u32,
) -> Queue<Request> {
    let mut out = Queue::uncapped();
    let party = match &scope {
        notes::Scope::Party { party } => *party,
        notes::Scope::Deployment
        | notes::Scope::Project { .. }
        | notes::Scope::Goal { .. }
        | notes::Scope::Resources { .. } => {
            call_decision(core, &mut out, call.to, call.key, scope_denied());
            return out;
        }
    };
    if let Some(part) = authorize_note(core, call.key, party) {
        call_decision(core, &mut out, call.to, call.key, part);
        return out;
    }
    assert!(core.pending_calls.insert(call.key, true) == Ok(None), "note deletion call room reserved");
    let owner = core.begin_note_route(NoteRoute::Call { to: call.to, key: call.key });
    work.push(Work::Event(Event::Notes(notes::Event::Edit {
        owner,
        party,
        name,
        scope,
        change: notes::Change::Delete { recalled },
    })));
    out
}
