//! Translate temper's visible card vocabulary into the common referee's observations.
use jig_client::contract::{ObjectKey, ProposalKey, Proposer};
use jig_client_kit::{
    harness::{CardKey, CardSeen, Seen},
    observe,
};
use skein_ui::{Key, NodeId, Role, Tree};
fn card_role(node: &skein_ui::Node) -> bool {
    (node.role == Role::Slot && node.styles.as_slice().contains(&temper_client_view::look::TEAM_CARD))
        || node.role == Role::Item
        || (node.role == Role::Slot && node.key.is_some_and(|key| key.kind == 1 || key.kind == 29))
}
fn identity(key: Key) -> Option<CardKey> {
    if key.owner != temper_client_view::frame::owner() {
        return None;
    }
    let object = match key.kind {
        0 => ObjectKey::Task(key.a),
        1 => ObjectKey::Proposal(ProposalKey { proposer: Proposer::Task(key.a), number: key.b }),
        2 => ObjectKey::Escalation(key.a),
        3 => ObjectKey::Question { task: key.a, message: key.b },
        4 => ObjectKey::PersonTask(key.a),
        5 => ObjectKey::Result(key.a),
        6 => ObjectKey::Note { project: u32::try_from(key.a).ok()?, note: key.b },
        29 => ObjectKey::Proposal(ProposalKey { proposer: Proposer::Party(key.a), number: key.b }),
        _ => return None,
    };
    Some(observe::key(object))
}
pub fn enclosing(tree: &Tree, node: NodeId) -> Option<CardKey> {
    let at = tree.nodes().iter().position(|value| value.id == node)?;
    tree.nodes().iter().enumerate().take(at.checked_add(1)?).rev().find_map(|(index, value)| {
        if !card_role(value) || index.checked_add(usize::try_from(value.size).ok()?)? <= at {
            return None;
        }
        identity(value.key?)
    })
}
pub fn seen(tree: &Tree) -> Vec<Seen> {
    let mut seen = Vec::new();
    // A dependency line's items are links keyed by their task, not cards (client/pages/goal.md, 2).
    let mut links_end = 0;
    for (index, node) in tree.nodes().iter().enumerate() {
        if node.styles.as_slice().contains(&temper_client_view::look::DEPENDENCIES)
            || node.styles.as_slice().contains(&temper_client_view::look::CONFIRM_FATES)
        {
            links_end = index.checked_add(usize::try_from(node.size).expect("link subtree fits")).expect("tree fits");
        }
        // A taskbar or chat bar remains the release surface on an own pause (client/pages/README.md and client/pages/task.md, 4).
        if node.styles.as_slice().contains(&temper_client_view::look::TASKBAR)
            || node.styles.as_slice().contains(&temper_client_view::look::SESSION_TOP)
        {
            let end = index.checked_add(usize::try_from(node.size).expect("taskbar fits")).expect("tree fits");
            let task = tree.nodes()[index..end].iter().find_map(|child| {
                if child.role != (Role::Heading { level: 1 }) {
                    return None;
                }
                let title = child.name.as_deref()?.strip_prefix(b"T")?;
                let number = title.split(|byte| *byte == b' ').next()?;
                skein_ui::number(number)
            });
            if let Some(task) = task {
                seen.push(Seen::Card {
                    key: observe::key(ObjectKey::Escalation(task)),
                    state: CardSeen::Waiting,
                    revision: None,
                });
            }
        }
        // The editor is the task's visible decision, named by its number, as
        // "Hold revision N" names a card's revision (client/pages/task.md, 4).
        let amending = node.role == Role::Region
            && (node.styles.as_slice().contains(&temper_client_view::look::TASK_AMEND)
                || node.styles.as_slice().contains(&temper_client_view::look::GOAL_FIELD));
        if (!card_role(node) && !amending) || index < links_end {
            continue;
        }
        let key = if amending {
            node.name
                .as_deref()
                .and_then(|name| name.strip_prefix(b"Amend task "))
                .and_then(skein_ui::number)
                .map(|task| observe::key(ObjectKey::Task(task)))
        } else {
            node.key.and_then(identity)
        };
        let Some(key) = key else { continue };
        let end = index.checked_add(usize::try_from(node.size).expect("card subtree fits")).expect("tree fits");
        let mut state = CardSeen::Waiting;
        let mut revision = None;
        for child in &tree.nodes()[index..end] {
            if child.styles.as_slice().contains(&temper_client_view::look::SETTINGS_PROPOSAL_STATE) {
                state = CardSeen::Done;
            }

            if let Some(name) = child.name.as_deref() {
                if !amending && let Some(number) = name.strip_prefix(b"Hold revision ") {
                    revision = skein_ui::number(number);
                }
                if child.role == Role::Status {
                    state = match name {
                        b"This proposal is no longer waiting in this chat." => CardSeen::Left { reason: true },
                        b"Pending" | b"Retrying" | b"Waiting to send" => CardSeen::Deciding,
                        b"Applied" | b"Accepted" | b"Rejected" | b"Answered" | b"Released" | b"Left held"
                        | b"Taken" | b"Handed back" | b"Cancelled" => CardSeen::Done,
                        _ => state,
                    };
                }
            }
        }
        seen.push(Seen::Card { key, state, revision });
    }
    seen
}

/// Temper's sheets and immediate Pause/Resume replace jig's deciding text (client/README.md, 3.6).
/// The view's confirmation tests hold their titles, active counts, fates and retry rules.
pub fn decision(ask: &jig_client::contract::Ask) -> Option<&jig_client::contract::Ask> {
    use jig_client::contract::{Ask, DecideOn, Decision, Operation};
    // A short preference is deleted at once, without a deciding region (client/pages/remembered.md).
    if matches!(ask, Ask::DeleteNote { project: 0, scope: jig_client::contract::NoteScope::Party { .. }, .. }) {
        return None;
    }
    let sheet = match ask {
        // Pause and Resume are immediate bar acts. Their status replaces a sheet's
        // facts; the request observer still checks Resume's delivered hold revision.
        Ask::Stop { .. }
        | Ask::Release { .. }
        | Ask::Cancel { .. }
        | Ask::Decide { on: DecideOn::Proposal { decision: Decision::Rejected { .. }, .. }, .. } => true,
        Ask::Compound { operations, .. } => {
            operations.iter().any(|operation| matches!(operation, Operation::Cancel { .. }))
                || matches!(operations.as_slice(), [Operation::Message { .. }, Operation::Release { .. }])
        }
        _ => false,
    };
    if sheet { None } else { Some(ask) }
}
