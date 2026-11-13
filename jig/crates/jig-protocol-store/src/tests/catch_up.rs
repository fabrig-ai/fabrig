//! The baseline's live record reaches the before-talk payload (protocol/store/opening.md, 3).
use skein_codec::patch::{Capacity, Patcher, Step};
use skein_lib::bytes;

fn patched(version: u16, patch: usize, root: &[u8], bytes: Box<[u8]>) -> Box<[u8]> {
    let texts = crate::patches::kept::jig_store_tasks::TEXTS;
    let mut patcher = Patcher::new(Capacity {
        versions: 3,
        patches: 2,
        text: skein_codec::schema::MAX_TEXT,
        bytes: 131_072,
        nodes: 16_384,
    });
    for text in texts {
        patcher.keep(text.as_bytes()).expect("kept schema");
    }
    let root = patcher.schema(version).expect("baseline schema").find(root).expect("old record");
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

fn baseline_golden(name: &str) -> Box<[u8]> {
    let schema = skein_codegen::parse(crate::patches::kept::jig_store_tasks::TEXTS[0]).expect("old schema");
    for golden in skein_codegen::generate(&schema).goldens {
        if golden.name == name {
            return golden.bytes.into_boxed_slice();
        }
    }
    unreachable!("old golden")
}

#[test]
fn a_baseline_live_row_keeps_its_fields_without_a_talk() {
    let smallest = baseline_golden("enum_value_live_smallest.bin");
    let (tag, record) = smallest.split_first().expect("live tag");
    let (_, fields) = record.split_first().expect("closer option");
    let live = bytes::concat(&[&[*tag, 1], &42_u64.to_be_bytes(), fields]);
    let before = bytes::concat(&[&[0, 9], &live, &[0, 0, 0, 0]]); // No model names.
    let bytes = patched(9, 0, b"Row", before);
    let (tag, fields) = live.split_first().expect("live tag");
    // Version, same live tag, activation false, session none; no model names.
    let expected = bytes::concat(&[&[0, 10, *tag, 0, 0], fields, &[0, 0, 0, 0]]);
    assert_eq!(bytes.as_ref(), expected.as_ref());
    let bytes = patched(10, 2, b"Row", bytes);
    let row = crate::tasks_v11::Row::decode(&crate::tasks_v11::CEILINGS, &mut skein_lib::Reader::new(&bytes))
        .expect("current row");
    let crate::tasks_v11::Value::Live(record) = row.into_parts().value else { panic!("before-talk live") };
    assert!(!record.activation_open());
    assert_eq!(record.session(), &None);
    assert_eq!(record.outcome().tries().infrastructure(), 0);
}

#[test]
fn a_stored_counter_before_infrastructure_decodes_with_zero_tries() {
    let mut old = skein_lib::Writer::new(24);
    for count in [1_u32, 2, 3, 4, 5, 6] {
        old.put(&count.to_be_bytes()).unwrap();
    }
    let bytes = patched(10, 2, b"Tries", old.finish());
    let wire = crate::tasks_v11::Tries::decode(&crate::tasks_v11::CEILINGS, &mut skein_lib::Reader::new(&bytes))
        .expect("patched tries");
    assert_eq!(
        crate::codec::tasks::value::read_tries(wire),
        jig_domain_tasks::Tries {
            transient: 1,
            permanent: 2,
            run: 3,
            agent: 4,
            lost: 5,
            invalid: 6,
            infrastructure: 0,
        }
    );
}
