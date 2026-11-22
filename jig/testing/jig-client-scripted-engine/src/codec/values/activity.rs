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
        c::SessionEnd::Talk => w::SessionEnd::Talk,
        c::SessionEnd::Left => w::SessionEnd::Left,
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
    Ok(w::ActivityLine {
        task: value.task,
        position: value.position,
        at: value.at.as_nanos(),
        line: wire_activity(&value.line, limits)?,
    })
}

pub(in crate::codec) fn wire_activity(value: &c::Activity, limits: &w::Limits) -> Result<w::Activity, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::Activity::Words { message, by, kind, words } => w::Activity::Words(w::ActivityWords {
            message: *(message),
            by: wire_requester(by, limits)?,
            kind: wire_message_kind(kind, limits)?,
            words: bytes::copy_of((words).bytes()),
        }),
        c::Activity::Heard(value) => w::Activity::Heard(w::ActivityHeard { value: wire_word(value, limits)? }),
        c::Activity::Notified { recipient, word } => w::Activity::Notified(w::ActivityNotified {
            recipient: wire_requester(recipient, limits)?,
            word: wire_word(word, limits)?,
        }),
        c::Activity::RunBegan { attempt, why, resumed, fresh: cause } => w::Activity::RunBegan(w::ActivityRunBegan {
            attempt: *(attempt),
            why: wire_run_why(why, limits)?,
            resumed: *(resumed),
            fresh: fresh(*cause),
        }),
        c::Activity::RunEnded { attempt, end, spent } => w::Activity::RunEnded(w::ActivityRunEnded {
            attempt: *(attempt),
            end: wire_run_end(end, limits)?,
            spent: *(spent),
        }),
        c::Activity::Turn { reply, attempt, turn, read } => w::Activity::Turn(w::ActivityTurn {
            reply: *(reply),
            attempt: *(attempt),
            turn: *(turn),
            read: (read).as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
        }),
        c::Activity::Decided { revision, proposal } => w::Activity::Decided(w::ActivityDecided {
            revision: *(revision),
            proposal: proposal
                .as_ref()
                .map(|value| w::ActivityDecidedProposal { number: value.number, made: value.made }),
        }),
        c::Activity::Milestone { position } => w::Activity::Milestone(w::ActivityMilestone { position: *(position) }),
        c::Activity::NowWritten { attempt, line } => {
            w::Activity::NowWritten(w::ActivityNowWritten { attempt: *attempt, line: line.bytes().into() })
        }
        c::Activity::Stood { attempt, words } => {
            w::Activity::Stood(w::ActivityStood { attempt: *attempt, words: words.bytes().into() })
        }
        c::Activity::TalkOpened { party } => w::Activity::TalkOpened(w::TalkOpened { party: *party }),
        c::Activity::TalkEnded { party, lapsed } => {
            w::Activity::TalkEnded(w::TalkEnded { party: *party, lapsed: *lapsed })
        }
        c::Activity::SessionEnded { attempt, end } => {
            w::Activity::SessionEnded(w::ActivitySessionEnded { attempt: *attempt, end: session_end(*end) })
        }
        c::Activity::Kept { attempt, pending, question } => w::Activity::Kept(w::ActivityKept {
            attempt: *(attempt),
            pending: list((pending).iter().map(|item| Ok::<_, w::Problem>(*(item))))?,
            question: (question).as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
        }),
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
            w::MessageKind::SettingsDecision(w::MessageKindSettingsDecision {
                proposal: *(proposal),
                outcome: wire_settings_outcome(outcome, limits)?,
            })
        }
        c::MessageKind::Escalation { task, revision } => {
            w::MessageKind::Escalation(w::MessageKindEscalation { task: *(task), revision: *(revision) })
        }
        c::MessageKind::Proposal { proposer, proposal, kind } => w::MessageKind::Proposal(w::MessageKindProposal {
            proposer: *(proposer),
            proposal: *(proposal),
            kind: wire_message_proposal_kind(kind, limits)?,
        }),
        c::MessageKind::PersonStalled { task } => {
            w::MessageKind::PersonStalled(w::MessageKindPersonStalled { task: *(task) })
        }
        c::MessageKind::ProposalDecision { proposal, accepted } => {
            w::MessageKind::ProposalDecision(w::MessageKindProposalDecision {
                proposal: *(proposal),
                accepted: *(accepted),
            })
        }
        c::MessageKind::QuestionAnswered { question } => {
            w::MessageKind::QuestionAnswered(w::MessageKindAnswer { question: *(question) })
        }
        c::MessageKind::QuestionEnded { question } => {
            w::MessageKind::QuestionEnded(w::MessageKindAnswer { question: *(question) })
        }
        c::MessageKind::Words => w::MessageKind::Words,
        c::MessageKind::EffectNotMade { entry } => {
            w::MessageKind::EffectNotMade(w::MessageKindEffectNotMade { entry: *(entry) })
        }
        c::MessageKind::Amendment { revision } => {
            w::MessageKind::Amendment(w::MessageKindAmendment { revision: *(revision) })
        }
        c::MessageKind::Question => w::MessageKind::Question,
        c::MessageKind::Answer { question } => w::MessageKind::Answer(w::MessageKindAnswer { question: *(question) }),
        c::MessageKind::Notice { subscription, target, state } => w::MessageKind::Notice(w::MessageKindNotice {
            subscription: *(subscription),
            target: *(target),
            state: wire_notice_state(state, limits)?,
        }),
        c::MessageKind::Timer { subscription } => {
            w::MessageKind::Timer(w::MessageKindTimer { subscription: *(subscription) })
        }
        c::MessageKind::News { subscription, class } => w::MessageKind::News(w::MessageKindNews {
            subscription: *(subscription),
            class: wire_news_class(class, limits)?,
        }),
        c::MessageKind::Result(value) => {
            w::MessageKind::Result(w::MessageKindResult { value: wire_word_result(value, limits)? })
        }
        c::MessageKind::Met { dependency, removed } => {
            w::MessageKind::Met(w::MessageKindMet { dependency: *(dependency), removed: *(removed) })
        }
    })
}

pub(in crate::codec) fn wire_settings_outcome(
    value: &c::SettingsOutcome,
    _limits: &w::Limits,
) -> Result<w::SettingsOutcome, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::SettingsOutcome::Accepted { revision } => {
            w::SettingsOutcome::Accepted(w::SettingsAccepted { revision: *(revision) })
        }
        c::SettingsOutcome::Rejected => w::SettingsOutcome::Rejected,
        c::SettingsOutcome::Stale { newest } => w::SettingsOutcome::Stale(w::SettingsStale { newest: *(newest) }),
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
        c::WordResult::Approved { state, words_present } => w::WordResult::Approved(w::WordResultApproved {
            state: bytes::copy_of((state).bytes()),
            words_present: *(words_present),
        }),
        c::WordResult::Report => w::WordResult::Report,
        c::WordResult::Verdict { code, fields } => w::WordResult::Verdict(w::WordResultVerdict {
            code: *(code),
            fields: list((fields).iter().map(|item| wire_result_field(item, limits)))?,
        }),
        c::WordResult::Change { connector, kind, resource } => {
            w::WordResult::Change(w::WordResultChange { connector: *(connector), kind: *(kind), resource: *(resource) })
        }
        c::WordResult::Failed => w::WordResult::Failed,
        c::WordResult::Cancelled => w::WordResult::Cancelled,
        c::WordResult::Job => w::WordResult::Job,
    })
}

pub(in crate::codec) fn wire_word(value: &c::Word, limits: &w::Limits) -> Result<w::Word, w::Problem> {
    Ok(w::Word {
        number: value.number,
        from: wire_requester(&value.from, limits)?,
        kind: wire_message_kind(&value.kind, limits)?,
        words: bytes::copy_of(value.words.bytes()),
        at: value.at.as_nanos(),
        hits: value.hits,
        eligible: value.eligible,
    })
}

pub(in crate::codec) fn wire_run_why(value: &c::RunWhy, limits: &w::Limits) -> Result<w::RunWhy, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::RunWhy::Made => w::RunWhy::Made,
        c::RunWhy::Ready => w::RunWhy::Ready,
        c::RunWhy::Message { message } => w::RunWhy::Message(w::RunWhyMessage { message: *(message) }),
        c::RunWhy::Released => w::RunWhy::Released,
        c::RunWhy::Retry { tries } => w::RunWhy::Retry(w::RunWhyRetry { tries: wire_failure_tries(tries, limits)? }),
        c::RunWhy::Narrowed => w::RunWhy::Narrowed,
        c::RunWhy::Changed => w::RunWhy::Changed,
    })
}

pub(in crate::codec) fn wire_run_end(value: &c::RunEnd, limits: &w::Limits) -> Result<w::RunEnd, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::RunEnd::Parked => w::RunEnd::Parked,
        c::RunEnd::Finished => w::RunEnd::Finished,
        c::RunEnd::Cancelled => w::RunEnd::Cancelled,
        c::RunEnd::Failed { class, invalid_reason } => w::RunEnd::Failed(w::RunEndFailed {
            class: wire_hold_failure(class, limits)?,
            invalid_reason: invalid_reason.map(wire_invalid_reason),
        }),
        c::RunEnd::Refused => w::RunEnd::Refused,
        c::RunEnd::Suspended { worker, why } => w::RunEnd::Suspended(w::RunEndSuspended {
            worker: *worker,
            why: match why {
                c::SuspendWhy::Stopped => w::SuspendWhy::Stopped,
                c::SuspendWhy::Restarted => w::SuspendWhy::Restarted,
            },
        }),
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

fn fresh(value: Option<c::RunFresh>) -> Option<w::RunFresh> {
    value.map(run_fresh)
}
fn run_fresh(value: c::RunFresh) -> w::RunFresh {
    match value {
        c::RunFresh::Moved => w::RunFresh::Moved,
        c::RunFresh::Closed => w::RunFresh::Closed,
        c::RunFresh::Cut => w::RunFresh::Cut,
        c::RunFresh::Limit => w::RunFresh::Limit,
        c::RunFresh::Session => w::RunFresh::Session,
    }
}
