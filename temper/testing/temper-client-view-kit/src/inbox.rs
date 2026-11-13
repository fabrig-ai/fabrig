use super as fixtures;
use jig_client::contract as c;
use skein_ui::{Node, Tree};
use temper_client_domain::{Domain, address::Filter};

pub fn subtree(tree: &Tree, index: usize) -> &[Node] {
    let node = &tree.nodes()[index];
    &tree.nodes()[index..index.checked_add(usize::try_from(node.size).expect("subtree size")).expect("subtree end")]
}

pub fn styled(tree: &Tree, token: skein_ui::Style) -> usize {
    for (index, node) in tree.nodes().iter().enumerate() {
        if node.styles.as_slice().contains(&token) {
            return index;
        }
    }
    panic!("styled node {token:?}");
}

pub fn question_root() -> Domain {
    let task = fixtures::task::document();
    let page = c::Question {
        task: task.chip,
        from: None,
        message: fixtures::INBOX_QUESTION,
        words: c::DocumentText::complete(c::Text::new(Box::from(*b"Which path?"), 32).expect("question")),
        since: skein_lib::Wall::EPOCH,
        acts: skein_lib::List::with_capacity(0),
        ending: None,
    };
    fixtures::inbox_with_document(
        Filter::ALL,
        Some((
            c::Read::Question { task: fixtures::INBOX_TASK, message: fixtures::INBOX_QUESTION },
            c::ReadValue::Question(Box::new(page)),
        )),
    )
}
