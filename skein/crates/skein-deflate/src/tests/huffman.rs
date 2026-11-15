//! The encoder's private Huffman builder, compared with a reference decoder.
#![expect(clippy::disallowed_types, reason = "the reference decoder and Huffman tree use finite vectors")]
#![expect(clippy::arithmetic_side_effects, reason = "the reference decoder and tree use ordinary test arithmetic")]

use crate::encoder as gzip;
use skein_lib::Rng;

/// A member's DEFLATE data read bit by bit, each byte's lowest bit first
/// (RFC 1951, 3.1.1).
struct Input<'a> {
    bytes: &'a [u8],
    bit: usize,
}

impl Input<'_> {
    /// `width` bits, the first lowest.
    fn bits(&mut self, width: u32) -> usize {
        let mut value = 0;
        for index in 0..width {
            let byte = self.bytes[self.bit / 8];
            value |= usize::from((byte >> (self.bit % 8)) & 1) << index;
            self.bit += 1;
        }
        value
    }

}

/// A canonical Huffman code (RFC 1951, 3.2.2), for decoding: how many codes
/// there are of each width, and the symbols in the codes' order.
struct Code {
    counts: [usize; 16],
    symbols: Vec<usize>,
}

impl Code {
    /// The code whose symbols have these widths, 0 for a symbol unused,
    /// asserted complete: every string of 15 bits starts one code.
    fn new(widths: &[usize]) -> Code {
        let mut counts = [0_usize; 16];
        let mut symbols = Vec::new();
        let mut filled = 0_usize;
        for (width, count) in counts.iter_mut().enumerate().skip(1) {
            for (symbol, each) in widths.iter().enumerate() {
                if *each == width {
                    *count += 1;
                    symbols.push(symbol);
                    filled += 1 << (15 - width);
                }
            }
        }
        for width in widths {
            assert!(*width <= 15, "no code is wider than 15 bits: {widths:?}");
        }
        assert_eq!(filled, 1 << 15_u32, "a complete code: {widths:?}");
        Code { counts, symbols }
    }

    /// The next symbol, its code read one bit at a time, most significant
    /// first.
    fn decode(&self, input: &mut Input<'_>) -> usize {
        let mut value = 0;
        let mut first = 0;
        let mut index = 0;
        for count in &self.counts[1..] {
            value |= input.bits(1);
            if value < first + count {
                return self.symbols[index + value - first];
            }
            index += count;
            first = (first + count) << 1_u32;
            value <<= 1_u32;
        }
        panic!("no code is longer than 15 bits");
    }
}

/// The code the encoder builds for `counts`, no wider than `widest`, each
/// symbol's width asserted: the code complete, each symbol's bits read
/// back to it, and a commoner symbol never given a wider code.
fn huffman(counts: &[u32], widest: u32) -> Vec<usize> {
    let mut codes = Vec::new();
    codes.resize(counts.len(), gzip::Code { bits: 0, width: 0 });
    gzip::huffman(counts, widest, &mut codes).unwrap();
    let mut widths = Vec::new();
    for each in &codes {
        widths.push(usize::try_from(each.width).unwrap());
    }
    let code = Code::new(&widths);
    for (symbol, each) in codes.iter().enumerate() {
        if each.width > 0 {
            let mut bytes = [0_u8; 2];
            for bit in 0..each.width {
                let at = usize::try_from(bit).unwrap();
                bytes[at / 8] |= u8::try_from((each.bits >> bit) & 1).unwrap() << (at % 8);
            }
            assert_eq!(code.decode(&mut Input { bytes: &bytes, bit: 0 }), symbol, "symbol {symbol}'s bits");
        }
    }
    for (one, width) in widths.iter().enumerate() {
        assert!(*width <= usize::try_from(widest).unwrap(), "symbol {one} is {width} bits wide");
        for (other, than) in widths.iter().enumerate() {
            if counts[one] > counts[other] && counts[other] > 0 {
                assert!(width <= than, "symbol {one} is commoner than {other} and wider");
            }
        }
    }
    widths
}

/// The fewest bits `counts` take in any prefix code: the weight of every
/// node an unbounded Huffman tree is built of.
fn least(counts: &[u32]) -> u64 {
    let mut weights = Vec::new();
    for count in counts {
        if *count > 0 {
            weights.push(u64::from(*count));
        }
    }
    let mut total = 0;
    while weights.len() > 1 {
        weights.sort_unstable();
        let node = weights.remove(0) + weights.remove(0);
        total += node;
        weights.push(node);
    }
    total
}

/// The bits `counts` take in codes of `widths`.
fn weight(counts: &[u32], widths: &[usize]) -> u64 {
    let mut total = 0;
    for (count, width) in counts.iter().zip(widths) {
        total += u64::from(*count) * u64::try_from(*width).unwrap();
    }
    total
}

#[test]
fn a_huffman_code_is_complete_canonical_and_as_short_as_any() {
    // Counts no more than five times apart build no tree deeper than 15:
    // the code is the tree's. Counts from 1 may: the code is no shorter.
    for (seed, rarest) in (0..4).zip([1000, 1].into_iter().cycle()) {
        let mut rng = Rng::new(seed);
        let mut counts = [0_u32; 288];
        for count in &mut counts {
            if rng.chance(700) {
                *count = u32::try_from(rng.between(rarest, 5000)).unwrap();
            }
        }
        let widths = huffman(&counts, 15);
        if rarest == 1 {
            assert!(weight(&counts, &widths) >= least(&counts), "seed {seed}: no shorter than the tree");
        } else {
            assert_eq!(weight(&counts, &widths), least(&counts), "seed {seed}: the counts in the fewest bits");
        }
    }
}

#[test]
fn a_code_deeper_than_its_widest_is_cut_to_it() {
    // Counts that grow as Fibonacci's numbers build a tree as deep as there
    // are symbols.
    let mut counts = [0_u32; 30];
    let (mut one, mut two) = (1, 1);
    for count in &mut counts {
        *count = one;
        (one, two) = (two, one + two);
    }
    let widths = huffman(&counts, 15);
    assert_eq!(widths.iter().max(), Some(&15), "cut to 15 bits");
    assert!(weight(&counts, &widths) > least(&counts), "the cut code is longer than the tree");
    let widths = huffman(&counts[..19], 7);
    assert_eq!(widths.iter().max(), Some(&7), "a width code cut to 7 bits");
    assert_eq!(huffman(&[1; 19], 7).iter().max(), Some(&5), "19 alike: four and five bits");
}

#[test]
fn a_code_of_one_symbol_or_none_has_two_of_one_bit() {
    let mut counts = [0_u32; 30];
    assert_eq!(huffman(&counts, 15)[..3], [1, 1, 0], "none: 0 and 1");
    counts[0] = 7;
    assert_eq!(huffman(&counts, 15)[..3], [1, 1, 0], "0 alone: 0 and 1");
    counts[0] = 0;
    counts[1] = 7;
    assert_eq!(huffman(&counts, 15)[..3], [1, 1, 0], "1 alone: 0 and 1");
    counts[1] = 0;
    counts[29] = 7;
    let widths = huffman(&counts, 15);
    assert_eq!((widths[0], widths[1], widths[29]), (1, 0, 1), "29 alone: 0 and 29");
}

#[test]
fn a_code_past_its_limits_is_refused() {
    let mut codes = [gzip::Code { bits: 0, width: 0 }; 30];
    assert_eq!(gzip::huffman(&[1; 30], 0, &mut codes), None, "no width");
    assert_eq!(gzip::huffman(&[1; 30], 16, &mut codes), None, "wider than DEFLATE's 15");
    assert_eq!(gzip::huffman(&[1; 29], 15, &mut codes), None, "a code for each count");
    assert_eq!(gzip::huffman(&[1; 30], 4, &mut codes), None, "30 symbols in four bits");
}

