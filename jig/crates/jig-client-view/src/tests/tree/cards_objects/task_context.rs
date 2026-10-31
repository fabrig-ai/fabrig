use crate::state::address::{Address, TaskSection};
use crate::tests::kit::Goldens;
use crate::tests::tree::cards_objects::*;

#[test]
fn task_context_view_preserves_supplied_runtime_spend_slots_and_impact_without_claiming_missing_facts() {
    let mut goldens = Goldens::new("tree/cards_objects/task_context/task_context_view");
    let reference = ApplicationPart::Panel { connector: 2, task: u64::MAX };
    for present in [false, true] {
        let mut row = task_doc(&[ActKind::Cancel]);
        row.chip.executor = ChipExecutor::Agent { charter: 1 };
        row.chip.title.cut = true;
        row.delegates = List::with_capacity(0);
        let context = row.context.as_mut().unwrap();
        context.runtime =
            Runtime::Running { attempt: u64::MAX, since: Wall::EPOCH, tool: Some(text(b"supplied work")) };
        context.parts = list([reference.clone()]);
        context.spend = list([SpendLine { task: u64::MAX, attempt: Some(u64::MAX), spent: u64::MAX }]);
        context.writer_slots =
            list([WriterSlot { resource: reference.clone(), task: u64::MAX, attempt: Some(u64::MAX) }]);
        context.impact.tasks.running = u32::MAX;
        row.acts.get_mut(0).unwrap().effect = if present { Some(Effect::Cancel(context.impact)) } else { None };
        if !present {
            row.context = None;
        }
        for section in [TaskSection::Overview, TaskSection::Activity, TaskSection::Budget, TaskSection::Panels] {
            let (mut domain, out) = crate::tests::tree::work_pages::entered(Address::Task { task: 8, section });
            let mut task_watch = None;
            for request in &out {
                if let Request::Client {
                    asked:
                        jig_client::Event::Watch {
                            handle: watch,
                            subject: jig_client::Watchable::Jig(Subject::Task { task: 8 }),
                        },
                } = request
                {
                    assert!(task_watch.replace(*watch).is_none());
                }
            }
            let watch = task_watch.unwrap();
            crate::tests::tree::pages::deliver(&mut domain, watch, Delivery::Task(Box::new(row.clone())));
            let (tree, bindings) = crate::tests::tree::pages::build(&domain, 3, words());
            crate::tests::tree::cards_access::check(&tree);
            assert!(tree.len() <= crate::task_nodes(&CLIENT, &(CLIENT).client, 0).unwrap());
            goldens.check(&tree);
            if section == TaskSection::Overview || section == TaskSection::Panels {
                let mut parts = 0;
                for node in tree.nodes() {
                    if let Some(slot) = node.slot {
                        assert_eq!(bindings.part(slot.token), Some(&reference));
                        parts += 1;
                    }
                }
                assert_eq!(parts, u32::from(present));
            }
        }
        let (mut domain, id, _) = loaded_with_watch(ObjectKey::Task(8), ReadValue::Task(Box::new(row.clone())));
        drive(&mut domain, Event::Act { action: Action::Intend { object: id.token(), act: ActKind::Cancel } });
        let mut bindings = JigBindings::new(limits().bindings, CLIENT.client.text);
        let tree = dialog(&domain, words(), &mut bindings);

        goldens.check(&tree);
        let Some(crate::state::objects::Body::Task(retained)) = &domain.objects().get(id).unwrap().body else {
            panic!("whole task");
        };
        assert_eq!(retained.context, row.context);
        assert!(retained.chip.title.cut);
    }
}
