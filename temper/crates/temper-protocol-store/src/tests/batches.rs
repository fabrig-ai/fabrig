//! Batch storage and the v17 single-member patch (domain/landing.md, 12).
use super::{
    catch_up::{items, kept_golden, kept_schema, kept_value, patched},
    fixtures,
};
use crate::patches::kept::temper_store::v17 as kept;
use crate::{records, v22};
use skein_codec::{
    schema::{FieldIx, Schema},
    value::{Node, Value},
};
use skein_lib::{Reader, Writer};
use temper_engine_domain_forge as forge;
use temper_engine_domain_forge_landing as landing;

fn replaced(value: &Value, node: Node, bytes: &[u8]) -> Box<[u8]> {
    let span = value.span(node).unwrap();
    skein_lib::bytes::concat(&[
        value.bytes().get(..usize::try_from(span.start).unwrap()).unwrap(),
        bytes,
        value.bytes().get(usize::try_from(span.end).unwrap()..).unwrap(),
    ])
}

fn fixed_option(value: &Value, record: Node, field: FieldIx) -> Option<[u8; 32]> {
    match value.some(value.field(record, field).unwrap()).unwrap() {
        Some(node) => Some(value.content(node).unwrap().try_into().unwrap()),
        None => None,
    }
}

fn uint_option(value: &Value, record: Node, field: FieldIx) -> Option<u64> {
    match value.some(value.field(record, field).unwrap()).unwrap() {
        Some(node) => Some(value.uint(node).unwrap()),
        None => None,
    }
}

fn v17_landing(schema: &Schema) -> Box<[u8]> {
    let mut bytes = kept_golden(17, "record_landing_row_full.bin");
    for (field, replacement) in [
        (kept::landing_round::COMBINED, &[1, 1][..]), // some squash
        (kept::landing_round::RECORDED, &[1][..]),
        (kept::landing_round::TOLD, &[1][..]),
        (kept::landing_round::JUDGING, &[1][..]),
    ] {
        let (value, row) = kept_value(schema, b"LandingRow", bytes);
        let state = value.field(row, kept::landing_row::LANDING).unwrap();
        let round = value.some(value.field(state, kept::landing_state::ROUND).unwrap()).unwrap().unwrap();
        bytes = replaced(&value, value.field(round, field).unwrap(), replacement);
    }
    bytes
}
fn upgraded(before: &[u8]) -> v22::LandingRow {
    let bytes =
        patched(19, 8, b"LandingRow", patched(18, 7, b"LandingRow", patched(17, 6, b"LandingRow", before.into())));
    let bytes = patched(20, 9, b"LandingRow", bytes);
    v22::LandingRow::decode(&v22::CEILINGS, &mut Reader::new(&bytes)).unwrap()
}
fn stored(row: v22::LandingRow) -> forge::Stored {
    let row = v22::Row::new(&v22::CEILINGS, v22::RowParts { value: v22::Value::Landing(row) }).unwrap();
    let mut writer = Writer::new(usize::try_from(row.measure()).unwrap());
    row.encode(&mut writer).unwrap();
    let bytes = writer.finish();
    records::decode(&bytes, &crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).unwrap()).unwrap()
}

#[test]
fn a_v17_member_keeps_its_round_values_through_the_batch_patch() {
    let schema = kept_schema(17);
    let before = v17_landing(&schema);
    let after = upgraded(&before);
    let (value, row) = kept_value(&schema, b"LandingRow", before);
    let state = value.field(row, kept::landing_row::LANDING).unwrap();
    let members = items(&value, value.field(state, kept::landing_state::MEMBERS).unwrap());
    let prior = *members.first().unwrap();
    let round_option = value.field(state, kept::landing_state::ROUND).unwrap();
    let round = value.some(round_option).unwrap().unwrap();
    let combined = value.some(value.field(round, kept::landing_round::COMBINED).unwrap()).unwrap().unwrap();
    assert_eq!(value.tag(combined).unwrap(), kept::landing_combine::SQUASH);
    for field in [kept::landing_round::RECORDED, kept::landing_round::TOLD, kept::landing_round::JUDGING] {
        assert!(value.boolean(value.field(round, field).unwrap()).unwrap());
    }
    let member = after.landing().members().as_slice().first().unwrap();
    assert_eq!(member.unit(), 0);
    assert_eq!(member.branch(), value.content(value.field(round, kept::landing_round::BRANCH).unwrap()).unwrap());
    assert_eq!(member.combined(), &Some(v22::LandingCombine::Squash));
    assert!(member.recorded() && member.told() && member.judging());
    assert!(member.leave_told(), "v17 already emitted its leave");
    assert_eq!(*member.head(), fixed_option(&value, prior, kept::landing_member::HEAD));
    assert_eq!(*member.base(), fixed_option(&value, prior, kept::landing_member::BASE));
    assert_eq!(*member.candidate(), fixed_option(&value, prior, kept::landing_member::CANDIDATE));
    assert_eq!(
        match member.asked() {
            Some(asked) => Some(u64::from(*asked)),
            None => None,
        },
        uint_option(&value, prior, kept::landing_member::ASKED)
    );
    assert_eq!(
        usize::try_from(member.gates().len()).unwrap(),
        items(&value, value.field(prior, kept::landing_member::GATES).unwrap()).len()
    );
    assert_eq!(
        usize::try_from(after.entries().len()).unwrap(),
        items(&value, value.field(row, kept::landing_row::ENTRIES).unwrap()).len()
    );
    assert_eq!(
        usize::try_from(after.asks().len()).unwrap(),
        items(&value, value.field(row, kept::landing_row::ASKS).unwrap()).len()
    );
    assert!(after.landing().round().as_ref().unwrap().changed().is_none());
    // A v15 single landing, already patched to v17, still has no round.
    let after = upgraded(&replaced(&value, round_option, &[0]));
    let member = after.landing().members().as_slice().first().unwrap();
    assert_eq!(member.branch(), []);
    assert!(member.combined().is_none());
    assert!(!member.recorded() && !member.told() && !member.judging());
}

#[test]
fn batch_members_and_leave_causes_round_trip() {
    let schema = kept_schema(17);
    let forge::Stored::Landing(row) = stored(upgraded(&v17_landing(&schema))) else { panic!("landing") };
    for why in [
        None,
        Some(landing::Leave::EarlierConflict),
        Some(landing::Leave::Unit { change: 13 }),
        Some(landing::Leave::Dependency { change: 14 }),
        Some(landing::Leave::Blamed { gate: 1, run: 9, red: [5; 32], green: [4; 32] }),
    ] {
        let mut row = row.clone();
        row.landing.phase = landing::Phase::Preparing;
        let mut members = skein_lib::List::with_capacity(3);
        for index in 0..3 {
            let mut member = row.landing.members[0].clone();
            member.change = index + 10;
            member.unit = u16::try_from(index).unwrap();
            member.branch = skein_lib::bytes::concat(&[b"temper/1/l61-", skein_lib::Decimal::of(index + 1).as_bytes()]);
            member.candidate = Some([u8::try_from(index).unwrap(); 32]);
            member.combined = if index == 1 { Some(landing::Combine::Merge) } else { Some(landing::Combine::Squash) };
            member.recorded = index == 0;
            member.told = index == 1;
            member.judging = index != 1;
            member.leave_told = index == 2;
            member.left = why;
            members.push(member).unwrap();
        }
        row.landing.members = members.into_boxed();
        row.landing.round.changed = Some(1);
        row.landing.round.search = Some(landing::Search { green: 1, red: 2 });
        row.asks = Box::new([None, None, None]);
        let row = forge::Stored::Landing(row);
        let limits = forge::Limits { landing_members: 3, ..fixtures::LIMITS };
        let bounds = crate::bounds(&limits, limits.client.op_bytes).unwrap();
        assert_eq!(records::decode(&records::encode(&row), &bounds), Ok(row));
    }
}

#[test]
fn landing_repair_context_round_trips_and_old_rows_have_none() {
    use crate::patches::kept::temper_store::v19 as old;
    use temper_engine_domain_forge_change::{Ask, Below, Landed, Repair, State};
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).unwrap();
    let schema = kept_schema(17);
    let forge::Stored::Landing(seed) = stored(upgraded(&v17_landing(&schema))) else { panic!("landing") };
    let mut changes = [None; 4];
    for (index, slot) in changes.iter_mut().enumerate() {
        *slot = Some(Landed { change: 2341 + u64::try_from(index).unwrap(), pull: 7 + u64::try_from(index).unwrap() });
    }
    for below in
        [None, Some(Below { green: [0xbb; 32], changes: [None; 4] }), Some(Below { green: [0xbb; 32], changes })]
    {
        let mut row = seed.clone();
        row.landing.phase = landing::Phase::Preparing;
        row.asks = Box::new([Some(Ask::Repair { gate: 1, candidate: [0xaa; 32], below })]);
        let row = forge::Stored::Landing(row);
        assert_eq!(records::decode(&records::encode(&row), &bounds), Ok(row));
        let row = forge::Stored::Change(Box::new(fixtures::change_row(State::Repairing {
            requested: true,
            why: Repair::LandingAt { gate: 1, candidate: [0xaa; 32], below },
        })));
        assert_eq!(records::decode(&records::encode(&row), &bounds), Ok(row));
    }
    // Rows of the preceding version acquire no boundary, whether the ask or repair is kept.
    let schema = kept_schema(19);
    // Both v19 repair records contain a u16 gate and a fixed 32-byte candidate.
    let before = skein_lib::bytes::concat(&[&1_u16.to_be_bytes(), &[0xaa; 32]]);
    let (value, row) = kept_value(&schema, b"LandingRepairAt", before.clone());
    assert_eq!(value.uint(value.field(row, old::landing_repair_at::GATE).unwrap()).unwrap(), 1);
    assert_eq!(value.content(value.field(row, old::landing_repair_at::CANDIDATE).unwrap()).unwrap(), [0xaa; 32]);
    let bytes = patched(19, 8, b"LandingRepairAt", before.clone());
    let after = v22::LandingRepairAt::decode(&v22::CEILINGS, &mut Reader::new(&bytes)).unwrap();
    assert_eq!((after.gate(), after.candidate(), after.below()), (1, &[0xaa; 32], &None));
    let (value, row) = kept_value(&schema, b"LandingRepairAsk", before.clone());
    assert_eq!(value.uint(value.field(row, old::landing_repair_ask::GATE).unwrap()).unwrap(), 1);
    assert_eq!(value.content(value.field(row, old::landing_repair_ask::CANDIDATE).unwrap()).unwrap(), [0xaa; 32]);
    let bytes = patched(19, 8, b"LandingRepairAsk", before);
    let after = v22::LandingRepairAsk::decode(&v22::CEILINGS, &mut Reader::new(&bytes)).unwrap();
    assert_eq!((after.gate(), after.candidate(), after.below()), (1, &[0xaa; 32], &None));
}

#[test]
fn a_stored_fallback_leaves_as_held_and_the_alone_mark_is_dropped() {
    use crate::patches::kept::temper_store::v18 as old;
    let schema = kept_schema(18);
    let before = kept_golden(18, "record_change_full.bin");
    let (value, row) = kept_value(&schema, b"Change", before);
    assert!(value.boolean(value.field(row, old::change::ALONE).unwrap()).unwrap());
    let bytes = patched(19, 8, b"Change", patched(18, 7, b"Change", value.bytes().into()));
    let bytes = patched(20, 9, b"Change", bytes);
    let after = v22::Change::decode(&v22::CEILINGS, &mut Reader::new(&bytes)).unwrap();
    assert_eq!(after.task(), value.uint(value.field(row, old::change::TASK).unwrap()).unwrap());
    assert_eq!(*after.landing(), uint_option(&value, row, old::change::LANDING));
    let schema = kept_schema(17);
    let mut row = upgraded(&v17_landing(&schema)).into_parts();
    let mut state = row.landing.into_parts();
    let member = state.members.get_mut(0).unwrap();
    let mut parts = member.clone().into_parts();
    parts.left = Some(v22::LandingLeave::Fallback(
        v22::LandingFallback::new(&v22::CEILINGS, v22::LandingFallbackParts { units: 3, top: [7; 32] }).unwrap(),
    ));
    *member = v22::LandingMember::new(&v22::CEILINGS, parts).unwrap();
    row.landing = v22::LandingState::new(&v22::CEILINGS, state).unwrap();
    let forge::Stored::Landing(after) = stored(v22::LandingRow::new(&v22::CEILINGS, row).unwrap()) else {
        panic!("landing")
    };
    assert_eq!(after.landing.members[0].left, Some(landing::Leave::Held));
}
