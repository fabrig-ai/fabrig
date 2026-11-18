//! Huffman prefixes consume only their code, never padded input (data/deflate.md, 3).
use super::{Code, Input, code};

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
