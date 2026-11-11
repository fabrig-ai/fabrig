//! Wire projections of owner activity facts (client/domain/testing.md, 4).
use super::super::{bytes, c, list, w};
use super::{wire_failure_tries, wire_hold_failure, wire_requester, wire_result_field};
fn session_end(value: c::SessionEnd) -> w::SessionEnd {
    match value {
        c::SessionEnd::Quiet => w::SessionEnd::Quiet,
        c::SessionEnd::Bound => w::SessionEnd::Bound,
        c::SessionEnd::Unoffered => w::SessionEnd::Unoffered,
        c::SessionEnd::Fresh => w::SessionEnd::Fresh,
        c::SessionEnd::Closed => w::SessionEnd::Closed,
        c::SessionEnd::Rules => w::SessionEnd::Rules,
    }
}

pub(in crate::codec) fn wire_message_proposal_kind(
    value: &c::MessageProposalKind,
    _limits: &w::Limits,
) -> Result<w::MessageProposalKind, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::MessageProposalKind::Settings => w::MessageProposalKind::Settings,
        c::MessageProposalKind::Batch => w::MessageProposalKind::Batch,
        c::MessageProposalKind::Effect => w::MessageProposalKind::Effect,
        c::MessageProposalKind::Amend => w::MessageProposalKind::Amend,
        c::MessageProposalKind::Widen => w::MessageProposalKind::Widen,
        c::MessageProposalKind::Release => w::MessageProposalKind::Release,
    })
}

pub(in crate::codec) fn wire_activity_line(
    value: &c::ActivityLine,
    limits: &w::Limits,
) -> Result<w::ActivityLine, w::Problem> {
    w::ActivityLine::new(
        limits,
        w::ActivityLineParts {
            task: value.task,
            position: value.position,
            at: value.at.as_nanos(),
            line: wire_activity(&value.line, limits)?,
        },
    )
}

pub(in crate::codec) fn wire_activity(value: &c::Activity, limits: &w::Limits) -> Result<w::Activity, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::Activity::Words { message, by, kind, words } => w::Activity::Words(w::ActivityWords::new(
            limits,
            w::ActivityWordsParts {
                message: *(message),
                by: wire_requester(by, limits)?,
                kind: wire_message_kind(kind, limits)?,
                words: bytes::copy_of((words).bytes()),
            },
        )?),
        c::Activity::Heard(value) => w::Activity::Heard(w::ActivityHeard::new(
            limits,
            w::ActivityHeardParts { value: wire_word(value, limits)? },
        )?),
        c::Activity::Notified { recipient, word } => w::Activity::Notified(w::ActivityNotified::new(
            limits,
            w::ActivityNotifiedParts { recipient: wire_requester(recipient, limits)?, word: wire_word(word, limits)? },
        )?),
        c::Activity::RunBegan { attempt, why, resumed } => w::Activity::RunBegan(w::ActivityRunBegan::new(
            limits,
            w::ActivityRunBeganParts { attempt: *(attempt), why: wire_run_why(why, limits)?, resumed: *(resumed) },
        )?),
        c::Activity::RunEnded { attempt, end, spent } => w::Activity::RunEnded(w::ActivityRunEnded::new(
            limits,
            w::ActivityRunEndedParts { attempt: *(attempt), end: wire_run_end(end, limits)?, spent: *(spent) },
        )?),
        c::Activity::Turn { reply, attempt, turn, read } => w::Activity::Turn(w::ActivityTurn::new(
            limits,
            w::ActivityTurnParts {
                reply: *(reply),
                attempt: *(attempt),
                turn: *(turn),
                read: (read).as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            },
        )?),
        c::Activity::Decided { revision, proposal } => w::Activity::Decided(w::ActivityDecided::new(
            limits,
            w::ActivityDecidedParts {
                revision: *(revision),
                proposal: match proposal {
                    Some(value) => Some(w::ActivityDecidedProposal::new(
                        limits,
                        w::ActivityDecidedProposalParts { number: value.number, made: value.made },
                    )?),
                    None => None,
                },
            },
        )?),
        c::Activity::Milestone { position } => w::Activity::Milestone(w::ActivityMilestone::new(
            limits,
            w::ActivityMilestoneParts { position: *(position) },
        )?),
        c::Activity::Stood { attempt, words } => w::Activity::Stood(w::ActivityStood::new(
            limits,
            w::ActivityStoodParts { attempt: *attempt, words: words.bytes().into() },
        )?),
        c::Activity::SessionEnded { attempt, end } => w::Activity::SessionEnded(w::ActivitySessionEnded::new(
            limits,
            w::ActivitySessionEndedParts { attempt: *attempt, end: session_end(*end) },
        )?),
        c::Activity::Kept { attempt, pending, question } => w::Activity::Kept(w::ActivityKept::new(
            limits,
            w::ActivityKeptParts {
                attempt: *(attempt),
                pending: list((pending).iter().map(|item| Ok::<_, w::Problem>(*(item))))?,
                question: (question).as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            },
        )?),
    })
}

pub(in crate::codec) fn wire_message_kind(
    value: &c::MessageKind,
    limits: &w::Limits,
) -> Result<w::MessageKind, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::MessageKind::QuestionOwed => w::MessageKind::QuestionOwed,
        c::MessageKind::NothingAwaited => w::MessageKind::NothingAwaited,
        c::MessageKind::SettingsDecision { proposal, outcome } => {
            w::MessageKind::SettingsDecision(w::MessageKindSettingsDecision::new(
                limits,
                w::MessageKindSettingsDecisionParts {
                    proposal: *(proposal),
                    outcome: wire_settings_outcome(outcome, limits)?,
                },
            )?)
        }
        c::MessageKind::Escalation { task, revision } => w::MessageKind::Escalation(w::MessageKindEscalation::new(
            limits,
            w::MessageKindEscalationParts { task: *(task), revision: *(revision) },
        )?),
        c::MessageKind::Proposal { proposer, proposal, kind } => w::MessageKind::Proposal(w::MessageKindProposal::new(
            limits,
            w::MessageKindProposalParts {
                proposer: *(proposer),
                proposal: *(proposal),
                kind: wire_message_proposal_kind(kind, limits)?,
            },
        )?),
        c::MessageKind::PersonStalled { task } => w::MessageKind::PersonStalled(w::MessageKindPersonStalled::new(
            limits,
            w::MessageKindPersonStalledParts { task: *(task) },
        )?),
        c::MessageKind::ProposalDecision { proposal, accepted } => {
            w::MessageKind::ProposalDecision(w::MessageKindProposalDecision::new(
                limits,
                w::MessageKindProposalDecisionParts { proposal: *(proposal), accepted: *(accepted) },
            )?)
        }
        c::MessageKind::QuestionAnswered { question } => w::MessageKind::QuestionAnswered(w::MessageKindAnswer::new(
            limits,
            w::MessageKindAnswerParts { question: *(question) },
        )?),
        c::MessageKind::QuestionEnded { question } => w::MessageKind::QuestionEnded(w::MessageKindAnswer::new(
            limits,
            w::MessageKindAnswerParts { question: *(question) },
        )?),
        c::MessageKind::Words => w::MessageKind::Words,
        c::MessageKind::EffectNotMade { entry } => w::MessageKind::EffectNotMade(w::MessageKindEffectNotMade::new(
            limits,
            w::MessageKindEffectNotMadeParts { entry: *(entry) },
        )?),
        c::MessageKind::Amendment { revision } => w::MessageKind::Amendment(w::MessageKindAmendment::new(
            limits,
            w::MessageKindAmendmentParts { revision: *(revision) },
        )?),
        c::MessageKind::Question => w::MessageKind::Question,
        c::MessageKind::Answer { question } => w::MessageKind::Answer(w::MessageKindAnswer::new(
            limits,
            w::MessageKindAnswerParts { question: *(question) },
        )?),
        c::MessageKind::Notice { subscription, target, state } => w::MessageKind::Notice(w::MessageKindNotice::new(
            limits,
            w::MessageKindNoticeParts {
                subscription: *(subscription),
                target: *(target),
                state: wire_notice_state(state, limits)?,
            },
        )?),
        c::MessageKind::Timer { subscription } => w::MessageKind::Timer(w::MessageKindTimer::new(
            limits,
            w::MessageKindTimerParts { subscription: *(subscription) },
        )?),
        c::MessageKind::News { subscription, class } => w::MessageKind::News(w::MessageKindNews::new(
            limits,
            w::MessageKindNewsParts { subscription: *(subscription), class: wire_news_class(class, limits)? },
        )?),
        c::MessageKind::Result(value) => w::MessageKind::Result(w::MessageKindResult::new(
            limits,
            w::MessageKindResultParts { value: wire_word_result(value, limits)? },
        )?),
        c::MessageKind::Met { dependency, removed } => w::MessageKind::Met(w::MessageKindMet::new(
            limits,
            w::MessageKindMetParts { dependency: *(dependency), removed: *(removed) },
        )?),
    })
}

pub(in crate::codec) fn wire_settings_outcome(
    value: &c::SettingsOutcome,
    limits: &w::Limits,
) -> Result<w::SettingsOutcome, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::SettingsOutcome::Accepted { revision } => w::SettingsOutcome::Accepted(w::SettingsAccepted::new(
            limits,
            w::SettingsAcceptedParts { revision: *(revision) },
        )?),
        c::SettingsOutcome::Rejected => w::SettingsOutcome::Rejected,
        c::SettingsOutcome::Stale { newest } => {
            w::SettingsOutcome::Stale(w::SettingsStale::new(limits, w::SettingsStaleParts { newest: *(newest) })?)
        }
    })
}

pub(in crate::codec) fn wire_notice_state(
    value: &c::NoticeState,
    _limits: &w::Limits,
) -> Result<w::NoticeState, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::NoticeState::Held => w::NoticeState::Held,
        c::NoticeState::Done => w::NoticeState::Done,
        c::NoticeState::Failed => w::NoticeState::Failed,
        c::NoticeState::Cancelled => w::NoticeState::Cancelled,
    })
}

pub(in crate::codec) fn wire_news_class(value: &c::NewsClass, _limits: &w::Limits) -> Result<w::NewsClass, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::NewsClass::Wakes => w::NewsClass::Wakes,
        c::NewsClass::Kept => w::NewsClass::Kept,
        c::NewsClass::Dropped => w::NewsClass::Dropped,
    })
}

pub(in crate::codec) fn wire_word_result(
    value: &c::WordResult,
    limits: &w::Limits,
) -> Result<w::WordResult, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::WordResult::Approved { state, words_present } => w::WordResult::Approved(w::WordResultApproved::new(
            limits,
            w::WordResultApprovedParts { state: bytes::copy_of((state).bytes()), words_present: *(words_present) },
        )?),
        c::WordResult::Report => w::WordResult::Report,
        c::WordResult::Verdict { code, fields } => w::WordResult::Verdict(w::WordResultVerdict::new(
            limits,
            w::WordResultVerdictParts {
                code: *(code),
                fields: list((fields).iter().map(|item| wire_result_field(item, limits)))?,
            },
        )?),
        c::WordResult::Change { connector, kind, resource } => w::WordResult::Change(w::WordResultChange::new(
            limits,
            w::WordResultChangeParts { connector: *(connector), kind: *(kind), resource: *(resource) },
        )?),
        c::WordResult::Failed => w::WordResult::Failed,
        c::WordResult::Cancelled => w::WordResult::Cancelled,
    })
}

pub(in crate::codec) fn wire_word(value: &c::Word, limits: &w::Limits) -> Result<w::Word, w::Problem> {
    w::Word::new(
        limits,
        w::WordParts {
            number: value.number,
            from: wire_requester(&value.from, limits)?,
            kind: wire_message_kind(&value.kind, limits)?,
            words: bytes::copy_of(value.words.bytes()),
            at: value.at.as_nanos(),
            hits: value.hits,
            eligible: value.eligible,
        },
    )
}

pub(in crate::codec) fn wire_run_why(value: &c::RunWhy, limits: &w::Limits) -> Result<w::RunWhy, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::RunWhy::Made => w::RunWhy::Made,
        c::RunWhy::Ready => w::RunWhy::Ready,
        c::RunWhy::Message { message } => {
            w::RunWhy::Message(w::RunWhyMessage::new(limits, w::RunWhyMessageParts { message: *(message) })?)
        }
        c::RunWhy::Released => w::RunWhy::Released,
        c::RunWhy::Retry { tries } => w::RunWhy::Retry(w::RunWhyRetry::new(
            limits,
            w::RunWhyRetryParts { tries: wire_failure_tries(tries, limits)? },
        )?),
        c::RunWhy::Narrowed => w::RunWhy::Narrowed,
        c::RunWhy::Changed => w::RunWhy::Changed,
    })
}

pub(in crate::codec) fn wire_run_end(value: &c::RunEnd, limits: &w::Limits) -> Result<w::RunEnd, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::RunEnd::Parked => w::RunEnd::Parked,
        c::RunEnd::Finished => w::RunEnd::Finished,
        c::RunEnd::Cancelled => w::RunEnd::Cancelled,
        c::RunEnd::Failed { class, invalid_reason } => w::RunEnd::Failed(w::RunEndFailed::new(
            limits,
            w::RunEndFailedParts {
                class: wire_hold_failure(class, limits)?,
                invalid_reason: invalid_reason.map(wire_invalid_reason),
            },
        )?),
        c::RunEnd::Refused => w::RunEnd::Refused,
    })
}

fn wire_invalid_reason(value: c::InvalidReason) -> w::InvalidReason {
    match value {
        c::InvalidReason::Form => w::InvalidReason::Form,
        c::InvalidReason::Verdict => w::InvalidReason::Verdict,
        c::InvalidReason::Words => w::InvalidReason::Words,
        c::InvalidReason::Change => w::InvalidReason::Change,
        c::InvalidReason::Followups => w::InvalidReason::Followups,
        c::InvalidReason::UnreadableResult => w::InvalidReason::UnreadableResult,
    }
}
