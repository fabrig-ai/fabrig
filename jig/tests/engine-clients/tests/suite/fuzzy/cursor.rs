//! Hostile cursor bytes (jig/docs/design/protocol/client/README.md, 4; skein/docs/design/data/codec/README.md, 6).
use jig_protocol_engine_clients::cursor_v1::{CEILINGS, Held};
use skein_lib::{Reader, Rng, Writer, length::len_u64};
use std::fs;

fn check(bytes: &[u8]) {
    if let Ok(held) = Held::decode(&CEILINGS, &mut Reader::new(bytes)) {
        let mut writer = Writer::new(skein_lib::length::usize_of(held.measure()));
        held.encode(&mut writer).expect("measured cursor");
        assert_eq!(writer.finish().as_ref(), bytes);
    }
}
fn goldens() -> Vec<Vec<u8>> {
    let root = skein_scratch::paths::tree().join("crates/jig-protocol-engine-clients/golden");
    let mut paths: Vec<_> = fs::read_dir(root).expect("cursor goldens").map(|p| p.expect("entry").path()).collect();
    paths.sort();
    let mut values = Vec::new();
    for path in paths {
        let name = path.file_name().expect("golden name").to_string_lossy();
        let prefix: &[u8] = if name.starts_with("record_held_") {
            &[]
        } else if name.starts_with("enum_form_") {
            &[0, 1]
        } else if name.starts_with("enum_note_scope_") {
            &[0, 1, 0, 1, 13]
        } else if name.starts_with("enum_scope_") {
            &[0, 1, 0, 1]
        } else {
            continue;
        };
        let mut bytes = prefix.to_vec();
        bytes.extend(fs::read(&path).expect("golden bytes"));
        if name.starts_with("enum_note_scope_") {
            bytes.push(0);
        }
        if name.starts_with("enum_scope_") || name.starts_with("enum_note_scope_") {
            bytes.push(0);
        }
        Held::decode(&CEILINGS, &mut Reader::new(&bytes)).expect("held golden");
        values.push(bytes);
    }
    assert_ne!(values.len(), 0);
    values
}
#[test]
fn arbitrary_cursor_bytes_never_panic_and_round_trip_when_they_decode() {
    for bytes in goldens() {
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
fn mutated_cursor_goldens_never_panic_and_round_trip_when_they_decode() {
    let mut rng = Rng::new(0xc075_0002);
    for golden in goldens() {
        check(&golden);
        for cut in 0..golden.len() {
            check(golden.get(..cut).expect("within golden"));
        }
        let mut trailing = golden.clone();
        trailing.push(0);
        assert!(Held::decode(&CEILINGS, &mut Reader::new(&trailing)).is_err());
        for _ in 0..100 {
            let mut bytes = golden.clone();
            let index = usize::try_from(rng.below(len_u64(bytes.len()))).expect("index");
            *bytes.get_mut(index).expect("chosen byte") = u8::try_from(rng.next_u64() & 0xff).expect("low byte");
            check(&bytes);
        }
    }
}
