//! The encoder's shorter paths keep the scalar parse and canonical fixed codes.
use super::*;

// The scalar head update: one absolute position and one hash per interior byte.
fn scalar_parse(encoder: &mut Encoder, body: &[u8], block: Block) -> Option<()> {
    let parse = &mut encoder.parse;
    parse.count = 0;
    parse.counts = Counts { literals: [0; SYMBOLS], distances: [0; DISTANCES], extra: 0 };
    let mut at = block.start;
    for _ in block.start..block.end {
        if at >= block.end {
            break;
        }
        let found = match word(body, at) {
            Some(bytes) => {
                let head = encoder.heads.get_mut(scalar_hash(bytes))?;
                let earlier = *head;
                *head = low(at);
                longest(body, at, block.end, earlier, bytes)
            }
            None => None,
        };
        let Some(found) = found else {
            tally(&mut parse.counts.literals, usize::from(*body.get(at)?))?;
            at = at.checked_add(1)?;
            continue;
        };
        record(parse, block, at, found)?;
        let next = at.checked_add(found.length)?;
        for inside in at.checked_add(1)?..next {
            let Some(bytes) = word(body, inside) else { break };
            *encoder.heads.get_mut(scalar_hash(bytes))? = low(inside);
        }
        at = next;
    }
    tally(&mut parse.counts.literals, END)
}

fn scalar_hash(word: u32) -> usize {
    usize::try_from(word.wrapping_mul(0x9E37_79B1) >> (32 - HASH)).unwrap()
}

#[test]
#[expect(clippy::disallowed_methods, reason = "the named block-crossing input repeats a finite byte fixture")]
fn windowed_heads_keep_the_scalar_parse_across_wrap_and_block_edges() {
    // Short tails, overlapping matches, and a second stored-size block whose
    // absolute positions wrap the sixteen-bit head counter.
    let crossing = b"abcde".repeat(BLOCK / 5 + LONGEST);
    for (name, body) in [("short tail", b"abc".as_slice()), ("overlap", b"aaaaaaaaabaaaaa"), ("crossing", &crossing)] {
        let mut windowed = EMPTY;
        let mut scalar = EMPTY;
        for index in 0..blocks(body.len()) {
            let start = index * BLOCK;
            let block = Block { start, end: (start + BLOCK).min(body.len()), last: index + 1 == blocks(body.len()) };
            assert_eq!(parse(&mut windowed, body, block), scalar_parse(&mut scalar, body, block), "{name}");
            assert_eq!(windowed.heads, scalar.heads, "{name}: every interior head");
            let a = &windowed.parse;
            let b = &scalar.parse;
            assert_eq!(a.count, b.count, "{name}");
            assert_eq!(&a.starts[..a.count], &b.starts[..b.count], "{name}");
            assert_eq!(&a.lengths[..a.count], &b.lengths[..b.count], "{name}");
            assert_eq!(&a.distances[..a.count], &b.distances[..b.count], "{name}");
            assert_eq!(a.counts.literals, b.counts.literals, "{name}");
            assert_eq!(a.counts.distances, b.counts.distances, "{name}");
            assert_eq!(a.counts.extra, b.counts.extra, "{name}");
        }
    }
}

#[test]
fn fixed_ranges_keep_the_canonical_codes_at_their_boundaries() {
    let direct = fixed().unwrap();
    let mut canonical_codes = Codes { literals: [NONE; SYMBOLS], distances: [Code { bits: 0, width: 5 }; DISTANCES] };
    for (symbol, code) in canonical_codes.literals.iter_mut().enumerate() {
        code.width = match symbol {
            144..=255 => 9,
            256..=279 => 7,
            _ => 8,
        };
    }
    canonical(&mut canonical_codes.literals).unwrap();
    canonical(&mut canonical_codes.distances).unwrap();
    // Adjacent boundaries distinguish the four literal ranges; distance codes
    // include the last usable distance, not the two reserved symbols.
    for symbol in [0, 143, 144, 255, 256, 279, 280, 287] {
        assert_eq!(direct.literals[symbol].bits, canonical_codes.literals[symbol].bits, "literal {symbol}");
        assert_eq!(direct.literals[symbol].width, canonical_codes.literals[symbol].width, "literal {symbol}");
    }
    for symbol in [0, 29] {
        assert_eq!(direct.distances[symbol].bits, canonical_codes.distances[symbol].bits, "distance {symbol}");
        assert_eq!(direct.distances[symbol].width, canonical_codes.distances[symbol].width, "distance {symbol}");
    }
}
