//! Length-prefixed kept parts (jig/docs/design/protocol/channel/records.md, 5).
//! The byte envelope keeps the record's worst case within the codec's u32 bound.
use crate::v22::{self as wire, Record};
use alloc::boxed::Box;
use skein_codec::Reason;
use skein_lib::{List, Reader, Writer};

const COUNT: u32 = 4096;

/// Encode a start's parts, each preceded by its four-byte length.
pub fn encode(parts: &[wire::KeptPart], limits: &wire::Limits) -> Result<Box<[u8]>, wire::Problem> {
    let path = wire::paths::ASSIGNMENT_PARTS;
    if parts.len() > skein_lib::length::usize_of(COUNT) {
        return Err(path.problem(Reason::Bound));
    }
    let mut length = 0_u32;
    for part in parts {
        length = length
            .checked_add(part.measure(limits)?)
            .ok_or(path.problem(Reason::Bound))?
            .checked_add(4)
            .ok_or(path.problem(Reason::Bound))?;
    }
    if length > limits.assignment_parts.min(wire::CEILINGS.assignment_parts) {
        return Err(path.problem(Reason::Bound));
    }
    let mut writer = Writer::new(skein_lib::length::usize_of(length));
    for part in parts {
        let bytes = part.encode(limits)?;
        skein_codec::piece::write(&mut writer, &bytes).or(Err(path.problem(Reason::Bound)))?;
    }
    Ok(writer.finish())
}

/// Decode a bounded start's parts before handing them to the hub.
pub fn decode(bytes: &[u8], limits: &wire::Limits) -> Result<List<wire::KeptPart>, wire::Problem> {
    let path = wire::paths::ASSIGNMENT_PARTS;
    if bytes.len() > skein_lib::length::usize_of(limits.assignment_parts.min(wire::CEILINGS.assignment_parts)) {
        return Err(path.problem(Reason::Bound));
    }
    let count = match skein_codec::piece::count(bytes) {
        Ok(count) => count,
        Err(reason) => return Err(path.problem(reason)),
    };
    if count > COUNT {
        return Err(path.problem(Reason::Bound));
    }
    let mut reader = Reader::new(bytes);
    let mut parts = List::with_capacity(count);
    for _ in 0..count {
        let bytes = skein_codec::piece::counted(&mut reader);
        let part = wire::KeptPart::decode(limits, &mut Reader::new(&bytes))?;
        parts.push(part).expect("counted parts fit");
    }
    Ok(parts)
}
