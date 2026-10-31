//! Instruction source presence and independent amendment choices (objects.md, 11).
use crate::state::{
    address::{Address, TaskSection},
    objects::Body,
};
use crate::tests::kit::Goldens;
use crate::tests::tree::cards_objects::*;

fn sources() -> [Option<DocumentText>; 6] {
    [
        Some(DocumentText::complete(text(b"supplied local effective text"))),
        None,
        Some(DocumentText::complete(text(b""))),
        Some(DocumentText { bytes: text(b""), cut: true }),
        Some(DocumentText { bytes: text(b"supplied partial text"), cut: true }),
        Some(DocumentText::complete(text(b"supplied charter effective text"))),
    ]
}
fn sent(out: &[Request]) -> &Amendment {
    out.iter()
        .find_map(|request| match request {
            Request::Client { asked: jig_client::Event::Send { request: jig_client::Ask::Jig(ask), .. } } => {
                match &**ask {
                    Ask::Amend { change, .. } => Some(change.as_ref()),
                    Ask::Compound { .. } => crate::tests::fixtures::selected_change(ask),
                    _ => None,
                }
            }
            _ => None,
        })
        .unwrap()
}

fn baseline(tree: &Tree, source: Option<&DocumentText>) {
    let node = tree.nodes().iter().find(|node| node.name.as_deref() == Some(b"New instructions")).unwrap();
    let expected = source.filter(|value| !value.cut).map_or(b"".as_slice(), DocumentText::bytes);
    assert_eq!(node.value.as_ref().map_or(b"".as_slice(), |value| &*value.text), expected);
}

#[test]
fn task_instruction_updates_distinguish_supplied_effective_empty_unknown_and_partial_text() {
    let mut goldens = Goldens::new("tree/cards_objects/instruction_facts/task_instruction_updates");
    let (mut domain, out) =
        crate::tests::tree::work_pages::entered(Address::Task { task: 8, section: TaskSection::Overview });
    let watch =
        out.iter()
            .find_map(|request| match request {
                Request::Client {
                    asked:
                        jig_client::Event::Watch {
                            handle: watch,
                            subject: jig_client::Watchable::Jig(Subject::Task { .. }),
                        },
                } => Some(*watch),
                _ => None,
            })
            .unwrap();
    for source in sources() {
        let mut row = task_doc(&[]);
        row.chip.executor = Executor::Agent { charter: 2 };
        row.instructions = source.clone();
        crate::tests::tree::pages::deliver(&mut domain, watch, Delivery::Task(Box::new(row)));
        let tree = crate::tests::tree::pages::build(&domain, 3, words()).0;
        goldens.check(&tree);
        let id = domain.objects().find(ObjectKey::Task(8)).unwrap();
        let projected = domain.objects().get(id).unwrap();
        let Body::Task(retained) = projected.body.as_ref().unwrap() else {
            panic!("task");
        };
        assert_eq!(retained.instructions, source);
    }
}

#[test]
fn task_instruction_amendment_uses_only_complete_source_without_selecting_it() {
    let mut goldens = Goldens::new("tree/cards_objects/instruction_facts/task_instruction_amendment");
    for source in sources() {
        let mut row = task_doc(&[ActKind::Amend]);
        row.instructions = source.clone();
        let (mut domain, id) = loaded(ObjectKey::Task(8), ReadValue::Task(Box::new(row.clone())));
        drive(&mut domain, Event::Act { action: Action::Intend { object: id.token(), act: ActKind::Amend } });
        drive(&mut domain, Event::Act { action: Action::Edit { field: Purpose::Reason, text: text(b"") } });
        let mut bindings = JigBindings::new(limits().bindings, CLIENT.client.text);
        let tree = dialog(&domain, words(), &mut bindings);
        goldens.check(&tree);
        baseline(&tree, source.as_ref());
        assert!(domain.confirmation().unwrap().choices.change.is_none());
        drive(&mut domain, Event::Act { action: input(&tree, b"New budget", b"25", &mut bindings) });
        let mut expected = Amendment {
            llm: None,
            addressed: None,
            spec: None,
            instructions: None,
            wake: None,
            dependencies: None,
            authority: Some(row.authority.whole().unwrap()),
            tracked: None,
        };
        expected.authority.as_mut().unwrap().spend = 25;
        assert_eq!(domain.confirmation().unwrap().choices.change.as_ref(), Some(&expected));
        let tree = dialog(&domain, words(), &mut bindings);
        baseline(&tree, source.as_ref());
        drive(&mut domain, Event::Act { action: input(&tree, b"New instructions", b"", &mut bindings) });
        expected.instructions = Some(text(b""));
        let out = drive(&mut domain, Event::Act { action: Action::Confirm });
        assert_eq!(sent(&out), &expected);
        let projected = domain.objects().get(id).unwrap();
        let Body::Task(retained) = projected.body.as_ref().unwrap() else {
            panic!("task");
        };
        assert_eq!(retained.instructions, source);
    }
}
