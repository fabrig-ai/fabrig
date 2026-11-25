//! Procedure parameters and delegate result vocabulary (temper/docs/design/domain/forge/change/README.md, 1).
use alloc::boxed::Box;
use jig_domain_tasks as tasks;
use temper_engine_domain_forge as forge;
use temper_engine_domain_forge_change as forge_change;

#[must_use]
pub fn initial_change_title(parameters: &[tasks::Parameter]) -> Box<[u8]> {
    let mut handed = false;
    let mut title = None;
    for parameter in parameters {
        match parameter {
            tasks::Parameter::Bytes { name: 3 | 19, .. } => handed = true,
            tasks::Parameter::Bytes { name: 10, value } => title = Some(value.as_ref()),
            tasks::Parameter::Bytes { .. } | tasks::Parameter::Number { .. } | tasks::Parameter::Resource { .. } => {}
        }
    }
    if !handed {
        return Box::new([]);
    }
    match title {
        Some(title) if !title.is_empty() => forge::text::fit(title, 256, "…".as_bytes()),
        Some(_) | None => Box::from(forge::DEFAULT_CHANGE_TITLE),
    }
}

/// Translate a delegate's terminal kind into status, failure and cancellation facts.
#[must_use]
pub fn change_delegate_status(kind: &tasks::MessageKind) -> Option<(forge_change::Status, bool, bool)> {
    match kind {
        tasks::MessageKind::Result(
            tasks::ResultKind::NoChange
            | tasks::ResultKind::Report
            | tasks::ResultKind::Change { .. }
            | tasks::ResultKind::Verdict { code: 1, .. }
            | tasks::ResultKind::Job,
        ) => Some((forge_change::Status::Passed, false, false)),
        tasks::MessageKind::Result(tasks::ResultKind::Failed) => Some((forge_change::Status::Failed, true, false)),
        tasks::MessageKind::Result(tasks::ResultKind::Cancelled) => Some((forge_change::Status::Failed, false, true)),
        tasks::MessageKind::Result(tasks::ResultKind::Verdict { .. } | tasks::ResultKind::Approved { .. }) => {
            Some((forge_change::Status::Failed, false, false))
        }
        tasks::MessageKind::Escalation { .. }
        | tasks::MessageKind::Proposal { .. }
        | tasks::MessageKind::SettingsDecision { .. }
        | tasks::MessageKind::ProposalDecision { .. }
        | tasks::MessageKind::Words
        | tasks::MessageKind::FinishRequested { .. }
        | tasks::MessageKind::Amendment { .. }
        | tasks::MessageKind::Question
        | tasks::MessageKind::Answer { .. }
        | tasks::MessageKind::Met { .. }
        | tasks::MessageKind::Notice { .. }
        | tasks::MessageKind::PassedQuestion { .. }
        | tasks::MessageKind::PassedAnswer { .. }
        | tasks::MessageKind::QuestionAnswered { .. }
        | tasks::MessageKind::QuestionOwed
        | tasks::MessageKind::NothingAwaited
        | tasks::MessageKind::QuestionEnded { .. }
        | tasks::MessageKind::Timer { .. }
        | tasks::MessageKind::PersonStalled { .. }
        | tasks::MessageKind::EffectNotMade { .. }
        | tasks::MessageKind::News { .. } => None,
    }
}

/// Only a repair's flake verdict asks for a regate.
#[must_use]
pub fn repair_verdict(kind: forge_change::Delegate, code: u32) -> bool {
    if let forge_change::Delegate::Repair(_) = kind { code == forge::REPAIR_FLAKE } else { false }
}
