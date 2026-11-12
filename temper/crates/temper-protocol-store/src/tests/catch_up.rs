//! Main's stored additions join the baseline catch-up (protocol/baseline.md, 9).
use skein_codec::patch::{Capacity, Patcher, Step};

fn patched(root: &[u8], bytes: Box<[u8]>) -> Box<[u8]> {
    let texts = crate::patches::kept::temper_store::TEXTS;
    let mut patcher = Patcher::new(Capacity {
        versions: 2, patches: 1, text: skein_codec::schema::MAX_TEXT,
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

fn baseline_golden(name: &str) -> Box<[u8]> {
    let schema = skein_codegen::parse(crate::patches::kept::temper_store::TEXTS[0]).expect("old schema");
    for golden in skein_codegen::generate(&schema).goldens {
        if golden.name == name {
            return skein_lib::bytes::copy_of(&golden.bytes);
        }
    }
    panic!("old golden");
}

#[test]
fn a_baseline_adopted_row_has_no_contexts() {
    let adopted = baseline_golden("record_adopted_full.bin");
    let mut before = skein_lib::Writer::new(adopted.len().checked_add(3).expect("golden with prefix"));
    before.put(&[0, 11, 2]).expect("row version, repository variant");
    before.put(&adopted).expect("adopted golden");
    let bytes = patched(b"Row", before.finish());
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
    let bytes = patched(b"Stands", before);
    let value = crate::v12::Stands::decode(&crate::v12::CEILINGS, &mut skein_lib::Reader::new(&bytes))
        .expect("current stands");
    assert_eq!(value, crate::v12::Stands::Retired);
}
