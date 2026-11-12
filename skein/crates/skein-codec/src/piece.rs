//! Items in one byte-bounded piece (data/codec/README.md, section 3).

use alloc::boxed::Box;
use skein_lib::{Overflow, Reader, Writer, len_u64};

use crate::Reason;

/// Writes one item after its big-endian `u32` length, or writes nothing.
/// The owner's writer bounds the whole piece, including every length.
pub fn write(out: &mut Writer, item: &[u8]) -> Result<(), Overflow> {
    let Ok(length) = u32::try_from(item.len()) else { return Err(Overflow) };
    let needed = item.len().checked_add(4).ok_or(Overflow)?;
    if needed > out.room() {
        return Err(Overflow);
    }
    out.put(&length.to_be_bytes())?;
    out.put(item)
}

/// Splits the next item from a bounded piece, or returns `None` at its end.
/// Refuses a partial length or an item past the end before allocating.
/// The owner decodes each item and counts its decoded size against its sum.
pub fn split(reader: &mut Reader<'_>) -> Result<Option<Box<[u8]>>, Reason> {
    match split_bytes(reader)? {
        Some(bytes) => Ok(Some(Box::from(bytes))),
        None => Ok(None),
    }
}

// The patch walk indexes pieces in their owner's bytes without allocating a box
// for each item (data/codec/patches.md, 4 and 7).
pub(crate) fn split_bytes<'a>(reader: &mut Reader<'a>) -> Result<Option<&'a [u8]>, Reason> {
    if reader.is_empty() {
        return Ok(None);
    }
    let length = reader.u32().ok_or(Reason::Malformed)?;
    let bytes = reader.bytes(length).ok_or(Reason::Malformed)?;
    Ok(Some(bytes))
}

/// Splits the next item and charges its bytes and its length to `sum`;
/// `Reason::Bound` past `most` or on overflow.
pub fn split_within(reader: &mut Reader<'_>, sum: &mut u64, most: u64) -> Result<Option<Box<[u8]>>, Reason> {
    let Some(item) = split(reader)? else { return Ok(None) };
    let length = len_u64(item.len());
    *sum = sum.checked_add(length).ok_or(Reason::Bound)?.checked_add(4).ok_or(Reason::Bound)?;
    if *sum > most {
        return Err(Reason::Bound);
    }
    Ok(Some(item))
}

/// The items a piece holds; `Reason::Malformed` on a partial length or an item past the end.
pub fn count(bytes: &[u8]) -> Result<u32, Reason> {
    let mut reader = Reader::new(bytes);
    for count in 0..=reader.remaining() {
        if reader.is_empty() {
            return Ok(count);
        }
        let length = reader.u32().ok_or(Reason::Malformed)?;
        reader.skip(length).ok_or(Reason::Malformed)?;
    }
    unreachable!("each item consumes at least its four-byte length")
}

/// The next item of a piece `count` has counted.
pub fn counted(reader: &mut Reader<'_>) -> Box<[u8]> {
    split(reader).expect("count accepted the piece framing").expect("count includes the next item")
}
