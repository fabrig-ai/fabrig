//! Seeded arbitrary and mutated kept records (codec.md, 6).

use skein_lib::{Rng, Wall};
use skein_oauth::{SavedToken, decode_record, encode_record};
use skein_oauth_world::{boxed, documents};

// Enough to reach every field of the fixture and exercise refused lengths.
const INPUT_BYTES: usize = 512;
const ARBITRARY_ROUNDS: u32 = 10_000;
const MUTATION_ROUNDS: u32 = 1_000;
const SEED: u64 = 0x0A07_0003_C0DE_0001;

fn check(input: &[u8]) {
    if let Ok(record) = decode_record(input, &documents()) {
        assert_eq!(encode_record(&record, &documents()).expect("decoded record encodes").as_ref(), input);
    }
}

fn full_record() -> SavedToken {
    SavedToken {
        key: 1,
        generation: 2,
        expires_at: Wall::from_nanos(3),
        access_token: boxed(b"access"),
        refresh_token: Some(boxed(b"refresh")),
        metadata: Some(boxed(b"account")),
        issued_client_id: Some(boxed(b"client")),
        granted_scopes: Some(boxed(b"read")),
        subject: Some(boxed(b"subject")),
        id_token: Some(boxed(b"header.payload.signature")),
        earliest_refresh_at: Some(Wall::from_nanos(1)),
    }
}

#[test]
fn arbitrary_and_mutated_records_round_trip_when_they_decode() {
    let mut rng = Rng::new(SEED);
    let mut input = [0; INPUT_BYTES];
    for _ in 0..ARBITRARY_ROUNDS {
        let length = usize::try_from(
            rng.below(u64::try_from(INPUT_BYTES).expect("input size").checked_add(1).expect("size fits")),
        )
        .expect("bounded length");
        let input = input.get_mut(..length).expect("within buffer");
        for byte in input.iter_mut() {
            *byte = u8::try_from(rng.next_u64() & 0xff).expect("low byte");
        }
        check(input);
    }
    let full = full_record();
    let absent = SavedToken {
        refresh_token: None,
        metadata: None,
        issued_client_id: None,
        granted_scopes: None,
        subject: None,
        id_token: None,
        earliest_refresh_at: None,
        ..full_record()
    };
    for record in [full, absent] {
        let encoded = encode_record(&record, &documents()).expect("valid seed");
        check(&encoded);
        for _ in 0..MUTATION_ROUNDS {
            let mut changed = encoded.clone();
            let index =
                usize::try_from(rng.below(u64::try_from(changed.len()).expect("bounded length"))).expect("index fits");
            *changed.get_mut(index).expect("chosen byte") = u8::try_from(rng.next_u64() & 0xff).expect("low byte");
            check(&changed);
        }
    }
}
