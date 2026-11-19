//! Concrete turns become durable transcript bytes (smith/docs/design/protocol/transcript.md,
//! sections 2–4 and 8; domain/session.md, section 3).
//!
//! This translation keeps no state and knows only configured endpoint names.
//! It never sees credential values, addresses or host policy. `encode_turn`
//! preserves provider blocks and typed tool outcomes within the codec bounds.

use alloc::boxed::Box;
use skein_lib::{List, Writer};
use smith_domain_session::{llm, record};
use smith_domain_tools as tools;
use smith_transcript::{self as wire, Record};

use crate::{Endpoints, Error};

/// Encode one domain turn under its configured endpoint and replay dialect.
pub fn encode_turn(
    source: &record::Turn,
    limits: &wire::v7::Limits,
    endpoints: &Endpoints,
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
    let usage = encode_usage(source.usage);
    let (kind, opening) = encode_kind(&source.kind, source.sequence)?;
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

fn encode_message(source: &llm::Message, dialect: u32, limits: &wire::v7::Limits) -> Result<wire::Message, Error> {
    let role = match source.role {
        llm::Role::User => wire::Role::User,
        llm::Role::Assistant => wire::Role::Assistant,
    };
    let mut blocks = List::with_capacity(count(source.content.len())?);
    for block in &source.content {
        if blocks.push(encode_block(block, dialect, limits)?).is_err() {
            return Err(Error::Capacity);
        }
    }
    Ok(wire::Message { role, blocks })
}

fn encode_block(source: &llm::Block, dialect: u32, limits: &wire::v7::Limits) -> Result<wire::Block, Error> {
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
        llm::Block::ToolResult { id, result } => {
            wire::Block::Result(wire::ToolResult { id: id.clone(), returned: encode_returned(result, limits)? })
        }
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

fn encode_returned(source: &llm::Returned, limits: &wire::v7::Limits) -> Result<wire::Returned, Error> {
    let returned = match source {
        llm::Returned::Owned { outcome } => {
            wire::Returned::Outcome(wire::OwnedOutcome { value: encode_outcome(outcome, limits)? })
        }
        llm::Returned::Invalid { problem } => {
            wire::Returned::Invalid(wire::Invalid { problem: encode_problem(problem) })
        }
        llm::Returned::NotRun { why } => wire::Returned::NotRun(wire::NotRun { why: encode_not_run(why)? }),
        llm::Returned::Text { text, error, replay } => {
            if replay.is_some() {
                return Err(Error::Transcript(record::Refusal::Unresolved));
            }
            wire::Returned::Said(wire::Said { text: text.clone(), error: *error })
        }
        llm::Returned::Withdrawn => wire::Returned::Withdrawn,
    };
    Ok(returned)
}

fn encode_not_run(source: &llm::NotRun) -> Result<wire::NotRunWhy, Error> {
    Ok(match source {
        llm::NotRun::Stopped => wire::NotRunWhy::Stopped,
        llm::NotRun::AfterFailedWrite { call } => {
            wire::NotRunWhy::AfterFailedWrite(wire::AfterFailedWrite { call: *call })
        }
        llm::NotRun::OutsideChoice { choice } => {
            let only = match choice {
                Some(names) => {
                    let mut allowed = List::with_capacity(count(names.len())?);
                    for name in names {
                        allowed.push(wire::CallName { name: name.clone() }).expect("room for allowed names");
                    }
                    Some(wire::AllowedCalls { names: allowed })
                }
                None => None,
            };
            wire::NotRunWhy::OutsideChoice(wire::OutsideChoice { only })
        }
    })
}

fn encode_problem(source: &llm::Problem) -> wire::CallProblem {
    match source {
        llm::Problem::UnknownTool => wire::CallProblem::UnknownTool,
        llm::Problem::Nesting => wire::CallProblem::Nesting,
        llm::Problem::NotAnObject => wire::CallProblem::NotAnObject,
        llm::Problem::Missing { field } => wire::CallProblem::Missing(wire::FieldProblem { field: field.clone() }),
        llm::Problem::WrongType { field } => wire::CallProblem::WrongType(wire::FieldProblem { field: field.clone() }),
        llm::Problem::BadValue { field } => wire::CallProblem::BadValue(wire::FieldProblem { field: field.clone() }),
        llm::Problem::JobsFull { jobs } => wire::CallProblem::JobsFull(wire::JobsFullProblem { jobs: *jobs }),
        llm::Problem::UnknownJob { job } => wire::CallProblem::UnknownJob(wire::UnknownJobProblem { job: *job }),
        llm::Problem::TooLarge => wire::CallProblem::TooLarge,
        llm::Problem::Oversize { bytes, bound } => {
            wire::CallProblem::Oversize(wire::OversizeProblem { bytes: *bytes, bound: *bound })
        }
        llm::Problem::CutOff { bytes } => wire::CallProblem::CutOff(wire::CutProblem { bytes: *bytes }),
    }
}

#[expect(clippy::too_many_lines, reason = "every typed tool outcome has an explicit durable representation")]
fn encode_outcome(source: &tools::Outcome, limits: &wire::v7::Limits) -> Result<wire::Outcome, Error> {
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
            cut: match cut {
                tools::Cut::Whole => wire::ReadCut::Whole,
                tools::Cut::Line => wire::ReadCut::Line,
            },
            window: match window {
                tools::BytesClamp::Within => wire::BytesClamp::Within,
                tools::BytesClamp::Clamped { asked, ceiling } => {
                    wire::BytesClamp::Clamped(wire::ByteClamp { asked: *asked, ceiling: *ceiling })
                }
            },
        }),
        tools::Outcome::Listed { entries, more } => {
            let mut output = List::with_capacity(count(entries.len())?);
            for entry in entries {
                let kind = match entry.kind {
                    tools::Kind::File => wire::Kind::File,
                    tools::Kind::Directory => wire::Kind::Directory,
                    tools::Kind::Link => wire::Kind::Link,
                    tools::Kind::Other => wire::Kind::Other,
                };
                if output.push(wire::Entry { name: Box::from(entry.name.as_ref()), kind }).is_err() {
                    return Err(Error::Capacity);
                }
            }
            wire::Outcome::Listed(wire::Listed { entries: output, more: *more })
        }
        tools::Outcome::Found { hits, more, timed_out } => {
            let mut bytes = 0_usize;
            for hit in hits {
                bytes = bytes
                    .checked_add(12)
                    .ok_or(Error::Capacity)?
                    .checked_add(hit.path.len())
                    .ok_or(Error::Capacity)?
                    .checked_add(hit.text.len())
                    .ok_or(Error::Capacity)?;
            }
            if bytes
                > usize::try_from(limits.found_hits.min(wire::CEILINGS.found_hits)).expect("piece bound fits usize")
            {
                return Err(Error::Capacity);
            }
            let mut output = Writer::new(bytes);
            for hit in hits {
                let value = wire::Hit { path: hit.path.clone(), line: hit.line, text: hit.text.clone() };
                match value.write(limits, &mut output) {
                    Ok(value) => value,
                    Err(problem) => return Err(Error::TranscriptBody(problem)),
                }
            }
            let output = output.finish();
            wire::Outcome::Found(wire::Found { hits: output, more: *more, timed_out: *timed_out })
        }
        tools::Outcome::Removed => wire::Outcome::Removed,
        tools::Outcome::NotEmpty => wire::Outcome::NotEmpty,
        tools::Outcome::Written { created } => wire::Outcome::Written(wire::Written { created: *created }),
        tools::Outcome::Edited { replaced } => wire::Outcome::Edited(wire::Edited { replaced: *replaced }),
        tools::Outcome::Exited { exit, head, tail, dropped, timeout, output } => {
            wire::Outcome::Exited(wire::CommandEnd {
                exit: encode_exit(*exit),
                head: head.clone(),
                last: tail.clone(),
                dropped: *dropped,
                timeout: encode_timeout(*timeout),
                output: encode_bytes(*output),
            })
        }
        tools::Outcome::Running { job, head, tail, dropped, timeout, output, after } => {
            wire::Outcome::Running(wire::CommandRunning {
                job: job.0,
                head: head.clone(),
                last: tail.clone(),
                dropped: *dropped,
                timeout: encode_timeout(*timeout),
                output: encode_bytes(*output),
                after: encode_timeout(*after),
            })
        }
        tools::Outcome::Job { job, head, tail, dropped, output, wait, end } => {
            let end = match end {
                tools::JobEnd::Running => wire::JobEnd::Running,
                tools::JobEnd::Ended(exit) => wire::JobEnd::Ended(wire::JobExit { exit: encode_exit(*exit) }),
                tools::JobEnd::Stopped => wire::JobEnd::Stopped,
                tools::JobEnd::Cancelled => wire::JobEnd::Cancelled,
            };
            wire::Outcome::Job(wire::JobRead {
                job: job.0,
                head: head.clone(),
                last: tail.clone(),
                dropped: *dropped,
                output: encode_bytes(*output),
                wait: encode_timeout(*wait),
                end,
            })
        }
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
        tools::Outcome::TooLarge { what, size, bound } => {
            wire::Outcome::TooLarge(wire::TooLarge { what: encode_size_part(*what), size: *size, bound: *bound })
        }
        tools::Outcome::NotRead => wire::Outcome::NotRead,
        tools::Outcome::Stale => wire::Outcome::Stale,
        tools::Outcome::EditFailed { edit, why } => {
            if *edit == 0 {
                return Err(Error::Transcript(record::Refusal::Malformed));
            }
            wire::Outcome::EditFailed(wire::EditFailed { edit: *edit, why: encode_mismatch(why)? })
        }
        tools::Outcome::Failed { fault } => {
            let fault = match fault {
                tools::Fault::Denied => wire::Fault::Denied,
                tools::Fault::NoSpace => wire::Fault::NoSpace,
                tools::Fault::Other => wire::Fault::Other,
            };
            wire::Outcome::Failed(wire::Failed { fault })
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

fn encode_mismatch(source: &tools::Mismatch) -> Result<wire::Mismatch, Error> {
    let mismatch = match source {
        tools::Mismatch::NoMatch => wire::Mismatch::NoMatch,
        tools::Mismatch::Unchanged => wire::Mismatch::Unchanged,
        tools::Mismatch::Ambiguous { count: found, lines } => {
            let mut output = List::with_capacity(count(lines.len())?);
            for line in lines {
                if output.push(wire::LineNumber { number: *line }).is_err() {
                    return Err(Error::Capacity);
                }
            }
            wire::Mismatch::Ambiguous(wire::Ambiguous { count: *found, lines: output })
        }
    };
    Ok(mismatch)
}

fn encode_size_part(what: tools::SizePart) -> wire::SizePart {
    match what {
        tools::SizePart::File => wire::SizePart::File,
        tools::SizePart::WriteContent => wire::SizePart::WriteContent,
        tools::SizePart::EditOld => wire::SizePart::EditOld,
        tools::SizePart::EditNew => wire::SizePart::EditNew,
        tools::SizePart::EditItems => wire::SizePart::EditItems,
        tools::SizePart::EditResult => wire::SizePart::EditResult,
        tools::SizePart::EditReplacements => wire::SizePart::EditReplacements,
        tools::SizePart::ListGlob => wire::SizePart::ListGlob,
    }
}

fn encode_exit(exit: tools::Exit) -> wire::Exit {
    match exit {
        tools::Exit::Code { code } => wire::Exit::Code(wire::ExitCode { code }),
        tools::Exit::Signal { signal } => wire::Exit::Signal(wire::ExitSignal { signal }),
        tools::Exit::TimedOut => wire::Exit::TimedOut,
    }
}

fn encode_timeout(timeout: tools::TimeoutClamp) -> wire::TimeoutClamp {
    match timeout {
        tools::TimeoutClamp::Within => wire::TimeoutClamp::Within,
        tools::TimeoutClamp::Clamped { asked, ceiling } => wire::TimeoutClamp::Clamped(wire::DurationClamp {
            asked_ns: asked.as_nanos(),
            ceiling_ns: ceiling.as_nanos(),
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
    let affinity = wire::Affinity { key: affinity.key, thread: affinity.thread };
    let backend = match backend {
        record::Backend::Direct => wire::Backend::Direct,
        record::Backend::Program { name } => wire::Backend::Program(wire::Program { name: name.clone() }),
    };
    wire::Opening { affinity, account, backend }
}

fn supplied_system(kind: &record::Kind) -> Option<Box<[u8]>> {
    match kind {
        record::Kind::First(opener) | record::Kind::Window(opener) => opener.system.clone(),
        record::Kind::Ordinary => None,
    }
}

fn encode_messages(source: &record::Turn, limits: &wire::v7::Limits) -> Result<List<wire::Message>, Error> {
    let mut messages = List::with_capacity(count(source.messages.len())?);
    for message in &source.messages {
        if messages.push(encode_message(message, source.dialect, limits)?).is_err() {
            return Err(Error::Capacity);
        }
    }
    Ok(messages)
}

fn encode_usage(usage: llm::Usage) -> wire::Usage {
    wire::Usage {
        input: usage.input_tokens,
        output: usage.output_tokens,
        cache_read: usage.cache_read_tokens,
        cache_write: usage.cache_write_tokens,
        reasoning: usage.reasoning_tokens,
    }
}

fn encode_kind(kind: &record::Kind, sequence: u32) -> Result<(wire::TurnKind, Option<wire::Opening>), Error> {
    Ok(match kind {
        record::Kind::First(opener) => {
            if sequence != 1 {
                return Err(Error::Transcript(record::Refusal::Malformed));
            }
            (wire::TurnKind::Ordinary, Some(encode_opening(opener.affinity, opener.account, &opener.backend)))
        }
        record::Kind::Ordinary => {
            if sequence == 1 {
                return Err(Error::Transcript(record::Refusal::Malformed));
            }
            (wire::TurnKind::Ordinary, None)
        }
        record::Kind::Window(opener) => {
            (wire::TurnKind::Window, Some(encode_opening(opener.affinity, opener.account, &opener.backend)))
        }
    })
}
