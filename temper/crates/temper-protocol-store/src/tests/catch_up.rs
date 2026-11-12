//! Main's stored additions join the baseline catch-up (protocol/baseline.md, 9).
use skein_codec::patch::{Capacity, Patcher, Step};
use skein_lib::bytes;

fn patched(version: u16, patch: usize, root: &[u8], bytes: Box<[u8]>) -> Box<[u8]> {
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
        versions: 6,
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
    for (version, patch) in [(11, 0), (12, 1), (13, 2), (14, 3), (15, 4)] {
        bytes = patched(version, patch, root, bytes);
    }
    bytes
}

fn kept_golden(text: usize, name: &str) -> Box<[u8]> {
    let schema = skein_codegen::parse(crate::patches::kept::temper_store::TEXTS[text]).expect("old schema");
    for golden in skein_codegen::generate(&schema).goldens {
        if golden.name == name {
            return golden.bytes.into_boxed_slice();
        }
    }
    unreachable!("old golden")
}

#[test]
fn a_baseline_adopted_row_has_no_contexts() {
    let adopted = kept_golden(0, "record_adopted_full.bin");
    let before = bytes::concat(&[&[0, 11, 2], &adopted]); // Row version, repository variant.
    let bytes = current(b"Row", before);
    let row = crate::v16::Row::decode(&crate::v16::CEILINGS, &mut skein_lib::Reader::new(&bytes)).expect("current row");
    let crate::v16::Value::Repository(record) = row.into_parts().value else { panic!("adopted repository") };
    assert!(record.contexts().is_empty());
    assert_eq!(record.project(), u32::MAX);
    assert!(record.home());
}

#[test]
fn a_foreman_with_a_task_becomes_retired() {
    let before = kept_golden(0, "enum_stands_foreman_full.bin");
    let bytes = current(b"Stands", before);
    let value =
        crate::v16::Stands::decode(&crate::v16::CEILINGS, &mut skein_lib::Reader::new(&bytes)).expect("current stands");
    assert_eq!(value, crate::v16::Stands::Retired);
}

#[test]
fn a_v13_closer_upgrades_with_no_question_relay() {
    let before = kept_golden(2, "record_closer_full.bin");
    let bytes = patched(13, 2, b"Closer", before);
    let bytes = patched(14, 3, b"Closer", bytes);
    let row = crate::v15::Closer::decode(&crate::v15::CEILINGS, &mut skein_lib::Reader::new(&bytes))
        .expect("upgraded closer");
    assert_eq!(row.relay(), &None);
    assert_eq!(row.task(), u64::MAX);
    assert_eq!(row.goal(), u64::MAX);
    assert!(row.done());
}

#[test]
fn a_v14_closer_keeps_its_question_relay_through_r2() {
    let before = kept_golden(3, "record_closer_full.bin");
    let bytes = patched(14, 3, b"Closer", before.clone());
    assert_eq!(bytes, before, "R2 only adds declarations and variants");
    let closer =
        crate::v16::Closer::decode(&crate::v16::CEILINGS, &mut skein_lib::Reader::new(&bytes)).expect("current closer");
    assert!(closer.relay().is_some());
}

#[test]
fn v15_landings_keep_their_in_flight_state_without_entering_a_round() {
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
        let mut bytes = Vec::new();
        bytes.extend(61_u64.to_be_bytes());
        bytes.extend(7_u32.to_be_bytes());
        bytes.push(place);
        if place == 3 || place == 4 {
            bytes.extend(123_u64.to_be_bytes());
        }
        bytes.push(u8::from(held));
        if held {
            bytes.extend(0_u16.to_be_bytes());
            bytes.extend(0_u16.to_be_bytes());
        }
        bytes.extend(u32::from(member).to_be_bytes());
        if member {
            bytes.extend(62_u64.to_be_bytes());
            bytes.extend(1_u16.to_be_bytes());
            bytes.extend(2_u32.to_be_bytes());
            bytes.extend(10_u64.to_be_bytes());
            for commit in [1_u8, 2, 3] {
                bytes.push(1);
                bytes.extend([commit; 32]);
            }
            bytes.extend(1_u32.to_be_bytes());
            bytes.push(2); // dispatched proof
            bytes.extend([4_u8; 32]);
            bytes.extend(2_u16.to_be_bytes());
            bytes.push(1); // pending posting
            bytes.extend(3_u32.to_be_bytes());
            bytes.push(1);
            bytes.extend(2_u16.to_be_bytes()); // asked preparation
            bytes.push(u8::from(place == 5));
            if place == 5 {
                bytes.extend([3_u8; 32]);
            }
        }
        bytes.extend(2_u16.to_be_bytes());
        bytes.extend(5_u32.to_be_bytes());
        bytes.extend(u32::from(member).to_be_bytes());
        if member {
            bytes.extend([1, 0]);
            bytes.extend([1_u8; 32]);
            bytes.extend([2_u8; 32]);
        }
        bytes.extend(u32::from(member).to_be_bytes());
        if member {
            bytes.extend(81_u64.to_be_bytes());
            bytes.extend(0_u16.to_be_bytes());
            bytes.extend(0_u16.to_be_bytes());
            bytes.push(4);
        }
        let mut context = crate::patches::Context { landing_branch: Some(b"release".as_slice().into()) };
        let bytes = patched_with(15, 4, b"LandingRow", bytes.into_boxed_slice(), &mut context);
        let row = crate::v16::LandingRow::decode(&crate::v16::CEILINGS, &mut skein_lib::Reader::new(&bytes))
            .expect("migrated landing");
        let landing = row.landing();
        assert_eq!(landing.task(), 61);
        assert_eq!(landing.project(), 7);
        assert_eq!(landing.branch(), b"release");
        assert!(landing.round().is_none());
        assert!(landing.own_hold().is_none());
        assert_eq!(landing.preparation(), 2);
        assert_eq!(landing.stale_proofs(), 5);
        let got_place = match landing.phase().place() {
            crate::v16::LandingPlace::Waiting => 0,
            crate::v16::LandingPlace::Preparing => 1,
            crate::v16::LandingPlace::Proving => 2,
            crate::v16::LandingPlace::Proved(at) => {
                assert_eq!(at.at(), 123);
                3
            }
            crate::v16::LandingPlace::Turn(at) => {
                assert_eq!(at.at(), 123);
                4
            }
            crate::v16::LandingPlace::Landed => 5,
            crate::v16::LandingPlace::Cancelled => 6,
        };
        assert_eq!(got_place, place);
        assert_eq!(landing.phase().held().is_some(), held);
        assert_eq!(landing.members().len(), u32::from(member));
        assert_eq!(row.entries().len(), u32::from(member));
        if let Some(entry) = row.entries().as_slice().first() {
            assert_eq!(entry.entry(), 81);
            assert_eq!(entry.operation(), &crate::v16::LandingOperation::Merge);
        }
        if let Some(member) = landing.members().as_slice().first() {
            assert_eq!(member.head(), &Some([1; 32]));
            assert_eq!(member.base(), &Some([2; 32]));
            assert_eq!(member.candidate(), &Some([3; 32]));
            assert_eq!(member.asked(), &Some(2));
            assert!(member.left().is_none());
            assert_eq!(member.gates().as_slice().first().expect("proof").faults(), 3);
        }
    }
}
