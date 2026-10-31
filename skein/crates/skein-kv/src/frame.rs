//! Versioned, checksummed commit frames.

#![expect(clippy::disallowed_types, reason = "encoded frames are bounded by the commit limit")]

use crate::Op;
use crate::crc::crc32c;
use alloc::boxed::Box;
use alloc::vec::Vec;

const MAGIC: &[u8; 4] = b"SKVC";
const VERSION: u16 = 1;
pub(crate) const HEADER: u32 = 22;
const CHECKSUM: u32 = 4;

/// A commit frame's header and checksum (skein/docs/design/kv.md, 3 and 7).
pub const FRAME_BYTES: u32 = HEADER.checked_add(CHECKSUM).expect("fixed frame overhead fits");

/// One put operation's encoded size, excluding the frame's overhead.
#[must_use]
pub fn put_bytes(key: u32, value: u32) -> Option<u32> {
    key.checked_add(value)?.checked_add(9)
}

/// One erase operation's encoded size, excluding the frame's overhead.
#[must_use]
pub fn erase_bytes(key: u32) -> Option<u32> {
    key.checked_add(5)
}

#[derive(PartialEq, Eq, Debug)]
pub(crate) struct Frame {
    pub(crate) number: u64,
    pub(crate) ops: Box<[Op]>,
}

pub(crate) fn encoded_len(ops: &[Op]) -> Option<usize> {
    let mut total = FRAME_BYTES;
    for op in ops {
        let size = match op {
            Op::Put { key, value } => put_bytes(u32::try_from(key.len()).ok()?, u32::try_from(value.len()).ok()?)?,
            Op::Erase { key } => erase_bytes(u32::try_from(key.len()).ok()?)?,
        };
        total = total.checked_add(size)?;
    }
    usize::try_from(total).ok()
}

pub(crate) fn encode(number: u64, ops: &[Op], limit: u32) -> Option<Box<[u8]>> {
    let total = encoded_len(ops)?;
    if total > usize::try_from(limit).ok()? {
        return None;
    }
    let count = u32::try_from(ops.len()).ok()?;
    let capacity = total.checked_sub(usize::try_from(FRAME_BYTES).ok()?)?;
    let mut body = Vec::with_capacity(capacity);
    for op in ops {
        match op {
            Op::Put { key, value } => {
                body.push(1);
                body.extend_from_slice(&u32::try_from(key.len()).ok()?.to_be_bytes());
                body.extend_from_slice(key);
                body.extend_from_slice(&u32::try_from(value.len()).ok()?.to_be_bytes());
                body.extend_from_slice(value);
            }
            Op::Erase { key } => {
                body.push(2);
                body.extend_from_slice(&u32::try_from(key.len()).ok()?.to_be_bytes());
                body.extend_from_slice(key);
            }
        }
    }
    let mut bytes = Vec::with_capacity(total);
    bytes.extend_from_slice(MAGIC);
    bytes.extend_from_slice(&VERSION.to_be_bytes());
    bytes.extend_from_slice(&number.to_be_bytes());
    bytes.extend_from_slice(&count.to_be_bytes());
    bytes.extend_from_slice(&u32::try_from(body.len()).ok()?.to_be_bytes());
    bytes.extend_from_slice(&body);
    bytes.extend_from_slice(&crc32c(&bytes).to_be_bytes());
    Some(bytes.into_boxed_slice())
}

#[must_use]
pub(crate) fn frame_len(head: &[u8], max: u32) -> Option<usize> {
    if head.len() < usize::try_from(HEADER).ok()? || head.get(..4)? != MAGIC || u16_at(head, 4)? != VERSION {
        return None;
    }
    let body = usize::try_from(u32_at(head, 18)?).ok()?;
    let total = usize::try_from(FRAME_BYTES).ok()?.checked_add(body)?;
    if total > usize::try_from(max).ok()? {
        return None;
    }
    Some(total)
}

pub(crate) fn decode(bytes: &[u8], max: u32) -> Option<Frame> {
    let total = frame_len(bytes, max)?;
    if bytes.len() != total {
        return None;
    }
    let crc_at = total.checked_sub(4)?;
    if u32_at(bytes, crc_at)? != crc32c(bytes.get(..crc_at)?) {
        return None;
    }
    let number = u64_at(bytes, 6)?;
    let count = usize::try_from(u32_at(bytes, 14)?).ok()?;
    let mut ops = Vec::new();
    let mut pos = usize::try_from(HEADER).ok()?;
    while pos < crc_at {
        if ops.len() >= count {
            return None;
        }
        let kind = *bytes.get(pos)?;
        pos = pos.checked_add(1)?;
        let key_len = usize::try_from(u32_at(bytes, pos)?).ok()?;
        pos = pos.checked_add(4)?;
        let key_end = pos.checked_add(key_len)?;
        let key = Box::from(bytes.get(pos..key_end)?);
        pos = key_end;
        match kind {
            1 => {
                let value_len = usize::try_from(u32_at(bytes, pos)?).ok()?;
                pos = pos.checked_add(4)?;
                let value_end = pos.checked_add(value_len)?;
                let value = Box::from(bytes.get(pos..value_end)?);
                pos = value_end;
                ops.push(Op::Put { key, value });
            }
            2 => ops.push(Op::Erase { key }),
            _ => return None,
        }
    }
    if pos != crc_at || ops.len() != count {
        return None;
    }
    Some(Frame { number, ops: ops.into_boxed_slice() })
}

pub(crate) fn u16_at(bytes: &[u8], at: usize) -> Option<u16> {
    let end = at.checked_add(2)?;
    Some(u16::from_be_bytes(bytes.get(at..end)?.try_into().ok()?))
}

pub(crate) fn u32_at(bytes: &[u8], at: usize) -> Option<u32> {
    let end = at.checked_add(4)?;
    Some(u32::from_be_bytes(bytes.get(at..end)?.try_into().ok()?))
}

pub(crate) fn u64_at(bytes: &[u8], at: usize) -> Option<u64> {
    let end = at.checked_add(8)?;
    Some(u64::from_be_bytes(bytes.get(at..end)?.try_into().ok()?))
}
