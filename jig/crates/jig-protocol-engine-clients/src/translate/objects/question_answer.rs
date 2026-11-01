//! Whole single-read owner documents (protocol/client.md, 3 and 6).
use super::super::{owner, w};
use jig_domain_tasks as t;
use skein_lib::{List, bytes};
pub fn message_kind(value: &t::MessageKind, limits: &w::Limits) -> Result<w::MessageKind, w::Problem> {
    Ok(match value {
        t::MessageKind::Met { dependency, removed } => w::MessageKind::Met(w::MessageKindMet::new(
            limits,
            w::MessageKindMetParts { dependency: *dependency, removed: *removed },
        )?),
        t::MessageKind::SettingsDecision { proposal, outcome } => {
            w::MessageKind::SettingsDecision(w::MessageKindSettingsDecision::new(
                limits,
                w::MessageKindSettingsDecisionParts {
                    proposal: *proposal,
                    outcome: settings_outcome(*outcome, limits)?,
                },
            )?)
        }
        t::MessageKind::QuestionAnswered { question } => w::MessageKind::QuestionAnswered(w::MessageKindAnswer::new(
            limits,
            w::MessageKindAnswerParts { question: *question },
        )?),
        t::MessageKind::QuestionOwed => w::MessageKind::QuestionOwed,
        t::MessageKind::NothingAwaited => w::MessageKind::NothingAwaited,
        t::MessageKind::QuestionEnded { question } => w::MessageKind::QuestionEnded(w::MessageKindAnswer::new(
            limits,
            w::MessageKindAnswerParts { question: *question },
        )?),
        t::MessageKind::Words => w::MessageKind::Words,
        t::MessageKind::Question => w::MessageKind::Question,
        t::MessageKind::Escalation { task, revision } => w::MessageKind::Escalation(w::MessageKindEscalation::new(
            limits,
            w::MessageKindEscalationParts { task: *task, revision: *revision },
        )?),
        t::MessageKind::Proposal { proposer, proposal, kind } => w::MessageKind::Proposal(w::MessageKindProposal::new(
            limits,
            w::MessageKindProposalParts {
                proposer: *proposer,
                proposal: *proposal,
                kind: super::proposals::kind(*kind),
            },
        )?),
        t::MessageKind::PersonStalled { task } => w::MessageKind::PersonStalled(w::MessageKindPersonStalled::new(
            limits,
            w::MessageKindPersonStalledParts { task: *task },
        )?),
        t::MessageKind::ProposalDecision { proposal, accepted } => {
            w::MessageKind::ProposalDecision(w::MessageKindProposalDecision::new(
                limits,
                w::MessageKindProposalDecisionParts { proposal: *proposal, accepted: *accepted },
            )?)
        }
        t::MessageKind::EffectNotMade { entry } => w::MessageKind::EffectNotMade(w::MessageKindEffectNotMade::new(
            limits,
            w::MessageKindEffectNotMadeParts { entry: *entry },
        )?),
        t::MessageKind::Amendment { revision } | t::MessageKind::FinishRequested { revision } => {
            w::MessageKind::Amendment(w::MessageKindAmendment::new(
                limits,
                w::MessageKindAmendmentParts { revision: *revision },
            )?)
        }
        t::MessageKind::Answer { question } => w::MessageKind::Answer(w::MessageKindAnswer::new(
            limits,
            w::MessageKindAnswerParts { question: *question },
        )?),
        t::MessageKind::Timer { subscription } => w::MessageKind::Timer(w::MessageKindTimer::new(
            limits,
            w::MessageKindTimerParts { subscription: *subscription },
        )?),
        t::MessageKind::Notice { subscription, target, state } => w::MessageKind::Notice(w::MessageKindNotice::new(
            limits,
            w::MessageKindNoticeParts {
                subscription: *subscription,
                target: *target,
                state: match state {
                    t::NoticeState::Held { .. } => w::NoticeState::Held,
                    t::NoticeState::Done => w::NoticeState::Done,
                    t::NoticeState::Failed => w::NoticeState::Failed,
                    t::NoticeState::Cancelled => w::NoticeState::Cancelled,
                },
            },
        )?),
        t::MessageKind::News { subscription, class } => w::MessageKind::News(w::MessageKindNews::new(
            limits,
            w::MessageKindNewsParts {
                subscription: *subscription,
                class: match class {
                    t::NewsClass::Wakes => w::NewsClass::Wakes,
                    t::NewsClass::Kept => w::NewsClass::Kept,
                    t::NewsClass::Dropped => w::NewsClass::Dropped,
                },
            },
        )?),
        t::MessageKind::Result(value) => w::MessageKind::Result(w::MessageKindResult::new(
            limits,
            w::MessageKindResultParts { value: word_result(value, limits)? },
        )?),
    })
}
pub fn word_result(value: &t::ResultKind, limits: &w::Limits) -> Result<w::WordResult, w::Problem> {
    Ok(match value {
        t::ResultKind::Report => w::WordResult::Report,
        t::ResultKind::Failed => w::WordResult::Failed,
        t::ResultKind::Cancelled => w::WordResult::Cancelled,
        t::ResultKind::Approved { state, words_present } => w::WordResult::Approved(w::WordResultApproved::new(
            limits,
            w::WordResultApprovedParts { state: bytes::copy_of(state), words_present: *words_present },
        )?),
        t::ResultKind::Change { connector, kind, resource } => w::WordResult::Change(w::WordResultChange::new(
            limits,
            w::WordResultChangeParts { connector: *connector, kind: *kind, resource: *resource },
        )?),
        t::ResultKind::Verdict { code, fields } => {
            let mut copied = List::with_capacity(super::super::count(fields.len())?);
            for row in fields {
                copied
                    .push(w::ResultField::new(
                        limits,
                        w::ResultFieldParts { name: bytes::copy_of(&row.name), value: bytes::copy_of(&row.value) },
                    )?)
                    .expect("supplied fields fit");
            }
            w::WordResult::Verdict(w::WordResultVerdict::new(
                limits,
                w::WordResultVerdictParts { code: *code, fields: copied },
            )?)
        }
    })
}
pub fn word(value: &t::Word, limits: &w::Limits) -> Result<w::Word, w::Problem> {
    w::Word::new(
        limits,
        w::WordParts {
            number: value.number,
            from: owner::requester(value.from, limits)?,
            kind: message_kind(&value.kind, limits)?,
            words: bytes::copy_of(&value.words),
            at: value.at.as_nanos(),
            hits: value.hits,
            eligible: value.eligible,
        },
    )
}
pub fn answer(value: &t::ActivityLine, limits: &w::Limits) -> Result<w::ActivityLine, w::Problem> {
    let line = match &value.line {
        t::Line::Words { message, by, kind, words } => w::Activity::Words(w::ActivityWords::new(
            limits,
            w::ActivityWordsParts {
                message: *message,
                by: owner::requester(*by, limits)?,
                kind: message_kind(kind, limits)?,
                words: bytes::copy_of(words),
            },
        )?),
        t::Line::Heard(value) => {
            w::Activity::Heard(w::ActivityHeard::new(limits, w::ActivityHeardParts { value: word(value, limits)? })?)
        }
        t::Line::Notified { recipient, word: value } => w::Activity::Notified(w::ActivityNotified::new(
            limits,
            w::ActivityNotifiedParts { recipient: owner::requester(*recipient, limits)?, word: word(value, limits)? },
        )?),
        t::Line::RunBegan { .. }
        | t::Line::RunEnded { .. }
        | t::Line::Turn { .. }
        | t::Line::Decided { .. }
        | t::Line::Kept { .. }
        | t::Line::Milestone { .. } => {
            return Err(w::Problem { path: w::Path::EndedValueQuestionAnswer, reason: skein_codec::Reason::Tag });
        }
    };
    w::ActivityLine::new(
        limits,
        w::ActivityLineParts { task: value.task, position: value.position, at: value.at.as_nanos(), line },
    )
}

fn settings_outcome(value: t::SettingsOutcome, limits: &w::Limits) -> Result<w::SettingsOutcome, w::Problem> {
    Ok(match value {
        t::SettingsOutcome::Accepted { revision } => {
            w::SettingsOutcome::Accepted(w::SettingsAccepted::new(limits, w::SettingsAcceptedParts { revision })?)
        }
        t::SettingsOutcome::Rejected => w::SettingsOutcome::Rejected,
        t::SettingsOutcome::Stale { newest } => {
            w::SettingsOutcome::Stale(w::SettingsStale::new(limits, w::SettingsStaleParts { newest })?)
        }
    })
}
