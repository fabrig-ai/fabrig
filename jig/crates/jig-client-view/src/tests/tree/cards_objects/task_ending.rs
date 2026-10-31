use crate::tests::kit::Goldens;
use crate::tests::tree::cards_objects::*;

#[test]
fn task_overview_shows_only_supplied_ending_facts_with_opaque_state_loss_and_raw_slots() {
    let mut goldens = Goldens::new("tree/cards_objects/task_ending/task_overview_shows");
    let segments = crate::tests::kit::list_in(CLIENT.client.rows, [Box::from(*b"a/b"), Box::from([255_u8, 0])]);
    let part = Part { connector: u16::MAX, kind: PartKind::Result, key: PartKey::Path(segments) };
    for result in [
        None,
        Some(ResultValue::Approved { state: Box::from([255_u8; 32]), words: None }),
        Some(ResultValue::Approved {
            state: Box::from([255_u8; 32]),
            words: Some(DocumentText { bytes: text(b""), cut: true }),
        }),
        Some(ResultValue::Change {
            kind: u16::MAX,
            part: part.clone(),
            words: DocumentText::complete(text(b"retained")),
        }),
    ] {
        let mut row = task_doc(&[]);
        row.result = Some(TaskResult::Ending(Box::new(ChipEnding::Cancelled {
            reason: DocumentText { bytes: text(b"source reason"), cut: true },
            result: result.clone(),
            by: Requester::Deployment { project: u32::MAX },
            at: Wall::from_nanos(u64::MAX),
        })));
        let (tree, bindings) = task_contract::page(row.clone());
        goldens.check(&tree);
        assert!(tree.len() <= crate::task_nodes(&CLIENT, &(CLIENT).client, 0).unwrap());
        let mut fragment_slots = 0;
        match result {
            Some(ResultValue::Approved { .. }) => {
                goldens.check(&tree);
            }
            Some(ResultValue::Change { .. }) => {
                fragment_slots = 1;
                let mut found = 0_u32;
                for node in tree.nodes() {
                    if let Some(slot) = node.slot {
                        assert_eq!(bindings.owner_part(slot.token), Some(&part));
                        assert_eq!(bindings.part(slot.token), None);
                        found += 1;
                    }
                }
                assert_eq!(found, 2, "shared card and overview each retain the whole raw part");
                goldens.check(&tree);
            }
            None => goldens.check(&tree),
            Some(
                ResultValue::Report(_)
                | ResultValue::Words(_)
                | ResultValue::Failure(_)
                | ResultValue::Verdict { .. }
                | ResultValue::OwnerVerdict { .. }
                | ResultValue::Connector { .. }
                | ResultValue::Choice { .. },
            ) => unreachable!(),
        }
        let (domain, _) = start();
        let envelope = limits();
        let mut builder = Builder::new(&envelope);
        builder.open(Role::Page);
        let mut bindings = JigBindings::new(envelope.bindings, CLIENT.client.text);
        crate::tree::fragments::task_result::render(
            &mut builder,
            &domain.view(),
            row.result.as_ref().unwrap(),
            words(),
            &mut bindings,
        );
        builder.close();
        let tree = builder.finish();
        assert!(tree.len() <= crate::tree::fragments::task_result::nodes(&CLIENT, &(CLIENT).client).unwrap());
        goldens.check(&tree);
        let slots: Vec<_> = tree.nodes().iter().filter_map(|node| node.slot).collect();
        assert_eq!(slots.len(), fragment_slots);
        for slot in slots {
            assert_eq!(bindings.owner_part(slot.token), Some(&part));
            assert_eq!(bindings.part(slot.token), None);
        }
    }
}

#[test]
fn task_result_document_keeps_its_own_time_value_and_departure_state_without_owner_conversion() {
    let mut goldens = Goldens::new("tree/cards_objects/task_ending/task_result_document");
    let result = result();
    let mut row = task_doc(&[]);
    row.result = Some(TaskResult::Document(Box::new(result.clone())));
    let (tree, _) = task_contract::page(row);
    goldens.check(&tree);
    assert_eq!(result.at, Some(Wall::EPOCH));
}
