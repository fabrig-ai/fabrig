//! The baseline's live record reaches the before-talk payload (protocol/store/opening.md, 3).
use skein_codec::patch::{Capacity, Patcher, Step};

fn patched(root: &[u8], bytes: Box<[u8]>) -> Box<[u8]> {
    let texts = crate::patches::kept::jig_store_tasks::TEXTS;
    let mut patcher = Patcher::new(Capacity {
        versions: 2, patches: 1, text: skein_codec::schema::MAX_TEXT,
        bytes: 131_072, nodes: 16_384,
    });
    for text in texts {
        patcher.keep(text.as_bytes()).expect("kept schema");
    }
    let root = patcher.schema(9).expect("baseline schema").find(root).expect("old record");
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
    let schema = skein_codegen::parse(crate::patches::kept::jig_store_tasks::TEXTS[0]).expect("old schema");
    skein_codegen::generate(&schema).goldens.into_iter().find(|golden| golden.name == name)
        .expect("old golden").bytes
}

#[test]
fn a_baseline_live_row_keeps_its_fields_without_a_talk() {
    let smallest = baseline_golden("enum_value_live_smallest.bin");
    let (tag, record) = smallest.split_first().expect("live tag");
    let (_, fields) = record.split_first().expect("closer option");
    let mut live = vec![*tag, 1];
    live.extend_from_slice(&42_u64.to_be_bytes());
    live.extend_from_slice(fields);
    let mut before = vec![0, 9];
    before.extend_from_slice(&live);
    before.extend_from_slice(&[0, 0, 0, 0]); // No model names.
    let bytes = patched(b"Row", before.into_boxed_slice());
    let (tag, fields) = live.split_first().expect("live tag");
    let mut expected = vec![0, 10, *tag, 0, 0]; // Version, same live tag, activation false, session none.
    expected.extend_from_slice(fields);
    expected.extend_from_slice(&[0, 0, 0, 0]);
    assert_eq!(bytes.as_ref(), expected);
    let row = crate::tasks_v10::Row::decode(&crate::tasks_v10::CEILINGS, &mut skein_lib::Reader::new(&bytes))
        .expect("current row");
    let crate::tasks_v10::Value::Live(record) = row.into_parts().value else { panic!("before-talk live") };
    assert!(!record.activation_open());
    assert_eq!(record.session(), &None);
}
