//! Whole single-read owner documents (protocol/client/wire.md, 1, and protocol/client/README.md, 4).
use super::super::{owner, w};
use alloc::boxed::Box;
use jig_domain_tasks as t;
use skein_lib::{List, bytes};
pub fn message_kind(value: &t::MessageKind) -> Result<w::MessageKind, w::Problem> {
    Ok(match value {
        t::MessageKind::Met { dependency, removed } => {
            w::MessageKind::Met(w::MessageKindMet { dependency: *dependency, removed: *removed })
        }
        t::MessageKind::SettingsDecision { proposal, outcome } => {
            w::MessageKind::SettingsDecision(w::MessageKindSettingsDecision {
                proposal: *proposal,
                outcome: settings_outcome(*outcome),
            })
        }
        t::MessageKind::QuestionAnswered { question } => {
            w::MessageKind::QuestionAnswered(w::MessageKindAnswer { question: *question })
        }
        t::MessageKind::QuestionOwed => w::MessageKind::QuestionOwed,
        t::MessageKind::NothingAwaited => w::MessageKind::NothingAwaited,
        t::MessageKind::QuestionEnded { question } => {
            w::MessageKind::QuestionEnded(w::MessageKindAnswer { question: *question })
        }
        t::MessageKind::Words => w::MessageKind::Words,
        t::MessageKind::Question => w::MessageKind::Question,
        t::MessageKind::Escalation { task, revision } => {
            w::MessageKind::Escalation(w::MessageKindEscalation { task: *task, revision: *revision })
        }
        t::MessageKind::Proposal { proposer, proposal, kind } => w::MessageKind::Proposal(w::MessageKindProposal {
            proposer: *proposer,
            proposal: *proposal,
            kind: super::proposals::kind(*kind),
        }),
        t::MessageKind::PersonStalled { task } => {
            w::MessageKind::PersonStalled(w::MessageKindPersonStalled { task: *task })
        }
        t::MessageKind::ProposalDecision { proposal, accepted } => {
            w::MessageKind::ProposalDecision(w::MessageKindProposalDecision {
                proposal: *proposal,
                accepted: *accepted,
            })
        }
        t::MessageKind::EffectNotMade { entry } => {
            w::MessageKind::EffectNotMade(w::MessageKindEffectNotMade { entry: *entry })
        }
        t::MessageKind::Amendment { revision } | t::MessageKind::FinishRequested { revision } => {
            w::MessageKind::Amendment(w::MessageKindAmendment { revision: *revision })
        }
        t::MessageKind::Answer { question } => w::MessageKind::Answer(w::MessageKindAnswer { question: *question }),
        t::MessageKind::Timer { subscription } => {
            w::MessageKind::Timer(w::MessageKindTimer { subscription: *subscription })
        }
        t::MessageKind::Notice { subscription, target, state } => w::MessageKind::Notice(w::MessageKindNotice {
            subscription: *subscription,
            target: *target,
            state: match state {
                t::NoticeState::Held { .. } => w::NoticeState::Held,
                t::NoticeState::Done => w::NoticeState::Done,
                t::NoticeState::Failed => w::NoticeState::Failed,
                t::NoticeState::Cancelled => w::NoticeState::Cancelled,
            },
        }),
        t::MessageKind::News { subscription, class } => w::MessageKind::News(w::MessageKindNews {
            subscription: *subscription,
            class: match class {
                t::NewsClass::Wakes => w::NewsClass::Wakes,
                t::NewsClass::Kept => w::NewsClass::Kept,
                t::NewsClass::Dropped => w::NewsClass::Dropped,
            },
        }),
        t::MessageKind::Result(value) => w::MessageKind::Result(w::MessageKindResult { value: word_result(value)? }),
    })
}
pub fn word_result(value: &t::ResultKind) -> Result<w::WordResult, w::Problem> {
    Ok(match value {
        t::ResultKind::Job => w::WordResult::Job,
        t::ResultKind::NoChange | t::ResultKind::Report => w::WordResult::Report,
        t::ResultKind::Failed => w::WordResult::Failed,
        t::ResultKind::Cancelled => w::WordResult::Cancelled,
        t::ResultKind::Approved { state, words_present } => w::WordResult::Approved(w::WordResultApproved {
            state: bytes::copy_of(state),
            words_present: *words_present,
        }),
        t::ResultKind::Change { connector, kind, resource } => {
            w::WordResult::Change(w::WordResultChange { connector: *connector, kind: *kind, resource: *resource })
        }
        t::ResultKind::Verdict { code, fields } => {
            let mut copied = List::with_capacity(super::super::count(fields.len())?);
            for row in fields {
                copied.push(crate::generated::client::result_field_to_record(row)).expect("supplied fields fit");
            }
            w::WordResult::Verdict(w::WordResultVerdict { code: *code, fields: copied })
        }
    })
}
fn crossing_words(kind: &t::MessageKind, words: &[u8]) -> Box<[u8]> {
    if let t::MessageKind::Result(t::ResultKind::Job) = kind { Box::new([]) } else { bytes::copy_of(words) }
}
pub fn word(value: &t::Word) -> Result<w::Word, w::Problem> {
    Ok(w::Word {
        number: value.number,
        from: owner::requester(value.from),
        kind: message_kind(&value.kind)?,
        words: crossing_words(&value.kind, &value.words),
        at: value.at.as_nanos(),
        hits: value.hits,
        eligible: value.eligible,
    })
}
pub fn answer(value: &t::ActivityLine) -> Result<w::ActivityLine, w::Problem> {
    let line = match &value.line {
        t::Line::Words { message, by, kind, words } => w::Activity::Words(w::ActivityWords {
            message: *message,
            by: owner::requester(*by),
            kind: message_kind(kind)?,
            words: crossing_words(kind, words),
        }),
        t::Line::Heard(value) => w::Activity::Heard(w::ActivityHeard { value: word(value)? }),
        t::Line::Notified { recipient, word: value } => {
            w::Activity::Notified(w::ActivityNotified { recipient: owner::requester(*recipient), word: word(value)? })
        }
        t::Line::RunBegan { .. }
        | t::Line::RunEnded { .. }
        | t::Line::Turn { .. }
        | t::Line::Decided { .. }
        | t::Line::NowWritten { .. }
        | t::Line::Stood { .. }
        | t::Line::TalkOpened { .. }
        | t::Line::TalkEnded { .. }
        | t::Line::SessionEnded { .. }
        | t::Line::Kept { .. }
        | t::Line::Milestone { .. } => {
            return Err(w::Problem { path: w::paths::ENDED_VALUE_QUESTION_ANSWER, reason: skein_codec::Reason::Tag });
        }
    };
    Ok(w::ActivityLine { task: value.task, position: value.position, at: value.at.as_nanos(), line })
}

fn settings_outcome(value: t::SettingsOutcome) -> w::SettingsOutcome {
    match value {
        t::SettingsOutcome::Accepted { revision } => w::SettingsOutcome::Accepted(w::SettingsAccepted { revision }),
        t::SettingsOutcome::Rejected => w::SettingsOutcome::Rejected,
        t::SettingsOutcome::Stale { newest } => w::SettingsOutcome::Stale(w::SettingsStale { newest }),
    }
}
