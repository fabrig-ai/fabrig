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

const TITLE: &[u8] = b"Tell temper how you work";
const ENTRY: &[u8] = b"How do you work?";
const STARTERS: [&[u8]; 3] = [
    b"I work alone on small changes: one chat per change, landed once CI passes.",
    b"I write a design draft first; agents plan its tasks and land them together once checked.",
    b"Not sure yet. What can temper do?",
];
const AGENT: &str = "temper’s configuring agent. Describe how you work, in your own words or by pointing at a document. It proposes how temper will run that; nothing changes until you accept.";
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

#[test]
fn requested_main_keeps_order_classes_and_phone_parts() {
    let root = root();
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
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
            (look::STARTER, vec![(Role::Paragraph, vec![]), (Role::List, vec![look::STARTERS])]),
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
            (look::THREAD, b"temper-thread"),
        ] {
            assert_eq!(look::NAMES.style(style), Some(name));
        }
        assert_eq!(styled(tree, look::CRUMBS)[0].name.as_deref(), Some(b"Where".as_slice()));
        assert_eq!(styled(tree, look::CONVO)[0].name.as_deref(), Some(b"Conversation".as_slice()));
        assert_eq!(styled(tree, look::STARTERS)[0].name.as_deref(), Some(b"Ways to begin".as_slice()));
        assert_eq!(styled(tree, look::SESSION_FOOT)[0].name.as_deref(), Some(TITLE));
        assert_eq!(named(tree, Role::Heading { level: 1 }, TITLE).name.as_deref(), Some(TITLE));
        assert_eq!(styled(tree, look::STARTER)[1].name.as_deref(), Some(AGENT.as_bytes()));
        assert_eq!(
            styled(tree, look::SESSION_HINT)[0].name.as_deref(),
            Some(b"Enter sends \xc2\xb7 Shift+Enter for a new line".as_slice())
        );
        let starters: Vec<_> = styled(tree, look::STARTERS)
            .iter()
            .filter(|node| node.role == Role::Button)
            .map(|node| node.name.as_deref())
            .collect();
        assert_eq!(starters, STARTERS.map(Some));
        for words in STARTERS {
            assert!(!disabled(named(tree, Role::Button, words)));
        }
        assert!(f::find(tree, Role::Disclosure, b"Change model").is_none());
    }
}

#[test]
fn starters_composer_and_path_emit_the_requested_acts() {
    let mut root = root();
    input(&mut root, pages::Act::ChatWords { words: DRAFT.into() });
    for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
        for words in STARTERS {
            let node = named(view.tree(), Role::Button, words).id;
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
            for words in STARTERS.into_iter().chain([b"Send".as_slice()]) {
                let node = named(view.tree(), Role::Button, words);
                assert!(disabled(node), "{words:?}");
                let id = node.id;
                assert_eq!(crate::act(&mut view, id, Act::Press), None);
            }
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
fn terminal_prints_the_path_agent_starters_and_composer() {
    let root = root();
    let view = f::page_view(&root);
    let out = print(&view);
    let text = String::from_utf8_lossy(out.text.as_slice());
    for words in
        ["Where:", "Tell temper how you work", "Conversation:", AGENT, "Ways to begin:", "How do you work?", "Send"]
    {
        assert!(text.contains(words), "missing {words}: {text}");
    }
    for words in STARTERS {
        let node = named(view.tree(), Role::Button, words);
        let number = out.numbers.as_slice().iter().position(|id| *id == node.id).unwrap().checked_add(1).unwrap();
        assert!(text.contains(&format!("[{number}] {}", String::from_utf8_lossy(words))), "{text}");
    }
    for node in children(styled(view.tree(), look::TRAIL)) {
        let number = out.numbers.as_slice().iter().position(|id| *id == node.id).unwrap().checked_add(1).unwrap();
        assert!(
            text.contains(&format!("[{number}] {}", String::from_utf8_lossy(node.name.as_deref().unwrap()))),
            "{text}"
        );
    }
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
        assert!(f::find(view.tree(), Role::Button, STARTERS[0]).is_none());
    }
}

#[test]
fn byline_joins_only_the_first_usable_account_and_project_model() {
    for (account, model, expected) in [
        (false, false, None),
        (true, false, Some("ChatGPT plan")),
        (false, true, Some("first model")),
        (true, true, Some("ChatGPT plan · first model")),
    ] {
        let (mut root, me_watch, _, _, accounts) =
            f::root_at_full(Address::Configure { project: f::PROJECT }, &f::domain_limits(), b"Shop");
        let mut me = root.pages().me(root.client()).unwrap().clone();
        let sample = self::root();
        let source = sample.pages().me(sample.client()).unwrap().projects.get(0).unwrap().models.get(0).unwrap();
        for (project, names) in [
            (f::PROJECT, if model { &["first model", "later model"][..] } else { &[][..] }),
            (f::SECOND_PROJECT, &["other project"][..]),
        ] {
            let member = me.projects.iter_mut().find(|m| m.project == project).unwrap();
            member.models = skein_lib::List::with_capacity(u32::try_from(names.len()).unwrap());
            for name in names {
                let mut offered = source.clone();
                offered.name = c::Text::new(name.as_bytes().into(), 32).unwrap();
                member.models.push(offered).unwrap();
            }
        }
        f::deliver_me(&mut root, me_watch, me);
        let mut doc = super::accounts::document(0);
        if account {
            let mut unusable = super::accounts::card_document(b"anthropic", b"", c::AccountKind::Program)
                .accounts
                .get(0)
                .unwrap()
                .clone();
            unusable.sources.get_mut(0).unwrap().usable = false;
            let usable =
                super::accounts::card_document(b"openai", b"", c::AccountKind::SignIn).accounts.get(0).unwrap().clone();
            doc.accounts = skein_lib::List::with_capacity(3);
            doc.accounts.push(unusable.clone()).unwrap();
            doc.accounts.push(usable).unwrap();
            unusable.sources.get_mut(0).unwrap().usable = true;
            doc.accounts.push(unusable).unwrap();
        }
        super::accounts::deliver(&mut root, accounts.unwrap(), doc);
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            let line = view.tree().nodes().iter().find(|n| n.styles.as_slice().contains(&look::BYLINE));
            assert_eq!(line.and_then(|n| n.name.as_deref()), expected.map(str::as_bytes));
        }
    }
}
