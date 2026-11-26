//! The best level's bounded hash chains and lazy parse (skein/docs/design/data/deflate.md, 2).

use super::{
    Block, EMPTY, END, Encoder, LONGEST, Match, SHORTEST, WINDOW, align, block_at, blocks, bound, extent, fixed, hash,
    record, tally, word, write,
};

#[cfg(test)]
mod tests;

const CHAIN: usize = 4_096;
/// A good match cuts the total search to a quarter of the full chain.
const GOOD: usize = 32;
const MASK: usize = WINDOW - 1;

struct Best {
    encoder: Encoder,
    /// Per window slot, the preceding slot in the same hash bucket, plus one.
    /// Zero ends a chain. Positions move through the slots in input order.
    previous: [u16; WINDOW],
}

#[expect(clippy::large_const_arrays, reason = "every best encode owns its chain on the stack")]
const EMPTY_BEST: Best = Best { encoder: EMPTY, previous: [0; WINDOW] };

/// Writes the best level's raw DEFLATE stream into the front of `out`.
/// Returns its length, at most [`bound`] of the input's, or `None` when
/// `out` is shorter than that bound. The same input gives the same bytes.
#[must_use]
pub fn encode_best(input: &[u8], out: &mut [u8]) -> Option<usize> {
    if out.len() < bound(input.len())? {
        return None;
    }
    let mut best = EMPTY_BEST;
    best.encoder.fixed = fixed()?;
    let blocks = blocks(input.len());
    for index in 0..blocks {
        let block = block_at(index, blocks, input.len())?;
        parse(&mut best, input, block)?;
        write(&mut best.encoder, input, block, out)?;
    }
    align(out, &mut best.encoder.bits)?;
    Some(best.encoder.bits.at)
}

/// Parses one block, taking at least one byte per turn. The dictionary
/// carries through block boundaries, but matches cannot cross the block's end.
fn parse(best: &mut Best, body: &[u8], block: Block) -> Option<()> {
    best.encoder.parse.reset();
    let mut at = block.start;
    let mut found = None;
    for _ in block.start..block.end {
        if at >= block.end {
            break;
        }
        let here = match found.take() {
            Some(ahead) => Some(ahead), // Lookahead has already inserted this position.
            None => {
                let here = seek(best, body, block, at, SHORTEST.checked_sub(1)?);
                link(best, body, at)?;
                here
            }
        };
        let next = at.checked_add(1)?;
        if let Some(here) = here {
            // On a tie, keep the earlier match. Otherwise write one literal
            // and carry the better match into the next turn.
            found = seek(best, body, block, next, here.length);
            link(best, body, next)?;
            if found.is_none() {
                record(&mut best.encoder.parse, block, at, here)?;
                let end = at.checked_add(here.length)?;
                for inside in next.checked_add(1)?..end {
                    link(best, body, inside)?;
                }
                at = end;
                continue;
            }
        }
        tally(&mut best.encoder.parse.counts.literals, usize::from(*body.get(at)?))?;
        at = next;
    }
    tally(&mut best.encoder.parse.counts.literals, END)
}

/// Finds a match strictly longer than the held length. Search before
/// inserting `at`: its slot still holds the far end of the window.
fn seek(best: &Best, body: &[u8], block: Block, at: usize, mut length: usize) -> Option<Match> {
    let most = block.end.checked_sub(at)?.min(LONGEST);
    if most <= length {
        return None;
    }
    let bytes = word(body, at)?;
    let mut candidate = *best.encoder.heads.get(hash(bytes))?;
    let mut distance = 0;
    let mut found = None;
    for step in 0..CHAIN {
        if candidate == 0 || (length >= GOOD && step >= CHAIN / 4) {
            break;
        }
        let slot = usize::from(candidate).checked_sub(1)?;
        let back = at.wrapping_sub(slot) & MASK;
        let back = if back == 0 { WINDOW } else { back };
        // Links may still point outside the window. A wrap would make
        // the distance cease to grow, so do not follow it into a reused slot.
        if back <= distance {
            break;
        }
        distance = back;
        let from = at.checked_sub(back)?;
        candidate = *best.previous.get(slot)?;
        // To beat the held length, the next byte and the hash prefix must agree.
        if body.get(at.checked_add(length)?)? != body.get(from.checked_add(length)?)? || word(body, from)? != bytes {
            continue;
        }
        let longer = extent(body, at, from, most)?;
        if longer <= length {
            continue;
        }
        length = longer;
        found = Some(Match { length, distance });
        if length == most {
            break;
        }
    }
    found
}

/// Inserts `at` in its hash bucket, retiring the slot's old position.
fn link(best: &mut Best, body: &[u8], at: usize) -> Option<()> {
    let Some(bytes) = word(body, at) else { return Some(()) };
    let slot = at & MASK;
    let head = u16::try_from(slot.checked_add(1)?).ok()?;
    if let Some(expired) = at.checked_sub(WINDOW) {
        // A bucket's head must still be in the window. Older links can be
        // left alone: the search stops when their distances wrap.
        let expired_bucket = hash(word(body, expired)?);
        let old = best.encoder.heads.get_mut(expired_bucket)?;
        if *old == head {
            *old = 0;
        }
    }
    let old = best.encoder.heads.get_mut(hash(bytes))?;
    *best.previous.get_mut(slot)? = *old;
    *old = head;
    Some(())
}
