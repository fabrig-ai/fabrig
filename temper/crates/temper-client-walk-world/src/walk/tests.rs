use super::*;
use skein_ui::{Builder, Limits, View};
const LIMITS: Limits = Limits { nodes: 8, depth: 2, text_per_node: 32, text_bytes: 128, patches: 48, ..Limits::EMPTY };
fn tree(revealed: bool) -> View {
    let mut view = View::new(&LIMITS);
    let mut b: Builder = view.builder();
    b.open(Role::Page);
    for role in [Role::TextArea, Role::TextField, Role::Check] {
        b.open(role);
        b.name(b"Control");
        b.close();
    }
    if revealed {
        b.open(Role::Button);
        b.name(b"Revealed");
        b.close();
    }
    b.close();
    view.finish(b, 1, &mut skein_lib::Queue::with_capacity(LIMITS.patches));
    view
}
fn lines(source: &[&str]) -> Vec<Numbered> {
    source
        .iter()
        .enumerate()
        .map(|(index, line)| Numbered {
            number: u32::try_from(index + 1).expect("line number"),
            line: temper_client_walk_lines::line::Line::parse(line.as_bytes()).expect("line"),
        })
        .collect()
}
#[test]
fn links_use_the_marked_tree_first_act_exact_role_name_order_and_lead() {
    let marked = tree(false);
    let typed = tree(true);
    let following = lines(&[
        "web input textbox \"Control\" \"typed\"",
        "web press button \"Revealed\"",
        "web toggle checkbox \"Control\"",
        "web mark",
    ]);
    assert!(
        link(marked.tree(), &following).0.is_empty(),
        "typing's revealed control is absent at the mark; no later act is substituted"
    );
    assert_eq!(link(typed.tree(), &following).0.len(), 1, "the same find sees it after typing");
    let (nodes, lead) = link(marked.tree(), &lines(&["web submit textbox \"Control\" 1", "web mark"]));
    assert_eq!(nodes.len(), 1);
    assert_eq!(marked.tree().find(nodes[0]).expect("found node").role, Role::TextField);
    assert_eq!(lead, Lead::Submit);
    let (nodes, lead) = link(marked.tree(), &lines(&["web toggle checkbox \"Control\"", "web mark"]));
    assert_eq!(nodes.len(), 1);
    assert_eq!(lead, Lead::Toggle);
    assert!(
        link(marked.tree(), &lines(&["web submit textbox \"control\"", "web mark"])).0.is_empty(),
        "whole case-sensitive names"
    );
    assert!(
        link(marked.tree(), &lines(&["web submit textbox \"Control\""])).0.is_empty(),
        "a final mark has no next page"
    );
}
