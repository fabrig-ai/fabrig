//! The stock reference in both shells (temper/docs/design/client/README.md, 3.5).
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    reason = "ordinary tests inspect semantic trees, actions and terminal prints"
)]
use super::fixtures as f;
use crate::{look, words::Shell};
use skein_ui::{Node, Role, State, Style, Tree};
use temper_client_domain::{
    self as d, Address,
    pages::{Act, chat::document::Mode},
};

const TITLE: &[u8] = b"How temper builds temper";
const GROUP: &[u8] = b"How temper builds temper, beside the chat";
const SECTION_HEADINGS: [&str; 5] = [
    "1. In one picture",
    "2. A pipeline that keeps growing",
    "3. In parallel, kept together",
    "4. Landing many",
    "5. temper itself, and yours",
];
// The stock page has seven settings rows and its header.
const TABLE_ROWS: usize = 8;
// Section 5 ends in a closing note.
const MUTED_NOTES: usize = 1;
const SUMMARY: &str = "How fabrig builds temper, with temper: one way among many. Start from it, from something smaller, or from your own words.";

fn root() -> d::Domain {
    f::playground_root(Address::Configure { project: f::PROJECT }, f::PROJECT, true)
}
fn input(root: &mut d::Domain, act: Act) {
    f::root_event(root, d::Event::Act { action: d::Action::Pages(act) });
}
fn view(root: &d::Domain, shell: Shell) -> crate::View {
    let domain = f::domain_limits();
    let words = crate::words::Words::english(f::FORGE, 64, shell).unwrap();
    let bounds = crate::Limits::of(&domain, &words).unwrap();
    let mut view = crate::View::new(&bounds, &domain, words, f::FORGE);
    f::build(&mut view, root);
    view
}
fn opened() -> d::Domain {
    let mut root = root();
    input(&mut root, Act::ShowReference);
    root
}
fn styled(tree: &Tree, style: Style) -> Vec<&Node> {
    tree.nodes().iter().filter(|node| node.styles.as_slice().contains(&style)).collect()
}
fn subtree<'a>(tree: &'a Tree, node: &Node) -> &'a [Node] {
    let at = tree.nodes().iter().position(|n| n.id == node.id).unwrap();
    let end = at.checked_add(usize::try_from(node.size).unwrap()).unwrap();
    &tree.nodes()[at..end]
}
fn children(nodes: &[Node]) -> Vec<&Node> {
    let mut out = Vec::new();
    let mut at = 1_usize;
    while at < nodes.len() {
        out.push(&nodes[at]);
        at = at.checked_add(usize::try_from(nodes[at].size).unwrap()).unwrap();
    }
    out
}
fn text(tree: &Tree, node: &Node) -> Vec<u8> {
    subtree(tree, node).iter().flat_map(|n| n.text.iter().flat_map(|r| r.text.iter().copied())).collect()
}
fn named<'a>(tree: &'a Tree, role: Role, name: &[u8]) -> &'a Node {
    tree.nodes().iter().find(|node| node.role == role && node.name.as_deref() == Some(name)).unwrap()
}
fn main(tree: &Tree) -> &Node {
    tree.nodes().iter().find(|node| node.role == Role::Main).unwrap()
}
fn pane(tree: &Tree) -> &Node {
    named(tree, Role::Region, TITLE)
}

#[test]
fn closed_reference_keeps_the_plain_configuring_layout() {
    let root = root();
    for view in [view(&root, Shell::Terminal), view(&root, Shell::Browser)] {
        let tree = view.tree();
        assert_eq!(main(tree).styles.as_slice(), [look::MAIN, look::SESSION]);
        assert!(styled(tree, look::DOC).is_empty());
        assert!(styled(tree, look::SEG).is_empty());
        assert_eq!(
            styled(tree, look::SESSION_HINT)[0].name.as_deref(),
            Some("Enter sends · Shift+Enter for a new line".as_bytes())
        );
    }
}

#[test]
fn opening_draws_the_stock_reference_in_both_shells() {
    let root = opened();
    for (view, terminal) in [(view(&root, Shell::Terminal), true), (view(&root, Shell::Browser), false)] {
        let tree = view.tree();
        assert_eq!(main(tree).styles.as_slice(), [look::MAIN, look::SESSION, look::MODE_SPLIT]);
        assert_eq!(
            children(subtree(tree, main(tree))).iter().map(|n| n.styles.as_slice()).collect::<Vec<_>>(),
            [vec![look::SESSION_TOP], vec![look::CONVO], vec![look::DOC]]
        );
        let right = styled(tree, look::SESSION_RIGHT)[0];
        assert_eq!(
            children(subtree(tree, right)).iter().map(|n| n.styles.as_slice()).collect::<Vec<_>>(),
            [vec![look::BYLINE], vec![look::SEG]]
        );
        let group = named(tree, Role::Group, GROUP);
        assert_eq!(group.styles.as_slice(), [look::SEG]);
        assert_eq!(
            children(subtree(tree, group)).iter().map(|n| n.name.as_deref().unwrap()).collect::<Vec<_>>(),
            [b"Chat".as_slice(), b"Split", b"Draft"]
        );
        assert_eq!(pane(tree).styles.as_slice(), [look::DOC]);
        assert_eq!(styled(tree, look::RAIL_UNFOLD).is_empty(), terminal);
        let inner = styled(tree, look::DOC_INNER)[0];
        let parts = children(subtree(tree, inner));
        assert_eq!(
            parts.iter().map(|n| n.role).collect::<Vec<_>>(),
            [
                Role::Paragraph,
                Role::Heading { level: 2 },
                Role::Paragraph,
                Role::Slot,
                Role::Slot,
                Role::Slot,
                Role::Slot,
                Role::Slot
            ]
        );
        assert_eq!(parts[0].styles.as_slice(), [look::DOC_PATH]);
        let path = children(subtree(tree, parts[0])).iter().map(|n| text(tree, n)).collect::<Vec<_>>();
        let mut expected = vec!["temper’s own way · a reference".as_bytes().to_vec()];
        if terminal {
            expected.push(" · ".as_bytes().to_vec());
        }
        expected.push(b"changes nothing".to_vec());
        assert_eq!(path, expected);
        assert_eq!(parts[1].name.as_deref(), Some(TITLE));
        assert_eq!(parts[2].styles.as_slice(), [look::DOC_SUB]);
        assert_eq!(text(tree, parts[2]), SUMMARY.as_bytes());
        let sections = styled(tree, look::DOC_SEC);
        assert_eq!(sections.len(), SECTION_HEADINGS.len());
        for (section, heading) in sections.iter().zip(SECTION_HEADINGS) {
            assert!(
                subtree(tree, section)
                    .iter()
                    .any(|n| n.role == Role::Heading { level: 3 } && text(tree, n) == heading.as_bytes()),
                "{heading}"
            );
        }
        assert_eq!(styled(tree, look::DOC_FLOW).len(), 1);
        assert_eq!(children(subtree(tree, sections[0]))[0].styles.as_slice(), [look::DOC_FLOW]);
        assert_eq!(styled(tree, look::DOC_MUTED).len(), MUTED_NOTES);
        let tables = subtree(tree, pane(tree)).iter().filter(|n| n.role == Role::Table).collect::<Vec<_>>();
        assert_eq!(tables.len(), 1);
        assert_eq!(children(subtree(tree, tables[0])).len(), TABLE_ROWS);
        assert_eq!(
            styled(tree, look::DOC_MUTED).iter().map(|n| text(tree, n)).collect::<Vec<_>>(),
            [b"Each row is a setting, unless it says otherwise. Tell the chat what to keep, and what to change."
                .to_vec()]
        );
    }
}

#[test]
fn reference_modes_keep_the_group_and_use_the_mode_style() {
    let mut root = opened();
    for (mode, style, pressed, next) in [
        (Mode::Chat, look::MODE_CHAT, b"Chat".as_slice(), b"Split".as_slice()),
        (Mode::Split, look::MODE_SPLIT, b"Split", b"Draft"),
        (Mode::Draft, look::MODE_DRAFT, b"Draft", b"Chat"),
    ] {
        input(&mut root, Act::Mode { mode });
        for mut view in [view(&root, Shell::Terminal), view(&root, Shell::Browser)] {
            let tree = view.tree();
            assert_eq!(main(tree).styles.as_slice(), [look::MAIN, look::SESSION, style]);
            let group = named(tree, Role::Group, GROUP);
            let buttons = children(subtree(tree, group));
            for button in buttons {
                assert_eq!(button.states.as_slice().contains(&State::Pressed), button.name.as_deref() == Some(pressed));
                assert_eq!(button.shortcut.is_some(), button.name.as_deref() == Some(next));
            }
            assert_eq!(styled(tree, look::DOC).is_empty(), mode == Mode::Chat);
            assert!(
                text(tree, styled(tree, look::SESSION_HINT)[0])
                    .windows(b"switches the draft".len())
                    .any(|part| part == b"switches the draft")
            );
            for (name, chosen) in [(b"Chat".as_slice(), Mode::Chat), (b"Split", Mode::Split), (b"Draft", Mode::Draft)] {
                let id = named(view.tree(), Role::Button, name).id;
                assert_eq!(
                    crate::act(&mut view, id, skein_ui::Act::Press),
                    Some(d::Action::Pages(Act::Mode { mode: chosen }))
                );
            }
        }
    }
}

#[test]
fn stock_project_ways_keep_the_reference_available() {
    for ways in [
        d::forge::ways::Ways::STOCK,
        d::forge::ways::Ways { drafts: true, goals: true, way: 2, ..d::forge::ways::Ways::STOCK },
    ] {
        let mut root = f::root_with_ways(Address::Configure { project: f::PROJECT }, Some(ways));
        input(&mut root, Act::ShowReference);
        for view in [view(&root, Shell::Terminal), view(&root, Shell::Browser)] {
            assert_eq!(pane(view.tree()).styles.as_slice(), [look::DOC]);
            assert_eq!(styled(view.tree(), look::STARTER_OWN).len(), 1);
        }
    }
}

#[test]
fn terminal_prints_reference_after_composer_in_section_order() {
    let root = opened();
    let view = view(&root, Shell::Terminal);
    let mut print = skein_ui_text::Print::new(&view.limits().tree).unwrap();
    skein_ui_text::render(view.tree(), b"temper", None, &mut print);
    let printed = String::from_utf8(print.text.as_slice().to_vec()).unwrap();
    // Inline runs may occupy separate terminal lines; retain their printed word order.
    let printed = printed.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut rest = printed.as_str();
    for words in [
        "Describe how you work, or what to change",
        "How temper builds temper",
        "temper’s own way · a reference · changes nothing",
        SUMMARY,
        SECTION_HEADINGS[0],
        "1. Design draft:",
        SECTION_HEADINGS[1],
        SECTION_HEADINGS[2],
        SECTION_HEADINGS[3],
        SECTION_HEADINGS[4],
        "Each row is a setting, unless it says otherwise.",
    ] {
        let at = rest.find(words).unwrap_or_else(|| panic!("missing or out of order {words}: {printed}"));
        rest = &rest[at + words.len()..];
    }
}

#[test]
fn reference_keeps_send_and_starter_acts() {
    let mut root = opened();
    input(&mut root, Act::ChatWords { words: b"My way".as_slice().into() });
    for mut view in [view(&root, Shell::Terminal), view(&root, Shell::Browser)] {
        for (name, act) in [
            (b"Send".as_slice(), Act::StartChat),
            (
                b"Start from it",
                Act::Starter {
                    words: b"Set Shop up the way temper builds itself, sized for this machine.".as_slice().into(),
                },
            ),
        ] {
            let id = named(view.tree(), Role::Button, name).id;
            assert_eq!(crate::act(&mut view, id, skein_ui::Act::Press), Some(d::Action::Pages(act)));
        }
        let id = named(view.tree(), Role::TextArea, b"Describe how you work, or what to change").id;
        assert_eq!(crate::act(&mut view, id, skein_ui::Act::Submit), Some(d::Action::Pages(Act::StartChat)));
    }
}

#[test]
fn a_closing_section_without_a_note_is_read_whole() {
    let bytes = "## 5. Settings\n\n| Name | Value |\n|---|---|\n| Plans | Yours |\n";
    let mut builder = skein_ui::Builder::new(&f::tree_limits());
    builder.open(Role::Page);
    crate::pages::reference::closing(&mut builder, bytes.as_bytes());
    builder.close();
    let tree = builder.finish();
    assert!(styled(&tree, look::DOC_MUTED).is_empty());
    assert!(text(&tree, &tree.nodes()[0]).windows(b"Yours".len()).any(|part| part == b"Yours"));
}
