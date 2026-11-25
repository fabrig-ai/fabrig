//! Field-for-field client contract translation (protocol/client/wire.md, 1, and protocol/client/README.md, 3).
use crate::Limits;
use crate::generated::client;
use crate::translate::{classify, text_problem};
use jig_client_domain::contract as c;
use jig_client_protocol_wire as w;
use jig_client_protocol_wire::Record as _;
use skein_lib::{List, Reader, Wall, bytes, length::len_u64};
pub(crate) fn seen(value: &w::SeenThrough) -> c::SeenThrough {
    client::seen_through_to_domain(value.clone())
}

pub(crate) fn proposal_key(value: &w::ProposalKey) -> c::ProposalKey {
    client::proposal_key_to_domain(value.clone())
}

pub(crate) fn proposal_choice(value: &w::ProposalChoice) -> c::ProposalChoice {
    client::proposal_choice_to_domain(value.clone())
}

pub(crate) fn revision(value: &w::Revision) -> c::Revision {
    client::revision_to_domain(value.clone())
}

pub(crate) fn escalation_choice(value: &w::EscalationChoice) -> c::EscalationChoice {
    client::escalation_choice_to_domain(value.clone())
}

pub(crate) fn document_outcome(value: &w::DocumentOutcome, limits: &Limits) -> Result<c::DocumentOutcome, w::Problem> {
    Ok(match value {
        w::DocumentOutcome::VisibilitySet(row) => {
            c::DocumentOutcome::VisibilitySet { project: row.project, public: row.public }
        }
        w::DocumentOutcome::ProjectRemoved(row) => c::DocumentOutcome::ProjectRemoved { project: row.project },
        w::DocumentOutcome::Created(row) => c::DocumentOutcome::Created {
            first: row.first,
            last: row.last,
            given: match &row.given {
                Some(v) => Some(client::created_given_to_domain(v.clone())),
                None => None,
            },
        },
        w::DocumentOutcome::Compound(row) => c::DocumentOutcome::Compound {
            project: row.project,
            operations: row.operations,
            first_message: row.first_message,
        },
        w::DocumentOutcome::MarkedSeen(row) => c::DocumentOutcome::MarkedSeen { seen: seen(&row.seen) },
        w::DocumentOutcome::NoteEdited(row) => c::DocumentOutcome::NoteEdited { name: row.name },
        w::DocumentOutcome::ServiceMade(row) => c::DocumentOutcome::ServiceMade { person: row.person },
        w::DocumentOutcome::Adopted(row) => c::DocumentOutcome::Adopted { project: row.project },
        w::DocumentOutcome::Left(row) => c::DocumentOutcome::Left { task: row.task },
        w::DocumentOutcome::Stopped(row) => c::DocumentOutcome::Stopped { task: row.task },
        w::DocumentOutcome::Concluded(row) => c::DocumentOutcome::Concluded { task: row.task },
        w::DocumentOutcome::Cancelled(row) => c::DocumentOutcome::Cancelled { task: row.task },
        w::DocumentOutcome::Lifted(row) => c::DocumentOutcome::Lifted { hold: row.hold, released: row.released },
        w::DocumentOutcome::Released(row) => c::DocumentOutcome::Released { task: row.task },
        w::DocumentOutcome::PersonTaken(row) => c::DocumentOutcome::PersonTaken { task: row.task },
        w::DocumentOutcome::PersonHandedBack(row) => c::DocumentOutcome::PersonHandedBack { task: row.task },
        w::DocumentOutcome::PersonAnswered(row) => c::DocumentOutcome::PersonAnswered { task: row.task },
        w::DocumentOutcome::Moved(row) => c::DocumentOutcome::Moved { task: row.task },
        w::DocumentOutcome::ProposalDecided(row) => c::DocumentOutcome::ProposalDecided {
            proposal: proposal_key(&row.proposal),
            by: row.by,
            at: Wall::from_nanos(row.at),
            choice: proposal_choice(&row.choice),
            repeated: row.repeated,
        },
        w::DocumentOutcome::Said(row) => c::DocumentOutcome::Said { task: row.task, message: row.message },
        w::DocumentOutcome::QuestionAnswered(row) => {
            c::DocumentOutcome::QuestionAnswered { task: row.task, question: row.question, message: row.message }
        }
        w::DocumentOutcome::Amended(row) => c::DocumentOutcome::Amended { task: row.task },
        w::DocumentOutcome::Proposed(row) => c::DocumentOutcome::Proposed { proposal: row.proposal, to: row.to },
        w::DocumentOutcome::Withdrawn(row) => c::DocumentOutcome::Withdrawn { proposal: row.proposal },
        w::DocumentOutcome::RolesSet(row) => c::DocumentOutcome::RolesSet { project: row.project },
        w::DocumentOutcome::Renamed(row) => c::DocumentOutcome::Renamed { project: row.project },
        w::DocumentOutcome::ProjectMade(row) => c::DocumentOutcome::ProjectMade { project: row.project },
        w::DocumentOutcome::PolicyChanged(row) => c::DocumentOutcome::PolicyChanged { project: row.project },
        w::DocumentOutcome::AccountSignInStarted(row) => {
            c::DocumentOutcome::AccountSignInStarted { source: row.source, address: bytes::copy_of(&row.address) }
        }
        w::DocumentOutcome::AccountKeyKept(row) => c::DocumentOutcome::AccountKeyKept { source: row.source },
        w::DocumentOutcome::AccountLoginAsked(row) => c::DocumentOutcome::AccountLoginAsked { number: row.number },
        w::DocumentOutcome::AccountProgramDeclared(row) => {
            if !c::valid_program_name(&row.name) {
                return Err(super::piece_problem(skein_codec::Reason::Malformed));
            }
            c::DocumentOutcome::AccountProgramDeclared { source: row.source, name: bytes::copy_of(&row.name) }
        }
        w::DocumentOutcome::AccountFinishing(row) => c::DocumentOutcome::AccountFinishing { source: row.source },
        w::DocumentOutcome::AccountMovedIn(row) => {
            c::DocumentOutcome::AccountMovedIn { account: row.account, renewed: row.renewed }
        }
        w::DocumentOutcome::AccountChanged => c::DocumentOutcome::AccountChanged,
        w::DocumentOutcome::AccountRemoved(row) => c::DocumentOutcome::AccountRemoved {
            confirmed: row.confirmed,
            failed_hosts: super::account_hosts::decode(&row.failed_hosts, row.confirmed, limits)?,
        },
        w::DocumentOutcome::AccountSignedOut(row) => c::DocumentOutcome::AccountSignedOut {
            confirmed: row.confirmed,
            failed_hosts: super::account_hosts::decode(&row.failed_hosts, row.confirmed, limits)?,
        },
        w::DocumentOutcome::PoolSet(row) => c::DocumentOutcome::PoolSet { project: row.project, person: row.person },
        w::DocumentOutcome::EscalationDecided(row) => c::DocumentOutcome::EscalationDecided {
            task: row.task,
            revision: revision(&row.revision),
            by: row.by,
            at: Wall::from_nanos(row.at),
            choice: escalation_choice(&row.choice),
        },
    })
}

pub(crate) fn pattern(value: &w::Pattern) -> c::Pattern {
    client::pattern_to_domain(value.clone())
}

pub(crate) fn note_scope(value: &w::NoteScope) -> Result<c::NoteScope, w::Problem> {
    Ok(match value {
        w::NoteScope::Tasks(_) => return Err(super::piece_problem(skein_codec::Reason::Malformed)),
        w::NoteScope::Party(row) => c::NoteScope::Party { party: row.party },
        w::NoteScope::Deployment => c::NoteScope::Deployment,
        w::NoteScope::Project(row) => c::NoteScope::Project(row.value),
        w::NoteScope::Resource(row) => {
            c::NoteScope::Resource { project: row.project, connector: row.connector, pattern: pattern(&row.pattern) }
        }
        w::NoteScope::Task(row) => c::NoteScope::Task { project: row.project, task: row.task },
    })
}

pub(crate) fn made(value: &w::Made) -> Result<c::Made, w::Problem> {
    Ok(match value {
        w::Made::Tasks(row) => {
            let mut list0 = List::with_capacity(row.tasks.len());
            for item in &row.tasks {
                list0.push(*item).expect("reserved supplied list");
            }
            c::Made::Tasks { tasks: list0 }
        }
        w::Made::MovingOut(row) => {
            c::Made::MovingOut { account: row.account, number: row.number, record: row.record.clone() }
        }
        w::Made::Service(row) => c::Made::Service { party: row.party },
        w::Made::Reissued(row) => c::Made::Reissued { party: row.party },
        w::Made::PersonResult(row) => c::Made::PersonResult { task: row.task },
        w::Made::Note(row) => c::Made::Note { note: row.note, scope: note_scope(&row.scope)? },
    })
}

pub(crate) fn decision(value: &w::Decision, limits: &Limits) -> Result<c::Decision, w::Problem> {
    Ok(match value {
        w::Decision::Accepted => c::Decision::Accepted,
        w::Decision::Rejected(row) => c::Decision::Rejected {
            reason: match c::Text::new(bytes::copy_of(&row.reason), limits.client().text) {
                Ok(value) => value,
                Err(_) => return Err(text_problem()),
            },
        },
        w::Decision::PassedUp => c::Decision::PassedUp,
        w::Decision::Released(row) => {
            let opt0 = match &row.again {
                Some(item) => Some(*item),
                None => None,
            };
            c::Decision::Released { again: opt0 }
        }
        w::Decision::LeftHeld(row) => c::Decision::LeftHeld {
            reason: match c::Text::new(bytes::copy_of(&row.reason), limits.client().text) {
                Ok(value) => value,
                Err(_) => return Err(text_problem()),
            },
        },
    })
}

pub(crate) fn held_role(value: &w::HeldRole) -> c::HeldRole {
    client::held_role_to_domain(value.clone())
}

pub(crate) fn placed(value: Option<&w::PlacedLine>, limits: &Limits) -> Result<Option<c::PlacedHold>, w::Problem> {
    let Some(value) = value else { return Ok(None) };
    if value.reason.len() > skein_lib::length::usize_of(limits.client().text) {
        return Err(super::too_large());
    }
    Ok(Some(client::placed_line_to_domain(value.clone())))
}
pub(crate) fn lacking(value: &w::Lacking) -> c::Lacking {
    client::lacking_to_domain(value.clone())
}

pub(crate) fn limit(value: &w::Limit) -> c::Limit {
    client::limit_to_domain(value.clone())
}

pub(crate) fn moved(value: &w::Moved, limits: &Limits) -> Result<c::Moved, w::Problem> {
    client::moved_to_domain(value.clone(), limits)
}

fn ending(value: &w::Ending, limits: &Limits) -> Result<c::Ending, w::Problem> {
    super::facts::ending(value, limits)
}

pub(crate) fn refusal(value: &w::Refusal, limits: &Limits) -> Result<c::Refusal, w::Problem> {
    Ok(match value {
        w::Refusal::Role(row) => {
            let opt0 = match &(&row.held) {
                Some(item) => Some(held_role(item)),
                None => None,
            };
            c::Refusal::Role { held: opt0 }
        }
        w::Refusal::Taken(row) => c::Refusal::Taken { by: row.by },
        w::Refusal::LiveTasks(value) => c::Refusal::LiveTasks { tasks: value.tasks.clone(), more: value.more },
        w::Refusal::Circle(value) => c::Refusal::Circle { tasks: value.tasks.clone(), cut: value.cut },
        w::Refusal::Kept(value) => c::Refusal::Kept { pending: value.pending.clone() },
        w::Refusal::Unlinked(value) => c::Refusal::Unlinked { worker: value.worker },
        w::Refusal::Concluded(value) => c::Refusal::Concluded { at: Wall::from_nanos(value.at) },
        w::Refusal::Dependency(value) => c::Refusal::Dependency {
            dependency: value.dependency,
            how: client::task_status_to_domain(value.how.clone()),
        },
        w::Refusal::Forgotten(value) => restore_refusal_forgotten(value),
        w::Refusal::Stale(value) => c::Refusal::Stale(super::settings::restore_refusal_stale(value)),
        w::Refusal::Full(value) => c::Refusal::Full(super::settings::restore_refusal_full(value)),
        w::Refusal::NotWaiting => c::Refusal::NotWaiting,
        w::Refusal::NoFurther => c::Refusal::NoFurther,
        w::Refusal::Authority(row) => {
            let opt0 = match &row.proposable {
                Some(item) => Some(*item),
                None => None,
            };
            c::Refusal::Authority { lacking: lacking(&row.lacking), proposable: opt0 }
        }
        w::Refusal::Limit(row) => c::Refusal::Limit { which: limit(&row.which) },
        w::Refusal::Moved(row) => c::Refusal::Moved { now: revision(&row.now), what: moved(&row.what, limits)? },
        w::Refusal::Ended(row) => c::Refusal::Ended { how: ending(&row.how, limits)? },
        w::Refusal::Malformed(row) => {
            c::Refusal::Malformed { at: row.at, why: client::malformed_to_domain(row.why.clone()) }
        }
        w::Refusal::SignInRedirect(row) => {
            c::Refusal::SignInRedirect { why: client::sign_in_redirect_why_to_domain(row.why.clone()) }
        }
        w::Refusal::AccountMove(row) => {
            c::Refusal::AccountMove { why: client::account_move_why_to_domain(row.why.clone()) }
        }
        w::Refusal::Account(row) => {
            c::Refusal::Account { account: row.account, why: client::account_why_to_domain(row.why.clone()) }
        }
        w::Refusal::Provider => c::Refusal::Provider,
        w::Refusal::Unknown => c::Refusal::Unknown,
        w::Refusal::NoEscalationHolder => c::Refusal::NoEscalationHolder,
        w::Refusal::Llm(row) => super::llm::refusal(row, limits)?,
        w::Refusal::Judged(row) => c::Refusal::Judged {
            at: row.at,
            setting: super::settings::restore_setting(&row.setting),
            why: match c::Text::new(bytes::copy_of(&row.why), limits.client().text) {
                Ok(v) => v,
                Err(_) => return Err(super::too_large()),
            },
        },
        w::Refusal::NameTaken => c::Refusal::NameTaken,
        w::Refusal::NotOffered => c::Refusal::NotOffered,
        w::Refusal::NoTerminal => c::Refusal::NoTerminal,
        w::Refusal::LoginUnderWay => c::Refusal::LoginUnderWay,
        w::Refusal::NotLending => c::Refusal::NotLending,
        w::Refusal::ProgramNameReserved => c::Refusal::ProgramNameReserved,
        w::Refusal::KeyConflict => c::Refusal::KeyConflict,
    })
}
/// Takes the shell-only credential beside the answer for a made or reissued service.
pub(crate) fn credential(record: &w::Owned, limits: &Limits) -> Option<(u64, skein_lib::Secret)> {
    if record.owner != w::Owner::Jig || record.version != jig_client_protocol_wire::VERSION {
        return None;
    }
    let w::Final::Done(value) = w::Final::decode(limits.wire(), &mut Reader::new(&record.record)).ok()? else {
        return None;
    };
    let w::Outcome::Made(value) = value.outcome else { return None };
    let parts = match value.what {
        w::Made::Service(service) | w::Made::Reissued(service) => service,
        w::Made::Tasks(_) | w::Made::PersonResult(_) | w::Made::Note(_) | w::Made::MovingOut(_) => return None,
    };
    if parts.credential.expose().is_empty() {
        return None;
    }
    Some((parts.party, parts.credential))
}
/// Decodes one final jig answer in its owner's vocabulary.
pub fn answer(record: w::KeyedAnswer, limits: &Limits) -> Result<c::Answer, super::Undecodable> {
    let value = match record {
        w::KeyedAnswer::FinalAnswer(value) => value,
        w::KeyedAnswer::NotServed(_)
        | w::KeyedAnswer::Busy
        | w::KeyedAnswer::NotReady
        | w::KeyedAnswer::AnswersFull(_)
        | w::KeyedAnswer::SignedOut => {
            return Err(super::Undecodable::Malformed);
        }
    };
    if value.owner != w::Owner::Jig || value.version != jig_client_protocol_wire::VERSION {
        return Err(super::Undecodable::Version);
    }
    if len_u64(value.record.len()) > limits.client().document_bytes {
        return Err(super::Undecodable::TooLarge);
    }
    let value = match w::Final::decode(limits.wire(), &mut Reader::new(&value.record)) {
        Ok(value) => value,
        Err(problem) => return Err(classify(problem)),
    };
    match client::final_to_domain(value, limits) {
        Ok(value) => Ok(value),
        Err(problem) => Err(classify(problem)),
    }
}

#[must_use]
pub fn restore_refusal_forgotten(value: &w::RefusalForgotten) -> c::Refusal {
    c::Refusal::Forgotten { oldest: value.oldest }
}

#[cfg(test)]
pub(crate) fn admission_limit(value: &w::AdmissionLimit) -> c::AdmissionLimit {
    client::admission_limit_to_domain(value.clone())
}

#[cfg(test)]
pub(crate) fn outcome(value: &w::Outcome, limits: &Limits) -> Result<c::Outcome, w::Problem> {
    client::outcome_to_domain(value.clone(), limits)
}
