//! Raw DEFLATE's bounds, deterministic bytes and refusals (data/deflate.md, 4).
#![expect(clippy::disallowed_types, reason = "finite test inputs and output rooms use vectors")]
#![expect(clippy::disallowed_macros, reason = "finite test buffers use vec!")]
#![expect(clippy::disallowed_methods, reason = "finite fixture vectors repeat and edit byte slices")]

use skein_lib::{List, Rng};

use crate::{bound, decode, encode};

mod golden;
mod huffman;

const BLOCK: usize = 65_535;
const STORED: &[u8] = b"\x01\x04\x00\xfb\xffa\nb\n";
const FIXED: &[u8] = &[0x4b, 0xe4, 0x4a, 0xe2, 0x02, 0x00];
const DYNAMIC: &[u8] = &[237, 193, 1, 13, 0, 0, 0, 194, 160, 218, 143, 111, 15, 7, 20, 0, 0, 0, 240, 110];

pub(crate) fn random(rng: &mut Rng, length: usize) -> Vec<u8> {
    let mut input = vec![0; length];
    for byte in &mut input {
        *byte = rng.next_u64().to_le_bytes()[0];
    }
    input
}

pub(crate) fn round_trip(input: &[u8]) -> Vec<u8> {
    let mut out = vec![0; bound(input.len()).unwrap()];
    let n = encode(input, &mut out).unwrap();
    assert!(n <= bound(input.len()).unwrap(), "output stays within bound");
    let mut back = List::with_capacity(u32::try_from(input.len()).unwrap());
    assert_eq!(decode(&out[..n], &mut back), Some(()));
    assert_eq!(back.as_slice(), input, "the input read back");
    out.truncate(n);
    out
}

#[test]
fn the_bound_counts_stored_blocks_and_refuses_overflow() {
    assert_eq!(bound(0), Some(5), "one block at least");
    assert_eq!(bound(BLOCK), Some(BLOCK + 5));
    assert_eq!(bound(BLOCK + 1), Some(BLOCK + 1 + 10), "a second block");
    assert_eq!(bound(usize::MAX), None, "the stored overhead cannot fit");
}

#[test]
fn inputs_round_trip_within_the_bound() {
    let mut rng = Rng::new(3);
    let repeated = golden::BODY.repeat(500);
    let short = round_trip(b"abc");
    assert!(!short.is_empty(), "a short input has a stream");
    assert_eq!(round_trip(b""), [3, 0], "the empty gzip member's fixed block is preserved");
    assert!(round_trip(&repeated).len() * 4 < repeated.len(), "repeated JSON compresses well");
    round_trip(&random(&mut rng, 4096));
    // Incompressible data crosses the stored block boundary too.
    round_trip(&random(&mut rng, BLOCK + 1000));
}

#[test]
fn encoding_keeps_the_golden_bytes_whatever_the_room_held() {
    assert_eq!(round_trip(golden::BODY), golden::BYTES);
    let mut out = vec![0xa5; bound(golden::BODY.len()).unwrap() + 100];
    let n = encode(golden::BODY, &mut out).unwrap();
    assert_eq!(&out[..n], golden::BYTES);
}

#[test]
fn trailing_bytes_and_truncated_streams_are_refused() {
    for data in [STORED, FIXED, DYNAMIC] {
        let mut out = List::with_capacity(4096);
        assert_eq!(decode(data, &mut out), Some(()), "the complete stream");
        let mut trailing = data.to_vec();
        trailing.push(0);
        assert_eq!(decode(&trailing, &mut out), None, "a trailing byte");
        assert_eq!(decode(&data[..data.len() - 1], &mut out), None, "a truncated stream");
    }
    assert_eq!(decode(b"", &mut List::with_capacity(0)), None, "no final block");
}

#[test]
fn malformed_codes_and_distances_before_the_start_are_refused() {
    for data in [
        b"\x07".as_slice(), // Reserved block type.
        b"\x01\x04\x00\xfa\xffa\nb\n", // Wrong stored length complement.
        b"\x1b\x03", // Fixed literal/length symbol 286 is reserved.
        b"\x03\x02\x00", // A length of three at distance one, with no output yet.
    ] {
        let mut out = List::with_capacity(16);
        assert_eq!(decode(data, &mut out), None, "malformed stream: {data:?}");
    }
    let mut out = List::with_capacity(4);
    assert_eq!(decode(b"\x4b\x04\x02\x00", &mut out), Some(()), "distance one after a literal");
    assert_eq!(out.as_slice(), b"aaaa", "overlapping copies are allowed");
}

#[test]
fn decoding_clears_the_list_and_never_passes_its_capacity() {
    for (data, capacity, expected) in [(STORED, 4, b"a\nb\n".as_slice()), (FIXED, 4, b"a\nb\n"), (DYNAMIC, 4096, &[b'x'; 4096])] {
        let mut out = List::with_capacity(capacity);
        out.push(0xff).unwrap();
        assert_eq!(decode(data, &mut out), Some(()), "fits exactly after clearing");
        assert_eq!(out.as_slice(), expected);
        let mut short = List::with_capacity(capacity - 1);
        assert_eq!(decode(data, &mut short), None, "one byte past capacity");
        assert_eq!(short.as_slice(), &expected[..expected.len() - 1], "the inflated prefix is kept");
        assert_eq!(short.capacity(), capacity - 1, "capacity never grows");
    }
    let mut out = List::with_capacity(1);
    out.push(0xff).unwrap();
    assert_eq!(decode(b"", &mut out), None);
    assert!(out.is_empty(), "even a refusal clears the previous contents");
}
