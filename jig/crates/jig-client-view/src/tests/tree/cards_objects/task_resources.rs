use crate::tests::kit::Goldens;
use crate::tests::tree::cards_objects::*;

#[test]
fn task_resource_metadata_keeps_each_raw_slot_writer_presence_full_width_and_independent_flags() {
    let mut goldens = Goldens::new("tree/cards_objects/task_resources/task_resource_metadata");
    let segments =
        crate::tests::kit::list_in(CLIENT.client.rows, [Box::<[u8]>::from(*b"a/b"), Box::<[u8]>::from([255_u8; 32])]);
    let reference = Part { connector: u16::MAX, kind: PartKind::Resource, key: PartKey::Path(segments) };
    for writer in [
        None,
        Some(ResourceWriter::Effect { entry: u64::MAX }),
        Some(ResourceWriter::Run { task: u64::MAX, attempt: u64::MAX }),
    ] {
        for taken in [false, true] {
            let mut row = task_doc(&[]);
            row.resources = List::with_capacity(CLIENT.client.rows);
            for holder in [None, Some(u64::MAX)] {
                row.resources
                    .push(ResourceLine {
                        part: reference.clone(),
                        kind: u16::MAX,
                        taken,
                        holder,
                        writer,
                        writer_lost: !taken,
                    })
                    .unwrap();
            }
            let (tree, bindings) = task_contract::page(row.clone());
            goldens.check(&tree);
            let mut slots = 0_u32;
            for node in tree.nodes() {
                if let Some(slot) = node.slot {
                    assert_eq!(bindings.owner_part(slot.token), Some(&reference));
                    assert_eq!(bindings.part(slot.token), None);
                    slots += 1;
                }
            }
            assert_eq!(slots, CLIENT.client.rows);
            assert!(tree.len() <= crate::task_nodes(&CLIENT, &(CLIENT).client, 0).unwrap());
            assert_eq!(row.resources.capacity(), CLIENT.client.rows);
        }
    }
}
