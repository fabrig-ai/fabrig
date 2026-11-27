//! Tool shape admission (temper/docs/design/domain/routing.md, 2).
use crate::{Call, EscalationChoice, MessageForm, ProposalChoice, ProposedAction, Tool};
use jig_domain_notes as notes;
use jig_domain_tasks as tasks;

#[must_use]
pub fn checked_call(mut body: Call, limits: &tasks::Limits, notes: &notes::Limits) -> Call {
    let note_refusal = note_shape(&body.tool, notes);
    if let Some(why) = note_refusal {
        body.tool = Tool::RejectedNote(why);
    }
    if let Some(why) = call_shape(&body.tool, limits) {
        body.tool = match body.tool {
            Tool::Message { .. } => Tool::RejectedMessage(why),
            Tool::Amend { .. }
            | Tool::Cancel { .. }
            | Tool::Release { .. }
            | Tool::Measure { .. }
            | Tool::DecideEscalation { .. } => Tool::RejectedControl(why),
            Tool::Propose { .. } | Tool::Decide { .. } | Tool::Withdraw { .. } => Tool::RejectedProposal(why),
            Tool::Note { .. } | Tool::NoteLines { .. } | Tool::Recall { .. } => {
                Tool::RejectedNote(jig_domain_notes::Refusal::Oversized)
            }
            Tool::Follow(_)
            | Tool::Finding(_)
            | Tool::Red(_)
            | Tool::Decided(_)
            | Tool::Goal { .. }
            | Tool::ReadTask { .. }
            | Tool::See { .. }
            | Tool::Document { .. }
            | Tool::Remember { .. }
            | Tool::Settings { .. }
            | Tool::ProposeSettings { .. }
            | Tool::ProposeSettingsNote { .. }
            | Tool::Change { .. }
            | Tool::Delegate { .. }
            | Tool::MarkRead { .. }
            | Tool::Introduce { .. }
            | Tool::Subscribe { .. }
            | Tool::SubscribeForge { .. }
            | Tool::SubscribeNamedForge { .. }
            | Tool::ReadBranchFile { .. }
            | Tool::ReadNamed { .. }
            | Tool::ReadForge { .. }
            | Tool::EffectForge { .. }
            | Tool::Unsubscribe { .. }
            | Tool::Rejected(_)
            | Tool::RejectedMessage(_)
            | Tool::RejectedControl(_)
            | Tool::RejectedProposal(_)
            | Tool::RejectedNote(_)
            | Tool::Unavailable => Tool::Rejected(why),
        };
    }
    body
}

fn note_shape(tool: &Tool, limits: &notes::Limits) -> Option<notes::Refusal> {
    match tool {
        Tool::Note { entry, .. } | Tool::NoteLines { entry, .. } => {
            if notes::valid_new(entry, limits) {
                None
            } else {
                Some(notes::new_refusal(entry, limits))
            }
        }
        Tool::Recall { by, .. } => (!notes::valid_recall(by, limits)).then_some(notes::Refusal::Oversized),
        Tool::Follow(_)
        | Tool::Finding(_)
        | Tool::Red(_)
        | Tool::Decided(_)
        | Tool::Goal { .. }
        | Tool::ReadTask { .. }
        | Tool::See { .. }
        | Tool::Document { .. }
        | Tool::Remember { .. }
        | Tool::Settings { .. }
        | Tool::ProposeSettings { .. }
        | Tool::ProposeSettingsNote { .. }
        | Tool::Change { .. }
        | Tool::Delegate { .. }
        | Tool::Message { .. }
        | Tool::MarkRead { .. }
        | Tool::Amend { .. }
        | Tool::Cancel { .. }
        | Tool::Release { .. }
        | Tool::Measure { .. }
        | Tool::Decide { .. }
        | Tool::DecideEscalation { .. }
        | Tool::Withdraw { .. }
        | Tool::Propose { .. }
        | Tool::Introduce { .. }
        | Tool::Subscribe { .. }
        | Tool::SubscribeForge { .. }
        | Tool::SubscribeNamedForge { .. }
        | Tool::ReadBranchFile { .. }
        | Tool::ReadNamed { .. }
        | Tool::ReadForge { .. }
        | Tool::EffectForge { .. }
        | Tool::Unsubscribe { .. }
        | Tool::Unavailable
        | Tool::Rejected(_)
        | Tool::RejectedMessage(_)
        | Tool::RejectedControl(_)
        | Tool::RejectedProposal(_)
        | Tool::RejectedNote(_) => None,
    }
}

#[must_use]
pub fn call_needs_input(tool: &Tool) -> bool {
    call_mode(tool).0
}

#[expect(clippy::too_many_lines, reason = "one exhaustive tool-shape match keeps bounded admission together")]
fn call_shape(tool: &Tool, limits: &tasks::Limits) -> Option<tasks::Refusal> {
    match tool {
        Tool::Propose { action, reason, .. } => proposal_shape(action, reason, limits),
        Tool::Decide { decision, .. } => match decision {
            ProposalChoice::Reject { reason } if reason.len() > skein_lib::usize_of(limits.message_bytes) => {
                Some(tasks::Refusal::Read)
            }
            ProposalChoice::Accept | ProposalChoice::Reject { .. } | ProposalChoice::Pass => None,
        },
        Tool::DecideEscalation { decision, .. } => match decision {
            EscalationChoice::Reject { reason } if reason.len() > skein_lib::usize_of(limits.result_bytes) => {
                Some(tasks::Refusal::Read)
            }
            EscalationChoice::Release | EscalationChoice::Reject { .. } | EscalationChoice::Pass => None,
        },
        Tool::Follow(_)
        | Tool::Finding(_)
        | Tool::Red(_)
        | Tool::Decided(_)
        | Tool::Goal { .. }
        | Tool::ReadTask { .. }
        | Tool::See { .. }
        | Tool::Document { .. }
        | Tool::Remember { .. }
        | Tool::Settings { .. }
        | Tool::ProposeSettings { .. }
        | Tool::ProposeSettingsNote { .. }
        | Tool::Change { .. }
        | Tool::Withdraw { .. }
        | Tool::Unavailable
        | Tool::Rejected(_)
        | Tool::RejectedMessage(_)
        | Tool::RejectedControl(_)
        | Tool::RejectedProposal(_)
        | Tool::RejectedNote(_)
        | Tool::Note { .. }
        | Tool::NoteLines { .. }
        | Tool::Recall { .. }
        | Tool::MarkRead { .. }
        | Tool::Introduce { .. }
        | Tool::Subscribe { .. }
        | Tool::SubscribeForge { .. }
        | Tool::SubscribeNamedForge { .. }
        | Tool::ReadBranchFile { .. }
        | Tool::ReadNamed { .. }
        | Tool::ReadForge { .. }
        | Tool::EffectForge { .. }
        | Tool::Unsubscribe { .. }
        | Tool::Release { .. }
        | Tool::Measure { .. } => None,
        Tool::Cancel { reason, .. } => {
            if reason.len() > skein_lib::usize_of(limits.result_bytes) {
                Some(tasks::Refusal::Read)
            } else {
                None
            }
        }
        Tool::Amend { amendment, .. } => {
            if amendment.reason.len() > skein_lib::usize_of(limits.message_bytes) {
                return Some(tasks::Refusal::Read);
            }
            if let Some(spec) = &amendment.spec
                && (!tasks::valid_spec(limits, spec) || !spec.inputs.is_empty())
            {
                return Some(tasks::Refusal::Spec);
            }
            if let Some(authority) = &amendment.authority
                && !tasks::valid_authority(limits, authority)
            {
                return Some(tasks::Refusal::AuthorityShape);
            }
            if let Some(dependencies) = &amendment.dependencies
                && dependencies.len() > skein_lib::usize_of(limits.dependencies)
            {
                return Some(tasks::Refusal::Dependencies);
            }
            None
        }
        Tool::Message { form, words, .. } => {
            if words.is_empty() || words.len() > skein_lib::usize_of(limits.message_bytes) {
                return Some(tasks::Refusal::Read);
            }
            match form {
                MessageForm::Answer { question } if *question == 0 => return Some(tasks::Refusal::Read),
                MessageForm::Words | MessageForm::Question | MessageForm::Answer { .. } => {}
            }
            None
        }
        Tool::Delegate { batch } => {
            if batch.len() > skein_lib::usize_of(limits.batch) {
                return Some(tasks::Refusal::Batch);
            }
            for member in batch {
                if !tasks::valid_spec(limits, &member.spec) {
                    return Some(tasks::Refusal::Spec);
                }
                if !tasks::valid_contract(limits, &member.contract) {
                    return Some(tasks::Refusal::Contract);
                }
                if !tasks::valid_authority(limits, &member.authority) {
                    return Some(tasks::Refusal::AuthorityShape);
                }
                if member.dependencies.len() > skein_lib::usize_of(limits.dependencies) {
                    return Some(tasks::Refusal::Dependencies);
                }
            }
            None
        }
    }
}

fn proposal_shape(action: &ProposedAction, reason: &[u8], limits: &tasks::Limits) -> Option<tasks::Refusal> {
    if reason.len() > skein_lib::usize_of(limits.message_bytes) {
        return Some(tasks::Refusal::Read);
    }
    match action {
        ProposedAction::Batch(batch) => {
            if batch.is_empty() || batch.len() > skein_lib::usize_of(limits.batch) {
                return Some(tasks::Refusal::Batch);
            }
            for member in batch {
                if !tasks::valid_spec(limits, &member.spec) || !member.spec.inputs.is_empty() {
                    return Some(tasks::Refusal::Spec);
                }
                if !tasks::valid_contract(limits, &member.contract)
                    || !tasks::valid_authority(limits, &member.authority)
                    || member.dependencies.len() > skein_lib::usize_of(limits.dependencies)
                {
                    return Some(tasks::Refusal::AuthorityShape);
                }
            }
            None
        }
        ProposedAction::Amend { amendment, .. } => {
            if amendment.reason.len() > skein_lib::usize_of(limits.message_bytes) {
                Some(tasks::Refusal::Read)
            } else if match &amendment.authority {
                Some(authority) => !tasks::valid_authority(limits, authority),
                None => false,
            } {
                Some(tasks::Refusal::AuthorityShape)
            } else {
                None
            }
        }
        ProposedAction::Widen { authority, .. } => {
            if tasks::valid_authority(limits, authority) {
                None
            } else {
                Some(tasks::Refusal::AuthorityShape)
            }
        }
        ProposedAction::Effect { .. } | ProposedAction::Release { .. } => None,
    }
}

#[must_use]
pub fn tool_kind(tool: &Tool) -> Option<jig_domain_core::ToolKind> {
    Some(match tool {
        Tool::See { .. } | Tool::Document { .. } | Tool::Remember { .. } => jig_domain_core::ToolKind::Ask,
        Tool::Settings { .. } | Tool::ProposeSettings { .. } | Tool::ProposeSettingsNote { .. } => {
            jig_domain_core::ToolKind::Settings
        }
        Tool::Follow(_)
        | Tool::Finding(_)
        | Tool::Red(_)
        | Tool::Goal { .. }
        | Tool::Change { .. }
        | Tool::Delegate { .. } => jig_domain_core::ToolKind::Delegate,
        Tool::MarkRead { .. } => jig_domain_core::ToolKind::MarkRead,
        Tool::Message { target, form: MessageForm::Answer { question }, .. } => {
            jig_domain_core::ToolKind::AnswerQuestion { target: *target, question: *question }
        }
        Tool::Message { target, form: MessageForm::Words | MessageForm::Question, .. } => {
            jig_domain_core::ToolKind::Message { target: *target }
        }
        Tool::Introduce { left, .. } => jig_domain_core::ToolKind::Message { target: *left },
        Tool::Decided(_) | Tool::Amend { .. } | Tool::Cancel { .. } | Tool::Release { .. } | Tool::Measure { .. } => {
            jig_domain_core::ToolKind::Control
        }
        Tool::Decide { .. } | Tool::Withdraw { .. } | Tool::DecideEscalation { .. } => {
            jig_domain_core::ToolKind::Decide
        }
        Tool::Propose { .. } => jig_domain_core::ToolKind::Propose,
        Tool::Subscribe { .. }
        | Tool::SubscribeForge { .. }
        | Tool::SubscribeNamedForge { .. }
        | Tool::Unsubscribe { .. } => jig_domain_core::ToolKind::Subscribe,
        Tool::EffectForge { .. } => jig_domain_core::ToolKind::Effect,
        Tool::Note { entry, .. } | Tool::NoteLines { entry, .. } => {
            jig_domain_core::ToolKind::Note { scope: entry.scope.clone() }
        }
        Tool::Recall { by, .. } => jig_domain_core::ToolKind::Recall { by: by.clone() },
        Tool::ReadTask { .. } | Tool::ReadBranchFile { .. } | Tool::ReadNamed { .. } | Tool::ReadForge { .. } => {
            jig_domain_core::ToolKind::Read
        }
        Tool::Unavailable
        | Tool::Rejected(_)
        | Tool::RejectedMessage(_)
        | Tool::RejectedControl(_)
        | Tool::RejectedProposal(_)
        | Tool::RejectedNote(_) => return None,
    })
}

#[must_use]
pub fn is_core_read(tool: &Tool) -> bool {
    call_mode(tool).1
}

fn call_mode(tool: &Tool) -> (bool, bool) {
    match tool {
        Tool::Follow(call) => match call {
            crate::FollowCall::List | crate::FollowCall::Show { .. } => (false, true),
            crate::FollowCall::Act { .. } => (false, false),
        },
        Tool::Red(call) => match call {
            crate::RedCall::List => (false, true),
            crate::RedCall::Record { .. } | crate::RedCall::End { .. } => (false, false),
        },
        Tool::Finding(call) => match call {
            crate::FindingCall::List { .. } | crate::FindingCall::Show { .. } => (false, true),
            crate::FindingCall::File { .. } | crate::FindingCall::Hand { .. } | crate::FindingCall::Triage { .. } => {
                (false, false)
            }
        },
        Tool::ReadTask { .. } | Tool::See { .. } | Tool::Settings { .. } => (false, true),
        Tool::Goal { agree, .. } => (*agree, false),
        Tool::Delegate { batch } => {
            for member in batch {
                if !member.spec.inputs.is_empty() {
                    return (true, false);
                }
            }
            (false, false)
        }
        Tool::Document { .. }
        | Tool::Remember { .. }
        | Tool::ProposeSettings { .. }
        | Tool::ProposeSettingsNote { .. }
        | Tool::Decided(_)
        | Tool::Note { .. }
        | Tool::NoteLines { .. }
        | Tool::Recall { .. }
        | Tool::EffectForge { .. }
        | Tool::ReadBranchFile { .. }
        | Tool::ReadNamed { .. }
        | Tool::ReadForge { .. }
        | Tool::Propose { .. }
        | Tool::Decide { .. }
        | Tool::Withdraw { .. }
        | Tool::DecideEscalation { .. }
        | Tool::Change { .. }
        | Tool::Amend { .. }
        | Tool::Cancel { .. }
        | Tool::Release { .. }
        | Tool::Measure { .. }
        | Tool::Message { .. }
        | Tool::MarkRead { .. }
        | Tool::Introduce { .. }
        | Tool::Subscribe { .. }
        | Tool::SubscribeForge { .. }
        | Tool::SubscribeNamedForge { .. }
        | Tool::Unsubscribe { .. }
        | Tool::Rejected(_)
        | Tool::RejectedMessage(_)
        | Tool::RejectedControl(_)
        | Tool::RejectedProposal(_)
        | Tool::RejectedNote(_)
        | Tool::Unavailable => (false, false),
    }
}

#[cfg(test)]
mod proposal_tests {
    use super::{ProposedAction, proposal_shape, tasks};
    use alloc::boxed::Box;

    fn amendment(authority: Option<tasks::Authority>, reason: &[u8]) -> ProposedAction {
        ProposedAction::Amend {
            task: 1,
            revision: 0,
            amendment: tasks::Amendment {
                tracked: None,
                addressed: None,
                instructions: Some(b"revised instructions".as_slice().into()),
                release: None,
                spec: None,
                wake: None,
                dependencies: None,
                authority,
                llm: None,
                reason: reason.into(),
            },
        }
    }

    #[test]
    fn an_amend_proposal_may_omit_authority() {
        let limits = crate::tests::task_limits();
        assert_eq!(proposal_shape(&amendment(None, b"new plan"), b"please amend", &limits), None);
    }

    #[test]
    fn an_amend_proposal_rejects_malformed_named_authority() {
        let limits = crate::tests::task_limits();
        let mut authority = tasks::Authority {
            tools: tasks::Tools(0),
            models: tasks::Models::NONE,
            grants: Box::new([]),
            delegation: tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 },
            budget: tasks::Budget { spend: 0, deadline: None },
            notes: tasks::Scopes(0),
            note_resources: Box::new([]),
        };
        assert!(tasks::valid_authority(&limits, &authority));
        assert_eq!(proposal_shape(&amendment(Some(authority.clone()), b"plan"), b"ask", &limits), None);
        authority.notes = tasks::Scopes(8);
        assert!(!tasks::valid_authority(&limits, &authority));
        assert_eq!(
            proposal_shape(&amendment(Some(authority), b"plan"), b"ask", &limits),
            Some(tasks::Refusal::AuthorityShape)
        );
    }

    #[test]
    fn an_amend_proposal_bounds_both_reasons() {
        let mut limits = crate::tests::task_limits();
        limits.message_bytes = 4;
        assert_eq!(proposal_shape(&amendment(None, b"plan"), b"ask!", &limits), None);
        assert_eq!(proposal_shape(&amendment(None, b"plan!"), b"ask!", &limits), Some(tasks::Refusal::Read));
        assert_eq!(proposal_shape(&amendment(None, b"plan"), b"ask!!", &limits), Some(tasks::Refusal::Read));
    }
}
