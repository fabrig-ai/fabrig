//! The live tree's revisions mapped by the host page part (skein/docs/design/ui/web/testing.md, 3).
use skein_lib::Queue;
use skein_ui::{Act, NodeId};
use skein_ui_example_view::tree::{self, Case, Tree};
use skein_ui_web::{
    Look,
    mapping::Primary,
    page::{Op, Page},
};

const PATCH_BUTTON: NodeId = NodeId(3); // Page, main, then their first control.

const MOUNT: &[&str] = &["insert 1 in 0 before end"]; // Install the initial subtree at the mount.

// The goldens include data-node: equal markup with recreated survivors is not enough.
fn check(case: Case, expected_ops: &[&[&str]], golden: &str) {
    let limits = tree::limits();
    let mut tree = Tree::new(case);
    let mut page = Page::new(
        Look { names: &[], components: &[], pictures: &[], elements: &[], draw: skein_ui_example_view::draw::draw },
        &limits,
        b"notes",
        Primary::Control,
    );
    let mut queue = Queue::with_capacity(limits.patches);
    let mut golden = golden.lines();
    for (revision, expected) in expected_ops.iter().enumerate() {
        if revision != 0 {
            tree.act(PATCH_BUTTON, Act::Press);
        }
        assert!(tree.patch(&mut queue), "delivery for revision {revision}");
        let patches: Vec<_> = std::iter::from_fn(|| queue.pop()).collect();
        let mut ops = Vec::new();
        page.apply(&patches, &mut ops);
        let ops: Vec<_> = ops.iter().map(operation).collect();
        assert_eq!(ops, *expected, "operations at revision {revision}");
        assert_eq!(page.html(), golden.next().expect("golden line for each revision"), "HTML at revision {revision}");
        assert!(!tree.patch(&mut queue), "one delivery at revision {revision}");
    }
    assert_eq!(golden.next(), None, "all golden revisions checked");
}

fn operation(op: &Op) -> String {
    let placement = |node: NodeId, parent: NodeId, before: Option<NodeId>, shown_folded| {
        let anchor = before.map_or_else(|| "end".to_owned(), |node| node.0.to_string());
        let place = if shown_folded { " in summary" } else { "" };
        format!("{} in {} before {anchor}{place}", node.0, parent.0)
    };
    match op {
        Op::Insert { node, parent, before, shown_folded, .. } => {
            format!("insert {}", placement(*node, *parent, *before, *shown_folded))
        }
        Op::Move { node, parent, before, shown_folded } => {
            format!("move {}", placement(*node, *parent, *before, *shown_folded))
        }
        Op::Remove { node } => format!("remove {}", node.0),
        Op::Attributes { node, .. } => format!("attributes {}", node.0),
        Op::Text { node, .. } => format!("text {}", node.0),
        Op::Focus { node } => format!("focus {}", node.0),
        Op::Reveal { node } => format!("reveal {}", node.0),
        other => format!("unexpected operation {other:?}"),
    }
}

#[test]
fn the_first_child_creates_a_tree_group() {
    check(
        Case::First,
        &[MOUNT, &["insert 13 in 7 before end", "attributes 4", "text 5"]],
        include_str!("../../golden/tree-first.html"),
    );
}

#[test]
fn a_child_is_inserted_before_its_keyed_sibling() {
    check(
        Case::Middle,
        &[MOUNT, &["insert 19 in 7 before 13", "attributes 4", "text 5"]],
        include_str!("../../golden/tree-middle.html"),
    );
}

#[test]
fn moving_keyed_children_keeps_their_node_ids() {
    check(
        Case::Move,
        &[
            MOUNT,
            &[
                "move 13 in 7 before end",
                "move 9 in 7 before 13",
                "move 15 in 7 before 9",
                "move 8 in 7 before 15",
                "attributes 4",
                "text 5",
            ],
        ],
        include_str!("../../golden/tree-move.html"),
    );
}

#[test]
fn renaming_a_keyed_child_keeps_its_node_id() {
    check(
        Case::Rename,
        &[MOUNT, &["attributes 4", "text 5", "attributes 13", "text 14"]],
        include_str!("../../golden/tree-rename.html"),
    );
}

#[test]
fn removing_the_last_child_removes_its_group() {
    check(Case::Last, &[MOUNT, &["remove 9", "attributes 4", "text 5"]], include_str!("../../golden/tree-last.html"));
}

#[test]
fn removing_the_refreshed_form_removes_its_label() {
    check(
        Case::Form,
        &[
            &["insert 1 in 0 before end", "focus 8", "reveal 8"],
            &["attributes 4", "text 5", "attributes 6", "attributes 7"],
            &["remove 7", "focus 6", "attributes 4", "text 5"],
        ],
        include_str!("../../golden/tree-form.html"),
    );
}

#[test]
fn a_paragraph_splits_then_rejoins_keyed_runs() {
    check(
        Case::Mixed,
        &[
            MOUNT,
            &["insert 19 in 7 before 13", "attributes 4", "text 5"],
            &["insert 23 in 7 before end", "insert 21 in 7 before 19", "attributes 4", "text 5"],
            &["remove 19", "attributes 4", "text 5"],
        ],
        include_str!("../../golden/tree-mixed.html"),
    );
}

#[test]
fn the_folded_group_patches_and_moves_through_the_summary() {
    check(
        Case::ShownFolded,
        &[
            MOUNT,
            &["insert 12 in 6 before 10 in summary", "attributes 4", "text 5"],
            &["attributes 4", "text 5", "attributes 6", "text 8"],
            &["remove 12", "attributes 4", "text 5"],
            &["attributes 4", "text 5", "attributes 7", "move 7 in 6 before 10"],
            &["attributes 4", "text 5", "attributes 7", "move 7 in 6 before 10 in summary"],
        ],
        include_str!("../../golden/tree-shown-folded.html"),
    );
}
