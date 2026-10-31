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
use smith_transcript as wire;

use crate::{Endpoints, Error};

/// Encode one domain turn under its configured endpoint and replay dialect.
pub fn encode_turn(
    source: &record::Turn,
    limits: &wire::v6::Limits,
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
    let mut messages = List::with_capacity(count(source.messages.len())?);
    for message in &source.messages {
        if messages.push(encode_message(message, source.dialect, limits)?).is_err() {
            return Err(Error::Capacity);
        }
    }
    let usage = match wire::Usage::new(
        limits,
        wire::UsageParts {
            input: source.usage.input_tokens,
            output: source.usage.output_tokens,
            cache_read: source.usage.cache_read_tokens,
            cache_write: source.usage.cache_write_tokens,
            reasoning: source.usage.reasoning_tokens,
        },
    ) {
        Ok(value) => value,
        Err(problem) => return Err(Error::TranscriptBody(problem)),
    };
    let (kind, opening) = match &source.kind {
        record::Kind::First(opener) => {
            if source.sequence != 1 {
                return Err(Error::Transcript(record::Refusal::Malformed));
            }
            (wire::TurnKind::Ordinary, Some(encode_opening(opener.affinity, opener.account, &opener.backend, limits)?))
        }
        record::Kind::Ordinary => {
            if source.sequence == 1 {
                return Err(Error::Transcript(record::Refusal::Malformed));
            }
            (wire::TurnKind::Ordinary, None)
        }
        record::Kind::Window(opener) => {
            (wire::TurnKind::Window, Some(encode_opening(opener.affinity, opener.account, &opener.backend, limits)?))
        }
    };
    let turn = match wire::Turn::new(
        limits,
        wire::TurnParts {
            kind,
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
        },
    ) {
        Ok(value) => value,
        Err(problem) => return Err(Error::TranscriptBody(problem)),
    };
    let Ok(length) = usize::try_from(turn.measure()) else {
        return Err(Error::Capacity);
    };
    let mut writer = Writer::new(length);
    match turn.encode(&mut writer) {
        Ok(value) => value,
        Err(problem) => return Err(Error::Write(problem)),
    }
    Ok(writer.finish())
}

fn encode_message(source: &llm::Message, dialect: u32, limits: &wire::v6::Limits) -> Result<wire::Message, Error> {
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
    Ok(match wire::Message::new(limits, wire::MessageParts { role, blocks }) {
        Ok(value) => value,
        Err(problem) => return Err(Error::TranscriptBody(problem)),
    })
}

fn encode_block(source: &llm::Block, dialect: u32, limits: &wire::v6::Limits) -> Result<wire::Block, Error> {
    let block = match source {
        llm::Block::Text { text, replay } => wire::Block::Text(
            match wire::Text::new(
                limits,
                wire::TextParts { text: text.clone(), replay: encode_replay(replay.as_ref(), dialect, limits)? },
            ) {
                Ok(value) => value,
                Err(problem) => return Err(Error::TranscriptBody(problem)),
            },
        ),
        llm::Block::Refusal { text, replay } => wire::Block::Refusal(
            match wire::Text::new(
                limits,
                wire::TextParts { text: text.clone(), replay: encode_replay(replay.as_ref(), dialect, limits)? },
            ) {
                Ok(value) => value,
                Err(problem) => return Err(Error::TranscriptBody(problem)),
            },
        ),
        llm::Block::ToolCall { id, name, input, call: _, replay } => wire::Block::Call(
            match wire::Call::new(
                limits,
                wire::CallParts {
                    id: id.clone(),
                    name: name.clone(),
                    input: input.clone(),
                    replay: encode_replay(replay.as_ref(), dialect, limits)?,
                },
            ) {
                Ok(value) => value,
                Err(problem) => return Err(Error::TranscriptBody(problem)),
            },
        ),
        llm::Block::ToolResult { id, result } => wire::Block::Result(
            match wire::ToolResult::new(
                limits,
                wire::ToolResultParts { id: id.clone(), returned: encode_returned(result, limits)? },
            ) {
                Ok(value) => value,
                Err(problem) => return Err(Error::TranscriptBody(problem)),
            },
        ),
        llm::Block::Opaque { bytes } => {
            wire::Block::Opaque(match wire::Opaque::new(limits, wire::OpaqueParts { dialect, bytes: bytes.clone() }) {
                Ok(value) => value,
                Err(problem) => return Err(Error::TranscriptBody(problem)),
            })
        }
    };
    Ok(block)
}

fn encode_replay(
    source: Option<&llm::Replay>,
    dialect: u32,
    limits: &wire::v6::Limits,
) -> Result<Option<wire::Replay>, Error> {
    match source {
        Some(replay) => {
            Ok(Some(match wire::Replay::new(limits, wire::ReplayParts { dialect, bytes: replay.bytes.clone() }) {
                Ok(value) => value,
                Err(problem) => return Err(Error::TranscriptBody(problem)),
            }))
        }
        None => Ok(None),
    }
}

fn encode_returned(source: &llm::Returned, limits: &wire::v6::Limits) -> Result<wire::Returned, Error> {
    let returned = match source {
        llm::Returned::Owned { outcome } => wire::Returned::Outcome(
            match wire::OwnedOutcome::new(limits, wire::OwnedOutcomeParts { value: encode_outcome(outcome, limits)? }) {
                Ok(value) => value,
                Err(problem) => return Err(Error::TranscriptBody(problem)),
            },
        ),
        llm::Returned::Invalid { problem } => wire::Returned::Invalid(
            match wire::Invalid::new(limits, wire::InvalidParts { problem: encode_problem(problem, limits)? }) {
                Ok(value) => value,
                Err(problem) => return Err(Error::TranscriptBody(problem)),
            },
        ),
        llm::Returned::NotRun { why } => wire::Returned::NotRun(
            match wire::NotRun::new(limits, wire::NotRunParts { why: encode_not_run(why, limits)? }) {
                Ok(value) => value,
                Err(problem) => return Err(Error::TranscriptBody(problem)),
            },
        ),
        llm::Returned::Text { text, error, replay } => {
            if replay.is_some() {
                return Err(Error::Transcript(record::Refusal::Unresolved));
            }
            wire::Returned::Said(match wire::Said::new(limits, wire::SaidParts { text: text.clone(), error: *error }) {
                Ok(value) => value,
                Err(problem) => return Err(Error::TranscriptBody(problem)),
            })
        }
        llm::Returned::Withdrawn => wire::Returned::Withdrawn,
    };
    Ok(returned)
}

fn encode_not_run(source: &llm::NotRun, limits: &wire::v6::Limits) -> Result<wire::NotRunWhy, Error> {
    Ok(match source {
        llm::NotRun::Stopped => wire::NotRunWhy::Stopped,
        llm::NotRun::AfterFailedWrite { call } => wire::NotRunWhy::AfterFailedWrite(match wire::AfterFailedWrite::new(
            limits,
            wire::AfterFailedWriteParts { call: *call },
        ) {
            Ok(value) => value,
            Err(problem) => return Err(Error::TranscriptBody(problem)),
        }),
        llm::NotRun::OutsideChoice { choice } => {
            let only = match choice {
                Some(names) => {
                    let mut allowed = List::with_capacity(count(names.len())?);
                    for name in names {
                        allowed
                            .push(match wire::CallName::new(limits, wire::CallNameParts { name: name.clone() }) {
                                Ok(value) => value,
                                Err(problem) => return Err(Error::TranscriptBody(problem)),
                            })
                            .expect("room for allowed names");
                    }
                    Some(match wire::AllowedCalls::new(limits, wire::AllowedCallsParts { names: allowed }) {
                        Ok(value) => value,
                        Err(problem) => return Err(Error::TranscriptBody(problem)),
                    })
                }
                None => None,
            };
            wire::NotRunWhy::OutsideChoice(match wire::OutsideChoice::new(limits, wire::OutsideChoiceParts { only }) {
                Ok(value) => value,
                Err(problem) => return Err(Error::TranscriptBody(problem)),
            })
        }
    })
}

fn encode_problem(source: &llm::Problem, limits: &wire::v6::Limits) -> Result<wire::CallProblem, Error> {
    let problem = match source {
        llm::Problem::UnknownTool => wire::CallProblem::UnknownTool,
        llm::Problem::Nesting => wire::CallProblem::Nesting,
        llm::Problem::NotAnObject => wire::CallProblem::NotAnObject,
        llm::Problem::Missing { field } => wire::CallProblem::Missing(
            match wire::FieldProblem::new(limits, wire::FieldProblemParts { field: field.clone() }) {
                Ok(value) => value,
                Err(problem) => return Err(Error::TranscriptBody(problem)),
            },
        ),
        llm::Problem::WrongType { field } => wire::CallProblem::WrongType(
            match wire::FieldProblem::new(limits, wire::FieldProblemParts { field: field.clone() }) {
                Ok(value) => value,
                Err(problem) => return Err(Error::TranscriptBody(problem)),
            },
        ),
        llm::Problem::BadValue { field } => wire::CallProblem::BadValue(
            match wire::FieldProblem::new(limits, wire::FieldProblemParts { field: field.clone() }) {
                Ok(value) => value,
                Err(problem) => return Err(Error::TranscriptBody(problem)),
            },
        ),
        llm::Problem::JobsFull { jobs } => wire::CallProblem::JobsFull(
            match wire::JobsFullProblem::new(limits, wire::JobsFullProblemParts { jobs: *jobs }) {
                Ok(value) => value,
                Err(problem) => return Err(Error::TranscriptBody(problem)),
            },
        ),
        llm::Problem::UnknownJob { job } => wire::CallProblem::UnknownJob(
            match wire::UnknownJobProblem::new(limits, wire::UnknownJobProblemParts { job: *job }) {
                Ok(value) => value,
                Err(problem) => return Err(Error::TranscriptBody(problem)),
            },
        ),
        llm::Problem::TooLarge => wire::CallProblem::TooLarge,
        llm::Problem::Oversize { bytes, bound } => wire::CallProblem::Oversize(
            match wire::OversizeProblem::new(limits, wire::OversizeProblemParts { bytes: *bytes, bound: *bound }) {
                Ok(value) => value,
                Err(problem) => return Err(Error::TranscriptBody(problem)),
            },
        ),
        llm::Problem::CutOff { bytes } => {
            wire::CallProblem::CutOff(match wire::CutProblem::new(limits, wire::CutProblemParts { bytes: *bytes }) {
                Ok(value) => value,
                Err(problem) => return Err(Error::TranscriptBody(problem)),
            })
        }
    };
    Ok(problem)
}

#[expect(clippy::too_many_lines, reason = "every typed tool outcome has an explicit durable representation")]
fn encode_outcome(source: &tools::Outcome, limits: &wire::v6::Limits) -> Result<wire::Outcome, Error> {
    let outcome = match source {
        tools::Outcome::Read { content, first, lines, total, next, cut, window } => wire::Outcome::Read(
            match wire::Read::new(
                limits,
                wire::ReadParts {
                    content: content.clone(),
                    first: *first,
                    lines: *lines,
                    total: *total,
                    next: match next {
                        tools::Next::Line(line) => wire::Next::Line(
                            match wire::LineNumber::new(limits, wire::LineNumberParts { number: *line }) {
                                Ok(value) => value,
                                Err(problem) => return Err(Error::TranscriptBody(problem)),
                            },
                        ),
                        tools::Next::End => wire::Next::End,
                    },
                    cut: match cut {
                        tools::Cut::Whole => wire::ReadCut::Whole,
                        tools::Cut::Line => wire::ReadCut::Line,
                    },
                    window: match window {
                        tools::BytesClamp::Within => wire::BytesClamp::Within,
                        tools::BytesClamp::Clamped { asked, ceiling } => wire::BytesClamp::Clamped(
                            match wire::ByteClamp::new(
                                limits,
                                wire::ByteClampParts { asked: *asked, ceiling: *ceiling },
                            ) {
                                Ok(value) => value,
                                Err(problem) => return Err(Error::TranscriptBody(problem)),
                            },
                        ),
                    },
                },
            ) {
                Ok(value) => value,
                Err(problem) => return Err(Error::TranscriptBody(problem)),
            },
        ),
        tools::Outcome::Listed { entries, more } => {
            let mut output = List::with_capacity(count(entries.len())?);
            for entry in entries {
                let kind = match entry.kind {
                    tools::Kind::File => wire::Kind::File,
                    tools::Kind::Directory => wire::Kind::Directory,
                    tools::Kind::Link => wire::Kind::Link,
                    tools::Kind::Other => wire::Kind::Other,
                };
                if output
                    .push(
                        match wire::Entry::new(limits, wire::EntryParts { name: Box::from(entry.name.as_ref()), kind })
                        {
                            Ok(value) => value,
                            Err(problem) => return Err(Error::TranscriptBody(problem)),
                        },
                    )
                    .is_err()
                {
                    return Err(Error::Capacity);
                }
            }
            wire::Outcome::Listed(match wire::Listed::new(limits, wire::ListedParts { entries: output, more: *more }) {
                Ok(value) => value,
                Err(problem) => return Err(Error::TranscriptBody(problem)),
            })
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
                let value = match wire::Hit::new(
                    limits,
                    wire::HitParts { path: hit.path.clone(), line: hit.line, text: hit.text.clone() },
                ) {
                    Ok(value) => value,
                    Err(problem) => return Err(Error::TranscriptBody(problem)),
                };
                match value.encode(&mut output) {
                    Ok(value) => value,
                    Err(problem) => return Err(Error::Write(problem)),
                }
            }
            let output = output.finish();
            wire::Outcome::Found(
                match wire::Found::new(limits, wire::FoundParts { hits: output, more: *more, timed_out: *timed_out }) {
                    Ok(value) => value,
                    Err(problem) => return Err(Error::TranscriptBody(problem)),
                },
            )
        }
        tools::Outcome::Removed => wire::Outcome::Removed,
        tools::Outcome::NotEmpty => wire::Outcome::NotEmpty,
        tools::Outcome::Written { created } => {
            wire::Outcome::Written(match wire::Written::new(limits, wire::WrittenParts { created: *created }) {
                Ok(value) => value,
                Err(problem) => return Err(Error::TranscriptBody(problem)),
            })
        }
        tools::Outcome::Edited { replaced } => {
            wire::Outcome::Edited(match wire::Edited::new(limits, wire::EditedParts { replaced: *replaced }) {
                Ok(value) => value,
                Err(problem) => return Err(Error::TranscriptBody(problem)),
            })
        }
        tools::Outcome::Exited { exit, head, tail, dropped, timeout, output } => wire::Outcome::Exited(
            match wire::CommandEnd::new(
                limits,
                wire::CommandEndParts {
                    exit: encode_exit(*exit, limits)?,
                    head: head.clone(),
                    last: tail.clone(),
                    dropped: *dropped,
                    timeout: encode_timeout(*timeout, limits)?,
                    output: encode_bytes(*output, limits)?,
                },
            ) {
                Ok(value) => value,
                Err(problem) => return Err(Error::TranscriptBody(problem)),
            },
        ),
        tools::Outcome::Running { job, head, tail, dropped, timeout, output, after } => wire::Outcome::Running(
            match wire::CommandRunning::new(
                limits,
                wire::CommandRunningParts {
                    job: job.0,
                    head: head.clone(),
                    last: tail.clone(),
                    dropped: *dropped,
                    timeout: encode_timeout(*timeout, limits)?,
                    output: encode_bytes(*output, limits)?,
                    after: encode_timeout(*after, limits)?,
                },
            ) {
                Ok(value) => value,
                Err(problem) => return Err(Error::TranscriptBody(problem)),
            },
        ),
        tools::Outcome::Job { job, head, tail, dropped, output, wait, end } => {
            let end = match end {
                tools::JobEnd::Running => wire::JobEnd::Running,
                tools::JobEnd::Ended(exit) => wire::JobEnd::Ended(
                    match wire::JobExit::new(limits, wire::JobExitParts { exit: encode_exit(*exit, limits)? }) {
                        Ok(value) => value,
                        Err(problem) => return Err(Error::TranscriptBody(problem)),
                    },
                ),
                tools::JobEnd::Stopped => wire::JobEnd::Stopped,
                tools::JobEnd::Cancelled => wire::JobEnd::Cancelled,
            };
            wire::Outcome::Job(
                match wire::JobRead::new(
                    limits,
                    wire::JobReadParts {
                        job: job.0,
                        head: head.clone(),
                        last: tail.clone(),
                        dropped: *dropped,
                        output: encode_bytes(*output, limits)?,
                        wait: encode_timeout(*wait, limits)?,
                        end,
                    },
                ) {
                    Ok(value) => value,
                    Err(problem) => return Err(Error::TranscriptBody(problem)),
                },
            )
        }
        tools::Outcome::NotGranted => wire::Outcome::NotGranted,
        tools::Outcome::Outside { path } => wire::Outcome::Outside(
            match wire::ResolvedFailure::new(
                limits,
                wire::ResolvedFailureParts { path: encode_resolved(path, limits)? },
            ) {
                Ok(value) => value,
                Err(problem) => return Err(Error::TranscriptBody(problem)),
            },
        ),
        tools::Outcome::ReadOnly { path } => wire::Outcome::ReadOnly(
            match wire::ResolvedFailure::new(
                limits,
                wire::ResolvedFailureParts { path: encode_resolved(path, limits)? },
            ) {
                Ok(value) => value,
                Err(problem) => return Err(Error::TranscriptBody(problem)),
            },
        ),
        tools::Outcome::TooLong => wire::Outcome::TooLong,
        tools::Outcome::NotFound { path } => wire::Outcome::NotFound(
            match wire::ResolvedFailure::new(
                limits,
                wire::ResolvedFailureParts { path: encode_resolved(path, limits)? },
            ) {
                Ok(value) => value,
                Err(problem) => return Err(Error::TranscriptBody(problem)),
            },
        ),
        tools::Outcome::NotFile => wire::Outcome::NotFile,
        tools::Outcome::Linked => wire::Outcome::Linked,
        tools::Outcome::Protected => wire::Outcome::Protected,
        tools::Outcome::NotDirectory { path } => wire::Outcome::NotDirectory(
            match wire::ResolvedFailure::new(
                limits,
                wire::ResolvedFailureParts { path: encode_resolved(path, limits)? },
            ) {
                Ok(value) => value,
                Err(problem) => return Err(Error::TranscriptBody(problem)),
            },
        ),
        tools::Outcome::TooLarge { what, size, bound } => wire::Outcome::TooLarge(
            match wire::TooLarge::new(
                limits,
                wire::TooLargeParts { what: encode_size_part(*what), size: *size, bound: *bound },
            ) {
                Ok(value) => value,
                Err(problem) => return Err(Error::TranscriptBody(problem)),
            },
        ),
        tools::Outcome::NotRead => wire::Outcome::NotRead,
        tools::Outcome::Stale => wire::Outcome::Stale,
        tools::Outcome::EditFailed { edit, why } => {
            if *edit == 0 {
                return Err(Error::Transcript(record::Refusal::Malformed));
            }
            wire::Outcome::EditFailed(
                match wire::EditFailed::new(
                    limits,
                    wire::EditFailedParts { edit: *edit, why: encode_mismatch(why, limits)? },
                ) {
                    Ok(value) => value,
                    Err(problem) => return Err(Error::TranscriptBody(problem)),
                },
            )
        }
        tools::Outcome::Failed { fault } => {
            let fault = match fault {
                tools::Fault::Denied => wire::Fault::Denied,
                tools::Fault::NoSpace => wire::Fault::NoSpace,
                tools::Fault::Other => wire::Fault::Other,
            };
            wire::Outcome::Failed(match wire::Failed::new(limits, wire::FailedParts { fault }) {
                Ok(value) => value,
                Err(problem) => return Err(Error::TranscriptBody(problem)),
            })
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
fn encode_resolved(source: &tools::Resolved, limits: &wire::v6::Limits) -> Result<wire::Resolved, Error> {
    let value = match source {
        tools::Resolved::Working { path } => wire::Resolved::Working(
            match wire::WorkingPath::new(limits, wire::WorkingPathParts { path: path.clone() }) {
                Ok(value) => value,
                Err(problem) => return Err(Error::TranscriptBody(problem)),
            },
        ),
        tools::Resolved::Other { directory, path } => wire::Resolved::Other(
            match wire::OtherPath::new(
                limits,
                wire::OtherPathParts { directory: Box::from(directory.as_bytes()), path: path.clone() },
            ) {
                Ok(value) => value,
                Err(problem) => return Err(Error::TranscriptBody(problem)),
            },
        ),
        tools::Resolved::Outside { path } => wire::Resolved::Outside(
            match wire::OutsidePath::new(limits, wire::OutsidePathParts { path: path.clone() }) {
                Ok(value) => value,
                Err(problem) => return Err(Error::TranscriptBody(problem)),
            },
        ),
    };
    Ok(value)
}

fn encode_mismatch(source: &tools::Mismatch, limits: &wire::v6::Limits) -> Result<wire::Mismatch, Error> {
    let mismatch = match source {
        tools::Mismatch::NoMatch => wire::Mismatch::NoMatch,
        tools::Mismatch::Unchanged => wire::Mismatch::Unchanged,
        tools::Mismatch::Ambiguous { count: found, lines } => {
            let mut output = List::with_capacity(count(lines.len())?);
            for line in lines {
                if output
                    .push(match wire::LineNumber::new(limits, wire::LineNumberParts { number: *line }) {
                        Ok(value) => value,
                        Err(problem) => return Err(Error::TranscriptBody(problem)),
                    })
                    .is_err()
                {
                    return Err(Error::Capacity);
                }
            }
            wire::Mismatch::Ambiguous(
                match wire::Ambiguous::new(limits, wire::AmbiguousParts { count: *found, lines: output }) {
                    Ok(value) => value,
                    Err(problem) => return Err(Error::TranscriptBody(problem)),
                },
            )
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

fn encode_exit(exit: tools::Exit, limits: &wire::v6::Limits) -> Result<wire::Exit, Error> {
    Ok(match exit {
        tools::Exit::Code { code } => {
            wire::Exit::Code(match wire::ExitCode::new(limits, wire::ExitCodeParts { code }) {
                Ok(value) => value,
                Err(problem) => return Err(Error::TranscriptBody(problem)),
            })
        }
        tools::Exit::Signal { signal } => {
            wire::Exit::Signal(match wire::ExitSignal::new(limits, wire::ExitSignalParts { signal }) {
                Ok(value) => value,
                Err(problem) => return Err(Error::TranscriptBody(problem)),
            })
        }
        tools::Exit::TimedOut => wire::Exit::TimedOut,
    })
}

fn encode_timeout(timeout: tools::TimeoutClamp, limits: &wire::v6::Limits) -> Result<wire::TimeoutClamp, Error> {
    Ok(match timeout {
        tools::TimeoutClamp::Within => wire::TimeoutClamp::Within,
        tools::TimeoutClamp::Clamped { asked, ceiling } => wire::TimeoutClamp::Clamped(match wire::DurationClamp::new(
            limits,
            wire::DurationClampParts { asked_ns: asked.as_nanos(), ceiling_ns: ceiling.as_nanos() },
        ) {
            Ok(value) => value,
            Err(problem) => return Err(Error::TranscriptBody(problem)),
        }),
    })
}

fn encode_bytes(output: tools::BytesClamp, limits: &wire::v6::Limits) -> Result<wire::BytesClamp, Error> {
    Ok(match output {
        tools::BytesClamp::Within => wire::BytesClamp::Within,
        tools::BytesClamp::Clamped { asked, ceiling } => {
            wire::BytesClamp::Clamped(match wire::ByteClamp::new(limits, wire::ByteClampParts { asked, ceiling }) {
                Ok(value) => value,
                Err(problem) => return Err(Error::TranscriptBody(problem)),
            })
        }
    })
}

fn encode_opening(
    affinity: llm::Affinity,
    account: Option<u32>,
    backend: &record::Backend,
    limits: &wire::v6::Limits,
) -> Result<wire::Opening, Error> {
    let affinity = match wire::Affinity::new(limits, wire::AffinityParts { key: affinity.key, thread: affinity.thread })
    {
        Ok(value) => value,
        Err(problem) => return Err(Error::TranscriptBody(problem)),
    };
    let backend = match backend {
        record::Backend::Direct => wire::Backend::Direct,
        record::Backend::Program { name } => {
            wire::Backend::Program(match wire::Program::new(limits, wire::ProgramParts { name: name.clone() }) {
                Ok(value) => value,
                Err(problem) => return Err(Error::TranscriptBody(problem)),
            })
        }
    };
    Ok(match wire::Opening::new(limits, wire::OpeningParts { affinity, account, backend }) {
        Ok(value) => value,
        Err(problem) => return Err(Error::TranscriptBody(problem)),
    })
}
