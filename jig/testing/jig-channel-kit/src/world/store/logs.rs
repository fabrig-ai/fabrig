//! Independently read observed payload framing (skein's data/kv.md, 6.1).
use jig_protocol_store::transcripts::frame::crc32c;
use skein_lib::{List, Reader};

/// Payload files linked between generations have one observed byte prefix.
#[must_use]
pub fn path(name: &[u8]) -> Option<Vec<u8>> {
    let marker = b"/store/logs/";
    let start = name.windows(marker.len()).position(|part| part == marker)?;
    Some(name.get(start.checked_add(7)?..)?.to_vec())
}

/// A committed extent row, not a component-family record.
#[must_use]
pub fn extent(key: &[u8], value: &[u8]) -> Option<(u64, u64)> {
    if key.len() != 10 || !key.starts_with(&jig_protocol_store::logs::EXTENTS) {
        return None;
    }
    let log = Reader::new(&key[2..]).u64()?;
    let mut value = Reader::new(value);
    let end = value.u64()?;
    if !value.is_empty() {
        return None;
    }
    Some((log, end))
}

/// One whole record, including decompression and both lengths, bounded before allocation.
#[must_use]
pub fn record(bytes: &[u8], bound: u32) -> Option<(usize, Box<[u8]>)> {
    let mut head = Reader::new(bytes);
    let length = head.u32()?;
    let frame = bytes.get(..usize::try_from(length).ok()?)?;
    let mut reader = Reader::new(frame);
    reader.u32()?;
    let kind = reader.u8()?;
    let size = reader.u32()?;
    if size > bound {
        return None;
    }
    let body = reader.bytes(length.checked_sub(17)?)?;
    let crc_end = frame.len().checked_sub(8)?;
    if reader.u32()? != crc32c(&frame[..crc_end]) || reader.u32()? != length || !reader.is_empty() {
        return None;
    }
    let record = match kind {
        0 => Box::from(body),
        1 => {
            let mut out = List::with_capacity(size);
            skein_deflate::decode(body, &mut out)?;
            out.into_boxed()
        }
        2..=u8::MAX => return None,
    };
    if record.len() != usize::try_from(size).ok()? {
        return None;
    }
    Some((frame.len(), record))
}

/// Keep every physical offset and the opaque turn's version while making its kind unread.
pub(super) fn corrupt(bytes: &[u8], limits: &jig_protocol_store::codec::View) -> Option<Box<[u8]>> {
    let bound = u32::try_from(jig_protocol_store::core_v6::TurnRecord::encoded_bound(&limits.core)?).ok()?;
    let (length, record) = record(bytes, bound)?;
    if length != bytes.len() {
        return None;
    }
    let turn = jig_protocol_store::codec::core::read_turn(&record, limits).ok()?;
    for kind in 3..=u8::MAX {
        let mut turn = turn.clone();
        *turn.transcript.get_mut(2)? = kind;
        let body = jig_protocol_store::codec::core::turn(&turn, limits).ok()?;
        let encoded = encode(&body)?;
        if encoded.len() == bytes.len() {
            return Some(encoded);
        }
    }
    None
}
fn encode(body: &[u8]) -> Option<Box<[u8]>> {
    let mut compressed = vec![0; skein_deflate::bound(body.len())?];
    let written = skein_deflate::encode(body, &mut compressed)?;
    let (kind, payload) = if written < body.len() { (1, &compressed[..written]) } else { (0, body) };
    let total = u32::try_from(payload.len().checked_add(17)?).ok()?;
    let mut out = total.to_be_bytes().to_vec();
    out.push(kind);
    out.extend(u32::try_from(body.len()).ok()?.to_be_bytes());
    out.extend(payload);
    out.extend(crc32c(&out).to_be_bytes());
    out.extend(total.to_be_bytes());
    Some(out.into_boxed_slice())
}
