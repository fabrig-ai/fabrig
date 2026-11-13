//! Batch storage and the v17 single-member patch (domain/landing.md, 12).
use super::{
    catch_up::{kept_golden, patched},
    fixtures,
};
use crate::{records, v17, v18};
use skein_lib::{Reader, Writer};
use temper_engine_domain_forge as forge;
use temper_engine_domain_forge_landing as landing;

fn v17_landing() -> v17::LandingRow {
    let bytes = kept_golden(17, "record_landing_row_full.bin");
    let mut row = v17::LandingRow::decode(&v17::CEILINGS, &mut Reader::new(&bytes)).unwrap().into_parts();
    let mut state = row.landing.into_parts();
    let mut round = state.round.unwrap().into_parts();
    round.combined = Some(v17::LandingCombine::Squash);
    round.recorded = true;
    round.told = true;
    round.judging = true;
    state.round = Some(v17::LandingRound::new(&v17::CEILINGS, round).unwrap());
    row.landing = v17::LandingState::new(&v17::CEILINGS, state).unwrap();
    v17::LandingRow::new(&v17::CEILINGS, row).unwrap()
}
fn upgraded(before: &v17::LandingRow) -> v18::LandingRow {
    let mut writer = Writer::new(usize::try_from(before.measure()).unwrap());
    before.encode(&mut writer).unwrap();
    let bytes = patched(17, 6, b"LandingRow", writer.finish());
    v18::LandingRow::decode(&v18::CEILINGS, &mut Reader::new(&bytes)).unwrap()
}
fn stored(row: v18::LandingRow) -> forge::Stored {
    let row = v18::Row::new(&v18::CEILINGS, v18::RowParts { value: v18::Value::Landing(row) }).unwrap();
    let mut writer = Writer::new(usize::try_from(row.measure()).unwrap());
    row.encode(&mut writer).unwrap();
    records::decode(&writer.finish(), &crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).unwrap())
        .unwrap()
}

#[test]
fn a_v17_member_keeps_its_round_values_through_the_batch_patch() {
    let before = v17_landing();
    let after = upgraded(&before);
    let member = after.landing().members().as_slice().first().unwrap();
    let prior = before.landing().members().as_slice().first().unwrap();
    let round = before.landing().round().as_ref().unwrap();
    assert_eq!(member.unit(), 0);
    assert_eq!(member.branch(), round.branch());
    assert_eq!(member.combined(), &Some(v18::LandingCombine::Squash));
    assert!(member.recorded() && member.told() && member.judging());
    assert!(member.leave_told(), "v17 already emitted its leave");
    assert_eq!(member.head(), prior.head());
    assert_eq!(member.base(), prior.base());
    assert_eq!(member.candidate(), prior.candidate());
    assert_eq!(member.asked(), prior.asked());
    assert_eq!(member.gates().len(), prior.gates().len());
    assert_eq!(after.entries().len(), before.entries().len());
    assert_eq!(after.asks().len(), before.asks().len());
    assert!(after.landing().round().as_ref().unwrap().changed().is_none());
    // A v15 single landing, already patched to v17, still has no round.
    let mut old = before.into_parts();
    let mut state = old.landing.into_parts();
    state.round = None;
    old.landing = v17::LandingState::new(&v17::CEILINGS, state).unwrap();
    let after = upgraded(&v17::LandingRow::new(&v17::CEILINGS, old).unwrap());
    let member = after.landing().members().as_slice().first().unwrap();
    assert_eq!(member.branch(), []);
    assert!(member.combined().is_none());
    assert!(!member.recorded() && !member.told() && !member.judging());
}

#[test]
fn batch_members_and_leave_causes_round_trip() {
    let forge::Stored::Landing(row) = stored(upgraded(&v17_landing())) else { panic!("landing") };
    for why in [
        None,
        Some(landing::Leave::EarlierConflict),
        Some(landing::Leave::Unit { change: 13 }),
        Some(landing::Leave::Dependency { change: 14 }),
        Some(landing::Leave::Fallback { units: 3, top: [5; 32] }),
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
        row.asks = Box::new([None, None, None]);
        let row = forge::Stored::Landing(row);
        let limits = forge::Limits { landing_members: 3, ..fixtures::LIMITS };
        let bounds = crate::bounds(&limits, limits.client.op_bytes).unwrap();
        assert_eq!(records::decode(&records::encode(&row), &bounds), Ok(row));
    }
}

#[test]
fn the_alone_mark_round_trips_and_defaults_false_for_v17() {
    for alone in [false, true] {
        let mut row = fixtures::change_row(temper_engine_domain_forge_change::State::Producing { requested: false });
        row.change.alone = alone;
        let row = forge::Stored::Change(Box::new(row));
        let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).unwrap();
        assert_eq!(records::decode(&records::encode(&row), &bounds), Ok(row));
    }
    let before = kept_golden(17, "record_change_full.bin");
    let bytes = patched(17, 6, b"Change", before);
    let after = v18::Change::decode(&v18::CEILINGS, &mut Reader::new(&bytes)).unwrap();
    assert!(!after.alone());
}
