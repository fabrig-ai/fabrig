//! Party routes and the party-scope run admission stub.
//! `party-runs` fills the scope checks and deletion route
//! (`jig/docs/design/domain/engine.md`, 10.1). Until then, party scopes
//! and every run deletion are refused beyond authority.

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
#[expect(
    clippy::unnecessary_wraps,
    reason = "party-runs admits a write by returning no refusal; the stub refuses every party scope"
)]
pub(crate) fn authorize_note(_core: &Core, _key: CallKey, _party: u64) -> Option<CallPart> {
    Some(scope_denied())
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
