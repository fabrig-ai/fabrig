//! Durable keyed answers (protocol/client/README.md, section 3; client/domain/contract/README.md, 3).
use super::refusals;
use alloc::boxed::Box;
use jig_client_protocol_wire::v22 as w;
use jig_domain_people as p;
use skein_lib::Writer;
pub(crate) fn answer(
    reply: &p::Reply,
    models: &[crate::Model],
    limits: &w::Limits,
) -> Result<w::KeyedAnswer, w::Problem> {
    let final_answer = match reply {
        p::Reply::Outcome(p::Outcome::AccountReading { count }) => w::Final::Done(w::Done::new(
            limits,
            w::DoneParts {
                outcome: w::Outcome::Account(w::OutcomeAccount::new(
                    limits,
                    w::OutcomeAccountParts {
                        answer: w::AccountAnswer::Reading(w::AccountReading::new(
                            limits,
                            w::AccountReadingParts { count: *count },
                        )?),
                    },
                )?),
            },
        )?),
        p::Reply::Outcome(p::Outcome::Restored { proposal, to, left_out }) => {
            let mut settings = skein_lib::List::with_capacity(super::count(left_out.len())?);
            for setting in left_out {
                settings.push(super::settings::name(*setting, limits)?).expect("left-out count");
            }
            let outcome = w::Outcome::Restored(w::OutcomeRestored::new(
                limits,
                w::OutcomeRestoredParts { proposal: *proposal, to: *to, left_out: settings },
            )?);
            w::Final::Done(w::Done::new(limits, w::DoneParts { outcome })?)
        }
        p::Reply::Outcome(p::Outcome::MovedToNewest { newest, moved, stayed }) => w::Final::Done(w::Done::new(
            limits,
            w::DoneParts {
                outcome: w::Outcome::MovedToNewest(w::OutcomeMovedToNewest::new(
                    limits,
                    w::OutcomeMovedToNewestParts { newest: *newest, moved: *moved, stayed: *stayed },
                )?),
            },
        )?),
        p::Reply::Outcome(p::Outcome::SettingsChanged { revision }) => w::Final::Done(w::Done::new(
            limits,
            w::DoneParts {
                outcome: w::Outcome::SettingsChanged(w::OutcomeSettingsChanged::new(
                    limits,
                    w::OutcomeSettingsChangedParts { revision: *revision },
                )?),
            },
        )?),
        p::Reply::Outcome(p::Outcome::Again(why)) | p::Reply::Again(why) => return again(*why, limits),
        p::Reply::Outcome(p::Outcome::SignedOut) | p::Reply::SignedOut => return Ok(w::KeyedAnswer::SignedOut),
        p::Reply::Outcome(p::Outcome::Refused(why)) | p::Reply::Refused(why) => w::Final::Refused(w::Refused::new(
            limits,
            w::RefusedParts { at: None, refusal: refusal(why.clone(), models, limits)? },
        )?),
        p::Reply::Outcome(p::Outcome::CompoundRefused { at, why }) => w::Final::Refused(w::Refused::new(
            limits,
            w::RefusedParts { refusal: refusal(why.clone(), models, limits)?, at: Some(*at) },
        )?),
        p::Reply::Outcome(p::Outcome::ServiceMade { person }) => {
            let what = w::Made::Service(w::MadeService::new(
                limits,
                w::MadeServiceParts { party: *person, credential: skein_lib::Secret::new(Box::new([])) },
            )?);
            let outcome = w::Outcome::Made(w::OutcomeMade::new(limits, w::OutcomeMadeParts { what })?);
            w::Final::Done(w::Done::new(limits, w::DoneParts { outcome })?)
        }
        p::Reply::Outcome(value) => {
            let value = owner(value, limits)?;
            let outcome = w::Outcome::Document(w::OutcomeDocument::new(limits, w::OutcomeDocumentParts { value })?);
            w::Final::Done(w::Done::new(limits, w::DoneParts { outcome })?)
        }
        p::Reply::Signer(_) | p::Reply::Full { .. } | p::Reply::SignedIn { .. } => return Err(problem()),
    };
    let mut out = Writer::new(skein_lib::length::usize_of(final_answer.measure()));
    final_answer.encode(&mut out).expect("measured final answer");
    owned(w::Owner::Jig, jig_client_protocol_wire::VERSION, out.finish(), limits)
}
/// A refusal, a refused model by its configured name.
fn refusal(value: p::Refusal, models: &[crate::Model], limits: &w::Limits) -> Result<w::Refusal, w::Problem> {
    if let p::Refusal::Llm { model, why } = value {
        return super::llm::refusal(model, why, models, limits);
    }
    refusals::refusal(value, limits)
}
pub(crate) fn service(
    party: u64,
    credential: skein_lib::Secret,
    limits: &w::Limits,
) -> Result<w::KeyedAnswer, w::Problem> {
    let what = w::Made::Service(w::MadeService::new(limits, w::MadeServiceParts { party, credential })?);
    let outcome = w::Outcome::Made(w::OutcomeMade::new(limits, w::OutcomeMadeParts { what })?);
    let answer = w::Final::Done(w::Done::new(limits, w::DoneParts { outcome })?);
    let mut out = Writer::new(skein_lib::length::usize_of(answer.measure()));
    answer.encode(&mut out).expect("measured final answer");
    owned(w::Owner::Jig, jig_client_protocol_wire::VERSION, out.finish(), limits)
}
pub(crate) fn moving_out(
    account: u32,
    number: u32,
    record: skein_lib::Secret,
    limits: &w::Limits,
) -> Result<w::KeyedAnswer, w::Problem> {
    let what = w::Made::MovingOut(w::MadeMovingOut::new(limits, w::MadeMovingOutParts { account, number, record })?);
    let outcome = w::Outcome::Made(w::OutcomeMade::new(limits, w::OutcomeMadeParts { what })?);
    let answer = w::Final::Done(w::Done::new(limits, w::DoneParts { outcome })?);
    let mut out = Writer::new(skein_lib::length::usize_of(answer.measure()));
    answer.encode(&mut out).expect("measured final answer");
    owned(w::Owner::Jig, jig_client_protocol_wire::VERSION, out.finish(), limits)
}
pub(crate) fn owned(
    owner: w::Owner,
    version: u32,
    record: Box<[u8]>,
    limits: &w::Limits,
) -> Result<w::KeyedAnswer, w::Problem> {
    Ok(w::KeyedAnswer::FinalAnswer(w::Owned::new(limits, w::OwnedParts { owner, version, record })?))
}
fn again(value: p::Again, limits: &w::Limits) -> Result<w::KeyedAnswer, w::Problem> {
    Ok(match value {
        p::Again::Busy => w::KeyedAnswer::Busy,
        p::Again::Full { held, room_at } => w::KeyedAnswer::AnswersFull(w::AnswersFull::new(
            limits,
            w::AnswersFullParts { held, room_at: room_at.as_nanos() },
        )?),
        p::Again::NotReady => w::KeyedAnswer::NotReady,
    })
}
fn problem() -> w::Problem {
    w::Problem { path: w::Path::OwnedRecord, reason: skein_codec::Reason::Malformed }
}
#[expect(clippy::too_many_lines, reason = "one total match names every people outcome")]
fn owner(value: &p::Outcome, limits: &w::Limits) -> Result<w::DocumentOutcome, w::Problem> {
    Ok(match value {
        p::Outcome::VisibilitySet { project, public } => w::DocumentOutcome::VisibilitySet(
            w::OutcomeVisibilitySet::new(limits, w::OutcomeVisibilitySetParts { project: *project, public: *public })?,
        ),
        p::Outcome::ProjectRemoved { project } => w::DocumentOutcome::ProjectRemoved(w::OutcomeProjectRemoved::new(
            limits,
            w::OutcomeProjectRemovedParts { project: *project },
        )?),
        p::Outcome::AccountFinishing { source } => w::DocumentOutcome::AccountFinishing(
            w::OutcomeAccountFinishing::new(limits, w::OutcomeAccountFinishingParts { source: *source })?,
        ),
        p::Outcome::AccountMovedIn { account, renewed } => {
            w::DocumentOutcome::AccountMovedIn(w::OutcomeAccountMovedIn::new(
                limits,
                w::OutcomeAccountMovedInParts { account: *account, renewed: *renewed },
            )?)
        }
        p::Outcome::Proposed { proposal, to } => w::DocumentOutcome::Proposed(w::OutcomeProposed::new(
            limits,
            w::OutcomeProposedParts { proposal: *proposal, to: *to },
        )?),
        p::Outcome::Withdrawn { proposal } => w::DocumentOutcome::Withdrawn(w::OutcomeWithdrawn::new(
            limits,
            w::OutcomeWithdrawnParts { proposal: *proposal },
        )?),
        p::Outcome::Created { first, last, given } => w::DocumentOutcome::Created(w::OutcomeCreated::new(
            limits,
            w::OutcomeCreatedParts {
                first: *first,
                last: *last,
                given: match given {
                    Some(v) => {
                        Some(w::CreatedGiven::new(limits, w::CreatedGivenParts { task: v.task, amount: v.amount })?)
                    }
                    None => None,
                },
            },
        )?),
        p::Outcome::Compound { project, operations, first_message } => {
            w::DocumentOutcome::Compound(w::OutcomeCompound::new(
                limits,
                w::OutcomeCompoundParts { project: *project, operations: *operations, first_message: *first_message },
            )?)
        }
        p::Outcome::NoteEdited { name } => w::DocumentOutcome::NoteEdited(w::OutcomeNoteEdited::new(
            limits,
            w::OutcomeNoteEditedParts { name: *name },
        )?),
        p::Outcome::ServiceMade { person } => w::DocumentOutcome::ServiceMade(w::OutcomeServiceMade::new(
            limits,
            w::OutcomeServiceMadeParts { person: *person },
        )?),
        p::Outcome::Adopted { project } => {
            w::DocumentOutcome::Adopted(w::OutcomeAdopted::new(limits, w::OutcomeAdoptedParts { project: *project })?)
        }
        p::Outcome::Left { task } => {
            w::DocumentOutcome::Left(w::OutcomeStopped::new(limits, w::OutcomeStoppedParts { task: *task })?)
        }
        p::Outcome::Stopped { task } => {
            w::DocumentOutcome::Stopped(w::OutcomeStopped::new(limits, w::OutcomeStoppedParts { task: *task })?)
        }
        p::Outcome::Cancelled { task } => {
            w::DocumentOutcome::Cancelled(w::OutcomeCancelled::new(limits, w::OutcomeCancelledParts { task: *task })?)
        }
        p::Outcome::Lifted { hold, released } => w::DocumentOutcome::Lifted(w::OutcomeLifted::new(
            limits,
            w::OutcomeLiftedParts { hold: *hold, released: *released },
        )?),
        p::Outcome::Released { task } => {
            w::DocumentOutcome::Released(w::OutcomeReleased::new(limits, w::OutcomeReleasedParts { task: *task })?)
        }
        p::Outcome::PersonTaken { task } => w::DocumentOutcome::PersonTaken(w::OutcomePersonTaken::new(
            limits,
            w::OutcomePersonTakenParts { task: *task },
        )?),
        p::Outcome::PersonHandedBack { task } => w::DocumentOutcome::PersonHandedBack(w::OutcomePersonHandedBack::new(
            limits,
            w::OutcomePersonHandedBackParts { task: *task },
        )?),
        p::Outcome::PersonAnswered { task } => w::DocumentOutcome::PersonAnswered(w::OutcomePersonAnswered::new(
            limits,
            w::OutcomePersonAnsweredParts { task: *task },
        )?),
        p::Outcome::Moved { task } => {
            w::DocumentOutcome::Moved(w::OutcomeMoved::new(limits, w::OutcomeMovedParts { task: *task })?)
        }
        p::Outcome::Said { task, message, .. } => w::DocumentOutcome::Said(w::OutcomeSaid::new(
            limits,
            w::OutcomeSaidParts { task: *task, message: *message },
        )?),
        p::Outcome::QuestionAnswered { task, question, message } => {
            w::DocumentOutcome::QuestionAnswered(w::OutcomeQuestionAnswered::new(
                limits,
                w::OutcomeQuestionAnsweredParts { task: *task, question: *question, message: *message },
            )?)
        }
        p::Outcome::Amended { task, .. } => {
            w::DocumentOutcome::Amended(w::OutcomeAmended::new(limits, w::OutcomeAmendedParts { task: *task })?)
        }
        p::Outcome::RolesSet { project } => w::DocumentOutcome::RolesSet(w::OutcomeRolesSet::new(
            limits,
            w::OutcomeRolesSetParts { project: *project },
        )?),
        p::Outcome::Renamed { project } => {
            w::DocumentOutcome::Renamed(w::OutcomeRenamed::new(limits, w::OutcomeRenamedParts { project: *project })?)
        }
        p::Outcome::ProjectMade { project } => w::DocumentOutcome::ProjectMade(w::OutcomeProjectMade::new(
            limits,
            w::OutcomeProjectMadeParts { project: *project },
        )?),
        p::Outcome::PolicyChanged { project } => w::DocumentOutcome::PolicyChanged(w::OutcomePolicyChanged::new(
            limits,
            w::OutcomePolicyChangedParts { project: *project },
        )?),
        p::Outcome::PoolSet { project, person } => w::DocumentOutcome::PoolSet(w::OutcomePoolSet::new(
            limits,
            w::OutcomePoolSetParts { project: *project, person: *person },
        )?),
        p::Outcome::MarkedSeen { seen } => w::DocumentOutcome::MarkedSeen(w::OutcomeMarkedSeen::new(
            limits,
            w::OutcomeMarkedSeenParts { seen: self::seen(*seen, limits)? },
        )?),
        p::Outcome::ProposalDecided { proposer, proposal, by, at, choice, repeated } => {
            let proposer = match proposer {
                p::Proposer::Task(value) => {
                    w::Proposer::Task(w::ProposerTask::new(limits, w::ProposerTaskParts { value: *value })?)
                }
                p::Proposer::Party(value) => {
                    w::Proposer::Party(w::ProposerParty::new(limits, w::ProposerPartyParts { value: *value })?)
                }
            };
            let proposal = w::ProposalKey::new(limits, w::ProposalKeyParts { proposer, number: *proposal })?;
            w::DocumentOutcome::ProposalDecided(w::OutcomeProposalDecided::new(
                limits,
                w::OutcomeProposalDecidedParts {
                    proposal,
                    by: *by,
                    at: at.as_nanos(),
                    choice: proposal_choice(*choice),
                    repeated: *repeated,
                },
            )?)
        }
        p::Outcome::EscalationDecided { task, revision, by, at, choice } => {
            let revision = w::Revision::new(limits, w::RevisionParts { value: *revision })?;
            w::DocumentOutcome::EscalationDecided(w::OutcomeEscalationDecided::new(
                limits,
                w::OutcomeEscalationDecidedParts {
                    task: *task,
                    revision,
                    by: *by,
                    at: at.as_nanos(),
                    choice: escalation_choice(*choice),
                },
            )?)
        }
        p::Outcome::AccountSignInStarted { source, address } => {
            w::DocumentOutcome::AccountSignInStarted(w::OutcomeAccountSignInStarted::new(
                limits,
                w::OutcomeAccountSignInStartedParts { source: *source, address: skein_lib::bytes::copy_of(address) },
            )?)
        }
        p::Outcome::AccountSignedOut { confirmed, failed_hosts } => {
            w::DocumentOutcome::AccountSignedOut(w::OutcomeAccountSignedOut::new(
                limits,
                w::OutcomeAccountSignedOutParts {
                    confirmed: *confirmed,
                    failed_hosts: super::account_hosts::encode(failed_hosts, limits)?,
                },
            )?)
        }
        p::Outcome::AccountKeyKept { source } => w::DocumentOutcome::AccountKeyKept(w::OutcomeAccountKeyKept::new(
            limits,
            w::OutcomeAccountKeyKeptParts { source: *source },
        )?),
        p::Outcome::AccountProgramDeclared { source, name } => {
            w::DocumentOutcome::AccountProgramDeclared(w::OutcomeAccountProgramDeclared::new(
                limits,
                w::OutcomeAccountProgramDeclaredParts { source: *source, name: name.clone() },
            )?)
        }
        p::Outcome::AccountLoginAsked { number } => w::DocumentOutcome::AccountLoginAsked(
            w::OutcomeAccountLoginAsked::new(limits, w::OutcomeAccountLoginAskedParts { number: *number })?,
        ),
        p::Outcome::AccountChanged => w::DocumentOutcome::AccountChanged,
        p::Outcome::AccountRemoved { confirmed, failed_hosts } => {
            w::DocumentOutcome::AccountRemoved(w::OutcomeAccountRemoved::new(
                limits,
                w::OutcomeAccountRemovedParts {
                    confirmed: *confirmed,
                    failed_hosts: super::account_hosts::encode(failed_hosts, limits)?,
                },
            )?)
        }
        p::Outcome::Restored { .. }
        | p::Outcome::SettingsChanged { .. }
        | p::Outcome::MovedToNewest { .. }
        | p::Outcome::AccountReading { .. }
        | p::Outcome::AccountMovingOut { .. }
        | p::Outcome::CompoundRefused { .. }
        | p::Outcome::Applied { .. }
        | p::Outcome::Again(_)
        | p::Outcome::SignedOut
        | p::Outcome::Refused(_) => {
            return Err(problem());
        }
    })
}
fn seen(value: p::Seen, limits: &w::Limits) -> Result<w::SeenThrough, w::Problem> {
    Ok(match value {
        p::Seen::Results { through } => {
            w::SeenThrough::Results(w::SeenResults::new(limits, w::SeenResultsParts { through })?)
        }
        p::Seen::Conversation { task, through } => {
            w::SeenThrough::Conversation(w::SeenConversation::new(limits, w::SeenConversationParts { task, through })?)
        }
    })
}
fn proposal_choice(value: p::ProposalChoice) -> w::ProposalChoice {
    match value {
        p::ProposalChoice::Accepted => w::ProposalChoice::Accepted,
        p::ProposalChoice::Rejected => w::ProposalChoice::Rejected,
        p::ProposalChoice::Passed => w::ProposalChoice::Passed,
        p::ProposalChoice::Withdrawn => w::ProposalChoice::Withdrawn,
        p::ProposalChoice::Stale => w::ProposalChoice::Stale,
    }
}
fn escalation_choice(value: p::EscalationChoice) -> w::EscalationChoice {
    match value {
        p::EscalationChoice::Released => w::EscalationChoice::Released,
        p::EscalationChoice::Rejected => w::EscalationChoice::Rejected,
        p::EscalationChoice::Passed => w::EscalationChoice::Passed,
    }
}
