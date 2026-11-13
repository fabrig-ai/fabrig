use super::fixtures;
pub(super) use super::fixtures::cards::batch_proposal;
use jig_client::contract as c;

fn occurrences(tree: &skein_ui::Tree, text: &[u8]) -> usize {
    let written = fixtures::text(tree);
    let mut count = 0_usize;
    for part in written.windows(text.len()) {
        if part == text {
            count = count.checked_add(1).unwrap();
        }
    }
    count
}
#[test]
fn proposal_titles_head_the_cards_and_funding_is_said_once_in_both_pages() {
    use skein_ui::Role;
    for (page, owner) in [(batch_proposal(), true), (document_proposal(), false)] {
        for root in fixtures::pair(
            fixtures::chat::proposal_root_with_document(None, Some(page.clone())),
            fixtures::inbox_with_document(
                temper_client_domain::address::Filter::ALL,
                Some((
                    c::Read::Proposal {
                        proposal: c::ProposalKey {
                            proposer: c::Proposer::Task(fixtures::INBOX_TASK),
                            number: fixtures::INBOX_QUESTION,
                        },
                    },
                    page,
                )),
            ),
        ) {
            let view = fixtures::page_view(&root);
            let tree = view.tree();
            let budget = super::chat::named_position(tree.nodes(), Role::Term, Some(b"Budget")).expect("Budget row");
            let description = budget.checked_add(usize::try_from(tree.nodes()[budget].size).unwrap()).unwrap();
            if owner {
                let title = super::chat::named_position(
                    tree.nodes(),
                    Role::Heading { level: if root.pages().inbox().is_some() { 3 } else { 2 } },
                    Some(b"Ship update"),
                )
                .expect("proposal title heads the card");
                assert!(title < budget);
                for expected in [b"Funding needed: 40".as_slice(), b"Funding left: 120", b"Funding after: 80"] {
                    assert!(skein_lib::bytes::find(&fixtures::node_text(tree, description), expected).is_some());
                    assert_eq!(occurrences(tree, expected), 1);
                }
                for expected in [b"May spend 6".as_slice(), b"May delegate 1 task, depth 1", b"Budget 6"] {
                    assert_eq!(occurrences(tree, expected), 0);
                }
            } else {
                let funding = b"Funding needed: 40";
                assert!(skein_lib::bytes::find(&fixtures::node_text(tree, description), funding).is_some());
                assert_eq!(occurrences(tree, funding), 1);
                for expected in [b"May spend 40".as_slice(), b"May delegate 1 task, depth 1"] {
                    assert_eq!(occurrences(tree, expected), 0);
                }
            }
        }
    }
}

#[test]
fn a_proposal_kicker_precedes_the_title_in_a_chat_but_not_in_the_inbox() {
    use crate::cards::Place;
    use skein_ui::Role;
    let page = batch_proposal();
    // One root at a time: two outgrow a stack array's 16 KiB.
    for place in [Place::Chat, Place::Inbox] {
        let root = if place == Place::Chat {
            fixtures::chat::proposal_root_with_document(None, Some(page.clone()))
        } else {
            fixtures::inbox_with_document(
                temper_client_domain::address::Filter::ALL,
                Some((
                    c::Read::Proposal {
                        proposal: c::ProposalKey {
                            proposer: c::Proposer::Task(fixtures::INBOX_TASK),
                            number: fixtures::INBOX_QUESTION,
                        },
                    },
                    page.clone(),
                )),
            )
        };
        let view = fixtures::page_view(&root);
        let tree = view.tree();
        let level = if place == Place::Chat { 2 } else { 3 };
        let title = super::chat::named_position(tree.nodes(), Role::Heading { level }, Some(b"Ship update")).unwrap();
        let mut kickers = 0_u32;
        let mut first = None;
        for (at, node) in tree.nodes().iter().enumerate() {
            if node.styles.as_slice().contains(&crate::look::PROPOSAL_KICKER) {
                kickers += 1;
                first = first.or(Some(at));
            }
        }
        if place == Place::Chat {
            assert_eq!(kickers, 1, "one Proposal kicker in a chat");
            let kicker = first.expect("Proposal kicker");
            assert_eq!(fixtures::node_text(tree, kicker).as_ref(), b"Proposal");
            assert!(kicker < title);
        } else {
            assert_eq!(kickers, 0, "the inbox's tag says the kind: no kicker");
        }
    }
}

use skein_lib::List;
use skein_ui::Role;
use temper_client_domain::{self as d, Action, Event, pages::Act};
use temper_client_protocol_wire::v1 as w;
const SETTINGS_KEY: c::ProposalKey = fixtures::chat::PROPOSAL;
const SETTINGS_REASON: &[u8] = b"I work alone\nKeep drafts beside the code";
const SETTINGS_KINDS: [(c::Setting, &[u8]); 16] = [
    (c::Setting::Charter(c::SettingNumber { number: 2 }), b"charter 2"),
    (c::Setting::Template(c::SettingNumber { number: 40 }), b"template 40"),
    (c::Setting::Policy(c::SettingNumber { number: 1 }), b"policy 1"),
    (c::Setting::Rules, b"rules"),
    (c::Setting::Application { kind: 1, scope: 0, number: 1 }, b"landing rule 1"),
    (c::Setting::Application { kind: 2, scope: fixtures::PROJECT, number: 1 }, b"landing rule 1"),
    (c::Setting::Application { kind: 3, scope: 0, number: 1 }, b"forge charter 1"),
    (c::Setting::Application { kind: 11, scope: 0, number: 4 }, b"template-goal 4"),
    (c::Setting::Application { kind: 12, scope: 2, number: 1 }, b"follow 1"),
    (c::Setting::ProjectPolicy { project: 2 }, b"project policy 2"),
    (c::Setting::Application { kind: 6, scope: fixtures::PROJECT, number: 1 }, b"project-questions 1"),
    (c::Setting::Application { kind: 7, scope: fixtures::PROJECT, number: 1 }, b"project-ways 1"),
    (c::Setting::Application { kind: 8, scope: fixtures::PROJECT, number: 1 }, b"project-classes 1"),
    (c::Setting::Application { kind: 5, scope: 7, number: 3 }, b"application kind 5, 7, 3"),
    (c::Setting::Application { kind: 9, scope: 0, number: 4 }, b"charter-words 4"),
    (c::Setting::Application { kind: 10, scope: fixtures::PROJECT, number: 1 }, b"project-words 1"),
];

#[test]
#[expect(clippy::disallowed_methods, reason = "ordinary tests compare semantic nodes and concatenate expected labels")]
fn a_settings_proposal_draws_its_title_lines_and_items() {
    let edits = [
        (c::RevisionItemEdit::Set, ""),
        (c::RevisionItemEdit::Removed, " removed"),
        (c::RevisionItemEdit::Reset, " reset"),
    ];
    let (root, _) = settings_root(settings_page(&[], true));
    for (setting, label) in SETTINGS_KINDS {
        for (edit, suffix) in &edits {
            let mut card = settings_card(&root);
            card.page = Some(settings_page(&[(setting, edit.clone())], true));
            let tree = draw_settings(&root, &card, false);
            assert!(fixtures::find(&tree, Role::Paragraph, b"How temper will run Shop").is_some());
            let said = fixtures::text(&tree);
            for line in SETTINGS_REASON.split(|b| *b == b'\n') {
                assert!(skein_lib::bytes::find(&said, line).is_some());
            }
            let expected = [label, suffix.as_bytes()].concat();
            assert!(skein_lib::bytes::find(&said, &expected).is_some(), "{expected:?}");
            let node = tree.nodes().iter().find(|n| n.text.iter().any(|run| run.text.as_ref() == expected)).unwrap();
            assert!(node.styles.as_slice().contains(&crate::look::SETTINGS_DETAIL));
        }
    }
    for owned in [Some(false), Some(true), None] {
        let items =
            [(c::Setting::Application { kind: 2, scope: fixtures::PROJECT, number: 1 }, c::RevisionItemEdit::Set)];
        let (mut root, out) = settings_root(settings_page(&items, true));
        if let Some(owned) = owned {
            proposed_values(&mut root, &out, true, w::QuestionsRoute::None, owned);
        }
        let tree = draw_settings(&root, &settings_card(&root), false);
        let sentence = b"On this machine temper's agents and CI jobs run as you, so this stops their mistakes, not one trying to get past it.";
        assert_eq!(
            skein_lib::bytes::find(&fixtures::text(&tree), sentence).is_some(),
            owned == Some(true),
            "owned_approval={owned:?}"
        );
        let inbox = super::needs::settings_owned(settings_page(&items, true), owned);
        let view = fixtures::browser_page_view(&inbox);
        assert_eq!(
            skein_lib::bytes::find(&fixtures::main_text(view.tree()), sentence).is_some(),
            owned == Some(true),
            "Needs you owned_approval={owned:?}"
        );
        if owned == Some(true) {
            let at = tree
                .nodes()
                .iter()
                .enumerate()
                .find(|(at, node)| node.role == Role::Paragraph && fixtures::node_text(&tree, *at).as_ref() == sentence)
                .unwrap()
                .0;
            assert!(tree.nodes()[at].styles.as_slice().contains(&crate::look::SETTINGS_PROPOSAL_ITEMS));
            assert!(tree.nodes().iter().any(|node| node.styles.as_slice().contains(&crate::look::SETTINGS_DETAIL)
                && node.text.iter().any(|run| run.text.as_ref() == sentence)));
            assert!(tree.nodes()[..at].iter().any(|node| node.role == Role::List
                && node.styles.as_slice().contains(&crate::look::SETTINGS_PROPOSAL_ITEMS)));
        }
    }
}

#[test]
fn accept_decides_with_running_work_staying() {
    let (mut root, _) = settings_root(settings_page(&[], true));
    let mut view = fixtures::page_view(&root);
    let accept = fixtures::find(view.tree(), Role::Button, b"Accept").unwrap();
    let node = view.tree().find(accept).unwrap();
    assert!(node.styles.as_slice().contains(&crate::look::SETTINGS_PRIMARY));
    let event = crate::act_event(&mut view, accept, skein_ui::Act::Press).unwrap();
    settings_drive(&mut root, event);
    let mut out = settings_drive(&mut root, Event::Act { action: Action::Pages(Act::Confirm) });
    let mut stored = settings_drive(&mut root, Event::Client { event: jig_client::Event::Stored });
    while let Some(request) = stored.pop() {
        out.push(request);
    }
    let mut decisions = List::with_capacity(1);
    for request in &out {
        if let d::Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down: jig_client::Down::Send { request: jig_client::Ask::Jig(ask), .. },
                },
        } = request
            && let c::Ask::Decide { on: c::DecideOn::Proposal { proposal, running, decision }, .. } = ask.as_ref()
        {
            decisions.push((*proposal, running.clone(), decision.clone())).unwrap();
        }
    }
    assert_eq!(decisions.as_slice(), [(SETTINGS_KEY, Some(c::Running::Stays), c::Decision::Accepted)]);
}
#[test]
fn change_something_focuses_the_composer() {
    for asking in [false, true] {
        let (mut root, _) = settings_root_in(settings_page(&[], true), asking);
        for count in [1, 2] {
            let mut view = fixtures::page_view(&root);
            let button = fixtures::find(view.tree(), Role::Button, b"Change something").unwrap();
            let action = crate::act(&mut view, button, skein_ui::Act::Press).unwrap();
            assert_eq!(action, Action::Pages(Act::ChangeProposal { proposal: SETTINGS_KEY }));
            let out = settings_drive(&mut root, Event::Act { action });
            for request in &out {
                if let d::Request::Unlinked {
                    request: jig_client::UnlinkedRequest::Down { down: jig_client::Down::Send { .. } },
                } = request
                {
                    panic!("focusing sent a request");
                }
            }
            let view = fixtures::page_view(&root);
            let composer = fixtures::find(view.tree(), Role::TextArea, b"Message").unwrap();
            let composer = view.tree().find(composer).unwrap();
            assert_eq!(composer.focus, Some(count));
            assert_eq!(root.pages().chat().unwrap().composer_focus, count);
        }
    }
}

pub(super) use crate::parts_tests::cards::{
    document_proposal, draw_settings, proposed_values, settings_card, settings_drive, settings_page, settings_root,
    settings_root_in,
};
