//! Whole pages and single documents (protocol/client.md, 3 and 6).
use super::{Limits, List, Wall, c, carried_turn_fields, text, too_large, w};
pub(crate) fn message_proposal_kind_fields(value: &w::MessageProposalKind) -> c::MessageProposalKind {
    match value {
        w::MessageProposalKind::Settings => c::MessageProposalKind::Settings,
        w::MessageProposalKind::Batch => c::MessageProposalKind::Batch,
        w::MessageProposalKind::Effect => c::MessageProposalKind::Effect,
        w::MessageProposalKind::Amend => c::MessageProposalKind::Amend,
        w::MessageProposalKind::Widen => c::MessageProposalKind::Widen,
        w::MessageProposalKind::Release => c::MessageProposalKind::Release,
    }
}

pub(crate) fn notice_state_fields(value: &w::NoticeState) -> c::NoticeState {
    match value {
        w::NoticeState::Held => c::NoticeState::Held,
        w::NoticeState::Done => c::NoticeState::Done,
        w::NoticeState::Failed => c::NoticeState::Failed,
        w::NoticeState::Cancelled => c::NoticeState::Cancelled,
    }
}

pub(crate) fn news_class_fields(value: &w::NewsClass) -> c::NewsClass {
    match value {
        w::NewsClass::Wakes => c::NewsClass::Wakes,
        w::NewsClass::Kept => c::NewsClass::Kept,
        w::NewsClass::Dropped => c::NewsClass::Dropped,
    }
}

pub(crate) fn word_result_fields(value: &w::WordResult, limits: &Limits) -> Result<c::WordResult, w::Problem> {
    Ok(match value {
        w::WordResult::Approved(row) => {
            c::WordResult::Approved { state: text(row.state(), limits)?, words_present: row.words_present() }
        }
        w::WordResult::Report => c::WordResult::Report,
        w::WordResult::Verdict(row) => {
            if row.fields().len() > limits.client().rows {
                return Err(too_large());
            }
            let mut list0 = List::with_capacity(row.fields().len());
            for item in row.fields() {
                list0.push(crate::translate::facts::result_field(item)).expect("reserved supplied list");
            }
            c::WordResult::Verdict { code: row.code(), fields: list0 }
        }
        w::WordResult::Change(row) => {
            c::WordResult::Change { connector: row.connector(), kind: row.kind(), resource: row.resource() }
        }
        w::WordResult::Failed => c::WordResult::Failed,
        w::WordResult::Cancelled => c::WordResult::Cancelled,
    })
}

pub(crate) fn message_kind_fields(value: &w::MessageKind, limits: &Limits) -> Result<c::MessageKind, w::Problem> {
    Ok(match value {
        w::MessageKind::SettingsDecision(row) => c::MessageKind::SettingsDecision {
            proposal: row.proposal(),
            outcome: match row.outcome() {
                w::SettingsOutcome::Accepted(value) => c::SettingsOutcome::Accepted { revision: value.revision() },
                w::SettingsOutcome::Rejected => c::SettingsOutcome::Rejected,
                w::SettingsOutcome::Stale(value) => c::SettingsOutcome::Stale { newest: value.newest() },
            },
        },
        w::MessageKind::Escalation(row) => c::MessageKind::Escalation { task: row.task(), revision: row.revision() },
        w::MessageKind::Proposal(row) => c::MessageKind::Proposal {
            proposer: row.proposer(),
            proposal: row.proposal(),
            kind: message_proposal_kind_fields(row.kind()),
        },
        w::MessageKind::PersonStalled(row) => c::MessageKind::PersonStalled { task: row.task() },
        w::MessageKind::ProposalDecision(row) => {
            c::MessageKind::ProposalDecision { proposal: row.proposal(), accepted: row.accepted() }
        }
        w::MessageKind::Met(row) => c::MessageKind::Met { dependency: row.dependency(), removed: row.removed() },
        w::MessageKind::QuestionAnswered(row) => c::MessageKind::QuestionAnswered { question: row.question() },
        w::MessageKind::QuestionOwed => c::MessageKind::QuestionOwed,
        w::MessageKind::NothingAwaited => c::MessageKind::NothingAwaited,
        w::MessageKind::QuestionEnded(row) => c::MessageKind::QuestionEnded { question: row.question() },
        w::MessageKind::Words => c::MessageKind::Words,
        w::MessageKind::EffectNotMade(row) => c::MessageKind::EffectNotMade { entry: row.entry() },
        w::MessageKind::Amendment(row) => c::MessageKind::Amendment { revision: row.revision() },
        w::MessageKind::Question => c::MessageKind::Question,
        w::MessageKind::Answer(row) => c::MessageKind::Answer { question: row.question() },
        w::MessageKind::Notice(row) => c::MessageKind::Notice {
            subscription: row.subscription(),
            target: row.target(),
            state: notice_state_fields(row.state()),
        },
        w::MessageKind::Timer(row) => c::MessageKind::Timer { subscription: row.subscription() },
        w::MessageKind::News(row) => {
            c::MessageKind::News { subscription: row.subscription(), class: news_class_fields(row.class()) }
        }
        w::MessageKind::Result(row) => c::MessageKind::Result(word_result_fields(row.value(), limits)?),
    })
}

pub(crate) fn word_fields(value: &w::Word, limits: &Limits) -> Result<c::Word, w::Problem> {
    Ok(c::Word {
        number: value.number(),
        from: crate::translate::facts::requester(value.from()),
        kind: message_kind_fields(value.kind(), limits)?,
        words: text(value.words(), limits)?,
        at: Wall::from_nanos(value.at()),
        hits: value.hits(),
        eligible: value.eligible(),
    })
}

pub(crate) fn run_why_fields(value: &w::RunWhy) -> c::RunWhy {
    match value {
        w::RunWhy::Made => c::RunWhy::Made,
        w::RunWhy::Ready => c::RunWhy::Ready,
        w::RunWhy::Message(row) => c::RunWhy::Message { message: row.message() },
        w::RunWhy::Released => c::RunWhy::Released,
        w::RunWhy::Retry(row) => c::RunWhy::Retry { tries: crate::translate::facts::failure_tries(row.tries()) },
        w::RunWhy::Narrowed => c::RunWhy::Narrowed,
        w::RunWhy::Changed => c::RunWhy::Changed,
    }
}

pub(crate) fn run_end_fields(value: &w::RunEnd) -> c::RunEnd {
    match value {
        w::RunEnd::Parked => c::RunEnd::Parked,
        w::RunEnd::Finished => c::RunEnd::Finished,
        w::RunEnd::Cancelled => c::RunEnd::Cancelled,
        w::RunEnd::Failed(row) => c::RunEnd::Failed { class: crate::translate::answers::hold_failure(row.class()) },
        w::RunEnd::Refused => c::RunEnd::Refused,
    }
}

pub(crate) fn activity_fields(value: &w::Activity, limits: &Limits) -> Result<c::Activity, w::Problem> {
    Ok(match value {
        w::Activity::Words(row) => c::Activity::Words {
            message: row.message(),
            by: crate::translate::facts::requester(row.by()),
            kind: message_kind_fields(row.kind(), limits)?,
            words: text(row.words(), limits)?,
        },
        w::Activity::Heard(row) => c::Activity::Heard(word_fields(row.value(), limits)?),
        w::Activity::Notified(row) => c::Activity::Notified {
            recipient: crate::translate::facts::requester(row.recipient()),
            word: word_fields(row.word(), limits)?,
        },
        w::Activity::RunBegan(row) => {
            c::Activity::RunBegan { attempt: row.attempt(), why: run_why_fields(row.why()), resumed: row.resumed() }
        }
        w::Activity::RunEnded(row) => {
            c::Activity::RunEnded { attempt: row.attempt(), end: run_end_fields(row.end()), spent: row.spent() }
        }
        w::Activity::Turn(row) => {
            let opt0 = match row.read() {
                Some(item) => Some(*item),
                None => None,
            };
            c::Activity::Turn { reply: row.reply(), attempt: row.attempt(), turn: row.turn(), read: opt0 }
        }
        w::Activity::Decided(row) => c::Activity::Decided { revision: row.revision() },
        w::Activity::Kept(row) => {
            c::Activity::Kept { attempt: row.attempt(), pending: row.pending().clone(), question: *row.question() }
        }
        w::Activity::Milestone(row) => c::Activity::Milestone { position: row.position() },
    })
}

pub(crate) fn activity_line_fields(value: &w::ActivityLine, limits: &Limits) -> Result<c::ActivityLine, w::Problem> {
    Ok(c::ActivityLine {
        task: value.task(),
        position: value.position(),
        at: Wall::from_nanos(value.at()),
        line: activity_fields(value.line(), limits)?,
    })
}

pub(crate) fn archived_turn_fields(value: &w::ArchivedTurn, limits: &Limits) -> c::ArchivedTurn {
    let opt0 = match value.read() {
        Some(item) => Some(*item),
        None => None,
    };
    c::ArchivedTurn {
        task: value.task(),
        name: turn_name_fields(value.name()),
        cumulative: value.cumulative(),
        read: opt0,
        at: Wall::from_nanos(value.at()),
        turn: match value.turn() {
            w::TurnBytes::Carried(row) => carried_turn_fields(row.bytes(), limits),
            w::TurnBytes::TooLarge(row) => {
                c::TurnRead::Undecodable { why: c::TurnProblem::TooLarge { bytes: row.bytes() } }
            }
        },
    }
}

pub(crate) fn turn_name_fields(value: &w::TurnName) -> c::TurnName {
    c::TurnName { attempt: value.attempt(), number: value.number() }
}
