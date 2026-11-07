//! Two worlds in lockstep compare by digest at each iteration, and by text where they differ
//! and once at a seed's end (skein/docs/foundation/testing-strategy.md, 6; skein/docs/design/lib.md, 5.1).

use core::fmt::{Debug, Display};
use core::hash::{Hash, Hasher};

/// Panics, with both states' text, unless `first` and `replay` digest alike.
pub fn same_digest<T: Hash + Debug>(first: &T, replay: &T, at: &dyn Display) {
    if digest(first) != digest(replay) {
        let first_text = format!("{first:?}");
        let replay_text = format!("{replay:?}");
        let text =
            if first_text == replay_text { "equal text: Hash covers state Debug omits" } else { "different text" };
        panic!("state digests differ at {at} ({text})\nfirst: {first_text}\nreplay: {replay_text}");
    }
}

/// Panics unless both print alike, which holds what `Hash` leaves out by design.
pub fn same_text<T: Debug>(first: &T, replay: &T, at: &dyn Display) {
    assert_eq!(format!("{first:?}"), format!("{replay:?}"), "complete state at {at}");
}

fn digest<T: Hash>(state: &T) -> u64 {
    let mut hasher = WordHasher(0x517c_c1b7_2722_0a95);
    state.hash(&mut hasher);
    hasher.finish()
}

// These unkeyed digests are compared once in one process and never kept. They
// need no stability across runs or machines; complete text is checked at each
// seed's end. Mix whole words rather than hashing every byte separately.
struct WordHasher(u64);

const WORD_BYTES: usize = core::mem::size_of::<u64>();

impl Hasher for WordHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        let (words, rest) = bytes.as_chunks::<WORD_BYTES>();
        for word in words {
            self.write_u64(u64::from_ne_bytes(*word));
        }
        if !rest.is_empty() {
            let mut word = [0; WORD_BYTES];
            word[..rest.len()].copy_from_slice(rest);
            self.write_u64(u64::from_ne_bytes(word));
        }
        self.write_usize(bytes.len());
    }

    fn write_u8(&mut self, value: u8) {
        self.write_u64(u64::from(value));
    }

    fn write_u16(&mut self, value: u16) {
        self.write_u64(u64::from(value));
    }

    fn write_u32(&mut self, value: u32) {
        self.write_u64(u64::from(value));
    }

    fn write_u64(&mut self, value: u64) {
        self.0 = (self.0 ^ value).wrapping_mul(0x9e37_79b1_85eb_ca87).rotate_left(27);
    }

    fn write_usize(&mut self, value: usize) {
        self.write_u64(value as u64);
    }

    fn write_isize(&mut self, value: isize) {
        self.write_usize(value.cast_unsigned());
    }
}
