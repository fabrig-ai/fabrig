//! Arbitrary and mutated channel parts (protocol/channel.md, section 9).
use super::super::values;
use skein_lib::{Reader, Rng};
use temper_protocol_worker_wire::v1;
use temper_protocol_worker_wire::v1::Record;

fn items(bytes: &[u8]) {
    if let Ok(record) = v1::Items::decode(&v1::CEILINGS, &mut Reader::new(bytes)) {
        let encoded = record.encode(&v1::CEILINGS).expect("decoded value within ceilings");
        assert_eq!(encoded.as_ref(), bytes, "Items");
    }
}

fn repository(bytes: &[u8]) {
    if let Ok(record) = v1::Repository::decode(&v1::CEILINGS, &mut Reader::new(bytes)) {
        let encoded = record.encode(&v1::CEILINGS).expect("decoded value within ceilings");
        assert_eq!(encoded.as_ref(), bytes, "Repository");
    }
}

fn left(bytes: &[u8]) {
    if let Ok(record) = v1::Left::decode(&v1::CEILINGS, &mut Reader::new(bytes)) {
        let encoded = record.encode(&v1::CEILINGS).expect("decoded value within ceilings");
        assert_eq!(encoded.as_ref(), bytes, "Left");
    }
}

fn saved(bytes: &[u8]) {
    if let Ok(record) = v1::Saved::decode(&v1::CEILINGS, &mut Reader::new(bytes)) {
        let encoded = record.encode(&v1::CEILINGS).expect("decoded value within ceilings");
        assert_eq!(encoded.as_ref(), bytes, "Saved");
    }
}

fn preparation(bytes: &[u8]) {
    if let Ok(record) = v1::Preparation::decode(&v1::CEILINGS, &mut Reader::new(bytes)) {
        let encoded = record.encode(&v1::CEILINGS).expect("decoded value within ceilings");
        assert_eq!(encoded.as_ref(), bytes, "Preparation");
    }
}

fn identity(bytes: &[u8]) {
    if let Ok(record) = v1::Identity::decode(&v1::CEILINGS, &mut Reader::new(bytes)) {
        let encoded = record.encode(&v1::CEILINGS).expect("decoded value within ceilings");
        assert_eq!(encoded.as_ref(), bytes, "Identity");
    }
}

#[test]
fn worker_wire_decoders_never_panic_and_reencode_the_same() {
    let decoders: [fn(&[u8]); 6] = [items, repository, left, saved, preparation, identity];
    let mut rng = Rng::new(0x7e3d_0301);
    let fixed_values = values::worker();
    let mut mutations = temper_codecs_world::MutationBatch::default();
    for fixed_value in [
        values::fixed_value(values::worker(), "enum_kind_forgejo_full.bin"),
        values::fixed_value(values::worker(), "enum_kind_github_full.bin"),
    ] {
        let kind = v1::Kind::decode(&v1::CEILINGS, &mut Reader::new(fixed_value)).expect("kind fixed-rule value");
        let value = v1::Identity {
            kind,
            login: b"bot".as_slice().into(),
            name: b"Bot".as_slice().into(),
            email: b"bot@test".as_slice().into(),
        };
        let seed = temper_protocol_worker_wire::identity(&value);
        identity(seed.expose());
        for _ in 0..8 {
            mutations.judge(temper_codecs_world::mutated(&mut rng, seed.expose()), |bytes| identity(&bytes));
        }
    }
    for _ in 0..2_000 {
        let bytes = temper_codecs_world::drawn(&mut rng, 512);
        for decode in decoders {
            decode(&bytes);
        }
    }
    for fixed_value in fixed_values {
        let fixed_value = &fixed_value.bytes;
        for decode in decoders {
            decode(fixed_value);
        }
        for _ in 0..8 {
            mutations.judge(temper_codecs_world::mutated(&mut rng, fixed_value), |bytes| {
                for decode in decoders {
                    decode(&bytes);
                }
            });
        }
    }
    mutations.finish();
}
