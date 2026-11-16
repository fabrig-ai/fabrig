//! Ordered script joining shared by the box and drive (testing/qa/deployment.md, 5).
use super::Document;
use crate::{Refusal, read};
use alloc::boxed::Box;
use jig_protocol_config::Path;
use skein_json::{Document as Json, Kind, writer};

/// Each source and running join is checked against the model document.
/// A refusal leaves the previous join intact so its shell can name the source.
#[derive(Debug)]
pub struct ScriptJoin {
    bytes: Box<[u8]>,
}
impl ScriptJoin {
    #[must_use]
    pub fn empty() -> Self {
        Self { bytes: skein_lib::bytes::copy_of(br#"{"cued":[],"asked":[],"otherwise":[]}"#) }
    }
    pub fn push(&mut self, document: &[u8], source: &[u8]) -> Result<(), Refusal> {
        Document::decode_with_scripts(document, Some(source))?;
        let previous = read::document(&self.bytes)?;
        let given = read::document(source)?;
        let limits = writer::Limits { depth: 32, length: crate::DOCUMENT_BYTES };
        let mut out = writer::Encoder::measure(&limits);
        joined(&previous, &given, &mut out)?;
        let length = match out.measured() {
            Ok(length) => length,
            Err(
                writer::Refusal::TooLong | writer::Refusal::TooDeep | writer::Refusal::Text | writer::Refusal::Number,
            ) => return Err(read::why(&Path::root().member(b"public document"), b"past byte bound")),
        };
        let mut out = writer::Encoder::write(length, &limits);
        joined(&previous, &given, &mut out)?;
        let bytes = out.finish();
        Document::decode_with_scripts(document, Some(&bytes))?;
        self.bytes = bytes;
        Ok(())
    }
    #[must_use]
    pub fn into_bytes(self) -> Box<[u8]> {
        self.bytes
    }
}
fn joined(previous: &Json, given: &Json, out: &mut writer::Encoder) -> Result<(), Refusal> {
    out.object_start();
    for name in [b"cued".as_slice(), b"asked"] {
        out.key(name);
        out.array_start();
        array_contents(previous, previous.member(0, name), out)?;
        array_contents(given, given.member(0, name), out)?;
        out.array_end();
    }
    out.key(b"otherwise");
    out.array_start();
    match given.member(0, b"otherwise") {
        Some(at) => array_contents(given, Some(at), out)?,
        None => array_contents(previous, previous.member(0, b"otherwise"), out)?,
    }
    out.array_end();
    out.object_end();
    Ok(())
}
fn array_contents(document: &Json, at: Option<u32>, out: &mut writer::Encoder) -> Result<(), Refusal> {
    let Some(at) = at else {
        return Ok(());
    };
    let path = Path::root();
    let missing = read::why(&path, b"invalid script array");
    let record = document.token(at).ok_or(missing.clone())?;
    if record.kind != Kind::ArrayStart {
        return Err(missing);
    }
    let start = at.checked_add(1).ok_or(missing.clone())?;
    let after = document.value_end(at).ok_or(missing.clone())?;
    let end = after.checked_sub(1).ok_or(missing.clone())?;
    for at in start..end {
        let record = document.token(at).ok_or(missing.clone())?;
        let text = document.text(record).ok_or(missing.clone())?;
        match record.kind {
            Kind::ObjectStart => out.object_start(),
            Kind::ObjectEnd => out.object_end(),
            Kind::ArrayStart => out.array_start(),
            Kind::ArrayEnd => out.array_end(),
            Kind::Key => out.key(text),
            Kind::String => out.string(text),
            Kind::Number => out.number(text),
            Kind::True => out.boolean(true),
            Kind::False => out.boolean(false),
            Kind::Null => out.null(),
            Kind::Long => return Err(missing),
        }
    }
    Ok(())
}
