//! Bare retained proposal fragments remain passive and preserve their old history position.
use crate::tests::kit::Goldens;
use crate::tests::tree::fragments::history::*;

use skein_ui::Role;

fn render(value: &ProposalHistory, words: &Words, limits: &Limits) -> (Tree, JigBindings) {
    let mut bindings = JigBindings::new(limits.bindings, 8);
    let mut builder = Builder::new(limits);
    view::proposal(&mut builder, value, words, &mut bindings, client_limits().client.rows);
    (builder.finish(), bindings)
}
fn bare(value: &ProposalHistory, words: &Words) -> (Tree, JigBindings, Limits) {
    let source = value.clone();
    let fragment = crate::tests::render_limits(&client_limits(), words).unwrap();

    let (tree, bindings) = render(value, words, &fragment);
    assert_eq!(source, *value);
    assert!(tree.len() <= fragment.nodes);
    let mut bytes = 0_u32;
    let mut slots = 0_u32;
    for node in tree.nodes() {
        assert!(node.text_bytes() <= fragment.text_per_node);
        bytes = bytes.checked_add(node.text_bytes()).unwrap();
        if let Some(name) = &node.name {
            assert!(name.len() <= usize::try_from(fragment.text_per_node).unwrap());
            bytes = bytes.checked_add(u32::try_from(name.len()).unwrap()).unwrap();
        }
        assert!(node.binding.is_none(), "retained facts supply no current controls");
        assert_ne!(node.role, Role::Button);
        assert_ne!(node.role, Role::TextField);
        assert_ne!(node.role, Role::TextArea);
        assert_ne!(node.role, Role::Choice);
        if let Some(slot) = node.slot {
            slots = slots.checked_add(1).unwrap();
            assert_eq!(bindings.owner_part(slot.token), Some(&part()));
        }
    }
    assert!(bytes <= fragment.text_bytes);
    assert!(slots <= fragment.slots);
    assert!(slots <= fragment.bindings);
    (tree, bindings, fragment)
}
fn states(action: RetainedAction, goldens: &mut Goldens) {
    let words = normal_words();
    for state in [
        ProposalHistoryState::Pending {
            holder: HistoryHolder::Policy { project: u32::MAX, kind: MessageProposalKind::Amend },
            since: Wall::EPOCH,
        },
        ProposalHistoryState::Accepted { by: Requester::Task(u64::MAX) },
        ProposalHistoryState::Rejected {
            by: Requester::Deployment { project: u32::MAX },
            reason: doc(UTF8_TEXTS[0], true),
        },
        ProposalHistoryState::Withdrawn,
    ] {
        let value = ProposalHistory { state, as_holder: false, ..proposal(action.clone()) };
        let (tree, _, _) = bare(&value, words);
        goldens.check(&tree);
    }
}
#[test]
fn retained_proposal_batch_fragment_keeps_full_nested_values_in_every_state_without_current_controls() {
    let mut goldens = Goldens::new("tree/fragments/history/proposal_fragment/retained_proposal_batch");
    states(actions().into_iter().next().unwrap(), &mut goldens);
}
#[test]
fn retained_proposal_effect_fragment_keeps_full_width_raw_coordinates_in_every_state_without_scope() {
    let mut goldens = Goldens::new("tree/fragments/history/proposal_fragment/retained_proposal_effect");
    states(actions().into_iter().nth(1).unwrap(), &mut goldens);
    let action = RetainedAction::ProcedureEffect {
        part: part(),
        authority: state().authority,
        step: u64::MAX,
        purpose: u64::MAX,
    };
    states(action.clone(), &mut goldens);
    let words = normal_words();
    let (tree, _, _) = bare(&proposal(action), words);
    goldens.check(&tree);
}
#[test]
fn retained_proposal_amend_fragment_keeps_boxed_optional_replacements_in_every_state() {
    let mut goldens = Goldens::new("tree/fragments/history/proposal_fragment/retained_proposal_amend");
    states(actions().into_iter().nth(2).unwrap(), &mut goldens);
}
#[test]
fn retained_proposal_widen_fragment_keeps_whole_authority_in_every_state() {
    let mut goldens = Goldens::new("tree/fragments/history/proposal_fragment/retained_proposal_widen");
    states(actions().into_iter().nth(3).unwrap(), &mut goldens);
}
#[test]
fn retained_proposal_release_fragment_keeps_named_task_in_every_state() {
    let mut goldens = Goldens::new("tree/fragments/history/proposal_fragment/retained_proposal_release");
    states(actions().into_iter().nth(4).unwrap(), &mut goldens);
}
#[test]
fn retained_proposal_fragment_delegation_preserves_old_history_nodes_metadata_geometry_and_slots() {
    let words = normal_words();
    let old_context =
        HistoryLine { amendment: Some((Box::new(state()), Box::new(state()))), at: Some(Wall::EPOCH), ..line() };
    let (prefix, _) = build(&old_context, words);
    let original_bound = crate::tests::render_limits(&client_limits(), words).unwrap();
    for action in actions() {
        let value = proposal(action);
        let (fragment_tree, _, fragment) = bare(&value, words);
        let history = HistoryLine { proposal: Some(value), ..old_context.clone() };

        assert_eq!(fragment, original_bound);
        let (whole, _) = build(&history, words);
        let mut heading = prefix.nodes()[0].clone();
        heading.size = whole.nodes()[0].size;
        assert_eq!(heading, whole.nodes()[0]);
        assert_eq!(&whole.nodes()[1..prefix.nodes().len()], &prefix.nodes()[1..]);
        assert_eq!(&whole.nodes()[prefix.nodes().len()..], fragment_tree.nodes());
        assert_eq!(whole.len(), prefix.len().checked_add(fragment_tree.len()).unwrap());
    }
}
#[test]
fn retained_proposal_bare_fragment_uses_the_selected_checked_language_for_every_retained_action() {
    let mut goldens = Goldens::new("tree/fragments/history/proposal_fragment/retained_proposal_bare");
    let words = words(true);
    for action in actions() {
        let value = proposal(action);
        let (tree, _, _) = bare(&value, &words);
        goldens.check(&tree);
    }
}
#[test]
fn retained_proposal_fragment_selected_language_and_exact_occupied_nodes_refuse_one_under() {
    let mut goldens = Goldens::new("tree/fragments/history/proposal_fragment/retained_proposal_fragment_selected");
    let words = words(true);
    let value = proposal(RetainedAction::Amend { task: 0, change: Box::new(delta()) });
    let (tree, _, fragment) = bare(&value, &words);
    goldens.check(&tree);
    let exact = Limits { nodes: tree.len(), ..fragment };
    let (again, _) = render(&value, &words, &exact);
    assert_eq!(again.nodes(), tree.nodes());
    let one_under = Limits { nodes: exact.nodes.checked_sub(1).unwrap(), ..exact };
    match std::panic::catch_unwind(core::panic::AssertUnwindSafe(|| render(&value, &words, &one_under))) {
        Err(_) => {}
        Ok(_) => panic!("one node below the occupied fragment must refuse"),
    }
}
#[test]
fn retained_proposal_fragment_preserves_absent_empty_and_independent_retained_losses() {
    let mut goldens = Goldens::new("tree/fragments/history/proposal_fragment/retained_proposal_fragment_preserves");
    let words = normal_words();
    let absent = RetainedAmendment {
        addressed: None,
        instructions: None,
        release: None,
        spec: None,
        authority: None,
        wake: None,
        dependencies: None,
        reason: doc(b"", false),
        cut: false,
    };
    let empty = RetainedAmendment {
        instructions: Some(doc(b"", true)),
        dependencies: Some(List::with_capacity(3)),
        cut: true,
        ..absent.clone()
    };
    let complete_empty = RetainedAmendment { instructions: Some(doc(b"", false)), cut: false, ..empty.clone() };
    let local_loss = RetainedAmendment { cut: false, ..empty.clone() };
    let dependency_loss = RetainedAmendment { cut: true, ..complete_empty.clone() };
    for (change, supplied, text_cut, dependency_cut) in [
        (absent, false, false, false),
        (complete_empty, true, false, false),
        (local_loss, true, true, false),
        (dependency_loss, true, false, true),
        (empty, true, true, true),
    ] {
        let value = proposal(RetainedAction::Amend { task: 0, change: Box::new(change) });
        let (tree, _, _) = bare(&value, words);
        match &value.action {
            RetainedAction::Amend { change, .. } => {
                assert_eq!(change.instructions.is_some(), supplied);
                assert_eq!(change.dependencies.is_some(), supplied);
                if let Some(instructions) = &change.instructions {
                    assert_eq!(instructions.bytes(), []);
                    assert_eq!(instructions.cut, text_cut);
                }
                assert_eq!(change.cut, dependency_cut);
                if let Some(values) = &change.dependencies {
                    assert!(values.is_empty());
                    assert_eq!(values.capacity(), 3);
                }
            }
            RetainedAction::Settings(_)
            | RetainedAction::Batch(_)
            | RetainedAction::Effect { .. }
            | RetainedAction::ProcedureEffect { .. }
            | RetainedAction::Widen { .. }
            | RetainedAction::Release { .. } => unreachable!(),
        }
        goldens.check(&tree);
    }
}

const UTF8_TEXTS: [&[u8]; 2] = [b"~\0", &[b'~'; 8]];
