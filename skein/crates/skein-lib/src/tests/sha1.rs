use crate::sha1::{self, Sha1};

fn hex(text: &str) -> [u8; 20] {
    let mut digest = [0; 20];
    for (byte, at) in digest.iter_mut().zip((0_usize..40).step_by(2)) {
        *byte = u8::from_str_radix(&text[at..][..2], 16).expect("two hex digits");
    }
    digest
}

#[test]
fn fips_180_4_examples_digest_to_their_published_values() {
    for (message, expected) in [
        (b"".as_slice(), "da39a3ee5e6b4b0d3255bfef95601890afd80709"),
        (b"abc", "a9993e364706816aba3e25717850c26c9cd0d89d"),
        (b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq", "84983e441c3bd26ebaae4aa1f95129e5e54670f1"),
    ] {
        assert_eq!(sha1::digest(message), hex(expected), "{expected}");
    }
}

#[test]
fn pieces_and_clones_at_block_and_padding_edges_digest_alike() {
    let message = [b'a'; 129];
    for length in [0, 1, 55, 56, 63, 64, 65, 127, 128, 129] {
        let message = &message[..length];
        let expected = sha1::digest(message);
        for cut in 0..=length {
            let mut state = Sha1::new();
            state.update(&message[..cut]);
            state.update(b"");
            let mut clone = state.clone();
            assert_eq!(clone.clone().finish(), sha1::digest(&message[..cut]));
            state.update(&message[cut..]);
            assert_eq!(state.finish(), expected, "{length} bytes, cut at {cut}");
            for byte in &message[cut..] {
                clone.update(&[*byte]);
            }
            assert_eq!(clone.finish(), expected, "{length} bytes, cloned at {cut}, then single bytes");
        }
    }
}

#[test]
fn debug_shows_the_count_hashed_and_not_the_state() {
    let mut state = Sha1::new();
    assert_eq!(alloc::fmt::format(format_args!("{state:?}")), "Sha1 { hashed: 0 }");
    state.update(b"ab");
    state.update(b"c");
    assert_eq!(alloc::fmt::format(format_args!("{state:?}")), "Sha1 { hashed: 3 }");
}
