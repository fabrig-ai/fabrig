//! Party words are UTF-8 at request admission (client/domain/contract/documents.md, 3).
use crate::{Ask, EscalationDecision, NoteChange, PersonResult, ProposalDecision};
use skein_lib::bytes::is_utf8;

pub(crate) fn valid(ask: &Ask) -> bool {
    match ask {
        Ask::Talk { words, .. } | Ask::Say { words, .. } | Ask::AnswerQuestion { words, .. } => is_utf8(words),
        Ask::Propose { action: crate::Action::Batch(batch), reason, .. } => is_utf8(reason) && batch_words(batch),
        Ask::Create { batch, .. } => batch_words(batch),
        Ask::MakeService { name, .. } => is_utf8(name),
        Ask::Restore { reason, .. }
        | Ask::Propose { action: crate::Action::Settings { .. }, reason, .. }
        | Ask::Move { reason, .. }
        | Ask::Cancel { reason, .. } => is_utf8(reason),
        Ask::DecideEscalation { decision, .. } => match decision {
            EscalationDecision::Reject { reason } => is_utf8(reason),
            EscalationDecision::Release | EscalationDecision::Pass => true,
        },
        Ask::DecideProposal { decision, .. } => match decision {
            ProposalDecision::Reject { reason } => is_utf8(reason),
            ProposalDecision::Accept | ProposalDecision::Pass => true,
        },
        Ask::AnswerPerson { result, .. } => match result {
            PersonResult::Report { words } | PersonResult::Verdict { words, .. } => is_utf8(words),
            PersonResult::Failure { reason } => is_utf8(reason),
            PersonResult::Approved { words, .. } => match words {
                Some(words) => is_utf8(words),
                None => true,
            },
        },
        Ask::EditNote { change, .. } => match change.as_ref() {
            NoteChange::Correct { description, body, .. } => is_utf8(description) && is_utf8(body),
            NoteChange::Delete { .. } => true,
        },
        Ask::Propose { action: crate::Action::Amend { amendment, .. }, reason, .. } => {
            is_utf8(reason) && amendment_words(amendment)
        }
        Ask::Amend { amendment, .. } => amendment_words(amendment),
        Ask::Compound { .. }
        | Ask::MarkSeen { .. }
        | Ask::Application { .. }
        | Ask::Adopt { .. }
        | Ask::TakePerson { .. }
        | Ask::HandBackPerson { .. }
        | Ask::Stop { .. }
        | Ask::Lift { .. }
        | Ask::Leave { .. }
        | Ask::Release { .. }
        | Ask::SetRoles { .. }
        | Ask::ChangePolicy { .. }
        | Ask::RemoveProject { .. }
        | Ask::SetVisibility { .. }
        | Ask::Rename { .. }
        | Ask::MakeProject { .. }
        | Ask::ChangeSettings { .. }
        | Ask::MoveToNewest { .. }
        | Ask::SetPool { .. }
        | Ask::Withdraw { .. }
        | Ask::AccountSignIn { .. }
        | Ask::AccountSignOut { .. }
        | Ask::AccountChange { .. } => true,
    }
}

fn batch_words(batch: &[crate::Member]) -> bool {
    for member in batch {
        if !is_utf8(&member.words) {
            return false;
        }
        for parameter in &member.parameters {
            match parameter {
                crate::Parameter::Bytes { value, .. } => {
                    if !is_utf8(value) {
                        return false;
                    }
                }
                crate::Parameter::Number { .. } | crate::Parameter::Resource { .. } => {}
            }
        }
    }
    true
}

fn amendment_words(amendment: &crate::Amendment) -> bool {
    if !is_utf8(&amendment.reason) {
        return false;
    }
    if let Some(instructions) = &amendment.instructions
        && !is_utf8(instructions)
    {
        return false;
    }
    if let Some(spec) = &amendment.spec {
        if !is_utf8(&spec.words) {
            return false;
        }
        for parameter in &spec.parameters {
            match parameter {
                crate::Parameter::Bytes { value, .. } => {
                    if !is_utf8(value) {
                        return false;
                    }
                }
                crate::Parameter::Number { .. } | crate::Parameter::Resource { .. } => {}
            }
        }
    }
    true
}
