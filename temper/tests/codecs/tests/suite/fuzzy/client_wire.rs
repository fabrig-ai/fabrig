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
fn playground_answer(bytes: &[u8]) {
    if let Some(record) = temper_client_protocol_wire::decode_playground_answer(bytes, &v1::CEILINGS) {
        assert_eq!(temper_client_protocol_wire::playground_answer(&record).as_ref(), bytes, "PlaygroundAnswer");
    }
}
fn playground(rng: &mut Rng) -> v1::PlaygroundAnswer {
    let state = match rng.below(4) {
        0 => v1::PlaygroundState::Offered,
        1 => v1::PlaygroundState::NotOffered,
        2 => v1::PlaygroundState::Running,
        _ => v1::PlaygroundState::Resetting,
    };
    let count = u32::try_from(rng.below(9)).expect("cue count");
    let mut cues = skein_lib::List::with_capacity(count);
    for _ in 0..count {
        let length = usize::try_from(rng.between(1, 256)).expect("cue bytes");
        let mut cue = skein_lib::bytes::zeroed(length);
        cue.fill(b'a');
        cues.push(cue).expect("cue room");
    }
    let forge = if rng.below(2) == 0 { None } else { Some(u16::try_from(rng.below(65_536)).expect("forge")) };
    let project = if rng.below(2) == 0 { None } else { Some(u32::try_from(rng.below(1_u64 << 32)).expect("project")) };
    v1::PlaygroundAnswer::new(
        &v1::CEILINGS,
        v1::PlaygroundAnswerParts {
            state,
            forge,
            project,
            provider: u32::try_from(rng.below(1_u64 << 32)).expect("provider"),
            owner: temper_codecs_world::drawn(rng, 1024).into_boxed_slice(),
            name: temper_codecs_world::drawn(rng, 1024).into_boxed_slice(),
            cues,
        },
    )
    .expect("bounded playground answer")
}
#[test]
fn client_wire_decoders_never_panic_and_reencode_the_same() {
    let decoders: [fn(&[u8]); 8] =
        [requesting, answer, reading, demo_answer, playground_answer, watching, forges, panel];
    let mut rng = Rng::new(0x7e3d_0302);
    let goldens = temper_codecs_world::goldens("crates/temper-client-protocol-wire/golden");
    let reset = v1::Answer::new(&v1::CEILINGS, v1::AnswerParts { outcome: v1::Outcome::Reset }).expect("reset answer");
    let reset = temper_client_protocol_wire::answer(&reset);
    answer(&reset);
    for _ in 0..2_000 {
        answer(&temper_codecs_world::mutated(&mut rng, &reset));
        let request = match rng.below(3) {
            0 => v1::Request::StartDemo,
            1 => v1::Request::StartPlayground,
            _ => v1::Request::EndPlayground,
        };
        let request =
            v1::Requesting::new(&v1::CEILINGS, v1::RequestingParts { request }).expect("start or end request");
        let encoded = temper_client_protocol_wire::requesting(&request);
        requesting(&encoded);
        requesting(&temper_codecs_world::mutated(&mut rng, &encoded));
        let record = playground(&mut rng);
        let encoded = temper_client_protocol_wire::playground_answer(&record);
        playground_answer(&encoded);
        playground_answer(&temper_codecs_world::mutated(&mut rng, &encoded));
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
