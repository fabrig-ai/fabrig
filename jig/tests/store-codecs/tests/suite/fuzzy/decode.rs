//! Arbitrary and mutated family bytes (protocol/store/records.md, section 1, and protocol/store/README.md, section 9).
use super::super::values;
use jig_protocol_store::tasks_v13::Record;
use jig_protocol_store::{accounts_v2, component_v3, notes_v1, people_v17};
use skein_lib::{Reader, Rng, length::len_u64};

fn notes(bytes: &[u8]) {
    if let Ok(scope) = notes_v1::Scope::decode(&notes_v1::CEILINGS, &mut Reader::new(bytes)) {
        assert_eq!(scope.encode(&notes_v1::CEILINGS).expect("decoded value encodes").as_ref(), bytes);
    }
    if let Ok(row) = notes_v1::Row::decode(&notes_v1::CEILINGS, &mut Reader::new(bytes)) {
        assert_eq!(row.encode(&notes_v1::CEILINGS).expect("decoded value encodes").as_ref(), bytes);
    }
}
fn component(bytes: &[u8]) {
    if let Ok(row) = component_v3::Format::decode(&component_v3::CEILINGS, &mut Reader::new(bytes)) {
        assert_eq!(row.encode(&component_v3::CEILINGS).expect("decoded value encodes").as_ref(), bytes);
    }
    if let Ok(row) = component_v3::Transcript::decode(&component_v3::CEILINGS, &mut Reader::new(bytes)) {
        assert_eq!(row.encode(&component_v3::CEILINGS).expect("decoded value encodes").as_ref(), bytes);
    }
    if let Ok(row) = component_v3::Attempt::decode(&component_v3::CEILINGS, &mut Reader::new(bytes)) {
        assert_eq!(row.encode(&component_v3::CEILINGS).expect("decoded value encodes").as_ref(), bytes);
    }
}
fn accounts(bytes: &[u8]) {
    if let Ok(row) = accounts_v2::Installation::decode(&accounts_v2::CEILINGS, &mut Reader::new(bytes)) {
        assert_eq!(row.encode(&accounts_v2::CEILINGS).expect("decoded value encodes").as_ref(), bytes);
    }
    if let Ok(row) = accounts_v2::Row::decode(&accounts_v2::CEILINGS, &mut Reader::new(bytes)) {
        assert_eq!(row.encode(&accounts_v2::CEILINGS).expect("decoded value encodes").as_ref(), bytes);
    }
}
fn arbitrary(seed: u64, check: fn(&[u8])) {
    let mut rng = Rng::new(seed);
    let mut buffer = [0_u8; 128];
    for _ in 0_u32..10_000 {
        let length = usize::try_from(rng.below(129)).expect("bounded length");
        for byte in buffer.get_mut(..length).expect("within buffer") {
            *byte = u8::try_from(rng.next_u64() & 0xff).expect("low byte");
        }
        check(buffer.get(..length).expect("within buffer"));
    }
}
fn mutated(text: &'static str, seed: u64, check: fn(&[u8])) {
    let values = skein_codegen::values(text);
    let mut fixed_values = values.fixed.clone();
    fixed_values.sort_by(|left, right| left.name.cmp(&right.name));
    let mut rng = Rng::new(seed);
    for fixed_value in fixed_values.into_iter().filter(|value| value.name.ends_with("_full.bin")) {
        let fixed_value = fixed_value.bytes;
        check(&fixed_value);
        for _ in 0_u32..100 {
            let mut bytes = fixed_value.clone();
            if !bytes.is_empty() {
                let index = usize::try_from(rng.below(len_u64(bytes.len()))).expect("index fits");
                *bytes.get_mut(index).expect("chosen byte") = u8::try_from(rng.next_u64() & 0xff).expect("low byte");
            }
            check(&bytes);
        }
    }
}
#[test]
fn arbitrary_notes_rows_never_panic_and_round_trip_when_they_decode() {
    arbitrary(0x5701_e014, notes);
}
#[test]
fn mutated_notes_fixed_rule_values_never_panic() {
    mutated(values::NOTES, 0x5702_e014, notes);
}
#[test]
fn arbitrary_component_rows_never_panic_and_round_trip_when_they_decode() {
    arbitrary(0x5703_e014, component);
}
#[test]
fn mutated_component_fixed_rule_values_never_panic() {
    mutated(values::COMPONENT, 0x5704_e014, component);
}
#[test]
fn arbitrary_accounts_rows_never_panic_and_round_trip_when_they_decode() {
    arbitrary(0x570a_e014, accounts);
}
#[test]
fn mutated_accounts_fixed_rule_values_never_panic() {
    mutated(values::ACCOUNTS, 0x570b_e014, accounts);
}

fn tasks(bytes: &[u8]) {
    use jig_protocol_store::tasks_v13;
    if let Ok(row) = tasks_v13::Row::decode(&tasks_v13::CEILINGS, &mut Reader::new(bytes)) {
        assert_eq!(row.encode(&tasks_v13::CEILINGS).expect("decoded row fits").as_ref(), bytes);
    }
}
#[test]
fn arbitrary_tasks_rows_never_panic_and_round_trip_when_they_decode() {
    arbitrary(0x5705_e014, tasks);
}
#[test]
fn mutated_tasks_fixed_rule_values_never_panic() {
    mutated(values::TASKS, 0x5706_e014, tasks);
}

fn people(bytes: &[u8]) {
    if let Ok(scope) = people_v17::NoteScope::decode(&people_v17::CEILINGS, &mut Reader::new(bytes)) {
        assert_eq!(scope.encode(&people_v17::CEILINGS).expect("decoded value encodes").as_ref(), bytes);
    }
    if let Ok(row) = people_v17::Row::decode(&people_v17::CEILINGS, &mut Reader::new(bytes)) {
        assert_eq!(row.encode(&people_v17::CEILINGS).expect("decoded value encodes").as_ref(), bytes);
    }
}

#[test]
fn arbitrary_people_rows_never_panic_and_round_trip_when_they_decode() {
    arbitrary(0x5701_e019, people);
}
#[test]
fn mutated_people_fixed_rule_values_never_panic() {
    mutated(values::PEOPLE, 0x5702_e019, people);
}

fn core(bytes: &[u8]) {
    use jig_protocol_store::core_v6;
    if let Ok(row) = core_v6::Row::decode(&core_v6::CEILINGS, &mut Reader::new(bytes)) {
        assert_eq!(row.encode(&core_v6::CEILINGS).expect("decoded value encodes").as_ref(), bytes);
    }
    if let Ok(row) = core_v6::ProjectionRecord::decode(&core_v6::CEILINGS, &mut Reader::new(bytes)) {
        assert_eq!(row.encode(&core_v6::CEILINGS).expect("decoded value encodes").as_ref(), bytes);
    }
}
#[test]
fn arbitrary_core_rows_never_panic_and_round_trip_when_they_decode() {
    arbitrary(0x5703_e019, core);
}
#[test]
fn mutated_core_fixed_rule_values_never_panic() {
    mutated(values::CORE, 0x5704_e019, core);
}

fn frames(bytes: &[u8]) {
    use jig_protocol_store::{codec, core_v6, notes_v1, tasks_v13, transcripts::frame};
    let limits = codec::View {
        settings: jig_protocol_store::settings_v3::CEILINGS,
        core: core_v6::CEILINGS,
        tasks: tasks_v13::CEILINGS,
        people: people_v17::CEILINGS,
        notes: notes_v1::CEILINGS,
    };
    if let Ok(turn) = frame::decode(bytes, &limits) {
        assert_eq!(frame::encode(&turn, &limits).expect("decoded frame fits").as_ref(), bytes);
    }
}
#[test]
fn arbitrary_frames_never_panic_and_round_trip_when_they_decode() {
    arbitrary(0x5701_e017, frames);
}
#[test]
fn mutated_and_short_frames_never_panic() {
    use jig_protocol_store::{codec, core_v6, notes_v1, tasks_v13, transcripts::frame};
    let limits = codec::View {
        settings: jig_protocol_store::settings_v3::CEILINGS,
        core: core_v6::CEILINGS,
        tasks: tasks_v13::CEILINGS,
        people: people_v17::CEILINGS,
        notes: notes_v1::CEILINGS,
    };
    let turn = core_v6::TurnRecord {
        opens_window: true,
        task: 11,
        attempt: 2,
        turn: 1,
        spent: 3,
        read: Some(4),
        at: 5,
        transcript: Box::from(b"turn bytes".as_slice()),
    };
    let body = turn.encode(&limits.core).expect("valid body");
    let turn = codec::core::read_turn(&body, &limits).expect("valid body");
    let bytes = frame::encode(&turn, &limits).expect("valid frame");
    frames(&bytes);
    for index in 0..bytes.len() {
        let mut changed = bytes.clone();
        *changed.get_mut(index).expect("chosen byte") ^= 1;
        frames(&changed);
        frames(bytes.get(..index).expect("short frame"));
    }
}

fn test_connector(bytes: &[u8]) {
    use jig_test_store::v2::{self, Record};
    if let Ok(row) = v2::Row::decode(&v2::CEILINGS, &mut Reader::new(bytes)) {
        assert_eq!(row.encode(&v2::CEILINGS).expect("decoded row").as_ref(), bytes);
    }
}
#[test]
fn arbitrary_test_connector_rows_never_panic_and_round_trip_when_they_decode() {
    arbitrary(0x5701_e018, test_connector);
}

#[test]
fn mutated_test_connector_fixed_rule_values_never_panic() {
    mutated(values::TEST_CONNECTOR, 0x5702_e018, test_connector);
}

fn settings(bytes: &[u8]) {
    use jig_protocol_store::settings_v3 as wire;
    if let Ok(row) = wire::Default::decode(&wire::CEILINGS, &mut Reader::new(bytes)) {
        assert_eq!(row.encode(&wire::CEILINGS).expect("decoded value encodes").as_ref(), bytes);
    }
    if let Ok(row) = wire::Changed::decode(&wire::CEILINGS, &mut Reader::new(bytes)) {
        assert_eq!(row.encode(&wire::CEILINGS).expect("decoded value encodes").as_ref(), bytes);
    }
    if let Ok(row) = wire::Revision::decode(&wire::CEILINGS, &mut Reader::new(bytes)) {
        assert_eq!(row.encode(&wire::CEILINGS).expect("decoded value encodes").as_ref(), bytes);
    }
    if let Ok(row) = wire::Extent::decode(&wire::CEILINGS, &mut Reader::new(bytes)) {
        assert_eq!(row.encode(&wire::CEILINGS).expect("decoded value encodes").as_ref(), bytes);
    }
    if let Ok(row) = wire::CharterVersion::decode(&wire::CEILINGS, &mut Reader::new(bytes)) {
        assert_eq!(row.encode(&wire::CEILINGS).expect("decoded value encodes").as_ref(), bytes);
    }
    if let Ok(row) = wire::Proposal::decode(&wire::CEILINGS, &mut Reader::new(bytes)) {
        assert_eq!(row.encode(&wire::CEILINGS).expect("decoded value encodes").as_ref(), bytes);
    }
}
#[test]
fn arbitrary_and_mutated_settings_records_never_panic_and_round_trip() {
    arbitrary(0x57aa_0191, settings);
    mutated(values::SETTINGS, 0x57aa_0192, settings);
}
