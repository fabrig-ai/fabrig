//! Settings rows and their bounds (temper/docs/design/protocol/store.md, sections 2 to 4).
use super::fixtures;
use crate::{Undecodable, keys, records, v11 as w};
use temper_engine_domain_forge::{self as forge, settings as d};

fn rule() -> d::LandingRule {
    d::LandingRule {
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
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).unwrap();
    let mut open = rule();
    open.pattern.last = forge::GateLast::Open(Box::from(b"feature".as_slice()));
    for (kind, value) in [(1, d::Value::Rule(rule())), (2, d::Value::Rule(open)), (3, d::Value::Charter(u32::MAX))] {
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
                assert_eq!(&bytes[..3], &[0, 11, 26]); // The appended value tag leaves earlier tags unchanged.
                assert_eq!(records::decode(&bytes, &bounds), Ok(row.clone()));
                let key = keys::encode(&forge::stored_key(&row), u32::MAX).unwrap();
                assert_eq!(records::row(&key, &bytes, &bounds), Ok(row));
                let wrong =
                    keys::encode(&forge::Key::Setting(d::SettingKey { number: 3, ..setting.key }), u32::MAX).unwrap();
                assert_eq!(records::row(&wrong, &bytes, &bounds), Err(Undecodable));
                for end in 0..bytes.len() {
                    assert_eq!(records::decode(&bytes[..end], &bounds), Err(Undecodable));
                }
                let mut trailing = skein_lib::Writer::new(bytes.len() + 1);
                trailing.put(&bytes).unwrap();
                trailing.put(&[0]).unwrap();
                assert_eq!(records::decode(&trailing.finish(), &bounds), Err(Undecodable));
            }
        }
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
                assert_eq!(decoded, Err(Undecodable));
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
        assert_eq!(records::decode(&writer.finish(), &bounds), Err(Undecodable));
    }
    let mut bytes = records::encode(&row(d::Edit::Remove));
    bytes[15] = 4; // Version, Value tag, kind, scope and number precede the slot.
    assert_eq!(records::decode(&bytes, &bounds), Err(Undecodable));
}

#[test]
fn a_replaced_row_has_store_v11_golden_bytes() {
    let row = forge::Stored::Setting(d::SettingRow {
        key: d::SettingKey { kind: 3, scope: 0, number: 1, slot: d::SettingSlot::Replaced(3) },
        edit: d::Edit::Set(d::SizedValue { value: d::Value::Charter(2), size: 4 }),
    });
    let bytes: &[u8] =
        &[0, 11, 26, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 1, 3, 0, 0, 0, 0, 0, 0, 0, 3, 0, 1, 0, 0, 0, 2, 0, 0, 0, 4];
    assert_eq!(records::encode(&row).as_ref(), bytes);
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).unwrap();
    assert_eq!(records::decode(bytes, &bounds), Ok(row));
}
