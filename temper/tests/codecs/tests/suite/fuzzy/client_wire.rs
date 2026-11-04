//! Arbitrary and mutated forge-family records (skein's codec.md, section 6).
use skein_lib::{List, Reader, Rng, Writer};
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
fn playground_answer(bytes: &[u8]) {
    if let Some(record) = temper_client_protocol_wire::decode_playground_answer(bytes, &v1::CEILINGS) {
        assert_eq!(temper_client_protocol_wire::playground_answer(&record).as_ref(), bytes, "PlaygroundAnswer");
    }
}
#[test]
fn client_wire_decoders_never_panic_and_reencode_the_same() {
    let decoders: [fn(&[u8]); 11] = [
        requesting,
        answer,
        reading,
        playground_answer,
        watching,
        forges,
        panel,
        landing_rule,
        forge_charter,
        setting_answer,
        proposal_items_answer,
    ];
    let mut rng = Rng::new(0x7e3d_0302);
    let goldens = temper_codecs_world::goldens("crates/temper-client-protocol-wire/golden");
    for _ in 0..2_000 {
        let request = v1::Requesting::new(&v1::CEILINGS, v1::RequestingParts { request: v1::Request::StartPlayground })
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
        let rule = drawn_rule(&mut rng);
        let mut writer = Writer::new(usize::try_from(rule.measure()).expect("measured length"));
        rule.encode(&mut writer).expect("measured room");
        let encoded = writer.finish();
        landing_rule(&encoded);
        landing_rule(&temper_codecs_world::mutated(&mut rng, &encoded));
        let charter = v1::ForgeCharter::new(
            &v1::CEILINGS,
            v1::ForgeCharterParts { charter: u32::try_from(rng.below(1_u64 << 32)).expect("u32 draw") },
        )
        .expect("charter");
        let mut writer = Writer::new(usize::try_from(charter.measure()).expect("measured length"));
        charter.encode(&mut writer).expect("measured room");
        let encoded = writer.finish();
        forge_charter(&encoded);
        forge_charter(&temper_codecs_world::mutated(&mut rng, &encoded));
        let bytes = temper_codecs_world::drawn(&mut rng, 512);
        for decode in decoders {
            decode(&bytes);
        }
    }
    for golden in &goldens {
        for decode in decoders {
            decode(golden);
        }
        for _ in 0..8 {
            let bytes = temper_codecs_world::mutated(&mut rng, golden);
            for decode in decoders {
                decode(&bytes);
            }
        }
    }
}

fn landing_rule(bytes: &[u8]) {
    let mut reader = Reader::new(bytes);
    if let Ok(record) = v1::LandingRule::decode(&v1::CEILINGS, &mut reader) {
        if !reader.is_empty() {
            return;
        }
        let mut writer = Writer::new(usize::try_from(record.measure()).expect("measured length"));
        record.encode(&mut writer).expect("measured room");
        assert_eq!(writer.finish().as_ref(), bytes, "LandingRule");
    }
}

fn forge_charter(bytes: &[u8]) {
    let mut reader = Reader::new(bytes);
    if let Ok(record) = v1::ForgeCharter::decode(&v1::CEILINGS, &mut reader) {
        if !reader.is_empty() {
            return;
        }
        let mut writer = Writer::new(usize::try_from(record.measure()).expect("measured length"));
        record.encode(&mut writer).expect("measured room");
        assert_eq!(writer.finish().as_ref(), bytes, "ForgeCharter");
    }
}

fn drawn_freshness(rng: &mut Rng) -> v1::RuleFreshness {
    if rng.chance(500) { v1::RuleFreshness::Exact } else { v1::RuleFreshness::Clean }
}

fn drawn_rule(rng: &mut Rng) -> v1::LandingRule {
    let limits = &v1::CEILINGS;
    let mut segments = List::with_capacity(5);
    for _ in 0..rng.below(6) {
        segments.push(temper_codecs_world::drawn(rng, 16).into_boxed_slice()).expect("bounded segments");
    }
    let value = temper_codecs_world::drawn(rng, 16).into_boxed_slice();
    let last = if rng.chance(500) {
        v1::LandingLast::Exact(v1::LandingExact::new(limits, v1::LandingExactParts { value }).expect("exact last"))
    } else {
        v1::LandingLast::Open(v1::LandingOpen::new(limits, v1::LandingOpenParts { value }).expect("open last"))
    };
    let pattern = v1::LandingPattern::new(limits, v1::LandingPatternParts { segments, last }).expect("pattern");
    let mut gates = List::with_capacity(3);
    for _ in 0..rng.below(4) {
        gates
            .push(
                v1::RuleGate::new(
                    limits,
                    v1::RuleGateParts {
                        number: u32::try_from(rng.below(1_u64 << 32)).expect("u32 draw"),
                        blocking: rng.chance(500),
                        freshness: drawn_freshness(rng),
                    },
                )
                .expect("gate"),
            )
            .expect("bounded gates");
    }
    let mut approvals = List::with_capacity(3);
    for _ in 0..rng.below(4) {
        approvals
            .push(
                v1::RuleApproval::new(
                    limits,
                    v1::RuleApprovalParts {
                        role: u32::try_from(rng.below(1_u64 << 32)).expect("u32 draw"),
                        people: u32::try_from(rng.below(1_u64 << 32)).expect("u32 draw"),
                        freshness: drawn_freshness(rng),
                    },
                )
                .expect("approval"),
            )
            .expect("bounded approvals");
    }
    v1::LandingRule::new(
        limits,
        v1::LandingRuleParts {
            pattern,
            enforce: rng.chance(500),
            ci: rng.chance(500),
            up_to_date: rng.chance(500),
            gates,
            approvals,
        },
    )
    .expect("bounded rule")
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

fn setting_answer(bytes: &[u8]) {
    if let Some(record) = temper_client_protocol_wire::decode_setting_answer(bytes, &v1::CEILINGS) {
        assert_eq!(temper_client_protocol_wire::setting_answer(&record).as_ref(), bytes, "SettingAnswer");
    }
}
fn proposal_items_answer(bytes: &[u8]) {
    if let Some(record) = temper_client_protocol_wire::decode_proposal_items_answer(bytes, &v1::CEILINGS) {
        assert_eq!(temper_client_protocol_wire::proposal_items_answer(&record).as_ref(), bytes, "ProposalItemsAnswer");
    }
}
