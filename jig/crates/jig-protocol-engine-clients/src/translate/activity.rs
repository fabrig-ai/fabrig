//! Page-carried immutable activity and opaque turns (protocol/client/wire.md, 1).
use super::{chips, objects::question_answer, w};
use crate::generated::client as mapped;
use jig_domain_core_vocabulary::reads;
use jig_domain_tasks as t;
use skein_lib::bytes;

pub fn line(value: &t::ActivityLine) -> Result<w::ActivityLine, w::Problem> {
    mapped::activity_line_to_record(value)
}

pub(crate) fn activity_to_record(value: &t::Line) -> Result<w::Activity, w::Problem> {
    Ok(match value {
        t::Line::Words { message, by, kind, words } => w::Activity::Words(w::ActivityWords {
            message: *message,
            by: super::owner::requester(*by),
            kind: question_answer::message_kind(kind)?,
            words: question_answer::crossing_words(kind, words),
        }),
        t::Line::Heard(value) => w::Activity::Heard(w::ActivityHeard { value: question_answer::word(value)? }),
        t::Line::Notified { recipient, word } => w::Activity::Notified(w::ActivityNotified {
            recipient: super::owner::requester(*recipient),
            word: question_answer::word(word)?,
        }),
        t::Line::RunBegan { attempt, why: value, resumed, fresh: cause } => {
            w::Activity::RunBegan(w::ActivityRunBegan {
                attempt: *attempt,
                why: run_why_to_record(*value),
                resumed: *resumed,
                fresh: match cause {
                    Some(cause) => Some(mapped::run_fresh_to_record(cause)),
                    None => None,
                },
            })
        }
        t::Line::RunEnded { attempt, end: value, spent } => w::Activity::RunEnded(w::ActivityRunEnded {
            attempt: *attempt,
            end: run_end_to_record(value),
            spent: *spent,
        }),
        t::Line::Turn { reply, attempt, turn, read } => {
            w::Activity::Turn(w::ActivityTurn { reply: *reply, attempt: *attempt, turn: *turn, read: *read })
        }
        t::Line::Decided { revision, proposal } => w::Activity::Decided(w::ActivityDecided {
            revision: *revision,
            proposal: match proposal {
                Some(value) => Some(mapped::activity_decided_proposal_to_record(value)),
                None => None,
            },
        }),
        t::Line::NowWritten { attempt, line } => {
            w::Activity::NowWritten(w::ActivityNowWritten { attempt: *attempt, line: line.clone() })
        }
        t::Line::Stood { attempt, words } => {
            w::Activity::Stood(w::ActivityStood { attempt: *attempt, words: words.clone() })
        }
        t::Line::TalkOpened { party } => w::Activity::TalkOpened(w::TalkOpened { party: *party }),
        t::Line::TalkEnded { party, lapsed } => w::Activity::TalkEnded(w::TalkEnded { party: *party, lapsed: *lapsed }),
        t::Line::SessionEnded { attempt, end } => w::Activity::SessionEnded(w::ActivitySessionEnded {
            attempt: *attempt,
            end: mapped::session_end_to_record(end),
        }),
        t::Line::Kept { attempt, pending, question } => {
            let mut rows =
                skein_lib::List::with_capacity(u32::try_from(pending.len()).expect("bounded pending dependencies"));
            for number in pending {
                rows.push(*number).expect("same pending dependencies");
            }
            w::Activity::Kept(w::ActivityKept { attempt: *attempt, pending: rows, question: *question })
        }
        t::Line::Milestone { position } => w::Activity::Milestone(w::ActivityMilestone { position: *position }),
    })
}
// A measurement is a release in the run history; only the procedure reads the distinct cause.
fn run_why_to_record(why: t::Why) -> w::RunWhy {
    match why {
        t::Why::Made => w::RunWhy::Made,
        t::Why::Ready => w::RunWhy::Ready,
        t::Why::Message { message } => w::RunWhy::Message(w::RunWhyMessage { message }),
        t::Why::Released | t::Why::Measured => w::RunWhy::Released,
        t::Why::Retry { tries } => w::RunWhy::Retry(w::RunWhyRetry { tries: mapped::failure_tries_to_record(&tries) }),
        t::Why::Narrowed => w::RunWhy::Narrowed,
        t::Why::Changed => w::RunWhy::Changed,
    }
}
#[cfg_attr(
    target_pointer_width = "64",
    expect(
        clippy::trivially_copy_pass_by_ref,
        reason = "client maps borrow values even below Clippy's 64-bit size limit"
    )
)]
pub(crate) fn run_end_to_record(value: &t::EndKind) -> w::RunEnd {
    match *value {
        t::EndKind::Parked => w::RunEnd::Parked,
        t::EndKind::Finished => w::RunEnd::Finished,
        t::EndKind::Cancelled => w::RunEnd::Cancelled,
        t::EndKind::Failed { class, invalid_result } => w::RunEnd::Failed(w::RunEndFailed {
            class: chips::class(class),
            invalid_reason: invalid_reason(class, invalid_result),
        }),
        t::EndKind::Refused => w::RunEnd::Refused,
        t::EndKind::Suspended { worker, why } => {
            w::RunEnd::Suspended(w::RunEndSuspended { worker, why: mapped::suspend_why_to_record(&why) })
        }
    }
}

fn invalid_reason(class: t::Class, reason: Option<t::InvalidResult>) -> Option<w::InvalidReason> {
    match class {
        t::Class::Invalid => Some(match reason {
            Some(t::InvalidResult::Form) => w::InvalidReason::Form,
            Some(t::InvalidResult::Verdict) => w::InvalidReason::Verdict,
            Some(t::InvalidResult::Words) => w::InvalidReason::Words,
            Some(t::InvalidResult::Change) => w::InvalidReason::Change,
            Some(t::InvalidResult::Followups) => w::InvalidReason::Followups,
            None => w::InvalidReason::UnreadableResult,
        }),
        t::Class::Transient
        | t::Class::Infrastructure
        | t::Class::Permanent
        | t::Class::Run
        | t::Class::Agent
        | t::Class::Lost => None,
    }
}

pub fn page_turn(
    task: u64,
    value: &t::ActivityLine,
    supplied: Option<&reads::PageTurn>,
    private: &crate::Private,
    limits: &w::Limits,
) -> Result<w::DecodedActivityLine, w::Problem> {
    let invalid = w::Problem { path: w::paths::DECODED_ACTIVITY_LINE_DECODED, reason: skein_codec::Reason::Tag };
    if value.task != task {
        return Err(invalid);
    }
    let decoded = match &value.line {
        t::Line::Turn { attempt, turn, .. } => {
            let supplied = supplied.ok_or(invalid)?;
            match supplied {
                reads::PageTurn::Carried(record) | reads::PageTurn::Withheld(record) => {
                    if (record.task, record.attempt, record.turn) != (task, *attempt, *turn) {
                        return Err(invalid);
                    }
                    let projection = match supplied {
                        reads::PageTurn::Withheld(_) => Some(private),
                        reads::PageTurn::Carried(_) => None,
                        reads::PageTurn::TooLarge { .. } | reads::PageTurn::Unread => unreachable!("carried turn"),
                    };
                    Some(archive(record, projection, limits)?)
                }
                reads::PageTurn::TooLarge { spent, read, at, bytes } => Some(w::ArchivedTurn {
                    task,
                    name: w::TurnName { attempt: *attempt, number: *turn },
                    cumulative: *spent,
                    read: *read,
                    at: at.as_nanos(),
                    turn: w::TurnBytes::TooLarge(w::TurnTooLarge { bytes: *bytes }),
                }),
                reads::PageTurn::Unread => None,
            }
        }
        t::Line::Words { .. }
        | t::Line::Heard(_)
        | t::Line::Notified { .. }
        | t::Line::RunBegan { .. }
        | t::Line::RunEnded { .. }
        | t::Line::Decided { .. }
        | t::Line::NowWritten { .. }
        | t::Line::Stood { .. }
        | t::Line::TalkOpened { .. }
        | t::Line::TalkEnded { .. }
        | t::Line::SessionEnded { .. }
        | t::Line::Kept { .. }
        | t::Line::Milestone { .. } => {
            if supplied.is_some() {
                return Err(invalid);
            }
            None
        }
    };
    Ok(w::DecodedActivityLine { source: line(value)?, decoded })
}

/// The same immutable archive serves a page and a single-turn read.
pub fn archive(
    record: &jig_domain_core_vocabulary::TurnRecord,
    private: Option<&crate::Private>,
    limits: &w::Limits,
) -> Result<w::ArchivedTurn, w::Problem> {
    let path = w::paths::TURN_CARRIED_BYTES;
    if record.transcript.len()
        > skein_lib::length::usize_of(limits.turn_carried_bytes.min(w::CEILINGS.turn_carried_bytes))
    {
        return Err(w::Problem { path, reason: skein_codec::Reason::Bound });
    }
    let bytes = match private {
        None => bytes::copy_of(&record.transcript),
        Some(private) => {
            let bound = limits.turn_carried_bytes.min(w::CEILINGS.turn_carried_bytes);
            let projection = jig_protocol_transcript::Limits::new(
                bound,
                jig_protocol_transcript::TurnLimits { messages: bound, blocks: bound, text: bound, outcome: bound },
            );
            match jig_protocol_transcript::withhold(&projection, &record.transcript, private) {
                Ok(bytes) => bytes,
                Err(jig_protocol_transcript::Undecodable::TooLarge) => {
                    return Err(w::Problem { path, reason: skein_codec::Reason::Bound });
                }
                Err(
                    jig_protocol_transcript::Undecodable::Version { .. }
                    | jig_protocol_transcript::Undecodable::Malformed
                    | jig_protocol_transcript::Undecodable::Unknown,
                ) => return Err(w::Problem { path, reason: skein_codec::Reason::Tag }),
            }
        }
    };
    Ok(w::ArchivedTurn {
        task: record.task,
        name: w::TurnName { attempt: record.attempt, number: record.turn },
        cumulative: record.spent,
        read: record.read,
        at: record.at.as_nanos(),
        turn: w::TurnBytes::Carried(w::TurnCarried { bytes }),
    })
}
