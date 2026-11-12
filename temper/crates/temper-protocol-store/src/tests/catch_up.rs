//! Main's stored additions join the baseline catch-up (protocol/baseline.md, 9).
use skein_codec::patch::{Capacity, Patcher, Step};
use skein_lib::bytes;

fn patched(root: &[u8], bytes: Box<[u8]>) -> Box<[u8]> {
    let texts = crate::patches::kept::temper_store::TEXTS;
    let mut patcher = Patcher::new(Capacity {
        versions: 4,
        patches: 3,
        text: skein_codec::schema::MAX_TEXT,
        bytes: 131_072,
        nodes: 16_384,
    });
    for text in texts {
        patcher.keep(text.as_bytes()).expect("kept schema");
    }
    let mut bytes = bytes;
    for (version, patch) in
        [(11, crate::patches::PATCHES[0]), (12, crate::patches::PATCHES[1]), (13, crate::patches::PATCHES[2])]
    {
        let root = patcher.schema(version).expect("baseline schema").find(root).expect("old record");
        let plan = patcher.plan(patch.as_bytes()).expect("valid plan").expect("family moves");
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
        bytes = patcher.finish().expect("patched record").into();
    }
    bytes
}

fn baseline_golden(name: &str) -> Box<[u8]> {
    let schema = skein_codegen::parse(crate::patches::kept::temper_store::TEXTS[0]).expect("old schema");
    for golden in skein_codegen::generate(&schema).goldens {
        if golden.name == name {
            return golden.bytes.into_boxed_slice();
        }
    }
    unreachable!("old golden")
}

#[test]
fn a_baseline_adopted_row_has_no_contexts() {
    let adopted = baseline_golden("record_adopted_full.bin");
    let before = bytes::concat(&[&[0, 11, 2], &adopted]); // Row version, repository variant.
    let bytes = patched(b"Row", before);
    let row = crate::v14::Row::decode(&crate::v14::CEILINGS, &mut skein_lib::Reader::new(&bytes)).expect("current row");
    let crate::v14::Value::Repository(record) = row.into_parts().value else { panic!("adopted repository") };
    assert!(record.contexts().is_empty());
    assert_eq!(record.project(), u32::MAX);
    assert!(record.home());
}

#[test]
fn a_foreman_with_a_task_becomes_retired() {
    let before = baseline_golden("enum_stands_foreman_full.bin");
    let bytes = patched(b"Stands", before);
    let value =
        crate::v14::Stands::decode(&crate::v14::CEILINGS, &mut skein_lib::Reader::new(&bytes)).expect("current stands");
    assert_eq!(value, crate::v14::Stands::Retired);
}
