#![expect(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "ordinary public tree proofs inspect nodes and decoded meanings"
)]
use crate::tests::tree::cards::{CLIENT, contains, words};
use crate::tests::tree::cards_objects::{loaded, task_doc};
use crate::{JigBindings, decode, tree::cards};
use skein_ui::Act;
use skein_ui::{Builder, Role};

use crate::state::action::Act as Action;
use jig_client::contract::*;
use skein_lib::{List, Wall};

const KINDS: [ActKind; 23] = [
    ActKind::Message,
    ActKind::Answer,
    ActKind::Take,
    ActKind::HandBack,
    ActKind::Accept,
    ActKind::Reject,
    ActKind::PassUp,
    ActKind::Release,
    ActKind::LeaveHeld,
    ActKind::Amend,
    ActKind::Withdraw,
    ActKind::Stop,
    ActKind::Cancel,
    ActKind::Move,
    ActKind::Reorder,
    ActKind::Seen,
    ActKind::EditNote,
    ActKind::DeleteNote,
    ActKind::Adopt,
    ActKind::ChangePolicy,
    ActKind::SetRoles,
    ActKind::MakeService,
    ActKind::SetPool,
];
#[test]
fn every_closed_offer_control_and_whole_refusal_fits_the_public_card_tree_with_two_paged_rows() {
    assert!(u32::try_from(KINDS.len()).unwrap() <= MAX_OFFERED_ACTS);
    let w = words();
    let mut limits = crate::tests::render_limits(&CLIENT, w).unwrap();
    limits.nodes = crate::card_nodes(&CLIENT, &(CLIENT).client).unwrap() + 1;
    limits.text_bytes = limits.nodes * limits.text_per_node;
    for refused in [false, true] {
        let mut value = task_doc(&KINDS);
        if refused {
            let reason = HoldReason::Tries {
                failures: {
                    let mut failures = List::with_capacity(CLIENT.client.rows);
                    for attempt in [0, u64::MAX] {
                        failures
                            .push(AttemptFailure {
                                attempt,
                                class: FailureClass::HostLost,
                                reason: crate::tests::tree::cards::text(b"whole failure"),
                                at: Wall::from_nanos(u64::MAX),
                            })
                            .unwrap();
                    }
                    failures
                },
            };
            for index in 0..value.acts.len() {
                let entry = value.acts.get_mut(index).unwrap();
                entry.admission = Admission::Refused(Refusal::Moved {
                    now: Revision { value: u64::MAX },
                    what: Moved::Hold { reason: Some(CurrentHold::Document(Box::new(reason.clone()))) },
                });
            }
        }
        let before = value.clone();
        let (domain, id) = loaded(ObjectKey::Task(8), ReadValue::Task(Box::new(value.clone())));
        let mut bindings = JigBindings::new(limits.bindings, CLIENT.client.text);
        let mut builder = Builder::new(&limits);
        builder.open(Role::Page);
        cards::card(&mut builder, &domain.view(), id, &cards::Place::Elsewhere, w, &mut bindings);
        builder.close();
        let tree = builder.finish();
        crate::tests::tree::cards_access::check(&tree);
        assert!(tree.len() <= limits.nodes);
        let mut total = 0_u64;
        let mut decoded = Vec::new();
        for node in tree.nodes() {
            assert!(crate::tests::text_bytes(node) <= limits.text_per_node);
            total += u64::from(crate::tests::text_bytes(node));
            if node.role == Role::Button
                && let Some(binding) = node.binding
                && let Some(Action::Intend { object, act }) = decode(&mut bindings, binding.token, Act::Press)
            {
                assert_eq!(object, id.token());
                decoded.push(act);
            }
        }
        assert!(total <= u64::from(limits.text_bytes));
        if refused {
            assert!(decoded.is_empty());
            assert!(contains(&tree, b"whole failure"));
            assert!(contains(&tree, b"18446744073709551615"));
            let revisions = tree
                .nodes()
                .iter()
                .filter(|node| {
                    node.text.iter().any(|run| skein_lib::bytes::find(&run.text, b"18446744073709551615").is_some())
                })
                .count();
            assert!(revisions >= usize::try_from(MAX_OFFERED_ACTS).unwrap());
        } else {
            assert_eq!(decoded, KINDS);
        }
        assert_eq!(value, before);
        assert_eq!(CLIENT.client.rows, 2);
    }
}
