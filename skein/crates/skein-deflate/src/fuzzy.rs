//! Generated codec properties (skein/docs/design/data/deflate.md, 4; skein/docs/design/data/gzip.md, 3).
//! Each case takes its seed from the sweep seam (skein/docs/design/testing/sweeps.md, 4.2–4.5).
#![expect(clippy::disallowed_macros, reason = "this_test! takes Cargo's package name for case identities")]

use core::convert::Infallible;

use skein_lib::{List, Rng};
use skein_world::sweep::{Case, cases, gate};
use skein_world::this_test;

use crate::tests::{random, round_trip, round_trip_best};
use crate::{decode, gzip};

#[test]
fn generated_inputs_round_trip_within_their_bound() -> Result<(), Case<Infallible>> {
    let test = this_test!("generated_inputs_round_trip_within_their_bound");
    cases(test, 0..128, |seed| {
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
        Ok(())
    })
}

#[test]
fn best_generated_inputs_round_trip_within_their_bound() -> Result<(), Case<Infallible>> {
    let test = this_test!("best_generated_inputs_round_trip_within_their_bound");
    // The chain does more work on small alphabets, so draw shorter inputs.
    cases(test, 0..24, |seed| {
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
        Ok(())
    })
}

#[test]
fn generated_streams_decode_or_refuse_without_growing_the_output() -> Result<(), Case<Infallible>> {
    let test = this_test!("generated_streams_decode_or_refuse_without_growing_the_output");
    let at_gate = gate();
    let mut accepted = 0_u32;
    let mut refused = 0_u32;
    cases(test, 0..2048, |seed| {
        let mut rng = Rng::new(seed);
        let length = usize::try_from(rng.between(0, 96)).unwrap();
        let mut input = random(&mut rng, length);
        if seed % 4 == 0 {
            input = round_trip(&input);
            if rng.chance(500) {
                // Even an empty input encodes to a nonempty stream.
                let at = usize::try_from(rng.below(skein_lib::len_u64(input.len()))).unwrap();
                input[at] ^= 1 << rng.below(8);
            }
        }
        let capacity = u32::try_from(rng.between(0, 96)).unwrap();
        let mut out = List::with_capacity(capacity);
        let outcome = decode(&input, &mut out);
        assert!(out.len() <= capacity, "seed {seed}: output stays bounded");
        assert_eq!(out.capacity(), capacity, "seed {seed}: capacity never changes");
        if at_gate {
            match outcome {
                Some(()) => accepted += 1,
                None => refused += 1,
            }
        }
        Ok(())
    })?;
    if at_gate {
        assert!(accepted > 0, "generated streams include successful decodes");
        assert!(refused > 0, "generated streams include refusals");
    }
    Ok(())
}

const GZIP_CASES: u64 = 128;
const BEST_CASES: u64 = 24;

// Sample best gzip across the full seed range at the gate's rate.
fn best_gzip(seed: u64) -> bool {
    seed % GZIP_CASES < BEST_CASES
}

#[test]
fn best_gzip_sampling_keeps_the_gate_cutoff_and_uses_full_width_seeds() {
    assert!(best_gzip(23), "the last best gzip gate seed");
    assert!(!best_gzip(24), "the first fast-only gzip gate seed");
    assert!(best_gzip(128 + 5), "a seed beyond the gate can select best gzip");
    assert!(!best_gzip(u64::MAX), "the largest seed selects only fast gzip");
}

#[test]
fn generated_gzip_members_round_trip_within_their_bound() -> Result<(), Case<Infallible>> {
    let test = this_test!("generated_gzip_members_round_trip_within_their_bound");
    cases(test, 0..GZIP_CASES, |seed| {
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
        if best_gzip(seed) {
            let written = gzip::encode_best(&body, &mut room).unwrap();
            check_gzip_round_trip(&body, &room, written, seed);
        }
        Ok(())
    })
}

fn check_gzip_round_trip(body: &[u8], room: &[u8], written: usize, seed: u64) {
    assert!(written <= room.len(), "seed {seed}: member stays within its bound");
    let mut out = List::allocated(u32::try_from(body.len()).unwrap());
    assert_eq!(gzip::decode(&room[..written], &mut out), Some(()), "seed {seed}");
    assert_eq!(out.as_slice(), body, "seed {seed}: input read back");
}

#[test]
#[expect(clippy::disallowed_methods, reason = "a generated member grows or shrinks by one byte for the decoder")]
fn changed_gzip_members_decode_to_the_input_or_are_refused() -> Result<(), Case<Infallible>> {
    let test = this_test!("changed_gzip_members_decode_to_the_input_or_are_refused");
    let at_gate = gate();
    let mut accepted = 0_u32;
    let mut refused = 0_u32;
    cases(test, 0..2048, |seed| {
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
        let outcome = gzip::decode(&member, &mut out);
        if outcome.is_some() {
            assert_eq!(out.as_slice(), body, "seed {seed}: accepted only the input");
        }
        assert!(out.len() <= capacity, "seed {seed}: output stays bounded");
        assert_eq!(out.capacity(), capacity, "seed {seed}: capacity never changes");
        if at_gate {
            match outcome {
                Some(()) => accepted += 1,
                None => refused += 1,
            }
        }
        Ok(())
    })?;
    if at_gate {
        assert!(accepted > 0, "generated changes include equivalent members");
        assert!(refused > 0, "generated changes include refusals");
    }
    Ok(())
}
