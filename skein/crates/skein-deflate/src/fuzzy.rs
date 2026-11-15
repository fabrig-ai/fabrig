//! Generated codec properties (skein/docs/design/data/deflate.md, 4).

use skein_lib::{List, Rng};

use crate::decode;
use crate::tests::{random, round_trip};

#[test]
fn generated_inputs_round_trip_within_their_bound() {
    for seed in 0..128 {
        let mut rng = Rng::new(seed);
        let length = usize::try_from(rng.between(0, 70_000)).unwrap();
        let mut input = random(&mut rng, length);
        // Mix arbitrary bytes with alphabets and runs, including block crossings.
        if seed % 3 != 0 {
            let alphabet = u8::try_from(rng.between(1, 16)).unwrap();
            for byte in &mut input {
                *byte %= alphabet;
            }
        }
        round_trip(&input);
    }
}

#[test]
fn generated_streams_decode_or_refuse_without_growing_the_output() {
    let mut accepted = 0_u32;
    let mut refused = 0_u32;
    for seed in 0..2048 {
        let mut rng = Rng::new(seed);
        let length = usize::try_from(rng.between(0, 96)).unwrap();
        let mut input = random(&mut rng, length);
        if seed % 4 == 0 {
            input = round_trip(&input);
            if rng.chance(500) {
                let at = usize::try_from(rng.below(skein_lib::len_u64(input.len()))).unwrap();
                input[at] ^= 1 << rng.below(8);
            }
        }
        let capacity = u32::try_from(rng.between(0, 96)).unwrap();
        let mut out = List::with_capacity(capacity);
        match decode(&input, &mut out) {
            Some(()) => accepted += 1,
            None => refused += 1,
        }
        assert!(out.len() <= capacity, "seed {seed}: output stays bounded");
        assert_eq!(out.capacity(), capacity, "seed {seed}: capacity never changes");
    }
    assert!(accepted > 0, "generated streams include successful decodes");
    assert!(refused > 0, "generated streams include refusals");
}
