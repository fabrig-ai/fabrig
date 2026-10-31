//! Page-carried immutable activity and opaque turns (protocol/client.md, 3).
use super::{chips, objects::question_answer, w};
use jig_domain_core::reads;
use jig_domain_tasks as t;
use skein_lib::bytes;

pub fn line(value: &t::ActivityLine, limits: &w::Limits) -> Result<w::ActivityLine, w::Problem> {
    let line = match &value.line {
        t::Line::Words { .. } | t::Line::Heard(_) | t::Line::Notified { .. } => {
            return question_answer::answer(value, limits);
        }
        t::Line::RunBegan { attempt, why: value, resumed } => w::Activity::RunBegan(w::ActivityRunBegan::new(
            limits,
            w::ActivityRunBeganParts { attempt: *attempt, why: why(*value, limits)?, resumed: *resumed },
        )?),
        t::Line::RunEnded { attempt, end: value, spent } => w::Activity::RunEnded(w::ActivityRunEnded::new(
            limits,
            w::ActivityRunEndedParts { attempt: *attempt, end: end(*value, limits)?, spent: *spent },
        )?),
        t::Line::Turn { reply, attempt, turn, read } => w::Activity::Turn(w::ActivityTurn::new(
            limits,
            w::ActivityTurnParts { reply: *reply, attempt: *attempt, turn: *turn, read: *read },
        )?),
        t::Line::Decided { revision } => {
            w::Activity::Decided(w::ActivityDecided::new(limits, w::ActivityDecidedParts { revision: *revision })?)
        }
        t::Line::Kept { attempt, pending, question } => {
            let mut rows =
                skein_lib::List::with_capacity(u32::try_from(pending.len()).expect("bounded pending dependencies"));
            for number in pending {
                rows.push(*number).expect("same pending dependencies");
            }
            w::Activity::Kept(w::ActivityKept::new(
                limits,
                w::ActivityKeptParts { attempt: *attempt, pending: rows, question: *question },
            )?)
        }
        t::Line::Milestone { position } => w::Activity::Milestone(w::ActivityMilestone::new(
            limits,
            w::ActivityMilestoneParts { position: *position },
        )?),
    };
    w::ActivityLine::new(
        limits,
        w::ActivityLineParts { task: value.task, position: value.position, at: value.at.as_nanos(), line },
    )
}
fn why(value: t::Why, limits: &w::Limits) -> Result<w::RunWhy, w::Problem> {
    Ok(match value {
        t::Why::Made => w::RunWhy::Made,
        t::Why::Ready => w::RunWhy::Ready,
        t::Why::Message { message } => {
            w::RunWhy::Message(w::RunWhyMessage::new(limits, w::RunWhyMessageParts { message })?)
        }
        t::Why::Released => w::RunWhy::Released,
        t::Why::Retry { tries } => {
            w::RunWhy::Retry(w::RunWhyRetry::new(limits, w::RunWhyRetryParts { tries: chips::tries(tries, limits)? })?)
        }
        t::Why::Narrowed => w::RunWhy::Narrowed,
        t::Why::Changed => w::RunWhy::Changed,
    })
}
fn end(value: t::EndKind, limits: &w::Limits) -> Result<w::RunEnd, w::Problem> {
    Ok(match value {
        t::EndKind::Parked => w::RunEnd::Parked,
        t::EndKind::Finished => w::RunEnd::Finished,
        t::EndKind::Cancelled => w::RunEnd::Cancelled,
        t::EndKind::Failed { class } => {
            w::RunEnd::Failed(w::RunEndFailed::new(limits, w::RunEndFailedParts { class: chips::class(class)? })?)
        }
        t::EndKind::Refused => w::RunEnd::Refused,
    })
}

pub fn page_turn(
    task: u64,
    value: &t::ActivityLine,
    supplied: Option<&reads::PageTurn>,
    limits: &w::Limits,
) -> Result<w::DecodedActivityLine, w::Problem> {
    let invalid = w::Problem { path: w::Path::DecodedActivityLineDecoded, reason: skein_codec::Reason::Tag };
    if value.task != task {
        return Err(invalid);
    }
    let decoded = match &value.line {
        t::Line::Turn { attempt, turn, .. } => {
            let supplied = supplied.ok_or(invalid)?;
            match supplied {
                reads::PageTurn::Carried(record) => {
                    if (record.task, record.attempt, record.turn) != (task, *attempt, *turn) {
                        return Err(invalid);
                    }
                    Some(archive(record, limits)?)
                }
                reads::PageTurn::TooLarge { spent, read, at, bytes } => Some(w::ArchivedTurn::new(
                    limits,
                    w::ArchivedTurnParts {
                        task,
                        name: w::TurnName::new(limits, w::TurnNameParts { attempt: *attempt, number: *turn })?,
                        cumulative: *spent,
                        read: *read,
                        at: at.as_nanos(),
                        turn: w::TurnBytes::TooLarge(w::TurnTooLarge::new(
                            limits,
                            w::TurnTooLargeParts { bytes: *bytes },
                        )?),
                    },
                )?),
                reads::PageTurn::Unread => None,
            }
        }
        t::Line::Words { .. }
        | t::Line::Heard(_)
        | t::Line::Notified { .. }
        | t::Line::RunBegan { .. }
        | t::Line::RunEnded { .. }
        | t::Line::Decided { .. }
        | t::Line::Kept { .. }
        | t::Line::Milestone { .. } => {
            if supplied.is_some() {
                return Err(invalid);
            }
            None
        }
    };
    w::DecodedActivityLine::new(limits, w::DecodedActivityLineParts { source: line(value, limits)?, decoded })
}

/// The same immutable archive serves a page and a single-turn read.
pub fn archive(record: &jig_domain_core::TurnRecord, limits: &w::Limits) -> Result<w::ArchivedTurn, w::Problem> {
    let path = w::Path::TurnCarriedBytes;
    if record.transcript.len()
        > skein_lib::length::usize_of(limits.turn_carried_bytes.min(w::CEILINGS.turn_carried_bytes))
    {
        return Err(w::Problem { path, reason: skein_codec::Reason::Bound });
    }
    w::ArchivedTurn::new(
        limits,
        w::ArchivedTurnParts {
            task: record.task,
            name: w::TurnName::new(limits, w::TurnNameParts { attempt: record.attempt, number: record.turn })?,
            cumulative: record.spent,
            read: record.read,
            at: record.at.as_nanos(),
            turn: w::TurnBytes::Carried(w::TurnCarried::new(
                limits,
                w::TurnCarriedParts { bytes: bytes::copy_of(&record.transcript) },
            )?),
        },
    )
}
