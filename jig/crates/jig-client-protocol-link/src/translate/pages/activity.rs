//! Whole pages and single documents (protocol/client/wire.md, 1, and protocol/client/README.md, 4).
use super::{Limits, List, Reader, Wall, c, carried_turn_fields, cursor_fields, long_text, text, too_large, w};
use crate::generated::client;
use jig_client_protocol_wire::Record as _;

pub(crate) fn word_result_fields(value: &w::WordResult, limits: &Limits) -> Result<c::WordResult, w::Problem> {
    Ok(match value {
        w::WordResult::Approved(row) => {
            c::WordResult::Approved { state: text(&row.state, limits)?, words_present: row.words_present }
        }
        w::WordResult::Report => c::WordResult::Report,
        w::WordResult::Verdict(row) => {
            if row.fields.len() > limits.client().rows {
                return Err(too_large());
            }
            let mut list0 = List::with_capacity(row.fields.len());
            for item in &row.fields {
                list0.push(crate::translate::facts::result_field(item)).expect("reserved supplied list");
            }
            c::WordResult::Verdict { code: row.code, fields: list0 }
        }
        w::WordResult::Change(row) => {
            c::WordResult::Change { connector: row.connector, kind: row.kind, resource: row.resource }
        }
        w::WordResult::Failed => c::WordResult::Failed,
        w::WordResult::Cancelled => c::WordResult::Cancelled,
        w::WordResult::Job => c::WordResult::Job,
    })
}

#[cfg(test)]
pub(crate) fn message_kind_fields(value: &w::MessageKind, limits: &Limits) -> Result<c::MessageKind, w::Problem> {
    client::message_kind_to_domain(value.clone(), limits)
}

pub(crate) fn word_fields(value: w::Word, limits: &Limits) -> Result<c::Word, w::Problem> {
    Ok(c::Word {
        number: value.number,
        from: crate::translate::facts::requester(&value.from),
        kind: client::message_kind_to_domain(value.kind, limits)?,
        words: long_text(&value.words, limits)?,
        at: Wall::from_nanos(value.at),
        hits: value.hits,
        eligible: value.eligible,
    })
}

#[cfg(test)]
pub(crate) fn activity_fields(value: &w::Activity, limits: &Limits) -> Result<c::Activity, w::Problem> {
    activity_from_record(value.clone(), limits)
}

pub(crate) fn activity_from_record(value: w::Activity, limits: &Limits) -> Result<c::Activity, w::Problem> {
    Ok(match value {
        w::Activity::Words(row) => c::Activity::Words {
            message: row.message,
            by: crate::translate::facts::requester(&row.by),
            kind: client::message_kind_to_domain(row.kind, limits)?,
            words: long_text(&row.words, limits)?,
        },
        w::Activity::Heard(row) => c::Activity::Heard(word_fields(row.value, limits)?),
        w::Activity::Notified(row) => c::Activity::Notified {
            recipient: crate::translate::facts::requester(&row.recipient),
            word: word_fields(row.word, limits)?,
        },
        w::Activity::RunBegan(row) => c::Activity::RunBegan {
            attempt: row.attempt,
            why: client::run_why_to_domain(row.why),
            resumed: row.resumed,
            fresh: read_fresh(row.fresh.as_ref()),
        },
        w::Activity::RunEnded(row) => {
            c::Activity::RunEnded { attempt: row.attempt, end: client::run_end_to_domain(row.end), spent: row.spent }
        }
        w::Activity::Turn(row) => {
            let opt0 = match &row.read {
                Some(item) => Some(*item),
                None => None,
            };
            c::Activity::Turn { reply: row.reply, attempt: row.attempt, turn: row.turn, read: opt0 }
        }
        w::Activity::Decided(row) => c::Activity::Decided {
            revision: row.revision,
            proposal: match &row.proposal {
                Some(value) => Some(c::DecidedProposal { number: value.number, made: value.made }),
                None => None,
            },
        },
        w::Activity::NowWritten(row) => {
            c::Activity::NowWritten { attempt: row.attempt, line: long_text(&row.line, limits)? }
        }
        w::Activity::Stood(row) => c::Activity::Stood { attempt: row.attempt, words: long_text(&row.words, limits)? },
        w::Activity::TalkOpened(row) => c::Activity::TalkOpened { party: row.party },
        w::Activity::TalkEnded(row) => c::Activity::TalkEnded { party: row.party, lapsed: row.lapsed },
        w::Activity::SessionEnded(row) => {
            c::Activity::SessionEnded { attempt: row.attempt, end: client::session_end_to_domain(row.end) }
        }
        w::Activity::Kept(row) => {
            c::Activity::Kept { attempt: row.attempt, pending: row.pending.clone(), question: row.question }
        }
        w::Activity::Milestone(row) => c::Activity::Milestone { position: row.position },
    })
}

pub(crate) fn archived_turn_fields(value: &w::ArchivedTurn, limits: &Limits) -> c::ArchivedTurn {
    let opt0 = match &value.read {
        Some(item) => Some(*item),
        None => None,
    };
    c::ArchivedTurn {
        task: value.task,
        name: client::turn_name_to_domain(value.name.clone()),
        cumulative: value.cumulative,
        read: opt0,
        at: Wall::from_nanos(value.at),
        turn: match &value.turn {
            w::TurnBytes::Carried(row) => carried_turn_fields(&row.bytes, limits),
            w::TurnBytes::TooLarge(row) => {
                c::TurnRead::Undecodable { why: c::TurnProblem::TooLarge { bytes: row.bytes } }
            }
        },
    }
}

pub(crate) fn activity_page_fields(value: &w::ActivityPage, limits: &Limits) -> Result<c::ActivityPage, w::Problem> {
    let opt0 = match &value.before {
        Some(item) => Some(cursor_fields(item, limits)?),
        None => None,
    };
    let opt1 = match &value.after {
        Some(item) => Some(cursor_fields(item, limits)?),
        None => None,
    };
    Ok(c::ActivityPage {
        cut: value.cut,
        rows: activity_line_piece(&value.rows, limits, limits.client().rows)?,
        before: opt0,
        after: opt1,
    })
}

pub(crate) fn activity_line_piece(
    bytes: &[u8],
    limits: &Limits,
    bound: u32,
) -> Result<List<c::DecodedActivityLine>, w::Problem> {
    let mut reader = Reader::new(bytes);
    let mut rows = List::with_capacity(bound);
    let mut sum = 0_u64;
    for _ in 0..=bound {
        let Some(item) = crate::translate::next_piece(&mut reader, &mut sum, limits)? else { return Ok(rows) };
        let row = w::DecodedActivityLine::decode(limits.receiving(), &mut Reader::new(&item))?;
        let row = client::decoded_activity_line_to_domain(row, limits)?;
        if rows.push(row).is_err() {
            return Err(too_large());
        }
    }
    Err(too_large())
}

fn read_fresh(value: Option<&w::RunFresh>) -> Option<c::RunFresh> {
    Some(client::run_fresh_to_domain(value?.clone()))
}
