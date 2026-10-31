//! Wire projections of owner activity facts (client/domain/testing.md, 4).
use super::super::{bytes, c, list, w};
use super::{wire_hold_failure, wire_owner_failure_tries, wire_owner_requester, wire_result_field};
pub(in crate::codec) fn wire_owner_message_proposal_kind(
    value: &c::OwnerMessageProposalKind,
    _limits: &w::Limits,
) -> Result<w::OwnerMessageProposalKind, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerMessageProposalKind::Settings => w::OwnerMessageProposalKind::Settings,
        c::OwnerMessageProposalKind::Batch => w::OwnerMessageProposalKind::Batch,
        c::OwnerMessageProposalKind::Effect => w::OwnerMessageProposalKind::Effect,
        c::OwnerMessageProposalKind::Amend => w::OwnerMessageProposalKind::Amend,
        c::OwnerMessageProposalKind::Widen => w::OwnerMessageProposalKind::Widen,
        c::OwnerMessageProposalKind::Release => w::OwnerMessageProposalKind::Release,
    })
}

pub(in crate::codec) fn wire_owner_activity_line(
    value: &c::OwnerActivityLine,
    limits: &w::Limits,
) -> Result<w::OwnerActivityLine, w::Problem> {
    w::OwnerActivityLine::new(
        limits,
        w::OwnerActivityLineParts {
            task: value.task,
            position: value.position,
            at: value.at.as_nanos(),
            line: wire_owner_activity(&value.line, limits)?,
        },
    )
}

pub(in crate::codec) fn wire_owner_activity(
    value: &c::OwnerActivity,
    limits: &w::Limits,
) -> Result<w::OwnerActivity, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerActivity::Words { message, by, kind, words } => w::OwnerActivity::Words(w::OwnerActivityWords::new(
            limits,
            w::OwnerActivityWordsParts {
                message: *(message),
                by: wire_owner_requester(by, limits)?,
                kind: wire_owner_message_kind(kind, limits)?,
                words: bytes::copy_of((words).bytes()),
            },
        )?),
        c::OwnerActivity::Heard(value) => w::OwnerActivity::Heard(w::OwnerActivityHeard::new(
            limits,
            w::OwnerActivityHeardParts { value: wire_owner_word(value, limits)? },
        )?),
        c::OwnerActivity::Notified { recipient, word } => w::OwnerActivity::Notified(w::OwnerActivityNotified::new(
            limits,
            w::OwnerActivityNotifiedParts {
                recipient: wire_owner_requester(recipient, limits)?,
                word: wire_owner_word(word, limits)?,
            },
        )?),
        c::OwnerActivity::RunBegan { attempt, why, resumed } => {
            w::OwnerActivity::RunBegan(w::OwnerActivityRunBegan::new(
                limits,
                w::OwnerActivityRunBeganParts {
                    attempt: *(attempt),
                    why: wire_owner_run_why(why, limits)?,
                    resumed: *(resumed),
                },
            )?)
        }
        c::OwnerActivity::RunEnded { attempt, end, spent } => {
            w::OwnerActivity::RunEnded(w::OwnerActivityRunEnded::new(
                limits,
                w::OwnerActivityRunEndedParts {
                    attempt: *(attempt),
                    end: wire_owner_run_end(end, limits)?,
                    spent: *(spent),
                },
            )?)
        }
        c::OwnerActivity::Turn { reply, attempt, turn, read } => w::OwnerActivity::Turn(w::OwnerActivityTurn::new(
            limits,
            w::OwnerActivityTurnParts {
                reply: *(reply),
                attempt: *(attempt),
                turn: *(turn),
                read: (read).as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            },
        )?),
        c::OwnerActivity::Decided { revision } => w::OwnerActivity::Decided(w::OwnerActivityDecided::new(
            limits,
            w::OwnerActivityDecidedParts { revision: *(revision) },
        )?),
        c::OwnerActivity::Milestone { position } => w::OwnerActivity::Milestone(w::OwnerActivityMilestone::new(
            limits,
            w::OwnerActivityMilestoneParts { position: *(position) },
        )?),
        c::OwnerActivity::Kept { attempt, pending, question } => w::OwnerActivity::Kept(w::OwnerActivityKept::new(
            limits,
            w::OwnerActivityKeptParts {
                attempt: *(attempt),
                pending: list((pending).iter().map(|item| Ok::<_, w::Problem>(*(item))))?,
                question: (question).as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            },
        )?),
    })
}

pub(in crate::codec) fn wire_owner_message_kind(
    value: &c::OwnerMessageKind,
    limits: &w::Limits,
) -> Result<w::OwnerMessageKind, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerMessageKind::QuestionOwed => w::OwnerMessageKind::QuestionOwed,
        c::OwnerMessageKind::SettingsDecision { proposal, outcome } => {
            w::OwnerMessageKind::SettingsDecision(w::OwnerMessageKindSettingsDecision::new(
                limits,
                w::OwnerMessageKindSettingsDecisionParts {
                    proposal: *(proposal),
                    outcome: wire_settings_outcome(outcome, limits)?,
                },
            )?)
        }
        c::OwnerMessageKind::Escalation { task, revision } => {
            w::OwnerMessageKind::Escalation(w::OwnerMessageKindEscalation::new(
                limits,
                w::OwnerMessageKindEscalationParts { task: *(task), revision: *(revision) },
            )?)
        }
        c::OwnerMessageKind::Proposal { proposer, proposal, kind } => {
            w::OwnerMessageKind::Proposal(w::OwnerMessageKindProposal::new(
                limits,
                w::OwnerMessageKindProposalParts {
                    proposer: *(proposer),
                    proposal: *(proposal),
                    kind: wire_owner_message_proposal_kind(kind, limits)?,
                },
            )?)
        }
        c::OwnerMessageKind::PersonStalled { task } => w::OwnerMessageKind::PersonStalled(
            w::OwnerMessageKindPersonStalled::new(limits, w::OwnerMessageKindPersonStalledParts { task: *(task) })?,
        ),
        c::OwnerMessageKind::ProposalDecision { proposal, accepted } => {
            w::OwnerMessageKind::ProposalDecision(w::OwnerMessageKindProposalDecision::new(
                limits,
                w::OwnerMessageKindProposalDecisionParts { proposal: *(proposal), accepted: *(accepted) },
            )?)
        }
        c::OwnerMessageKind::QuestionAnswered { question } => w::OwnerMessageKind::QuestionAnswered(
            w::OwnerMessageKindAnswer::new(limits, w::OwnerMessageKindAnswerParts { question: *(question) })?,
        ),
        c::OwnerMessageKind::QuestionEnded { question } => w::OwnerMessageKind::QuestionEnded(
            w::OwnerMessageKindAnswer::new(limits, w::OwnerMessageKindAnswerParts { question: *(question) })?,
        ),
        c::OwnerMessageKind::Words => w::OwnerMessageKind::Words,
        c::OwnerMessageKind::EffectNotMade { entry } => w::OwnerMessageKind::EffectNotMade(
            w::OwnerMessageKindEffectNotMade::new(limits, w::OwnerMessageKindEffectNotMadeParts { entry: *(entry) })?,
        ),
        c::OwnerMessageKind::Amendment { revision } => w::OwnerMessageKind::Amendment(
            w::OwnerMessageKindAmendment::new(limits, w::OwnerMessageKindAmendmentParts { revision: *(revision) })?,
        ),
        c::OwnerMessageKind::Question => w::OwnerMessageKind::Question,
        c::OwnerMessageKind::Answer { question } => w::OwnerMessageKind::Answer(w::OwnerMessageKindAnswer::new(
            limits,
            w::OwnerMessageKindAnswerParts { question: *(question) },
        )?),
        c::OwnerMessageKind::Notice { subscription, target, state } => {
            w::OwnerMessageKind::Notice(w::OwnerMessageKindNotice::new(
                limits,
                w::OwnerMessageKindNoticeParts {
                    subscription: *(subscription),
                    target: *(target),
                    state: wire_owner_notice_state(state, limits)?,
                },
            )?)
        }
        c::OwnerMessageKind::Timer { subscription } => w::OwnerMessageKind::Timer(w::OwnerMessageKindTimer::new(
            limits,
            w::OwnerMessageKindTimerParts { subscription: *(subscription) },
        )?),
        c::OwnerMessageKind::News { subscription, class } => w::OwnerMessageKind::News(w::OwnerMessageKindNews::new(
            limits,
            w::OwnerMessageKindNewsParts {
                subscription: *(subscription),
                class: wire_owner_news_class(class, limits)?,
            },
        )?),
        c::OwnerMessageKind::Result(value) => w::OwnerMessageKind::Result(w::OwnerMessageKindResult::new(
            limits,
            w::OwnerMessageKindResultParts { value: wire_owner_word_result(value, limits)? },
        )?),
        c::OwnerMessageKind::Met { dependency, removed } => w::OwnerMessageKind::Met(w::OwnerMessageKindMet::new(
            limits,
            w::OwnerMessageKindMetParts { dependency: *(dependency), removed: *(removed) },
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

pub(in crate::codec) fn wire_owner_notice_state(
    value: &c::OwnerNoticeState,
    _limits: &w::Limits,
) -> Result<w::OwnerNoticeState, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerNoticeState::Held => w::OwnerNoticeState::Held,
        c::OwnerNoticeState::Done => w::OwnerNoticeState::Done,
        c::OwnerNoticeState::Failed => w::OwnerNoticeState::Failed,
        c::OwnerNoticeState::Cancelled => w::OwnerNoticeState::Cancelled,
    })
}

pub(in crate::codec) fn wire_owner_news_class(
    value: &c::OwnerNewsClass,
    _limits: &w::Limits,
) -> Result<w::OwnerNewsClass, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerNewsClass::Wakes => w::OwnerNewsClass::Wakes,
        c::OwnerNewsClass::Kept => w::OwnerNewsClass::Kept,
        c::OwnerNewsClass::Dropped => w::OwnerNewsClass::Dropped,
    })
}

pub(in crate::codec) fn wire_owner_word_result(
    value: &c::OwnerWordResult,
    limits: &w::Limits,
) -> Result<w::OwnerWordResult, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerWordResult::Approved { state, words_present } => {
            w::OwnerWordResult::Approved(w::OwnerWordResultApproved::new(
                limits,
                w::OwnerWordResultApprovedParts {
                    state: bytes::copy_of((state).bytes()),
                    words_present: *(words_present),
                },
            )?)
        }
        c::OwnerWordResult::Report => w::OwnerWordResult::Report,
        c::OwnerWordResult::Verdict { code, fields } => w::OwnerWordResult::Verdict(w::OwnerWordResultVerdict::new(
            limits,
            w::OwnerWordResultVerdictParts {
                code: *(code),
                fields: list((fields).iter().map(|item| wire_result_field(item, limits)))?,
            },
        )?),
        c::OwnerWordResult::Change { connector, kind, resource } => {
            w::OwnerWordResult::Change(w::OwnerWordResultChange::new(
                limits,
                w::OwnerWordResultChangeParts { connector: *(connector), kind: *(kind), resource: *(resource) },
            )?)
        }
        c::OwnerWordResult::Failed => w::OwnerWordResult::Failed,
        c::OwnerWordResult::Cancelled => w::OwnerWordResult::Cancelled,
    })
}

pub(in crate::codec) fn wire_owner_word(value: &c::OwnerWord, limits: &w::Limits) -> Result<w::OwnerWord, w::Problem> {
    w::OwnerWord::new(
        limits,
        w::OwnerWordParts {
            number: value.number,
            from: wire_owner_requester(&value.from, limits)?,
            kind: wire_owner_message_kind(&value.kind, limits)?,
            words: bytes::copy_of(value.words.bytes()),
            at: value.at.as_nanos(),
            hits: value.hits,
            eligible: value.eligible,
        },
    )
}

pub(in crate::codec) fn wire_owner_run_why(
    value: &c::OwnerRunWhy,
    limits: &w::Limits,
) -> Result<w::OwnerRunWhy, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerRunWhy::Made => w::OwnerRunWhy::Made,
        c::OwnerRunWhy::Ready => w::OwnerRunWhy::Ready,
        c::OwnerRunWhy::Message { message } => w::OwnerRunWhy::Message(w::OwnerRunWhyMessage::new(
            limits,
            w::OwnerRunWhyMessageParts { message: *(message) },
        )?),
        c::OwnerRunWhy::Released => w::OwnerRunWhy::Released,
        c::OwnerRunWhy::Retry { tries } => w::OwnerRunWhy::Retry(w::OwnerRunWhyRetry::new(
            limits,
            w::OwnerRunWhyRetryParts { tries: wire_owner_failure_tries(tries, limits)? },
        )?),
        c::OwnerRunWhy::Narrowed => w::OwnerRunWhy::Narrowed,
        c::OwnerRunWhy::Changed => w::OwnerRunWhy::Changed,
    })
}

pub(in crate::codec) fn wire_owner_run_end(
    value: &c::OwnerRunEnd,
    limits: &w::Limits,
) -> Result<w::OwnerRunEnd, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerRunEnd::Parked => w::OwnerRunEnd::Parked,
        c::OwnerRunEnd::Finished => w::OwnerRunEnd::Finished,
        c::OwnerRunEnd::Cancelled => w::OwnerRunEnd::Cancelled,
        c::OwnerRunEnd::Failed { class } => w::OwnerRunEnd::Failed(w::OwnerRunEndFailed::new(
            limits,
            w::OwnerRunEndFailedParts { class: wire_hold_failure(class, limits)? },
        )?),
        c::OwnerRunEnd::Refused => w::OwnerRunEnd::Refused,
    })
}
