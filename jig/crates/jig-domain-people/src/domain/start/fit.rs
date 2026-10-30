//! Owned representation bounds, independent of admission judgments (jig/docs/design/domain/engine.md, 6).
use crate::{Act, Ask, Identity, Limits, Outcome, Refusal, RestoreBound, RestoreCause};
use alloc::boxed::Box;

pub(super) fn within(which: RestoreBound, len: usize, limit: u32) -> Result<(), RestoreCause> {
    let Ok(held) = u64::try_from(len) else { return Err(RestoreCause::Accounting) };
    measured(which, held, limit)
}

fn measured(which: RestoreBound, held: u64, limit: u32) -> Result<(), RestoreCause> {
    let limit = u64::from(limit);
    if held > limit {
        return Err(RestoreCause::Bound { which, held, limit });
    }
    Ok(())
}

pub(super) fn occupancy(which: RestoreBound, first: u32, second: u32, limit: u32) -> Result<(), RestoreCause> {
    let held = u64::from(first)
        .checked_add(u64::from(second))
        .ok_or(RestoreCause::Accounting)?
        .checked_add(1)
        .ok_or(RestoreCause::Accounting)?;
    measured(which, held, limit)
}

pub(super) fn identity(limits: &Limits, identity: &Identity) -> Result<(), RestoreCause> {
    let held = identity
        .key
        .subject
        .len()
        .checked_add(identity.login.len())
        .ok_or(RestoreCause::Accounting)?
        .checked_add(identity.name.len())
        .ok_or(RestoreCause::Accounting)?;
    within(RestoreBound::IdentityBytes, held, limits.identity_bytes)
}

pub(super) fn models(names: &[Box<[u8]>]) -> Result<(), RestoreCause> {
    within(RestoreBound::ModelNames, names.len(), crate::MODELS)?;
    for name in names {
        within(RestoreBound::ModelNameBytes, name.len(), crate::MODEL_NAME_BYTES)?;
    }
    Ok(())
}

fn bytes(which: RestoreBound, value: Option<u64>, limit: u32) -> Result<(), RestoreCause> {
    measured(which, value.ok_or(RestoreCause::Accounting)?, limit)
}

fn choice(value: Option<&crate::LlmChoice>) -> Result<(), RestoreCause> {
    if let Some(value) = value {
        within(RestoreBound::Subagents, value.subagents.len(), crate::MODELS)?;
    }
    Ok(())
}

fn batch(limits: &Limits, members: &[crate::Member]) -> Result<(), RestoreCause> {
    within(RestoreBound::Batch, members.len(), limits.batch)?;
    for member in members {
        within(RestoreBound::Words, member.words.len(), limits.words)?;
        choice(member.llm.as_ref())?;
        bytes(RestoreBound::AmendmentBytes, crate::member_bytes(member), limits.amendment_bytes)?;
    }
    Ok(())
}

fn amendment(limits: &Limits, value: &crate::Amendment) -> Result<(), RestoreCause> {
    within(RestoreBound::Words, value.reason.len(), limits.words)?;
    choice(value.llm.as_ref())?;
    bytes(RestoreBound::AmendmentBytes, crate::amendment_bytes(value), limits.amendment_bytes)
}

fn adoption(limits: &Limits, value: &crate::Adoption) -> Result<(), RestoreCause> {
    let items = value.resource.path.len().checked_add(value.options.len()).ok_or(RestoreCause::Accounting)?;
    let mut bytes = items.checked_mul(size_of::<Box<[u8]>>()).ok_or(RestoreCause::Accounting)?;
    for part in &value.resource.path {
        bytes = bytes.checked_add(part.len()).ok_or(RestoreCause::Accounting)?;
    }
    for part in &value.options {
        bytes = bytes.checked_add(part.len()).ok_or(RestoreCause::Accounting)?;
    }
    within(RestoreBound::AmendmentBytes, bytes, limits.amendment_bytes)
}

fn note(limits: &Limits, scope: &crate::NoteScope, change: &crate::NoteChange) -> Result<(), RestoreCause> {
    let scope = match scope {
        crate::NoteScope::Deployment | crate::NoteScope::Project | crate::NoteScope::Goal { .. } => 0,
        crate::NoteScope::Resources { pattern, .. } => {
            super::super::note_pattern_bytes(pattern).ok_or(RestoreCause::Accounting)?
        }
    };
    let change = match change {
        crate::NoteChange::Delete { .. } => 0,
        crate::NoteChange::Correct { description, body, references, .. } => {
            let bytes = description.len().checked_add(body.len()).ok_or(RestoreCause::Accounting)?;
            let refs = references.len().checked_mul(size_of::<u64>()).ok_or(RestoreCause::Accounting)?;
            bytes.checked_add(refs).ok_or(RestoreCause::Accounting)?
        }
    };
    let held = scope.checked_add(change).ok_or(RestoreCause::Accounting)?;
    within(RestoreBound::AmendmentBytes, held, limits.amendment_bytes)
}

fn person_result(limits: &Limits, result: &crate::PersonResult) -> Result<(), RestoreCause> {
    let held = match result {
        crate::PersonResult::Report { words } | crate::PersonResult::Verdict { words, .. } => words.len(),
        crate::PersonResult::Failure { reason } => reason.len(),
        crate::PersonResult::Approved { state, words } => {
            let words = match words {
                Some(words) => words.len(),
                None => 0,
            };
            state.len().checked_add(words).ok_or(RestoreCause::Accounting)?
        }
    };
    within(RestoreBound::Words, held, limits.words)
}

pub(super) fn ask(limits: &Limits, value: &Ask) -> Result<(), RestoreCause> {
    match value {
        Ask::Compound { operations, .. } => {
            measured(RestoreBound::Operations, u64::from(operations.len()), limits.operations)?;
            for operation in operations {
                let fits = match operation {
                    crate::Operation::Cancel { reason, .. } | crate::Operation::Move { reason, .. } => {
                        within(RestoreBound::Words, reason.len(), limits.words)
                    }
                    crate::Operation::Message { words, .. } => within(RestoreBound::Words, words.len(), limits.words),
                    crate::Operation::Amend { amendment: value, .. } => amendment(limits, value),
                    crate::Operation::Release { .. } | crate::Operation::Stop { .. } => Ok(()),
                };
                fits?;
            }
            Ok(())
        }
        Ask::Create { batch: members, .. } => batch(limits, members),
        Ask::Propose { action, reason, .. } => {
            within(RestoreBound::Words, reason.len(), limits.words)?;
            match action {
                crate::Action::Batch(members) => batch(limits, members),
                crate::Action::Amend { amendment: value, .. } => amendment(limits, value),
                crate::Action::Settings { .. } => Ok(()),
            }
        }
        Ask::Amend { amendment: value, .. } => amendment(limits, value),
        Ask::Say { words, .. } | Ask::AnswerQuestion { words, .. } => {
            within(RestoreBound::Words, words.len(), limits.words)
        }
        Ask::Move { reason, .. } | Ask::Cancel { reason, .. } => {
            within(RestoreBound::Words, reason.len(), limits.words)
        }
        Ask::AnswerPerson { result, .. } => person_result(limits, result),
        Ask::Adopt { adoption: value, .. } => adoption(limits, value),
        Ask::EditNote { scope, change, .. } => note(limits, scope, change),
        Ask::MakeService { name, .. } => {
            within(RestoreBound::Words, name.len(), limits.words)?;
            let held = name.len().checked_add(size_of::<u64>()).ok_or(RestoreCause::Accounting)?;
            within(RestoreBound::IdentityBytes, held, limits.identity_bytes)
        }
        Ask::SetRoles { holdings, .. } => within(RestoreBound::Holdings, holdings.len(), limits.holdings),
        Ask::Rename { name, .. } | Ask::MakeProject { name, .. } => {
            within(RestoreBound::NameBytes, name.len(), u32::try_from(crate::NAME_BYTES).expect("name bound fits"))
        }
        Ask::ChangePolicy { change, .. } => {
            bytes(RestoreBound::AmendmentBytes, crate::policy_change_bytes(change), limits.amendment_bytes)
        }
        Ask::DecideEscalation { decision, .. } => match decision {
            crate::EscalationDecision::Reject { reason } => within(RestoreBound::Words, reason.len(), limits.words),
            crate::EscalationDecision::Release | crate::EscalationDecision::Pass => Ok(()),
        },
        Ask::DecideProposal { decision, .. } => match decision {
            crate::ProposalDecision::Reject { reason } => within(RestoreBound::Words, reason.len(), limits.words),
            crate::ProposalDecision::Accept | crate::ProposalDecision::Pass => Ok(()),
        },
        Ask::AccountChange { change, .. } => match change {
            crate::AccountChange::DeclareProgram { name, .. } => within(RestoreBound::ProgramNameBytes, name.len(), 32),
            crate::AccountChange::SetUses { uses: Some(uses), .. } => {
                within(RestoreBound::Uses, uses.len(), crate::USES)?;
                let held = uses.len().checked_mul(size_of::<crate::Use>()).ok_or(RestoreCause::Accounting)?;
                within(RestoreBound::Words, held, limits.words)
            }
            crate::AccountChange::SetUses { uses: None, .. }
            | crate::AccountChange::CheckProgram { .. }
            | crate::AccountChange::AddKey { .. }
            | crate::AccountChange::ReplaceKey { .. }
            | crate::AccountChange::RemoveKey { .. }
            | crate::AccountChange::Resume { .. }
            | crate::AccountChange::Remove { .. }
            | crate::AccountChange::MoveAccount { .. }
            | crate::AccountChange::MoveSource { .. }
            | crate::AccountChange::MoveOut { .. }
            | crate::AccountChange::Written { .. }
            | crate::AccountChange::MoveIn { .. }
            | crate::AccountChange::FinishSignIn { .. } => Ok(()),
        },
        Ask::MarkSeen { .. }
        | Ask::Application { .. }
        | Ask::ChangeSettings { .. }
        | Ask::MoveToNewest { .. }
        | Ask::AccountSignIn { .. }
        | Ask::AccountSignOut { .. }
        | Ask::SetPool { .. }
        | Ask::Withdraw { .. }
        | Ask::Lift { .. }
        | Ask::TakePerson { .. }
        | Ask::HandBackPerson { .. }
        | Ask::Stop { .. }
        | Ask::Release { .. } => Ok(()),
    }
}

pub(super) fn act(limits: &Limits, value: &Act) -> Result<(), RestoreCause> {
    match value {
        Act::Close { reason, .. } => within(RestoreBound::Words, reason.len(), limits.words),
        Act::Adopt { adoption: value, .. } => adoption(limits, value),
        Act::MakeProject { name, .. } => {
            within(RestoreBound::NameBytes, name.len(), u32::try_from(crate::NAME_BYTES).expect("name bound fits"))
        }
        Act::AddKey { .. } | Act::RemoveAccount { .. } => Ok(()),
    }
}

pub(super) fn outcome(limits: &Limits, outcome: &Outcome) -> Result<(), RestoreCause> {
    match outcome {
        Outcome::Refused(why) | Outcome::CompoundRefused { why, .. } => refusal(why),
        Outcome::AccountSignedOut { failed_hosts, .. } | Outcome::AccountRemoved { failed_hosts, .. } => {
            within(RestoreBound::ProgramHosts, failed_hosts.len(), limits.program_hosts)?;
            for row in failed_hosts {
                within(RestoreBound::ProgramHostNameBytes, row.name.len(), limits.program_host_name_bytes)?;
            }
            Ok(())
        }
        Outcome::AccountProgramDeclared { name, .. } => within(RestoreBound::ProgramNameBytes, name.len(), 32),
        Outcome::AccountSignInStarted { address, .. } => {
            within(RestoreBound::AddressBytes, address.len(), crate::ADDRESS)
        }
        Outcome::Compound { .. }
        | Outcome::MarkedSeen { .. }
        | Outcome::Applied { .. }
        | Outcome::NoteEdited { .. }
        | Outcome::ServiceMade { .. }
        | Outcome::Adopted { .. }
        | Outcome::Created { .. }
        | Outcome::Proposed { .. }
        | Outcome::Withdrawn { .. }
        | Outcome::Stopped { .. }
        | Outcome::Cancelled { .. }
        | Outcome::Lifted { .. }
        | Outcome::Released { .. }
        | Outcome::PersonTaken { .. }
        | Outcome::PersonHandedBack { .. }
        | Outcome::PersonAnswered { .. }
        | Outcome::Moved { .. }
        | Outcome::ProposalDecided { .. }
        | Outcome::Said { .. }
        | Outcome::QuestionAnswered { .. }
        | Outcome::Amended { .. }
        | Outcome::RolesSet { .. }
        | Outcome::PolicyChanged { .. }
        | Outcome::Renamed { .. }
        | Outcome::ProjectMade { .. }
        | Outcome::SettingsChanged { .. }
        | Outcome::MovedToNewest { .. }
        | Outcome::PoolSet { .. }
        | Outcome::EscalationDecided { .. }
        | Outcome::AccountKeyKept { .. }
        | Outcome::AccountMovingOut { .. }
        | Outcome::AccountFinishing { .. }
        | Outcome::AccountMovedIn { .. }
        | Outcome::AccountChanged
        | Outcome::SignedOut
        | Outcome::Again(_) => Ok(()),
    }
}

fn refusal(why: &Refusal) -> Result<(), RestoreCause> {
    match why {
        Refusal::Circle { tasks, .. } | Refusal::Full { tasks, .. } => {
            within(RestoreBound::NamedTasks, tasks.len(), crate::NAMED_TASKS)
        }
        Refusal::Kept { pending } => within(RestoreBound::KeptPending, pending.len(), 64),
        Refusal::SignInRedirect { .. }
        | Refusal::Stale { .. }
        | Refusal::NameTaken
        | Refusal::AccountMove { .. }
        | Refusal::Dependency { .. }
        | Refusal::Account { .. }
        | Refusal::NotWellFormed { .. }
        | Refusal::Role { .. }
        | Refusal::Authority { .. }
        | Refusal::Limit { .. }
        | Refusal::Moved { .. }
        | Refusal::Ended { .. }
        | Refusal::Taken { .. }
        | Refusal::NotWaiting
        | Refusal::NoFurther
        | Refusal::Unknown
        | Refusal::NotOffered
        | Refusal::Provider
        | Refusal::KeyConflict
        | Refusal::NoEscalationHolder
        | Refusal::Llm { .. } => Ok(()),
    }
}
