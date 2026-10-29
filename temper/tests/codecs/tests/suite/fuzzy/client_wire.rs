//! Arbitrary and mutated forge-family records (skein's codec.md, section 6).
use skein_lib::{Reader, Rng, Writer};
use temper_client_protocol_wire::v1;

fn requesting(bytes: &[u8]) {
    if let Some(record) = temper_client_protocol_wire::decode_requesting(bytes, &v1::CEILINGS) {
        assert_eq!(temper_client_protocol_wire::requesting(&record).as_ref(), bytes, "Requesting");
    }
}

fn answer(bytes: &[u8]) {
    let mut reader = Reader::new(bytes);
    if let Ok(record) = v1::Answer::decode(&v1::CEILINGS, &mut reader) {
        if !reader.is_empty() {
            return;
        }
        let mut writer = Writer::new(usize::try_from(record.measure()).expect("measured length"));
        record.encode(&mut writer).expect("measured room");
        assert_eq!(writer.finish().as_ref(), bytes, "Answer");
    }
}

fn reading(bytes: &[u8]) {
    if let Some(record) = temper_client_protocol_wire::decode_reading(bytes, &v1::CEILINGS) {
        assert_eq!(temper_client_protocol_wire::reading(&record).as_ref(), bytes, "Reading");
    }
}
fn demo_answer(bytes: &[u8]) {
    if let Some(record) = temper_client_protocol_wire::decode_demo_answer(bytes, &v1::CEILINGS) {
        assert_eq!(temper_client_protocol_wire::demo_answer(&record).as_ref(), bytes, "DemoAnswer");
    }
}
#[test]
fn client_wire_decoders_never_panic_and_reencode_the_same() {
    let decoders: [fn(&[u8]); 7] = [requesting, answer, reading, demo_answer, watching, forges, panel];
    let mut rng = Rng::new(0x7e3d_0302);
    let goldens = temper_codecs_world::goldens("crates/temper-client-protocol-wire/golden");
    for _ in 0..2_000 {
        let request = v1::Requesting::new(&v1::CEILINGS, v1::RequestingParts { request: v1::Request::StartDemo })
            .expect("start request");
        requesting(&temper_client_protocol_wire::requesting(&request));
        let why = match rng.below(4) {
            0 => v1::StartFailure::Group(
                v1::GroupFailed::new(
                    &v1::CEILINGS,
                    v1::GroupFailedParts { why: temper_codecs_world::drawn(&mut rng, 256).into_boxed_slice() },
                )
                .expect("bounded why"),
            ),
            1 => v1::StartFailure::Account,
            2 => v1::StartFailure::Adoption,
            _ => v1::StartFailure::NotConnected,
        };
        let record = v1::Answer::new(
            &v1::CEILINGS,
            v1::AnswerParts {
                outcome: v1::Outcome::StartFailed(
                    v1::StartFailed::new(&v1::CEILINGS, v1::StartFailedParts { why }).expect("failure"),
                ),
            },
        )
        .expect("answer");
        let encoded = temper_client_protocol_wire::answer(&record);
        answer(&encoded);
        answer(&temper_codecs_world::mutated(&mut rng, &encoded));
        let bytes = temper_codecs_world::drawn(&mut rng, 512);
        for decode in decoders {
            decode(&bytes);
        }
    }
    for golden in &goldens {
        for _ in 0..8 {
            let bytes = temper_codecs_world::mutated(&mut rng, golden);
            for decode in decoders {
                decode(&bytes);
            }
        }
    }
}

fn watching(bytes: &[u8]) {
    if let Some(record) = temper_client_protocol_wire::decode_watching(bytes, &v1::CEILINGS) {
        assert_eq!(temper_client_protocol_wire::watching(&record).as_ref(), bytes, "Watching");
    }
}
fn forges(bytes: &[u8]) {
    if let Some(record) = temper_client_protocol_wire::decode_forges(bytes, &v1::CEILINGS) {
        assert_eq!(temper_client_protocol_wire::forges(&record).as_ref(), bytes, "Forges");
    }
}
fn panel(bytes: &[u8]) {
    if let Some(record) = temper_client_protocol_wire::decode_change_panel(bytes, &v1::CEILINGS) {
        assert_eq!(temper_client_protocol_wire::change_panel(&record).as_ref(), bytes, "ChangePanel");
    }
}
