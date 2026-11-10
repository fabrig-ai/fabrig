//! Checksummed snapshot format. Chunks are independently validated; a
//! complete trailer is required before any snapshot is accepted.

#![expect(clippy::disallowed_types, reason = "snapshot chunks and recovery are bounded by configured limits")]

use crate::Row;
use crate::crc::crc32c;
use crate::frame::{u16_at, u32_at, u64_at};
use alloc::boxed::Box;
use alloc::vec::Vec;

const MAGIC: &[u8; 4] = b"SKVS";
const END: &[u8; 4] = b"END!";
const TRAILER: &[u8; 4] = b"SNPE";
const VERSION: u16 = 1;
const CHUNK_BYTES: u32 = 12;
const ROW_BYTES: u32 = 8;
pub(crate) const HEADER: usize = 18;
pub(crate) const TRAILER_LEN: usize = 28;

/// A single-row snapshot chunk's encoded size, including its lengths and checksum
/// (skein/docs/design/data/kv.md, 7). Excludes the snapshot header and trailer.
#[must_use]
pub fn chunk_bytes(key: u32, value: u32) -> Option<u32> {
    row_bytes(key, value)?.checked_add(CHUNK_BYTES)
}

fn row_bytes(key: u32, value: u32) -> Option<u32> {
    key.checked_add(value)?.checked_add(ROW_BYTES)
}

#[must_use]
pub(crate) fn header(start: u64) -> Box<[u8]> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(MAGIC);
    bytes.extend_from_slice(&VERSION.to_be_bytes());
    bytes.extend_from_slice(&start.to_be_bytes());
    bytes.extend_from_slice(&crc32c(&bytes).to_be_bytes());
    bytes.into_boxed_slice()
}

pub(crate) fn chunk(rows: &[Row], limit: u32) -> Option<Box<[u8]>> {
    if rows.is_empty() {
        return None;
    }
    let mut total = CHUNK_BYTES;
    for row in rows {
        total =
            total.checked_add(row_bytes(u32::try_from(row.key.len()).ok()?, u32::try_from(row.value.len()).ok()?)?)?;
    }
    if total > limit {
        return None;
    }
    let body_len = total.checked_sub(CHUNK_BYTES)?;
    let mut bytes = Vec::with_capacity(usize::try_from(total).ok()?);
    bytes.extend_from_slice(&u32::try_from(rows.len()).ok()?.to_be_bytes());
    bytes.extend_from_slice(&body_len.to_be_bytes());
    for row in rows {
        bytes.extend_from_slice(&u32::try_from(row.key.len()).ok()?.to_be_bytes());
        bytes.extend_from_slice(&u32::try_from(row.value.len()).ok()?.to_be_bytes());
        bytes.extend_from_slice(&row.key);
        bytes.extend_from_slice(&row.value);
    }
    bytes.extend_from_slice(&crc32c(&bytes).to_be_bytes());
    Some(bytes.into_boxed_slice())
}

#[must_use]
pub(crate) fn trailer(rows: u64, bytes: u64) -> Box<[u8]> {
    let mut trailer = Vec::new();
    trailer.extend_from_slice(TRAILER);
    trailer.extend_from_slice(&rows.to_be_bytes());
    trailer.extend_from_slice(&bytes.to_be_bytes());
    trailer.extend_from_slice(&crc32c(&trailer).to_be_bytes());
    trailer.extend_from_slice(END);
    trailer.into_boxed_slice()
}

/// Incremental snapshot validation (skein/docs/design/data/kv.md, 3, 7 and 12).
/// The buffer holds only the header, one chunk, or the trailer; rows leave
/// only after their whole chunk checks.
#[derive(Debug)]
pub(crate) struct Reader {
    bytes: Vec<u8>,
    stage: ReadStage,
}

#[derive(Debug)]
enum ReadStage {
    Header,
    Chunks { start: u64, rows: u64, payload: u64, previous: Option<Box<[u8]>> },
    Finished { start: u64 },
}

#[derive(Debug)]
pub(crate) enum Part {
    NeedMore,
    Start { start: u64 },
    Rows { rows: Box<[Row]> },
    End,
}

impl Reader {
    pub(crate) fn new() -> Reader {
        Reader { bytes: Vec::new(), stage: ReadStage::Header }
    }

    /// Consumes at most one record, leaving the rest of this read to the caller.
    /// None means corruption; `NeedMore` means the record needs another read.
    pub(crate) fn next(&mut self, input: &mut &[u8], chunk_limit: u32) -> Option<Part> {
        match &mut self.stage {
            ReadStage::Header => {
                if !fill(&mut self.bytes, input, HEADER)? {
                    return Some(Part::NeedMore);
                }
                let bytes = &self.bytes;
                if bytes.get(..4)? != MAGIC
                    || u16_at(bytes, 4)? != VERSION
                    || u32_at(bytes, 14)? != crc32c(bytes.get(..14)?)
                {
                    return None;
                }
                let start = u64_at(bytes, 6)?;
                if start == 0 {
                    return None;
                }
                self.bytes.clear();
                self.stage = ReadStage::Chunks { start, rows: 0, payload: 0, previous: None };
                Some(Part::Start { start })
            }
            ReadStage::Chunks { start, rows, payload, previous } => {
                if !fill(&mut self.bytes, input, 8)? {
                    return Some(Part::NeedMore);
                }
                if self.bytes.get(..4)? == TRAILER {
                    if !fill(&mut self.bytes, input, TRAILER_LEN)? {
                        return Some(Part::NeedMore);
                    }
                    let bytes = &self.bytes;
                    if bytes.get(24..)? != END
                        || u32_at(bytes, 20)? != crc32c(bytes.get(..20)?)
                        || u64_at(bytes, 4)? != *rows
                        || u64_at(bytes, 12)? != *payload
                    {
                        return None;
                    }
                    self.stage = ReadStage::Finished { start: *start };
                    self.bytes.clear();
                    return Some(Part::End);
                }
                let count = usize::try_from(u32_at(&self.bytes, 0)?).ok()?;
                let len = usize::try_from(u32_at(&self.bytes, 4)?).ok()?;
                let total = len.checked_add(usize::try_from(CHUNK_BYTES).ok()?)?;
                if count == 0 || total > usize::try_from(chunk_limit).ok()? {
                    return None;
                }
                if !fill(&mut self.bytes, input, total)? {
                    return Some(Part::NeedMore);
                }
                let decoded = decode_chunk(&self.bytes, count, previous, payload)?;
                *rows = rows.checked_add(skein_lib::len_u64(decoded.len()))?;
                self.bytes.clear();
                Some(Part::Rows { rows: decoded })
            }
            ReadStage::Finished { .. } => None,
        }
    }

    /// A checked trailer must end exactly at the file's reported length.
    pub(crate) fn finish(&self) -> Option<u64> {
        match self.stage {
            ReadStage::Finished { start } => Some(start),
            ReadStage::Header | ReadStage::Chunks { .. } => None,
        }
    }
}

fn fill(bytes: &mut Vec<u8>, input: &mut &[u8], total: usize) -> Option<bool> {
    if bytes.len() >= total {
        return Some(true);
    }
    let left = total.checked_sub(bytes.len())?;
    let take = left.min(input.len());
    // Reserve only the record's validated length, without geometric growth.
    bytes.reserve_exact(total.checked_sub(bytes.len())?);
    bytes.extend_from_slice(input.get(..take)?);
    *input = input.get(take..)?;
    Some(bytes.len() == total)
}

fn decode_chunk(bytes: &[u8], count: usize, previous: &mut Option<Box<[u8]>>, payload: &mut u64) -> Option<Box<[Row]>> {
    let crc_at = bytes.len().checked_sub(4)?;
    if u32_at(bytes, crc_at)? != crc32c(bytes.get(..crc_at)?) {
        return None;
    }
    let body = bytes.get(8..crc_at)?;
    if count > body.len().checked_div(usize::try_from(ROW_BYTES).ok()?)? {
        return None;
    }
    let mut rows = Vec::with_capacity(count);
    let mut cursor = 0_usize;
    for _ in 0..count {
        let key_len = usize::try_from(u32_at(body, cursor)?).ok()?;
        cursor = cursor.checked_add(4)?;
        let value_len = usize::try_from(u32_at(body, cursor)?).ok()?;
        cursor = cursor.checked_add(4)?;
        let key_end = cursor.checked_add(key_len)?;
        let key: Box<[u8]> = Box::from(body.get(cursor..key_end)?);
        cursor = key_end;
        let value_end = cursor.checked_add(value_len)?;
        let value = Box::from(body.get(cursor..value_end)?);
        cursor = value_end;
        if let Some(old) = previous.as_ref()
            && old.as_ref() >= key.as_ref()
        {
            return None;
        }
        *previous = Some(key.clone());
        *payload = payload.checked_add(u64::try_from(key_len.checked_add(value_len)?).ok()?)?;
        rows.push(Row { key, value });
    }
    if cursor != body.len() {
        return None;
    }
    Some(rows.into_boxed_slice())
}
