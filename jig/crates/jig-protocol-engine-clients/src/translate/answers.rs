//! Durable keyed answers (protocol/client/README.md, section 3; client/domain/contract/README.md, 3).
use super::refusals;
use alloc::boxed::Box;
use jig_client_protocol_wire as w;
use jig_client_protocol_wire::Record as _;
use jig_domain_people as p;
use skein_lib::Writer;
pub(crate) fn answer(
    reply: &p::Reply,
    models: &[crate::Model],
    limits: &w::Limits,
) -> Result<w::KeyedAnswer, w::Problem> {
    let final_answer = match reply {
        p::Reply::Outcome(p::Outcome::WorkerPauseAsked { worker }) => w::Final::Done(w::Done {
            outcome: w::Outcome::WorkerPauseAsked(w::OutcomeWorkerPauseAsked { worker: *worker }),
        }),
        p::Reply::Outcome(p::Outcome::AccountReading { count }) => w::Final::Done(w::Done {
            outcome: w::Outcome::Account(w::OutcomeAccount {
                answer: w::AccountAnswer::Reading(w::AccountReading { count: *count }),
            }),
        }),
        p::Reply::Outcome(p::Outcome::Restored { proposal, to, left_out }) => {
            let mut settings = skein_lib::List::with_capacity(super::count(left_out.len())?);
            for setting in left_out {
                settings.push(super::settings::name(*setting, limits)?).expect("left-out count");
            }
            let outcome = w::Outcome::Restored(w::OutcomeRestored { proposal: *proposal, to: *to, left_out: settings });
            w::Final::Done(w::Done { outcome })
        }
        p::Reply::Outcome(p::Outcome::MovedToNewest { newest, moved, stayed }) => w::Final::Done(w::Done {
            outcome: w::Outcome::MovedToNewest(w::OutcomeMovedToNewest {
                newest: *newest,
                moved: *moved,
                stayed: *stayed,
            }),
        }),
        p::Reply::Outcome(p::Outcome::SettingsChanged { revision }) => w::Final::Done(w::Done {
            outcome: w::Outcome::SettingsChanged(w::OutcomeSettingsChanged { revision: *revision }),
        }),
        p::Reply::Outcome(p::Outcome::Again(why)) | p::Reply::Again(why) => return again(*why, limits),
        p::Reply::Outcome(p::Outcome::SignedOut) | p::Reply::SignedOut => return Ok(w::KeyedAnswer::SignedOut),
        p::Reply::Outcome(p::Outcome::Refused(why)) | p::Reply::Refused(why) => {
            w::Final::Refused(w::Refused { at: None, refusal: refusal(why.clone(), models, limits)? })
        }
        p::Reply::Outcome(p::Outcome::CompoundRefused { at, why }) => {
            w::Final::Refused(w::Refused { refusal: refusal(why.clone(), models, limits)?, at: Some(*at) })
        }
        p::Reply::Outcome(p::Outcome::ServiceMade { person }) => {
            let what =
                w::Made::Service(w::MadeService { party: *person, credential: skein_lib::Secret::new(Box::new([])) });
            let outcome = w::Outcome::Made(w::OutcomeMade { what });
            w::Final::Done(w::Done { outcome })
        }
        p::Reply::Outcome(p::Outcome::ServiceReissued { .. }) => {
            w::Final::Refused(w::Refused { at: None, refusal: w::Refusal::NotOffered })
        }
        p::Reply::Outcome(value) => {
            let value = owner(value, limits)?;
            let outcome = w::Outcome::Document(w::OutcomeDocument { value });
            w::Final::Done(w::Done { outcome })
        }
        p::Reply::Signer(_) | p::Reply::Full { .. } | p::Reply::SignedIn { .. } => return Err(problem()),
    };
    let mut out = Writer::new(skein_lib::length::usize_of(final_answer.measure(limits)?));
    final_answer.write(limits, &mut out).expect("measured final answer");
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
    let what = w::Made::Service(w::MadeService { party, credential });
    let outcome = w::Outcome::Made(w::OutcomeMade { what });
    let answer = w::Final::Done(w::Done { outcome });
    let mut out = Writer::new(skein_lib::length::usize_of(answer.measure(limits)?));
    answer.write(limits, &mut out).expect("measured final answer");
    owned(w::Owner::Jig, jig_client_protocol_wire::VERSION, out.finish(), limits)
}
pub(crate) fn moving_out(
    account: u32,
    number: u32,
    record: skein_lib::Secret,
    limits: &w::Limits,
) -> Result<w::KeyedAnswer, w::Problem> {
    let what = w::Made::MovingOut(w::MadeMovingOut { account, number, record });
    let outcome = w::Outcome::Made(w::OutcomeMade { what });
    let answer = w::Final::Done(w::Done { outcome });
    let mut out = Writer::new(skein_lib::length::usize_of(answer.measure(limits)?));
    answer.write(limits, &mut out).expect("measured final answer");
    owned(w::Owner::Jig, jig_client_protocol_wire::VERSION, out.finish(), limits)
}
#[expect(clippy::unnecessary_wraps, reason = "translation helpers keep the fallible codec boundary")]
pub(crate) fn owned(
    owner: w::Owner,
    version: u32,
    record: Box<[u8]>,
    _limits: &w::Limits,
) -> Result<w::KeyedAnswer, w::Problem> {
    Ok(w::KeyedAnswer::FinalAnswer(w::Owned { owner, version, record }))
}
#[expect(clippy::unnecessary_wraps, reason = "translation helpers keep the fallible codec boundary")]
fn again(value: p::Again, _limits: &w::Limits) -> Result<w::KeyedAnswer, w::Problem> {
    Ok(match value {
        p::Again::Busy => w::KeyedAnswer::Busy,
        p::Again::Full { held, room_at } => {
            w::KeyedAnswer::AnswersFull(w::AnswersFull { held, room_at: room_at.as_nanos() })
        }
        p::Again::NotReady => w::KeyedAnswer::NotReady,
    })
}
fn problem() -> w::Problem {
    w::Problem { path: w::paths::OWNED_RECORD, reason: skein_codec::Reason::Malformed }
}
#[expect(clippy::too_many_lines, reason = "one total match names every people outcome")]
fn owner(value: &p::Outcome, limits: &w::Limits) -> Result<w::DocumentOutcome, w::Problem> {
    Ok(match value {
        p::Outcome::VisibilitySet { project, public } => {
            w::DocumentOutcome::VisibilitySet(w::OutcomeVisibilitySet { project: *project, public: *public })
        }
        p::Outcome::ProjectRemoved { project } => {
            w::DocumentOutcome::ProjectRemoved(w::OutcomeProjectRemoved { project: *project })
        }
        p::Outcome::AccountFinishing { source } => {
            w::DocumentOutcome::AccountFinishing(w::OutcomeAccountFinishing { source: *source })
        }
        p::Outcome::AccountMovedIn { account, renewed } => {
            w::DocumentOutcome::AccountMovedIn(w::OutcomeAccountMovedIn { account: *account, renewed: *renewed })
        }
        p::Outcome::Proposed { proposal, to } => {
            w::DocumentOutcome::Proposed(w::OutcomeProposed { proposal: *proposal, to: *to })
        }
        p::Outcome::Withdrawn { proposal } => {
            w::DocumentOutcome::Withdrawn(w::OutcomeWithdrawn { proposal: *proposal })
        }
        p::Outcome::Created { first, last, given } => w::DocumentOutcome::Created(w::OutcomeCreated {
            first: *first,
            last: *last,
            given: match given {
                Some(v) => Some(w::CreatedGiven { task: v.task, amount: v.amount }),
                None => None,
            },
        }),
        p::Outcome::Compound { project, operations, first_message } => {
            w::DocumentOutcome::Compound(w::OutcomeCompound {
                project: *project,
                operations: *operations,
                first_message: *first_message,
            })
        }
        p::Outcome::NoteEdited { name } => w::DocumentOutcome::NoteEdited(w::OutcomeNoteEdited { name: *name }),
        p::Outcome::ServiceMade { person } => {
            w::DocumentOutcome::ServiceMade(w::OutcomeServiceMade { person: *person })
        }
        p::Outcome::Adopted { project } => w::DocumentOutcome::Adopted(w::OutcomeAdopted { project: *project }),
        p::Outcome::Left { task } => w::DocumentOutcome::Left(w::OutcomeStopped { task: *task }),
        p::Outcome::Stopped { task } => w::DocumentOutcome::Stopped(w::OutcomeStopped { task: *task }),
        p::Outcome::Concluded { task } => w::DocumentOutcome::Concluded(w::OutcomeConcluded { task: *task }),
        p::Outcome::Cancelled { task } => w::DocumentOutcome::Cancelled(w::OutcomeCancelled { task: *task }),
        p::Outcome::Lifted { hold, released } => {
            w::DocumentOutcome::Lifted(w::OutcomeLifted { hold: *hold, released: *released })
        }
        p::Outcome::Released { task } => w::DocumentOutcome::Released(w::OutcomeReleased { task: *task }),
        p::Outcome::PersonTaken { task } => w::DocumentOutcome::PersonTaken(w::OutcomePersonTaken { task: *task }),
        p::Outcome::PersonHandedBack { task } => {
            w::DocumentOutcome::PersonHandedBack(w::OutcomePersonHandedBack { task: *task })
        }
        p::Outcome::PersonAnswered { task } => {
            w::DocumentOutcome::PersonAnswered(w::OutcomePersonAnswered { task: *task })
        }
        p::Outcome::Moved { task } => w::DocumentOutcome::Moved(w::OutcomeMoved { task: *task }),
        p::Outcome::Said { task, message, .. } => {
            w::DocumentOutcome::Said(w::OutcomeSaid { task: *task, message: *message })
        }
        p::Outcome::QuestionAnswered { task, question, message } => {
            w::DocumentOutcome::QuestionAnswered(w::OutcomeQuestionAnswered {
                task: *task,
                question: *question,
                message: *message,
            })
        }
        p::Outcome::Amended { task, .. } => w::DocumentOutcome::Amended(w::OutcomeAmended { task: *task }),
        p::Outcome::RolesSet { project } => w::DocumentOutcome::RolesSet(w::OutcomeRolesSet { project: *project }),
        p::Outcome::Renamed { project } => w::DocumentOutcome::Renamed(w::OutcomeRenamed { project: *project }),
        p::Outcome::ProjectMade { project } => {
            w::DocumentOutcome::ProjectMade(w::OutcomeProjectMade { project: *project })
        }
        p::Outcome::PolicyChanged { project } => {
            w::DocumentOutcome::PolicyChanged(w::OutcomePolicyChanged { project: *project })
        }
        p::Outcome::PoolSet { project, person } => {
            w::DocumentOutcome::PoolSet(w::OutcomePoolSet { project: *project, person: *person })
        }
        p::Outcome::MarkedSeen { seen } => {
            w::DocumentOutcome::MarkedSeen(w::OutcomeMarkedSeen { seen: self::seen(*seen, limits)? })
        }
        p::Outcome::ProposalDecided { proposer, proposal, by, at, choice, repeated } => {
            let proposer = match proposer {
                p::Proposer::Task(value) => w::Proposer::Task(w::ProposerTask { value: *value }),
                p::Proposer::Party(value) => w::Proposer::Party(w::ProposerParty { value: *value }),
            };
            let proposal = w::ProposalKey { proposer, number: *proposal };
            w::DocumentOutcome::ProposalDecided(w::OutcomeProposalDecided {
                proposal,
                by: *by,
                at: at.as_nanos(),
                choice: proposal_choice(*choice),
                repeated: *repeated,
            })
        }
        p::Outcome::EscalationDecided { task, revision, by, at, choice } => {
            let revision = w::Revision { value: *revision };
            w::DocumentOutcome::EscalationDecided(w::OutcomeEscalationDecided {
                task: *task,
                revision,
                by: *by,
                at: at.as_nanos(),
                choice: escalation_choice(*choice),
            })
        }
        p::Outcome::AccountSignInStarted { source, address } => {
            w::DocumentOutcome::AccountSignInStarted(w::OutcomeAccountSignInStarted {
                source: *source,
                address: skein_lib::bytes::copy_of(address),
            })
        }
        p::Outcome::AccountSignedOut { confirmed, failed_hosts } => {
            w::DocumentOutcome::AccountSignedOut(w::OutcomeAccountSignedOut {
                confirmed: *confirmed,
                failed_hosts: super::account_hosts::encode(failed_hosts, limits)?,
            })
        }
        p::Outcome::AccountKeyKept { source } => {
            w::DocumentOutcome::AccountKeyKept(w::OutcomeAccountKeyKept { source: *source })
        }
        p::Outcome::AccountProgramDeclared { source, name } => {
            w::DocumentOutcome::AccountProgramDeclared(w::OutcomeAccountProgramDeclared {
                source: *source,
                name: name.clone(),
            })
        }
        p::Outcome::AccountLoginAsked { number } => {
            w::DocumentOutcome::AccountLoginAsked(w::OutcomeAccountLoginAsked { number: *number })
        }
        p::Outcome::AccountChanged => w::DocumentOutcome::AccountChanged,
        p::Outcome::AccountRemoved { confirmed, failed_hosts } => {
            w::DocumentOutcome::AccountRemoved(w::OutcomeAccountRemoved {
                confirmed: *confirmed,
                failed_hosts: super::account_hosts::encode(failed_hosts, limits)?,
            })
        }
        p::Outcome::Restored { .. }
        | p::Outcome::SettingsChanged { .. }
        | p::Outcome::MovedToNewest { .. }
        | p::Outcome::WorkerPauseAsked { .. }
        | p::Outcome::ServiceReissued { .. }
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
#[expect(clippy::unnecessary_wraps, reason = "translation helpers keep the fallible codec boundary")]
fn seen(value: p::Seen, _limits: &w::Limits) -> Result<w::SeenThrough, w::Problem> {
    Ok(match value {
        p::Seen::Results { through } => w::SeenThrough::Results(w::SeenResults { through }),
        p::Seen::Conversation { task, through } => w::SeenThrough::Conversation(w::SeenConversation { task, through }),
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
