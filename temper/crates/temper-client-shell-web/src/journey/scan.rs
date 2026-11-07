//! The last scan (temper/docs/design/client/journeys.md, 3.7–3.8).
use super::{Line, Originals, Streams, Undecoded, fields_mut};
use skein_lib::bytes::find;
mod join;
mod shapes;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shape {
    Key,
    Jwt,
    Bearer,
    Code,
    RefreshToken,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum What {
    Original,
    Shape(Shape),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Place {
    Field,
    Record,
    Plain,
    RunText,
}
/// A refusal names its record and kind, never its bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Found {
    pub line: u32,
    pub record: &'static str,
    pub place: Place,
    pub what: What,
}

/// Seek originals, their digests, and the shapes of journeys.md, 3.7.
#[must_use]
pub fn bytes(bytes: &[u8], originals: &Originals) -> Option<What> {
    if original(bytes, originals) { Some(What::Original) } else { shape(bytes).map(What::Shape) }
}
/// Seek every form and digest of a sought original.
#[must_use]
pub fn original(bytes: &[u8], originals: &Originals) -> bool {
    for original in originals.sought() {
        if original.forms().iter().any(|form| find(bytes, form).is_some())
            || original.digests().iter().any(|(digest, _)| find(bytes, digest).is_some())
        {
            return true;
        }
    }
    false
}
/// Seek the shapes of journeys.md, 3.7.
#[must_use]
pub fn shape(bytes: &[u8]) -> Option<Shape> {
    shapes::scan(bytes)
}
/// A page's html, its four escapes read back first.
#[must_use]
pub fn page(html: &str, originals: &Originals) -> Option<What> {
    let mut decoded = Vec::with_capacity(html.len());
    let mut rest = html.as_bytes();
    while let Some((&first, tail)) = rest.split_first() {
        let escape = [
            (b"&lt;".as_slice(), b'<'),
            (b"&gt;".as_slice(), b'>'),
            (b"&quot;".as_slice(), b'"'),
            (b"&amp;".as_slice(), b'&'),
        ]
        .into_iter()
        .find(|(text, _)| rest.starts_with(text));
        if let Some((text, byte)) = escape {
            decoded.push(byte);
            rest = &rest[text.len()..];
        } else {
            decoded.push(first);
            rest = tail;
        }
    }
    bytes(&decoded, originals)
}
/// Every field, a stream's records and plain text, and each run's joined text.
pub fn lines(lines: &[Line], originals: &Originals) -> Result<Option<Found>, Undecoded> {
    for (index, line) in lines.iter().enumerate() {
        for (field, _) in fields_mut(&mut line.clone()) {
            if let Some(what) = bytes(field, originals) {
                return Ok(Some(Found { line: number(index), record: line.kind(), place: Place::Field, what }));
            }
        }
    }
    let streams = Streams::read(lines)?;
    let mut contents: Vec<_> = streams
        .events()
        .iter()
        .map(|event| (event.line, Place::Record, event.record.as_ref()))
        .chain(streams.plain().iter().map(|plain| (plain.line, Place::Plain, plain.bytes.as_ref())))
        .collect();
    contents.sort_by_key(|(line, _, _)| *line);
    for (line, place, content) in contents {
        if let Some(what) = bytes(content, originals) {
            return Ok(Some(found(lines, line, place, what)));
        }
    }
    Ok(join::scan(&streams, originals).map(|(line, what)| found(lines, line, Place::RunText, what)))
}
fn number(index: usize) -> u32 {
    u32::try_from(index.checked_add(1).expect("recording line count fits usize"))
        .expect("recording line count fits u32")
}
fn found(lines: &[Line], line: u32, place: Place, what: What) -> Found {
    let index = usize::try_from(line).expect("recording line fits usize").checked_sub(1).expect("lines start at one");
    Found { line, record: lines[index].kind(), place, what }
}
#[cfg(test)]
mod tests;
