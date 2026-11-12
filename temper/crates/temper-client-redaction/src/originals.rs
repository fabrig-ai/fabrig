//! The originals of a recording (temper/docs/design/client/web/journeys.md, 3.3 and 3.6).
use skein_lib::sha256::digest;
use std::{cmp::Reverse, fmt};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Link,
    Stored,
    Typed,
    Given,
    Approval,
    Credential,
    Bearer,
}

/// One original, sought everywhere. Its Debug prints its kind and length alone.
pub struct Original {
    kind: Kind,
    len: usize,
    forms: Vec<Box<[u8]>>,
    digests: Vec<([u8; 32], [u8; 32])>,
}
impl Original {
    #[must_use]
    pub fn kind(&self) -> Kind {
        self.kind
    }
    /// The placeholder's length, the original's bytes.
    #[must_use]
    #[expect(clippy::len_without_is_empty, reason = "an Original always has nonempty bytes")]
    pub fn len(&self) -> usize {
        self.len
    }
    /// Its forms, its bytes first, each once.
    #[must_use]
    pub fn forms(&self) -> &[Box<[u8]>] {
        &self.forms
    }
    /// The sha-256 pairs of the original and placeholder, including surrounding whitespace.
    #[must_use]
    pub fn digests(&self) -> &[([u8; 32], [u8; 32])] {
        &self.digests
    }
    fn new(kind: Kind, bytes: &[u8]) -> Self {
        let mut forms = vec![Box::from(bytes)];
        for form in [percent(bytes, false), percent(bytes, true), html(bytes)] {
            if !forms.contains(&form) {
                forms.push(form);
            }
        }
        if let Ok(text) = std::str::from_utf8(bytes) {
            let debug = format!("{text:?}");
            let form = Box::from(&debug.as_bytes()[1..debug.len() - 1]);
            if !forms.contains(&form) {
                forms.push(form);
            }
        }
        Self { kind, len: bytes.len(), forms, digests: vec![(digest(bytes), digest(&vec![b'x'; bytes.len()]))] }
    }
}
#[expect(clippy::missing_fields_in_debug, reason = "original bytes and digests must never appear in Debug")]
impl fmt::Debug for Original {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Original").field("kind", &self.kind).field("len", &self.len).finish()
    }
}

pub struct Originals {
    sought: Vec<Original>,
    counted: Vec<(Kind, Box<[u8]>)>,
}
impl Originals {
    /// The originals of the candidates given, then each given text (client/web/journeys.md, 3.6).
    #[must_use]
    pub fn new(mut candidates: Vec<(Kind, Box<[u8]>)>, given: &[Box<[u8]>]) -> Self {
        candidates.extend(given.iter().map(|value| (Kind::Given, value.clone())));
        let mut result = Self { sought: Vec::new(), counted: Vec::new() };
        for (kind, value) in &candidates {
            let bytes = match kind {
                Kind::Typed | Kind::Given => trim(value),
                Kind::Link | Kind::Stored | Kind::Approval | Kind::Credential | Kind::Bearer => value,
            };
            if bytes.iter().all(|byte| *byte == b'x') {
                continue;
            }
            if *kind == Kind::Typed
                && candidates.iter().any(|(other_kind, other)| {
                    *other_kind == Kind::Typed && trim(other).len() > bytes.len() && trim(other).starts_with(bytes)
                })
            {
                continue;
            }
            if !result.counted.iter().any(|(seen_kind, seen)| seen_kind == kind && seen.as_ref() == bytes) {
                result.counted.push((*kind, bytes.into()));
            }
            if *kind == Kind::Typed && bytes.len() < 4 {
                continue;
            }
            let index = result.sought.iter().position(|one| one.forms[0].as_ref() == bytes);
            let one = if let Some(index) = index {
                &mut result.sought[index]
            } else {
                result.sought.push(Original::new(*kind, bytes));
                result.sought.last_mut().expect("an original was pushed")
            };
            if bytes != value.as_ref() {
                let pair = (digest(value), digest(&placeholder(value)));
                if !one.digests.contains(&pair) {
                    one.digests.push(pair);
                }
            }
        }
        result.sought.sort_by_key(|one| Reverse(one.len));
        result
    }
    /// Longest first, each once.
    #[must_use]
    pub fn sought(&self) -> &[Original] {
        &self.sought
    }
    /// Distinct originals of a kind, sought or not; a typing run counts as one.
    #[must_use]
    pub fn count(&self, kind: Kind) -> u32 {
        u32::try_from(self.counted.iter().filter(|(seen, _)| *seen == kind).count()).expect("original count fits u32")
    }
}

fn trim(value: &[u8]) -> &[u8] {
    let start = value.iter().position(|byte| !space(*byte)).unwrap_or(value.len());
    let end = value.iter().rposition(|byte| !space(*byte)).map_or(start, |index| index + 1);
    &value[start..end]
}

fn space(byte: u8) -> bool {
    byte == b' ' || byte == b'\t' || byte == b'\r' || byte == b'\n'
}

/// The value's placeholder, with surrounding spaces, tabs, CRs and LFs standing.
#[must_use]
pub fn placeholder(value: &[u8]) -> Box<[u8]> {
    let bytes = trim(value);
    let start = value.iter().position(|byte| !space(*byte)).unwrap_or(value.len());
    let mut result = value.to_vec();
    result[start..start + bytes.len()].fill(b'x');
    result.into()
}

fn percent(bytes: &[u8], lower: bool) -> Box<[u8]> {
    let hex = if lower { b"0123456789abcdef" } else { b"0123456789ABCDEF" };
    let mut out = Vec::new();
    for &byte in bytes {
        if byte.is_ascii_alphanumeric() || b"-._~".contains(&byte) {
            out.push(byte);
        } else {
            out.extend_from_slice(&[b'%', hex[usize::from(byte >> 4)], hex[usize::from(byte & 15)]]);
        }
    }
    out.into()
}

// The four dom::print escapes (temper/docs/design/client/web/journeys.md, 5.3).
fn html(bytes: &[u8]) -> Box<[u8]> {
    let mut out = Vec::new();
    for &byte in bytes {
        match byte {
            b'<' => out.extend_from_slice(b"&lt;"),
            b'>' => out.extend_from_slice(b"&gt;"),
            b'"' => out.extend_from_slice(b"&quot;"),
            b'&' => out.extend_from_slice(b"&amp;"),
            _ => out.push(byte),
        }
    }
    out.into()
}

#[cfg(test)]
mod tests;
