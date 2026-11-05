use super::DecodeError;
use alloc::boxed::Box;
use skein_json::writer;
use skein_lib::{List, bytes, usize_of};

pub(crate) fn append(out: &mut List<u8>, bytes: &[u8], cap: crate::Cap) -> Result<(), DecodeError> {
    if bytes.len() > usize_of(out.room()) {
        return Err(DecodeError::limit(cap, out.capacity()));
    }
    for &b in bytes {
        if out.push(b).is_err() {
            return Err(DecodeError::limit(cap, out.capacity()));
        }
    }
    Ok(())
}
pub(crate) fn clipped(bytes: &[u8], limit: u32) -> Box<[u8]> {
    let mut count = bytes.len().min(usize_of(limit));
    // Tokenizer strings are UTF-8. A prefix ends before a continuation byte,
    // so truncating diagnostics never creates a new malformed string.
    for _back in 0..4_u32 {
        match bytes.get(count) {
            Some(byte) if byte & 0xc0 == 0x80 => count = count.saturating_sub(1),
            Some(_) | None => break,
        }
    }
    bytes::copy_of(bytes.get(..count).expect("within bytes"))
}

pub(crate) fn measured(encoder: writer::Encoder, limits: writer::Limits, cap: crate::Cap) -> Result<u32, DecodeError> {
    match encoder.measured() {
        Ok(len) => Ok(len),
        Err(writer::Refusal::TooLong) => Err(DecodeError::limit(cap, limits.length)),
        Err(writer::Refusal::TooDeep) => Err(DecodeError::limit(crate::Cap::Depth, limits.depth)),
        Err(writer::Refusal::Text | writer::Refusal::Number) => Err(DecodeError::Malformed),
    }
}

/// The typed cap is the source of a local limit's diagnostic.
pub(crate) fn limit_detail(which: crate::Cap, bound: u64) -> Box<[u8]> {
    let digits = skein_lib::Decimal::of(bound);
    let length = which
        .name()
        .len()
        .checked_add(digits.as_bytes().len())
        .expect("fixed diagnostic prefix fits usize")
        .checked_add(b" exceeds bound ".len())
        .expect("fixed diagnostic fits usize");
    let mut out = skein_lib::Writer::new(length);
    out.put(which.name()).expect("measured cap name");
    out.put(b" exceeds bound ").expect("measured separator");
    out.put(digits.as_bytes()).expect("measured bound");
    out.finish()
}
