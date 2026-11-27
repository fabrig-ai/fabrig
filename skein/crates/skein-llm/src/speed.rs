//! Provider speed names shared by request admission and response accounting
//! (llm/vocabulary.md, section 1.3).
use crate::wire::json;
use alloc::boxed::Box;
use skein_json::Document;
use skein_lib::bytes;

pub(crate) fn valid(name: &[u8]) -> bool {
    if name.is_empty() || name.len() > 32 {
        return false;
    }
    for &byte in name {
        if !(byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-' || byte == b'_') {
            return false;
        }
    }
    true
}

pub(crate) fn reported(value: (&Document, json::Span)) -> Option<Box<[u8]>> {
    let name = json::text_ref(value).ok()?;
    if valid(name) { Some(bytes::copy_of(name)) } else { None }
}

pub(crate) fn admit(speed: Option<&[u8]>) -> Result<(), crate::Error> {
    if let Some(name) = speed
        && !valid(name)
    {
        return Err(crate::Error::Invalid { cause: crate::Invalid::Speed });
    }
    Ok(())
}
