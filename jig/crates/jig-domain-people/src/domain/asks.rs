//! Request shape and payload validation (domain/people.md, 5.1).
use crate::{Ask, LimitKind, Limits, Refusal, Seen};
use alloc::boxed::Box;

pub(super) fn project(ask: &Ask) -> u32 {
    match ask {
        Ask::Application { scope, .. } => match scope {
            crate::ApplicationScope::Project(project) => *project,
            crate::ApplicationScope::Own | crate::ApplicationScope::Deployment => 0,
        },
        // An account request names no project; its check is its holder (domain/authority.md, 8.4).
        Ask::Lift { .. }
        | Ask::MarkSeen { .. }
        | Ask::AccountSignIn { .. }
        | Ask::AccountSignOut { .. }
        | Ask::AccountChange { .. } => 0,
        Ask::MakeProject { .. } | Ask::ChangeSettings { .. } | Ask::MoveToNewest { .. } => crate::DEPLOYMENT,
        Ask::RemoveProject { project, .. }
        | Ask::SetVisibility { project, .. }
        | Ask::Rename { project, .. }
        | Ask::Compound { project, .. }
        | Ask::EditNote { project, .. }
        | Ask::MakeService { project, .. }
        | Ask::Adopt { project, .. }
        | Ask::SetRoles { project, .. }
        | Ask::ChangePolicy { project, .. }
        | Ask::SetPool { project, .. }
        | Ask::DecideEscalation { project, .. }
        | Ask::DecideProposal { project, .. }
        | Ask::Say { project, .. }
        | Ask::AnswerQuestion { project, .. }
        | Ask::Amend { project, .. }
        | Ask::Move { project, .. }
        | Ask::TakePerson { project, .. }
        | Ask::HandBackPerson { project, .. }
        | Ask::AnswerPerson { project, .. }
        | Ask::Stop { project, .. }
        | Ask::Cancel { project, .. }
        | Ask::Release { project, .. }
        | Ask::Create { project, .. }
        | Ask::Propose { project, .. }
        | Ask::Withdraw { project, .. } => *project,
    }
}

pub(super) fn valid_adoption(limits: &Limits, adoption: &crate::Adoption) -> bool {
    let Some(items) = adoption.resource.path.len().checked_add(adoption.options.len()) else { return false };
    let Some(mut bytes) = items.checked_mul(size_of::<Box<[u8]>>()) else { return false };
    for segment in &adoption.resource.path {
        let Some(next) = bytes.checked_add(segment.len()) else { return false };
        bytes = next;
    }
    for option in &adoption.options {
        let Some(next) = bytes.checked_add(option.len()) else { return false };
        bytes = next;
    }
    !adoption.resource.path.is_empty() && bytes <= skein_lib::length::usize_of(limits.amendment_bytes)
}

pub(super) fn note_pattern_bytes(pattern: &crate::Pattern) -> Option<usize> {
    let mut bytes = pattern.segments.len().checked_mul(size_of::<Box<[u8]>>())?;
    for segment in &pattern.segments {
        bytes = bytes.checked_add(segment.len())?;
    }
    let last = match &pattern.last {
        crate::Last::Exact(bytes) | crate::Last::Open(bytes) => bytes.len(),
    };
    bytes.checked_add(last)
}

fn valid_note(limits: &Limits, name: u64, scope: &crate::NoteScope, change: &crate::NoteChange) -> bool {
    if name == 0 {
        return false;
    }
    let scope_bytes = match scope {
        crate::NoteScope::Deployment | crate::NoteScope::Project => 0,
        crate::NoteScope::Goal { goal } => {
            if *goal == 0 {
                return false;
            }
            0
        }
        crate::NoteScope::Resources { pattern, .. } => {
            let Some(bytes) = note_pattern_bytes(pattern) else { return false };
            bytes
        }
    };
    let change_bytes = match change {
        crate::NoteChange::Correct { description, body, references, recalled } => {
            if *recalled == 0 {
                return false;
            }
            for number in references {
                if *number == 0 {
                    return false;
                }
            }
            let Some(bytes) = description.len().checked_add(body.len()) else { return false };
            let Some(ref_bytes) = references.len().checked_mul(size_of::<u64>()) else { return false };
            let Some(total) = bytes.checked_add(ref_bytes) else { return false };
            total
        }
        crate::NoteChange::Delete { recalled } => {
            if *recalled == 0 {
                return false;
            }
            0
        }
    };
    match scope_bytes.checked_add(change_bytes) {
        Some(bytes) => bytes <= skein_lib::length::usize_of(limits.amendment_bytes),
        None => false,
    }
}

fn valid_person_result(limits: &Limits, result: &crate::PersonResult) -> bool {
    let bound = skein_lib::length::usize_of(limits.words);
    match result {
        crate::PersonResult::Report { words } | crate::PersonResult::Verdict { words, .. } => words.len() <= bound,
        crate::PersonResult::Failure { reason } => reason.len() <= bound,
        crate::PersonResult::Approved { state, words } => {
            let words = match words {
                Some(words) => words.len(),
                None => 0,
            };
            match state.len().checked_add(words) {
                Some(bytes) => bytes <= bound,
                None => false,
            }
        }
    }
}

pub(crate) fn valid_ask(limits: &Limits, ask: &Ask) -> bool {
    if !crate::words::valid(ask) {
        return false;
    }
    match ask {
        Ask::Compound { operations, .. } => crate::compound::refusal(limits, operations).is_none(),
        Ask::MarkSeen { seen } => match seen {
            Seen::Results { .. } => true,
            Seen::Conversation { task, .. } => *task != 0,
        },
        Ask::AccountChange { change: crate::AccountChange::DeclareProgram { name, .. }, .. } => {
            crate::program_name(name)
        }
        Ask::AccountChange { change: crate::AccountChange::SetUses { uses: Some(uses), .. }, .. } => {
            uses.len() <= skein_lib::length::usize_of(crate::USES)
                && match uses.len().checked_mul(size_of::<crate::Use>()) {
                    Some(bytes) => bytes <= skein_lib::length::usize_of(limits.words),
                    None => false,
                }
        }
        Ask::RemoveProject { .. }
        | Ask::SetVisibility { .. }
        | Ask::ChangeSettings { .. }
        | Ask::MoveToNewest { .. }
        | Ask::Application { .. }
        | Ask::AccountSignIn { .. }
        | Ask::AccountSignOut { .. }
        | Ask::AccountChange { .. } => true,
        Ask::EditNote { name, scope, change, .. } => valid_note(limits, *name, scope, change),
        Ask::MakeService { name, .. } => {
            let Some(bytes) = name.len().checked_add(size_of::<u64>()) else {
                return false;
            };
            !name.is_empty()
                && name.len() <= skein_lib::length::usize_of(limits.words)
                && bytes <= skein_lib::length::usize_of(limits.identity_bytes)
        }
        Ask::Move { task, reason, .. } | Ask::Cancel { task, reason, .. } => {
            *task != 0 && reason.len() <= skein_lib::length::usize_of(limits.words)
        }
        Ask::Lift { hold, .. } => *hold != 0,
        Ask::TakePerson { task, .. }
        | Ask::HandBackPerson { task, .. }
        | Ask::Stop { task, .. }
        | Ask::Release { task, .. } => *task != 0,
        Ask::AnswerPerson { task, result, .. } => *task != 0 && valid_person_result(limits, result),
        Ask::Adopt { adoption, .. } => valid_adoption(limits, adoption),
        Ask::SetRoles { holdings, .. } => holdings.len() <= skein_lib::length::usize_of(limits.holdings),
        Ask::Rename { name, .. } | Ask::MakeProject { name, .. } => name.len() <= crate::NAME_BYTES,
        Ask::ChangePolicy { change, .. } => match crate::policy_change_bytes(change) {
            Some(bytes) => bytes <= u64::from(limits.amendment_bytes),
            None => false,
        },
        Ask::SetPool { person, .. } => *person != 0,
        Ask::Create { batch, .. } => crate::batch::valid(limits, batch),
        Ask::Propose { action: crate::Action::Batch(batch), reason, .. } => {
            crate::batch::valid(limits, batch) && reason.len() <= skein_lib::length::usize_of(limits.words)
        }
        Ask::Propose { action: crate::Action::Amend { task, amendment, .. }, reason, .. } => {
            *task != 0
                && reason.len() <= skein_lib::length::usize_of(limits.words)
                && crate::amendment::valid(limits, amendment)
        }
        Ask::Propose { project, action: crate::Action::Settings { .. }, reason } => {
            *project == crate::DEPLOYMENT && reason.len() <= skein_lib::length::usize_of(limits.words)
        }
        Ask::Withdraw { proposal, .. } => *proposal != 0,
        Ask::Say { task, words, .. } => {
            *task != 0 && !words.is_empty() && words.len() <= skein_lib::length::usize_of(limits.words)
        }
        Ask::AnswerQuestion { task, question, words, .. } => {
            *task != 0
                && *question != 0
                && !words.is_empty()
                && words.len() <= skein_lib::length::usize_of(limits.words)
        }
        Ask::Amend { task, amendment, .. } => *task != 0 && crate::amendment::valid(limits, amendment),
        Ask::DecideEscalation { task, revision, decision, .. } => {
            *task != 0
                && *revision != 0
                && match decision {
                    crate::EscalationDecision::Release | crate::EscalationDecision::Pass => true,
                    crate::EscalationDecision::Reject { reason } => {
                        reason.len() <= skein_lib::length::usize_of(limits.words)
                    }
                }
        }
        Ask::DecideProposal { proposer, proposal, decision, running, .. } => {
            (running.is_none() || *decision == crate::ProposalDecision::Accept)
                && proposer.number() != 0
                && *proposal != 0
                && match decision {
                    crate::ProposalDecision::Accept | crate::ProposalDecision::Pass => true,
                    crate::ProposalDecision::Reject { reason } => {
                        reason.len() <= skein_lib::length::usize_of(limits.words)
                    }
                }
        }
    }
}

// Admission identifies the bounded payload before any keyed flight is made.
pub(crate) fn invalid_ask(limits: &Limits, ask: &Ask) -> Refusal {
    let which = match ask {
        Ask::Compound { operations, .. } => {
            return crate::compound::refusal(limits, operations).expect("invalid compound");
        }
        Ask::EditNote { name, .. } => {
            if *name == 0 {
                return Refusal::Unknown;
            }
            LimitKind::Notes
        }
        Ask::MakeService { name, .. } => {
            if name.is_empty() {
                LimitKind::Empty
            } else {
                LimitKind::Identity
            }
        }
        Ask::Move { task, .. } | Ask::Cancel { task, .. } | Ask::AnswerPerson { task, .. } => {
            if *task == 0 {
                return Refusal::Unknown;
            }
            LimitKind::Words
        }
        Ask::MarkSeen { .. }
        | Ask::Application { .. }
        | Ask::TakePerson { .. }
        | Ask::HandBackPerson { .. }
        | Ask::Stop { .. }
        | Ask::Lift { .. }
        | Ask::Release { .. }
        | Ask::SetPool { .. }
        | Ask::Withdraw { .. }
        | Ask::AccountSignIn { .. }
        | Ask::AccountSignOut { .. } => return Refusal::Unknown,
        Ask::Adopt { .. } => LimitKind::Resource,
        Ask::SetRoles { .. } => LimitKind::Roles,
        Ask::ChangePolicy { .. } => LimitKind::Policy,
        Ask::Create { batch, .. } | Ask::Propose { action: crate::Action::Batch(batch), .. } => {
            crate::batch::invalid(limits, batch)
        }
        Ask::RemoveProject { .. }
        | Ask::SetVisibility { .. }
        | Ask::Rename { .. }
        | Ask::MakeProject { .. }
        | Ask::ChangeSettings { .. }
        | Ask::MoveToNewest { .. }
        | Ask::AccountChange { .. }
        | Ask::Propose { action: crate::Action::Amend { .. } | crate::Action::Settings { .. }, .. } => LimitKind::Words,
        Ask::Say { task, words, .. } => {
            if *task == 0 {
                return Refusal::Unknown;
            }
            if words.is_empty() { LimitKind::Empty } else { LimitKind::Words }
        }
        Ask::AnswerQuestion { task, question, words, .. } => {
            if *task == 0 || *question == 0 {
                return Refusal::Unknown;
            }
            if words.is_empty() { LimitKind::Empty } else { LimitKind::Words }
        }
        Ask::Amend { task, .. } => {
            if *task == 0 {
                return Refusal::Unknown;
            }
            LimitKind::Spec
        }
        Ask::DecideEscalation { task, revision, .. } => {
            if *task == 0 {
                return Refusal::Unknown;
            }
            if *revision == 0 { LimitKind::Revision } else { LimitKind::Words }
        }
        Ask::DecideProposal { proposer, proposal, decision, running, .. } => {
            if running.is_some() && *decision != crate::ProposalDecision::Accept {
                return Refusal::NotWellFormed { at: 0, why: crate::Malformed::Running };
            }
            if proposer.number() == 0 || *proposal == 0 {
                return Refusal::Unknown;
            }
            LimitKind::Words
        }
    };
    Refusal::Limit { which }
}
