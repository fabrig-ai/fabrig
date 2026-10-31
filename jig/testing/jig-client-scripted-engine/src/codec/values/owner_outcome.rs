//! Wire projections of owner outcome facts (client/domain/testing.md, 4).
use super::super::{bytes, c, list, w};
use super::{wire_created_given, wire_proposal_key, wire_revision, wire_seen_through};
pub(in crate::codec) fn wire_owner_proposal_choice(
    value: &c::OwnerProposalChoice,
    _limits: &w::Limits,
) -> Result<w::OwnerProposalChoice, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerProposalChoice::Accepted => w::OwnerProposalChoice::Accepted,
        c::OwnerProposalChoice::Rejected => w::OwnerProposalChoice::Rejected,
        c::OwnerProposalChoice::Passed => w::OwnerProposalChoice::Passed,
        c::OwnerProposalChoice::Withdrawn => w::OwnerProposalChoice::Withdrawn,
        c::OwnerProposalChoice::Stale => w::OwnerProposalChoice::Stale,
    })
}

#[expect(clippy::too_many_lines, reason = "the exhaustive owner outcome vocabulary is encoded together")]
pub(in crate::codec) fn wire_owner_outcome(
    value: &c::OwnerOutcome,
    limits: &w::Limits,
) -> Result<w::OwnerOutcome, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerOutcome::Lifted { hold, released } => w::OwnerOutcome::Lifted(w::OwnerOutcomeLifted::new(
            limits,
            w::OwnerOutcomeLiftedParts { hold: *(hold), released: *(released) },
        )?),
        c::OwnerOutcome::AccountFinishing { source } => w::OwnerOutcome::AccountFinishing(
            w::OwnerOutcomeAccountFinishing::new(limits, w::OwnerOutcomeAccountFinishingParts { source: *(source) })?,
        ),
        c::OwnerOutcome::Created { first, last, given } => w::OwnerOutcome::Created(w::OwnerOutcomeCreated::new(
            limits,
            w::OwnerOutcomeCreatedParts {
                first: *(first),
                last: *(last),
                given: (given).as_ref().map(|item| wire_created_given(item, limits)).transpose()?,
            },
        )?),
        c::OwnerOutcome::Compound { project, operations, first_message } => {
            w::OwnerOutcome::Compound(w::OwnerOutcomeCompound::new(
                limits,
                w::OwnerOutcomeCompoundParts {
                    project: *(project),
                    operations: *(operations),
                    first_message: *(first_message),
                },
            )?)
        }
        c::OwnerOutcome::MarkedSeen { seen } => w::OwnerOutcome::MarkedSeen(w::OwnerOutcomeMarkedSeen::new(
            limits,
            w::OwnerOutcomeMarkedSeenParts { seen: wire_seen_through(seen, limits)? },
        )?),
        c::OwnerOutcome::NoteEdited { name } => w::OwnerOutcome::NoteEdited(w::OwnerOutcomeNoteEdited::new(
            limits,
            w::OwnerOutcomeNoteEditedParts { name: *(name) },
        )?),
        c::OwnerOutcome::ServiceMade { person } => w::OwnerOutcome::ServiceMade(w::OwnerOutcomeServiceMade::new(
            limits,
            w::OwnerOutcomeServiceMadeParts { person: *(person) },
        )?),
        c::OwnerOutcome::Adopted { project } => w::OwnerOutcome::Adopted(w::OwnerOutcomeAdopted::new(
            limits,
            w::OwnerOutcomeAdoptedParts { project: *(project) },
        )?),
        c::OwnerOutcome::Stopped { task } => w::OwnerOutcome::Stopped(w::OwnerOutcomeStopped::new(
            limits,
            w::OwnerOutcomeStoppedParts { task: *(task) },
        )?),
        c::OwnerOutcome::Cancelled { task } => w::OwnerOutcome::Cancelled(w::OwnerOutcomeCancelled::new(
            limits,
            w::OwnerOutcomeCancelledParts { task: *(task) },
        )?),
        c::OwnerOutcome::Released { task } => w::OwnerOutcome::Released(w::OwnerOutcomeReleased::new(
            limits,
            w::OwnerOutcomeReleasedParts { task: *(task) },
        )?),
        c::OwnerOutcome::PersonTaken { task } => w::OwnerOutcome::PersonTaken(w::OwnerOutcomePersonTaken::new(
            limits,
            w::OwnerOutcomePersonTakenParts { task: *(task) },
        )?),
        c::OwnerOutcome::PersonHandedBack { task } => w::OwnerOutcome::PersonHandedBack(
            w::OwnerOutcomePersonHandedBack::new(limits, w::OwnerOutcomePersonHandedBackParts { task: *(task) })?,
        ),
        c::OwnerOutcome::PersonAnswered { task } => w::OwnerOutcome::PersonAnswered(
            w::OwnerOutcomePersonAnswered::new(limits, w::OwnerOutcomePersonAnsweredParts { task: *(task) })?,
        ),
        c::OwnerOutcome::Moved { task } => {
            w::OwnerOutcome::Moved(w::OwnerOutcomeMoved::new(limits, w::OwnerOutcomeMovedParts { task: *(task) })?)
        }
        c::OwnerOutcome::ProposalDecided { proposal, by, at, choice, repeated } => {
            w::OwnerOutcome::ProposalDecided(w::OwnerOutcomeProposalDecided::new(
                limits,
                w::OwnerOutcomeProposalDecidedParts {
                    proposal: wire_proposal_key(proposal, limits)?,
                    by: *(by),
                    at: (at).as_nanos(),
                    choice: wire_owner_proposal_choice(choice, limits)?,
                    repeated: *(repeated),
                },
            )?)
        }
        c::OwnerOutcome::Said { task, message } => w::OwnerOutcome::Said(w::OwnerOutcomeSaid::new(
            limits,
            w::OwnerOutcomeSaidParts { task: *(task), message: *(message) },
        )?),
        c::OwnerOutcome::QuestionAnswered { task, question, message } => {
            w::OwnerOutcome::QuestionAnswered(w::OwnerOutcomeQuestionAnswered::new(
                limits,
                w::OwnerOutcomeQuestionAnsweredParts { task: *(task), question: *(question), message: *(message) },
            )?)
        }
        c::OwnerOutcome::Amended { task } => w::OwnerOutcome::Amended(w::OwnerOutcomeAmended::new(
            limits,
            w::OwnerOutcomeAmendedParts { task: *(task) },
        )?),
        c::OwnerOutcome::Proposed { proposal, to } => w::OwnerOutcome::Proposed(w::OwnerOutcomeProposed::new(
            limits,
            w::OwnerOutcomeProposedParts { proposal: *(proposal), to: *(to) },
        )?),
        c::OwnerOutcome::Withdrawn { proposal } => w::OwnerOutcome::Withdrawn(w::OwnerOutcomeWithdrawn::new(
            limits,
            w::OwnerOutcomeWithdrawnParts { proposal: *(proposal) },
        )?),
        c::OwnerOutcome::RolesSet { project } => w::OwnerOutcome::RolesSet(w::OwnerOutcomeRolesSet::new(
            limits,
            w::OwnerOutcomeRolesSetParts { project: *(project) },
        )?),
        c::OwnerOutcome::PolicyChanged { project } => w::OwnerOutcome::PolicyChanged(
            w::OwnerOutcomePolicyChanged::new(limits, w::OwnerOutcomePolicyChangedParts { project: *(project) })?,
        ),
        c::OwnerOutcome::Renamed { project } => w::OwnerOutcome::Renamed(w::OwnerOutcomeRenamed::new(
            limits,
            w::OwnerOutcomeRenamedParts { project: *(project) },
        )?),
        c::OwnerOutcome::ProjectRemoved { project } => w::OwnerOutcome::ProjectRemoved(
            w::OwnerOutcomeProjectRemoved::new(limits, w::OwnerOutcomeProjectRemovedParts { project: *(project) })?,
        ),
        c::OwnerOutcome::ProjectMade { project } => w::OwnerOutcome::ProjectMade(w::OwnerOutcomeProjectMade::new(
            limits,
            w::OwnerOutcomeProjectMadeParts { project: *(project) },
        )?),
        c::OwnerOutcome::PoolSet { project, person } => w::OwnerOutcome::PoolSet(w::OwnerOutcomePoolSet::new(
            limits,
            w::OwnerOutcomePoolSetParts { project: *(project), person: *(person) },
        )?),
        c::OwnerOutcome::EscalationDecided { task, revision, by, at, choice } => {
            w::OwnerOutcome::EscalationDecided(w::OwnerOutcomeEscalationDecided::new(
                limits,
                w::OwnerOutcomeEscalationDecidedParts {
                    task: *(task),
                    revision: wire_revision(revision, limits)?,
                    by: *(by),
                    at: (at).as_nanos(),
                    choice: wire_owner_escalation_choice(choice, limits)?,
                },
            )?)
        }
        c::OwnerOutcome::AccountKeyKept { source } => w::OwnerOutcome::AccountKeyKept(
            w::OwnerOutcomeAccountKeyKept::new(limits, w::OwnerOutcomeAccountKeyKeptParts { source: *(source) })?,
        ),
        c::OwnerOutcome::AccountProgramDeclared { source, name } => {
            w::OwnerOutcome::AccountProgramDeclared(w::OwnerOutcomeAccountProgramDeclared::new(
                limits,
                w::OwnerOutcomeAccountProgramDeclaredParts { source: *(source), name: bytes::copy_of(name) },
            )?)
        }
        c::OwnerOutcome::AccountChanged => w::OwnerOutcome::AccountChanged,
        c::OwnerOutcome::AccountMovedIn { account, renewed } => {
            w::OwnerOutcome::AccountMovedIn(w::OwnerOutcomeAccountMovedIn::new(
                limits,
                w::OwnerOutcomeAccountMovedInParts { account: *(account), renewed: *(renewed) },
            )?)
        }
        c::OwnerOutcome::AccountRemoved { confirmed, failed_hosts } => {
            w::OwnerOutcome::AccountRemoved(w::OwnerOutcomeAccountRemoved::new(
                limits,
                w::OwnerOutcomeAccountRemovedParts {
                    confirmed: *(confirmed),
                    failed_hosts: list((failed_hosts).iter().map(|item| wire_account_host_failure(item, limits)))?,
                },
            )?)
        }
        c::OwnerOutcome::AccountSignInStarted { source, address } => {
            w::OwnerOutcome::AccountSignInStarted(w::OwnerOutcomeAccountSignInStarted::new(
                limits,
                w::OwnerOutcomeAccountSignInStartedParts { source: *(source), address: bytes::copy_of(address) },
            )?)
        }
        c::OwnerOutcome::AccountSignedOut { confirmed, failed_hosts } => {
            w::OwnerOutcome::AccountSignedOut(w::OwnerOutcomeAccountSignedOut::new(
                limits,
                w::OwnerOutcomeAccountSignedOutParts {
                    confirmed: *(confirmed),
                    failed_hosts: list((failed_hosts).iter().map(|item| wire_account_host_failure(item, limits)))?,
                },
            )?)
        }
    })
}

pub(in crate::codec) fn wire_owner_escalation_choice(
    value: &c::OwnerEscalationChoice,
    _limits: &w::Limits,
) -> Result<w::OwnerEscalationChoice, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerEscalationChoice::Released => w::OwnerEscalationChoice::Released,
        c::OwnerEscalationChoice::Rejected => w::OwnerEscalationChoice::Rejected,
        c::OwnerEscalationChoice::Passed => w::OwnerEscalationChoice::Passed,
    })
}

pub(in crate::codec) fn wire_account_host_failure(
    value: &c::AccountHostFailure,
    limits: &w::Limits,
) -> Result<w::AccountHostFailure, w::Problem> {
    w::AccountHostFailure::new(
        limits,
        w::AccountHostFailureParts {
            name: bytes::copy_of(&value.name),
            why: wire_account_host_why(&value.why, limits)?,
        },
    )
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
