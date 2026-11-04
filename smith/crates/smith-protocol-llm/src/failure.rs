//! Lossless neutral failure/evidence and bounded mechanical invalid-input feedback.
//! No diagnostic wording participates in tool decisions.

use alloc::boxed::Box;

use skein_lib::{Decimal, Token, Writer, bytes};
use skein_llm::client;
use smith_domain::{Event, llm};

use crate::{Context, Error};

/// Moves one observed shared Failed terminal into the root boundary unchanged in meaning.
/// Detail is exact under the secured receiving cap; the policy consumes/drops it
/// after observation and retains only typed class/evidence in facts and End.
/// An over-cap diagnostic is a sender contract refusal.
/// The component retains the lower client until its independent Closed terminal.
pub fn failed(
    context: Context,
    failure: skein_llm::Failure,
    evidence: client::Evidence,
    detail: Box<[u8]>,
    report: skein_llm::Report,
) -> Result<Event, Error> {
    if detail.len() > usize::try_from(context.receiving.max_failure_bytes).expect("u32 fits usize") {
        return Err(Error::Limit);
    }
    let failure = class(failure);
    let evidence = match evidence {
        client::Evidence::Unsent => llm::Evidence::Unsent,
        client::Evidence::Unknown => llm::Evidence::Unknown,
        client::Evidence::Response { status } => llm::Evidence::Response { status: Some(status) },
    };
    Ok(Event::Below(smith_domain::Below::Failed {
        owner: context.owner,
        failure,
        evidence,
        detail,
        report: llm::Report { code: report.code, request: report.request, field: report.field },
    }))
}

/// Each shared failure keeps its session class (domain/session.md, section 4).
pub(crate) fn class(failure: skein_llm::Failure) -> llm::Failure {
    match failure {
        skein_llm::Failure::Overloaded => llm::Failure::Overloaded,
        skein_llm::Failure::RateLimited { retry_after } => llm::Failure::RateLimited { retry_after },
        skein_llm::Failure::Unavailable => llm::Failure::Unavailable,
        skein_llm::Failure::TimedOut { phase } => llm::Failure::TimedOut {
            phase: match phase {
                skein_llm::Phase::Connect => llm::CompletionPhase::Connect,
                skein_llm::Phase::Handshake => llm::CompletionPhase::Handshake,
                skein_llm::Phase::Head => llm::CompletionPhase::Head,
                skein_llm::Phase::Idle => llm::CompletionPhase::Idle,
                skein_llm::Phase::Whole => llm::CompletionPhase::Whole,
            },
        },
        skein_llm::Failure::ContextTooLong => llm::Failure::ContextTooLong,
        skein_llm::Failure::Refused => llm::Failure::Refused,
        skein_llm::Failure::Unsupported => llm::Failure::Unsupported,
        skein_llm::Failure::Invalid => llm::Failure::Invalid,
        skein_llm::Failure::Unauthorized => llm::Failure::Unauthorized,
        skein_llm::Failure::Exhausted { retry_after } => llm::Failure::Exhausted { retry_after },
        skein_llm::Failure::Limit { which, bound } => {
            llm::Failure::Limit { which: llm::Limit::Path(cap(which)), bound }
        }
        skein_llm::Failure::Protocol => llm::Failure::Protocol,
        skein_llm::Failure::Cancelled => llm::Failure::Cancelled,
    }
}

/// Translates the observed shared cancellation terminal after real lower settlement.
/// A cancellation request alone never permits calling this function; already
/// won Completed/Failed terminals keep their values and consume context instead.
#[must_use]
pub fn cancelled(context: Context) -> Event {
    Event::Below(smith_domain::Below::Cancelled { owner: context.owner })
}

/// Reports a preparation refusal as an unsent root terminal; no wire call existed.
/// Unsupported is explicitly Invalid, with its distinct bounded diagnostic.
/// Fixed local ASCII detail is clipped only to the receiving diagnostic allowance.
#[must_use]
pub fn refusal(owner: Token, error: Error, max_failure_bytes: u32) -> Event {
    let (failure, text): (llm::Failure, &[u8]) = match error {
        Error::Limit => (llm::Failure::ApplicationLimit, b"skein Client preparation exceeds receiving limits"),
        Error::PathLimit { which, bound } => (
            llm::Failure::Limit { which: llm::Limit::Path(which), bound },
            b"native Client preparation exceeds its named path bound",
        ),
        Error::Invalid => (llm::Failure::Invalid, b"invalid tool prompt or configured call"),
        Error::Unsupported => (llm::Failure::Invalid, b"unsupported replay or configured call"),
    };
    let length = text.len().min(usize::try_from(max_failure_bytes).expect("u32 fits usize"));
    let detail = bytes::copy_of(text.get(..length).expect("clipped fixed local diagnostic"));
    Event::Below(smith_domain::Below::Failed {
        owner,
        failure,
        evidence: llm::Evidence::Unsent,
        detail,
        report: llm::Report::none(),
    })
}

pub(crate) fn problem(problem: llm::Problem, maximum: u32) -> Result<Box<[u8]>, Error> {
    match problem {
        llm::Problem::Oversize { bytes, bound } => return call_size_problem(bytes, Some(bound), maximum),
        llm::Problem::CutOff { bytes } => return call_size_problem(bytes, None, maximum),
        llm::Problem::JobsFull { jobs } => {
            return job_problem(
                b"invalid yield_seconds: at most ",
                jobs,
                b" jobs may run at once; job with stop stops one, or reads the end of one that ended",
                maximum,
            );
        }
        llm::Problem::UnknownJob { job } => {
            return job_problem(
                b"invalid job ",
                job,
                b": no running answer named it, or its end was already answered",
                maximum,
            );
        }
        llm::Problem::UnknownTool
        | llm::Problem::Nesting
        | llm::Problem::NotAnObject
        | llm::Problem::Missing { .. }
        | llm::Problem::WrongType { .. }
        | llm::Problem::BadValue { .. }
        | llm::Problem::TooLarge => {}
    }
    let (prefix, field): (&[u8], Option<Box<[u8]>>) = match problem {
        llm::Problem::UnknownTool => (b"unknown tool", None),
        llm::Problem::Nesting => (b"a sub-agent does its work itself", None),
        llm::Problem::NotAnObject => (b"input is not a complete JSON object", None),
        llm::Problem::Missing { field } => (b"missing field ", Some(field)),
        llm::Problem::WrongType { field } => (b"wrong field type ", Some(field)),
        llm::Problem::BadValue { field } => (b"invalid field value ", Some(field)),
        llm::Problem::TooLarge => (b"input exceeds receiving limits", None),
        llm::Problem::Oversize { .. }
        | llm::Problem::CutOff { .. }
        | llm::Problem::JobsFull { .. }
        | llm::Problem::UnknownJob { .. } => unreachable!("scalar problems rendered above"),
    };
    let mut length = prefix.len();
    if let Some(field) = &field {
        length = length.checked_add(2).ok_or(Error::Limit)?;
        for byte in field {
            let width = if printable(*byte) { 1_usize } else { 4 };
            length = length.checked_add(width).ok_or(Error::Limit)?;
        }
    }
    if length > usize::try_from(maximum).expect("u32 fits usize") {
        return Err(Error::Limit);
    }
    let mut output = Writer::new(length);
    output.put(prefix).expect("measured invalid-input prefix");
    if let Some(field) = field {
        output.put(b"\"").expect("measured opening quote");
        for byte in field {
            if printable(byte) {
                output.put(&[byte]).expect("measured literal field byte");
            } else {
                let high = bytes::hex_digit(byte >> 4_u8);
                let low = bytes::hex_digit(byte & 15_u8);
                output.put(&[b'\\', b'x', high, low]).expect("measured escaped field byte");
            }
        }
        output.put(b"\"").expect("measured closing quote");
    }
    Ok(output.finish())
}

fn printable(byte: u8) -> bool {
    (32..=126).contains(&byte) && byte != b'"' && byte != b'\\'
}

pub(crate) const WRITE_SIZE_SUFFIX: &[u8] = b" bytes; make a smaller write or an edit. For a larger file, write its first part within the bound, then extend it with edits; put as many edits in one call as fit the bound";

fn call_size_problem(bytes: u64, bound: Option<u32>, maximum: u32) -> Result<Box<[u8]>, Error> {
    let size = Decimal::of(bytes);
    let bound = match bound {
        Some(bound) => Some(Decimal::of(u64::from(bound))),
        None => None,
    };
    let (prefix, suffix): (&[u8], &[u8]) = match bound {
        Some(_) => (b"input too large: ", WRITE_SIZE_SUFFIX),
        None => (b"input cut off after ", b" bytes; make a smaller call and try again"),
    };
    let mut length = prefix.len().checked_add(size.as_bytes().len()).ok_or(Error::Limit)?;
    if let Some(bound) = &bound {
        length = length.checked_add(b" bytes, bound ".len()).ok_or(Error::Limit)?;
        length = length.checked_add(bound.as_bytes().len()).ok_or(Error::Limit)?;
    }
    length = length.checked_add(suffix.len()).ok_or(Error::Limit)?;
    if length > usize::try_from(maximum).expect("u32 fits usize") {
        return Err(Error::Limit);
    }
    let mut output = Writer::new(length);
    output.put(prefix).expect("measured note");
    output.put(size.as_bytes()).expect("measured size");
    if let Some(bound) = bound {
        output.put(b" bytes, bound ").expect("measured unit");
        output.put(bound.as_bytes()).expect("measured bound");
    }
    output.put(suffix).expect("measured remedy");
    Ok(output.finish())
}

pub(crate) const fn cap(which: skein_llm::Cap) -> llm::Cap {
    match which {
        skein_llm::Cap::Request => llm::Cap::Request,
        skein_llm::Cap::RequestHead => llm::Cap::RequestHead,
        skein_llm::Cap::ResponseHead => llm::Cap::ResponseHead,
        skein_llm::Cap::ResponseFields => llm::Cap::ResponseFields,
        skein_llm::Cap::Skip => llm::Cap::Skip,
        skein_llm::Cap::Field => llm::Cap::Field,
        skein_llm::Cap::Retained => llm::Cap::Retained,
        skein_llm::Cap::Tokens => llm::Cap::Tokens,
        skein_llm::Cap::Depth => llm::Cap::Depth,
        skein_llm::Cap::Strings => llm::Cap::Strings,
        skein_llm::Cap::Number => llm::Cap::Number,
        skein_llm::Cap::Reasoning => llm::Cap::Reasoning,
        skein_llm::Cap::OutputItems => llm::Cap::OutputItems,
        skein_llm::Cap::Metadata => llm::Cap::Metadata,
        skein_llm::Cap::Tools => llm::Cap::Tools,
        skein_llm::Cap::HistoryItems => llm::Cap::HistoryItems,
        skein_llm::Cap::Output => llm::Cap::Output,
        skein_llm::Cap::Answer => llm::Cap::Answer,
        skein_llm::Cap::ErrorBody => llm::Cap::ErrorBody,
        skein_llm::Cap::Value => llm::Cap::AccessToken,
    }
}

fn job_problem(prefix: &[u8], number: u32, suffix: &[u8], maximum: u32) -> Result<Box<[u8]>, Error> {
    let number = Decimal::of(u64::from(number));
    let length = prefix
        .len()
        .checked_add(number.as_bytes().len())
        .ok_or(Error::Limit)?
        .checked_add(suffix.len())
        .ok_or(Error::Limit)?;
    if length > usize::try_from(maximum).expect("u32 fits usize") {
        return Err(Error::Limit);
    }
    let mut writer = Writer::new(length);
    writer.put(prefix).expect("measured problem prefix");
    writer.put(number.as_bytes()).expect("measured problem number");
    writer.put(suffix).expect("measured problem suffix");
    Ok(writer.finish())
}

/// Bounds the rendered bytes of one job problem at the largest job count or identifier.
pub(crate) fn job_problem_bound() -> Option<u64> {
    let mut largest = 0;
    for problem in [llm::Problem::JobsFull { jobs: u32::MAX }, llm::Problem::UnknownJob { job: u32::MAX }] {
        let rendered = self::problem(problem, u32::MAX).ok()?;
        largest = largest.max(u64::try_from(rendered.len()).ok()?);
    }
    Some(largest)
}
