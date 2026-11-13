//! The loaded list joins jig's durable write answer (temper/docs/design/domain/agent/ask-temper.md, 4).
use crate::{CallAnswer, RememberAction, RememberDid, RememberRefusal, Remembered};
use alloc::boxed::Box;
use jig_domain_notes as notes;
use skein_lib::List;

#[derive(Hash, Debug)]
pub(crate) struct RememberFlight {
    did: RememberDid,
    line: u32,
    lines: List<Box<[u8]>>,
    bound: u32,
}

impl RememberFlight {
    pub(crate) fn new(action: &RememberAction, line: u32, loaded: &List<notes::Line>, bound: u32) -> Self {
        let mut lines = List::with_capacity(bound);
        let did = match action {
            RememberAction::Add { .. } => RememberDid::Kept,
            RememberAction::Change { .. } => RememberDid::Changed,
            RememberAction::Drop { .. } => RememberDid::Dropped,
        };
        let mut place = 0_u32;
        for entry in loaded {
            place = place.checked_add(1).expect("bounded list");
            match action {
                RememberAction::Change { text, .. } if place == line => lines.push(text.clone()).expect("bounded list"),
                RememberAction::Drop { .. } if place == line => {}
                RememberAction::Add { .. } | RememberAction::Change { .. } | RememberAction::Drop { .. } => {
                    lines.push(entry.description.clone()).expect("bounded list");
                }
            }
        }
        match action {
            RememberAction::Add { text } => lines.push(text.clone()).expect("admitted add"),
            RememberAction::Change { .. } | RememberAction::Drop { .. } => {}
        }
        Self { did, line, lines, bound }
    }

    pub(crate) fn join(&self, answer: CallAnswer) -> CallAnswer {
        let answer = match answer {
            CallAnswer::NoteWritten { .. } | CallAnswer::NoteDeleted { .. } => {
                Remembered::Done { did: self.did, line: self.line, lines: self.lines.clone(), bound: self.bound }
            }
            CallAnswer::NoteRefused(why) => Remembered::Refused(match why {
                notes::Refusal::Moved { .. } | notes::Refusal::Missing => {
                    RememberRefusal::Moved { action: self.did, line: self.line }
                }
                notes::Refusal::Busy => RememberRefusal::Busy,
                notes::Refusal::PastBound { .. }
                | notes::Refusal::Oversized
                | notes::Refusal::Full
                | notes::Refusal::Exists
                | notes::Refusal::RevisionExhausted
                | notes::Refusal::PastEnd { .. } => RememberRefusal::Note(why),
            }),
            answer @ (CallAnswer::ToolDenied { .. }
            | CallAnswer::Follow(_)
            | CallAnswer::Finding(_) | CallAnswer::Red(_)
            | CallAnswer::Decided { .. }
            | CallAnswer::TaskRead(_)
            | CallAnswer::SettingsRefused(_)
            | CallAnswer::HoldOwner { .. }
            | CallAnswer::Again { .. }
            | CallAnswer::PassedUp { .. }
            | CallAnswer::Revised { .. }
            | CallAnswer::Amended { .. }
            | CallAnswer::Moved { .. }
            | CallAnswer::ForgeEffect { .. }
            | CallAnswer::ForgeEffectRefused(_)
            | CallAnswer::ForgeEffectDenied { .. }
            | CallAnswer::ForgeRead(_)
            | CallAnswer::EscalationDecided { .. }
            | CallAnswer::EscalationRefused(_)
            | CallAnswer::Proposed { .. }
            | CallAnswer::ProposalDecided { .. }
            | CallAnswer::ProposalRefused(_)
            | CallAnswer::Controlled
            | CallAnswer::ControlRefused(_)
            | CallAnswer::EffectRefused(_)
            | CallAnswer::ControlDenied { .. }
            | CallAnswer::Sent { .. }
            | CallAnswer::Introduced
            | CallAnswer::MessageRefused(_)
            | CallAnswer::Subscribed { .. }
            | CallAnswer::Unsubscribed
            | CallAnswer::SubscriptionRefused(_)
            | CallAnswer::ForgeSubscriptionRefused { .. }
            | CallAnswer::Delegated(_)
            | CallAnswer::DelegationDenied { .. }
            | CallAnswer::DelegationRefused(_)
            | CallAnswer::ReadRepositoryRefused { .. }
            | CallAnswer::SubscriptionRepositoryRefused { .. }
            | CallAnswer::ChangeRefused { .. }
            | CallAnswer::NoteRecalled { .. }
            | CallAnswer::Document(_)
            | CallAnswer::Remember(_)
            | CallAnswer::Parts(_)
            | CallAnswer::Unavailable) => return answer,
        };
        CallAnswer::Remember(Box::new(answer))
    }
}
