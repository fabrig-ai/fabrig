//! Local usage reads (protocol/programs.md, 5.11; protocol/host.md, 2).
//! Keeps one bounded child answer, never a run, slot or credential. The caller
//! drives `SourceInspection` first, as for `ProgramSource`, then `new`, `from_io`
//! and `fire`; `answer` returns once after every child resource settles.

mod words;

use alloc::boxed::Box;
use skein_io::{self as io, kernel};
use skein_json::{Document, Kind};
use skein_lib::{List, Queue, Time, Token, Wall};
use smith_host_domain::{FailureReason, ProgramFailure, ProgramUsage, SourceName};
use smith_protocol_program::{self as program, Command, Ended, Programs, USAGE_BYTES};

/// One host usage request on an inspected source, answered after its child settles.
#[derive(Debug)]
pub struct UsageRead {
    command: Command,
    detail_bytes: u32,
    asked: Wall,
    answered: bool,
}

impl UsageRead {
    /// Start after `SourceInspection` finds both levels private, placing reset years by the host's ask time.
    #[expect(clippy::too_many_arguments, reason = "a host's inspected source binding and deadline cross together")]
    pub fn new(
        io_owner: Token,
        programs: &Programs,
        source: SourceName,
        base: &[Box<[u8]>],
        root: kernel::Fd,
        directory: Box<[u8]>,
        deadline: Time,
        asked: Wall,
        detail_bytes: u32,
        below: &mut Queue<io::Request>,
    ) -> Result<Self, program::Error> {
        let configuration = program::configuration_directory(programs, source.as_bytes())?;
        let environment = environment(base, &configuration)?;
        let mut arguments = List::with_capacity(8);
        for argument in program::USAGE {
            arguments.push(Box::from(argument)).expect("usage argument slot");
        }
        let launch = program::Launch {
            program: programs.claude.clone(),
            arguments: arguments.into_boxed(),
            environment,
            root,
            directory,
        };
        Ok(Self {
            command: Command::with_output(io_owner, launch, USAGE_BYTES, deadline, detail_bytes, below),
            detail_bytes,
            asked,
            answered: false,
        })
    }

    /// Whether this child or pipe event belongs to the usage command.
    #[must_use]
    pub fn owns(&self, owner: Token) -> bool {
        self.command.owns(owner)
    }

    /// Drive a child or output-pipe event.
    pub fn from_io(&mut self, event: io::Event, below: &mut Queue<io::Request>) {
        self.command.from_io(event, below);
    }

    /// Enforce the same deadline used for directory inspection.
    pub fn fire(&mut self, now: Time, below: &mut Queue<io::Request>) {
        self.command.fire(now, below);
    }

    /// Stop the operation, retaining its resources until closure, as a source check does.
    pub fn cancel(&mut self, below: &mut Queue<io::Request>) {
        self.command.cancel(kernel::Reason::Cancelled, below);
    }

    /// The host deadline while the command is active.
    #[must_use]
    pub fn next_deadline(&self) -> Option<Time> {
        self.command.next_deadline()
    }

    /// Take one answer after the child and its output pipes settle.
    pub fn answer(&mut self) -> Option<ProgramUsage> {
        if self.answered {
            return None;
        }
        let ended = self.command.ended()?;
        let answer = translate(ended, self.detail_bytes, self.asked);
        self.answered = true;
        Some(answer)
    }
}

fn environment(base: &[Box<[u8]>], configuration: &[u8]) -> Result<Box<[Box<[u8]>]>, program::Error> {
    let mut entries = List::with_capacity(program::ENVIRONMENT_ENTRIES);
    for entry in base {
        let equal = skein_lib::bytes::find(entry, b"=").ok_or(program::Error::Environment)?;
        let name = entry.get(..equal).expect("entry name");
        if entry.contains(&0) {
            return Err(program::Error::Environment);
        }
        if name != b"TZ" && program::keeps(name) && entries.push(entry.clone()).is_err() {
            return Err(program::Error::Environment);
        }
    }
    if entries.push(Box::from(b"TZ=UTC".as_slice())).is_err() {
        return Err(program::Error::Environment);
    }
    program::environment(entries.as_slice(), configuration)
}

fn translate(ended: Ended<'_>, detail_bytes: u32, asked: Wall) -> ProgramUsage {
    match ended {
        Ended::Unspawned { detail } => failed(FailureReason::Missing, detail),
        Ended::Deadline { detail } => failed(FailureReason::Deadline, detail),
        Ended::Exited { exit, stdout, detail } => match exit {
            kernel::Exit::Code(0) => match stdout {
                Some(stdout) => match result(stdout, detail_bytes, asked) {
                    Some(answer) => answer,
                    None => failed(FailureReason::Unreadable, detail),
                },
                None => failed(FailureReason::Unreadable, detail),
            },
            kernel::Exit::Code(_) | kernel::Exit::Signal(_) => failed(FailureReason::Exited, detail),
        },
    }
}

fn failed(reason: FailureReason, detail: &[u8]) -> ProgramUsage {
    ProgramUsage::Failed(ProgramFailure { reason, detail: Box::from(detail) })
}

fn result(bytes: &[u8], detail_bytes: u32, asked: Wall) -> Option<ProgramUsage> {
    let document = program::usage_result(bytes)?;
    if spent(&document) {
        return Some(ProgramUsage::Spent);
    }
    let error = document.token(document.member(0, b"is_error")?)?.kind;
    let words = text(&document, document.member(0, b"result")?)?;
    match error {
        Kind::True => {
            let length = words.len().min(usize::try_from(detail_bytes).ok()?);
            Some(failed(FailureReason::Exited, words.get(..length)?))
        }
        Kind::False => {
            let windows = words::read(words, asked);
            if windows.is_empty() { None } else { Some(ProgramUsage::Read(windows)) }
        }
        Kind::ObjectStart
        | Kind::ObjectEnd
        | Kind::ArrayStart
        | Kind::ArrayEnd
        | Kind::Key
        | Kind::String
        | Kind::Number
        | Kind::Null
        | Kind::Long => None,
    }
}

fn text(document: &Document, index: u32) -> Option<&[u8]> {
    let token = document.token(index)?;
    if token.kind == Kind::String { document.text(token) } else { None }
}

fn spent(document: &Document) -> bool {
    if let Some(index) = document.member(0, b"total_cost_usd")
        && positive(document, index)
    {
        return true;
    }
    if let Some(usage) = document.member(0, b"usage")
        && let Some(end) = document.value_end(usage)
    {
        for index in usage..end {
            if let Some(key) = document.token(index)
                && key.kind == Kind::Key
                && let Some(name) = document.text(key)
                && name.ends_with(b"_tokens")
                && let Some(value) = index.checked_add(1)
                && positive(document, value)
            {
                return true;
            }
        }
    }
    false
}

fn positive(document: &Document, index: u32) -> bool {
    let Some(token) = document.token(index) else {
        return false;
    };
    if token.kind != Kind::Number {
        return false;
    }
    let Some(bytes) = document.text(token) else {
        return false;
    };
    if bytes.starts_with(b"-") {
        return false;
    }
    // JSON has already validated the number. Its exponent cannot turn a zero
    // mantissa positive, or a positive mantissa zero.
    for byte in bytes {
        if *byte == b'e' || *byte == b'E' {
            break;
        }
        if (b'1'..=b'9').contains(byte) {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests;
