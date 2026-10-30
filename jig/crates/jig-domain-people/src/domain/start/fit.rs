//! Owned representation bounds, independent of admission judgments (engine.md, 6).
use crate::{Act, Ask, Identity, Limits, Outcome, Refusal};
use alloc::boxed::Box;

pub(super) fn within(len: usize, limit: u32) -> bool {
    match u32::try_from(len) {
        Ok(len) => len <= limit,
        Err(_) => false,
    }
}

pub(super) fn identity(limits: &Limits, identity: &Identity) -> bool {
    let Some(first) = identity.key.subject.len().checked_add(identity.login.len()) else {
        return false;
    };
    match first.checked_add(identity.name.len()) {
        Some(bytes) => within(bytes, limits.identity_bytes),
        None => false,
    }
}

pub(super) fn models(names: &[Box<[u8]>]) -> bool {
    if !within(names.len(), crate::MODELS) {
        return false;
    }
    for name in names {
        if !within(name.len(), crate::MODEL_NAME_BYTES) {
            return false;
        }
    }
    true
}

fn bytes(value: Option<u64>, limit: u32) -> bool {
    match value {
        Some(value) => value <= u64::from(limit),
        None => false,
    }
}

fn batch(limits: &Limits, members: &[crate::Member]) -> bool {
    if !within(members.len(), limits.batch) {
        return false;
    }
    for member in members {
        if !within(member.words.len(), limits.words)
            || !crate::llm::valid(member.llm.as_ref())
            || !bytes(crate::member_bytes(member), limits.amendment_bytes)
        {
            return false;
        }
    }
    true
}

fn amendment(limits: &Limits, value: &crate::Amendment) -> bool {
    within(value.reason.len(), limits.words)
        && crate::llm::valid(value.llm.as_ref())
        && bytes(crate::amendment_bytes(value), limits.amendment_bytes)
}

fn adoption(limits: &Limits, value: &crate::Adoption) -> bool {
    let Some(items) = value.resource.path.len().checked_add(value.options.len()) else {
        return false;
    };
    let Some(mut bytes) = items.checked_mul(size_of::<Box<[u8]>>()) else {
        return false;
    };
    for part in &value.resource.path {
        let Some(next) = bytes.checked_add(part.len()) else {
            return false;
        };
        bytes = next;
    }
    for part in &value.options {
        let Some(next) = bytes.checked_add(part.len()) else {
            return false;
        };
        bytes = next;
    }
    within(bytes, limits.amendment_bytes)
}

fn note(limits: &Limits, scope: &crate::NoteScope, change: &crate::NoteChange) -> bool {
    let scope = match scope {
        crate::NoteScope::Deployment | crate::NoteScope::Project | crate::NoteScope::Goal { .. } => Some(0),
        crate::NoteScope::Resources { pattern, .. } => super::super::note_pattern_bytes(pattern),
    };
    let change = match change {
        crate::NoteChange::Delete { .. } => Some(0),
        crate::NoteChange::Correct { description, body, references, .. } => {
            let Some(bytes) = description.len().checked_add(body.len()) else {
                return false;
            };
            let Some(refs) = references.len().checked_mul(size_of::<u64>()) else {
                return false;
            };
            bytes.checked_add(refs)
        }
    };
    match (scope, change) {
        (Some(scope), Some(change)) => match scope.checked_add(change) {
            Some(bytes) => within(bytes, limits.amendment_bytes),
            None => false,
        },
        (Some(_) | None, None) | (None, Some(_)) => false,
    }
}

pub(super) fn ask(limits: &Limits, value: &Ask) -> bool {
    match value {
        Ask::Compound { operations, .. } => {
            if operations.len() > limits.operations {
                return false;
            }
            for operation in operations {
                let fits = match operation {
                    crate::Operation::Cancel { reason, .. } | crate::Operation::Move { reason, .. } => {
                        within(reason.len(), limits.words)
                    }
                    crate::Operation::Message { words, .. } => within(words.len(), limits.words),
                    crate::Operation::Amend { amendment: value, .. } => amendment(limits, value),
                    crate::Operation::Release { .. } | crate::Operation::Stop { .. } => true,
                };
                if !fits {
                    return false;
                }
            }
            true
        }
        Ask::Create { batch: members, .. } => batch(limits, members),
        Ask::Propose { action, reason, .. } => {
            within(reason.len(), limits.words)
                && match action {
                    crate::Action::Batch(members) => batch(limits, members),
                    crate::Action::Amend { amendment: value, .. } => amendment(limits, value),
                    crate::Action::Settings { .. } => true,
                }
        }
        Ask::Amend { amendment: value, .. } => amendment(limits, value),
        Ask::Say { words, .. } | Ask::AnswerQuestion { words, .. } => within(words.len(), limits.words),
        Ask::Move { reason, .. } | Ask::Cancel { reason, .. } => within(reason.len(), limits.words),
        Ask::AnswerPerson { result, .. } => super::super::valid_person_result(limits, result),
        Ask::Adopt { adoption: value, .. } => adoption(limits, value),
        Ask::EditNote { scope, change, .. } => note(limits, scope, change),
        Ask::MakeService { name, .. } => {
            within(name.len(), limits.words)
                && match name.len().checked_add(size_of::<u64>()) {
                    Some(bytes) => within(bytes, limits.identity_bytes),
                    None => false,
                }
        }
        Ask::SetRoles { holdings, .. } => within(holdings.len(), limits.holdings),
        Ask::Rename { name, .. } | Ask::MakeProject { name, .. } => name.len() <= crate::NAME_BYTES,
        Ask::ChangePolicy { change, .. } => bytes(crate::policy_change_bytes(change), limits.amendment_bytes),
        Ask::DecideEscalation { decision, .. } => match decision {
            crate::EscalationDecision::Reject { reason } => within(reason.len(), limits.words),
            crate::EscalationDecision::Release | crate::EscalationDecision::Pass => true,
        },
        Ask::DecideProposal { decision, .. } => match decision {
            crate::ProposalDecision::Reject { reason } => within(reason.len(), limits.words),
            crate::ProposalDecision::Accept | crate::ProposalDecision::Pass => true,
        },
        Ask::AccountChange { change, .. } => match change {
            crate::AccountChange::DeclareProgram { name, .. } => name.len() <= 32,
            crate::AccountChange::SetUses { uses: Some(uses), .. } => {
                within(uses.len(), crate::USES)
                    && match uses.len().checked_mul(size_of::<crate::Use>()) {
                        Some(bytes) => within(bytes, limits.words),
                        None => false,
                    }
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
            | crate::AccountChange::FinishSignIn { .. } => true,
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
        | Ask::Release { .. } => true,
    }
}

pub(super) fn act(limits: &Limits, value: &Act) -> bool {
    match value {
        Act::Close { reason, .. } => within(reason.len(), limits.words),
        Act::Adopt { adoption: value, .. } => adoption(limits, value),
        Act::MakeProject { name, .. } => name.len() <= crate::NAME_BYTES,
        Act::AddKey { .. } | Act::RemoveAccount { .. } => true,
    }
}

pub(super) fn outcome(limits: &Limits, outcome: &Outcome) -> bool {
    match outcome {
        Outcome::Refused(why) | Outcome::CompoundRefused { why, .. } => refusal(why),
        Outcome::AccountSignedOut { failed_hosts, .. } | Outcome::AccountRemoved { failed_hosts, .. } => {
            if !within(failed_hosts.len(), limits.program_hosts) {
                return false;
            }
            for row in failed_hosts {
                if !within(row.name.len(), limits.program_host_name_bytes) {
                    return false;
                }
            }
            true
        }
        Outcome::AccountProgramDeclared { name, .. } => name.len() <= 32,
        Outcome::AccountSignInStarted { address, .. } => within(address.len(), crate::ADDRESS),
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
        | Outcome::Again(_) => true,
    }
}

fn refusal(why: &Refusal) -> bool {
    match why {
        Refusal::Circle { tasks, .. } | Refusal::Full { tasks, .. } => within(tasks.len(), crate::NAMED_TASKS),
        Refusal::Kept { pending } => pending.len() <= 64,
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
        | Refusal::Llm { .. } => true,
    }
}
