//! Concrete turns become durable transcript bytes (smith/docs/design/protocol/transcript.md,
//! sections 2–4 and 8; domain/session.md, section 3).
//!
//! This translation keeps no state and knows only configured endpoint names.
//! It never sees credential values, addresses or host policy. `encode_turn`
//! preserves provider blocks and typed tool outcomes within the codec bounds.

use alloc::boxed::Box;
use skein_codec::{Reason, convert};
use skein_lib::{List, Writer};
use smith_domain_session::{llm, record};
use smith_domain_tools as tools;
use smith_transcript::{self as wire, Record};

use crate::generated::transcript as mapped;
use crate::transcript::{MessageContext, job_number_to_record, ran_ms_to_record};
use crate::{Endpoints, Error};

/// Encode one domain turn under its configured endpoint and replay dialect.
pub fn encode_turn(
    source: &record::Turn,
    limits: &wire::v8::Limits,
    endpoints: &Endpoints,
) -> Result<Box<[u8]>, Error> {
    encode(source, limits, endpoints, false)
}

/// Encode a part body, whose first-turn result carries no conversation opening.
pub fn encode_part(
    source: &record::Turn,
    limits: &wire::v8::Limits,
    endpoints: &Endpoints,
) -> Result<Box<[u8]>, Error> {
    encode(source, limits, endpoints, true)
}

fn encode(
    source: &record::Turn,
    limits: &wire::v8::Limits,
    endpoints: &Endpoints,
    part: bool,
) -> Result<Box<[u8]>, Error> {
    if source.version != record::VERSION || source.sequence == 0 {
        return Err(Error::Transcript(record::Refusal::Version));
    }
    let number = source.endpoint.ok_or(Error::Transcript(record::Refusal::Malformed))?.0;
    let endpoint = match endpoints.name_of(number, source.dialect) {
        Some(name) => Box::from(name),
        None => return Err(Error::Transcript(record::Refusal::Malformed)),
    };
    let dialect = source.dialect;
    let messages = encode_messages(source, limits)?;
    let usage = mapped::usage_to_record(&source.usage);
    let (kind, opening) = encode_kind(&source.kind, source.sequence, part)?;
    let turn = wire::Turn {
        kind,
        system: supplied_system(&source.kind),
        opening,
        endpoint,
        model: source.model.clone(),
        effort: source.effort.clone(),
        dialect,
        place: source.sequence,
        usage,
        spent: source.spent,
        jobs: source.jobs,
        messages,
    };
    match turn.encode(limits) {
        Ok(bytes) => Ok(bytes),
        Err(problem) => Err(Error::TranscriptBody(problem)),
    }
}

pub(super) fn block_to_record(source: &llm::Block, context: &MessageContext) -> Result<wire::Block, wire::Problem> {
    let dialect = context.dialect;
    let block = match source {
        llm::Block::Text { text, replay } => {
            wire::Block::Text(wire::Text { text: text.clone(), replay: encode_replay(replay.as_ref(), dialect) })
        }
        llm::Block::Refusal { text, replay } => {
            wire::Block::Refusal(wire::Text { text: text.clone(), replay: encode_replay(replay.as_ref(), dialect) })
        }
        llm::Block::ToolCall(llm::ToolCall { id, name, input, call: _, replay }) => wire::Block::Call(wire::Call {
            id: id.clone(),
            name: name.clone(),
            input: input.clone(),
            replay: encode_replay(replay.as_ref(), dialect),
        }),
        llm::Block::ToolResult { id, result } => wire::Block::Result(wire::ToolResult {
            id: id.clone(),
            returned: encode_returned(result, &context.limits)?,
        }),
        llm::Block::Opaque { bytes } => wire::Block::Opaque(wire::Opaque { dialect, bytes: bytes.clone() }),
    };
    Ok(block)
}

fn encode_replay(source: Option<&llm::Replay>, dialect: u32) -> Option<wire::Replay> {
    match source {
        Some(replay) => Some(wire::Replay { dialect, bytes: replay.bytes.clone() }),
        None => None,
    }
}

fn encode_returned(source: &llm::Returned, limits: &wire::v8::Limits) -> Result<wire::Returned, wire::Problem> {
    let returned = match source {
        llm::Returned::Owned { outcome } => {
            wire::Returned::Outcome(wire::OwnedOutcome { value: encode_outcome(outcome, limits)? })
        }
        llm::Returned::Invalid { problem } => {
            wire::Returned::Invalid(wire::Invalid { problem: mapped::call_problem_to_record(problem) })
        }
        llm::Returned::NotRun { why } => {
            wire::Returned::NotRun(wire::NotRun { why: mapped::not_run_why_to_record(why)? })
        }
        llm::Returned::Text { text, error, replay } => {
            if replay.is_some() {
                return Err(wire::paths::RETURNED_TAG.problem(Reason::Malformed));
            }
            wire::Returned::Said(wire::Said { text: text.clone(), error: *error })
        }
        llm::Returned::Withdrawn => wire::Returned::Withdrawn,
    };
    Ok(returned)
}

#[expect(clippy::too_many_lines, reason = "every typed tool outcome has an explicit durable representation")]
fn encode_outcome(source: &tools::Outcome, limits: &wire::v8::Limits) -> Result<wire::Outcome, wire::Problem> {
    let outcome = match source {
        tools::Outcome::Read { content, first, lines, total, next, cut, window } => wire::Outcome::Read(wire::Read {
            content: content.clone(),
            first: *first,
            lines: *lines,
            total: *total,
            next: match next {
                tools::Next::Line(line) => wire::Next::Line(wire::LineNumber { number: *line }),
                tools::Next::End => wire::Next::End,
            },
            cut: mapped::read_cut_to_record(cut),
            window: encode_bytes(*window),
        }),
        tools::Outcome::Listed { entries, more } => {
            let count = count(entries.len()).or(Err(wire::paths::LISTED_ENTRIES.problem(Reason::Bound)))?;
            let mut output = List::with_capacity(count);
            for entry in entries {
                let item = wire::Entry { name: entry.name.clone(), kind: mapped::kind_to_record(&entry.kind) };
                assert!(output.push(item).is_ok(), "one output per entry");
            }
            wire::Outcome::Listed(wire::Listed { entries: output, more: *more })
        }
        tools::Outcome::Found { hits, more, timed_out } => {
            let bound = wire::paths::FOUND_HITS.problem(Reason::Bound);
            let mut bytes = 0_usize;
            for hit in hits {
                bytes = bytes
                    .checked_add(12)
                    .ok_or(bound)?
                    .checked_add(hit.path.len())
                    .ok_or(bound)?
                    .checked_add(hit.text.len())
                    .ok_or(bound)?;
            }
            if bytes > skein_lib::usize_of(limits.found_hits.min(wire::CEILINGS.found_hits)) {
                return Err(bound);
            }
            let mut output = Writer::new(bytes);
            for hit in hits {
                mapped::hit_to_record(hit).write(limits, &mut output)?;
            }
            wire::Outcome::Found(wire::Found { hits: output.finish(), more: *more, timed_out: *timed_out })
        }
        tools::Outcome::Removed => wire::Outcome::Removed,
        tools::Outcome::NotEmpty => wire::Outcome::NotEmpty,
        tools::Outcome::Written { created } => wire::Outcome::Written(wire::Written { created: *created }),
        tools::Outcome::Edited { replaced } => wire::Outcome::Edited(wire::Edited { replaced: *replaced }),
        tools::Outcome::Exited { exit, head, tail, dropped, timeout, output } => {
            wire::Outcome::Exited(wire::CommandEnd {
                exit: mapped::exit_to_record(exit),
                head: head.clone(),
                last: tail.clone(),
                dropped: *dropped,
                timeout: encode_timeout(*timeout),
                output: encode_bytes(*output),
            })
        }
        tools::Outcome::Running { job, head, tail, dropped, timeout, output, after } => {
            wire::Outcome::Running(wire::CommandRunning {
                job: job_number_to_record(*job),
                head: head.clone(),
                last: tail.clone(),
                dropped: *dropped,
                timeout: encode_timeout(*timeout),
                output: encode_bytes(*output),
                after: encode_timeout(*after),
            })
        }
        tools::Outcome::Interrupted { ran, head, last, dropped } => {
            let ran_ms = match ran {
                Some(ran) => Some(ran_ms_to_record(*ran)),
                None => None,
            };
            wire::Outcome::Interrupted(wire::Interrupted {
                ran_ms,
                head: head.clone(),
                last: last.clone(),
                dropped: *dropped,
            })
        }
        tools::Outcome::Job { job, head, tail, dropped, output, wait, end } => wire::Outcome::Job(wire::JobRead {
            job: job_number_to_record(*job),
            head: head.clone(),
            last: tail.clone(),
            dropped: *dropped,
            output: encode_bytes(*output),
            wait: encode_timeout(*wait),
            end: mapped::job_end_to_record(end),
        }),
        tools::Outcome::NotGranted => wire::Outcome::NotGranted,
        tools::Outcome::Outside { path } => {
            wire::Outcome::Outside(wire::ResolvedFailure { path: encode_resolved(path) })
        }
        tools::Outcome::ReadOnly { path } => {
            wire::Outcome::ReadOnly(wire::ResolvedFailure { path: encode_resolved(path) })
        }
        tools::Outcome::TooLong => wire::Outcome::TooLong,
        tools::Outcome::NotFound { path } => {
            wire::Outcome::NotFound(wire::ResolvedFailure { path: encode_resolved(path) })
        }
        tools::Outcome::NotFile => wire::Outcome::NotFile,
        tools::Outcome::Linked => wire::Outcome::Linked,
        tools::Outcome::Protected => wire::Outcome::Protected,
        tools::Outcome::NotDirectory { path } => {
            wire::Outcome::NotDirectory(wire::ResolvedFailure { path: encode_resolved(path) })
        }
        tools::Outcome::TooLarge { what, size, bound } => wire::Outcome::TooLarge(wire::TooLarge {
            what: mapped::size_part_to_record(what),
            size: *size,
            bound: *bound,
        }),
        tools::Outcome::NotRead => wire::Outcome::NotRead,
        tools::Outcome::Stale => wire::Outcome::Stale,
        tools::Outcome::EditFailed { edit, why } => {
            if *edit == 0 {
                return Err(wire::paths::EDIT_FAILED_EDIT.problem(Reason::Malformed));
            }
            wire::Outcome::EditFailed(wire::EditFailed { edit: *edit, why: mapped::mismatch_to_record(why)? })
        }
        tools::Outcome::Failed { fault } => {
            wire::Outcome::Failed(wire::Failed { fault: mapped::fault_to_record(fault) })
        }
        tools::Outcome::TimedOut => wire::Outcome::TimedOut,
        tools::Outcome::Cancelled => wire::Outcome::Cancelled,
        tools::Outcome::Busy => wire::Outcome::Busy,
        tools::Outcome::NulByte => wire::Outcome::NulByte,
    };
    Ok(outcome)
}

fn count(length: usize) -> Result<u32, Error> {
    match u32::try_from(length) {
        Ok(count) => Ok(count),
        Err(_) => Err(Error::Capacity),
    }
}

/// Preserve the typed normalized failure descriptor under transcript version 6.
fn encode_resolved(source: &tools::Resolved) -> wire::Resolved {
    match source {
        tools::Resolved::Working { path } => wire::Resolved::Working(wire::WorkingPath { path: path.clone() }),
        tools::Resolved::Other { directory, path } => {
            wire::Resolved::Other(wire::OtherPath { directory: Box::from(directory.as_bytes()), path: path.clone() })
        }
        tools::Resolved::Outside { path } => wire::Resolved::Outside(wire::OutsidePath { path: path.clone() }),
    }
}

fn encode_timeout(timeout: tools::TimeoutClamp) -> wire::TimeoutClamp {
    match timeout {
        tools::TimeoutClamp::Within => wire::TimeoutClamp::Within,
        tools::TimeoutClamp::Clamped { asked, ceiling } => wire::TimeoutClamp::Clamped(wire::DurationClamp {
            asked_ns: convert::duration_to_record(&asked),
            ceiling_ns: convert::duration_to_record(&ceiling),
        }),
    }
}

fn encode_bytes(output: tools::BytesClamp) -> wire::BytesClamp {
    match output {
        tools::BytesClamp::Within => wire::BytesClamp::Within,
        tools::BytesClamp::Clamped { asked, ceiling } => wire::BytesClamp::Clamped(wire::ByteClamp { asked, ceiling }),
    }
}

fn encode_opening(affinity: llm::Affinity, account: Option<u32>, backend: &record::Backend) -> wire::Opening {
    wire::Opening {
        affinity: mapped::affinity_to_record(&affinity),
        account,
        backend: mapped::backend_to_record(backend),
    }
}

fn supplied_system(kind: &record::Kind) -> Option<Box<[u8]>> {
    match kind {
        record::Kind::First(opener) | record::Kind::Window(opener) => opener.system.clone(),
        record::Kind::Ordinary => None,
    }
}

fn encode_messages(source: &record::Turn, limits: &wire::v8::Limits) -> Result<List<wire::Message>, Error> {
    let context = MessageContext { dialect: source.dialect, limits: *limits, search_hits: 0 };
    let mut messages = List::with_capacity(count(source.messages.len())?);
    for message in &source.messages {
        let value = match mapped::message_to_record(message, &context) {
            Ok(value) => value,
            Err(problem) => return Err(message_error(problem)),
        };
        assert!(messages.push(value).is_ok(), "one output per message");
    }
    Ok(messages)
}

// Whole-list and aggregate piece limits belong to turn translation;
// an individual hit's record write reports the codec problem itself.
fn message_error(problem: wire::Problem) -> Error {
    use wire::paths;
    if problem.path == paths::RETURNED_TAG {
        Error::Transcript(record::Refusal::Unresolved)
    } else if problem.path == paths::EDIT_FAILED_EDIT {
        Error::Transcript(record::Refusal::Malformed)
    } else if problem.reason == Reason::Bound
        && (problem.path == paths::MESSAGE_BLOCKS
            || problem.path == paths::ALLOWED_CALLS_NAMES
            || problem.path == paths::AMBIGUOUS_LINES
            || problem.path == paths::LISTED_ENTRIES
            || problem.path == paths::FOUND_HITS)
    {
        Error::Capacity
    } else {
        Error::TranscriptBody(problem)
    }
}

fn encode_kind(
    kind: &record::Kind,
    sequence: u32,
    part: bool,
) -> Result<(wire::TurnKind, Option<wire::Opening>), Error> {
    Ok(match kind {
        record::Kind::First(opener) => {
            if sequence != 1 {
                return Err(Error::Transcript(record::Refusal::Malformed));
            }
            (wire::TurnKind::Ordinary, Some(encode_opening(opener.affinity, opener.account, &opener.backend)))
        }
        record::Kind::Ordinary => {
            if sequence == 1 && !part {
                return Err(Error::Transcript(record::Refusal::Malformed));
            }
            (wire::TurnKind::Ordinary, None)
        }
        record::Kind::Window(opener) => {
            (wire::TurnKind::Window, Some(encode_opening(opener.affinity, opener.account, &opener.backend)))
        }
    })
}
