use crate::Binding;
use crate::Limits;
use crate::{Builder, DrawingKind, Key, Owner, Role, Run, State, Style};

use skein_lib::{List, Token};

use super::fixtures::LIMITS;

#[test]
fn nodes_are_preorder_and_sizes_include_descendants() {
    let mut builder = Builder::new(&LIMITS);
    builder.open(Role::Page);
    builder.open(Role::List);
    builder.open(Role::Item);
    builder.key(Key { owner: Owner { number: 65_536 }, kind: 1, a: 7, b: 0 });
    builder.text(Run::Plain, b"one");
    builder.close();
    builder.open(Role::Item);
    builder.close();
    builder.close();
    builder.close();
    let tree = builder.finish();
    let mut roles = List::with_capacity(5);
    for node in tree.nodes() {
        roles.push(node.role).expect("five nodes");
    }
    assert_eq!(roles.as_slice(), [Role::Page, Role::List, Role::Item, Role::Text, Role::Item]);
    let mut sizes = List::with_capacity(5);
    for node in tree.nodes() {
        sizes.push(node.size).expect("five nodes");
    }
    assert_eq!(sizes.as_slice(), [5, 4, 2, 1, 1]);
}

#[test]
fn a_fields_own_write_count_and_semantic_attributes_are_kept() {
    let mut builder = Builder::new(&LIMITS);
    builder.open(Role::TextField);
    builder.name(b"Amount");
    builder.value(b"42", 19);
    builder.state(State::Invalid);
    builder.state(State::Invalid);
    builder.style(Style { owner: Owner { number: 3 }, token: 2 });
    let binding = Binding { owner: Owner { number: 8 }, token: Token::new(5) };
    builder.bind(binding);
    let shortcut = crate::Shortcut { key: b'a', modifier: crate::Modifier::Primary };
    builder.shortcut(shortcut);
    builder.close();
    let tree = builder.finish();
    let node = tree.get(0).expect("field exists");
    let value = node.value.as_ref().expect("value exists");
    assert_eq!(value.written, 19);
    assert_eq!(value.text.as_ref(), b"42");
    assert_eq!(node.states.as_slice(), [State::Invalid]);
    assert_eq!(node.binding, Some(binding));
    assert_eq!(node.shortcut, Some(shortcut));
}

#[test]
fn a_drawing_keeps_opaque_data_and_readable_fallback() {
    let mut builder = Builder::new(&LIMITS);
    let kind = DrawingKind { owner: Owner { number: 2 }, kind: 1 };
    builder.drawing(kind, b"Activity", b"1234", 4);
    builder.text(Run::Strong, b"four changes");
    builder.close();
    let tree = builder.finish();
    let node = tree.get(0).expect("drawing exists");
    assert_eq!(node.size, 2);
    assert_eq!(node.name.as_deref(), Some(b"Activity".as_slice()));
    assert_eq!(node.drawing.as_ref().expect("drawing data").data.as_ref(), b"1234");
}

#[test]
fn a_drawing_fills_its_owners_byte_bound() {
    const OWNER_BYTES: u32 = 4;
    let mut builder = Builder::new(&LIMITS);
    builder.drawing(DrawingKind { owner: Owner { number: 65_536 }, kind: 1 }, b"name", b"1234", OWNER_BYTES);
    builder.close();
    let tree = builder.finish();
    let drawing = tree.get(0).expect("drawing").drawing.as_ref().expect("drawing data");
    assert_eq!(u32::try_from(drawing.data.len()).unwrap(), OWNER_BYTES);
}

#[test]
fn a_drawing_fills_the_trees_byte_bound() {
    let limits = Limits { drawing_bytes: 8, ..LIMITS };
    let mut builder = Builder::new(&limits);
    builder.drawing(DrawingKind { owner: Owner { number: 65_536 }, kind: 1 }, b"name", b"12345678", 16);
    builder.close();
    let tree = builder.finish();
    let drawing = tree.get(0).expect("drawing").drawing.as_ref().expect("drawing data");
    assert_eq!(u32::try_from(drawing.data.len()).unwrap(), limits.drawing_bytes);
}

pub(super) const BINDING_ROLES: &[(Role, bool)] = &[
    (Role::Page, false),
    (Role::Banner, false),
    (Role::Navigation, false),
    (Role::Main, false),
    (Role::Complementary, false),
    (Role::ContentInfo, false),
    (Role::Form, false),
    (Role::Search, false),
    (Role::Region, false),
    (Role::Group, false),
    (Role::Heading { level: 1 }, false),
    (Role::Paragraph, false),
    (Role::Text, false),
    (Role::List, true),
    (Role::Item, false),
    (Role::Table, false),
    (Role::Row, false),
    (Role::Cell, false),
    (Role::ColumnHeader, false),
    (Role::RowHeader, false),
    (Role::Select, false),
    (Role::Option, true),
    (Role::Term, false),
    (Role::Description, false),
    (Role::Button, true),
    (Role::Link, true),
    (Role::TextField, true),
    (Role::TextArea, true),
    (Role::Choice, true),
    (Role::ChoiceGroup, false),
    (Role::Check, true),
    (Role::Dialog, false),
    (Role::Status, false),
    (Role::Alert, false),
    (Role::Log, true),
    (Role::Tree, false),
    (Role::TreeItem, false),
    (Role::Tabs, false),
    (Role::Tab, true),
    (Role::Progress, false),
    (Role::Time, false),
    (Role::Code, false),
    (Role::Quote, false),
    (Role::Separator, false),
    (Role::Menu, false),
    (Role::MenuItem, true),
    (Role::Disclosure, false),
    (Role::Image, false),
    (Role::Figure, false),
    (Role::Slot, false),
    (Role::Drawing, true),
];

#[test]
fn bindings_are_kept_on_roles_a_person_acts_on() {
    for &(role, allowed) in BINDING_ROLES {
        if allowed {
            let mut builder = Builder::new(&LIMITS);
            builder.open(role);
            let binding = Binding { owner: Owner { number: 1 }, token: Token::new(1) };
            builder.bind(binding);
            builder.close();
            assert_eq!(builder.finish().get(0).unwrap().binding, Some(binding), "{role:?}");
        }
    }
}

#[test]
fn a_page_and_its_text_fill_the_node_capacity() {
    let limits = Limits { nodes: 2, ..LIMITS };
    let mut builder = Builder::new(&limits);
    builder.open(Role::Page);
    builder.text(Run::Plain, b"child");
    builder.close();
    assert_eq!(builder.finish().used().nodes, limits.nodes);
}

#[test]
fn a_page_and_its_region_fill_the_depth_capacity() {
    let limits = Limits { depth: 2, ..LIMITS };
    let mut builder = Builder::new(&limits);
    builder.open(Role::Page);
    builder.open(Role::Region);
    builder.close();
    builder.close();
    assert_eq!(builder.finish().used().depth, limits.depth);
}

#[test]
fn a_name_and_value_fill_the_nodes_byte_bound() {
    let limits = Limits { text_per_node: 6, ..LIMITS };
    let mut builder = Builder::new(&limits);
    builder.open(Role::TextField);
    builder.name(b"name");
    builder.value(b"12", 1);
    builder.close();
    assert_eq!(builder.finish().used().text_per_node, limits.text_per_node);
}

#[test]
fn two_texts_fill_the_trees_byte_bound() {
    let limits = Limits { text_bytes: 6, ..LIMITS };
    let mut builder = Builder::new(&limits);
    builder.text(Run::Plain, b"one");
    builder.text(Run::Plain, b"two");
    assert_eq!(builder.finish().used().text_bytes, limits.text_bytes);
}

#[test]
fn spare_node_storage_is_reused_and_replaced_bytes_are_recounted() {
    let limits = Limits { text_bytes: 5, ..LIMITS };
    let mut builder = Builder::new(&limits);
    builder.open(Role::TextArea);
    builder.value(b"long", 1);
    builder.value(b"small", 2);
    builder.close();
    let tree = builder.finish();
    let storage = tree.nodes().as_ptr();
    let mut builder = Builder::reuse(tree, &limits);
    builder.text(Run::Plain, b"again");
    let tree = builder.finish();
    assert_eq!(tree.nodes().as_ptr(), storage);
    assert_eq!(tree.len(), 1);
    assert!(crate::valid(&limits));
}

#[test]
fn runs_and_link_addresses_keep_their_owner_tokens() {
    let mut builder = Builder::new(&LIMITS);
    builder.open(Role::Link);
    let link = Binding { owner: Owner { number: 1 }, token: Token::new(77) };
    builder.link(link);
    builder.open(Role::Text);
    builder.run(Run::Plain, b"read ");
    builder.run(Run::Emphasis, b"more");
    builder.close();
    builder.close();
    let tree = builder.finish();
    assert_eq!(tree.get(0).expect("link").link, Some(link));
    assert_eq!(tree.get(1).expect("text").text.len(), 2);
}

#[test]
fn a_built_text_leaf_reserves_one_run() {
    let mut builder = Builder::new(&LIMITS);
    builder.text(Run::Plain, b"one");
    let tree = builder.finish();
    let node = tree.get(0).expect("text node exists");
    assert_eq!(node.text.capacity(), 1);
}

#[test]
fn an_image_keeps_its_owners_picture() {
    let mut builder = Builder::new(&LIMITS);
    builder.open(Role::Image);
    builder.name(b"A mark");
    let picture = crate::Picture { owner: Owner { number: 3 }, number: 7 };
    builder.picture(picture);
    builder.close();
    let tree = builder.finish();
    assert_eq!(tree.get(0).unwrap().source, Some(crate::Source::Picture(picture)));
    assert_eq!(crate::Made::of(tree.get(0).unwrap()).attribute.source, Some(crate::Source::Picture(picture)));
}

#[test]
fn a_description_and_name_fill_the_nodes_byte_bound() {
    let limits = Limits { text_per_node: 8, ..LIMITS };
    let mut b = Builder::new(&limits);
    b.open(Role::TextField);
    b.name(b"Name");
    b.description(b"Help");
    b.close();
    assert_eq!(b.finish().used().text_per_node, limits.text_per_node);
}

#[test]
fn a_hint_and_name_fill_the_node_and_tree_byte_bounds() {
    const HEADING_BYTES: u32 = 8; // Four name bytes and four hint bytes.
    for first in [None, Some(b"Old".as_slice())] {
        let limits = Limits { text_per_node: HEADING_BYTES, text_bytes: HEADING_BYTES, ..LIMITS };
        let mut b = Builder::new(&limits);
        b.open(Role::Heading { level: 1 });
        b.name(b"Name");
        if let Some(first) = first {
            b.hint(first);
        }
        b.hint(b"Hint");
        b.close();
        let tree = b.finish();
        let node = tree.get(0).unwrap();
        assert_eq!(node.hint.as_deref(), Some(b"Hint".as_slice()));
        assert_eq!(tree.used().text_per_node, limits.text_per_node);
        assert_eq!(tree.used().text_bytes, limits.text_bytes);
    }
}

const IMAGE_ADDRESSES: &[(&[u8], bool)] = &[
    (b"/a.png", true),
    (b"//x/a.png", false),
    (b"https://x/a.png", true),
    (b"http://x/a.png", false),
    (b"/a b.png", false),
    (b"https://", false),
    (b"https://?x", false),
    (b"https:///x", false),
    (b"/a\x7f.png", false),
    (b"/a\".png", false),
    (b"/a'.png", false),
    (b"/a<.png", false),
];

#[test]
fn an_image_address_must_be_a_path_or_https() {
    for &(address, expected) in IMAGE_ADDRESSES {
        assert_eq!(crate::markdown::safe_image_address(address), expected, "{address:?}");
        if expected {
            let mut b = Builder::new(&LIMITS);
            b.open(Role::Image);
            b.image_address(address);
            b.close();
            drop(b.finish());
        }
    }
}

#[test]
fn an_image_address_counts_in_the_nodes_text_bytes() {
    let mut b = Builder::new(&LIMITS);
    b.open(Role::Image);
    b.image_address(b"/a.png");
    b.close();
    assert_eq!(b.finish().get(0).unwrap().text_bytes(), 6);
}

#[test]
fn a_reused_builder_accepts_a_focus_request() {
    let mut b = Builder::new(&LIMITS);
    b.open(Role::Main);
    b.focus(1);
    b.close();
    let mut b = Builder::reuse(b.finish(), &LIMITS);
    b.open(Role::Main);
    b.focus(2);
    b.close();
    assert_eq!(b.finish().get(0).unwrap().focus, Some(2));
}

#[test]
fn required_is_kept_on_each_kind_of_field() {
    for (role, state) in [
        (Role::TextField, State::Required),
        (Role::TextArea, State::Required),
        (Role::Select, State::Required),
        (Role::Check, State::Required),
        (Role::ChoiceGroup, State::Required),
        (Role::Menu, State::Options),
    ] {
        let mut b = Builder::new(&LIMITS);
        b.open(role);
        b.state(state);
        b.close();
        assert_eq!(b.finish().get(0).unwrap().states.as_slice(), [state], "{role:?}");
    }
}

#[test]
fn a_completed_item_keeps_every_state_it_takes() {
    const STATES: &[State] = &[
        State::Disabled,
        State::Expanded,
        State::Busy,
        State::Current,
        State::Invalid,
        State::Pressed,
        State::Selected,
        State::Completed,
        State::SubmitsOnEnter,
    ];
    let mut b = Builder::new(&LIMITS);
    b.open(Role::Item);
    for &state in STATES {
        b.state(state);
    }
    b.close();
    let tree = b.finish();
    assert_eq!(tree.get(0).unwrap().states.as_slice(), STATES);
    assert_eq!(tree.get(0).unwrap().states.capacity(), 10);
}

#[test]
fn a_replaced_placeholder_fills_the_node_and_tree_byte_bounds() {
    const FIELD_BYTES: u32 = 8; // Four name bytes and four placeholder bytes.
    for role in [Role::TextField, Role::TextArea] {
        let limits = Limits { text_per_node: FIELD_BYTES, text_bytes: FIELD_BYTES, ..LIMITS };
        let mut b = Builder::new(&limits);
        b.open(role);
        b.name(b"Name");
        b.placeholder(b"Old");
        b.placeholder(b"Hint");
        b.close();
        let tree = b.finish();
        let node = tree.get(0).unwrap();
        assert_eq!(node.placeholder.as_deref(), Some(b"Hint".as_slice()));
        assert_eq!(tree.used().text_per_node, limits.text_per_node);
        assert_eq!(tree.used().text_bytes, limits.text_bytes);
    }
}

#[test]
fn a_progress_holds_its_fraction_and_its_words() {
    let mut b = Builder::new(&LIMITS);
    b.open(Role::Progress);
    b.run(Run::Plain, b"42");
    b.run(Run::Strong, b"%");
    let fraction = crate::Fraction { done: 42, total: 100 };
    b.fraction(fraction);
    b.close();
    let tree = b.finish();
    let node = tree.get(0).expect("progress exists");
    assert_eq!(node.fraction, Some(fraction));
    assert_eq!(crate::Made::of(node).attribute.fraction, Some(fraction));
    assert_eq!(node.text.len(), 2);
    assert_eq!(node.text.get(0).expect("first run").text.as_ref(), b"42");
    assert_eq!(node.text.get(1).expect("second run").text.as_ref(), b"%");
}

#[test]
fn a_primary_backslash_is_a_shortcut() {
    let mut builder = Builder::new(&LIMITS);
    builder.open(Role::Button);
    builder.bind(Binding { owner: Owner { number: 65_536 }, token: Token::new(1) });
    builder.shortcut(crate::Shortcut { key: b'\\', modifier: crate::Modifier::Primary });
    builder.close();
    let tree = builder.finish();
    assert_eq!(
        tree.get(0).expect("button").shortcut,
        Some(crate::Shortcut { key: b'\\', modifier: crate::Modifier::Primary })
    );
}
