//! Main's stored additions join the baseline catch-up (protocol/baseline.md, 9).
use skein_codec::patch::{Capacity, Patcher, Step};

fn patched(root: &[u8], bytes: Box<[u8]>) -> Box<[u8]> {
    let texts = crate::patches::kept::temper_store::TEXTS;
    let mut patcher = Patcher::new(Capacity {
        versions: 3, patches: 1, text: skein_codec::schema::MAX_TEXT,
        bytes: 131_072, nodes: 16_384,
    });
    for text in texts {
        patcher.keep(text.as_bytes()).expect("kept schema");
    }
    let root = patcher.schema(11).expect("baseline schema").find(root).expect("old record");
    let plan = patcher.plan(crate::patches::PATCHES[0].as_bytes()).expect("valid plan").expect("family moves");
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

fn baseline_golden(name: &str) -> Vec<u8> {
    let schema = skein_codegen::parse(crate::patches::kept::temper_store::TEXTS[0]).expect("old schema");
    skein_codegen::generate(&schema).goldens.into_iter().find(|golden| golden.name == name)
        .expect("old golden").bytes
}

#[test]
fn a_baseline_adopted_row_has_no_contexts() {
    let adopted = baseline_golden("record_adopted_full.bin");
    let mut before = vec![0, 11, 2]; // Row version, repository variant.
    before.extend_from_slice(&adopted);
    let bytes = patched(b"Row", before.into_boxed_slice());
    let row = crate::v12::Row::decode(&crate::v12::CEILINGS, &mut skein_lib::Reader::new(&bytes))
        .expect("current row");
    let crate::v12::Value::Repository(record) = row.into_parts().value else { panic!("adopted repository") };
    assert!(record.contexts().is_empty());
    assert_eq!(record.project(), u32::MAX);
    assert!(record.home());
}

#[test]
fn a_foreman_with_a_task_becomes_retired() {
    let before = baseline_golden("enum_stands_foreman_full.bin");
    let bytes = patched(b"Stands", before.into_boxed_slice());
    let value = crate::v12::Stands::decode(&crate::v12::CEILINGS, &mut skein_lib::Reader::new(&bytes))
        .expect("current stands");
    assert_eq!(value, crate::v12::Stands::Retired);
}
