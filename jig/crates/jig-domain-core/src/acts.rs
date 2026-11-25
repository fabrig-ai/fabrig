//! Party acts judged from current engine facts (`client/domain/contract/requests.md`, section 2, and `client/domain/contract/documents.md`, section 3
//! ; `domain/people/acts.md`, section 1).
//! No state, drafts, keys or document cache is retained. A request and a
//! document use the same judgments. Draft-dependent acts carry numerical and
//! listed bounds; the client compares its form with these engine facts.

pub use crate::routing::party::propose::policy_proposal_allowed;
pub use jig_domain_core_vocabulary::acts::*;

pub(crate) mod bounds;
mod objects;
mod requests;
mod standing;

use crate::{Core, Limits, documents};
use jig_domain_people as people;
pub(crate) use objects::{at_question_offer, escalation, mutable_phase, person, proposal, question, write, write_in};
pub(crate) use requests::request;
use skein_lib::Wall;
pub(crate) use standing::role;

/// The policy roles addressed for one proposal kind, or none when nobody decides it.
/// Reads only the authority child; also used by applications' settings admission walks
/// (`domain/tasks/proposals.md`, where proposals wait).
#[must_use]
pub fn policy_recipient(
    authority: &jig_domain_authority::Domain,
    project: u32,
    kind: jig_domain_authority::ProposalKind,
) -> Option<Recipient> {
    let policy = authority.policy(project)?;
    let mut roles = skein_lib::List::with_capacity(authority.limits().roles);
    for role in &policy.roles {
        if role.decides.allows(kind) {
            roles.push(role.number).expect("policy roles fit the admitted authority bound");
        }
    }
    if roles.is_empty() { None } else { Some(Recipient::Roles { project, roles }) }
}

/// Judge an act using the same current facts as its authenticated request.
#[must_use]
pub fn admit(
    core: &Core,
    limits: &Limits,
    party: u64,
    act: Act,
    object: &documents::Subject,
    decision: Wall,
) -> Admission {
    objects::admit(core, limits, party, act, object, decision)
}

fn allowed(value: Result<(), people::Refusal>) -> Admission {
    match value {
        Ok(()) => Admission::Allowed,
        Err(why) => Admission::Refused(why),
    }
}

/// Judge a note edit with the scope supplied by its owner-loaded entry.
pub(crate) fn note(
    core: &Core,
    party: u64,
    project: u32,
    scope: &people::NoteScope,
    act: Act,
) -> Result<(), people::Refusal> {
    match scope {
        people::NoteScope::Party { party: owner } => {
            return if project == people::DEPLOYMENT && crate::reads::party::scope_visible(core, party, *owner) {
                Ok(())
            } else {
                Err(people::Refusal::Unknown)
            };
        }
        people::NoteScope::Deployment
        | people::NoteScope::Project
        | people::NoteScope::Goal { .. }
        | people::NoteScope::Resources { .. } => {}
    }
    let held = core.people.role(party, project);
    role(core, project, held, act)?;
    if core.note_authorized(party, project, held, scope) {
        Ok(())
    } else {
        Err(people::Refusal::Authority { proposable: None, lacking: people::Lack::Shape })
    }
}
