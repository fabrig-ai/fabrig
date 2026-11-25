//! Saved turn bytes become concrete session history (smith/docs/design/protocol/transcript.md,
//! sections 2–4 and 8; domain/session.md, section 3).
//!
//! This pure translation keeps no state. It knows configured endpoint names
//! and dialect identities, never provider addresses or credential values.
//! `decode_transcript` checks the codec and sequence before the domain admits
//! the history; the domain still checks call/result pairing and ownership.

use alloc::boxed::Box;
use skein_codec::{Reason, convert};
use skein_lib::{Duration, List, Reader, Writer};
use smith_domain_session::{llm, record};
use smith_domain_tools as tools;
use smith_transcript::{self as wire, Record};

use crate::Endpoints;
use crate::generated::transcript as mapped;

pub(super) use crate::turn::block_to_record;

/// The turn's replay dialect, piece limits and receiving hit count.
pub(super) struct MessageContext {
    pub dialect: u32,
    pub limits: wire::v8::Limits,
    pub search_hits: u32,
}

/// Decode the host's ordered saved turn bodies into the domain's concrete history.
pub fn decode_transcript(
    piece: &[u8],
    resume_turns: u32,
    search_hits: u32,
    limits: &wire::v8::Limits,
    endpoints: &Endpoints,
) -> Result<Option<record::Transcript>, record::Refusal> {
    if piece.is_empty() {
        return Ok(None);
    }
    let mut reader = Reader::new(piece);
    let mut count = 0_u32;
    for _ in 0..=piece.len() / 4 {
        if reader.is_empty() {
            break;
        }
        let length = reader.u32().ok_or(record::Refusal::Malformed)?;
        reader.bytes(length).ok_or(record::Refusal::Malformed)?;
        count = count.checked_add(1).ok_or(record::Refusal::TooLarge)?;
        if count > resume_turns {
            return Err(record::Refusal::TooLarge);
        }
    }
    let mut reader = Reader::new(piece);
    let mut turns = List::with_capacity(count);
    let mut first_endpoint: Option<Box<[u8]>> = None;
    let mut first_dialect: Option<u32> = None;
    let mut previous: Option<u32> = None;
    for _ in 0..count {
        let length = reader.u32().ok_or(record::Refusal::Malformed)?;
        let body = reader.bytes(length).ok_or(record::Refusal::Malformed)?;
        if body.get(..2) != Some(wire::wire::VERSION.to_be_bytes().as_slice()) {
            return Err(record::Refusal::Version);
        }
        let place = match previous {
            Some(place) => Some(place.checked_add(1).ok_or(record::Refusal::Malformed)?),
            None => None,
        };
        let source = saved_turn(body, place, limits)?;
        match &first_endpoint {
            Some(first) if first.as_ref() != source.endpoint.as_ref() => return Err(record::Refusal::Malformed),
            Some(_) => {}
            None => first_endpoint = Some(Box::from(source.endpoint.as_ref())),
        }
        let endpoint = saved_endpoint(endpoints, source.endpoint.as_ref());
        let dialect = source.dialect;
        let turn = translate_turn(source, endpoint, dialect, limits, search_hits, false)?;
        if turns.is_empty() {
            match turn.kind {
                record::Kind::First(_) | record::Kind::Window(_) => {}
                record::Kind::Ordinary => return Err(record::Refusal::Malformed),
            }
        } else {
            match turn.kind {
                record::Kind::First(_) | record::Kind::Window(_) => return Err(record::Refusal::Malformed),
                record::Kind::Ordinary => {}
            }
        }
        previous = Some(turn.sequence);
        match first_dialect {
            Some(first) if first != turn.dialect => return Err(record::Refusal::Malformed),
            Some(_) => {}
            None => first_dialect = Some(turn.dialect),
        }
        if turns.push(turn).is_err() {
            return Err(record::Refusal::TooLarge);
        }
    }
    Ok(Some(record::Transcript {
        version: record::VERSION,
        endpoint: turns.get(0).ok_or(record::Refusal::Malformed)?.endpoint,
        dialect: first_dialect.ok_or(record::Refusal::Malformed)?,
        turns: turns.into_boxed(),
    }))
}

/// Join opaque saved turns, each preceded by its big-endian u32 length.
#[must_use]
pub fn transcript_piece(bodies: &[Box<[u8]>]) -> Option<Box<[u8]>> {
    let mut bytes = 0_usize;
    for body in bodies {
        u32::try_from(body.len()).ok()?;
        bytes = bytes.checked_add(4)?.checked_add(body.len())?;
    }
    u32::try_from(bytes).ok()?;
    let mut writer = Writer::new(bytes);
    for body in bodies {
        writer.put(&u32::try_from(body.len()).ok()?.to_be_bytes()).ok()?;
        writer.put(body).ok()?;
    }
    Some(writer.finish())
}

/// Decode one live numbered turn without requiring its earlier saved prefix.
pub fn decode_turn(
    bytes: &[u8],
    number: u32,
    limits: &wire::v8::Limits,
    endpoints: &Endpoints,
) -> Result<record::Turn, record::Refusal> {
    if number == 0 {
        return Err(record::Refusal::Malformed);
    }
    let source = saved_turn(bytes, Some(number), limits)?;
    let endpoint = saved_endpoint(endpoints, source.endpoint.as_ref());
    let dialect = source.dialect;
    translate_turn(source, endpoint, dialect, limits, u32::MAX, false)
}

/// Decode a part's turn body without imposing a main-conversation sequence.
pub fn decode_part(
    bytes: &[u8],
    limits: &wire::v8::Limits,
    endpoints: &Endpoints,
) -> Result<record::Turn, record::Refusal> {
    let source = saved_turn(bytes, None, limits)?;
    let endpoint = saved_endpoint(endpoints, source.endpoint.as_ref());
    let dialect = source.dialect;
    translate_turn(source, endpoint, dialect, limits, u32::MAX, true)
}

fn saved_endpoint(endpoints: &Endpoints, name: &[u8]) -> Option<llm::Endpoint> {
    let endpoint = endpoints.resolve(name)?;
    Some(llm::Endpoint(endpoint.number))
}

fn saved_turn(bytes: &[u8], number: Option<u32>, limits: &wire::v8::Limits) -> Result<wire::Turn, record::Refusal> {
    if bytes.get(..2) != Some(wire::wire::VERSION.to_be_bytes().as_slice()) {
        return Err(record::Refusal::Version);
    }
    let Ok(source) = wire::Turn::decode(limits, &mut Reader::new(bytes)) else {
        return Err(record::Refusal::Malformed);
    };
    if source.place == 0 {
        return Err(record::Refusal::Malformed);
    }
    match number {
        Some(number) if source.place != number => return Err(record::Refusal::Malformed),
        Some(_) | None => {}
    }
    Ok(source)
}

fn translate_turn(
    mut source: wire::Turn,
    endpoint: Option<llm::Endpoint>,
    dialect: u32,
    limits: &wire::v8::Limits,
    search_hits: u32,
    part: bool,
) -> Result<record::Turn, record::Refusal> {
    let kind = decode_kind(&mut source, part)?;
    let context = MessageContext { dialect, limits: *limits, search_hits };
    let messages = decode_messages(source.messages, &context)?;
    Ok(record::Turn {
        version: record::VERSION,
        kind,
        endpoint,
        model: source.model,
        effort: source.effort,
        dialect,
        sequence: source.place,
        usage: mapped::usage_to_domain(source.usage),
        spent: source.spent,
        jobs: source.jobs,
        messages: messages.into_boxed(),
    })
}

pub(super) fn block_to_domain(source: wire::Block, context: &MessageContext) -> Result<llm::Block, wire::Problem> {
    let dialect = context.dialect;
    let value = match source {
        wire::Block::Text(text) => llm::Block::Text { text: text.text, replay: replay(text.replay, dialect)? },
        wire::Block::Refusal(text) => llm::Block::Refusal { text: text.text, replay: replay(text.replay, dialect)? },
        wire::Block::Call(call) => llm::Block::ToolCall(llm::ToolCall {
            id: call.id,
            name: call.name,
            input: call.input,
            call: llm::Decoded::Historical,
            replay: replay(call.replay, dialect)?,
        }),
        wire::Block::Result(result) => {
            llm::Block::ToolResult { id: result.id, result: returned(result.returned, context)? }
        }
        wire::Block::Opaque(opaque) => {
            if opaque.dialect != dialect {
                return Err(wire::paths::OPAQUE_DIALECT.problem(Reason::Malformed));
            }
            llm::Block::Opaque { bytes: opaque.bytes }
        }
    };
    Ok(value)
}

fn replay(source: Option<wire::Replay>, dialect: u32) -> Result<Option<llm::Replay>, wire::Problem> {
    match source {
        Some(value) => {
            if value.dialect != dialect {
                return Err(wire::paths::REPLAY_DIALECT.problem(Reason::Malformed));
            }
            Ok(Some(llm::Replay { bytes: value.bytes }))
        }
        None => Ok(None),
    }
}

fn returned(source: wire::Returned, context: &MessageContext) -> Result<llm::Returned, wire::Problem> {
    let value = match source {
        wire::Returned::Outcome(value) => {
            let outcome = match outcome(&value.value, &context.limits, context.search_hits) {
                Ok(outcome) => outcome,
                Err(refusal) => return Err(outcome_problem(refusal)),
            };
            llm::Returned::Owned { outcome }
        }
        wire::Returned::Said(value) => llm::Returned::Text { text: value.text, error: value.error, replay: None },
        wire::Returned::Invalid(value) => {
            llm::Returned::Invalid { problem: mapped::call_problem_to_domain(value.problem) }
        }
        wire::Returned::NotRun(value) => llm::Returned::NotRun { why: mapped::not_run_why_to_domain(value.why) },
        wire::Returned::Withdrawn => llm::Returned::Withdrawn,
    };
    Ok(value)
}

// Outcome admission distinguishes a receiving limit from malformed data. Keep
// the distinction through the generated block list to the transcript boundary.
fn outcome_problem(refusal: record::Refusal) -> wire::Problem {
    let reason = match refusal {
        record::Refusal::TooLarge => Reason::Bound,
        record::Refusal::Malformed | record::Refusal::Unresolved => Reason::Malformed,
        record::Refusal::Version => Reason::Version,
    };
    wire::paths::OWNED_OUTCOME_VALUE.problem(reason)
}

fn outcome(
    source: &wire::Outcome,
    limits: &wire::v8::Limits,
    search_hits: u32,
) -> Result<tools::Outcome, record::Refusal> {
    let value = match source {
        wire::Outcome::Read(read) => read_outcome(read)?,
        wire::Outcome::Listed(listed) => listed_outcome(listed)?,
        wire::Outcome::Found(found) => {
            let hits = found_piece(found.hits.as_ref(), limits, search_hits)?;
            tools::Outcome::Found { hits, more: found.more, timed_out: found.timed_out }
        }
        wire::Outcome::Removed => tools::Outcome::Removed,
        wire::Outcome::NotEmpty => tools::Outcome::NotEmpty,
        wire::Outcome::Written(written) => tools::Outcome::Written { created: written.created },
        wire::Outcome::Edited(edited) => tools::Outcome::Edited { replaced: edited.replaced },
        wire::Outcome::Exited(command) => tools::Outcome::Exited {
            exit: mapped::exit_to_domain(command.exit.clone()),
            head: Box::from(command.head.as_ref()),
            tail: Box::from(command.last.as_ref()),
            dropped: command.dropped,
            timeout: timeout_clamp(&command.timeout)?,
            output: byte_clamp(&command.output)?,
        },
        wire::Outcome::NotGranted => tools::Outcome::NotGranted,
        wire::Outcome::Outside(failure) => tools::Outcome::Outside { path: resolved(&failure.path)? },
        wire::Outcome::ReadOnly(failure) => tools::Outcome::ReadOnly { path: resolved(&failure.path)? },
        wire::Outcome::TooLong => tools::Outcome::TooLong,
        wire::Outcome::NotFound(failure) => tools::Outcome::NotFound { path: resolved(&failure.path)? },
        wire::Outcome::NotFile => tools::Outcome::NotFile,
        wire::Outcome::Linked => tools::Outcome::Linked,
        wire::Outcome::Protected => tools::Outcome::Protected,
        wire::Outcome::NotDirectory(failure) => tools::Outcome::NotDirectory { path: resolved(&failure.path)? },
        wire::Outcome::TooLarge(size) => tools::Outcome::TooLarge {
            what: mapped::size_part_to_domain(size.what.clone()),
            size: size.size,
            bound: size.bound,
        },
        wire::Outcome::NotRead => tools::Outcome::NotRead,
        wire::Outcome::Stale => tools::Outcome::Stale,
        wire::Outcome::EditFailed(failed) => {
            if failed.edit == 0 {
                return Err(record::Refusal::Malformed);
            }
            tools::Outcome::EditFailed { edit: failed.edit, why: mapped::mismatch_to_domain(failed.why.clone()) }
        }
        wire::Outcome::Failed(failed) => {
            tools::Outcome::Failed { fault: mapped::fault_to_domain(failed.fault.clone()) }
        }
        wire::Outcome::TimedOut => tools::Outcome::TimedOut,
        wire::Outcome::Cancelled => tools::Outcome::Cancelled,
        wire::Outcome::Busy => tools::Outcome::Busy,
        wire::Outcome::NulByte => tools::Outcome::NulByte,
        wire::Outcome::Running(command) => tools::Outcome::Running {
            job: job_number_to_domain(command.job).or(Err(record::Refusal::Malformed))?,
            head: Box::from(command.head.as_ref()),
            tail: Box::from(command.last.as_ref()),
            dropped: command.dropped,
            timeout: timeout_clamp(&command.timeout)?,
            output: byte_clamp(&command.output)?,
            after: timeout_clamp(&command.after)?,
        },
        wire::Outcome::Interrupted(command) => {
            let ran = match command.ran_ms {
                Some(ms) => Some(ran_ms_to_domain(ms)),
                None => None,
            };
            tools::Outcome::Interrupted {
                ran,
                head: command.head.clone(),
                last: command.last.clone(),
                dropped: command.dropped,
            }
        }
        wire::Outcome::Job(read) => tools::Outcome::Job {
            job: job_number_to_domain(read.job).or(Err(record::Refusal::Malformed))?,
            head: Box::from(read.head.as_ref()),
            tail: Box::from(read.last.as_ref()),
            dropped: read.dropped,
            output: byte_clamp(&read.output)?,
            wait: timeout_clamp(&read.wait)?,
            end: mapped::job_end_to_domain(read.end.clone()),
        },
    };
    Ok(value)
}

/// Restore a normalized descriptor, refusing a malformed directory name or path.
fn resolved(source: &wire::Resolved) -> Result<tools::Resolved, record::Refusal> {
    let value = match source {
        wire::Resolved::Working(working) => {
            normalized(working.path.as_ref())?;
            tools::Resolved::Working { path: Box::from(working.path.as_ref()) }
        }
        wire::Resolved::Other(other) => {
            normalized(other.path.as_ref())?;
            let directory = tools::Name::new(Box::from(other.directory.as_ref())).ok_or(record::Refusal::Malformed)?;
            tools::Resolved::Other { directory, path: Box::from(other.path.as_ref()) }
        }
        wire::Resolved::Outside(outside) => {
            normalized(outside.path.as_ref())?;
            tools::Resolved::Outside { path: Box::from(outside.path.as_ref()) }
        }
    };
    Ok(value)
}

fn normalized(path: &[u8]) -> Result<(), record::Refusal> {
    if path.is_empty() {
        return Ok(());
    }
    let mut start = 0_usize;
    for (index, byte) in path.iter().enumerate() {
        if *byte == 0 {
            return Err(record::Refusal::Malformed);
        }
        if *byte == b'/' {
            let name = path.get(start..index).expect("component within path");
            component(name)?;
            start = index.checked_add(1).expect("path index");
        }
    }
    component(path.get(start..).expect("remaining component"))
}

fn component(name: &[u8]) -> Result<(), record::Refusal> {
    if name.is_empty() || name == b"." || name == b".." {
        return Err(record::Refusal::Malformed);
    }
    Ok(())
}

fn byte_clamp(source: &wire::BytesClamp) -> Result<tools::BytesClamp, record::Refusal> {
    match source {
        wire::BytesClamp::Within => Ok(tools::BytesClamp::Within),
        wire::BytesClamp::Clamped(clamp) => {
            if clamp.ceiling == 0 || clamp.asked <= clamp.ceiling {
                return Err(record::Refusal::Malformed);
            }
            Ok(tools::BytesClamp::Clamped { asked: clamp.asked, ceiling: clamp.ceiling })
        }
    }
}

fn hit_length(bytes: &[u8]) -> Result<u32, record::Refusal> {
    let mut reader = Reader::new(bytes);
    let start = reader.remaining();
    let path = reader.u32().ok_or(record::Refusal::Malformed)?;
    reader.bytes(path).ok_or(record::Refusal::Malformed)?;
    reader.u32().ok_or(record::Refusal::Malformed)?;
    let text = reader.u32().ok_or(record::Refusal::Malformed)?;
    reader.bytes(text).ok_or(record::Refusal::Malformed)?;
    start.checked_sub(reader.remaining()).ok_or(record::Refusal::Malformed)
}

fn found_piece(
    piece: &[u8],
    limits: &wire::v8::Limits,
    search_hits: u32,
) -> Result<Box<[tools::Hit]>, record::Refusal> {
    let mut reader = Reader::new(piece);
    let mut count = 0_u32;
    for _ in 0..=piece.len() / 12 {
        if reader.is_empty() {
            break;
        }
        let remaining = reader.remaining();
        let offset = u32::try_from(piece.len())
            .or(Err(record::Refusal::TooLarge))?
            .checked_sub(remaining)
            .ok_or(record::Refusal::Malformed)?;
        let tail = piece.get(usize::try_from(offset).expect("piece offset")..).ok_or(record::Refusal::Malformed)?;
        let length = hit_length(tail)?;
        reader.bytes(length).ok_or(record::Refusal::Malformed)?;
        count = count.checked_add(1).ok_or(record::Refusal::TooLarge)?;
        if count > search_hits {
            return Err(record::Refusal::TooLarge);
        }
    }
    let mut hits = List::with_capacity(count);
    let mut offset = 0_u32;
    for _ in 0..count {
        let tail = piece.get(usize::try_from(offset).expect("piece offset")..).ok_or(record::Refusal::Malformed)?;
        let length = hit_length(tail)?;
        let body = tail.get(..usize::try_from(length).expect("hit length")).ok_or(record::Refusal::Malformed)?;
        let hit = wire::Hit::decode(limits, &mut Reader::new(body)).or(Err(record::Refusal::Malformed))?;
        hits.push(mapped::hit_to_domain(hit)).or(Err(record::Refusal::TooLarge))?;
        offset = offset.checked_add(length).ok_or(record::Refusal::Malformed)?;
    }
    Ok(hits.into_boxed())
}

fn read_outcome(read: &wire::Read) -> Result<tools::Outcome, record::Refusal> {
    if read.first == 0 {
        return Err(record::Refusal::Malformed);
    }
    let next = match &read.next {
        wire::Next::Line(line) => {
            if line.number == 0 {
                return Err(record::Refusal::Malformed);
            }
            tools::Next::Line(line.number)
        }
        wire::Next::End => tools::Next::End,
    };
    let window = byte_clamp(&read.window)?;
    Ok(tools::Outcome::Read {
        content: Box::from(read.content.as_ref()),
        first: read.first,
        lines: read.lines,
        total: read.total,
        next,
        cut: mapped::read_cut_to_domain(read.cut.clone()),
        window,
    })
}

fn timeout_clamp(source: &wire::TimeoutClamp) -> Result<tools::TimeoutClamp, record::Refusal> {
    match source {
        wire::TimeoutClamp::Within => Ok(tools::TimeoutClamp::Within),
        wire::TimeoutClamp::Clamped(clamp) => {
            if clamp.ceiling_ns == 0 || clamp.asked_ns <= clamp.ceiling_ns {
                return Err(record::Refusal::Malformed);
            }
            Ok(tools::TimeoutClamp::Clamped {
                asked: convert::duration_to_domain(clamp.asked_ns),
                ceiling: convert::duration_to_domain(clamp.ceiling_ns),
            })
        }
    }
}

pub(super) fn ran_ms_to_record(value: Duration) -> u64 {
    value.as_millis()
}

fn ran_ms_to_domain(value: u64) -> Duration {
    Duration::from_millis(value)
}

pub(super) fn job_number_to_record(value: tools::JobNumber) -> u32 {
    value.0
}

fn job_number_to_domain(number: u32) -> Result<tools::JobNumber, Reason> {
    if number == 0 {
        return Err(Reason::Malformed);
    }
    Ok(tools::JobNumber(number))
}

fn listed_outcome(listed: &wire::Listed) -> Result<tools::Outcome, record::Refusal> {
    let mut entries = List::with_capacity(listed.entries.len());
    for entry in &listed.entries {
        normalized(entry.name.as_ref())?;
        if entry.name.as_ref().is_empty() {
            return Err(record::Refusal::Malformed);
        }
        let name = Box::from(entry.name.as_ref());
        let kind = mapped::kind_to_domain(entry.kind.clone());
        if entries.push(tools::Entry { name, kind }).is_err() {
            return Err(record::Refusal::TooLarge);
        }
    }
    Ok(tools::Outcome::Listed { entries: entries.into_boxed(), more: listed.more })
}

#[cfg(test)]
mod edit_tests {
    use super::{outcome, record, tools, wire};

    #[test]
    fn an_edit_failure_index_counts_from_one_at_transcript_admission() {
        let zero = wire::Outcome::EditFailed(wire::EditFailed { edit: 0, why: wire::Mismatch::NoMatch });
        assert_eq!(outcome(&zero, &wire::CEILINGS, 256), Err(record::Refusal::Malformed));
        let two = wire::Outcome::EditFailed(wire::EditFailed { edit: 2, why: wire::Mismatch::NoMatch });
        assert_eq!(
            outcome(&two, &wire::CEILINGS, 256),
            Ok(tools::Outcome::EditFailed { edit: 2, why: tools::Mismatch::NoMatch })
        );
    }
}

#[cfg(test)]
mod found_tests {
    use super::found_piece;
    use smith_domain_session::record;
    use smith_transcript::CEILINGS;

    #[test]
    fn a_found_piece_whose_hits_pass_its_end_is_malformed() {
        assert_eq!(found_piece(&[0, 0, 0, 2, 1], &CEILINGS, 100), Err(record::Refusal::Malformed));
    }

    #[test]
    fn a_found_piece_past_the_tools_hit_count_is_too_large() {
        assert_eq!(found_piece(&[0; 12], &CEILINGS, 0), Err(record::Refusal::TooLarge));
    }
}

fn decode_kind(source: &mut wire::Turn, part: bool) -> Result<record::Kind, record::Refusal> {
    let opening = match source.opening.take() {
        Some(value) => {
            Some((mapped::affinity_to_domain(value.affinity), value.account, mapped::backend_to_domain(value.backend)))
        }
        None => None,
    };
    Ok(match &source.kind {
        wire::TurnKind::Ordinary => match opening {
            Some((affinity, account, backend)) if source.place == 1 => record::Kind::First(Box::new(record::Opener {
                system: source.system.take(),
                affinity,
                account,
                backend,
            })),
            None if (source.place > 1 || part) && source.system.is_none() => record::Kind::Ordinary,
            Some(_) | None => return Err(record::Refusal::Malformed),
        },
        wire::TurnKind::Window => match opening {
            Some((affinity, account, backend)) => record::Kind::Window(Box::new(record::Opener {
                system: source.system.take(),
                affinity,
                account,
                backend,
            })),
            None => return Err(record::Refusal::Malformed),
        },
    })
}

fn decode_messages(
    source: List<wire::Message>,
    context: &MessageContext,
) -> Result<List<llm::Message>, record::Refusal> {
    let mut messages = List::with_capacity(source.len());
    for message in source.into_boxed() {
        let value = match mapped::message_to_domain(message, context) {
            Ok(value) => value,
            Err(problem) => return Err(message_refusal(problem)),
        };
        assert!(messages.push(value).is_ok(), "one output per message");
    }
    Ok(messages)
}

fn message_refusal(problem: wire::Problem) -> record::Refusal {
    match problem.reason {
        Reason::Bound => record::Refusal::TooLarge,
        Reason::Version => record::Refusal::Version,
        Reason::Short | Reason::Tag | Reason::Bool | Reason::Utf8 | Reason::Trailing | Reason::Malformed => {
            record::Refusal::Malformed
        }
    }
}
