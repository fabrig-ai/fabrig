//! The request's configuring page in both shells (temper/docs/design/client/README.md, 3.5).
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    reason = "ordinary tests compare semantic subtrees and terminal text"
)]
use super::fixtures as f;
use crate::look;
use jig_client::contract as c;
use skein_ui::{Act, Node, Role, State, Style, Tree};
use temper_client_domain::{self as d, Address, pages};

const TITLE: &[u8] = b"Configure temper";
const ENTRY: &[u8] = b"Describe how you work, or what to change";
const STARTERS: [&[u8]; 4] = [
    b"I work alone on small changes: one chat per change, landed once CI passes.",
    b"I write a design draft first; agents plan its tasks and land them together once checked.",
    b"Use Claude for reviews and my ChatGPT plan for everything else.",
    b"Not sure yet. What can temper do?",
];
const QUESTION: &str = "How should temper run Shop?";
const LEDE: &str = "Tell it how you work, in your own words or by pointing at a document.";
const NOTE: &str = "It proposes the settings; nothing changes until you accept.";
const SETS: [(&str, &str); 3] = [
    ("How work runs", "how chats and goals start, are checked and land"),
    ("Models", "which plan or model each role runs on"),
    ("Approvals", "what waits for you, and what lands on its own"),
];
const CARDS: [(&str, &str); 3] = [
    ("Small changes, alone", "One chat per change, landed once CI passes."),
    ("A design draft first", "Agents plan its tasks; they land together."),
    ("Models by role", "Claude for reviews, my ChatGPT plan for the rest."),
];
const UNSURE: &str = "Not sure yet? Ask what temper can do.";
const DRAFT: &[u8] = b"My way of working";
const UNKNOWN: u32 = 99;

fn root() -> d::Domain {
    f::playground_root(Address::Configure { project: f::PROJECT }, f::PROJECT, true)
}
fn input(root: &mut d::Domain, act: pages::Act) {
    f::root_event(root, d::Event::Act { action: d::Action::Pages(act) });
}
fn named<'a>(tree: &'a Tree, role: Role, name: &[u8]) -> &'a Node {
    tree.nodes().iter().find(|n| n.role == role && n.name.as_deref() == Some(name)).expect("named node")
}
fn styled(tree: &Tree, style: Style) -> &[Node] {
    let at = tree.nodes().iter().position(|n| n.styles.as_slice().contains(&style)).expect("styled node");
    let end = at.checked_add(usize::try_from(tree.nodes()[at].size).unwrap()).unwrap();
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
fn parts(tree: &Tree, style: Style) -> Vec<(Role, Vec<Style>)> {
    children(styled(tree, style)).into_iter().map(|n| (n.role, n.styles.as_slice().to_vec())).collect()
}
fn print(view: &crate::View) -> skein_ui_text::Print {
    let mut out = skein_ui_text::Print::new(&view.limits().tree).unwrap();
    skein_ui_text::render(view.tree(), b"temper", None, &mut out);
    out
}
fn disabled(node: &Node) -> bool {
    node.states.as_slice().contains(&State::Disabled)
}

fn starters(tree: &Tree) -> Vec<&Node> {
    styled(tree, look::STARTER_WAYS).iter().filter(|n| n.role == Role::Button).collect()
}
fn subtree<'a>(tree: &'a Tree, node: &Node) -> &'a [Node] {
    let at = tree.nodes().iter().position(|n| n.id == node.id).unwrap();
    let end = at.checked_add(usize::try_from(node.size).unwrap()).unwrap();
    &tree.nodes()[at..end]
}
fn leaves(tree: &Tree, node: &Node) -> Vec<Vec<u8>> {
    children(subtree(tree, node))
        .iter()
        .map(|n| {
            assert_eq!(n.role, Role::Text);
            n.text.iter().flat_map(|run| run.text.iter().copied()).collect()
        })
        .collect()
}

#[test]
fn requested_main_keeps_order_classes_and_phone_parts() {
    let root = root();
    for (view, browser) in [(f::page_view(&root), false), (f::browser_page_view(&root), true)] {
        let tree = view.tree();
        assert_eq!(styled(tree, look::SESSION)[0].styles.as_slice(), [look::MAIN, look::SESSION]);
        for (style, expected) in [
            (look::SESSION, vec![(Role::Slot, vec![look::SESSION_TOP]), (Role::Region, vec![look::CONVO])]),
            (look::SESSION_TOP, vec![(Role::Navigation, vec![look::CRUMBS]), (Role::Slot, vec![look::SESSION_RIGHT])]),
            (
                look::CRUMBS,
                vec![
                    (Role::Link, vec![look::BACK]),
                    (Role::Slot, vec![look::TRAIL]),
                    (Role::Heading { level: 1 }, vec![]),
                ],
            ),
            (look::TRAIL, vec![(Role::Link, vec![]), (Role::Link, vec![])]),
            (look::CONVO, vec![(Role::Slot, vec![look::THREAD]), (Role::Form, vec![look::SESSION_FOOT])]),
            (look::THREAD, vec![(Role::Slot, vec![look::STARTER])]),
            (
                look::STARTER,
                vec![
                    (Role::Heading { level: 2 }, vec![]),
                    (Role::Paragraph, vec![look::STARTER_LEDE]),
                    (Role::List, vec![look::STARTER_SETS]),
                    (Role::Paragraph, vec![look::STARTER_NOTE]),
                    (Role::Region, vec![look::STARTER_WAYS]),
                ],
            ),
            (look::STARTER_SETS, vec![(Role::Item, vec![]); 3]),
            (
                look::STARTER_WAYS,
                if browser {
                    vec![
                        (Role::Heading { level: 3 }, vec![]),
                        (Role::List, vec![look::STARTERS]),
                        (Role::Button, vec![look::STARTER_UNSURE]),
                    ]
                } else {
                    vec![(Role::List, vec![look::STARTERS]), (Role::Button, vec![look::STARTER_UNSURE])]
                },
            ),
            (look::STARTERS, vec![(Role::Item, vec![]), (Role::Item, vec![]), (Role::Item, vec![])]),
            (
                look::SESSION_FOOT,
                vec![(Role::Slot, vec![look::SESSION_COMPOSER]), (Role::Paragraph, vec![look::SESSION_HINT])],
            ),
            (look::SESSION_COMPOSER, vec![(Role::TextArea, vec![]), (Role::Button, vec![])]),
        ] {
            assert_eq!(parts(tree, style), expected, "{style:?}");
        }
        for (style, name) in [
            (look::SESSION, b"temper-session".as_slice()),
            (look::SESSION_TOP, b"temper-session-top"),
            (look::SESSION_FOOT, b"temper-session-foot"),
            (look::SESSION_COMPOSER, b"temper-session-composer"),
            (look::SESSION_HINT, b"temper-session-hint"),
            (look::CONVO, b"temper-convo"),
            (look::CRUMBS, b"temper-crumbs"),
            (look::BACK, b"temper-back"),
            (look::TRAIL, b"temper-trail"),
            (look::BYLINE, b"temper-byline"),
            (look::STARTER, b"temper-starter"),
            (look::STARTERS, b"temper-starters"),
            (look::STARTER_LEDE, b"temper-starter-lede"),
            (look::STARTER_SETS, b"temper-starter-sets"),
            (look::STARTER_NOTE, b"temper-starter-note"),
            (look::STARTER_WAYS, b"temper-starter-ways"),
            (look::STARTER_UNSURE, b"temper-starter-unsure"),
            (look::THREAD, b"temper-thread"),
        ] {
            assert_eq!(look::NAMES.style(style), Some(name));
        }
        assert_eq!(styled(tree, look::CRUMBS)[0].name.as_deref(), Some(b"Where".as_slice()));
        assert_eq!(styled(tree, look::CONVO)[0].name.as_deref(), Some(b"Conversation".as_slice()));
        assert_eq!(styled(tree, look::STARTERS)[0].name, None);
        assert_eq!(styled(tree, look::STARTER_WAYS)[0].name.as_deref(), Some(b"Ways to begin".as_slice()));
        assert_eq!(styled(tree, look::STARTER_SETS)[0].name.as_deref(), Some(b"What it sets".as_slice()));
        for (style, words) in [(look::STARTER_LEDE, LEDE), (look::STARTER_NOTE, NOTE)] {
            let paragraph = &styled(tree, style)[0];
            assert_eq!(paragraph.name, None);
            assert_eq!(leaves(tree, paragraph), [words.as_bytes()]);
        }
        assert_eq!(styled(tree, look::SESSION_FOOT)[0].name.as_deref(), Some(TITLE));
        assert_eq!(named(tree, Role::Heading { level: 1 }, TITLE).name.as_deref(), Some(TITLE));
        assert_eq!(styled(tree, look::STARTER)[1].name.as_deref(), Some(QUESTION.as_bytes()));
        assert_eq!(
            styled(tree, look::SESSION_HINT)[0].name.as_deref(),
            Some(b"Enter sends \xc2\xb7 Shift+Enter for a new line".as_slice())
        );
        assert_eq!(starters(tree).len(), 4);
        for node in starters(tree) {
            assert!(!disabled(node));
        }
        assert_eq!(named(tree, Role::Button, UNSURE.as_bytes()).styles.as_slice(), [look::STARTER_UNSURE]);
        assert!(f::find(tree, Role::Disclosure, b"Change model").is_none());
    }
}

#[test]
fn starters_composer_and_path_emit_the_requested_acts() {
    let mut root = root();
    input(&mut root, pages::Act::ChatWords { words: DRAFT.into() });
    for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
        let buttons: Vec<_> = starters(view.tree()).iter().map(|n| n.id).collect();
        for (node, words) in buttons.into_iter().zip(STARTERS) {
            assert_eq!(
                crate::act(&mut view, node, Act::Press),
                Some(d::Action::Pages(pages::Act::Starter { words: words.into() }))
            );
        }
        let entry = named(view.tree(), Role::TextArea, ENTRY).id;
        assert_eq!(
            crate::act(&mut view, entry, Act::Input { value: b"Changed".as_slice().into() }),
            Some(d::Action::Pages(pages::Act::ChatWords { words: b"Changed".as_slice().into() }))
        );
        assert_eq!(crate::act(&mut view, entry, Act::Submit), Some(d::Action::Pages(pages::Act::StartChat)));
        let send = named(view.tree(), Role::Button, b"Send").id;
        assert_eq!(crate::act(&mut view, send, Act::Press), Some(d::Action::Pages(pages::Act::StartChat)));
        let links: Vec<_> = children(styled(view.tree(), look::TRAIL)).into_iter().map(|n| n.id).collect();
        let back = styled(view.tree(), look::BACK)[0].id;
        for (node, address) in [
            (back, Address::Settings { project: f::PROJECT }),
            (links[0], Address::Home { project: f::PROJECT }),
            (links[1], Address::Settings { project: f::PROJECT }),
        ] {
            assert_eq!(crate::href(&mut view, node).as_deref(), Some(d::address::print(&address).as_ref()));
            assert_eq!(crate::act(&mut view, node, Act::Press), Some(d::Action::Pages(pages::Act::Go { address })));
        }
    }
}

#[test]
fn empty_composer_is_focused_with_send_disabled() {
    let root = root();
    for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
        let entry = named(view.tree(), Role::TextArea, ENTRY);
        assert_eq!(entry.placeholder.as_deref(), Some(ENTRY));
        assert_eq!(entry.focus, Some(0));
        assert_eq!(entry.value.as_ref().unwrap().text.as_ref(), b"");
        assert!(entry.states.as_slice().contains(&State::SubmitsOnEnter));
        let send = named(view.tree(), Role::Button, b"Send");
        assert!(disabled(send));
        let id = send.id;
        assert_eq!(crate::act(&mut view, id, Act::Press), None);
    }
}

#[test]
fn pending_starts_disable_starters_and_send_keep_the_words() {
    for (start, expected) in
        [(pages::Act::StartChat, DRAFT), (pages::Act::Starter { words: STARTERS[0].into() }, STARTERS[0])]
    {
        let mut root = root();
        input(&mut root, pages::Act::ChatWords { words: DRAFT.into() });
        input(&mut root, start);
        assert!(root.pages().chats().unwrap().start.is_some());
        for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
            let buttons: Vec<_> = starters(view.tree())
                .iter()
                .map(|n| {
                    assert!(disabled(n), "{n:?}");
                    n.id
                })
                .collect();
            assert_eq!(buttons.len(), 4);
            for id in buttons {
                assert_eq!(crate::act(&mut view, id, Act::Press), None);
            }
            let send = named(view.tree(), Role::Button, b"Send");
            assert!(disabled(send));
            let id = send.id;
            assert_eq!(crate::act(&mut view, id, Act::Press), None);
            let entry = named(view.tree(), Role::TextArea, ENTRY);
            assert_eq!(entry.value.as_ref().unwrap().text.as_ref(), expected);
            assert!(!entry.states.as_slice().contains(&State::SubmitsOnEnter));
            let id = entry.id;
            assert_eq!(crate::act(&mut view, id, Act::Submit), None);
        }
    }
}

#[test]
fn refused_start_keeps_words_and_puts_refusal_under_composer() {
    let mut root = root();
    input(&mut root, pages::Act::ChatWords { words: DRAFT.into() });
    let mut out = skein_lib::Queue::uncapped();
    d::step(
        &mut root,
        &skein_lib::Env { now: skein_lib::Time::ZERO, wall: f::NOW, limits: f::domain_limits() },
        d::Event::Act { action: d::Action::Pages(pages::Act::StartChat) },
        &mut out,
    );
    d::step(
        &mut root,
        &skein_lib::Env { now: skein_lib::Time::ZERO, wall: f::NOW, limits: f::domain_limits() },
        d::Event::Client { event: jig_client::Event::Stored },
        &mut out,
    );
    let mut sent = None;
    for request in &out {
        if let d::Request::Unlinked {
            request: jig_client::UnlinkedRequest::Down { down: jig_client::Down::Send { attempt, .. } },
        } = request
        {
            sent = Some(*attempt);
        }
    }
    let attempt = sent.expect("sent start");
    f::root_event(
        &mut root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Answer {
                    attempt,
                    envelope: skein_lib::RequestEnvelope::Final,
                    why: None,
                    answer: Some(jig_client::Answered::Jig(c::Answer::Refused {
                        refusal: c::Refusal::NotOffered,
                        at: None,
                    })),
                },
            },
        },
    );
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        let entry = named(view.tree(), Role::TextArea, ENTRY);
        assert_eq!(entry.value.as_ref().unwrap().text.as_ref(), DRAFT);
        assert!(!disabled(named(view.tree(), Role::Button, b"Send")));
        let foot = children(styled(view.tree(), look::SESSION_FOOT));
        assert_eq!(foot.last().unwrap().role, Role::Status);
        assert!(String::from_utf8_lossy(&f::main_text(view.tree())).contains("not offered"));
    }
}

#[test]
fn browser_pairs_are_two_leaves_and_terminal_pairs_are_one_line() {
    let root = root();
    for (view, browser) in [(f::page_view(&root), false), (f::browser_page_view(&root), true)] {
        let tree = view.tree();
        let items = children(styled(tree, look::STARTER_SETS));
        let buttons = starters(tree);
        for (node, (term, line)) in items.into_iter().zip(SETS).chain(buttons.into_iter().zip(CARDS)) {
            assert_eq!(node.name, None);
            let expected = if browser {
                vec![term.as_bytes().to_vec(), line.as_bytes().to_vec()]
            } else {
                vec![format!("{term}: {line}").into_bytes()]
            };
            assert_eq!(leaves(tree, node), expected);
        }
    }
}

#[test]
fn question_uses_the_project_name() {
    for name in [b"Hello".as_slice(), b"Another project"] {
        let (root, _, _, _, _) = f::root_at_full(Address::Configure { project: f::PROJECT }, &f::domain_limits(), name);
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            let question = format!("How should temper run {}?", String::from_utf8_lossy(name));
            named(view.tree(), Role::Heading { level: 2 }, question.as_bytes());
        }
    }
}

#[test]
fn terminal_prints_the_path_opening_starters_and_composer_in_order() {
    let root = root();
    let view = f::page_view(&root);
    let out = print(&view);
    let text = String::from_utf8_lossy(out.text.as_slice());
    let numbered = |node: &Node, words: &str| {
        let number = out.numbers.as_slice().iter().position(|id| *id == node.id).unwrap() + 1;
        format!("[{number}] {words}")
    };
    // skein-ui-text prints the level-1 heading as the page title.
    let mut expected = vec!["== Configure temper ==".to_owned(), "Where:".to_owned()];
    for node in children(styled(view.tree(), look::TRAIL)) {
        expected.push(numbered(node, &String::from_utf8_lossy(node.name.as_deref().unwrap())));
    }
    expected.extend(["Conversation:", QUESTION, LEDE, "What it sets:"].map(str::to_owned));
    expected.extend(SETS.map(|(term, line)| format!("{term}: {line}")));
    expected.extend([NOTE, "Ways to begin:"].map(str::to_owned));
    for (node, (title, line)) in starters(view.tree()).into_iter().zip(CARDS) {
        expected.push(numbered(node, &format!("{title}: {line}")));
    }
    expected.push(numbered(named(view.tree(), Role::Button, UNSURE.as_bytes()), UNSURE));
    expected.extend([String::from_utf8_lossy(ENTRY).into_owned(), "Send".to_owned()]);
    let mut remaining = text.as_ref();
    for words in expected {
        let at = remaining.find(&words).unwrap_or_else(|| panic!("missing or out of order {words}: {text}"));
        remaining = &remaining[at + words.len()..];
    }
    assert_eq!(text.matches("Ways to begin:").count(), 1, "{text}");
    assert!(!text.contains("temper’s configuring agent"));
}

#[test]
fn settings_is_current_in_the_rail_and_phone_tabs() {
    let root = root();
    for (view, tabs) in [(f::page_view(&root), false), (f::browser_page_view(&root), true)] {
        assert_eq!(view.tree().nodes().iter().any(|node| node.styles.as_slice().contains(&look::TABBAR)), tabs);
        for style in [look::RAIL, look::TABBAR] {
            // The terminal has no tab bar.
            if let Some(at) = view.tree().nodes().iter().position(|n| n.styles.as_slice().contains(&style)) {
                let end = at.checked_add(usize::try_from(view.tree().nodes()[at].size).unwrap()).unwrap();
                let current: Vec<_> = view.tree().nodes()[at..end]
                    .iter()
                    .filter(|n| n.states.as_slice().contains(&State::Current))
                    .map(|n| n.name.as_deref())
                    .collect();
                assert_eq!(current, [Some(b"Settings".as_slice())]);
            }
        }
    }
}

#[test]
fn unlisted_project_says_what_home_says_in_both_shells() {
    let (root, _, _) = f::root_at(Address::Configure { project: UNKNOWN }, &f::domain_limits());
    for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
        assert!(String::from_utf8_lossy(&f::main_text(view.tree())).contains("Project 99 is not one of yours."));
        let node = named(view.tree(), Role::Link, b"Chats").id;
        assert_eq!(
            crate::act(&mut view, node, Act::Press),
            Some(d::Action::Pages(pages::Act::Go { address: Address::Chats { filter: d::ChatsFilter::All } }))
        );
        assert!(f::find(view.tree(), Role::TextArea, ENTRY).is_none());
        assert!(!view.tree().nodes().iter().any(|n| n.styles.as_slice().contains(&look::STARTER)));
    }
}

#[test]
fn byline_uses_the_projects_first_ready_provider_and_its_account() {
    use c::Readiness::{Unavailable, Unknown, Usable};
    for (ready, accounts, models, expected) in [
        (
            [Usable, Unavailable],
            [true, false],
            true,
            Some("your Claude subscription, through Claude Code · claude-opus-5-5"),
        ),
        ([Usable, Usable], [true, true], true, Some("ChatGPT plan · gpt-6.1-sol")),
        ([Unavailable, Usable], [true, true], true, Some("ChatGPT plan · gpt-6.1-sol")),
        ([Unknown, Usable], [true, true], true, Some("ChatGPT plan · gpt-6.1-sol")),
        ([Unavailable, Unavailable], [false, false], true, Some("Using the default model")),
        ([Unknown, Unknown], [true, true], true, Some("Using the default model")),
        ([Usable, Usable], [true, true], false, None),
        ([Unavailable, Unavailable], [false, false], false, None),
    ] {
        let (root, _) =
            super::choice::provider_byline_root(Address::Configure { project: f::PROJECT }, ready, accounts, models);
        assert_eq!(root.pages().choice().project(), None);
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            let line = view.tree().nodes().iter().find(|n| n.styles.as_slice().contains(&look::BYLINE));
            assert_eq!(line.and_then(|n| n.name.as_deref()), expected.map(str::as_bytes), "{ready:?}, {accounts:?}");
        }
    }
}
