//! Bounded raw DEFLATE decoding (skein/docs/design/data/deflate.md, 3; RFC 1951).

use skein_lib::List;

#[cfg(test)]
mod tests;

/// The order in which a block's header sends the widths of its width code
/// (RFC 1951, 3.2.7).
const ORDER: [u8; 19] = [16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15];

/// Each length symbol from 257: its base and extra bits (RFC 1951, 3.2.5).
const LENGTHS: [(u16, u8); 29] = [
    (3, 0),
    (4, 0),
    (5, 0),
    (6, 0),
    (7, 0),
    (8, 0),
    (9, 0),
    (10, 0),
    (11, 1),
    (13, 1),
    (15, 1),
    (17, 1),
    (19, 2),
    (23, 2),
    (27, 2),
    (31, 2),
    (35, 3),
    (43, 3),
    (51, 3),
    (59, 3),
    (67, 4),
    (83, 4),
    (99, 4),
    (115, 4),
    (131, 5),
    (163, 5),
    (195, 5),
    (227, 5),
    (258, 0),
];

/// Each distance symbol: its base and extra bits (RFC 1951, 3.2.5).
const DISTANCES: [(u16, u8); 30] = [
    (1, 0),
    (2, 0),
    (3, 0),
    (4, 0),
    (5, 1),
    (7, 1),
    (9, 2),
    (13, 2),
    (17, 3),
    (25, 3),
    (33, 4),
    (49, 4),
    (65, 5),
    (97, 5),
    (129, 6),
    (193, 6),
    (257, 7),
    (385, 7),
    (513, 8),
    (769, 8),
    (1025, 9),
    (1537, 9),
    (2049, 10),
    (3073, 10),
    (4097, 11),
    (6145, 11),
    (8193, 12),
    (12289, 12),
    (16385, 13),
    (24577, 13),
];

/// The most symbols one code holds: 286 literal/length and 30 distance
/// widths are sent together, and a width code has 19.
const SYMBOLS: usize = 320;

/// Short canonical codes are resolved by their first nine bits. Longer or
/// unused prefixes retain the canonical search (RFC 1951, 3.2.2).
const PREFIX_BITS: u32 = 9;
const PREFIXES: usize = 1 << PREFIX_BITS;

/// Inflates exactly one raw DEFLATE stream, clearing `out` first.
/// Its fixed capacity is the output limit; no trailing bytes are accepted.
#[must_use]
pub fn decode(data: &[u8], out: &mut List<u8>) -> Option<()> {
    out.clear();
    let end = blocks(data, out)?;
    if end != data.len() {
        return None;
    }
    Some(())
}

fn blocks(data: &[u8], out: &mut List<u8>) -> Option<usize> {
    let mut input = Input { bit: 0 };
    let mut last = false;
    // Each block takes at least three bits.
    for _block in 0..=data.len() {
        if last {
            break;
        }
        last = input.bits(data, 1)? == 1;
        match input.bits(data, 2)? {
            0 => stored(&mut input, data, out)?,
            1 => {
                let (literals, distances) = (Code::FixedLiterals, Code::FixedDistances);
                codes(&mut input, data, out, &literals, &distances)?;
            }
            2 => {
                let (literals, distances) = header(&mut input, data)?;
                codes(&mut input, data, out, &literals, &distances)?;
            }
            _ => return None,
        }
    }
    if !last {
        return None;
    }
    input.align()
}

/// A member's DEFLATE data, each byte's lowest bit first
/// (RFC 1951, 3.1.1).
struct Input {
    bit: usize,
}

impl Input {
    /// `width` bits, at most 16, the first lowest.
    fn bits(&mut self, data: &[u8], width: u32) -> Option<u32> {
        let width = width.min(16);
        let (value, available) = self.peek(data)?;
        if available < width {
            return None;
        }
        self.bit = self.bit.checked_add(usize::try_from(width).ok()?)?;
        Some(value & 1_u32.checked_shl(width)?.checked_sub(1)?)
    }

    /// Up to sixteen bits without advancing; padding never counts as input.
    fn peek(&self, data: &[u8]) -> Option<(u32, u32)> {
        let bytes = data.get(self.bit >> 3_u32..)?;
        let shift = u32::try_from(self.bit & 7).ok()?;
        let value = u32::from_le_bytes([
            bytes.first().copied().unwrap_or(0),
            bytes.get(1).copied().unwrap_or(0),
            bytes.get(2).copied().unwrap_or(0),
            0,
        ]);
        let available = bytes.len().min(3).checked_mul(8)?.checked_sub(usize::try_from(shift).ok()?)?;
        Some((value.checked_shr(shift)?, u32::try_from(available.min(16)).ok()?))
    }

    /// Skips to the next byte's start, and returns where that is.
    fn align(&mut self) -> Option<usize> {
        let at = self.bit.div_ceil(8);
        self.bit = at.checked_mul(8)?;
        Some(at)
    }
}

/// Fixed or canonical Huffman codes (RFC 1951, 3.2.2, 3.2.6).
/// Only a dynamic code needs counts and symbols in canonical order.
#[expect(clippy::large_enum_variant, reason = "DEFLATE decoding allocates no tables on the heap (data/deflate.md, 1)")]
enum Code {
    FixedLiterals,
    FixedDistances,
    Canonical { counts: [u16; 16], symbols: [u16; SYMBOLS], shortest: u8, prefixes: [u16; PREFIXES] },
}

/// The code whose symbols have these widths, 0 for a symbol unused; an
/// oversubscribed code is refused, and an incomplete one fails only on a
/// string that starts no code.
fn code(widths: &[u8]) -> Option<Code> {
    let mut counts = [0_u16; 16];
    let mut symbols = [0_u16; SYMBOLS];
    let mut shortest = 15_u8;
    for width in widths {
        let count = counts.get_mut(usize::from(*width))?;
        *count = count.checked_add(1)?;
        if *width != 0 {
            shortest = shortest.min(*width);
        }
    }
    *counts.get_mut(0)? = 0;
    let mut left = 1_i32;
    let mut offsets = [0_u16; 16];
    for width in 1..16_usize {
        let count = *counts.get(width)?;
        left = left.checked_mul(2)?.checked_sub(i32::from(count))?;
        if left < 0_i32 {
            return None;
        }
        if let Some(next) = offsets.get(width.checked_add(1)?).copied() {
            *offsets.get_mut(width.checked_add(1)?)? = next.checked_add(offsets.get(width)?.checked_add(count)?)?;
        }
    }
    for (symbol, width) in widths.iter().enumerate() {
        if *width != 0 {
            let offset = offsets.get_mut(usize::from(*width))?;
            *symbols.get_mut(usize::from(*offset))? = u16::try_from(symbol).ok()?;
            *offset = offset.checked_add(1)?;
        }
    }
    let mut prefixes = [0_u16; PREFIXES];
    let mut first = 0_usize;
    let mut index = 0_usize;
    for (width, count) in counts.iter().enumerate().skip(1) {
        let count = usize::from(*count);
        if width <= usize::try_from(PREFIX_BITS).ok()? {
            let shift = PREFIX_BITS.checked_sub(u32::try_from(width).ok()?)?;
            for offset in 0..count {
                let symbol = *symbols.get(index.checked_add(offset)?)?;
                let entry = symbol | u16::try_from(width).ok()?.checked_shl(PREFIX_BITS)?;
                let start = first.checked_add(offset)?.checked_shl(shift)?;
                let end = start.checked_add(1_usize.checked_shl(shift)?)?;
                prefixes.get_mut(start..end)?.fill(entry);
            }
        }
        first = first.checked_add(count)?.checked_mul(2)?;
        index = index.checked_add(count)?;
    }
    Some(Code::Canonical { counts, symbols, shortest, prefixes })
}

impl Code {
    fn decode(&self, input: &mut Input, data: &[u8]) -> Option<u16> {
        let (bits, available) = input.peek(data)?;
        let bits = bits.reverse_bits();
        let (symbol, width) = match self {
            Self::FixedLiterals => fixed_literal(bits)?,
            Self::FixedDistances => {
                let symbol = u16::try_from(bits.checked_shr(27)?).ok()?;
                if symbol >= 30 {
                    return None;
                }
                (symbol, 5)
            }
            Self::Canonical { counts, symbols, shortest, prefixes } => {
                let prefix = usize::try_from(bits >> 23_u32).ok()?;
                let entry = *prefixes.get(prefix)?;
                if entry == 0 {
                    canonical(bits, counts, symbols, *shortest)?
                } else {
                    (entry & 0x1ff, u32::from(entry >> PREFIX_BITS))
                }
            }
        };
        if width > available {
            return None;
        }
        input.bit = input.bit.checked_add(usize::try_from(width).ok()?)?;
        Some(symbol)
    }
}

/// The fixed codes' four canonical ranges (RFC 1951, 3.2.6).
fn fixed_literal(bits: u32) -> Option<(u16, u32)> {
    let seven = u16::try_from(bits.checked_shr(25)?).ok()?;
    if seven < 24 {
        return Some((seven.checked_add(256)?, 7));
    }
    let eight = u16::try_from(bits.checked_shr(24)?).ok()?;
    if eight < 192 {
        return Some((eight.checked_sub(48)?, 8));
    }
    if eight < 200 {
        return Some((eight.checked_add(88)?, 8));
    }
    let nine = u16::try_from(bits.checked_shr(23)?).ok()?;
    Some((nine.checked_sub(256)?, 9))
}

/// Compare canonical codes starting at the shortest width, over one peek.
fn canonical(bits: u32, counts: &[u16; 16], symbols: &[u16; SYMBOLS], shortest: u8) -> Option<(u16, u32)> {
    let mut width = u32::from(shortest);
    let mut first = 0_u32;
    let mut index = 0_u32;
    for count in counts.get(usize::from(shortest)..)? {
        let value = bits.checked_shr(32_u32.checked_sub(width)?)?;
        let count = u32::from(*count);
        if value < first.checked_add(count)? {
            let at = index.checked_add(value.checked_sub(first)?)?;
            let symbol = symbols.get(usize::try_from(at).ok()?).copied()?;
            return Some((symbol, width));
        }
        index = index.checked_add(count)?;
        first = first.checked_add(count)?.checked_shl(1)?;
        width = width.checked_add(1)?;
    }
    None
}

/// A stored block (RFC 1951, 3.2.4): its length, that length's complement,
/// and its bytes.
fn stored(input: &mut Input, data: &[u8], out: &mut List<u8>) -> Option<()> {
    let at = input.align()?;
    let [a, b, c, d] = <[u8; 4]>::try_from(data.get(at..at.checked_add(4)?)?).ok()?;
    let length = u16::from_le_bytes([a, b]);
    if length != !u16::from_le_bytes([c, d]) {
        return None;
    }
    let start = at.checked_add(4)?;
    let end = start.checked_add(usize::from(length))?;
    for byte in data.get(start..end)? {
        out.push(*byte).ok()?;
    }
    input.bit = end.checked_mul(8)?;
    Some(())
}

/// A block's header in its own codes (RFC 1951, 3.2.7): its literal/length
/// and distance codes.
fn header(input: &mut Input, data: &[u8]) -> Option<(Code, Code)> {
    let literals = usize::try_from(input.bits(data, 5)?.checked_add(257)?).ok()?;
    let distances = usize::try_from(input.bits(data, 5)?.checked_add(1)?).ok()?;
    let sent = usize::try_from(input.bits(data, 4)?.checked_add(4)?).ok()?;
    if literals > 286 || distances > 30 {
        return None;
    }
    let mut widths = [0_u8; 19];
    for symbol in ORDER.get(..sent)? {
        *widths.get_mut(usize::from(*symbol))? = u8::try_from(input.bits(data, 3)?).ok()?;
    }
    let width_code = code(&widths)?;
    let total = literals.checked_add(distances)?;
    let mut all = [0_u8; SYMBOLS];
    let mut filled = 0_usize;
    for _run in 0..total {
        if filled == total {
            break;
        }
        let (width, repeats) = match width_code.decode(input, data)? {
            width @ 0..=15 => (u8::try_from(width).ok()?, 1),
            16 => (*all.get(filled.checked_sub(1)?)?, input.bits(data, 2)?.checked_add(3)?),
            17 => (0, input.bits(data, 3)?.checked_add(3)?),
            18 => (0, input.bits(data, 7)?.checked_add(11)?),
            _ => return None,
        };
        let end = filled.checked_add(usize::try_from(repeats).ok()?)?;
        if end > total {
            return None;
        }
        for slot in all.get_mut(filled..end)? {
            *slot = width;
        }
        filled = end;
    }
    if filled != total || *all.get(256)? == 0 {
        return None;
    }
    Some((code(all.get(..literals)?)?, code(all.get(literals..total)?)?))
}

/// Copies a back-reference in growing, non-overlapping slices: an overlap
/// repeats what has just been copied. A full list retains the inflated prefix.
fn repeat(out: &mut List<u8>, distance: u32, length: u32) -> Option<()> {
    let from = out.len().checked_sub(distance)?;
    let fits = length.min(out.room());
    let end = out.len().checked_add(fits)?;
    // Each copy at least doubles the source available for the next, bounded
    // by the match's length (at most 258 in RFC 1951).
    for _ in 0..length {
        let left = end.checked_sub(out.len())?;
        if left == 0 {
            break;
        }
        let count = left.min(out.len().checked_sub(from)?);
        if count == 0 {
            return None;
        }
        out.extend_from_within(from..from.checked_add(count)?)?;
    }
    (fits == length).then_some(())
}

/// Reads a block's symbols in these codes to its end.
fn codes(input: &mut Input, data: &[u8], out: &mut List<u8>, literals: &Code, distances: &Code) -> Option<()> {
    // Each symbol takes a bit at least, so the member's bits bound them.
    let bound = data.len().checked_mul(8)?;
    for _symbol in 0..=bound {
        let symbol = literals.decode(input, data)?;
        match symbol {
            0..=255 => out.push(u8::try_from(symbol).ok()?).ok()?,
            256 => return Some(()),
            _ => {
                let (base, extra) = *LENGTHS.get(usize::from(symbol.checked_sub(257)?))?;
                let length = u32::from(base).checked_add(input.bits(data, u32::from(extra))?)?;
                let (base, extra) = *DISTANCES.get(usize::from(distances.decode(input, data)?))?;
                let distance = u32::from(base).checked_add(input.bits(data, u32::from(extra))?)?;
                repeat(out, distance, length)?;
            }
        }
    }
    None
}
