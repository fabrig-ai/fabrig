//! Hostile cursor bytes (jig/docs/design/protocol/client/README.md, 4; skein/docs/design/data/codec/README.md, 6).
use jig_protocol_engine_clients::cursor_v2::{CEILINGS, Held, Record};
use skein_lib::{Reader, Rng, length::len_u64};

fn check(bytes: &[u8]) {
    if let Ok(held) = Held::decode(&CEILINGS, &mut Reader::new(bytes)) {
        assert_eq!(held.encode(&CEILINGS).expect("decoded cursor").as_ref(), bytes);
    }
}
fn fixed_values() -> Vec<Vec<u8>> {
    let text = jig_protocol_engine_clients::SCHEMA;
    let mut fixed_values = skein_codegen::values(text).fixed.clone();
    fixed_values.sort_by(|left, right| left.name.cmp(&right.name));
    let mut values = Vec::new();
    for fixed_value in fixed_values {
        let name = &fixed_value.name;
        let prefix: &[u8] = if name.starts_with("record_held_") {
            &[]
        } else if name.starts_with("enum_form_") {
            &[0, 2]
        } else if name.starts_with("enum_note_scope_") {
            &[0, 2, 0, 1, 13]
        } else if name.starts_with("enum_scope_") {
            &[0, 2, 0, 1]
        } else {
            continue;
        };
        let mut bytes = prefix.to_vec();
        bytes.extend(fixed_value.bytes);
        if name.starts_with("enum_note_scope_") {
            bytes.push(0);
        }
        if name.starts_with("enum_scope_") || name.starts_with("enum_note_scope_") {
            bytes.push(0);
        }
        Held::decode(&CEILINGS, &mut Reader::new(&bytes)).expect("held fixed-rule value");
        values.push(bytes);
    }
    assert_ne!(values.len(), 0);
    values
}
#[test]
fn arbitrary_cursor_bytes_never_panic_and_round_trip_when_they_decode() {
    for bytes in fixed_values() {
        check(&bytes);
    }
    let mut rng = Rng::new(0xc075_0001);
    let mut buffer = [0_u8; 1024];
    for _ in 0..10_000 {
        let length = usize::try_from(rng.below(1025)).expect("bounded length");
        let bytes = buffer.get_mut(..length).expect("within buffer");
        for byte in bytes.iter_mut() {
            *byte = u8::try_from(rng.next_u64() & 0xff).expect("low byte");
        }
        check(bytes);
    }
}
#[test]
fn mutated_cursor_fixed_values_never_panic_and_round_trip_when_they_decode() {
    let mut rng = Rng::new(0xc075_0002);
    for fixed_value in fixed_values() {
        check(&fixed_value);
        for cut in 0..fixed_value.len() {
            check(fixed_value.get(..cut).expect("within fixed-rule value"));
        }
        let mut trailing = fixed_value.clone();
        trailing.push(0);
        assert!(Held::decode(&CEILINGS, &mut Reader::new(&trailing)).is_err());
        for _ in 0..100 {
            let mut bytes = fixed_value.clone();
            let index = usize::try_from(rng.below(len_u64(bytes.len()))).expect("index");
            *bytes.get_mut(index).expect("chosen byte") = u8::try_from(rng.next_u64() & 0xff).expect("low byte");
            check(&bytes);
        }
    }
}
