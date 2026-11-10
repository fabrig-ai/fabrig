//! Shared head readings (http.md, 2).
//!
//! Header fields (RFC 9110, 5): a name and a value, both bytes, the names
//! compared without regard to case; the lists some values are; and the
//! lines a head comes in.

use alloc::boxed::Box;

use skein_lib::{List, bytes};

use crate::Version;

/// A header field, as a head carries it: its name, and its value with the
/// whitespace around it trimmed (RFC 9110, 5.5).
///
/// Names are compared without regard to case ([`Header::is`]); the bytes
/// are kept as the peer sent them. A value is not checked as UTF-8: a field
/// whose value is text is the application's to check.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Header {
    pub name: Box<[u8]>,
    pub value: Box<[u8]>,
}

// skein/docs/design/net/http/README.md, 2: a header prints its name and its value's length.
impl core::fmt::Debug for Header {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Header").field("name", &self.name).field("value_len", &self.value.len()).finish()
    }
}

impl Header {
    /// Whether this field is named `name`, without regard to case.
    #[must_use]
    pub fn is(&self, name: &[u8]) -> bool {
        self.name.eq_ignore_ascii_case(name)
    }
}

/// Whether `byte` may be part of a token, as a field's name or a method is
/// (RFC 9110, 5.6.2).
#[must_use]
pub fn is_tchar(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || b"!#$%&'*+-.^_`|~".contains(&byte)
}

/// Whether `byte` may be part of a field's value (RFC 9110, 5.5): a visible
/// character, a space, a tab, or a byte past ASCII. No other control
/// character, and so no CR, LF or NUL.
#[must_use]
pub fn is_field_byte(byte: u8) -> bool {
    byte == b'\t' || byte == b' ' || (0x21..=0x7E).contains(&byte) || byte >= 0x80
}

/// Not empty, and every byte a tchar (RFC 9110, 5.6.2).
#[must_use]
pub fn is_token(bytes: &[u8]) -> bool {
    if bytes.is_empty() {
        return false;
    }
    for &byte in bytes {
        if !is_tchar(byte) {
            return false;
        }
    }
    true
}

/// The bytes a field takes in a head: its name, `: `, its value and a line's end.
#[must_use]
pub const fn field_len(name: usize, value: usize) -> Option<usize> {
    match name.checked_add(value) {
        Some(len) => len.checked_add(4),
        None => None,
    }
}

/// Whether `byte` is whitespace around a value: a space or a tab (RFC 9110,
/// 5.6.3).
pub(crate) fn is_ows(byte: u8) -> bool {
    byte == b' ' || byte == b'\t'
}

/// `bytes` without the spaces and tabs around them.
#[must_use]
pub fn trim(bytes: &[u8]) -> &[u8] {
    let mut start = bytes.len();
    for (at, &byte) in bytes.iter().enumerate() {
        if !is_ows(byte) {
            start = at;
            break;
        }
    }
    let mut end = start;
    for (at, &byte) in bytes.iter().enumerate().skip(start) {
        if !is_ows(byte) {
            end = at.saturating_add(1);
        }
    }
    bytes.get(start..end).expect("within the bytes")
}

/// The next element of a comma-separated list (RFC 9110, 5.6.1) in `value`
/// from `at`, trimmed, and where the one after it begins; `None` once past
/// the end. Empty elements are elements here, the one after a trailing
/// comma among them: the caller skips them, as a recipient must.
pub(crate) fn element(value: &[u8], at: usize) -> Option<(&[u8], usize)> {
    let rest = value.get(at..)?;
    let mut end = rest.len();
    for (offset, &byte) in rest.iter().enumerate() {
        if byte == b',' {
            end = offset;
            break;
        }
    }
    let element = trim(rest.get(..end)?);
    Some((element, at.checked_add(end)?.checked_add(1)?))
}

/// Whether any field named `name` among `headers` lists `token`, without
/// regard to case: `Connection: close`, `Connection: keep-alive`.
pub(crate) fn lists(headers: &[Header], name: &[u8], token: &[u8]) -> bool {
    for header in headers {
        if !header.is(name) {
            continue;
        }
        let mut at = 0;
        // Bounded by the value: each element moves past its comma.
        for _ in 0..=header.value.len() {
            let Some((element, next)) = element(&header.value, at) else { break };
            if element.eq_ignore_ascii_case(token) {
                return true;
            }
            at = next;
        }
    }
    false
}

/// A line's content: the bytes before its LF, and before a CR that
/// precedes it (RFC 9112, 2.2). `None` for bytes that do not end with LF:
/// a scan that reached its maximum first.
pub(crate) fn content(bytes: &[u8]) -> Option<&[u8]> {
    let line = bytes.strip_suffix(b"\n")?;
    match line.strip_suffix(b"\r") {
        Some(line) => Some(line),
        None => Some(line),
    }
}

/// Why a field cannot be read into the head.
pub(crate) enum FieldError {
    Malformed,
    Full,
}

/// `Name: value` (RFC 9112, 5): a token, a colon with no whitespace
/// before it, and a value, trimmed.
pub(crate) fn field(line: &[u8], headers: &mut List<Header>) -> Result<(), FieldError> {
    let mut colon = None;
    for (at, &byte) in line.iter().enumerate() {
        if byte == b':' {
            colon = Some(at);
            break;
        }
        if !is_tchar(byte) {
            return Err(FieldError::Malformed);
        }
    }
    let (name, value) = match colon {
        Some(at) if at > 0 => (line.get(..at), line.get(at.saturating_add(1)..)),
        Some(_) | None => return Err(FieldError::Malformed),
    };
    let name = name.expect("before the colon");
    let value = trim(value.expect("after the colon"));
    for &byte in value {
        if !is_field_byte(byte) {
            return Err(FieldError::Malformed);
        }
    }
    if headers.room() == 0 {
        return Err(FieldError::Full);
    }
    let header = Header { name: bytes::copy_of(name), value: bytes::copy_of(value) };
    headers.push(header).expect("room was checked");
    Ok(())
}

/// Whether the connection persists after the exchange (RFC 9112, 9.3): in
/// HTTP/1.1 unless it says `Connection: close`; in HTTP/1.0 only if it says
/// `Connection: keep-alive`, and not `close`.
pub(crate) fn persistent(version: Version, headers: &[Header]) -> bool {
    let close = lists(headers, b"connection", b"close");
    match version {
        Version::Http11 => !close,
        Version::Http10 => !close && lists(headers, b"connection", b"keep-alive"),
    }
}

/// What a head's transfer codings say, in order across its fields.
pub(crate) enum Coding {
    Absent,
    Chunked,
    /// Other codings followed by `chunked`, given once.
    Coded,
    /// No final `chunked`, repeated `chunked`, or nothing listed.
    Faulty,
}

/// The head's `Transfer-Encoding`: whether `chunked` is last and once,
/// and whether other codings precede it.
pub(crate) fn transfer_coding(headers: &[Header]) -> Coding {
    let mut present = false;
    let mut chunked = 0_u32;
    let mut other = false;
    let mut last_chunked = false;
    for header in headers {
        if !header.is(b"transfer-encoding") {
            continue;
        }
        present = true;
        let mut at = 0;
        // Bounded by the value: each element moves past its comma.
        for _ in 0..=header.value.len() {
            let Some((element, next)) = element(&header.value, at) else { break };
            at = next;
            if element.is_empty() {
                continue;
            }
            last_chunked = element.eq_ignore_ascii_case(b"chunked");
            if last_chunked {
                chunked = chunked.saturating_add(1);
            } else {
                other = true;
            }
        }
    }
    if !present {
        return Coding::Absent;
    }
    if !last_chunked || chunked > 1 {
        return Coding::Faulty;
    }
    if other {
        return Coding::Coded;
    }
    Coding::Chunked
}

/// What a head's `Content-Length` says.
pub(crate) enum Length {
    Absent,
    /// One length, however many times it is given (RFC 9110, 8.6).
    Valid(u64),
    /// Not a length, past a `u64`, two different ones, or none listed.
    Invalid,
}

pub(crate) fn content_length(headers: &[Header]) -> Length {
    let mut present = false;
    let mut length = None;
    for header in headers {
        if !header.is(b"content-length") {
            continue;
        }
        present = true;
        let mut at = 0;
        // Bounded by the value: each element moves past its comma.
        for _ in 0..=header.value.len() {
            let Some((element, next)) = element(&header.value, at) else { break };
            at = next;
            if element.is_empty() {
                continue;
            }
            let Some(this) = bytes::decimal(element) else { return Length::Invalid };
            match length {
                Some(earlier) if earlier != this => return Length::Invalid,
                Some(_) | None => length = Some(this),
            }
        }
    }
    match length {
        Some(length) => Length::Valid(length),
        None if present => Length::Invalid,
        None => Length::Absent,
    }
}
