//! Durable task values translated exhaustively (protocol/store/records.md, 1).
use super::{proposals, state, value};
use crate::tasks_v13 as wire;
use jig_domain_tasks as domain;
use skein_lib::Wall;

/// Translate `NoticeState` to its durable tag.
pub fn notice_state(source: domain::NoticeState, _limits: &wire::Limits) -> Result<wire::NoticeState, wire::Problem> {
    Ok(match source {
        domain::NoticeState::Held { revision } => wire::NoticeState::Held(wire::NoticeHeld { revision }),
        domain::NoticeState::Done => wire::NoticeState::Done,
        domain::NoticeState::Failed => wire::NoticeState::Failed,
        domain::NoticeState::Cancelled => wire::NoticeState::Cancelled,
    })
}
/// Read `NoticeState` from its durable tag.
#[must_use]
pub fn read_notice_state(source: wire::NoticeState) -> domain::NoticeState {
    match source {
        wire::NoticeState::Held(held) => domain::NoticeState::Held { revision: held.revision },
        wire::NoticeState::Done => domain::NoticeState::Done,
        wire::NoticeState::Failed => domain::NoticeState::Failed,
        wire::NoticeState::Cancelled => domain::NoticeState::Cancelled,
    }
}

/// Translate `NewsClass` to its durable tag.
#[must_use]
pub fn news_class(source: domain::NewsClass) -> wire::NewsClass {
    match source {
        domain::NewsClass::Wakes => wire::NewsClass::Wakes,
        domain::NewsClass::Kept => wire::NewsClass::Kept,
        domain::NewsClass::Dropped => wire::NewsClass::Dropped,
    }
}
/// Read `NewsClass` from its durable tag.
#[must_use]
pub fn read_news_class(source: wire::NewsClass) -> domain::NewsClass {
    match source {
        wire::NewsClass::Wakes => domain::NewsClass::Wakes,
        wire::NewsClass::Kept => domain::NewsClass::Kept,
        wire::NewsClass::Dropped => domain::NewsClass::Dropped,
    }
}

/// Translate `ResultKind` to its durable value.
pub fn result_kind(source: &domain::ResultKind, limits: &wire::Limits) -> Result<wire::ResultKind, wire::Problem> {
    Ok(match source {
        domain::ResultKind::Approved { state, words_present } => {
            wire::ResultKind::Approved(wire::ApprovalMessage { state: state.clone(), words_present: *words_present })
        }
        domain::ResultKind::Job => wire::ResultKind::Job,
        domain::ResultKind::NoChange => wire::ResultKind::NoChange,
        domain::ResultKind::Report => wire::ResultKind::Report,
        domain::ResultKind::Verdict { code, fields } => wire::ResultKind::Verdict(wire::VerdictMessage {
            code: *code,
            fields: super::pieces::fields(
                fields,
                limits,
                limits.verdict_message_fields,
                wire::paths::VERDICT_MESSAGE_FIELDS,
            )?,
        }),
        domain::ResultKind::Change { connector, kind, resource } => {
            wire::ResultKind::Change(wire::ChangeMessage { connector: *connector, kind: *kind, resource: *resource })
        }
        domain::ResultKind::Failed => wire::ResultKind::Failed,
        domain::ResultKind::Cancelled => wire::ResultKind::Cancelled,
    })
}
/// Read `ResultKind` from its durable value.
pub fn read_result_kind(source: wire::ResultKind, limits: &wire::Limits) -> Result<domain::ResultKind, wire::Problem> {
    Ok(match source {
        wire::ResultKind::Approved(source) => {
            let wire::ApprovalMessage { state, words_present } = source;
            domain::ResultKind::Approved { state, words_present }
        }
        wire::ResultKind::Job => domain::ResultKind::Job,
        wire::ResultKind::NoChange => domain::ResultKind::NoChange,
        wire::ResultKind::Report => domain::ResultKind::Report,
        wire::ResultKind::Verdict(source) => {
            let wire::VerdictMessage { code, fields } = source;
            domain::ResultKind::Verdict {
                code,
                fields: super::pieces::read_fields(fields, limits, wire::paths::VERDICT_MESSAGE_FIELDS)?,
            }
        }
        wire::ResultKind::Change(source) => {
            let wire::ChangeMessage { connector, kind, resource } = source;
            domain::ResultKind::Change { connector, kind, resource }
        }
        wire::ResultKind::Failed => domain::ResultKind::Failed,
        wire::ResultKind::Cancelled => domain::ResultKind::Cancelled,
    })
}

/// Translate `MessageKind` to its durable value.
pub fn message_kind(source: &domain::MessageKind, limits: &wire::Limits) -> Result<wire::MessageKind, wire::Problem> {
    Ok(match source {
        domain::MessageKind::Escalation { task, revision } => {
            wire::MessageKind::Escalation(wire::EscalationMessage { task: *task, revision: *revision })
        }
        domain::MessageKind::Proposal { proposer, proposal, kind } => {
            wire::MessageKind::Proposal(wire::ProposalMessage {
                proposer: *proposer,
                proposal: *proposal,
                kind: proposals::kind(kind)?,
            })
        }
        domain::MessageKind::PersonStalled { task } => {
            wire::MessageKind::PersonStalled(wire::StalledMessage { task: *task })
        }
        domain::MessageKind::SettingsDecision { proposal, outcome } => {
            wire::MessageKind::SettingsDecision(wire::SettingsDecisionMessage {
                proposal: *proposal,
                outcome: settings_outcome(*outcome, limits),
            })
        }
        domain::MessageKind::ProposalDecision { proposal, accepted } => {
            wire::MessageKind::ProposalDecision(wire::DecisionMessage { proposal: *proposal, accepted: *accepted })
        }
        domain::MessageKind::QuestionAnswered { question } => {
            wire::MessageKind::QuestionAnswered(wire::AnswerMessage { question: *question })
        }
        domain::MessageKind::QuestionOwed => wire::MessageKind::QuestionOwed,
        domain::MessageKind::NothingAwaited => wire::MessageKind::NothingAwaited,
        domain::MessageKind::QuestionEnded { question } => {
            wire::MessageKind::QuestionEnded(wire::AnswerMessage { question: *question })
        }
        domain::MessageKind::Words => wire::MessageKind::Words,
        domain::MessageKind::EffectNotMade { entry } => {
            wire::MessageKind::EffectNotMade(wire::EffectMessage { entry: *entry })
        }
        domain::MessageKind::Amendment { revision } => {
            wire::MessageKind::Amendment(wire::AmendmentMessage { revision: *revision })
        }
        domain::MessageKind::FinishRequested { revision } => {
            wire::MessageKind::FinishRequested(wire::AmendmentMessage { revision: *revision })
        }
        domain::MessageKind::Met { dependency, removed } => {
            wire::MessageKind::Met(wire::MetMessage { dependency: *dependency, removed: *removed })
        }
        domain::MessageKind::Question => wire::MessageKind::Question,
        domain::MessageKind::Answer { question } => {
            wire::MessageKind::Answer(wire::AnswerMessage { question: *question })
        }
        domain::MessageKind::Notice { subscription, target, state } => wire::MessageKind::Notice(wire::NoticeMessage {
            subscription: *subscription,
            target: *target,
            state: notice_state(*state, limits)?,
        }),
        domain::MessageKind::Timer { subscription } => {
            wire::MessageKind::Timer(wire::TimerMessage { subscription: *subscription })
        }
        domain::MessageKind::News { subscription, class } => {
            wire::MessageKind::News(wire::NewsMessage { subscription: *subscription, class: news_class(*class) })
        }
        domain::MessageKind::Result(kind) => {
            wire::MessageKind::Result(wire::ResultMessage { kind: result_kind(kind, limits)? })
        }
    })
}
/// Read `MessageKind` from its durable value.
pub fn read_message_kind(
    source: wire::MessageKind,
    limits: &wire::Limits,
) -> Result<domain::MessageKind, wire::Problem> {
    Ok(match source {
        wire::MessageKind::Escalation(source) => {
            let wire::EscalationMessage { task, revision } = source;
            domain::MessageKind::Escalation { task, revision }
        }
        wire::MessageKind::Proposal(source) => {
            let wire::ProposalMessage { proposer, proposal, kind } = source;
            domain::MessageKind::Proposal { proposer, proposal, kind: proposals::read_kind(kind)? }
        }
        wire::MessageKind::PersonStalled(source) => {
            let wire::StalledMessage { task } = source;
            domain::MessageKind::PersonStalled { task }
        }
        wire::MessageKind::SettingsDecision(source) => domain::MessageKind::SettingsDecision {
            proposal: source.proposal,
            outcome: read_settings_outcome(source.outcome),
        },
        wire::MessageKind::ProposalDecision(source) => {
            let wire::DecisionMessage { proposal, accepted } = source;
            domain::MessageKind::ProposalDecision { proposal, accepted }
        }
        wire::MessageKind::QuestionAnswered(source) => {
            domain::MessageKind::QuestionAnswered { question: source.question }
        }
        wire::MessageKind::QuestionOwed => domain::MessageKind::QuestionOwed,
        wire::MessageKind::NothingAwaited => domain::MessageKind::NothingAwaited,
        wire::MessageKind::QuestionEnded(source) => domain::MessageKind::QuestionEnded { question: source.question },
        wire::MessageKind::Words => domain::MessageKind::Words,
        wire::MessageKind::EffectNotMade(source) => {
            let wire::EffectMessage { entry } = source;
            domain::MessageKind::EffectNotMade { entry }
        }
        wire::MessageKind::Amendment(source) => {
            let wire::AmendmentMessage { revision } = source;
            domain::MessageKind::Amendment { revision }
        }
        wire::MessageKind::FinishRequested(source) => {
            let wire::AmendmentMessage { revision } = source;
            domain::MessageKind::FinishRequested { revision }
        }
        wire::MessageKind::Met(source) => {
            let wire::MetMessage { dependency, removed } = source;
            domain::MessageKind::Met { dependency, removed }
        }
        wire::MessageKind::Question => domain::MessageKind::Question,
        wire::MessageKind::Answer(source) => {
            let wire::AnswerMessage { question } = source;
            domain::MessageKind::Answer { question }
        }
        wire::MessageKind::Notice(source) => {
            let wire::NoticeMessage { subscription, target, state } = source;
            domain::MessageKind::Notice { subscription, target, state: read_notice_state(state) }
        }
        wire::MessageKind::Timer(source) => {
            let wire::TimerMessage { subscription } = source;
            domain::MessageKind::Timer { subscription }
        }
        wire::MessageKind::News(source) => {
            let wire::NewsMessage { subscription, class } = source;
            domain::MessageKind::News { subscription, class: read_news_class(class) }
        }
        wire::MessageKind::Result(source) => {
            let wire::ResultMessage { kind } = source;
            domain::MessageKind::Result(read_result_kind(kind, limits)?)
        }
    })
}

/// Translate `Word` to its durable record.
pub fn word(source: &domain::Word, limits: &wire::Limits) -> Result<wire::Word, wire::Problem> {
    let domain::Word { number, from, kind, words, at, hits, eligible } = source;
    Ok(wire::Word {
        number: *number,
        from: value::party(from, limits)?,
        kind: message_kind(kind, limits)?,
        words: words.clone(),
        at: at.as_nanos(),
        hits: *hits,
        eligible: *eligible,
    })
}
/// Read `Word` from its durable record.
pub fn read_word(source: wire::Word, limits: &wire::Limits) -> Result<domain::Word, wire::Problem> {
    let wire::Word { number, from, kind, words, at, hits, eligible } = source;
    Ok(domain::Word {
        number,
        from: value::read_party(from),
        kind: read_message_kind(kind, limits)?,
        words,
        at: Wall::from_nanos(at),
        hits,
        eligible,
    })
}

/// Translate `EndKind` to its durable value.
pub fn end_kind(source: &domain::EndKind, _limits: &wire::Limits) -> Result<wire::EndKind, wire::Problem> {
    Ok(match source {
        domain::EndKind::Parked => wire::EndKind::Parked,
        domain::EndKind::Finished => wire::EndKind::Finished,
        domain::EndKind::Cancelled => wire::EndKind::Cancelled,
        domain::EndKind::Failed { class, invalid_result } => wire::EndKind::Failed(wire::FailedEnd {
            class: value::class(*class),
            invalid_result: match invalid_result {
                Some(reason) => Some(state::invalid_result(*reason)),
                None => None,
            },
        }),
        domain::EndKind::Refused => wire::EndKind::Refused,
    })
}
/// Read `EndKind` from its durable value.
pub fn read_end_kind(source: wire::EndKind) -> Result<domain::EndKind, wire::Problem> {
    Ok(match source {
        wire::EndKind::Parked => domain::EndKind::Parked,
        wire::EndKind::Finished => domain::EndKind::Finished,
        wire::EndKind::Cancelled => domain::EndKind::Cancelled,
        wire::EndKind::Failed(source) => {
            let wire::FailedEnd { class, invalid_result } = source;
            domain::EndKind::Failed {
                class: value::read_class(class),
                invalid_result: match invalid_result {
                    Some(reason) => Some(state::read_invalid_result(reason)),
                    None => None,
                },
            }
        }
        wire::EndKind::Refused => domain::EndKind::Refused,
    })
}

fn session_end(value: domain::SessionEnd) -> wire::SessionEnd {
    match value {
        domain::SessionEnd::Quiet => wire::SessionEnd::Quiet,
        domain::SessionEnd::Bound => wire::SessionEnd::Bound,
        domain::SessionEnd::Unoffered => wire::SessionEnd::Unoffered,
        domain::SessionEnd::Fresh => wire::SessionEnd::Fresh,
        domain::SessionEnd::Closed => wire::SessionEnd::Closed,
        domain::SessionEnd::Talk => wire::SessionEnd::Talk,
        domain::SessionEnd::Left => wire::SessionEnd::Left,
        domain::SessionEnd::Rules => wire::SessionEnd::Rules,
    }
}

fn read_session_end(value: wire::SessionEnd) -> domain::SessionEnd {
    match value {
        wire::SessionEnd::Quiet => domain::SessionEnd::Quiet,
        wire::SessionEnd::Bound => domain::SessionEnd::Bound,
        wire::SessionEnd::Unoffered => domain::SessionEnd::Unoffered,
        wire::SessionEnd::Fresh => domain::SessionEnd::Fresh,
        wire::SessionEnd::Closed => domain::SessionEnd::Closed,
        wire::SessionEnd::Talk => domain::SessionEnd::Talk,
        wire::SessionEnd::Left => domain::SessionEnd::Left,
        wire::SessionEnd::Rules => domain::SessionEnd::Rules,
    }
}

/// Translate `Line` to its durable value.
pub fn line(source: &domain::Line, limits: &wire::Limits) -> Result<wire::Line, wire::Problem> {
    Ok(match source {
        domain::Line::Words { message, by, kind, words } => wire::Line::Words(wire::ActivityWords {
            message: *message,
            by: value::party(by, limits)?,
            kind: message_kind(kind, limits)?,
            words: words.clone(),
        }),
        domain::Line::Heard(message) => wire::Line::Heard(wire::HeardMessage { word: word(message, limits)? }),
        domain::Line::Notified { recipient, word: message } => wire::Line::Notified(wire::NotifiedMessage {
            recipient: value::party(recipient, limits)?,
            word: word(message, limits)?,
        }),
        domain::Line::RunBegan { attempt, why, resumed } => wire::Line::RunBegan(wire::BeganLine {
            attempt: *attempt,
            why: value::why(*why, limits)?,
            resumed: *resumed,
        }),
        domain::Line::RunEnded { attempt, end, spent } => {
            wire::Line::RunEnded(wire::EndedLine { attempt: *attempt, end: end_kind(end, limits)?, spent: *spent })
        }
        domain::Line::Turn { reply, attempt, turn, read } => {
            wire::Line::Turn(wire::TurnLine { reply: *reply, attempt: *attempt, turn: *turn, read: *read })
        }
        domain::Line::Decided { revision, proposal } => wire::Line::Decided(wire::DecidedLine {
            revision: *revision,
            proposal: match proposal {
                Some(value) => Some(wire::DecidedProposal { number: value.number, made: value.made }),
                None => None,
            },
        }),
        domain::Line::NowWritten { attempt, line } => {
            wire::Line::NowWritten(wire::NowWrittenLine { attempt: *attempt, line: line.clone() })
        }
        domain::Line::Stood { attempt, words } => {
            wire::Line::Stood(wire::StoodLine { attempt: *attempt, words: words.clone() })
        }
        domain::Line::TalkOpened { party } => wire::Line::TalkOpened(wire::TalkOpenedLine { party: *party }),
        domain::Line::TalkEnded { party, lapsed } => {
            wire::Line::TalkEnded(wire::TalkEndedLine { party: *party, lapsed: *lapsed })
        }
        domain::Line::SessionEnded { attempt, end } => {
            wire::Line::SessionEnded(wire::SessionEndedLine { attempt: *attempt, end: session_end(*end) })
        }
        domain::Line::Kept { attempt, pending, question } => wire::Line::Kept(wire::KeptLine {
            attempt: *attempt,
            question: *question,
            pending: super::phase::references(
                pending,
                limits.kept_line_pending,
                wire::paths::KEPT_LINE_PENDING,
                limits,
            )?,
        }),
        domain::Line::Milestone { position } => wire::Line::Milestone(wire::MilestoneLine { position: *position }),
    })
}
/// Read `Line` from its durable value.
pub fn read_line(source: wire::Line, limits: &wire::Limits) -> Result<domain::Line, wire::Problem> {
    Ok(match source {
        wire::Line::Words(source) => {
            let wire::ActivityWords { message, by, kind, words } = source;
            domain::Line::Words { message, by: value::read_party(by), kind: read_message_kind(kind, limits)?, words }
        }
        wire::Line::Heard(source) => {
            let wire::HeardMessage { word } = source;
            domain::Line::Heard(read_word(word, limits)?)
        }
        wire::Line::Notified(source) => {
            let wire::NotifiedMessage { recipient, word } = source;
            domain::Line::Notified { recipient: value::read_party(recipient), word: read_word(word, limits)? }
        }
        wire::Line::RunBegan(source) => {
            let wire::BeganLine { attempt, why, resumed } = source;
            domain::Line::RunBegan { attempt, why: value::read_why(why), resumed }
        }
        wire::Line::RunEnded(source) => {
            let wire::EndedLine { attempt, end, spent } = source;
            domain::Line::RunEnded { attempt, end: read_end_kind(end)?, spent }
        }
        wire::Line::Turn(source) => {
            let wire::TurnLine { reply, attempt, turn, read } = source;
            domain::Line::Turn { reply, attempt, turn, read }
        }
        wire::Line::Decided(source) => {
            let wire::DecidedLine { revision, proposal } = source;
            domain::Line::Decided {
                revision,
                proposal: match proposal {
                    Some(value) => Some(domain::DecidedProposal { number: value.number, made: value.made }),
                    None => None,
                },
            }
        }
        wire::Line::NowWritten(source) => {
            let wire::NowWrittenLine { attempt, line } = source;
            domain::Line::NowWritten { attempt, line }
        }
        wire::Line::Stood(source) => {
            let wire::StoodLine { attempt, words } = source;
            domain::Line::Stood { attempt, words }
        }
        wire::Line::TalkOpened(source) => domain::Line::TalkOpened { party: source.party },
        wire::Line::TalkEnded(source) => domain::Line::TalkEnded { party: source.party, lapsed: source.lapsed },
        wire::Line::SessionEnded(source) => {
            let wire::SessionEndedLine { attempt, end } = source;
            domain::Line::SessionEnded { attempt, end: read_session_end(end) }
        }
        wire::Line::Kept(source) => {
            let wire::KeptLine { attempt, pending, question } = source;
            domain::Line::Kept { attempt, pending: super::phase::read_references(pending), question }
        }
        wire::Line::Milestone(source) => {
            let wire::MilestoneLine { position } = source;
            domain::Line::Milestone { position }
        }
    })
}

/// Translate `ActivityLine` to its durable record.
pub fn activity_line(
    source: &domain::ActivityLine,
    limits: &wire::Limits,
) -> Result<wire::ActivityLine, wire::Problem> {
    let domain::ActivityLine { task, position, at, line: activity } = source;
    Ok(wire::ActivityLine { task: *task, position: *position, at: at.as_nanos(), line: line(activity, limits)? })
}
/// Read `ActivityLine` from its durable record.
pub fn read_activity_line(
    source: wire::ActivityLine,
    limits: &wire::Limits,
) -> Result<domain::ActivityLine, wire::Problem> {
    let wire::ActivityLine { task, position, at, line } = source;
    Ok(domain::ActivityLine { task, position, at: Wall::from_nanos(at), line: read_line(line, limits)? })
}

fn settings_outcome(value: domain::SettingsOutcome, _limits: &wire::Limits) -> wire::SettingsOutcome {
    match value {
        domain::SettingsOutcome::Accepted { revision } => {
            wire::SettingsOutcome::Accepted(wire::SettingsAcceptedRevision { revision })
        }
        domain::SettingsOutcome::Rejected => wire::SettingsOutcome::Rejected,
        domain::SettingsOutcome::Stale { newest } => {
            wire::SettingsOutcome::Stale(wire::SettingsStaleRevision { newest })
        }
    }
}
fn read_settings_outcome(value: wire::SettingsOutcome) -> domain::SettingsOutcome {
    match value {
        wire::SettingsOutcome::Accepted(value) => domain::SettingsOutcome::Accepted { revision: value.revision },
        wire::SettingsOutcome::Rejected => domain::SettingsOutcome::Rejected,
        wire::SettingsOutcome::Stale(value) => domain::SettingsOutcome::Stale { newest: value.newest },
    }
}
