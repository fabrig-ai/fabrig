use crate::tests::kit::Goldens;
use crate::tests::tree::cards_objects::*;

#[test]
fn owner_hold_confirmation_keeps_exact_prior_counts_loss_clock_presence_and_repeat_bindings() {
    let mut goldens = Goldens::new("tree/cards_objects/task_hold/owner_hold_confirmation");
    let reference = held_reference();
    for uncertain in [false, true] {
        for wait in [None, Some(Wall::from_nanos(u64::MAX))] {
            for prior in [
                PriorPhase::Waiting,
                PriorPhase::Active(RunStage::Claimed { attempt: u64::MAX }),
                PriorPhase::Closing(ClosingStage::Effects),
            ] {
                let held = ChipHold {
                    words: None,
                    placed: None,
                    circle: None,
                    account: None,
                    source: Some(reference.clone()),
                    reason: if uncertain {
                        HoldCategory::Uncertain { entry: u64::MAX }
                    } else {
                        HoldCategory::Procedure
                    },
                    revision: u64::MAX,
                    before: prior,
                    escalation: Some(HoldEscalation::Rejected {
                        by: u64::MAX,
                        reason: DocumentText { bytes: text(b"exact held rejection"), cut: true },
                    }),
                    tries: FailureTries { transient: 1, permanent: 2, run: 3, agent: 4, lost: 5, invalid: u32::MAX, infrastructure: 0 },
                    hold_wait_since: wait,
                };
                let mut row = task_doc(&[ActKind::Release]);
                row.hold = Some(Box::new(held.clone()));
                row.acts.get_mut(0).unwrap().effect = None;
                let (mut domain, id, _) = loaded_with_watch(ObjectKey::Task(8), ReadValue::Task(Box::new(row)));
                drive(&mut domain, Event::Act { action: Action::Intend { object: id.token(), act: ActKind::Release } });
                let envelope = limits();
                let mut bindings = JigBindings::new(envelope.bindings, CLIENT.client.text);
                let tree = dialog(&domain, words(), &mut bindings);
                goldens.check(&tree);
                let mut choices = Vec::new();
                let mut raw_slots = 0_u32;
                for node in tree.nodes() {
                    if let Some(binding) = node.binding
                        && let Some(Action::Again { again }) = decode(&mut bindings, binding.token, Act::Press)
                    {
                        choices.push(again);
                    }
                    if let Some(slot) = node.slot {
                        assert_eq!(bindings.owner_part(slot.token), Some(&reference));
                        assert_eq!(bindings.part(slot.token), None);
                        raw_slots += 1;
                    }
                }
                assert_eq!(raw_slots, 1);
                assert_eq!(choices, if uncertain { vec![true, false] } else { vec![] });

                let Some(crate::state::objects::Body::Task(retained)) = &domain.objects().get(id).unwrap().body else {
                    panic!("whole task")
                };
                assert_eq!(retained.hold, Some(Box::new(held)));
            }
        }
    }
}

fn held_reference() -> Part {
    let mut segments = List::with_capacity(CLIENT.client.rows);
    for _ in 0..CLIENT.client.rows {
        segments.push(Box::<[u8]>::from([255_u8; 32])).unwrap();
    }
    Part { connector: u16::MAX, kind: PartKind::Resource, key: PartKey::Path(segments) }
}
