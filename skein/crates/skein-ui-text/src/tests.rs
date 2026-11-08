use crate::{Print, Terminal, render};
use skein_lib::Token;
use skein_ui::{Role, Run};
mod bound;
mod command;
mod fixtures;

#[test]
fn a_terminal_owns_its_namespace_alone() {
    for (op, owned) in [
        (Token::tagged(4, 7).unwrap(), true),
        (Token::new(7), false),
        (Token::tagged(2, 7).unwrap(), false),
        (Token::tagged(5, 7).unwrap(), false),
    ] {
        assert_eq!(Terminal::owns(op), owned, "namespace {}", op.tag());
    }
}

#[test]
#[expect(clippy::disallowed_macros, reason = "hand-written golden bytes fix the renderer's contract")]
fn a_tree_with_every_role_and_state_prints_its_golden_text() {
    let tree = fixtures::page(fixtures::roles);
    let mut print = Print::new(&fixtures::LIMITS).unwrap();
    render(&tree, b"ops", None, &mut print);
    assert_eq!(print.text.as_slice(), include_bytes!("tests/golden/roles.txt"));
    assert_eq!(print.numbers.len(), 14);
}
#[test]
#[expect(clippy::disallowed_macros, reason = "hand-written golden bytes cover unsafe terminal text")]
fn control_bytes_and_bad_utf8_print_as_replacement() {
    let tree = fixtures::page(|b| {
        b.open(Role::Region);
        b.name(b"Indented");
        b.text(Run::Plain, b"a\x00\x1b\x1f\x7f\xc2\x80\xc2\x9f\tend\nnext\xff\xc0\x80\xe2\x82\xac");
        b.close();
    });
    let mut print = Print::new(&fixtures::LIMITS).unwrap();
    render(&tree, b"ops", None, &mut print);
    assert_eq!(print.text.as_slice(), include_bytes!("tests/golden/bytes.txt"));
}
#[test]
fn a_name_ending_in_its_own_mark_takes_no_colon() {
    let tree = fixtures::page(|b| {
        for name in [b"What shall we change?".as_slice(), b"Stop!", b"Note:", b"Plain"] {
            b.open(Role::Region);
            b.name(name);
            fixtures::leaf(b, Role::Paragraph, b"in");
            b.close();
        }
    });
    let mut print = Print::new(&fixtures::LIMITS).unwrap();
    render(&tree, b"ops", None, &mut print);
    assert_eq!(
        print.text.as_slice(),
        b"== ops == \nWhat shall we change?\n  in\nStop!\n  in\nNote:\n  in\nPlain:\n  in\n".as_slice()
    );
}
#[test]
fn a_named_disclosure_prints_its_description_on_its_name_line() {
    let tree = fixtures::page(|b| {
        for (name, description) in [
            (Some(b"1 tool call".as_slice()), Some(b"wait".as_slice())),
            (Some(b"1 tool call".as_slice()), None),
            (None, Some(b"wait".as_slice())),
        ] {
            b.open(Role::Disclosure);
            if let Some(name) = name {
                b.name(name);
            }
            if let Some(description) = description {
                b.description(description);
            }
            fixtures::leaf(b, Role::Paragraph, b"wait");
            b.close();
        }
        b.open(Role::Disclosure);
        b.name(b"More?");
        b.description(b"first\nnext\tline\x1b\xff");
        b.state(skein_ui::State::Expanded);
        fixtures::leaf(b, Role::Paragraph, b"child");
        b.close();
    });
    let mut print = Print::new(&fixtures::LIMITS).unwrap();
    render(&tree, b"ops", None, &mut print);
    assert_eq!(
        print.text.as_slice(),
        concat!(
            "== ops == \n",
            "1 tool call: wait\n  wait\n",
            "1 tool call:\n  wait\n",
            "wait\nwait\n",
            "More? first\n  next line\u{fffd}\u{fffd} (expanded)\n  child\n",
        )
        .as_bytes()
    );
}
#[test]
fn an_all_empty_header_row_prints_no_line() {
    let tree = fixtures::page(|b| {
        for header in [b"".as_slice(), b"Name"] {
            b.open(Role::Table);
            b.open(Role::Row);
            fixtures::leaf(b, Role::ColumnHeader, b"");
            fixtures::leaf(b, Role::ColumnHeader, header);
            b.close();
            b.open(Role::Row);
            fixtures::leaf(b, Role::Cell, b"left");
            fixtures::leaf(b, Role::Cell, b"right");
            b.close();
            b.close();
        }
        b.open(Role::Region);
        b.name(b"Indented");
        b.open(Role::Table);
        b.open(Role::Row);
        b.description(b"Header description");
        b.position(1);
        b.state(skein_ui::State::Selected);
        b.open(Role::ColumnHeader);
        b.description(b"Cell description");
        b.close();
        fixtures::leaf(b, Role::ColumnHeader, b"");
        b.close();
        b.open(Role::Row);
        fixtures::leaf(b, Role::Cell, b"");
        fixtures::leaf(b, Role::RowHeader, b"");
        b.close();
        b.close();
        b.close();
    });
    let mut print = Print::new(&fixtures::LIMITS).unwrap();
    render(&tree, b"ops", None, &mut print);
    assert_eq!(
        print.text.as_slice(),
        b"== ops == \nleft | right\n | Name\nleft | right\nIndented:\n  Header description\n  Cell description\n   | \n"
    );
}
#[test]
fn a_disabled_button_prints_a_dash_not_a_number() {
    let tree = fixtures::page(|b| {
        fixtures::button(b, b"Accept", true);
        fixtures::button(b, b"Release", false);
    });
    let mut print = Print::new(&fixtures::LIMITS).unwrap();
    render(&tree, b"ops", None, &mut print);
    assert_eq!(print.text.as_slice(), b"== ops == \n[1] Accept\n[-] Release (disabled)\n");
    assert_eq!(print.numbers.as_slice(), [tree.get(3).unwrap().id]);
}
#[test]
fn the_first_line_is_the_main_heading_and_the_link() {
    let tree = fixtures::page(|b| {
        b.open(Role::Region);
        b.region(fixtures::STATUS);
        fixtures::leaf(b, Role::Status, b"Connected");
        fixtures::leaf(b, Role::Status, b"Behind");
        b.close();
        fixtures::leaf(b, Role::Heading { level: 1 }, b"Inbox");
        fixtures::leaf(b, Role::Heading { level: 1 }, b"Again");
    });
    let mut print = Print::new(&fixtures::LIMITS).unwrap();
    render(&tree, b"ops", Some(fixtures::STATUS), &mut print);
    assert_eq!(print.text.as_slice(), b"== Inbox == Connected; Behind\n# Again\n");
}

#[test]
fn no_status_region_ends_the_first_line_after_its_space() {
    let tree = fixtures::page(|b| {
        b.open(Role::Region);
        b.region(fixtures::STATUS);
        fixtures::leaf(b, Role::Status, b"Connected");
        b.close();
        fixtures::leaf(b, Role::Heading { level: 1 }, b"Inbox");
    });
    let mut print = Print::new(&fixtures::LIMITS).unwrap();
    render(&tree, b"ops", None, &mut print);
    assert_eq!(print.text.as_slice(), b"== Inbox == \n* Connected\n");
}

#[test]
fn the_properties_print_their_words() {
    let tree = fixtures::page(fixtures::properties);
    let mut print = Print::new(&fixtures::LIMITS).unwrap();
    render(&tree, b"ops", None, &mut print);
    let expected = std::fs::read("src/tests/golden/properties.txt").expect("hand-written property words");
    assert_eq!(print.text.as_slice(), expected.as_slice());
}

#[test]
fn a_secret_never_prints_its_bytes() {
    let tree = fixtures::page(|b| {
        b.open(Role::TextField);
        b.name(b"Password");
        b.entry(skein_ui::EntryKind::Secret);
        b.value(b"private-token", 1);
        b.bind(fixtures::binding());
        b.close();
    });
    let mut print = Print::new(&fixtures::LIMITS).unwrap();
    render(&tree, b"ops", None, &mut print);
    assert_eq!(print.text.as_slice(), b"== ops == \n[1] Password: (hidden)\n");
}

#[test]
fn the_first_line_is_followed_by_its_descriptions() {
    let tree = fixtures::page(|b| {
        b.open(Role::Heading { level: 1 });
        b.description(b"Heading description");
        b.open(Role::Text);
        b.text(Run::Plain, b"Inbox");
        b.description(b"Inline description");
        b.close();
        b.close();
    });
    let mut print = Print::new(&fixtures::LIMITS).unwrap();
    render(&tree, b"ops", None, &mut print);
    assert_eq!(print.text.as_slice(), b"== Inbox == \n  Heading description\n  Inline description\n");
}

#[test]
fn every_role_state_and_property_prints() {
    let tree = fixtures::page_with_limits(fixtures::every_node, &fixtures::EVERY_LIMITS);
    let mut print = Print::new(&fixtures::EVERY_LIMITS).unwrap();
    render(&tree, b"ops", None, &mut print);
    let expected = std::fs::read("src/tests/golden/every.txt").expect("hand-written complete tree");
    assert_eq!(print.text.as_slice(), expected.as_slice());
}

#[test]
fn the_first_line_without_its_status_region() {
    let tree = fixtures::page(fixtures::first_lines);
    let mut print = Print::new(&fixtures::LIMITS).unwrap();
    render(&tree, b"ops", Some(fixtures::STATUS), &mut print);
    let expected = std::fs::read("src/tests/golden/first-line.txt").expect("hand-written absent status region");
    assert_eq!(print.text.as_slice(), expected.as_slice());
}

#[test]
fn control_bytes_print_as_replacements_everywhere() {
    let tree = fixtures::page(fixtures::control_bytes);
    let mut print = Print::new(&fixtures::LIMITS).unwrap();
    render(&tree, b"ops", None, &mut print);
    let expected = std::fs::read("src/tests/golden/bytes-everywhere.txt").expect("hand-written replacement characters");
    assert_eq!(print.text.as_slice(), expected.as_slice());
}

#[test]
#[expect(clippy::disallowed_macros, reason = "the reader's report and hand-checked text golden are shared fixtures")]
fn a_markdown_table_prints_each_row_with_its_cells() {
    const READER_LEVELS: u32 = 5; // ui.md, 10: table, row, cell, link, text.
    let print = markdown_print(include_bytes!("../../skein-ui/src/tests/markdown/13.md"), 1, READER_LEVELS);
    assert_eq!(print.text.as_slice(), include_bytes!("tests/golden/markdown-table.txt"));
}

#[test]
#[expect(clippy::disallowed_macros, reason = "acceptance cases and hand-checked fallback text are shared fixtures")]
fn a_markdown_flow_prints_its_name_and_fallback() {
    const READER_LEVELS: u32 = 3; // ui.md, 10.1: drawing, fallback paragraph, text.
    let input = skein_lib::bytes::concat(&[
        include_bytes!("../../../docs/design/draft/diagrams/how-hello-works.md"),
        b"\n",
        include_bytes!("../../../docs/design/draft/diagrams/review.md"),
    ]);
    let print = markdown_print(&input, 0, READER_LEVELS);
    assert_eq!(print.text.as_slice(), include_bytes!("tests/golden/markdown-flow.txt"));
}

#[test]
#[expect(clippy::disallowed_macros, reason = "the reader's report and hand-checked text golden are shared fixtures")]
fn numbered_markdown_prints_its_numbers_as_written() {
    const READER_LEVELS: u32 = 6; // ui.md, 10: two list/item pairs, link, text.
    let print = markdown_print(include_bytes!("../../skein-ui/src/tests/markdown/17.md"), 2, READER_LEVELS);
    assert_eq!(print.text.as_slice(), include_bytes!("tests/golden/markdown-numbered.txt"));
}

fn markdown_print(input: &[u8], nesting: u32, reader_levels: u32) -> Print {
    let bytes = u32::try_from(input.len()).unwrap();
    let limits = skein_ui::Limits {
        nodes: skein_ui::markdown::max_nodes(bytes).unwrap().checked_add(1).unwrap(),
        depth: reader_levels.checked_add(1).unwrap(), // ui.md, 10: reader levels below the page.
        text_per_node: skein_ui::markdown::max_text_bytes(bytes).unwrap(),
        text_bytes: skein_ui::markdown::max_text_bytes(bytes).unwrap(),
        markdown_depth: nesting,
        drawing_bytes: bytes,
        ..fixtures::LIMITS
    };
    let mut builder = skein_ui::Builder::new(&limits);
    builder.open(Role::Page);
    skein_ui::markdown::read(input, 0, &mut builder);
    builder.close();
    let tree = builder.finish();
    let mut print = Print::new(&limits).unwrap();
    render(&tree, b"ops", None, &mut print);
    print
}

#[test]
fn a_menu_item_prints_its_description_on_its_own_line() {
    let tree = fixtures::page(|b| {
        b.open(Role::MenuItem);
        b.name(b"Chat");
        b.description(b"Ask a question.");
        b.bind(fixtures::binding());
        b.close();
    });
    let mut print = Print::new(&fixtures::LIMITS).unwrap();
    render(&tree, b"ops", None, &mut print);
    assert_eq!(print.text.as_slice(), b"== ops == \n[1] Chat\n  Ask a question.\n");
}

#[test]
fn an_ordinary_group_prints_its_children_as_a_slot_does() {
    let mut expected = None;
    for role in [Role::Slot, Role::Group] {
        let mut b = skein_ui::Builder::new(&fixtures::LIMITS);
        b.open(Role::Page);
        b.open(Role::Main);
        b.open(role);
        b.name(b"The draft");
        fixtures::button(&mut b, b"Chat", true);
        fixtures::button(&mut b, b"Draft", true);
        b.close();
        b.close();
        b.close();
        let tree = b.finish();
        let mut print = Print::new(&fixtures::LIMITS).unwrap();
        render(&tree, b"ops", None, &mut print);
        assert_eq!(print.numbers.len(), 2);
        let text = skein_lib::bytes::copy_of(print.text.as_slice());
        if let Some(expected) = &expected {
            assert_eq!(&text, expected);
        } else {
            expected = Some(text);
        }
    }
    for (outer, name, middle, item, inner, inner_name, expected) in [
        (
            Role::Region,
            b"Ways".as_slice(),
            None,
            false,
            Role::Tabs,
            b"Ways".as_slice(),
            b"== ops == \nWays:\n  [1] Pick\n".as_slice(),
        ),
        (Role::Drawing, b"Path", None, false, Role::List, b"Path", b"== ops == \nPath:\n  - first\n  - second\n"),
        (
            Role::Region,
            b"Ways",
            None,
            false,
            Role::List,
            b"Other",
            b"== ops == \nWays:\n  Other:\n    - first\n    - second\n",
        ),
        (
            Role::Region,
            b"Ways",
            Some(b"Inner".as_slice()),
            false,
            Role::List,
            b"Ways",
            b"== ops == \nWays:\n  Inner:\n    Ways:\n      - first\n      - second\n",
        ),
        (
            Role::Region,
            b"Spend",
            None,
            true,
            Role::List,
            b"Spend",
            b"== ops == \nSpend:\n  - Spend:\n      - first\n      - second\n",
        ),
    ] {
        let mut b = skein_ui::Builder::new(&fixtures::LIMITS);
        b.open(Role::Page);
        b.open(outer);
        b.name(name);
        if let Some(middle) = middle {
            b.open(Role::Region);
            b.name(middle);
        }
        if item {
            b.open(Role::Item);
        }
        b.open(inner);
        b.name(inner_name);
        if inner == Role::Tabs {
            b.open(Role::Tab);
            b.name(b"Pick");
            b.bind(fixtures::binding());
            b.close();
        } else {
            fixtures::leaf(&mut b, Role::Item, b"first");
            fixtures::leaf(&mut b, Role::Item, b"second");
        }
        b.close();
        if item {
            b.close();
        }
        if middle.is_some() {
            b.close();
        }
        b.close();
        b.close();
        let tree = b.finish();
        let mut print = Print::new(&fixtures::LIMITS).unwrap();
        render(&tree, b"ops", None, &mut print);
        assert_eq!(print.text.as_slice(), expected);
    }
}

#[test]
fn only_direct_items_take_their_lists_numbers_without_wrapping() {
    for (first, expected) in [
        (0, b"== ops == \n0. first\n- indirect\n- indirect\n- tree item\n1. second\n".as_slice()),
        (
            u32::MAX,
            b"== ops == \n4294967295. first\n- indirect\n- indirect\n- tree item\n4294967296. second\n".as_slice(),
        ),
    ] {
        let mut builder = skein_ui::Builder::new(&fixtures::LIMITS);
        builder.open(Role::Page);
        {
            let b = &mut builder;
            b.open(Role::List);
            b.ordered(first);
            fixtures::leaf(b, Role::Item, b"first");
            for role in [Role::Group, Role::Slot] {
                b.open(role);
                fixtures::leaf(b, Role::Item, b"indirect");
                b.close();
            }
            fixtures::leaf(b, Role::TreeItem, b"tree item");
            fixtures::leaf(b, Role::Item, b"second");
            b.close();
        }
        builder.close();
        let tree = builder.finish();
        let mut print = Print::new(&fixtures::LIMITS).unwrap();
        render(&tree, b"ops", None, &mut print);
        assert_eq!(print.text.as_slice(), expected);
    }
}
