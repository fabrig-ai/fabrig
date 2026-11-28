//! The frame registry in loaded golden copies (temper/docs/design/protocol/floor.md, 5).
use crate::log;
use jig_domain_core::{CoreKey, Key};
use jig_protocol_store::{keys, registry_v1 as registry};
use skein_codec::Record as _;
use skein_kv::Op;
use skein_lib::Reader;
use std::collections::BTreeMap;

/// Build the commit to append to the kept floor or dev store's single, complete log.
#[must_use]
pub fn frame_registry_commit(bytes: &[u8]) -> Box<[u8]> {
    let (frames, used) = log::frames(bytes, 0);
    assert_eq!(used, bytes.len(), "whole kept log");
    let mut store = BTreeMap::new();
    let mut last = 0;
    for (number, ops) in frames {
        assert_eq!(number, last + 1, "consecutive kept commits");
        for op in ops {
            match op {
                Op::Put { key, value } => {
                    store.insert(key, value);
                }
                Op::Erase { key } => {
                    store.remove(&key);
                }
                Op::Append { .. } | Op::Drop { .. } => unreachable!("decoded frames contain map operations"),
            }
        }
        last = number;
    }
    let ops = [("jig-store-core", "TurnRecord", 9, 6), ("smith-transcript", "Turn", 0, 7)].map(
        |(family, root, number, id)| {
            let key = keys::encode_own(&keys::Own::Layout { family: family.into(), number, id }, 64)
                .expect("frame layout key");
            assert!(!store.contains_key(key.as_ref()), "the kept copy predates the frame registry");
            let header = format!("family {family} {id}\n");
            let mut kept = store.iter().filter_map(|(key, value)| {
                let Some(keys::Own::Text(digest)) = keys::decode_own(key) else { return None };
                let text =
                    registry::Text::decode(&registry::CEILINGS, &mut Reader::new(value)).expect("kept registry text");
                text.text.starts_with(header.as_bytes()).then_some((digest, text.text))
            });
            let (text, bytes) = kept.next().unwrap_or_else(|| panic!("kept schema for {family} {id}"));
            assert!(kept.next().is_none(), "one kept schema for {family} {id}");
            let schema = skein_codec::schema::parse(&bytes).expect("kept schema");
            // The turn inside TurnRecord has its own layout, not a held text in this row.
            let layout = schema
                .layout(schema.root(root.as_bytes(), None).expect("frame root"), &[])
                .expect("frame layout")
                .digest();
            let row = registry::Layout {
                root: root.as_bytes().into(),
                variant: None,
                layout,
                text,
                held: skein_lib::List::with_capacity(0),
            };
            Op::Put { key, value: row.encode(&registry::CEILINGS).expect("frame registry row") }
        },
    );
    let [core, smith] = ops;
    let key = keys::encode(&Key::Core(CoreKey::Deployment), 64).expect("deployment key");
    let mut value = store.remove(key.as_ref()).expect("kept deployment header");
    let number = advance_header(&mut value);
    assert_eq!(number, last + 1, "kept header agrees with the log");
    log::commit(number, &[core, smith, Op::Put { key, value }])
}

fn advance_header(value: &mut [u8]) -> u64 {
    // The kept core-6 Deployment is headed by layout id and tag, then
    // the 16-byte deployment id and eight u64 counters before `commits`.
    const COMMITS: usize = 2 + 1 + 16 + 8 * 8;
    assert!(value.starts_with(&[0, 6, 1]), "kept core-6 deployment layout");
    let field = &mut value[COMMITS..COMMITS + 8];
    let next = u64::from_be_bytes(field.try_into().expect("commit counter")) + 1;
    field.copy_from_slice(&next.to_be_bytes());
    next
}
