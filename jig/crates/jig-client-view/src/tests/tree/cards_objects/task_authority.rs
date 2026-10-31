//! Whole owner authority remains independent of selected changes and document loss.
use crate::state::address::{Address, TaskSection};
use crate::tests::kit::Goldens;
use crate::tests::tree::cards_objects::*;

fn supplied() -> OwnerAuthorityDocument {
    let segments = crate::tests::kit::list_in(CLIENT.client.rows, [Box::from(*b"a/b"), Box::from(*b"\xff\0")]);
    let pattern = OwnerPattern { segments, last: OwnerPatternEnd::Open(Box::from(*b"\xff/")) };
    let grants = crate::tests::kit::list_in(
        CLIENT.client.rows,
        [OwnerGrant { connector: u16::MAX, kind: u16::MAX, pattern: pattern.clone() }],
    );
    let kinds = crate::tests::kit::list_in(
        CLIENT.client.rows,
        [OwnerAuthorityExecutor::Charter(u32::MAX), OwnerAuthorityExecutor::Role(u32::MAX)],
    );
    let note_resources =
        crate::tests::kit::list_in(CLIENT.client.rows, [OwnerResourceScope { connector: u16::MAX, pattern }]);
    OwnerAuthorityDocument {
        authority: OwnerAuthority {
            models: None,
            tools: u64::MAX,
            grants,
            delegation: OwnerDelegation { kinds, tasks: u32::MAX, depth: u32::MAX },
            spend: 10,
            deadline: None,
            notes: u8::MAX,
            note_resources,
        },
        cut: false,
    }
}

#[test]
fn task_authority_forms_preserve_all_permissions_and_selected_delta_without_reconstructing_loss() {
    let mut goldens = Goldens::new("tree/cards_objects/task_authority/task_authority_forms");
    for cut in [false, true] {
        let mut row = task_doc(&[ActKind::Amend]);
        row.authority = supplied();
        row.authority.cut = cut;
        let before = row.clone();
        let (mut domain, id) = loaded(ObjectKey::Task(8), ReadValue::Task(Box::new(row)));
        drive(&mut domain, Event::Act { action: Action::Intend { object: id.token(), act: ActKind::Amend } });
        drive(&mut domain, Event::Act { action: Action::Edit { field: Purpose::Reason, text: text(b"") } });
        let words = words();
        let mut bindings = JigBindings::new(limits().bindings, CLIENT.client.text);
        let tree = dialog(&domain, words, &mut bindings);
        goldens.check(&tree);
        let action = input(&tree, b"New instructions", b"", &mut bindings);
        drive(&mut domain, Event::Act { action });
        let tree = dialog(&domain, words, &mut bindings);
        if !cut {
            let action = input(&tree, b"New budget", b"25", &mut bindings);
            drive(&mut domain, Event::Act { action });
        }
        let selected = domain.confirmation().unwrap().choices.change.as_ref().unwrap();
        assert_eq!(selected.instructions, Some(text(b"")));
        assert!(selected.spec.is_none());
        assert!(selected.wake.is_none());
        if cut {
            assert!(selected.authority.is_none());
        } else {
            let mut expected = before.authority.authority.clone();
            expected.spend = 25;
            assert_eq!(selected.authority, Some(expected));
            assert_eq!(selected.authority.as_ref().unwrap().grants.capacity(), CLIENT.client.rows);
            assert_eq!(
                selected.authority.as_ref().unwrap().grants.get(0).unwrap().pattern.segments.capacity(),
                CLIENT.client.rows
            );
        }
        let expected = selected.clone();
        let out = drive(&mut domain, Event::Act { action: Action::Confirm });
        let sent: Vec<_> = out
            .iter()
            .filter_map(|request| match request {
                Request::Client { asked: jig_client::Event::Send { request: jig_client::Ask::Jig(ask), .. } } => {
                    match &**ask {
                        Ask::Amend { change, .. } => Some(change.as_ref()),
                        _ => None,
                    }
                }
                _ => None,
            })
            .collect();
        assert_eq!(sent, vec![&expected]);
        let Some(crate::state::objects::Body::Task(actual)) = &domain.objects().get(id).unwrap().body else {
            panic!("whole source remains");
        };
        assert_eq!(**actual, before);
    }
}

#[test]
fn task_budget_section_renders_exact_bitmap_delegation_and_literal_authority_loss_within_bounds() {
    let mut goldens = Goldens::new("tree/cards_objects/task_authority/task_budget_section");
    for cut in [false, true] {
        for kind in [
            OwnerAuthorityExecutor::Charter(u32::MAX),
            OwnerAuthorityExecutor::Procedure(u32::MAX),
            OwnerAuthorityExecutor::Role(u32::MAX),
        ] {
            let (mut domain, out) =
                crate::tests::tree::work_pages::entered(Address::Task { task: 8, section: TaskSection::Budget });
            let watch = out
                .iter()
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
            let mut row = task_doc(&[]);
            row.chip.executor = Executor::Agent { charter: 1 };
            row.authority = supplied();
            row.authority.cut = cut;
            *row.authority.authority.delegation.kinds.get_mut(0).unwrap() = kind;
            crate::tests::tree::pages::deliver(&mut domain, watch, Delivery::Task(Box::new(row.clone())));
            let (tree, _) = crate::tests::tree::pages::build(&domain, 3, words());
            crate::tests::tree::cards_access::check(&tree);
            goldens.check(&tree);
            assert!(tree.len() <= crate::task_nodes(&CLIENT, &(CLIENT).client, 0).unwrap());
            let id = domain.task_page().header.unwrap();
            let Some(crate::state::objects::Body::Task(actual)) = &domain.objects().get(id).unwrap().body else {
                panic!("whole source remains");
            };
            assert_eq!(**actual, row);
        }
    }
}
