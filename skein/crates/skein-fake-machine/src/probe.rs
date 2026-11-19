//! Shared reach probe vocabulary (testing/README.md, 4).
//!
//! Each argument asks one question; each report line answers one question,
//! in order. Paths are hex bytes, so non-UTF-8 paths, spaces and newlines
//! round-trip. Nested questions and answers occupy their parent's line.
//! This is a fixture format, not an engine, worker or client wire.

mod codec;

use skein_io::kernel::{Error, Rights};

/// One operation observed inside the spawned tree, not by its owner.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Question {
    /// Open an existing file for reading.
    Read(Box<[u8]>),
    /// Open an existing file for writing, without creating or truncating it.
    Write(Box<[u8]>),
    /// Open a directory for listing.
    List(Box<[u8]>),
    /// Execute the named program.
    Execute(Box<[u8]>),
    /// Read the process's no-new-privileges bit.
    NoNewPrivs,
    /// Execute the named set-id fixture and compare its ids with the probe's.
    SetId(Box<[u8]>),
    /// Try signalling the parent outside the spawned tree.
    SignalParent,
    /// Enumerate the descriptors held by the probe.
    Descriptors,
    /// Add a reach beneath the inherited layers, then ask these questions.
    Nest { rules: Box<[Rule]>, questions: Box<[Question]> },
}

/// A nested rule relative to the probe's working directory. Include any
/// support paths the real fixture needs to execute its nested probe too.
/// Absolute paths are refused as by `RESOLVE_BENEATH`.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Rule {
    pub path: Box<[u8]>,
    pub rights: Rights,
}

/// Whether executing the set-id fixture retained the probe's ids.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Ids {
    Kept,
    Gained,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Answer {
    Granted,
    Refused(Error),
    NoNewPrivs(bool),
    Ids(Ids),
    /// Sorted child descriptor numbers, with no helper descriptors.
    Descriptors(Box<[u32]>),
    Nested(Report),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Report {
    pub answers: Box<[Answer]>,
}

/// Arguments excluding argv[0], usable by a std fixture binary.
#[must_use]
pub fn arguments(questions: &[Question]) -> Box<[Box<[u8]>]> {
    questions.iter().map(|question| codec::question(question).into_bytes().into_boxed_slice()).collect()
}

pub fn questions(args: &[Box<[u8]>]) -> Result<Box<[Question]>, Error> {
    args.iter().map(|arg| codec::read_question(arg)).collect()
}

#[must_use]
pub fn encode(report: &Report) -> Box<[u8]> {
    let mut text = String::new();
    for answer in &report.answers {
        text.push_str(&codec::answer(answer));
        text.push('\n');
    }
    text.into_bytes().into_boxed_slice()
}

pub fn decode(bytes: &[u8]) -> Result<Report, Error> {
    let text = core::str::from_utf8(bytes).map_err(|_| Error::InvalidArgument)?;
    if !text.is_empty() && !text.ends_with('\n') {
        return Err(Error::InvalidArgument);
    }
    Ok(Report { answers: text.lines().map(|line| codec::read_answer(line.as_bytes())).collect::<Result<_, _>>()? })
}

#[cfg(test)]
mod tests;
