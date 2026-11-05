//! The chat's document in both shells (temper/docs/design/client/README.md, 3.5).
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    reason = "ordinary tests inspect semantic trees and prints"
)]
use super::{documents, fixtures as f};
use crate::{look, words::Shell};
use jig_client::contract as c;
use skein_lib::{Env, List, Queue, Time, Wall};
use skein_ui::{Node, Role, State, Style, Tree};
use temper_client_domain::{
    self as d,
    pages::{Act, chat::document::Mode},
};

const BODY: &[u8] =
    b"# How Shop works\n\nDraft, with the user.\n\n## Starts\n\nWrite a draft.\n\n## Lands\n\nCheck and land.\n";
const TITLE: &[u8] = b"How Shop works";
const TASK: u64 = f::chat::TASK;
const LONG: u32 = 65_536;
const FIRST: &str = "Note 1 revision 1 written · 12 lines";
const LATER: &str = "Note 1 revision 2 written · +1 −1 · line 11";

fn limits() -> d::Limits {
    let mut limits = f::domain_limits();
    limits.client.domain.long_text = LONG;
    limits.client.domain.ask_bytes = 131_072;
    limits.client.domain.requests.bytes = 524_288;
    limits.client.domain.text = 512;
    limits.client.domain.turn_text = 2048;
    limits.client.domain.rows = 4;
    limits
}
fn activity(outcomes: &[&str]) -> c::ActivityPage {
    let mut rows = List::with_capacity(1);
    if !outcomes.is_empty() {
        let mut blocks = List::with_capacity(u32::try_from(outcomes.len()).unwrap());
        for (index, outcome) in outcomes.iter().enumerate() {
            blocks
                .push(c::Block::Call(c::Call {
                    number: u64::try_from(index).unwrap().checked_add(1).unwrap(),
                    tool: c::Text::new(Box::from(b"note".as_slice()), 512).unwrap(),
                    input: c::Text::new(Box::default(), 512).unwrap(),
                    outcome: Some(c::Text::new(Box::from(outcome.as_bytes()), 512).unwrap()),
                    failed: false,
                    objects: List::with_capacity(0),
                    parts: List::with_capacity(0),
                }))
                .unwrap();
        }
        let mut messages = List::with_capacity(1);
        messages.push(c::Message { speaker: c::Speaker::Assistant, blocks }).unwrap();
        rows.push(documents::turn(
            1,
            Wall::EPOCH,
            c::TurnName { attempt: 1, number: 1 },
            c::TurnRead::Decoded(c::Turn {
                messages,
                used: c::Usage { input: 0, output: 0, cached: 0, cache_written: 0 },
                spent: 0,
                read_through: None,
                cut: false,
            }),
        ))
        .unwrap();
    }
    c::ActivityPage { rows, before: None, after: None, cut: false }
}
fn root(body: Option<&[u8]>, mode: Mode, outcomes: &[&str]) -> d::Domain {
    let limits = limits();
    let (mut root, _, watch, _, _) = f::root_at_with_activity(
        d::Address::Chat { task: TASK },
        &limits,
        b"Shop",
        None,
        Some(activity(outcomes)),
        None,
    );
    let source = f::chat::messaging_root();
    let task = source.pages().chat().unwrap().task(source.client()).unwrap().clone();
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let mut out = Queue::uncapped();
    d::step(
        &mut root,
        &env,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: watch.unwrap(),
                    event: c::WatchEvent::Delivered(c::Delivery::Task(Box::new(task))),
                },
            },
        },
        &mut out,
    );
    for _ in 0_u32..64 {
        let Some(request) = out.pop() else { break };
        if let d::Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down: jig_client::Down::Read { read, what: jig_client::Readable::Jig(c::Read::Notes { scope, .. }) },
                },
        } = request
        {
            let mut rows = List::with_capacity(1);
            if let Some(body) = body {
                rows.push(c::Note {
                    number: 1,
                    revision: c::Revision { value: 2 },
                    scope,
                    description: c::DocumentText::complete(c::Text::new(Box::from(TITLE), 512).unwrap()),
                    body: c::DocumentText::complete(c::Text::new(Box::from(body), LONG).unwrap()),
                    writer: c::NoteAuthor::Task { task: TASK, attempt: 1 },
                    tasks: List::with_capacity(0),
                    changed: Some(Wall::EPOCH),
                    acts: List::with_capacity(0),
                    cut: false,
                })
                .unwrap();
            }
            d::step(
                &mut root,
                &env,
                d::Event::Client {
                    event: jig_client::Event::Up {
                        up: jig_client::Up::ReadDone {
                            read,
                            result: jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Notes(Box::new(
                                c::NotePage { rows, before: None, after: None, cut: false },
                            )))),
                        },
                    },
                },
                &mut out,
            );
        }
    }
    assert_eq!(root.pages().chat().unwrap().draft.is_some(), body.is_some());
    d::step(&mut root, &env, d::Event::Act { action: d::Action::Pages(Act::Mode { mode }) }, &mut out);
    root
}
fn view(root: &d::Domain, shell: Shell) -> crate::View {
    let words = crate::words::Words::english(f::FORGE, 512, shell).unwrap();
    let limits = limits();
    let bounds = crate::Limits::of(&limits, &words).unwrap();
    let mut view = crate::View::new(&bounds, &limits, words, f::FORGE);
    f::build(&mut view, root);
    view
}
fn styled(tree: &Tree, style: Style) -> Vec<&Node> {
    tree.nodes().iter().filter(|node| node.styles.as_slice().contains(&style)).collect()
}
fn section_words(tree: &Tree, style: Style) -> Vec<u8> {
    let node = styled(tree, style)[0];
    let at = tree.nodes().iter().position(|n| n.id == node.id).unwrap();
    f::node_text(tree, at).into_vec()
}

#[test]
fn no_document_no_group_no_pane() {
    for mode in [Mode::Chat, Mode::Split, Mode::Draft] {
        let view = view(&root(None, mode, &[]), Shell::Browser);
        assert!(styled(view.tree(), look::SEG).is_empty());
        assert!(styled(view.tree(), look::DOC).is_empty());
    }
}
#[test]
fn the_group_presses_its_mode() {
    for (mode, name, next) in [
        (Mode::Chat, b"Chat".as_slice(), b"Split".as_slice()),
        (Mode::Split, b"Split", b"Draft"),
        (Mode::Draft, b"Draft", b"Chat"),
    ] {
        let root = root(Some(BODY), mode, &[]);
        let view = view(&root, Shell::Browser);
        let group = styled(view.tree(), look::SEG)[0];
        assert_eq!(group.name.as_deref(), Some(b"The draft".as_slice()));
        let buttons: Vec<_> = view
            .tree()
            .nodes()
            .iter()
            .filter(|node| {
                node.role == Role::Button
                    && [b"Chat".as_slice(), b"Split", b"Draft"].contains(&node.name.as_deref().unwrap_or_default())
            })
            .collect();
        assert_eq!(buttons.len(), 3);
        for button in buttons {
            assert_eq!(button.states.as_slice().contains(&State::Pressed), button.name.as_deref() == Some(name));
            assert_eq!(button.shortcut.is_some(), button.name.as_deref() == Some(next));
        }
        let split = f::find(view.tree(), Role::Button, b"Split").unwrap();
        let Some(d::Event::Act { action: d::Action::Pages(Act::Mode { mode }) }) =
            crate::event(&view, split, skein_ui::Act::Press)
        else {
            unreachable!("a mode button sends its page act")
        };
        assert_eq!(mode, Mode::Split);
    }
}
#[test]
fn chat_draws_no_pane() {
    let view = view(&root(Some(BODY), Mode::Chat, &[]), Shell::Browser);
    assert_eq!(styled(view.tree(), look::SEG).len(), 1);
    assert!(styled(view.tree(), look::DOC).is_empty());
    assert!(!f::text(view.tree()).windows(b"Write a draft.".len()).any(|w| w == b"Write a draft."));
}
#[test]
fn split_draws_the_pane_after_the_conversation() {
    let view = view(&root(Some(BODY), Mode::Split, &[]), Shell::Browser);
    let nodes = view.tree().nodes();
    let convo = nodes.iter().position(|n| n.styles.as_slice().contains(&look::CONVO)).unwrap();
    let pane = nodes.iter().position(|n| n.styles.as_slice().contains(&look::DOC)).unwrap();
    assert_eq!(pane, convo + usize::try_from(nodes[convo].size).unwrap());
    assert_eq!(nodes[pane].role, Role::Region);
    assert_eq!(nodes[pane].name.as_deref(), Some(TITLE));
    assert_eq!(styled(view.tree(), look::MODE_SPLIT).len(), 1);
    assert_eq!(styled(view.tree(), look::DOC_MUTED)[0].role, Role::Paragraph);
    let path = section_words(view.tree(), look::DOC_PATH);
    assert!(path.starts_with(b"docs/design/draft/how-shop-works.md"));
    assert!(path.ends_with(b"edited just now"));
}
#[test]
fn sections_are_keyed_by_heading() {
    let before = view(&root(Some(b"# Title\n\n## Same\n\na\n\n## Same\n\nb\n"), Mode::Split, &[]), Shell::Browser);
    let after = view(
        &root(Some(b"# Title\n\n## New\n\nx\n\n## Same\n\na\n\n## Same\n\nb\n"), Mode::Split, &[]),
        Shell::Browser,
    );
    let a: Vec<_> = styled(before.tree(), look::DOC_SEC).iter().map(|n| n.key.unwrap()).collect();
    let b: Vec<_> = styled(after.tree(), look::DOC_SEC).iter().map(|n| n.key.unwrap()).collect();
    assert_eq!(a[1], b[2]);
    assert_eq!(a[2], b[3]);
    assert_eq!(a[1].a, a[2].a);
    assert_eq!(a[1].b, 0);
    assert_eq!(a[2].b, 1);
    assert_ne!(a[0].a, a[1].a);
}
#[test]
fn a_change_keeps_the_other_sections_nodes() {
    let mut view = view(&root(Some(BODY), Mode::Split, &[]), Shell::Browser);
    let before: Vec<_> = styled(view.tree(), look::DOC_SEC).iter().map(|n| n.id).collect();
    let mut changed = root(Some(b"# How Shop works\n\nDraft, with the user.\n\n## Inserted\n\nNew.\n\n## Starts\n\nDifferent prose.\n\n## Lands\n\nCheck and land.\n"), Mode::Split, &[]);
    f::root_event(
        &mut changed,
        d::Event::Act { action: d::Action::Pages(Act::MessageWords { words: Box::from(b"reply".as_slice()) }) },
    );
    f::build(&mut view, &changed);
    let after = styled(view.tree(), look::DOC_SEC);
    assert_eq!(after.len(), 4);
    assert_eq!(before[0], after[0].id);
    assert_eq!(before[1], after[2].id);
    assert_eq!(before[2], after[3].id);
}
#[test]
fn the_tint_follows_the_newest_write() {
    let view = view(&root(Some(BODY), Mode::Split, &[FIRST, LATER]), Shell::Browser);
    let changed = String::from_utf8(section_words(view.tree(), look::DOC_CHANGED)).unwrap();
    assert!(changed.starts_with("Lands"));
    assert!(changed.ends_with("Changed by T9 just now"));
    assert_eq!(styled(view.tree(), look::DOC_CHANGED).len(), 1);
    assert_eq!(styled(view.tree(), look::DOC_NOTE).len(), 1);
}
#[test]
fn a_first_write_tints_the_first_second_level_section() {
    let view = view(&root(Some(BODY), Mode::Split, &[FIRST]), Shell::Browser);
    let changed = String::from_utf8(section_words(view.tree(), look::DOC_CHANGED)).unwrap();
    assert!(changed.starts_with("Starts"));
    assert!(changed.ends_with("Added by T9 just now"));
    assert_eq!(styled(view.tree(), look::DOC_CHANGED).len(), 1);
}
#[test]
fn the_terminal_prints_the_document_after_the_conversation() {
    for mode in [Mode::Split, Mode::Draft] {
        let view = view(&root(Some(BODY), mode, &[]), Shell::Terminal);
        let mut print = skein_ui_text::Print::new(&view.limits().tree).unwrap();
        skein_ui_text::render(view.tree(), b"temper", None, &mut print);
        let printed = String::from_utf8(print.text.as_slice().to_vec()).unwrap();
        let conversation = printed.find("Conversation").unwrap();
        let document = printed.find("How Shop works").unwrap();
        assert!(conversation < document, "{printed}");
        assert!(printed.contains("Check and land."));
        assert!(printed.contains("how-shop-works.md · edited just now"));
        assert!(f::find(view.tree(), Role::Region, TITLE).is_some());
        assert!(styled(view.tree(), look::RAIL_UNFOLD).is_empty());
    }
}
#[test]
fn a_full_document_fits_the_views_limits() {
    let short = b"x\n\n".repeat(21_845);
    let mut body = short;
    body.push(b'x');
    assert_eq!(body.len(), usize::try_from(LONG).unwrap());
    for body in [body, vec![b'x'; usize::try_from(LONG).unwrap()], b"## h\n".repeat(13_107)] {
        let view = view(&root(Some(&body), Mode::Split, &[]), Shell::Browser);
        assert!(styled(view.tree(), look::DOC).len() == 1);
        let count: usize = view.tree().nodes().iter().flat_map(|node| node.text.iter()).map(|run| run.text.len()).sum();
        assert!(count >= body.iter().filter(|b| **b == b'x' || **b == b'h').count());
    }
}
