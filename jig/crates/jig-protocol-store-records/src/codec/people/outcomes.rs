//! Permanent people outcomes and their refusal facts (protocol/store/records.md, 1).
use super::{domain, wire};
use skein_lib::{Token, Wall};
mod facts;
use facts::{
    decode_again, decode_escalation_choice, decode_proposal_choice, encode_again, encode_escalation_choice,
    encode_proposal_choice,
};
#[expect(clippy::too_many_lines, reason = "one exhaustive permanent outcome translation table")]
pub(super) fn encode_outcome(value: &domain::Outcome, limits: &wire::Limits) -> Result<wire::Outcome, wire::Problem> {
    Ok(match value {
        domain::Outcome::VisibilitySet { project, public } => {
            wire::Outcome::VisibilitySet(wire::OutcomeVisibilitySet { project: *project, public: *public })
        }
        domain::Outcome::Restored { proposal, to, left_out } => {
            let mut settings = skein_lib::List::with_capacity(limits.outcome_restored_left_out);
            for &setting in left_out {
                let setting = facts::encode_setting(setting, limits);
                if settings.push(setting).is_err() {
                    return Err(super::problem());
                }
            }
            wire::Outcome::Restored(wire::OutcomeRestored { proposal: *proposal, to: *to, left_out: settings })
        }
        domain::Outcome::ProjectRemoved { project } => {
            wire::Outcome::ProjectRemoved(wire::OutcomeProjectRemoved { project: *project })
        }
        domain::Outcome::MovedToNewest { newest, moved, stayed } => {
            wire::Outcome::MovedToNewest(wire::OutcomeMovedToNewest { newest: *newest, moved: *moved, stayed: *stayed })
        }
        domain::Outcome::SettingsChanged { revision } => {
            wire::Outcome::SettingsChanged(wire::OutcomeSettingsChanged { revision: *revision })
        }
        domain::Outcome::Compound { project, operations, first_message } => {
            wire::Outcome::Compound(wire::OutcomeCompound {
                project: *project,
                operations: *operations,
                first_message: *first_message,
            })
        }
        domain::Outcome::CompoundRefused { at, why } => {
            wire::Outcome::CompoundRefused(wire::OutcomeCompoundRefused { at: *at, why: encode_refusal(why, limits)? })
        }
        domain::Outcome::Created { first, last, given } => encode_created(*first, *last, *given, limits),
        domain::Outcome::MarkedSeen { seen } => wire::Outcome::MarkedSeen(marked_seen(seen, limits)),
        domain::Outcome::Applied { connector, token } => {
            wire::Outcome::Applied(wire::OutcomeApplied { connector: *connector, token: token.raw() })
        }
        domain::Outcome::Again(value) => wire::Outcome::Again(wire::OutcomeAgain { value: encode_again(*value) }),
        domain::Outcome::AccountKeyKept { source } => {
            wire::Outcome::AccountKeyKept(wire::OutcomeAccountKeyKept { source: *source })
        }
        domain::Outcome::AccountLoginAsked { number } => {
            wire::Outcome::AccountLoginAsked(wire::OutcomeAccountLoginAsked { number: *number })
        }
        domain::Outcome::AccountProgramDeclared { source, name } => {
            wire::Outcome::AccountProgramDeclared(wire::OutcomeAccountProgramDeclared {
                source: *source,
                name: name.clone(),
            })
        }
        domain::Outcome::AccountMovingOut { account, number, source } => {
            wire::Outcome::AccountMovingOut(wire::OutcomeAccountMovingOut {
                account: *account,
                number: *number,
                source: *source,
            })
        }
        domain::Outcome::AccountFinishing { source } => {
            wire::Outcome::AccountFinishing(wire::OutcomeAccountFinishing { source: *source })
        }
        domain::Outcome::AccountMovedIn { account, renewed } => {
            wire::Outcome::AccountMovedIn(wire::OutcomeAccountMovedIn { account: *account, renewed: *renewed })
        }
        domain::Outcome::AccountReading { count } => {
            wire::Outcome::AccountReading(wire::OutcomeAccountReading { count: *count })
        }
        domain::Outcome::WorkerPauseAsked { worker } => {
            wire::Outcome::WorkerPauseAsked(wire::OutcomeWorkerPauseAsked { worker: *worker })
        }
        domain::Outcome::AccountChanged => wire::Outcome::AccountChanged,
        domain::Outcome::AccountRemoved { confirmed, failed_hosts } => {
            wire::Outcome::AccountRemoved(wire::OutcomeAccountRemoved {
                confirmed: *confirmed,
                failed_hosts: super::account_hosts::encode(failed_hosts, limits)?,
            })
        }
        domain::Outcome::SignedOut => wire::Outcome::SignedOut,
        domain::Outcome::AccountSignInStarted { source, address } => account_sign_in_started(*source, address, limits),
        domain::Outcome::AccountSignedOut { confirmed, failed_hosts } => {
            wire::Outcome::AccountSignedOut(wire::OutcomeAccountSignedOut {
                confirmed: *confirmed,
                failed_hosts: super::account_hosts::encode(failed_hosts, limits)?,
            })
        }
        domain::Outcome::NoteEdited { name } => wire::Outcome::NoteEdited(wire::OutcomeNoteEdited { name: *name }),
        domain::Outcome::ServiceMade { person } => {
            wire::Outcome::ServiceMade(wire::OutcomeServiceMade { person: *person })
        }
        domain::Outcome::Adopted { project } => wire::Outcome::Adopted(wire::OutcomeAdopted { project: *project }),
        domain::Outcome::Proposed { proposal, to } => encode_proposed(*proposal, *to, limits),
        domain::Outcome::Withdrawn { proposal } => encode_withdrawn(*proposal, limits),
        domain::Outcome::Left { task } => wire::Outcome::Left(wire::OutcomeStopped { task: *task }),
        domain::Outcome::Stopped { task } => wire::Outcome::Stopped(wire::OutcomeStopped { task: *task }),
        domain::Outcome::Concluded { task } => wire::Outcome::Concluded(wire::OutcomeConcluded { task: *task }),
        domain::Outcome::Cancelled { task } => wire::Outcome::Cancelled(wire::OutcomeCancelled { task: *task }),
        domain::Outcome::Lifted { hold, released } => {
            wire::Outcome::Lifted(wire::OutcomeLifted { hold: *hold, released: *released })
        }
        domain::Outcome::Released { task } => wire::Outcome::Released(wire::OutcomeReleased { task: *task }),
        domain::Outcome::PersonTaken { task } => wire::Outcome::PersonTaken(wire::OutcomePersonTaken { task: *task }),
        domain::Outcome::PersonHandedBack { task } => {
            wire::Outcome::PersonHandedBack(wire::OutcomePersonHandedBack { task: *task })
        }
        domain::Outcome::PersonAnswered { task } => {
            wire::Outcome::PersonAnswered(wire::OutcomePersonAnswered { task: *task })
        }
        domain::Outcome::Moved { task } => wire::Outcome::Moved(wire::OutcomeMoved { task: *task }),
        domain::Outcome::ProposalDecided { proposer, proposal, by, at, choice, repeated } => {
            wire::Outcome::ProposalDecided(encode_proposal_decided(
                proposer, *proposal, *by, *at, *choice, *repeated, limits,
            ))
        }
        domain::Outcome::Said { task, message } => wire::Outcome::Said(encode_said(*task, *message, limits)),
        domain::Outcome::QuestionAnswered { task, question, message } => {
            wire::Outcome::QuestionAnswered(encode_question_answered(*task, *question, *message, limits))
        }
        domain::Outcome::Amended { task, released } => {
            wire::Outcome::Amended(wire::OutcomeAmended { task: *task, released: *released })
        }

        domain::Outcome::RolesSet { project } => wire::Outcome::RolesSet(wire::OutcomeRolesSet { project: *project }),
        domain::Outcome::Renamed { project } => wire::Outcome::Renamed(wire::OutcomeRenamed { project: *project }),
        domain::Outcome::ProjectMade { project } => {
            wire::Outcome::ProjectMade(wire::OutcomeProjectMade { project: *project })
        }
        domain::Outcome::PolicyChanged { project } => {
            wire::Outcome::PolicyChanged(wire::OutcomePolicyChanged { project: *project })
        }
        domain::Outcome::PoolSet { project, person } => {
            wire::Outcome::PoolSet(wire::OutcomePoolSet { project: *project, person: *person })
        }
        domain::Outcome::EscalationDecided { task, revision, by, at, choice } => {
            wire::Outcome::EscalationDecided(encode_escalation_decided(*task, *revision, *by, *at, *choice, limits))
        }
        domain::Outcome::Refused(value) => {
            wire::Outcome::Refused(wire::OutcomeRefused { value: encode_refusal(value, limits)? })
        }
    })
}
fn account_sign_in_started(source: u32, address: &[u8], _limits: &wire::Limits) -> wire::Outcome {
    let parts = wire::OutcomeAccountSignInStarted { source, address: skein_lib::bytes::copy_of(address) };
    wire::Outcome::AccountSignInStarted(parts)
}

#[expect(clippy::too_many_lines, reason = "one exhaustive permanent outcome translation table")]
pub(super) fn decode_outcome(value: wire::Outcome) -> domain::Outcome {
    match value {
        wire::Outcome::VisibilitySet(value) => {
            domain::Outcome::VisibilitySet { project: value.project, public: value.public }
        }
        wire::Outcome::ProjectRemoved(value) => domain::Outcome::ProjectRemoved { project: value.project },
        wire::Outcome::MovedToNewest(value) => {
            domain::Outcome::MovedToNewest { newest: value.newest, moved: value.moved, stayed: value.stayed }
        }
        wire::Outcome::SettingsChanged(value) => domain::Outcome::SettingsChanged { revision: value.revision },
        wire::Outcome::Compound(value) => {
            let wire::OutcomeCompound { project, operations, first_message } = value;
            domain::Outcome::Compound { project, operations, first_message }
        }
        wire::Outcome::CompoundRefused(value) => {
            let wire::OutcomeCompoundRefused { at, why } = value;
            domain::Outcome::CompoundRefused { at, why: decode_refusal(why) }
        }
        wire::Outcome::Created(value) => decode_created(value),
        wire::Outcome::MarkedSeen(value) => decode_marked_seen(value),
        wire::Outcome::Applied(value) => {
            let wire::OutcomeApplied { connector, token } = value;
            domain::Outcome::Applied { connector, token: Token::new(token) }
        }
        wire::Outcome::Again(value) => {
            let wire::OutcomeAgain { value } = value;
            domain::Outcome::Again(decode_again(value))
        }
        wire::Outcome::AccountKeyKept(value) => domain::Outcome::AccountKeyKept { source: value.source },
        wire::Outcome::AccountLoginAsked(value) => domain::Outcome::AccountLoginAsked { number: value.number },
        wire::Outcome::AccountProgramDeclared(value) => domain::Outcome::AccountProgramDeclared {
            source: value.source,
            name: skein_lib::bytes::copy_of(&value.name),
        },
        wire::Outcome::AccountMovingOut(value) => {
            let wire::OutcomeAccountMovingOut { account, number, source } = value;
            domain::Outcome::AccountMovingOut { account, number, source }
        }
        wire::Outcome::AccountFinishing(row) => domain::Outcome::AccountFinishing { source: row.source },
        wire::Outcome::AccountMovedIn(value) => {
            let wire::OutcomeAccountMovedIn { account, renewed } = value;
            domain::Outcome::AccountMovedIn { account, renewed }
        }
        wire::Outcome::AccountReading(value) => domain::Outcome::AccountReading { count: value.count },
        wire::Outcome::WorkerPauseAsked(value) => domain::Outcome::WorkerPauseAsked { worker: value.worker },
        wire::Outcome::AccountChanged => domain::Outcome::AccountChanged,
        wire::Outcome::AccountRemoved(value) => domain::Outcome::AccountRemoved {
            confirmed: value.confirmed,
            failed_hosts: super::account_hosts::decode(&value.failed_hosts),
        },
        wire::Outcome::SignedOut => domain::Outcome::SignedOut,
        wire::Outcome::AccountSignInStarted(value) => {
            let wire::OutcomeAccountSignInStarted { source, address } = value;
            domain::Outcome::AccountSignInStarted { source, address }
        }
        wire::Outcome::AccountSignedOut(value) => domain::Outcome::AccountSignedOut {
            confirmed: value.confirmed,
            failed_hosts: super::account_hosts::decode(&value.failed_hosts),
        },
        wire::Outcome::NoteEdited(value) => {
            let wire::OutcomeNoteEdited { name } = value;
            domain::Outcome::NoteEdited { name }
        }
        wire::Outcome::ServiceMade(value) => {
            let wire::OutcomeServiceMade { person } = value;
            domain::Outcome::ServiceMade { person }
        }
        wire::Outcome::Adopted(value) => {
            let wire::OutcomeAdopted { project } = value;
            domain::Outcome::Adopted { project }
        }
        wire::Outcome::Restored(value) => {
            let wire::OutcomeRestored { proposal, to, left_out } = value;
            let mut settings = skein_lib::List::with_capacity(left_out.len());
            for setting in &left_out {
                settings.push(facts::decode_setting(setting)).expect("decoded left-out settings count");
            }
            domain::Outcome::Restored { proposal, to, left_out: settings.into_boxed() }
        }
        wire::Outcome::Proposed(value) => {
            let wire::OutcomeProposed { proposal, to } = value;
            domain::Outcome::Proposed { proposal, to }
        }
        wire::Outcome::Withdrawn(value) => domain::Outcome::Withdrawn { proposal: value.proposal },
        wire::Outcome::Left(value) => domain::Outcome::Left { task: value.task },
        wire::Outcome::Stopped(value) => {
            let wire::OutcomeStopped { task } = value;
            domain::Outcome::Stopped { task }
        }
        wire::Outcome::Concluded(value) => domain::Outcome::Concluded { task: value.task },
        wire::Outcome::Cancelled(value) => {
            let wire::OutcomeCancelled { task } = value;
            domain::Outcome::Cancelled { task }
        }
        wire::Outcome::Lifted(value) => {
            let wire::OutcomeLifted { hold, released } = value;
            domain::Outcome::Lifted { hold, released }
        }
        wire::Outcome::Released(value) => {
            let wire::OutcomeReleased { task } = value;
            domain::Outcome::Released { task }
        }
        wire::Outcome::PersonTaken(value) => {
            let wire::OutcomePersonTaken { task } = value;
            domain::Outcome::PersonTaken { task }
        }
        wire::Outcome::PersonHandedBack(value) => {
            let wire::OutcomePersonHandedBack { task } = value;
            domain::Outcome::PersonHandedBack { task }
        }
        wire::Outcome::PersonAnswered(value) => {
            let wire::OutcomePersonAnswered { task } = value;
            domain::Outcome::PersonAnswered { task }
        }
        wire::Outcome::Moved(value) => {
            let wire::OutcomeMoved { task } = value;
            domain::Outcome::Moved { task }
        }
        wire::Outcome::ProposalDecided(value) => decode_proposal_decided(value),
        wire::Outcome::Said(value) => {
            let wire::OutcomeSaid { task, message } = value;
            domain::Outcome::Said { task, message }
        }
        wire::Outcome::QuestionAnswered(value) => decode_question_answered(value),
        wire::Outcome::Amended(value) => {
            let wire::OutcomeAmended { task, released } = value;
            domain::Outcome::Amended { task, released }
        }
        wire::Outcome::RolesSet(value) => {
            let wire::OutcomeRolesSet { project } = value;
            domain::Outcome::RolesSet { project }
        }
        wire::Outcome::Renamed(value) => domain::Outcome::Renamed { project: value.project },
        wire::Outcome::ProjectMade(value) => domain::Outcome::ProjectMade { project: value.project },
        wire::Outcome::PolicyChanged(value) => {
            let wire::OutcomePolicyChanged { project } = value;
            domain::Outcome::PolicyChanged { project }
        }
        wire::Outcome::PoolSet(value) => {
            let wire::OutcomePoolSet { project, person } = value;
            domain::Outcome::PoolSet { project, person }
        }
        wire::Outcome::EscalationDecided(value) => decode_escalation_decided(value),
        wire::Outcome::Refused(value) => {
            let wire::OutcomeRefused { value } = value;
            domain::Outcome::Refused(decode_refusal(value))
        }
    }
}
fn encode_proposal_decided(
    proposer: &domain::Proposer,
    proposal: u64,
    by: u64,
    at: Wall,
    choice: domain::ProposalChoice,
    repeated: bool,
    limits: &wire::Limits,
) -> wire::OutcomeProposalDecided {
    wire::OutcomeProposalDecided {
        proposer: super::asks::encode_proposer(proposer, limits),
        proposal,
        by,
        at: at.as_nanos(),
        choice: encode_proposal_choice(choice),
        repeated,
    }
}
fn decode_proposal_decided(value: wire::OutcomeProposalDecided) -> domain::Outcome {
    let wire::OutcomeProposalDecided { proposer, proposal, by, at, choice, repeated } = value;
    domain::Outcome::ProposalDecided {
        proposer: super::asks::decode_proposer(proposer),
        proposal,
        by,
        at: Wall::from_nanos(at),
        choice: decode_proposal_choice(choice),
        repeated,
    }
}
fn encode_escalation_decided(
    task: u64,
    revision: u64,
    by: u64,
    at: Wall,
    choice: domain::EscalationChoice,
    _limits: &wire::Limits,
) -> wire::OutcomeEscalationDecided {
    wire::OutcomeEscalationDecided { task, revision, by, at: at.as_nanos(), choice: encode_escalation_choice(choice) }
}
fn decode_escalation_decided(value: wire::OutcomeEscalationDecided) -> domain::Outcome {
    let wire::OutcomeEscalationDecided { task, revision, by, at, choice } = value;
    domain::Outcome::EscalationDecided {
        task,
        revision,
        by,
        at: Wall::from_nanos(at),
        choice: decode_escalation_choice(choice),
    }
}
fn encode_question_answered(
    task: u64,
    question: u64,
    message: u64,
    _limits: &wire::Limits,
) -> wire::OutcomeQuestionAnswered {
    wire::OutcomeQuestionAnswered { task, question, message }
}
fn decode_question_answered(value: wire::OutcomeQuestionAnswered) -> domain::Outcome {
    let wire::OutcomeQuestionAnswered { task, question, message } = value;
    domain::Outcome::QuestionAnswered { task, question, message }
}
fn encode_said(task: u64, message: u64, _limits: &wire::Limits) -> wire::OutcomeSaid {
    wire::OutcomeSaid { task, message }
}

fn marked_seen(seen: &domain::Seen, limits: &wire::Limits) -> wire::OutcomeMarkedSeen {
    wire::OutcomeMarkedSeen { seen: super::asks::encode_seen(seen, limits) }
}

fn encode_created(first: u64, last: u64, given: Option<domain::Given>, _limits: &wire::Limits) -> wire::Outcome {
    wire::Outcome::Created(wire::OutcomeCreated {
        first,
        last,
        given: match given {
            Some(domain::Given { task, amount }) => Some(wire::Given { task, amount }),
            None => None,
        },
    })
}
fn decode_created(value: wire::OutcomeCreated) -> domain::Outcome {
    let wire::OutcomeCreated { first, last, given } = value;
    let mut amount = None;
    if let Some(value) = given {
        let wire::Given { task, amount: value } = value;
        amount = Some(domain::Given { task, amount: value });
    }
    domain::Outcome::Created { first, last, given: amount }
}

fn decode_marked_seen(value: wire::OutcomeMarkedSeen) -> domain::Outcome {
    let wire::OutcomeMarkedSeen { seen } = value;
    domain::Outcome::MarkedSeen { seen: super::asks::decode_seen(seen) }
}

fn encode_proposed(proposal: u64, to: u32, _limits: &wire::Limits) -> wire::Outcome {
    wire::Outcome::Proposed(wire::OutcomeProposed { proposal, to })
}
fn encode_withdrawn(proposal: u64, _limits: &wire::Limits) -> wire::Outcome {
    wire::Outcome::Withdrawn(wire::OutcomeWithdrawn { proposal })
}

pub(crate) use facts::{decode_refusal, encode_refusal};
