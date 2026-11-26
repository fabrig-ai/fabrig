//! Generated codec properties (skein/docs/design/data/deflate.md, 4; skein/docs/design/data/gzip.md, 3).

use skein_lib::{List, Rng};

use crate::tests::{random, round_trip, round_trip_best};
use crate::{decode, gzip};

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
fn best_generated_inputs_round_trip_within_their_bound() {
    // The chain does more work on small alphabets, so draw shorter inputs.
    for seed in 0..24 {
        let mut rng = Rng::new(seed);
        let length = usize::try_from(rng.between(2048, 12_000)).unwrap();
        let mut input = random(&mut rng, length);
        if seed % 3 != 0 {
            let alphabet = u8::try_from(rng.between(1, 16)).unwrap();
            for byte in &mut input {
                *byte %= alphabet;
            }
        }
        round_trip_best(&input);
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

#[test]
fn generated_gzip_members_round_trip_within_their_bound() {
    // Chain searches cost more on small alphabets; sample both kinds of input at best.
    const BEST_CASES: u64 = 24;
    for seed in 0..128 {
        let mut rng = Rng::new(seed);
        let length = usize::try_from(rng.between(0, 4096)).unwrap();
        let mut body = random(&mut rng, length);
        if seed % 2 == 0 {
            let alphabet = u8::try_from(rng.between(1, 16)).unwrap();
            for byte in &mut body {
                *byte %= alphabet;
            }
        }
        let mut room = skein_lib::bytes::zeroed(gzip::bound(length).unwrap());
        let written = gzip::encode(&body, &mut room).unwrap();
        check_gzip_round_trip(&body, &room, written, seed);
        if seed < BEST_CASES {
            let written = gzip::encode_best(&body, &mut room).unwrap();
            check_gzip_round_trip(&body, &room, written, seed);
        }
    }
}

fn check_gzip_round_trip(body: &[u8], room: &[u8], written: usize, seed: u64) {
    assert!(written <= room.len(), "seed {seed}: member stays within its bound");
    let mut out = List::allocated(u32::try_from(body.len()).unwrap());
    assert_eq!(gzip::decode(&room[..written], &mut out), Some(()), "seed {seed}");
    assert_eq!(out.as_slice(), body, "seed {seed}: input read back");
}

#[test]
#[expect(clippy::disallowed_methods, reason = "a generated member grows or shrinks by one byte for the decoder")]
fn changed_gzip_members_decode_to_the_input_or_are_refused() {
    let mut accepted = 0_u32;
    let mut refused = 0_u32;
    for seed in 0..2048 {
        let mut rng = Rng::new(seed);
        let length = u32::try_from(rng.between(0, 96)).unwrap();
        let body = random(&mut rng, skein_lib::usize_of(length));
        let mut room = skein_lib::bytes::zeroed(gzip::bound(body.len()).unwrap());
        let written = gzip::encode(&body, &mut room).unwrap();
        let mut member = room[..written].to_vec();
        let at = usize::try_from(rng.below(skein_lib::len_u64(written))).unwrap();
        match seed % 3 {
            0 => member[at] ^= 1 << rng.below(8),
            1 => {
                let _removed = member.remove(at);
            }
            _ => member.insert(at, rng.next_u64().to_le_bytes()[0]),
        }
        let capacity = if seed % 2 == 0 { length } else { u32::try_from(rng.between(0, 96)).unwrap() };
        let mut out = List::allocated(capacity);
        match gzip::decode(&member, &mut out) {
            Some(()) => {
                assert_eq!(out.as_slice(), body, "seed {seed}: accepted only the input");
                accepted += 1;
            }
            None => refused += 1,
        }
        assert!(out.len() <= capacity, "seed {seed}: output stays bounded");
        assert_eq!(out.capacity(), capacity, "seed {seed}: capacity never changes");
    }
    assert!(accepted > 0, "generated changes include equivalent members");
    assert!(refused > 0, "generated changes include refusals");
}
