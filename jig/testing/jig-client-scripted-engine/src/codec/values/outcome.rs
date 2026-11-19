//! Wire projections of owner outcome facts (client/domain/testing.md, 4).
use super::super::{bytes, c, list, w};
use super::{wire_created_given, wire_proposal_key, wire_revision, wire_seen_through};
pub(in crate::codec) fn wire_proposal_choice(
    value: &c::ProposalChoice,
    _limits: &w::Limits,
) -> Result<w::ProposalChoice, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::ProposalChoice::Accepted => w::ProposalChoice::Accepted,
        c::ProposalChoice::Rejected => w::ProposalChoice::Rejected,
        c::ProposalChoice::Passed => w::ProposalChoice::Passed,
        c::ProposalChoice::Withdrawn => w::ProposalChoice::Withdrawn,
        c::ProposalChoice::Stale => w::ProposalChoice::Stale,
    })
}

#[expect(clippy::too_many_lines, reason = "the exhaustive owner outcome vocabulary is encoded together")]
pub(in crate::codec) fn wire_document_outcome(
    value: &c::DocumentOutcome,
    limits: &w::Limits,
) -> Result<w::DocumentOutcome, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::DocumentOutcome::Concluded { task } => w::DocumentOutcome::Concluded(w::OutcomeConcluded { task: *task }),

        c::DocumentOutcome::Left { task } => w::DocumentOutcome::Left(w::OutcomeStopped { task: *task }),
        c::DocumentOutcome::Lifted { hold, released } => {
            w::DocumentOutcome::Lifted(w::OutcomeLifted { hold: *(hold), released: *(released) })
        }
        c::DocumentOutcome::AccountFinishing { source } => {
            w::DocumentOutcome::AccountFinishing(w::OutcomeAccountFinishing { source: *(source) })
        }
        c::DocumentOutcome::Created { first, last, given } => w::DocumentOutcome::Created(w::OutcomeCreated {
            first: *(first),
            last: *(last),
            given: (given).as_ref().map(|item| wire_created_given(item, limits)).transpose()?,
        }),
        c::DocumentOutcome::Compound { project, operations, first_message } => {
            w::DocumentOutcome::Compound(w::OutcomeCompound {
                project: *(project),
                operations: *(operations),
                first_message: *(first_message),
            })
        }
        c::DocumentOutcome::MarkedSeen { seen } => {
            w::DocumentOutcome::MarkedSeen(w::OutcomeMarkedSeen { seen: wire_seen_through(seen, limits)? })
        }
        c::DocumentOutcome::NoteEdited { name } => {
            w::DocumentOutcome::NoteEdited(w::OutcomeNoteEdited { name: *(name) })
        }
        c::DocumentOutcome::ServiceMade { person } => {
            w::DocumentOutcome::ServiceMade(w::OutcomeServiceMade { person: *(person) })
        }
        c::DocumentOutcome::Adopted { project } => {
            w::DocumentOutcome::Adopted(w::OutcomeAdopted { project: *(project) })
        }
        c::DocumentOutcome::Stopped { task } => w::DocumentOutcome::Stopped(w::OutcomeStopped { task: *(task) }),
        c::DocumentOutcome::Cancelled { task } => w::DocumentOutcome::Cancelled(w::OutcomeCancelled { task: *(task) }),
        c::DocumentOutcome::Released { task } => w::DocumentOutcome::Released(w::OutcomeReleased { task: *(task) }),
        c::DocumentOutcome::PersonTaken { task } => {
            w::DocumentOutcome::PersonTaken(w::OutcomePersonTaken { task: *(task) })
        }
        c::DocumentOutcome::PersonHandedBack { task } => {
            w::DocumentOutcome::PersonHandedBack(w::OutcomePersonHandedBack { task: *(task) })
        }
        c::DocumentOutcome::PersonAnswered { task } => {
            w::DocumentOutcome::PersonAnswered(w::OutcomePersonAnswered { task: *(task) })
        }
        c::DocumentOutcome::Moved { task } => w::DocumentOutcome::Moved(w::OutcomeMoved { task: *(task) }),
        c::DocumentOutcome::ProposalDecided { proposal, by, at, choice, repeated } => {
            w::DocumentOutcome::ProposalDecided(w::OutcomeProposalDecided {
                proposal: wire_proposal_key(proposal, limits)?,
                by: *(by),
                at: (at).as_nanos(),
                choice: wire_proposal_choice(choice, limits)?,
                repeated: *(repeated),
            })
        }
        c::DocumentOutcome::Said { task, message } => {
            w::DocumentOutcome::Said(w::OutcomeSaid { task: *(task), message: *(message) })
        }
        c::DocumentOutcome::QuestionAnswered { task, question, message } => {
            w::DocumentOutcome::QuestionAnswered(w::OutcomeQuestionAnswered {
                task: *(task),
                question: *(question),
                message: *(message),
            })
        }
        c::DocumentOutcome::Amended { task } => w::DocumentOutcome::Amended(w::OutcomeAmended { task: *(task) }),
        c::DocumentOutcome::Proposed { proposal, to } => {
            w::DocumentOutcome::Proposed(w::OutcomeProposed { proposal: *(proposal), to: *(to) })
        }
        c::DocumentOutcome::Withdrawn { proposal } => {
            w::DocumentOutcome::Withdrawn(w::OutcomeWithdrawn { proposal: *(proposal) })
        }
        c::DocumentOutcome::RolesSet { project } => {
            w::DocumentOutcome::RolesSet(w::OutcomeRolesSet { project: *(project) })
        }
        c::DocumentOutcome::PolicyChanged { project } => {
            w::DocumentOutcome::PolicyChanged(w::OutcomePolicyChanged { project: *(project) })
        }
        c::DocumentOutcome::Renamed { project } => {
            w::DocumentOutcome::Renamed(w::OutcomeRenamed { project: *(project) })
        }
        c::DocumentOutcome::VisibilitySet { project, public } => {
            w::DocumentOutcome::VisibilitySet(w::OutcomeVisibilitySet { project: *project, public: *public })
        }
        c::DocumentOutcome::ProjectRemoved { project } => {
            w::DocumentOutcome::ProjectRemoved(w::OutcomeProjectRemoved { project: *(project) })
        }
        c::DocumentOutcome::ProjectMade { project } => {
            w::DocumentOutcome::ProjectMade(w::OutcomeProjectMade { project: *(project) })
        }
        c::DocumentOutcome::PoolSet { project, person } => {
            w::DocumentOutcome::PoolSet(w::OutcomePoolSet { project: *(project), person: *(person) })
        }
        c::DocumentOutcome::EscalationDecided { task, revision, by, at, choice } => {
            w::DocumentOutcome::EscalationDecided(w::OutcomeEscalationDecided {
                task: *(task),
                revision: wire_revision(revision, limits)?,
                by: *(by),
                at: (at).as_nanos(),
                choice: wire_escalation_choice(choice, limits)?,
            })
        }
        c::DocumentOutcome::AccountKeyKept { source } => {
            w::DocumentOutcome::AccountKeyKept(w::OutcomeAccountKeyKept { source: *(source) })
        }
        c::DocumentOutcome::AccountLoginAsked { number } => {
            w::DocumentOutcome::AccountLoginAsked(w::OutcomeAccountLoginAsked { number: *number })
        }
        c::DocumentOutcome::AccountProgramDeclared { source, name } => {
            w::DocumentOutcome::AccountProgramDeclared(w::OutcomeAccountProgramDeclared {
                source: *(source),
                name: bytes::copy_of(name),
            })
        }
        c::DocumentOutcome::AccountChanged => w::DocumentOutcome::AccountChanged,
        c::DocumentOutcome::AccountMovedIn { account, renewed } => {
            w::DocumentOutcome::AccountMovedIn(w::OutcomeAccountMovedIn { account: *(account), renewed: *(renewed) })
        }
        c::DocumentOutcome::AccountRemoved { confirmed, failed_hosts } => {
            w::DocumentOutcome::AccountRemoved(w::OutcomeAccountRemoved {
                confirmed: *(confirmed),
                failed_hosts: list((failed_hosts).iter().map(|item| wire_account_host_failure(item, limits)))?,
            })
        }
        c::DocumentOutcome::AccountSignInStarted { source, address } => {
            w::DocumentOutcome::AccountSignInStarted(w::OutcomeAccountSignInStarted {
                source: *(source),
                address: bytes::copy_of(address),
            })
        }
        c::DocumentOutcome::AccountSignedOut { confirmed, failed_hosts } => {
            w::DocumentOutcome::AccountSignedOut(w::OutcomeAccountSignedOut {
                confirmed: *(confirmed),
                failed_hosts: list((failed_hosts).iter().map(|item| wire_account_host_failure(item, limits)))?,
            })
        }
    })
}

pub(in crate::codec) fn wire_escalation_choice(
    value: &c::EscalationChoice,
    _limits: &w::Limits,
) -> Result<w::EscalationChoice, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::EscalationChoice::Released => w::EscalationChoice::Released,
        c::EscalationChoice::Rejected => w::EscalationChoice::Rejected,
        c::EscalationChoice::Passed => w::EscalationChoice::Passed,
    })
}

pub(in crate::codec) fn wire_account_host_failure(
    value: &c::AccountHostFailure,
    limits: &w::Limits,
) -> Result<w::AccountHostFailure, w::Problem> {
    Ok(w::AccountHostFailure { name: bytes::copy_of(&value.name), why: wire_account_host_why(&value.why, limits)? })
}

pub(in crate::codec) fn wire_account_host_why(
    value: &c::AccountHostWhy,
    _limits: &w::Limits,
) -> Result<w::AccountHostWhy, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::AccountHostWhy::NotInContact => w::AccountHostWhy::NotInContact,
        c::AccountHostWhy::Lost => w::AccountHostWhy::Lost,
        c::AccountHostWhy::Failed => w::AccountHostWhy::Failed,
    })
}
