use super::fixtures;
use crate::keys;
use skein_lib::Rng;
use temper_engine_domain_forge::Key;

#[test]
fn encoded_keys_sort_as_the_derived_order() {
    let mut random = Rng::new(0x0401);
    let mut previous = fixtures::key(&mut random);
    for _ in 0_u32..2_000 {
        let next = fixtures::key(&mut random);
        let encoded = keys::encode(&previous, u32::MAX).cmp(&keys::encode(&next, u32::MAX));
        assert_eq!(encoded, previous.cmp(&next), "{previous:?} against {next:?}");
        previous = next;
    }
}

#[test]
fn a_decoded_key_is_the_key_encoded() {
    let mut random = Rng::new(0x0401);
    for _ in 0_u32..2_000 {
        let key = fixtures::key(&mut random);
        let bytes = keys::encode(&key, u32::MAX).expect("a drawn key fits");
        assert_eq!(keys::decode(&bytes, &fixtures::LIMITS), Some(key));
    }
}

#[test]
fn a_shorter_branch_sorts_before_its_extensions() {
    let shorter = fixtures::branch(Box::new([Box::from([0, 1, 255])]));
    let longer = fixtures::branch(Box::new([Box::from([0, 1, 255]), Box::new([])]));
    assert!(keys::encode(&shorter, u32::MAX) < keys::encode(&longer, u32::MAX));
}

#[test]
fn a_key_decoder_refuses_each_malformed_key() {
    let mut what = skein_kv::KeyWriter::new(u32::MAX);
    what.tag(3).u64(1).u64(2).tag(4);
    let mut forge = skein_kv::KeyWriter::new(u32::MAX);
    forge.tag(2).u64(65_536).u64(2);
    let mut trailing = skein_kv::KeyWriter::new(u32::MAX);
    trailing.tag(4).u64(1).tag(0);
    let mut branch = skein_kv::KeyWriter::new(u32::MAX);
    branch.tag(3).u64(1).u64(2).tag(1);
    for _ in 0_u32..129 {
        branch.tag(1).bytes(&[]);
    }
    branch.tag(0);
    let mut making_forge = skein_kv::KeyWriter::new(u32::MAX);
    making_forge.tag(keys::MAKING).u64(65_536).bytes(b"owner").bytes(b"repo");
    let mut making_trailing = skein_kv::KeyWriter::new(u32::MAX);
    making_trailing.tag(keys::MAKING).u64(1).bytes(b"owner").bytes(b"repo").tag(0);
    for (name, bytes) in [
        ("making forge width", making_forge.finish().expect("small key")),
        ("making trailing byte", making_trailing.finish().expect("small key")),
        ("family tag", Box::from([19])),
        ("What tag", what.finish().expect("small key")),
        ("forge width", forge.finish().expect("small key")),
        ("trailing byte", trailing.finish().expect("small key")),
        ("branch count", branch.finish().expect("small key")),
        ("cut key", Box::from([4, 0, 0, 0, 0, 0, 0, 0])),
    ] {
        assert_eq!(keys::decode(&bytes, &fixtures::LIMITS), None, "{name}");
    }
}

#[test]
fn every_forge_key_lies_in_the_interval_and_no_core_key_does() {
    let interval = crate::interval();
    let mut random = Rng::new(0x0402);
    for _ in 0_u32..500 {
        let key = fixtures::key(&mut random);
        let relative = keys::encode(&key, u32::MAX).expect("a drawn key fits");
        let tagged = jig_protocol_store::keys::application(&relative, u32::MAX).expect("tagged");
        assert!(fixtures::within(&interval, &tagged), "{key:?}");
        assert!(temper_engine_domain::Range::Forge.contains(&temper_engine_domain::Key::Forge(key)));
    }
    for key in fixtures::core_keys() {
        let tagged = jig_protocol_store::keys::encode(&key, u32::MAX).expect("a core key fits");
        assert!(!fixtures::within(&interval, &tagged), "{key:?}");
        assert!(!temper_engine_domain::Range::Forge.contains(&temper_engine_domain::Key::Core(key)));
    }
}

#[test]
fn branch_empty_segments_and_binding_escapes_fit_the_calculated_key_bound() {
    // One family byte, an eight-byte forge, and two escaped 128-byte names with terminators.
    const MAKING_KEY_BYTES: u32 = 525;
    let mut segments: skein_lib::List<Box<[u8]>> = skein_lib::List::with_capacity(fixtures::LIMITS.name_bytes);
    for _ in 0..fixtures::LIMITS.name_bytes {
        segments.push(Box::new([])).expect("bounded branch");
    }
    let key = fixtures::branch(segments.into_boxed());
    let bytes = keys::encode(&key, u32::MAX).expect("largest key fits");
    assert_eq!(bytes.len(), 403);
    assert_eq!(keys::decode(&bytes, &fixtures::LIMITS), Some(key));
    let key = Key::Binding(Box::new([0; 254]));
    let bytes = keys::encode(&key, 511).expect("escaped binding bound");
    assert_eq!(bytes.len(), 511);
    assert_eq!(keys::encode(&key, 510), None);
    assert_eq!(keys::decode(&bytes, &fixtures::LIMITS), Some(key));
    let name_bytes = usize::try_from(fixtures::LIMITS.name_bytes).expect("fixture name bound");
    assert_eq!(keys::largest(&fixtures::LIMITS), Some(MAKING_KEY_BYTES));
    let key = Key::Making(temper_engine_domain_forge::MakingKey {
        forge: u16::MAX,
        org: skein_lib::bytes::zeroed(name_bytes),
        name: skein_lib::bytes::zeroed(name_bytes),
    });
    let bytes = keys::encode(&key, MAKING_KEY_BYTES).expect("escaped making bound");
    assert_eq!(bytes.first(), Some(&35));
    assert_eq!(bytes.len(), usize::try_from(MAKING_KEY_BYTES).expect("fixture key bound"));
    assert_eq!(keys::encode(&key, MAKING_KEY_BYTES - 1), None);

    assert_eq!(keys::decode(&bytes, &fixtures::LIMITS), Some(key));
}

#[test]
fn application_keys_follow_the_release_family_and_round_trip() {
    use temper_engine_domain_forge::Key;
    let previous = keys::encode(&Key::Release(u64::MAX), u32::MAX).expect("release key");
    for token in [0, 1, u64::MAX] {
        let key = Key::Application(token);
        let bytes = keys::encode(&key, u32::MAX).expect("application key");
        assert_eq!(bytes[0], 15);
        assert_eq!(keys::decode(&bytes, &fixtures::LIMITS), Some(key));
        assert!(bytes > previous);
    }
}

#[test]
fn writer_keys_follow_application_and_take_every_forge_number() {
    use temper_engine_domain_forge::Key;
    let previous = keys::encode(&Key::Application(u64::MAX), u32::MAX).expect("application key");
    for forge in [0, 1, u16::MAX] {
        let key = Key::Writer(forge);
        let bytes = keys::encode(&key, u32::MAX).expect("writer key");
        assert_eq!(bytes[0], 16);
        assert_eq!(bytes.len(), 9);
        assert_eq!(keys::decode(&bytes, &fixtures::LIMITS), Some(key));
        assert!(bytes > previous);
    }
    let mut too_wide = skein_kv::KeyWriter::new(u32::MAX);
    too_wide.tag(16).u64(65_536);
    assert_eq!(keys::decode(&too_wide.finish().expect("small key"), &fixtures::LIMITS), None);
}

#[test]
fn binding_keys_use_family_eighteen_and_bounded_addresses() {
    for address in [b"a@b".as_slice(), b"sam@example.com", b"Sam@example.com", b"a@@b", b"no-at", b"", b"a\0b"] {
        let key = Key::Binding(address.into());
        let bytes = keys::encode(&key, u32::MAX).unwrap();
        assert_eq!(bytes[0], keys::BINDING);
        assert_eq!(keys::decode(&bytes, &fixtures::LIMITS), Some(key));
        let mut trailing = skein_kv::KeyWriter::new(u32::MAX);
        trailing.tag(keys::BINDING).bytes(address).tag(0);
        assert_eq!(keys::decode(&trailing.finish().expect("small key"), &fixtures::LIMITS), None);
    }
    let address = [0; 255];
    assert_eq!(keys::encode(&Key::Binding(Box::new(address)), u32::MAX), None);
    let mut oversized = skein_kv::KeyWriter::new(u32::MAX);
    oversized.tag(keys::BINDING).bytes(&address);
    assert_eq!(keys::decode(&oversized.finish().unwrap(), &fixtures::LIMITS), None);
    for bytes in [b"\x12\0".as_slice(), b"\x12\0\x01", b"\x12x", b"\xff\0\0"] {
        assert_eq!(keys::decode(bytes, &fixtures::LIMITS), None);
    }
}

#[test]
fn flight_keys_name_task_then_entry_under_family_twenty_one() {
    const KEY_BYTES: u32 = 17; // Family byte and two u64s (protocol/store.md, 2).
    let key = Key::Flight { task: 7, entry: 11 };
    let bytes = keys::encode(&key, KEY_BYTES).expect("exact flight key room");
    assert_eq!(bytes.as_ref(), &[21, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 11]);
    assert_eq!(keys::decode(&bytes, &fixtures::LIMITS), Some(key));
    assert_eq!(keys::flight(&bytes), Some((7, 11)));
    assert_eq!(keys::encode(&Key::Flight { task: 7, entry: 11 }, KEY_BYTES - 1), None);
    let mut writer = skein_lib::Writer::new(usize::try_from(KEY_BYTES + 1).expect("small key"));
    writer.put(&bytes).expect("key room");
    writer.put(&[0]).expect("trailing byte room");
    let trailing = writer.finish();
    assert_eq!(keys::flight(&trailing), None);
    assert_eq!(keys::flight(&trailing[..16]), None);
    assert_eq!(keys::flight(&[20]), None);
}

#[test]
fn forge_and_playground_keys_follow_receipt_and_refuse_malformed_bytes() {
    for number in [0, 1, u16::MAX] {
        let key = Key::Forge(number);
        let bytes = keys::encode(&key, 9).expect("forge key");
        assert_eq!(bytes[0], 25);
        assert_eq!(bytes.len(), 9);
        assert_eq!(keys::decode(&bytes, &fixtures::LIMITS), Some(key));
        assert_eq!(keys::encode(&Key::Forge(number), 8), None);
        let mut trailing = skein_lib::Writer::new(bytes.len() + 1);
        trailing.put(&bytes).expect("key room");
        trailing.put(&[0]).expect("trailing byte room");
        assert_eq!(keys::decode(&trailing.finish(), &fixtures::LIMITS), None);
    }
    for (key, tag) in [(Key::ForgeNumber, 26), (Key::Playground, 27)] {
        let bytes = keys::encode(&key, 1).expect("singleton key");
        assert_eq!(bytes.as_ref(), &[tag]);
        assert_eq!(keys::decode(&bytes, &fixtures::LIMITS), Some(key.clone()));
        assert_eq!(keys::encode(&key, 0), None);
        assert_eq!(keys::decode(&[tag, 0], &fixtures::LIMITS), None);
    }
    let mut too_wide = skein_kv::KeyWriter::new(9);
    too_wide.tag(keys::FORGE).u64(65_536);
    assert_eq!(keys::decode(&too_wide.finish().expect("small key"), &fixtures::LIMITS), None);
    assert_eq!(keys::decode(&[keys::FORGE], &fixtures::LIMITS), None);
    assert_eq!(keys::decode(&[28], &fixtures::LIMITS), None);
}

#[test]
fn removed_family_bytes_are_reserved() {
    for family in [17, 19] {
        let mut key = skein_kv::KeyWriter::new(u32::MAX);
        key.tag(family).u64(1);
        assert_eq!(keys::decode(&key.finish().unwrap(), &fixtures::LIMITS), None);
    }
}

#[test]
fn setting_keys_use_family_twenty_and_refuse_unknown_tags_widths_and_trailing_bytes() {
    use temper_engine_domain_forge::settings::{SettingKey, SettingSlot};
    for kind in 1..=3 {
        for slot in [
            SettingSlot::Default,
            SettingSlot::Change,
            SettingSlot::Proposal(u64::MAX),
            SettingSlot::Replaced(u64::MAX),
        ] {
            let key = Key::Setting(SettingKey { kind, scope: u32::MAX, number: u32::MAX, slot });
            let bytes = keys::encode(&key, u32::MAX).unwrap();
            assert_eq!(bytes[0], keys::SETTING);
            let length = match slot {
                SettingSlot::Default | SettingSlot::Change => 26,
                SettingSlot::Proposal(_) | SettingSlot::Replaced(_) => 34,
            };
            assert_eq!(bytes.len(), length);
            assert_eq!(keys::decode(&bytes, &fixtures::LIMITS), Some(key.clone()));
            assert_eq!(keys::encode(&key, u32::try_from(length - 1).unwrap()), None);
            for end in 0..length {
                assert_eq!(keys::decode(&bytes[..end], &fixtures::LIMITS), None);
            }
            let mut trailing = skein_lib::Writer::new(length + 1);
            trailing.put(&bytes).unwrap();
            trailing.put(&[0]).unwrap();
            assert_eq!(keys::decode(&trailing.finish(), &fixtures::LIMITS), None);
        }
    }
    for (kind, scope, number, slot) in [
        (0, 0, 1, 0),
        (4, 0, 1, 0),
        (1_u64 << 32_u8, 0, 1, 0),
        (1, 1_u64 << 32_u8, 1, 0),
        (1, 0, 1_u64 << 32_u8, 0),
        (1, 0, 1, 4),
    ] {
        let mut writer = skein_kv::KeyWriter::new(u32::MAX);
        writer.tag(keys::SETTING).u64(kind).u64(scope).u64(number).tag(slot);
        assert_eq!(keys::decode(&writer.finish().unwrap(), &fixtures::LIMITS), None);
    }
}

#[test]
fn dismissed_keys_use_family_thirty_four_and_round_trip() {
    use temper_engine_domain_forge::PersonFact;
    for (party, project) in [(0, 0), (7, 3), (u64::MAX, u32::MAX)] {
        let key = Key::Dismissed { party, what: PersonFact::ToBegin { project } };
        let bytes = keys::encode(&key, u32::MAX).expect("dismissed key fits");
        let mut expected = skein_kv::KeyWriter::new(u32::MAX);
        expected.tag(34).u64(party).tag(0).u64(u64::from(project));
        assert_eq!(bytes, expected.finish().expect("fixed key fits"));
        assert_eq!(keys::decode(&bytes, &fixtures::LIMITS), Some(key));
        let mut previous = PersonFact::GoalFrom { project: u32::MAX, chat: u64::MAX, goal: u64::MAX, at: u64::MAX };
        let subjects = temper_protocol_reaches::Kind::ALL.map(temper_protocol_reaches::Subject::Kind);
        for subject in subjects
            .into_iter()
            .chain([temper_protocol_reaches::Subject::Set(0), temper_protocol_reaches::Subject::Set(u8::MAX)])
        {
            let what = PersonFact::KeepAsking { project, subject, at: u64::MAX };
            let key = Key::Dismissed { party, what };
            let bytes = keys::encode(&key, u32::MAX).expect("keep asking key");
            assert_eq!(keys::decode(&bytes, &fixtures::LIMITS), Some(key));
            assert!(previous < what);
            assert!(keys::encode(&Key::Dismissed { party, what: previous }, u32::MAX).unwrap() < bytes);
            previous = what;
        }
    }
}

#[test]
fn dismissed_key_decoder_refuses_unknown_kind_width_and_trailing_bytes() {
    let mut unknown = skein_kv::KeyWriter::new(u32::MAX);
    unknown.tag(34).u64(7).tag(3).u64(3);
    let mut wide = skein_kv::KeyWriter::new(u32::MAX);
    wide.tag(34).u64(7).tag(0).u64(4_294_967_296);
    let mut trailing = skein_kv::KeyWriter::new(u32::MAX);
    trailing.tag(34).u64(7).tag(0).u64(3).tag(0);
    for (name, bytes) in [("unknown kind", unknown), ("project width", wide), ("trailing bytes", trailing)] {
        assert_eq!(keys::decode(&bytes.finish().expect("fixed key fits"), &fixtures::LIMITS), None, "{name}");
    }
}

#[test]
fn landing_log_keys_use_family_thirty_and_repository_order() {
    let repository = skein_forge::api::Repository { forge: 1, repository: 2 };
    let key = Key::LandingLog { repository };
    let bytes = keys::encode(&key, u32::MAX).unwrap();
    let mut expected = skein_kv::KeyWriter::new(u32::MAX);
    expected.tag(30).u64(1).u64(2);
    assert_eq!(bytes, expected.finish().unwrap());
    assert_eq!(keys::decode(&bytes, &fixtures::LIMITS), Some(key.clone()));
    assert!(Key::Verification { repository, commit: [255; 32] } < key);
    assert!(key < Key::Dismissed { party: 0, what: temper_engine_domain_forge::PersonFact::ToBegin { project: 0 } });
}

#[test]
fn person_facts_sort_by_party_then_kind_and_fields() {
    use temper_engine_domain_forge::PersonFact;
    let mut previous = None;
    for (party, what) in [
        (0, PersonFact::GoalFrom { project: u32::MAX, chat: u64::MAX, goal: u64::MAX, at: u64::MAX }),
        (7, PersonFact::ToBegin { project: 0 }),
        (7, PersonFact::ToBegin { project: u32::MAX }),
        (7, PersonFact::GoalFrom { project: 0, chat: 0, goal: 0, at: 0 }),
        (7, PersonFact::GoalFrom { project: 0, chat: 0, goal: 0, at: u64::MAX }),
        (7, PersonFact::GoalFrom { project: 0, chat: 0, goal: u64::MAX, at: u64::MAX }),
        (7, PersonFact::GoalFrom { project: 0, chat: 1, goal: 0, at: 0 }),
        (7, PersonFact::GoalFrom { project: 1, chat: 0, goal: 0, at: 0 }),
        (u64::MAX, PersonFact::ToBegin { project: 0 }),
    ] {
        let key = Key::Dismissed { party, what };
        let bytes = keys::encode(&key, u32::MAX).unwrap();
        assert_eq!(keys::decode(&bytes, &fixtures::LIMITS), Some(key.clone()));
        if let Some((prior_key, prior_bytes)) = previous {
            assert!(key > prior_key);
            assert!(bytes > prior_bytes);
        }
        previous = Some((key, bytes));
    }
}

#[test]
fn status_keys_use_families_thirty_six_and_thirty_seven_in_order() {
    for (party, project) in [(0, 0), (7, 3), (u64::MAX, u32::MAX)] {
        let facts = Key::ProjectFacts { project };
        let looked = Key::Looked { party, project };
        let mut expected_facts = skein_kv::KeyWriter::new(u32::MAX);
        expected_facts.tag(36).u64(u64::from(project));
        let mut expected_looked = skein_kv::KeyWriter::new(u32::MAX);
        expected_looked.tag(37).u64(party).u64(u64::from(project));
        for (key, expected) in [(facts.clone(), expected_facts), (looked.clone(), expected_looked)] {
            let bytes = keys::encode(&key, u32::MAX).unwrap();
            assert_eq!(bytes, expected.finish().unwrap());
            assert_eq!(keys::decode(&bytes, &fixtures::LIMITS), Some(key));
        }
        assert!(
            Key::Making(temper_engine_domain_forge::MakingKey {
                forge: u16::MAX,
                org: Box::new([]),
                name: Box::new([])
            }) < facts
        );
        assert!(facts < looked);
    }
    let earlier = Key::Looked { party: 1, project: u32::MAX };
    let later = Key::Looked { party: 2, project: 0 };
    assert!(earlier < later);
    assert!(keys::encode(&earlier, u32::MAX) < keys::encode(&later, u32::MAX));
}

#[test]
fn status_keys_refuse_project_width_truncation_and_trailing_bytes() {
    for family in [36, 37] {
        for (project, trailing) in [(4_294_967_296, false), (0, true)] {
            let mut writer = skein_kv::KeyWriter::new(u32::MAX);
            writer.tag(family);
            if family == 37 {
                writer.u64(7);
            }
            writer.u64(project);
            if trailing {
                writer.tag(0);
            }
            assert_eq!(keys::decode(&writer.finish().unwrap(), &fixtures::LIMITS), None);
        }
        assert_eq!(keys::decode(&[family], &fixtures::LIMITS), None);
    }
}

#[test]
fn away_and_decided_keys_keep_their_family_and_field_order() {
    for (party, project, number) in [(0, 0, 0), (7, 3, 2), (u64::MAX, u32::MAX, u32::MAX)] {
        let mut away = skein_kv::KeyWriter::new(u32::MAX);
        away.tag(38).u64(party).u64(u64::from(project));
        let mut decided = skein_kv::KeyWriter::new(u32::MAX);
        decided.tag(39).u64(u64::from(project)).u64(u64::from(number));
        for (key, expected) in [(Key::Away { party, project }, away), (Key::Decided { project, number }, decided)] {
            let bytes = keys::encode(&key, u32::MAX).unwrap();
            assert_eq!(bytes, expected.finish().unwrap());
            assert_eq!(keys::decode(&bytes, &fixtures::LIMITS), Some(key.clone()));
            assert_eq!(keys::encode(&key, u32::try_from(bytes.len() - 1).unwrap()), None);
        }
    }
    for (earlier, later) in [
        (Key::Looked { party: u64::MAX, project: u32::MAX }, Key::Away { party: 0, project: 0 }),
        (Key::Away { party: u64::MAX, project: u32::MAX }, Key::Decided { project: 0, number: 0 }),
        (Key::Away { party: 1, project: u32::MAX }, Key::Away { party: 2, project: 0 }),
        (Key::Decided { project: 1, number: u32::MAX }, Key::Decided { project: 2, number: 0 }),
    ] {
        assert!(earlier < later);
        assert!(keys::encode(&earlier, u32::MAX) < keys::encode(&later, u32::MAX));
    }
}
#[test]
fn away_and_decided_keys_refuse_truncation_widths_and_trailing_bytes() {
    for (family, first, second) in [(38, 7, 4_294_967_296), (39, 4_294_967_296, 0), (39, 0, 4_294_967_296)] {
        let mut writer = skein_kv::KeyWriter::new(u32::MAX);
        writer.tag(family).u64(first).u64(second);
        assert_eq!(keys::decode(&writer.finish().unwrap(), &fixtures::LIMITS), None);
    }
    for family in [38, 39] {
        let mut writer = skein_kv::KeyWriter::new(u32::MAX);
        writer.tag(family).u64(7).u64(3);
        let bytes = writer.finish().unwrap();
        for len in 0..bytes.len() {
            assert_eq!(keys::decode(&bytes[..len], &fixtures::LIMITS), None);
        }
        let mut trailing = skein_kv::KeyWriter::new(u32::MAX);
        trailing.tag(family).u64(7).u64(3).tag(0);
        assert_eq!(keys::decode(&trailing.finish().unwrap(), &fixtures::LIMITS), None);
    }
}

#[test]
fn findings_keys_and_topic_use_the_reserved_bytes_and_order() {
    for (project, number) in [(0, 0), (1, u32::MAX), (u32::MAX, u32::MAX)] {
        let key = Key::Finding { project, number };
        let mut writer = skein_kv::KeyWriter::new(u32::MAX);
        writer.tag(31).u64(u64::from(project)).u64(u64::from(number));
        let bytes = keys::encode(&key, u32::MAX).unwrap();
        assert_eq!(bytes, writer.finish().unwrap());
        assert_eq!(keys::decode(&bytes, &fixtures::LIMITS), Some(key));
        let topic =
            Key::Subscription { task: u64::MAX, topic: temper_engine_domain_forge::Topic::Findings { project } };
        let encoded = keys::encode(&topic, u32::MAX).unwrap();
        assert_eq!(keys::decode(&encoded, &fixtures::LIMITS), Some(topic));
        for len in 0..bytes.len() {
            assert_eq!(keys::decode(&bytes[..len], &fixtures::LIMITS), None);
        }
        let mut trailing = skein_kv::KeyWriter::new(u32::MAX);
        trailing.tag(31).u64(u64::from(project)).u64(u64::from(number)).tag(0);
        assert_eq!(keys::decode(&trailing.finish().unwrap(), &fixtures::LIMITS), None);
    }
    assert!(Key::Finding { project: 1, number: u32::MAX } < Key::Finding { project: 2, number: 0 });
    for (project, number) in [(u64::from(u32::MAX) + 1, 0), (0, u64::from(u32::MAX) + 1)] {
        let mut writer = skein_kv::KeyWriter::new(u32::MAX);
        writer.tag(31).u64(project).u64(number);
        assert_eq!(keys::decode(&writer.finish().unwrap(), &fixtures::LIMITS), None);
    }
}
