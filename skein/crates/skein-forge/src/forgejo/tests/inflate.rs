//! Raw DEFLATE fixtures from Python zlib cover all three block types.
#![expect(clippy::disallowed_macros, reason = "the dynamic DEFLATE fixture builds its expected byte vector")]
#![expect(clippy::disallowed_methods, reason = "malformed fixtures edit copies of finite byte streams")]
use crate::forgejo::inflate;
use skein_lib::List;

// The dynamic fixture expands to 4096 repeated x bytes.
const DYNAMIC_BYTES: u32 = 4096;
const DYNAMIC: &[u8] = &[237, 193, 1, 13, 0, 0, 0, 194, 160, 218, 143, 111, 15, 7, 20, 0, 0, 0, 240, 110];
const FIXED: &[u8] = &[0x4b, 0xe4, 0x4a, 0xe2, 0x02, 0x00];
const STORED: &[u8] = b"\x01\x04\x00\xfb\xffa\nb\n";
const FILE: &[u8] = b"a\nb\n";
const FILE_BYTES: u32 = 4;

#[test]
fn all_block_types_inflate_at_the_output_bound() {
    let repeated = vec![b'x'; usize::try_from(DYNAMIC_BYTES).expect("fixture size")];
    for (data, bound, expected) in
        [(STORED, FILE_BYTES, FILE), (FIXED, FILE_BYTES, FILE), (DYNAMIC, DYNAMIC_BYTES, repeated.as_slice())]
    {
        let mut out = List::with_capacity(bound);
        assert_eq!(inflate::deflate(data, &mut out), Some(()));
        assert_eq!(out.as_slice(), expected);
    }
}

#[test]
fn malformed_streams_and_output_past_the_bound_are_refused() {
    let mut trailing = FIXED.to_vec();
    trailing.push(0);
    let mut stored = STORED.to_vec();
    // The stored block's complemented length must match its length.
    *stored.get_mut(3).expect("length complement") ^= 1;
    for (data, bound) in [
        (STORED, FILE_BYTES.saturating_sub(1)),
        (FIXED, FILE_BYTES.saturating_sub(1)),
        (DYNAMIC, DYNAMIC_BYTES.saturating_sub(1)),
        (b"".as_slice(), FILE_BYTES),
        (b"\x07".as_slice(), FILE_BYTES),
        (&FIXED[..FIXED.len().saturating_sub(1)], FILE_BYTES),
        (trailing.as_slice(), FILE_BYTES),
        (stored.as_slice(), FILE_BYTES),
    ] {
        let mut out = List::with_capacity(bound);
        assert_eq!(inflate::deflate(data, &mut out), None, "{data:?}, bound={bound}");
    }
}
