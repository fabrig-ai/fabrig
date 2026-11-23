//! Streaming SHA-1 over bounded pieces (runtime/lib-bytes.md, 1.3).

/// SHA-1 (FIPS 180-4) over pieces cut anywhere (runtime/lib-bytes.md, 1.3).
#[derive(Clone)]
pub struct Sha1 {
    state: [u32; 5],
    block: [u8; 64],
    hashed: u64,
}

impl Sha1 {
    /// A hasher that has hashed nothing.
    #[expect(clippy::new_without_default, reason = "step code names no Default (programming-model.md, 10.2)")]
    #[must_use]
    pub const fn new() -> Sha1 {
        Sha1 { state: INITIAL, block: [0; 64], hashed: 0 }
    }

    /// Hashes the next piece; asserts the standard's bound, 2^64 - 1 bits.
    #[expect(clippy::disallowed_methods, reason = "bulk copies only the incomplete block's bounded pieces")]
    pub fn update(&mut self, mut piece: &[u8]) {
        let used = crate::usize_of(u32::try_from(self.hashed % 64).expect("block position below 64"));
        self.hashed = self.hashed.checked_add(crate::len_u64(piece.len())).expect("SHA-1 hashes at most 2^64 - 1 bits");
        assert!(self.hashed <= u64::MAX / 8, "SHA-1 hashes at most 2^64 - 1 bits");
        if used != 0 {
            let missing = 64_usize.saturating_sub(used);
            let count = missing.min(piece.len());
            let (front, rest) = piece.split_at_checked(count).expect("carried bytes fit piece");
            let tail = self.block.get_mut(used..).expect("block position fits");
            let (target, _) = tail.split_at_mut_checked(count).expect("carried bytes fit block");
            target.copy_from_slice(front);
            if count < missing {
                return;
            }
            compress(&mut self.state, &self.block);
            piece = rest;
        }
        let (blocks, rest) = piece.as_chunks::<64>();
        for block in blocks {
            compress(&mut self.state, block);
        }
        self.block.get_mut(..rest.len()).expect("partial block fits").copy_from_slice(rest);
    }

    /// Pads and ends the digest: its 20 bytes in the standard's order.
    #[expect(clippy::disallowed_methods, reason = "bulk copies the fixed eight-byte bit count into its block")]
    #[must_use]
    pub fn finish(mut self) -> [u8; 20] {
        let used = crate::usize_of(u32::try_from(self.hashed % 64).expect("block position below 64"));
        self.block.get_mut(used..).expect("block position fits").fill(0);
        *self.block.get_mut(used).expect("block position below 64") = 0x80;
        if used >= 56 {
            compress(&mut self.state, &self.block);
            self.block = [0; 64];
        }
        let bits = self.hashed.checked_mul(8).expect("SHA-1 bit count fits u64").to_be_bytes();
        self.block.get_mut(56..).expect("length occupies last eight bytes").copy_from_slice(&bits);
        compress(&mut self.state, &self.block);
        let mut digest = [0; 20];
        for (bytes, word) in digest.as_chunks_mut::<4>().0.iter_mut().zip(self.state) {
            *bytes = word.to_be_bytes();
        }
        digest
    }
}

#[expect(
    clippy::missing_fields_in_debug,
    reason = "runtime/lib-bytes.md, 1.3: the count of bytes hashed, not the state"
)]
impl core::fmt::Debug for Sha1 {
    /// `Sha1 { hashed: N }`.
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Sha1").field("hashed", &self.hashed).finish()
    }
}

/// The one-shot form: one message's digest.
#[must_use]
pub fn digest(message: &[u8]) -> [u8; 20] {
    let mut state = Sha1::new();
    state.update(message);
    state.finish()
}

const INITIAL: [u32; 5] = [0x6745_2301, 0xefcd_ab89, 0x98ba_dcfe, 0x1032_5476, 0xc3d2_e1f0];

#[expect(clippy::many_single_char_names, reason = "SHA-1 names its five working words a through e")]
fn compress(state: &mut [u32; 5], block: &[u8; 64]) {
    let mut words = [0_u32; 80];
    for (word, bytes) in words.iter_mut().zip(block.as_chunks::<4>().0) {
        *word = u32::from_be_bytes(*bytes);
    }
    for index in 16_usize..80 {
        let word = *words.get(index.saturating_sub(3)).expect("schedule index")
            ^ *words.get(index.saturating_sub(8)).expect("schedule index")
            ^ *words.get(index.saturating_sub(14)).expect("schedule index")
            ^ *words.get(index.saturating_sub(16)).expect("schedule index");
        *words.get_mut(index).expect("schedule index") = word.rotate_left(1);
    }
    let [mut a, mut b, mut c, mut d, mut e] = *state;
    for (index, word) in words.into_iter().enumerate() {
        let (function, constant) = if index < 20 {
            ((b & c) | ((!b) & d), 0x5a82_7999)
        } else if index < 40 {
            (b ^ c ^ d, 0x6ed9_eba1)
        } else if index < 60 {
            ((b & c) | (b & d) | (c & d), 0x8f1b_bcdc)
        } else {
            (b ^ c ^ d, 0xca62_c1d6)
        };
        let next = a.rotate_left(5).wrapping_add(function).wrapping_add(e).wrapping_add(constant).wrapping_add(word);
        e = d;
        d = c;
        c = b.rotate_left(30);
        b = a;
        a = next;
    }
    for (value, word) in state.iter_mut().zip([a, b, c, d, e]) {
        *value = value.wrapping_add(word);
    }
}
