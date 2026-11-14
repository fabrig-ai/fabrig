//! Finished tree prices (skein's ui/README.md, section 13).

use crate::{Binding, Builder, Limits, Node, Owner, Role, Run, SlotRef, Tree};
use skein_lib::{Token, bytes::copy_of};

const OWNER: Owner = Owner { number: 1 };
const BINDING: Binding = Binding { owner: OWNER, token: Token::new(1) };
// A root with two leaves, one carrying a five-byte name and a binding.
const NAMED: Limits = Limits { nodes: 3, depth: 2, text_per_node: 5, text_bytes: 5, bindings: 1, ..Limits::EMPTY };

#[test]
fn hand_built_tree_prices_nodes_depth_text_and_binding() {
    let mut tree = Tree::new(NAMED.nodes);
    assert_eq!(tree.used(), Limits::EMPTY);
    let mut root = Node::vacant();
    root.role = Role::Group;
    root.size = NAMED.nodes;
    tree.nodes.push(root).unwrap();
    let mut text = Node::vacant();
    text.name = Some(copy_of(b"hello"));
    text.binding = Some(BINDING);
    tree.nodes.push(text).unwrap();
    tree.nodes.push(Node::vacant()).unwrap();
    assert_eq!(tree.used(), NAMED);
}

#[test]
fn slot_price_includes_reference_and_loading_fallback() {
    // Slot and status under a root; "busy" is the fallback's only text.
    const PRICE: Limits = Limits { nodes: 3, depth: 3, text_per_node: 4, text_bytes: 4, slots: 1, ..Limits::EMPTY };
    let mut builder = Builder::new(&PRICE);
    builder.open(Role::Group);
    builder.slot(SlotRef { owner: OWNER, token: Token::new(2) }, b"busy");
    builder.close();
    assert_eq!(builder.finish().used(), PRICE);
}

fn one_pass(limits: &Limits) -> Tree {
    let mut builder = Builder::new(limits);
    builder.open(Role::Group);
    builder.open(Role::Group);
    builder.open(Role::Link);
    builder.link(BINDING);
    builder.bind(BINDING);
    builder.name(b"longer name replaced");
    builder.name(b"hello");
    builder.close();
    builder.close();
    builder.open(Role::TextField);
    builder.name(b"name");
    builder.description(b"desc");
    builder.placeholder(b"hint");
    builder.hint(b"tip");
    builder.value(b"value", 0);
    builder.bind(BINDING);
    builder.close();
    builder.text(Run::Plain, b"words");
    builder.close();
    // A second root checks that completed subtrees no longer add depth.
    builder.open(Role::Group);
    builder.close();
    builder.finish()
}

#[test]
fn one_pass_prices_match_exact_builder_charges() {
    // Six nodes, deepest at the link; field bytes 4+4+4+3+5 = 20,
    // plus five bytes each on the link and text. The link's two tokens
    // share one builder binding charge; the field carries the other.
    const PRICE: Limits = Limits { nodes: 6, depth: 3, text_per_node: 20, text_bytes: 30, bindings: 2, ..Limits::EMPTY };
    let tree = one_pass(&super::fixtures::LIMITS);
    assert_eq!(tree.used(), PRICE);
    assert_eq!(one_pass(&tree.used()).used(), PRICE);
}
