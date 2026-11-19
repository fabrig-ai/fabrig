//! Huffman prefixes consume only their code, never padded input (data/deflate.md, 3).
use super::{Code, Input, canonical, code, repeat};
use skein_lib::List;

#[test]
fn prefix_lookup_keeps_the_canonical_search_and_input_boundary() {
    // Short prefixes, the nine-bit lookup boundary, its long-code fallback,
    // and an incomplete code's unused prefix.
    for (widths, bits, bytes, offset) in [
        ([1, 9, 9], 0_u32, 1, 7),
        ([1, 9, 9], 257, 2, 0),
        ([1, 15, 15], 16_385, 3, 7),
        ([1, 9, 9], 1, 1, 0),
        ([1, 9, 9], 3, 2, 0),
    ] {
        let code = code(&widths).unwrap();
        let data = (bits << offset).to_le_bytes();
        let data = &data[..bytes];
        let mut input = Input { bit: offset };
        let (peek, available) = input.peek(data).unwrap();
        let Code::Canonical { counts, symbols, shortest, .. } = &code else { panic!("canonical fixture") };
        let (symbol, consumed) = match canonical(peek.reverse_bits(), counts, symbols, *shortest) {
            Some((symbol, width)) if width <= available => (Some(symbol), usize::try_from(width).unwrap()),
            Some(_) | None => (None, 0),
        };
        assert_eq!(code.decode(&mut input, data), symbol);
        assert_eq!(input.bit, offset + consumed);
    }
}

#[test]
fn sliced_back_references_keep_the_scalar_prefix_even_when_full() {
    // A non-overlap, an overlap, and the maximum match all keep the scalar
    // copy's prefix on either side of the output capacity.
    const MAX_MATCH: u32 = 258; // RFC 1951, 3.2.5.
    const SOURCE: &[u8] = b"abcd";
    const SOURCE_BYTES: u32 = 4; // SOURCE's four bytes precede each back-reference.
    for (distance, length) in [(4, 3), (3, 8), (1, MAX_MATCH)] {
        for room in [length, length - 1] {
            let mut sliced = List::with_capacity(SOURCE_BYTES + room);
            for byte in SOURCE {
                sliced.push(*byte).unwrap();
            }
            let mut scalar = sliced.clone();
            let from = scalar.len() - distance;
            let mut expected = Some(());
            for from in from..from + length {
                let byte = *scalar.get(from).unwrap();
                if scalar.push(byte).is_err() {
                    expected = None;
                    break;
                }
            }
            assert_eq!(repeat(&mut sliced, distance, length), expected);
            assert_eq!(sliced, scalar, "distance {distance}, length {length}, room {room}");
        }
    }
}

#[test]
fn fixed_codes_keep_their_ranges_and_reserved_distances() {
    // RFC 1951, 3.2.6: the boundaries of all four literal/length ranges.
    for (bits, width, symbol) in [
        (12_u16, 8, 0),
        (253, 8, 143),
        (19, 9, 144),
        (511, 9, 255),
        (0, 7, 256),
        (116, 7, 279),
        (3, 8, 280),
        (227, 8, 287),
    ] {
        let mut input = Input { bit: 0 };
        assert_eq!(Code::FixedLiterals.decode(&mut input, &bits.to_le_bytes()), Some(symbol));
        assert_eq!(input.bit, width);
    }
    // RFC 1951, 3.2.5: thirty distance symbols; the last two codes are reserved.
    for (bits, expected) in [(0_u8, Some(0)), (23, Some(29)), (15, None), (31, None)] {
        let mut input = Input { bit: 0 };
        assert_eq!(Code::FixedDistances.decode(&mut input, &[bits]), expected);
    }
    let mut short = Input { bit: 0 };
    assert_eq!(Code::FixedLiterals.decode(&mut short, &[19]), None, "nine-bit literal needs its last bit");
}

#[test]
fn short_and_long_codes_keep_their_bit_boundaries() {
    // An incomplete canonical code: 0, 100000000, 100000001.
    let code = code(&[1, 9, 9]).expect("valid incomplete code");
    for offset in 0..8 {
        for (bits, width, symbol) in [(0_u32, 1, 0), (1, 9, 1), (257, 9, 2)] {
            let bytes = (bits << offset).to_le_bytes();
            let mut input = Input { bit: offset };
            assert_eq!(code.decode(&mut input, &bytes), Some(symbol));
            assert_eq!(input.bit, offset + width, "only the selected code is consumed");
        }
    }
    let mut last = Input { bit: 7 };
    assert_eq!(code.decode(&mut last, &[0]), Some(0), "one remaining bit is a complete short code");
    assert_eq!(last.bit, 8);
    assert_eq!(code.decode(&mut last, &[0]), None, "padding is not another short code");
    let mut short = Input { bit: 0 };
    assert_eq!(code.decode(&mut short, &[1]), None, "eight bits cannot complete a nine-bit code");
    let mut unused = Input { bit: 0 };
    assert_eq!(code.decode(&mut unused, &[3, 0]), None, "an unused prefix is still refused");
}
