//! Arbitrary and mutated forge-family records (skein's data/codec/README.md, section 6).
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
#[expect(clippy::too_many_lines, reason = "one mutation sweep checks the forge-family decoders and request outcomes")]
fn client_wire_decoders_never_panic_and_reencode_the_same() {
    let decoders: [fn(&[u8]); 17] = [
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
        ways_panel,
        view_answer,
        project_areas,
        project_triager,
        findings,
        finding_answer,
    ];
    let mut rng = Rng::new(0x7e3d_0302);
    let goldens = temper_codecs_world::goldens("crates/temper-client-protocol-wire/golden");
    for _ in 0..2_000 {
        let request = v1::Requesting::new(&v1::CEILINGS, v1::RequestingParts { request: drawn_request(&mut rng) })
            .expect("bounded request");
        let bytes = temper_client_protocol_wire::requesting(&request);
        assert_eq!(temper_client_protocol_wire::decode_requesting(&bytes, &v1::CEILINGS), Some(request));
        requesting(&bytes);
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
                outcome: match rng.below(3) {
                    0 => v1::Outcome::StartFailed(
                        v1::StartFailed::new(&v1::CEILINGS, v1::StartFailedParts { why }).expect("failure"),
                    ),
                    1 => v1::Outcome::MakeKept,
                    _ => v1::Outcome::MakeRefused(
                        v1::MakeRefused::new(
                            &v1::CEILINGS,
                            v1::MakeRefusedParts {
                                why: match rng.below(7) {
                                    0 => v1::MakeRefusal::NoSuchForge,
                                    1 => v1::MakeRefusal::NotOffered,
                                    2 => v1::MakeRefusal::NotOwner,
                                    3 => v1::MakeRefusal::Invalid,
                                    4 => v1::MakeRefusal::HomeHeld,
                                    5 => v1::MakeRefusal::Full,
                                    _ => v1::MakeRefusal::NotFailed,
                                },
                            },
                        )
                        .expect("drawn fields fit the schema ceilings"),
                    ),
                },
            },
        )
        .expect("answer");
        let encoded = temper_client_protocol_wire::answer(&record);
        answer(&encoded);
        answer(&temper_codecs_world::mutated(&mut rng, &encoded));
        let record = drawn_makings(&mut rng);
        let encoded = temper_client_protocol_wire::forges(&record);
        forges(&encoded);
        forges(&temper_codecs_world::mutated(&mut rng, &encoded));
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

fn drawn_make_from(rng: &mut Rng) -> v1::MakeFrom {
    if rng.chance(500) {
        v1::MakeFrom::Empty
    } else {
        v1::MakeFrom::Copy(
            v1::MakeCopy::new(
                &v1::CEILINGS,
                v1::MakeCopyParts { address: temper_codecs_world::drawn(rng, 512).into_boxed_slice() },
            )
            .expect("drawn fields fit the schema ceilings"),
        )
    }
}
fn drawn_make_into(rng: &mut Rng) -> Option<v1::MakeInto> {
    if rng.chance(500) {
        None
    } else {
        Some(
            v1::MakeInto::new(
                &v1::CEILINGS,
                v1::MakeIntoParts {
                    project: u32::try_from(rng.below(1_u64 << 32)).expect("u32 draw"),
                    home: rng.chance(500),
                },
            )
            .expect("drawn fields fit the schema ceilings"),
        )
    }
}
fn drawn_making_state(rng: &mut Rng) -> v1::MakingState {
    match rng.below(5) {
        0 => v1::MakingState::Waiting,
        1 => v1::MakingState::Making,
        2 => v1::MakingState::Made(drawn_made(rng)),
        3 => v1::MakingState::Adopting(drawn_made(rng)),
        _ => v1::MakingState::Failed(
            v1::MakingFailed::new(
                &v1::CEILINGS,
                v1::MakingFailedParts {
                    step: match rng.below(5) {
                        0 => v1::MakeStep::Admin,
                        1 => v1::MakeStep::Org,
                        2 => v1::MakeStep::Repository,
                        3 => v1::MakeStep::File,
                        _ => v1::MakeStep::Adoption,
                    },
                    reason: match rng.below(6) {
                        0 => v1::MakeFailure::Taken,
                        1 => v1::MakeFailure::Empty,
                        2 => v1::MakeFailure::TimedOut,
                        3 => v1::MakeFailure::Refused(
                            v1::Refused::new(
                                &v1::CEILINGS,
                                v1::RefusedParts { status: u16::try_from(rng.below(65_536)).expect("u16 draw") },
                            )
                            .expect("drawn fields fit the schema ceilings"),
                        ),
                        4 => v1::MakeFailure::NotAllowed,
                        _ => v1::MakeFailure::Other,
                    },
                    words: temper_codecs_world::drawn(rng, 256).into_boxed_slice(),
                },
            )
            .expect("drawn fields fit the schema ceilings"),
        ),
    }
}
fn drawn_made(rng: &mut Rng) -> v1::MakingMade {
    v1::MakingMade::new(
        &v1::CEILINGS,
        v1::MakingMadeParts { id: rng.next_u64(), branch: temper_codecs_world::drawn(rng, 1024).into_boxed_slice() },
    )
    .expect("drawn fields fit the schema ceilings")
}
fn drawn_makings(rng: &mut Rng) -> v1::Forges {
    // All unrelated fields stay at the generated smallest row's values.
    let mut parts = v1::ForgeRow::decode(
        &v1::CEILINGS,
        &mut Reader::new(include_bytes!(
            "../../../../../crates/temper-client-protocol-wire/golden/record_forge_row_smallest.bin"
        )),
    )
    .expect("drawn fields fit the schema ceilings")
    .into_parts();
    parts.makings = List::with_capacity(1);
    parts
        .makings
        .push(
            v1::RepositoryMaking::new(
                &v1::CEILINGS,
                v1::RepositoryMakingParts {
                    org: temper_codecs_world::drawn(rng, 1024).into_boxed_slice(),
                    name: temper_codecs_world::drawn(rng, 1024).into_boxed_slice(),
                    from: drawn_make_from(rng),
                    into: drawn_make_into(rng),
                    state: drawn_making_state(rng),
                },
            )
            .expect("drawn fields fit the schema ceilings"),
        )
        .expect("drawn fields fit the schema ceilings");
    let mut forges = List::with_capacity(1);
    forges
        .push(v1::ForgeRow::new(&v1::CEILINGS, parts).expect("drawn fields fit the schema ceilings"))
        .expect("drawn fields fit the schema ceilings");
    v1::Forges::new(&v1::CEILINGS, v1::ForgesParts { forges, linked: false })
        .expect("drawn fields fit the schema ceilings")
}

fn drawn_request(rng: &mut Rng) -> v1::Request {
    match rng.below(6) {
        0 => v1::Request::StartPlayground,
        1 => v1::Request::DismissMaking(
            v1::DismissMaking::new(
                &v1::CEILINGS,
                v1::DismissMakingParts {
                    into: if rng.chance(500) {
                        Some(u32::try_from(rng.below(1_u64 << 32)).expect("u32 draw"))
                    } else {
                        None
                    },
                    forge: u16::try_from(rng.below(65_536)).expect("u16 draw"),
                    org: temper_codecs_world::drawn(rng, 1024).into_boxed_slice(),
                    name: temper_codecs_world::drawn(rng, 1024).into_boxed_slice(),
                },
            )
            .expect("bounded dismissal"),
        ),
        2 => v1::Request::MakeRepository(
            v1::MakeRepository::new(
                &v1::CEILINGS,
                v1::MakeRepositoryParts {
                    forge: u16::try_from(rng.below(65_536)).expect("u16 draw"),
                    org: temper_codecs_world::drawn(rng, 1024).into_boxed_slice(),
                    name: temper_codecs_world::drawn(rng, 1024).into_boxed_slice(),
                    from: drawn_make_from(rng),
                    into: drawn_make_into(rng),
                },
            )
            .expect("bounded make request"),
        ),
        3 => v1::Request::FileFinding(
            v1::FileFinding::new(
                &v1::CEILINGS,
                v1::FileFindingParts {
                    project: u32::try_from(rng.below(1_u64 << 32)).expect("u32 draw"),
                    kind: match rng.below(5) {
                        0 => v1::FindingKind::Qa,
                        1 => v1::FindingKind::Flake,
                        2 => v1::FindingKind::Red,
                        3 => v1::FindingKind::Review,
                        _ => v1::FindingKind::Feedback,
                    },
                    severity: match rng.below(3) {
                        0 => v1::Severity::Low,
                        1 => v1::Severity::Medium,
                        _ => v1::Severity::High,
                    },
                    area: if rng.chance(500) { Some(drawn_words(rng, 32)) } else { None },
                    title: drawn_words(rng, 200),
                    body: drawn_words(rng, 4096),
                },
            )
            .expect("bounded finding"),
        ),
        4 => v1::Request::HandFinding(
            v1::HandFinding::new(
                &v1::CEILINGS,
                v1::HandFindingParts {
                    project: u32::try_from(rng.below(1_u64 << 32)).expect("u32 draw"),
                    number: u32::try_from(rng.below(1_u64 << 32)).expect("u32 draw"),
                    area: drawn_words(rng, 32),
                    why: drawn_words(rng, 200),
                },
            )
            .expect("bounded hand"),
        ),
        _ => v1::Request::TriageFinding(
            v1::TriageFinding::new(
                &v1::CEILINGS,
                v1::TriageFindingParts {
                    project: u32::try_from(rng.below(1_u64 << 32)).expect("u32 draw"),
                    number: u32::try_from(rng.below(1_u64 << 32)).expect("u32 draw"),
                    act: match rng.below(3) {
                        0 => v1::Triage::Route(
                            v1::FindingRoute::new(&v1::CEILINGS, v1::FindingRouteParts { task: rng.below(u64::MAX) })
                                .expect("route"),
                        ),
                        1 => v1::Triage::Backlog,
                        _ => v1::Triage::Drop(
                            v1::FindingDrop::new(&v1::CEILINGS, v1::FindingDropParts { reason: drawn_words(rng, 400) })
                                .expect("drop"),
                        ),
                    },
                },
            )
            .expect("bounded triage"),
        ),
    }
}

fn ways_panel(bytes: &[u8]) {
    if let Some(record) = temper_client_protocol_wire::decode_ways_panel(bytes, &v1::CEILINGS) {
        assert_eq!(temper_client_protocol_wire::ways_panel(&record).as_ref(), bytes, "WaysPanel");
    }
}

fn view_answer(bytes: &[u8]) {
    if let Some(record) = temper_client_protocol_wire::decode_view_answer(bytes, &v1::CEILINGS) {
        assert_eq!(temper_client_protocol_wire::view_answer(&record).as_ref(), bytes, "ViewAnswer");
    }
}

fn project_areas(bytes: &[u8]) {
    let mut reader = Reader::new(bytes);
    if let Ok(record) = v1::ProjectAreas::decode(&v1::CEILINGS, &mut reader) {
        assert!(reader.is_empty());
        let mut writer = Writer::new(usize::try_from(record.measure()).expect("measured bytes"));
        record.encode(&mut writer).expect("measured room");
        assert_eq!(writer.finish().as_ref(), bytes);
    }
}

fn project_triager(bytes: &[u8]) {
    let mut reader = Reader::new(bytes);
    if let Ok(record) = v1::ProjectTriager::decode(&v1::CEILINGS, &mut reader) {
        assert!(reader.is_empty());
        let mut writer = Writer::new(usize::try_from(record.measure()).expect("measured bytes"));
        record.encode(&mut writer).expect("measured room");
        assert_eq!(writer.finish().as_ref(), bytes);
    }
}

fn drawn_words(rng: &mut Rng, most: u32) -> Box<[u8]> {
    let length = usize::try_from(rng.between(1, u64::from(most))).expect("word length");
    vec![b'a'; length].into_boxed_slice()
}
fn findings(bytes: &[u8]) {
    if let Some(record) = temper_client_protocol_wire::decode_findings(bytes, &v1::CEILINGS) {
        assert_eq!(temper_client_protocol_wire::findings(&record).as_ref(), bytes, "Findings");
    }
}
fn finding_answer(bytes: &[u8]) {
    if let Some(record) = temper_client_protocol_wire::decode_finding_answer(bytes, &v1::CEILINGS) {
        assert_eq!(temper_client_protocol_wire::finding_answer(&record).as_ref(), bytes, "FindingAnswer");
    }
}
