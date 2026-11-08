//! Options attached to the nearest field in their innermost form
//! (skein/docs/design/ui.md, 3.2; skein/docs/design/ui-screen.md, 5.3, 5.4 and 6.2).

use skein_ui::{Node, NodeId, Role, State, Tree};

/// The field an options menu follows, excluding fields in nested forms.
#[must_use]
pub fn field(tree: &Tree, menu: NodeId) -> Option<&Node> {
    let mut index = None;
    for at in 0..tree.len() {
        let node = tree.get(at).expect("preorder node exists");
        if node.id == menu {
            if node.role != Role::Menu || !node.states.as_slice().contains(&State::Options) {
                return None;
            }
            index = Some(at);
            break;
        }
    }
    let menu = index?;
    // A closed nested form must not hide an enclosing form.
    let mut enclosing = None;
    for at in 0..menu {
        let node = tree.get(at).expect("node before menu exists");
        if node.role == Role::Form && at.saturating_add(node.size) > menu {
            enclosing = Some(at);
        }
    }
    let form = enclosing?;
    let mut field = None;
    let mut skip = 0;
    for at in form.saturating_add(1)..menu {
        if at < skip {
            continue;
        }
        let node = tree.get(at).expect("node before menu exists");
        if node.role == Role::Form {
            skip = at.saturating_add(node.size);
        } else if node.role.is_text_field() {
            field = Some(node);
        }
    }
    field
}

/// The first options menu following this field in its form.
#[must_use]
pub fn menu(tree: &Tree, focused: Option<NodeId>) -> Option<&Node> {
    let focused = focused?;
    let node = tree.find(focused)?;
    if !node.role.is_text_field() || tree.disabled(focused) || tree.bound(focused).is_none() {
        return None;
    }
    for node in tree.nodes() {
        if node.role == Role::Menu
            && !tree.disabled(node.id)
            && let Some(field) = field(tree, node.id)
            && field.id == focused
        {
            return Some(node);
        }
    }
    None
}

/// Keeps an enabled item, otherwise the selected enabled item or the first.
#[must_use]
pub fn active(tree: &Tree, menu: NodeId, current: Option<NodeId>) -> Option<NodeId> {
    let mut inside = false;
    let mut left = 0;
    let mut first = None;
    let mut selected = None;
    for node in tree.nodes() {
        if node.id == menu {
            inside = true;
            left = node.size.saturating_sub(1);
            continue;
        }
        if !inside {
            continue;
        }
        if left == 0 {
            break;
        }
        left = left.saturating_sub(1);
        if node.role == Role::MenuItem && !tree.disabled(node.id) {
            if current == Some(node.id) {
                return current;
            }
            if first.is_none() {
                first = Some(node.id);
            }
            if selected.is_none() && node.states.as_slice().contains(&State::Selected) {
                selected = Some(node.id);
            }
        }
    }
    selected.or(first)
}
