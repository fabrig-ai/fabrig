//! Keys' bytes against the order Rust derives (protocol/store/README.md, 4.1, 9).
use super::fixtures;
use crate::keys;
use jig_domain_core_vocabulary::{CoreKey, Key};
use skein_lib::Rng;

#[test]
fn an_opening_key_is_the_escaped_worker_name_after_wires() {
    let key = keys::Own::Opening("w\0é".into());
    let expected = &[0, 5, b'w', 0, 255, 0xc3, 0xa9, 0, 0];
    assert_eq!(keys::encode_own(&key, 9).as_deref(), Some(expected.as_slice()));
    assert_eq!(keys::decode_own(expected), Some(key.clone()));
    assert_eq!(keys::encode_own(&key, 8), None);
    assert_eq!(keys::decode_own(&[0, 5, 255, 0, 0]), None, "worker names are UTF-8");
    assert_eq!(keys::decode_own(&[0, 5, b'w', 0, 0, 0]), None, "no trailing bytes");
    let names = ["", "a", "a\0", "a\0b", "aa", "w1", "w2", "é"];
    let mut previous = keys::encode_own(&keys::Own::Wires, 64).expect("wires key");
    for name in names {
        let key = keys::Own::Opening(name.into());
        let encoded = keys::encode_own(&key, 64).expect("opening key");
        assert!(encoded > previous);
        assert_eq!(keys::decode_own(&encoded), Some(key));
        previous = encoded;
    }
}

fn samples() -> core::ops::Range<u32> {
    0..500
}

#[test]
fn core_and_people_keys_sort_as_bytes_as_they_derive() {
    let mut rng = Rng::new(0x5701_e001);
    for _ in 0_u32..2_000 {
        let left = fixtures::key(&mut rng);
        let right = fixtures::key(&mut rng);
        let left_bytes = keys::encode(&left, fixtures::KEY_LIMIT).expect("within the key limit");
        let right_bytes = keys::encode(&right, fixtures::KEY_LIMIT).expect("within the key limit");
        assert_eq!(left.cmp(&right), left_bytes.cmp(&right_bytes), "{left:?} against {right:?}");
    }
}
#[test]
fn every_generated_key_decodes_to_itself() {
    let mut rng = Rng::new(0x5701_e002);
    for _ in samples() {
        let key = fixtures::key(&mut rng);
        let bytes = keys::encode(&key, fixtures::KEY_LIMIT).expect("within the key limit");
        assert_eq!(keys::decode(&bytes), Some(key));
    }
}
#[test]
fn own_keys_sort_as_bytes_as_they_derive() {
    let mut rng = Rng::new(0x5701_e003);
    for _ in 0_u32..2_000 {
        let left = fixtures::own(&mut rng);
        let right = fixtures::own(&mut rng);
        let left_bytes = keys::encode_own(&left, fixtures::KEY_LIMIT).expect("within the key limit");
        let right_bytes = keys::encode_own(&right, fixtures::KEY_LIMIT).expect("within the key limit");
        assert_eq!(left.cmp(&right), left_bytes.cmp(&right_bytes), "{left:?} against {right:?}");
    }
}
#[test]
fn every_generated_own_key_decodes_to_itself() {
    let mut rng = Rng::new(0x5701_e013);
    for _ in 0_u32..2_000 {
        let key = fixtures::own(&mut rng);
        let bytes = keys::encode_own(&key, fixtures::KEY_LIMIT).expect("within the key limit");
        assert_eq!(keys::decode_own(&bytes), Some(key));
    }
}
#[test]
fn a_key_with_bytes_left_over_does_not_decode() {
    assert_eq!(keys::decode(&[1, 0, 0, 0]), None);
    assert_eq!(keys::decode_own(&[0, 0, 0]), None);
}
#[test]
fn a_key_past_its_limit_is_none() {
    assert_eq!(keys::encode(&Key::Core(CoreKey::Deployment), 2), None);
    assert_eq!(keys::encode_own(&keys::Own::Format, 1), None);
}
#[test]
fn the_installations_key_names_no_account_and_an_accounts_key_reads_its_numbers() {
    let installation = keys::installation(fixtures::KEY_LIMIT).expect("within the key limit");
    assert_eq!(keys::decode(&installation), None);
    for (key, expected) in [
        (jig_domain_accounts::Key::Facts { account: 7 }, &[1, 4, 5, 0, 0, 0, 0, 0, 0, 0, 7][..]),
        (jig_domain_accounts::Key::Facts { account: u32::MAX }, &[1, 4, 5, 0, 0, 0, 0, 255, 255, 255, 255][..]),
    ] {
        let key = Key::Accounts(key);
        let bytes = keys::encode(&key, fixtures::KEY_LIMIT).expect("within key limit");
        assert_eq!(bytes.as_ref(), expected);
        assert_eq!(keys::decode(&bytes), Some(key));
    }
    let source = Key::Accounts(jig_domain_accounts::Key::Source { account: 3, number: 4 });
    let bytes = keys::encode(&source, fixtures::KEY_LIMIT).expect("within the key limit");
    assert_eq!(&*bytes, &[1, 4, 2, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 4]);
    let mut wide = bytes.clone();
    *wide.get_mut(10).expect("the account's low byte") = 0;
    *wide.get_mut(6).expect("past a u32") = 1;
    assert_eq!(keys::decode(&wide), None, "a number past the child's u32 names no source");
}
#[test]
fn an_application_key_carries_tag_two_and_strips_back() {
    let bytes = keys::application(&[0, 1, 255], 4).expect("at the key limit");
    assert_eq!(&*bytes, &[2, 0, 1, 255]);
    assert_eq!(keys::of_application(&bytes), Some(&[0, 1, 255][..]));
}
#[test]
fn an_application_key_past_its_limit_is_none() {
    assert_eq!(keys::application(&[0, 1, 255], 3), None);
}
#[test]
fn an_unknown_or_short_key_does_not_decode() {
    for bytes in [&[][..], &[1, 0, 255], &[1, 0, 2, 0], &[1, 2, 6], &[2, 0, 0]] {
        assert_eq!(keys::decode(bytes), None, "{bytes:?}");
    }
    for bytes in [&[][..], &[0, 255], &[0, 1, 0], &[1, 0]] {
        assert_eq!(keys::decode_own(bytes), None, "{bytes:?}");
    }
}
#[test]
fn an_application_key_must_have_its_owners_tag() {
    assert_eq!(keys::of_application(&[]), None);
    assert_eq!(keys::of_application(&[1, 0]), None);
}

#[test]
fn tasks_keys_sort_as_bytes_as_they_derive() {
    let mut rng = Rng::new(0x5701_e014);
    for _ in samples() {
        let left = Key::Tasks(fixtures::tasks_key(&mut rng));
        let right = Key::Tasks(fixtures::tasks_key(&mut rng));
        let left_bytes = keys::encode(&left, fixtures::KEY_LIMIT).expect("within the key limit");
        let right_bytes = keys::encode(&right, fixtures::KEY_LIMIT).expect("within the key limit");
        assert_eq!(left.cmp(&right), left_bytes.cmp(&right_bytes), "{left:?} against {right:?}");
    }
}
#[test]
fn notes_keys_with_shared_pattern_prefixes_sort_as_they_derive() {
    let mut rng = Rng::new(0x5701_e015);
    for _ in 0_u32..2_000 {
        let left = Key::Notes(fixtures::pattern_key(fixtures::pattern(&mut rng).segments));
        let right = Key::Notes(fixtures::pattern_key(fixtures::pattern(&mut rng).segments));
        let left_bytes = keys::encode(&left, fixtures::KEY_LIMIT).expect("within the key limit");
        let right_bytes = keys::encode(&right, fixtures::KEY_LIMIT).expect("within the key limit");
        assert_eq!(left.cmp(&right), left_bytes.cmp(&right_bytes), "{left:?} against {right:?}");
    }
}
#[test]
fn a_shorter_pattern_sorts_before_its_extension() {
    use alloc::boxed::Box;
    use jig_domain_notes as notes;
    for scope in [
        notes::Scope::Project { project: u32::MAX },
        notes::Scope::Goal { project: u32::MAX, goal: u64::MAX },
        notes::Scope::Resources {
            project: u32::MAX,
            connector: u16::MAX,
            pattern: notes::Pattern { segments: Box::new([]), last: notes::Last::Open(Box::new([])) },
        },
    ] {
        let project = Key::Notes(notes::Key::Line { scope, name: u64::MAX });
        let party = Key::Notes(notes::Key::Line { scope: notes::Scope::Party { party: 0 }, name: 0 });
        let bytes = keys::encode(&party, fixtures::KEY_LIMIT).expect("party key fits");
        assert_eq!(bytes.get(3), Some(&4));
        assert_eq!(keys::decode(&bytes), Some(party.clone()));
        assert!(project < party);
        assert!(keys::encode(&project, fixtures::KEY_LIMIT).unwrap() < bytes);
    }
    let short = Key::Notes(fixtures::pattern_key(Box::new([])));
    let longer = Key::Notes(fixtures::pattern_key(Box::new([Box::new([])])));
    let short_bytes = keys::encode(&short, fixtures::KEY_LIMIT).expect("within the key limit");
    let longer_bytes = keys::encode(&longer, fixtures::KEY_LIMIT).expect("within the key limit");
    assert!(short < longer, "derived list order puts an empty list first");
    assert!(short_bytes < longer_bytes, "end mark sorts before a segment mark");
}

#[test]
fn acted_keys_round_trip_after_the_ended_read_tag() {
    use jig_domain_people::{ActKey, Key as PeopleKey};
    let key = Key::People(PeopleKey::Acted(ActKey { connector: u16::MAX, key: [255; 16] }));
    let bytes = keys::encode(&key, fixtures::KEY_LIMIT).expect("bounded acted key");
    assert_eq!(bytes.get(2), Some(&8));
    assert_eq!(keys::decode(&bytes), Some(key.clone()));
    let ended = Key::People(PeopleKey::EndedRead { task: u64::MAX, party: u64::MAX });
    assert!(ended < key);
    assert!(keys::encode(&ended, fixtures::KEY_LIMIT).expect("bounded ended key") < bytes);
}

#[test]
fn settings_subfamilies_and_history_follow_their_domain_order() {
    use jig_domain_core_vocabulary::settings::{Key as SettingKey, Setting};
    let settings = [
        SettingKey::Default(Setting::Charter(1)),
        SettingKey::Default(Setting::Charter(256)),
        SettingKey::Default(Setting::Template(1)),
        SettingKey::Default(Setting::Policy(1)),
        SettingKey::Default(Setting::Rules),
        SettingKey::Default(Setting::Application { kind: 1, scope: 0, number: 1 }),
        SettingKey::Default(Setting::ProjectPolicy { project: 1 }),
        SettingKey::Default(Setting::Model(1)),
        SettingKey::Changed(Setting::Charter(1)),
        SettingKey::Changed(Setting::Model(1)),
        SettingKey::Held(1),
        SettingKey::Charter { charter: 1, revision: 2 },
        SettingKey::Proposal { proposer: jig_domain_people::Proposer::Task(1), proposal: 1 },
        SettingKey::Proposal { proposer: jig_domain_people::Proposer::Party(1), proposal: 1 },
        SettingKey::Extent,
        SettingKey::History(1),
        SettingKey::History(255),
        SettingKey::History(256),
        SettingKey::History(u32::MAX),
    ];
    let mut previous_key = Key::Core(CoreKey::EscalationDecision { task: 1, revision: 1 });
    let mut previous = keys::encode(&previous_key, fixtures::KEY_LIMIT).unwrap();
    for setting in settings {
        let key = Key::Core(CoreKey::Settings(setting));
        let bytes = keys::encode(&key, fixtures::KEY_LIMIT).unwrap();
        assert!(previous < bytes, "{setting:?} follows its predecessor");
        assert!(previous_key < key, "{key:?} follows {previous_key:?} in derived order");
        assert_eq!(keys::decode(&bytes), Some(key.clone()));
        previous_key = key;
        previous = bytes;
    }
}

#[test]
fn a_deletion_key_round_trips_under_tag_nine() {
    use jig_domain_people::Key as PeopleKey;
    for project in [0, 1, u32::MAX] {
        let key = Key::People(PeopleKey::Deletion(project));
        let bytes = keys::encode(&key, fixtures::KEY_LIMIT).expect("bounded deletion key");
        assert_eq!(bytes.get(2), Some(&9));
        assert_eq!(keys::decode(&bytes), Some(key.clone()));
        assert!(
            PeopleKey::Acted(jig_domain_people::ActKey { connector: u16::MAX, key: [255; 16] })
                < PeopleKey::Deletion(project)
        );
        assert!(PeopleKey::Deletion(project).live());
    }
}
