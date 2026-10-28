//! Translate temper's visible card vocabulary into the common referee's observations.
use jig_client::contract::{ObjectKey, ProposalKey, Proposer};
use jig_client_world::{
    harness::{CardKey, CardSeen, Seen},
    observe,
};
use skein_ui::{Key, NodeId, Role, Tree};
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
        if value.role != Role::Item || index.checked_add(usize::try_from(value.size).ok()?)? <= at {
            return None;
        }
        identity(value.key?)
    })
}
pub fn seen(tree: &Tree) -> Vec<Seen> {
    let mut seen = Vec::new();
    // A dependency line's items are links keyed by their task, not cards (client/README.md, 5.8).
    let mut links_end = 0;
    for (index, node) in tree.nodes().iter().enumerate() {
        if node.styles.as_slice().contains(&temper_client_view::look::DEPENDENCIES) {
            links_end = index.checked_add(usize::try_from(node.size).expect("link subtree fits")).expect("tree fits");
        }
        // The editor is the task's visible decision, named by its number, as
        // "Hold revision N" names a card's revision (client/README.md, 5.11).
        let amending =
            node.role == Role::Region && node.styles.as_slice().contains(&temper_client_view::look::TASK_AMEND);
        if (node.role != Role::Item && !amending) || index < links_end {
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
