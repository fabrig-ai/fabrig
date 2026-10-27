//! Parse past a memory-only refusal to compose the whole start (protocol/configuration.md, 13.8).
use super::{Problems, What, member};
use jig_protocol_config::{self as config, Path, Segment, read};
use skein_json::{Compact, Document, document::Limits};
use skein_lib::{Decimal, List};

#[derive(Clone, Copy, Debug)]
pub(super) struct Envelope {
    pub declared: u64,
    pub need: u64,
}

/// Only a sole memory refusal at this member permits a typed retry.
pub(super) fn envelope(document: &Document, at: u32, found: &config::Problems) -> Option<Envelope> {
    if found.past != 0 || found.problems.len() != 1 {
        return None;
    }
    let problem = found.problems.get(0)?;
    if problem.path != Path::root().member(b"memory_bytes") {
        return None;
    }
    let config::What::Memory { over } = problem.what else { return None };
    let declared = read::integer(document, member(document, at, b"memory_bytes")?, u64::MAX).ok()?;
    Some(Envelope { declared, need: declared.checked_add(over)? })
}

/// Copy retained tokens, replacing one validated number. Token indices stay stable;
/// the original document and the person's declaration remain untouched.
pub(super) fn raised(document: &Document, at: u32, need: u64) -> Document {
    let number = Decimal::of(need);
    let extra = u32::try_from(number.as_bytes().len()).expect("u64 decimal length");
    let capacity = document.text_len().checked_add(extra).expect("bounded document plus one u64");
    let mut text = List::with_capacity(capacity);
    let mut tokens = List::with_capacity(document.len());
    for index in 0..document.len() {
        let mut token = *document.token(index).expect("retained token");
        let bytes = if index == at { number.as_bytes() } else { document.text(&token).expect("retained text") };
        if !bytes.is_empty() {
            token = Compact {
                kind: token.kind,
                start: text.len(),
                len: u32::try_from(bytes.len()).expect("bounded retained text"),
            };
            for byte in bytes {
                text.push(*byte).expect("document text plus one u64");
            }
        }
        tokens.push(token).expect("unchanged token count");
    }
    Document::from_parts(text.into_boxed(), tokens.into_boxed(), &Limits { tokens: document.len(), text: capacity })
        .expect("copied records and one validated number")
}

#[derive(Debug)]
pub(super) struct Retry {
    document: Document,
    entries: List<(Path, Envelope)>,
}
impl Retry {
    pub fn document(&self) -> &Document {
        &self.document
    }

    /// Retry a whole load only when every reported problem is memory at an
    /// engine or worker declaration. Other diagnostics keep their original order.
    pub fn new(document: &Document, found: &Problems) -> Option<Self> {
        if found.past != 0 || found.problems.is_empty() {
            return None;
        }
        let mut retry = Self { document: document.clone(), entries: List::with_capacity(found.problems.len()) };
        for problem in &found.problems {
            let What::Jig(config::What::Memory { over }) = problem.what else { return None };
            let mut at = 0;
            for segment in &problem.path.segments {
                at = match segment {
                    Segment::Member(name) => member(document, at, name)?,
                    Segment::Index(index) => *read::list(document, at, document.len()).ok()?.get(*index)?,
                };
            }
            let valid = problem.path == Path::root().member(b"engine").member(b"memory_bytes")
                || match problem.path.segments.as_slice() {
                    [Segment::Member(workers), Segment::Index(_), Segment::Member(memory)] => {
                        workers.as_ref() == b"workers" && memory.as_ref() == b"memory_bytes"
                    }
                    _ => false,
                };
            if !valid {
                return None;
            }
            let declared = read::integer(document, at, u64::MAX).ok()?;
            let envelope = Envelope { declared, need: declared.checked_add(over)? };
            retry.document = raised(&retry.document, at, envelope.need);
            retry.entries.push((problem.path.clone(), envelope)).expect("one entry per refusal");
        }
        Some(retry)
    }
    pub fn restore(&self, path: &Path, memory: &mut u64) -> u64 {
        for (at, envelope) in &self.entries {
            if at == path {
                *memory = envelope.declared;
                return envelope.need;
            }
        }
        0
    }
}

pub(super) fn worker(
    document: &Document,
    at: u32,
    directories: &config::Directories,
) -> Result<(config::Worker, u64), config::Problems> {
    let found = match config::worker(document, at, directories) {
        Ok(worker) => return Ok((worker, 0)),
        Err(found) => found,
    };
    let Some(envelope) = envelope(document, at, &found) else { return Err(found) };
    let copy = raised(document, member(document, at, b"memory_bytes").expect("checked memory"), envelope.need);
    let Ok(mut worker) = config::worker(&copy, at, directories) else { return Err(found) };
    worker.memory = envelope.declared;
    Ok((worker, envelope.need))
}
