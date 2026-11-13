//! Settings rows and their bounds (temper/docs/design/protocol/store.md, sections 2 to 4).
use super::fixtures;
use crate::{Undecodable, keys, records, v17 as w};
use temper_engine_domain_forge_vocabulary::{self as forge, settings as d};

fn rule() -> d::LandingRule {
    d::LandingRule {
        sets: Box::new([]),
        pattern: forge::GatePattern {
            segments: Box::new([Box::from(b"forge".as_slice()), Box::from(b"branch".as_slice())]),
            last: forge::GateLast::Exact(Box::from(b"main".as_slice())),
        },
        enforce: true,
        ci: false,
        up_to_date: true,
        gates: Box::new([
            d::RuleGate { number: 7, blocking: true, freshness: forge::JudgeFreshness::Exact },
            d::RuleGate { number: 9, blocking: false, freshness: forge::JudgeFreshness::Clean },
        ]),
        approvals: Box::new([
            d::RuleApproval { role: 1, people: 2, freshness: forge::JudgeFreshness::Clean },
            d::RuleApproval { role: 2, people: 3, freshness: forge::JudgeFreshness::Exact },
        ]),
    }
}
fn row(edit: d::Edit) -> forge::Stored {
    forge::Stored::Setting(d::SettingRow {
        key: d::SettingKey { kind: 1, scope: 0, number: 1, slot: d::SettingSlot::Default },
        edit,
    })
}
#[test]
fn settings_round_trip_every_slot_edit_value_and_pattern_last_and_match_their_key() {
    use temper_protocol_reaches::{Class, Classes, Item, PathSet, Subject};
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).unwrap();
    let classes = Classes {
        items: Box::new([Item {
            subject: Subject::Set(6),
            words: b"keep the wire format".as_slice().into(),
            class: Class::Ask,
        }]),
        sets: Box::new([PathSet {
            number: 6,
            name: b"protocol".as_slice().into(),
            patterns: Box::new([b"src/**".as_slice().into(), b"docs/*.md".as_slice().into()]),
        }]),
    };
    let mut open = rule();
    open.pattern.last = forge::GateLast::Open(Box::from(b"feature".as_slice()));
    open.sets = Box::new([6, 5, 4, 3, 2, 1]);
    for (kind, value) in [
        (1, d::Value::Rule(rule())),
        (2, d::Value::Rule(open)),
        (3, d::Value::Charter(u32::MAX)),
        (6, d::Value::Questions(temper_engine_domain_forge_change::Route::None)),
        (6, d::Value::Questions(temper_engine_domain_forge_change::Route::Up)),
        (7, d::Value::Ways(d::ProjectWays { drafts: true, goals: false, changes: true, way: u32::MAX })),
        (9, d::Value::CharterWords(Box::from([b'l'; 120].as_slice()))),
        (
            10,
            d::Value::ProjectWords(d::ProjectWords {
                headline: Box::from([b'h'; 120].as_slice()),
                lede: Box::from([b'l'; 480].as_slice()),
            }),
        ),
        (8, d::Value::Classes(classes.clone())),
        (
            14,
            d::Value::ProjectAreas(d::ProjectAreas {
                areas: Box::new([d::Area {
                    name: b"tests".as_slice().into(),
                    words: b"C tests".as_slice().into(),
                    owner: d::AreaOwner::Task(u64::MAX),
                }]),
            }),
        ),
        (15, d::Value::ProjectTriager(d::ProjectTriager { owner: d::AreaOwner::Task(u64::MAX) })),
        (
            14,
            d::Value::ProjectAreas(d::ProjectAreas {
                areas: Box::new([d::Area {
                    name: b"tests".as_slice().into(),
                    words: b"C tests".as_slice().into(),
                    owner: d::AreaOwner::Party(u64::MAX),
                }]),
            }),
        ),
        (15, d::Value::ProjectTriager(d::ProjectTriager { owner: d::AreaOwner::Party(u64::MAX) })),
        (
            14,
            d::Value::ProjectAreas(d::ProjectAreas {
                areas: Box::new([d::Area {
                    name: b"tests".as_slice().into(),
                    words: b"C tests".as_slice().into(),
                    owner: d::AreaOwner::Role(u32::MAX),
                }]),
            }),
        ),
        (15, d::Value::ProjectTriager(d::ProjectTriager { owner: d::AreaOwner::Role(u32::MAX) })),
        (
            14,
            d::Value::ProjectAreas(d::ProjectAreas {
                areas: Box::new([d::Area {
                    name: b"tests".as_slice().into(),
                    words: b"C tests".as_slice().into(),
                    owner: d::AreaOwner::Charter(u32::MAX),
                }]),
            }),
        ),
        (15, d::Value::ProjectTriager(d::ProjectTriager { owner: d::AreaOwner::Charter(u32::MAX) })),
        (11, d::Value::Goal(d::TemplateGoal { design: false, close_out: false })),
        (11, d::Value::Goal(d::TemplateGoal { design: true, close_out: true })),
        (11, d::Value::Goal(d::TemplateGoal { design: true, close_out: false })),
        (11, d::Value::Goal(d::TemplateGoal { design: false, close_out: true })),
    ] {
        for slot in [
            d::SettingSlot::Default,
            d::SettingSlot::Change,
            d::SettingSlot::Proposal(u64::MAX),
            d::SettingSlot::Replaced(u64::MAX),
        ] {
            for edit in
                [d::Edit::Set(d::SizedValue { value: value.clone(), size: 123 }), d::Edit::Remove, d::Edit::Reset]
            {
                let setting = d::SettingRow { key: d::SettingKey { kind, scope: 5, number: 2, slot }, edit };
                let row = forge::Stored::Setting(setting.clone());
                let meter = skein_world::domain::heap::Meter::new();
                meter.start();
                let cloned = row.clone();
                let held = meter.end().held();
                assert_eq!(forge::stored_bytes(&cloned), Some(held));
                drop(cloned);
                let bytes = records::encode(&row);
                assert_eq!(&bytes[..3], &[0, 17, 26]); // The appended value tag leaves earlier tags unchanged.
                assert_eq!(records::decode(&bytes, &bounds), Ok(row.clone()));
                let key = keys::encode(&forge::stored_key(&row), u32::MAX).unwrap();
                assert_eq!(records::row(&key, &bytes, &bounds), Ok(row));
                let wrong =
                    keys::encode(&forge::Key::Setting(d::SettingKey { number: 3, ..setting.key }), u32::MAX).unwrap();
                assert_eq!(records::row(&wrong, &bytes, &bounds), Err(Undecodable::Malformed));
                for end in 0..bytes.len() {
                    assert_eq!(records::decode(&bytes[..end], &bounds), Err(Undecodable::Malformed));
                }
                let mut trailing = skein_lib::Writer::new(bytes.len() + 1);
                trailing.put(&bytes).unwrap();
                trailing.put(&[0]).unwrap();
                assert_eq!(records::decode(&trailing.finish(), &bounds), Err(Undecodable::Malformed));
            }
        }
    }
    for invalid in [
        Classes {
            items: Box::new([Item {
                subject: Subject::Kind(temper_protocol_reaches::Kind::Money),
                words: Box::new([]),
                class: Class::Ask,
            }]),
            ..classes.clone()
        },
        Classes { sets: Box::new([]), ..classes },
    ] {
        assert_eq!(
            records::decode(
                &records::encode(&row(d::Edit::Set(d::SizedValue { value: d::Value::Classes(invalid), size: 1 }))),
                &bounds
            ),
            Err(Undecodable::Malformed)
        );
    }
}
#[test]
fn setting_rules_decode_at_each_bound_and_refuse_one_more() {
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).unwrap();
    assert_eq!(bounds.limits().landing_rule_gates, fixtures::LIMITS.change_policy.gates);
    assert_eq!(bounds.limits().landing_rule_approvals, fixtures::LIMITS.roles);
    assert_eq!(
        bounds.limits().rule_pattern_segments,
        fixtures::LIMITS.authority_segments * (fixtures::LIMITS.name_bytes + 8)
    );
    for (field, bound) in [
        (0_u8, fixtures::LIMITS.change_policy.gates),
        (1, fixtures::LIMITS.roles),
        (2, fixtures::LIMITS.authority_segments),
        (3, fixtures::LIMITS.name_bytes),
        (4, fixtures::LIMITS.name_bytes),
    ] {
        for extra in [0, 1] {
            let count = bound + extra;
            let mut rule = rule();
            match field {
                0 => {
                    let mut gates = skein_lib::List::with_capacity(count);
                    for _ in 0..count {
                        gates.push(rule.gates[0]).unwrap();
                    }
                    rule.gates = gates.into_boxed();
                }
                1 => {
                    let mut approvals = skein_lib::List::with_capacity(count);
                    for _ in 0..count {
                        approvals.push(rule.approvals[0]).unwrap();
                    }
                    rule.approvals = approvals.into_boxed();
                }
                2 => {
                    let mut segments = skein_lib::List::with_capacity(count);
                    for _ in 0..count {
                        segments.push(Box::<[u8]>::default()).unwrap();
                    }
                    rule.pattern.segments = segments.into_boxed();
                }
                3 => rule.pattern.segments = Box::new([skein_lib::bytes::zeroed(usize::try_from(count).unwrap())]),
                4 => {
                    rule.pattern.last =
                        forge::GateLast::Open(skein_lib::bytes::zeroed(usize::try_from(count).unwrap()));
                }
                _ => unreachable!("table field"),
            }
            let row = row(d::Edit::Set(d::SizedValue { value: d::Value::Rule(rule), size: 1 }));
            let decoded = records::decode(&records::encode(&row), &bounds);
            if extra == 0 {
                assert_eq!(decoded, Ok(row));
            } else {
                assert_eq!(decoded, Err(Undecodable::Malformed));
            }
        }
    }
}
#[test]
fn setting_records_refuse_unknown_kinds_and_slot_tags() {
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).unwrap();
    for kind in [0, 4, u32::MAX] {
        let row = w::SettingRow::new(
            &w::CEILINGS,
            w::SettingRowParts {
                kind,
                scope: 0,
                number: 1,
                slot: w::SettingSlot::Default,
                edit: w::SettingEdit::Remove,
            },
        )
        .unwrap();
        let row = w::Row::new(&w::CEILINGS, w::RowParts { value: w::Value::Setting(row) }).unwrap();
        let mut writer = skein_lib::Writer::new(usize::try_from(row.measure()).unwrap());
        row.encode(&mut writer).unwrap();
        assert_eq!(records::decode(&writer.finish(), &bounds), Err(Undecodable::Malformed));
    }
    for (kind, value) in [
        (9, d::Value::CharterWords(Box::from(b"".as_slice()))),
        (9, d::Value::CharterWords(Box::from(b"line\n".as_slice()))),
        (
            10,
            d::Value::ProjectWords(d::ProjectWords {
                headline: Box::from(b"".as_slice()),
                lede: Box::from(b"lede".as_slice()),
            }),
        ),
        (
            10,
            d::Value::ProjectWords(d::ProjectWords {
                headline: Box::from(b"headline".as_slice()),
                lede: Box::from(b"lede\t".as_slice()),
            }),
        ),
    ] {
        let row = forge::Stored::Setting(d::SettingRow {
            key: d::SettingKey { kind, scope: 0, number: 1, slot: d::SettingSlot::Default },
            edit: d::Edit::Set(d::SizedValue { value, size: 1 }),
        });
        assert_eq!(
            records::decode(&records::encode(&row), &bounds),
            Err(Undecodable::Malformed),
            "invalid words for kind {kind}"
        );
    }
    let mut bytes = records::encode(&row(d::Edit::Remove));
    bytes[15] = 4; // Version, Value tag, kind, scope and number precede the slot.
    assert_eq!(records::decode(&bytes, &bounds), Err(Undecodable::Malformed));
}

#[test]
fn a_replaced_row_has_store_v17_golden_bytes() {
    let row = forge::Stored::Setting(d::SettingRow {
        key: d::SettingKey { kind: 3, scope: 0, number: 1, slot: d::SettingSlot::Replaced(3) },
        edit: d::Edit::Set(d::SizedValue { value: d::Value::Charter(2), size: 4 }),
    });
    let bytes: &[u8] =
        &[0, 17, 26, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 1, 3, 0, 0, 0, 0, 0, 0, 0, 3, 0, 1, 0, 0, 0, 2, 0, 0, 0, 4];
    assert_eq!(records::encode(&row).as_ref(), bytes);
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).unwrap();
    assert_eq!(records::decode(bytes, &bounds), Ok(row));
}

fn follow_value() -> d::Follow {
    d::Follow {
        source: d::FollowSource {
            repository: skein_forge::api::Repository { forge: 1, repository: 12 },
            branch: b"main".as_slice().into(),
            paths: Box::new([b"src".as_slice().into(), b"include".as_slice().into()]),
        },
        target: d::FollowTarget {
            repository: skein_forge::api::Repository { forge: 1, repository: 13 },
            branch: b"stable".as_slice().into(),
        },
        owner: d::FollowOwner::Person(4),
        start: d::FollowStart::Code,
        producer: Some(b"6;3.1;".as_slice().into()),
        check: d::FollowCheck {
            workflow: b"tests.yml".as_slice().into(),
            artifact: b"results".as_slice().into(),
            format: temper_engine_domain_forge_tiers::Format::Lines,
        },
    }
}

#[test]
fn a_follow_setting_round_trips_its_wire_value_and_every_slot() {
    follow_round_trip(follow_value());
}

#[test]
fn a_task_owned_follow_round_trips_stored_and_wire_values() {
    let mut follow = follow_value();
    follow.owner = d::FollowOwner::Task(52);
    let wire = d::follow_record(&follow).unwrap();
    let mut writer = skein_lib::Writer::new(9);
    wire.owner().encode(&mut writer).unwrap();
    assert_eq!(writer.finish().as_ref(), &[1, 0, 0, 0, 0, 0, 0, 0, 52]);
    let owner =
        w::FollowOwner::Task(w::FollowOwnerTask::new(&w::CEILINGS, w::FollowOwnerTaskParts { task: 52 }).unwrap());
    let mut writer = skein_lib::Writer::new(9);
    owner.encode(&mut writer).unwrap();
    assert_eq!(writer.finish().as_ref(), &[1, 0, 0, 0, 0, 0, 0, 0, 52]);
    follow_round_trip(follow);
}

fn follow_round_trip(follow: d::Follow) {
    use skein_lib::{Reader, Writer};
    use temper_client_protocol_wire::v1;
    let wire = d::follow_record(&follow).unwrap();
    let mut writer = Writer::new(usize::try_from(wire.measure()).unwrap());
    wire.encode(&mut writer).unwrap();
    let bytes = writer.finish();
    assert_eq!(d::follow_from(v1::Follow::decode(&v1::CEILINGS, &mut Reader::new(&bytes)).unwrap()), follow);
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).unwrap();
    for slot in [d::SettingSlot::Change, d::SettingSlot::Proposal(7), d::SettingSlot::Replaced(2)] {
        for edit in [
            d::Edit::Set(d::SizedValue { value: d::Value::Follow(follow.clone()), size: wire.measure() }),
            d::Edit::Remove,
            d::Edit::Reset,
        ] {
            let key = d::SettingKey { kind: 12, scope: 2, number: 1, slot };
            let row = forge::Stored::Setting(d::SettingRow { key, edit });
            let meter = skein_world::domain::heap::Meter::new();
            meter.start();
            let cloned = row.clone();
            let held = meter.end().held();
            assert_eq!(forge::stored_bytes(&cloned), Some(held));
            drop(cloned);
            let bytes = records::encode(&row);
            assert_eq!(records::decode(&bytes, &bounds), Ok(row.clone()));
            let key = keys::encode(&forge::Key::Setting(key), u32::MAX).unwrap();
            assert_eq!(records::row(&key, &bytes, &bounds), Ok(row));
        }
    }
}

#[test]
fn retired_setting_tags_refuse_with_the_setting_identity() {
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).unwrap();
    let mut tag = skein_lib::Writer::new(1);
    w::Route::Retired.encode(&mut tag).unwrap();
    assert_eq!(tag.finish().as_ref(), &[1], "the retired route retains the old tag");
    let mut tag = skein_lib::Writer::new(1);
    w::Route::Up.encode(&mut tag).unwrap();
    assert_eq!(tag.finish().as_ref(), &[2], "up is never read as the retired route");
    for (kind, scope, number, value) in [
        (
            6,
            2,
            1,
            w::SettingValue::Questions(
                w::ProjectQuestions::new(&w::CEILINGS, w::ProjectQuestionsParts { route: w::Route::Retired }).unwrap(),
            ),
        ),
        (
            3,
            0,
            3,
            w::SettingValue::Charter(w::CharterValue::new(&w::CEILINGS, w::CharterValueParts { number: 6 }).unwrap()),
        ),
    ] {
        for slot in [d::SettingSlot::Change, d::SettingSlot::Replaced(2)] {
            let key = d::SettingKey { kind, scope, number, slot };
            let stored_slot = match slot {
                d::SettingSlot::Change => w::SettingSlot::Change,
                d::SettingSlot::Replaced(number) => {
                    w::SettingSlot::Replaced(w::Number::new(&w::CEILINGS, w::NumberParts { number }).unwrap())
                }
                d::SettingSlot::Default | d::SettingSlot::Proposal(_) => {
                    unreachable!("the test exercises current and replaced rows")
                }
            };
            let row = w::SettingRow::new(
                &w::CEILINGS,
                w::SettingRowParts {
                    kind,
                    scope,
                    number,
                    slot: stored_slot,
                    edit: w::SettingEdit::Set(
                        w::SizedValue::new(&w::CEILINGS, w::SizedValueParts { value: value.clone(), size: 1 }).unwrap(),
                    ),
                },
            )
            .unwrap();
            let row = w::Row::new(&w::CEILINGS, w::RowParts { value: w::Value::Setting(row) }).unwrap();
            let mut bytes = skein_lib::Writer::new(usize::try_from(row.measure()).unwrap());
            row.encode(&mut bytes).unwrap();
            let bytes = bytes.finish();
            assert_eq!(records::decode(&bytes, &bounds), Err(Undecodable::Setting(key)));
            let encoded_key = keys::encode(&forge::Key::Setting(key), u32::MAX).unwrap();
            assert_eq!(records::row(&encoded_key, &bytes, &bounds), Err(Undecodable::Setting(key)));
            assert_eq!(records::row(b"wrong key", &bytes, &bounds), Err(Undecodable::Malformed));
        }
    }
}
