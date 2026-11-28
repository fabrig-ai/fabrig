//! Job hook paths and failure lines (temper/docs/design/domain/worker/jobs.md, 2),
//! and the time file (temper/docs/design/protocol/hooks.md, 2.3).
use alloc::boxed::Box;
use skein_lib::{Decimal, Writer, bytes};

#[cfg(test)]
mod tests;

/// The time file's repository-relative path.
pub const CONFIG: &[u8] = b".temper/hooks/config.toml";
/// The maximum size of a time file, in bytes.
pub const CONFIG_BYTES: usize = 4096;
const HOOK_DIR: &[u8] = b".temper/hooks/";

/// Times in seconds as written, before applying any default or ceiling.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Times {
    pub deliver: Option<u32>,
    pub finish: Option<u32>,
    pub merge: Option<u32>,
    pub close: Option<u32>,
}

/// Why a time file was refused, on a line from 1 or at 0 for the whole file.
#[derive(Debug, PartialEq, Eq)]
pub struct Fault {
    pub line: u32,
    pub why: Box<[u8]>,
}

impl Fault {
    /// The time file's path, the line number and the reason.
    #[must_use]
    pub fn text(&self) -> Box<[u8]> {
        bytes::concat(&[CONFIG, b":", Decimal::of(u64::from(self.line)).as_bytes(), b": ", &self.why])
    }
}

/// A line for a hook time cut to the project's ceiling. `moment` is one of
/// `deliver`, `finish`, `merge` or `close`.
#[must_use]
pub fn cut_text(moment: &[u8], asked: u32, ceiling: u32) -> Box<[u8]> {
    bytes::concat(&[
        b"hook `",
        HOOK_DIR,
        moment,
        b"`: asks ",
        Decimal::of(u64::from(asked)).as_bytes(),
        b" s, past the project's ceiling: runs at ",
        Decimal::of(u64::from(ceiling)).as_bytes(),
        b" s",
    ])
}

/// Read the entire time file, refusing it at the first faulty line.
pub fn times(file: &[u8]) -> Result<Times, Fault> {
    if file.len() > CONFIG_BYTES {
        return Err(fault(0, b"file too large"));
    }
    let mut times = Times { deliver: None, finish: None, merge: None, close: None };
    let mut seen = 0;
    let mut current = None;
    let mut rest = file;
    // Each line consumes at least one byte, including its newline if it has one.
    for line in 1..=u32::try_from(file.len()).expect("the time file is at most 4096 bytes") {
        if rest.is_empty() {
            break;
        }
        let raw;
        if let Some(end) = bytes::find(rest, b"\n") {
            let (body, after) = rest.split_at_checked(end).expect("the newline is in this slice");
            raw = body.strip_suffix(b"\r").unwrap_or(body);
            rest = after.strip_prefix(b"\n").expect("the split was at the newline");
        } else {
            raw = rest;
            rest = b"";
        }
        if !bytes::is_utf8(raw) {
            return Err(fault(line, b"not UTF-8"));
        }
        let body = match bytes::find(raw, b"#") {
            Some(comment) => raw.get(..comment).expect("the comment starts in this line"),
            None => raw,
        };
        let body = trim(body);
        if body.is_empty() {
            continue;
        }
        if let Some(header) = body.strip_prefix(b"[") {
            let moment = table(header, line)?;
            if seen & moment.bit() != 0 {
                return Err(fault(line, b"duplicate table"));
            }
            seen |= moment.bit();
            current = Some(moment);
            continue;
        }
        let Some((key, value)) = assignment(body) else {
            return Err(fault(line, b"does not parse"));
        };
        let Some(moment) = current else {
            return Err(fault(line, b"key before any header"));
        };
        if key != b"timeout_seconds" {
            return Err(Fault { line, why: bytes::concat(&[b"unknown key `", key, b"`"]) });
        }
        let seconds = moment.seconds(&mut times);
        if seconds.is_some() {
            return Err(fault(line, b"duplicate key"));
        }
        if value == 0 {
            return Err(fault(line, b"a value of 0"));
        }
        *seconds = Some(value);
    }
    Ok(times)
}

#[derive(Clone, Copy)]
enum Moment {
    Deliver,
    Finish,
    Merge,
    Close,
}

impl Moment {
    fn bit(self) -> u8 {
        match self {
            Self::Deliver => 1,
            Self::Finish => 2,
            Self::Merge => 4,
            Self::Close => 8,
        }
    }

    fn seconds(self, times: &mut Times) -> &mut Option<u32> {
        match self {
            Self::Deliver => &mut times.deliver,
            Self::Finish => &mut times.finish,
            Self::Merge => &mut times.merge,
            Self::Close => &mut times.close,
        }
    }
}

fn table(header: &[u8], line: u32) -> Result<Moment, Fault> {
    let Some(name) = header.strip_suffix(b"]") else {
        return Err(fault(line, b"does not parse"));
    };
    let name = trim(name);
    match name {
        b"deliver" => Ok(Moment::Deliver),
        b"finish" => Ok(Moment::Finish),
        b"merge" => Ok(Moment::Merge),
        b"close" => Ok(Moment::Close),
        _ if bare(name) => Err(Fault { line, why: bytes::concat(&[b"unknown table `", name, b"`"]) }),
        _ => Err(fault(line, b"does not parse")),
    }
}

fn assignment(body: &[u8]) -> Option<(&[u8], u32)> {
    let equal = bytes::find(body, b"=")?;
    let (key, value) = body.split_at_checked(equal)?;
    let key = trim(key);
    if !bare(key) {
        return None;
    }
    let value = bytes::decimal(trim(value.strip_prefix(b"=")?))?;
    let value = u32::try_from(value).ok()?;
    Some((key, value))
}

// A bare TOML name lets us distinguish an unknown name from malformed syntax.
fn bare(name: &[u8]) -> bool {
    for byte in name {
        if !byte.is_ascii_alphanumeric() && *byte != b'_' && *byte != b'-' {
            return false;
        }
    }
    !name.is_empty()
}

fn trim(mut body: &[u8]) -> &[u8] {
    for &byte in body {
        if byte != b' ' && byte != b'\t' {
            break;
        }
        body = body.get(1..).expect("there is a leading space to remove");
    }
    for &byte in body.iter().rev() {
        if byte != b' ' && byte != b'\t' {
            break;
        }
        let (_, head) = body.split_last().expect("there is a trailing space to remove");
        body = head;
    }
    body
}

fn fault(line: u32, why: &[u8]) -> Fault {
    Fault { line, why: bytes::copy_of(why) }
}

/// Splits a hook command into its repository name and its path in that repository.
#[must_use]
pub fn hook_path(command: &[u8]) -> Option<(&[u8], &[u8])> {
    let slash = bytes::find(command, b"/")?;
    let (repository, path) = command.split_at_checked(slash)?;
    let path = path.strip_prefix(b"/")?;
    if repository.is_empty() || !path.starts_with(HOOK_DIR) { None } else { Some((repository, path)) }
}

/// The one-line tail of a command that could not run its hook.
#[must_use]
pub fn hook_failure(path: &[u8], why: &[u8]) -> Box<[u8]> {
    bytes::concat(&[b"hook `", &one_line(path), b"`: ", why])
}

/// Keep the diagnostic's first line and the last bytes of its error output.
#[must_use]
pub fn tail_after_line(line: &[u8], tail: &[u8], most: u32) -> Box<[u8]> {
    let most = skein_lib::usize_of(most);
    if tail.is_empty() || most.saturating_sub(line.len()) <= 1 {
        return line.get(..line.len().min(most)).expect("bounded line").into();
    }
    let start = tail.len().saturating_sub(most.saturating_sub(line.len()).saturating_sub(1));
    bytes::concat(&[line, b"\n", tail.get(start..).expect("bounded output tail")])
}

fn one_line(input: &[u8]) -> Box<[u8]> {
    let mut len = 0_usize;
    for byte in input {
        len = len.checked_add(byte.escape_ascii().len()).expect("bounded hook path");
    }
    let mut text = Writer::new(len);
    for byte in input {
        for ascii in byte.escape_ascii() {
            text.put(core::slice::from_ref(&ascii)).expect("measured diagnostic");
        }
    }
    text.finish()
}
