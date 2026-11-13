//! Main's stored additions join the baseline catch-up (protocol/baseline.md, 9).
use skein_codec::patch::{Capacity, Patcher, Step};
use skein_codec::{
    schema::Schema,
    value::{Node, Value},
};
use skein_lib::bytes;

pub(super) fn patched(version: u16, patch: usize, root: &[u8], bytes: Box<[u8]>) -> Box<[u8]> {
    patched_with(version, patch, root, bytes, &mut crate::patches::Context::default())
}
fn patched_with(
    version: u16,
    patch: usize,
    root: &[u8],
    bytes: Box<[u8]>,
    context: &mut crate::patches::Context,
) -> Box<[u8]> {
    let texts = crate::patches::kept::temper_store::TEXTS;
    let mut patcher = Patcher::new(Capacity {
        versions: u32::try_from(texts.len()).expect("kept version count"),
        patches: 1,
        text: skein_codec::schema::MAX_TEXT,
        bytes: 131_072,
        nodes: 16_384,
    });
    for text in texts {
        patcher.keep(text.as_bytes()).expect("kept schema");
    }
    let root = patcher.schema(version).expect("kept schema").find(root).expect("old record");
    let plan = patcher.plan(crate::patches::PATCHES[patch].as_bytes()).expect("valid plan").expect("family moves");
    patcher.start(plan, root, bytes).expect("baseline record");
    loop {
        match patcher.next().expect("patch step") {
            Step::Written { site, at } => {
                let (old, new) = patcher.parts();
                crate::patches::written(site, old, new, at, context).expect("written conversion");
            }
            Step::Held { at, .. } => {
                let (old, new) = patcher.parts();
                new.carry(old, at).expect("unchanged held field");
            }
            Step::Done => break,
        }
    }
    patcher.finish().expect("patched record").into()
}

fn current(root: &[u8], mut bytes: Box<[u8]>) -> Box<[u8]> {
    for (version, patch) in
        [(11, 0), (12, 1), (13, 2), (14, 3), (15, 4), (16, 5), (17, 6), (18, 7), (19, 8), (20, 9), (21, 10)]
    {
        bytes = patched(version, patch, root, bytes);
    }
    bytes
}

pub(super) fn kept_golden(version: u16, name: &str) -> Box<[u8]> {
    for text in crate::patches::kept::temper_store::TEXTS {
        let schema = skein_codegen::parse(text).expect("kept schema");
        if schema.version != version {
            continue;
        }
        for golden in skein_codegen::generate(&schema).goldens {
            if golden.name == name {
                return golden.bytes.into_boxed_slice();
            }
        }
    }
    unreachable!("kept golden")
}

// Select by the kept text's header, not its position in TEXTS.
pub(super) fn kept_schema(version: u16) -> Schema {
    let header =
        bytes::concat(&[b"family temper-store ", skein_lib::Decimal::of(u64::from(version)).as_bytes(), b"\n"]);
    for text in crate::patches::kept::temper_store::TEXTS {
        if text.as_bytes().starts_with(&header) {
            return skein_codec::schema::parse(text.as_bytes()).expect("kept schema");
        }
    }
    panic!("kept version")
}

pub(super) fn kept_value(schema: &Schema, root: &[u8], bytes: Box<[u8]>) -> (Value, Node) {
    let mut value = Value::with_room(16_384);
    let row = value.decode(schema, schema.find(root).expect("kept root"), bytes).expect("kept record");
    (value, row)
}

pub(super) fn items(value: &Value, list: Node) -> Box<[Node]> {
    let mut items = value.items(list).expect("list");
    let mut nodes = skein_lib::List::with_capacity(65_536);
    while let Some(node) = value.next_item(&mut items) {
        nodes.push(node).expect("kept list bound");
    }
    nodes.into_boxed()
}

#[test]
fn a_baseline_adopted_row_has_no_contexts() {
    let adopted = kept_golden(11, "record_adopted_full.bin");
    let before = bytes::concat(&[&[0, 11, 2], &adopted]); // Row version, repository variant.
    let bytes = current(b"Row", before);
    let row = crate::v22::Row::decode(&crate::v22::CEILINGS, &mut skein_lib::Reader::new(&bytes)).expect("current row");
    let crate::v22::Value::Repository(record) = row.into_parts().value else { panic!("adopted repository") };
    assert!(record.contexts().is_empty());
    assert_eq!(record.project(), u32::MAX);
    assert!(record.home());
}

#[test]
fn a_foreman_with_a_task_becomes_retired() {
    let before = kept_golden(11, "enum_stands_foreman_full.bin");
    let bytes = current(b"Stands", before);
    let value =
        crate::v22::Stands::decode(&crate::v22::CEILINGS, &mut skein_lib::Reader::new(&bytes)).expect("current stands");
    assert_eq!(value, crate::v22::Stands::Retired);
}

#[test]
fn a_v13_closer_upgrades_with_no_question_relay() {
    let before = kept_golden(13, "record_closer_full.bin");
    let bytes = patched(13, 2, b"Closer", before);
    let bytes = patched(14, 3, b"Closer", bytes);
    let bytes = patched(15, 4, b"Closer", bytes);
    let bytes = patched(16, 5, b"Closer", bytes);
    let bytes = patched(17, 6, b"Closer", bytes);
    let bytes = patched(18, 7, b"Closer", bytes);
    let bytes = patched(19, 8, b"Closer", bytes);
    let row = crate::v22::Closer::decode(&crate::v22::CEILINGS, &mut skein_lib::Reader::new(&bytes))
        .expect("upgraded closer");
    assert_eq!(row.relay(), &None);
    assert_eq!(row.task(), u64::MAX);
    assert_eq!(row.goal(), u64::MAX);
    assert!(row.done());
}

#[test]
fn a_v16_person_owned_follow_keeps_its_bytes_and_owner() {
    let before = kept_golden(16, "record_follow_full.bin");
    let bytes = patched(16, 5, b"Follow", before.clone());
    assert_eq!(bytes, before, "adding task owners leaves person owners unchanged");
    let follow = crate::v22::Follow::decode(&crate::v22::CEILINGS, &mut skein_lib::Reader::new(&bytes)).unwrap();
    let crate::v22::FollowOwner::Person(owner) = follow.owner() else { panic!("old person owner") };
    assert_eq!(owner.person(), u64::MAX);
}

#[test]
fn a_v14_closer_keeps_its_question_relay_through_r2() {
    let before = kept_golden(14, "record_closer_full.bin");
    let bytes = patched(14, 3, b"Closer", before.clone());
    assert_eq!(bytes, before, "R2 only adds declarations and variants");
    let bytes = patched(15, 4, b"Closer", bytes);
    let bytes = patched(16, 5, b"Closer", bytes);
    let bytes = patched(17, 6, b"Closer", bytes);
    let bytes = patched(18, 7, b"Closer", bytes);
    let bytes = patched(19, 8, b"Closer", bytes);
    let closer =
        crate::v22::Closer::decode(&crate::v22::CEILINGS, &mut skein_lib::Reader::new(&bytes)).expect("current closer");
    assert!(closer.relay().is_some());
}

const V15_LANDING_BYTES: u32 = 512;

fn v15_landing(place: u8, held: bool, member: bool) -> Box<[u8]> {
    let mut bytes = skein_lib::Intake::with_capacity(V15_LANDING_BYTES);
    bytes.append(&61_u64.to_be_bytes()).expect("bounded landing fixture");
    bytes.append(&7_u32.to_be_bytes()).expect("bounded landing fixture");
    bytes.append(&[place]).expect("bounded landing fixture");
    if place == 3 || place == 4 {
        bytes.append(&123_u64.to_be_bytes()).expect("bounded landing fixture");
    }
    bytes.append(&[u8::from(held)]).expect("bounded landing fixture");
    if held {
        bytes.append(&0_u16.to_be_bytes()).expect("bounded landing fixture");
        bytes.append(&0_u16.to_be_bytes()).expect("bounded landing fixture");
    }
    bytes.append(&u32::from(member).to_be_bytes()).expect("bounded landing fixture");
    if member {
        bytes.append(&62_u64.to_be_bytes()).expect("bounded landing fixture");
        bytes.append(&1_u16.to_be_bytes()).expect("bounded landing fixture");
        bytes.append(&2_u32.to_be_bytes()).expect("bounded landing fixture");
        bytes.append(&10_u64.to_be_bytes()).expect("bounded landing fixture");
        for commit in [1_u8, 2, 3] {
            bytes.append(&[1]).expect("bounded landing fixture");
            bytes.append(&[commit; 32]).expect("bounded landing fixture");
        }
        bytes.append(&1_u32.to_be_bytes()).expect("bounded landing fixture");
        bytes.append(&[2]).expect("bounded landing fixture"); // dispatched proof
        bytes.append(&[4_u8; 32]).expect("bounded landing fixture");
        bytes.append(&2_u16.to_be_bytes()).expect("bounded landing fixture");
        bytes.append(&[1]).expect("bounded landing fixture"); // pending posting
        bytes.append(&3_u32.to_be_bytes()).expect("bounded landing fixture");
        bytes.append(&[1]).expect("bounded landing fixture");
        bytes.append(&2_u16.to_be_bytes()).expect("bounded landing fixture"); // asked preparation
        bytes.append(&[u8::from(place == 5)]).expect("bounded landing fixture");
        if place == 5 {
            bytes.append(&[3_u8; 32]).expect("bounded landing fixture");
        }
    }
    bytes.append(&2_u16.to_be_bytes()).expect("bounded landing fixture");
    bytes.append(&5_u32.to_be_bytes()).expect("bounded landing fixture");
    bytes.append(&u32::from(member).to_be_bytes()).expect("bounded landing fixture");
    if member {
        bytes.append(&[1, 0]).expect("bounded landing fixture");
        bytes.append(&[1_u8; 32]).expect("bounded landing fixture");
        bytes.append(&[2_u8; 32]).expect("bounded landing fixture");
    }
    bytes.append(&u32::from(member).to_be_bytes()).expect("bounded landing fixture");
    if member {
        bytes.append(&81_u64.to_be_bytes()).expect("bounded landing fixture");
        bytes.append(&0_u16.to_be_bytes()).expect("bounded landing fixture");
        bytes.append(&0_u16.to_be_bytes()).expect("bounded landing fixture");
        bytes.append(&[4]).expect("bounded landing fixture");
    }
    bytes.meet(skein_lib::stream::Read::Fill(bytes.len())).expect("whole landing fixture")
}
#[test]
fn v15_landings_keep_their_in_flight_state_without_entering_a_round() {
    use crate::patches::kept::temper_store::v16 as kept;
    let schema = kept_schema(16);
    for (place, held, member) in [
        (0_u8, false, true),
        (1, false, true),
        (2, false, true),
        (3, false, true),
        (4, false, true),
        (5, false, true),
        (6, false, false),
        (2, true, true),
    ] {
        let bytes = v15_landing(place, held, member);
        let mut context = crate::patches::Context { landing_branch: Some(b"release".as_slice().into()) };
        let bytes = patched_with(15, 4, b"LandingRow", bytes, &mut context);
        let (value, row) = kept_value(&schema, b"LandingRow", bytes);
        let landing = value.field(row, kept::landing_row::LANDING).unwrap();
        assert_eq!(value.uint(value.field(landing, kept::landing_state::TASK).unwrap()).unwrap(), 61);
        assert_eq!(value.uint(value.field(landing, kept::landing_state::PROJECT).unwrap()).unwrap(), 7);
        assert_eq!(value.content(value.field(landing, kept::landing_state::BRANCH).unwrap()).unwrap(), b"release");
        assert!(value.some(value.field(landing, kept::landing_state::ROUND).unwrap()).unwrap().is_none());
        assert!(value.some(value.field(landing, kept::landing_state::OWN_HOLD).unwrap()).unwrap().is_none());
        assert_eq!(value.uint(value.field(landing, kept::landing_state::PREPARATION).unwrap()).unwrap(), 2);
        assert_eq!(value.uint(value.field(landing, kept::landing_state::STALE_PROOFS).unwrap()).unwrap(), 5);
        let phase = value.field(landing, kept::landing_state::PHASE).unwrap();
        let got_place = value.field(phase, kept::landing_phase::PLACE).unwrap();
        let expected = match place {
            0 => kept::landing_place::WAITING,
            1 => kept::landing_place::PREPARING,
            2 => kept::landing_place::PROVING,
            3 => kept::landing_place::PROVED,
            4 => kept::landing_place::TURN,
            5 => kept::landing_place::LANDED,
            6 => kept::landing_place::CANCELLED,
            _ => panic!("fixture place"),
        };
        assert_eq!(value.tag(got_place).unwrap(), expected);
        if place == 3 || place == 4 {
            let at = value.payload(got_place).unwrap().unwrap();
            assert_eq!(value.uint(value.field(at, kept::landing_at::AT).unwrap()).unwrap(), 123);
        }
        assert_eq!(value.some(value.field(phase, kept::landing_phase::HELD).unwrap()).unwrap().is_some(), held);
        let members = items(&value, value.field(landing, kept::landing_state::MEMBERS).unwrap());
        let entries = items(&value, value.field(row, kept::landing_row::ENTRIES).unwrap());
        assert_eq!(members.len(), usize::from(member));
        assert_eq!(entries.len(), usize::from(member));
        if let Some(&entry) = entries.first() {
            assert_eq!(value.uint(value.field(entry, kept::landing_entry::ENTRY).unwrap()).unwrap(), 81);
            assert_eq!(
                value.tag(value.field(entry, kept::landing_entry::OPERATION).unwrap()).unwrap(),
                kept::landing_operation::MERGE
            );
        }
        if let Some(&member) = members.first() {
            for (field, expected) in [
                (kept::landing_member::HEAD, [1; 32]),
                (kept::landing_member::BASE, [2; 32]),
                (kept::landing_member::CANDIDATE, [3; 32]),
            ] {
                let some = value.some(value.field(member, field).unwrap()).unwrap().unwrap();
                assert_eq!(value.content(some).unwrap(), expected);
            }
            let asked = value.some(value.field(member, kept::landing_member::ASKED).unwrap()).unwrap().unwrap();
            assert_eq!(value.uint(asked).unwrap(), 2);
            assert!(value.some(value.field(member, kept::landing_member::LEFT).unwrap()).unwrap().is_none());
            let gates = items(&value, value.field(member, kept::landing_member::GATES).unwrap());
            assert_eq!(
                value.uint(value.field(*gates.first().expect("proof"), kept::gate_proof::FAULTS).unwrap()).unwrap(),
                3
            );
        }
    }
}
