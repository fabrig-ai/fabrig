//! Main's stored additions join the baseline catch-up (protocol/baseline.md, 9).
use skein_codec::patch::{Capacity, Patcher, Step};
use skein_lib::bytes;

fn patched(version: u16, patch: usize, root: &[u8], bytes: Box<[u8]>) -> Box<[u8]> {
    let texts = crate::patches::kept::temper_store::TEXTS;
    let mut patcher = Patcher::new(Capacity {
        versions: 4,
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
                crate::patches::written(site, old, new, at, &mut crate::patches::Context::default())
                    .expect("written conversion");
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
    let bytes = patched(11, 0, b"Row", before);
    let row = crate::v12::Row::decode(&crate::v12::CEILINGS, &mut skein_lib::Reader::new(&bytes)).expect("current row");
    let crate::v12::Value::Repository(record) = row.into_parts().value else { panic!("adopted repository") };
    assert!(record.contexts().is_empty());
    assert_eq!(record.project(), u32::MAX);
    assert!(record.home());
}

#[test]
fn a_foreman_with_a_task_becomes_retired() {
    let before = kept_golden(0, "enum_stands_foreman_full.bin");
    let bytes = patched(11, 0, b"Stands", before);
    let value =
        crate::v12::Stands::decode(&crate::v12::CEILINGS, &mut skein_lib::Reader::new(&bytes)).expect("current stands");
    assert_eq!(value, crate::v12::Stands::Retired);
}

#[test]
fn a_v13_closer_upgrades_with_no_question_relay() {
    let before = kept_golden(2, "record_closer_full.bin");
    let bytes = patched(13, 2, b"Closer", before);
    let row = crate::v14::Closer::decode(&crate::v14::CEILINGS, &mut skein_lib::Reader::new(&bytes))
        .expect("upgraded closer");
    assert_eq!(row.relay(), &None);
    assert_eq!(row.task(), u64::MAX);
    assert_eq!(row.goal(), u64::MAX);
    assert!(row.done());
}
