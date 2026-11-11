//! Only you's browser region (temper/docs/design/client/pages/README.md, Only you).
#![expect(clippy::disallowed_methods, clippy::disallowed_types, reason = "ordinary tests inspect semantic subtrees")]
use super::{accounts, fixtures as f, inbox, needs};
use crate::look;
use jig_client::contract as c;
use skein_lib::{Env, List, Queue, Time};
use skein_ui::{Act, Role};
use temper_client_domain::{self as d, Address, pages};

const SIGNED_OUT: &str = "Your ChatGPT sign-in is signed out. Sign in again?";
const EXPIRING: &str = "Your Claude Code login claude-alice is expiring. Sign in again?";

fn deliver(root: &mut d::Domain, document: c::Accounts) {
    let handle = pages::accounts::handle();
    f::root_event(root, d::Event::Client { event: jig_client::Event::Unwatch { handle } });
    let mut out = Queue::uncapped();
    d::step(
        root,
        &Env { now: Time::ZERO, wall: f::NOW, limits: f::domain_limits() },
        d::Event::Client {
            event: jig_client::Event::Watch { handle, subject: jig_client::Watchable::Jig(c::Subject::Accounts) },
        },
        &mut out,
    );
    let watch = out
        .iter()
        .find_map(|request| {
            if let d::Request::Unlinked {
                request:
                    jig_client::UnlinkedRequest::Down {
                        down:
                            jig_client::Down::Open { watch, subject: jig_client::Watchable::Jig(c::Subject::Accounts) },
                    },
            } = request
            {
                Some(*watch)
            } else {
                None
            }
        })
        .expect("accounts watch reopened");
    accounts::deliver(root, watch, document);
}

fn document(party: u64, questions: &[&str], cut: bool) -> c::Accounts {
    let mut document = accounts::document(0);
    document.accounts = List::with_capacity(u32::try_from(questions.len()).unwrap());
    document.cut = cut;
    for (index, question) in questions.iter().enumerate() {
        let program = *question == EXPIRING;
        let mut fixture = accounts::card_document(
            if program { b"anthropic" } else { b"openai" },
            if program { b"claude-alice" } else { b"Alice" },
            if program { c::AccountKind::Program } else { c::AccountKind::SignIn },
        );
        let row = fixture.accounts.get_mut(0).unwrap();
        row.holder = c::AccountHolder::Party(party);
        row.number = u32::try_from(index).unwrap().checked_add(10).unwrap();
        let source = row.sources.get_mut(0).unwrap();
        source.number = row.number.checked_add(20).unwrap();
        source.state = if program { c::AccountState::Granted } else { c::AccountState::SignInAgain };
        source.expiring = program;
        document.accounts.push(row.clone()).unwrap();
    }
    document
}

#[test]
fn only_you_follows_the_answered_inbox_and_links_its_sources_to_accounts() {
    for (name, known, open, cut, questions, heading, filter) in [
        ("no item", true, true, false, &[][..], None, None),
        ("inbox not answered", false, false, false, &[SIGNED_OUT][..], None, None),
        ("signed out with a card", true, true, false, &[SIGNED_OUT][..], Some("Only you · 1"), None),
        ("expiring login", true, true, false, &[EXPIRING][..], Some("Only you · 1"), None),
        ("cut in document order", true, true, true, &[EXPIRING, SIGNED_OUT][..], Some("Only you · 2+"), None),
        ("done", true, false, false, &[SIGNED_OUT][..], Some("Only you · 1"), None),
        ("filtered", true, true, false, &[SIGNED_OUT][..], Some("Only you · 1"), Some(f::PROJECT)),
    ] {
        let filter = d::address::Filter { project: filter, ..d::address::Filter::ALL };
        let mut root = if !known {
            f::root_at(Address::Inbox { filter }, &f::domain_limits()).0
        } else if open && questions.is_empty() {
            needs::fixture("Which way?\nChoice: This way", false, 1, false, filter)
        } else {
            needs::decided_fixture(filter, 4, open)
        };
        let party = root.pages().me(root.client()).unwrap().party.number;
        deliver(&mut root, document(party, questions, cut));
        assert_eq!(usize::try_from(pages::only_you::read(&root).items.len()).unwrap(), questions.len(), "{name}");
        let mut view = f::browser_page_view(&root);
        let tree = view.tree();
        let regions: Vec<_> = tree.nodes().iter().filter(|n| n.styles.as_slice().contains(&look::ONLY)).collect();
        assert_eq!(regions.len(), usize::from(heading.is_some()), "{name}");
        let Some(heading) = heading else { continue };
        assert_eq!(regions[0].role, Role::Region, "{name}");
        assert_eq!(regions[0].name.as_deref(), Some(heading.as_bytes()), "{name}");
        let region = inbox::styled(tree, look::ONLY);
        let nodes = inbox::subtree(tree, region);
        assert!(
            nodes.iter().any(|n| n.role == Role::Heading { level: 2 } && n.name.as_deref() == Some(heading.as_bytes()))
        );
        assert!(nodes.iter().any(|n| n.name.as_deref() == Some(b"work goes on around it")));
        let cards: Vec<_> = nodes.iter().filter(|n| n.styles.as_slice().contains(&look::ONLY_ITEM)).collect();
        assert_eq!(cards.len(), questions.len(), "{name}");
        for (card, question) in cards.iter().zip(questions) {
            let at = tree.nodes().iter().position(|n| n.id == card.id).unwrap();
            let card = inbox::subtree(tree, at);
            let words: Vec<_> = card.iter().filter_map(|n| n.name.as_deref()).collect();
            assert_eq!(
                words,
                [b"Deployment".as_slice(), b"an LLM account", question.as_bytes(), b"Sign in again"],
                "{name}"
            );
            assert!(
                card.iter()
                    .any(|n| n.styles.as_slice().contains(&look::CHIP) && n.name.as_deref() == Some(b"Deployment"))
            );
            assert!(
                card.iter()
                    .any(|n| n.styles.as_slice().contains(&look::ONLY_Q)
                        && n.name.as_deref() == Some(question.as_bytes()))
            );
            let choices = card.iter().position(|n| n.styles.as_slice().contains(&look::ASK_CHOICES)).unwrap();
            let link = &card[choices + 1];
            assert_eq!(link.role, Role::Link);
            assert!(link.styles.as_slice().contains(&look::RECOMMENDED));
        }
        let main = inbox::subtree(tree, tree.nodes().iter().position(|n| n.role == Role::Main).unwrap());
        let before = main.iter().position(|n| n.styles.as_slice().contains(&look::ONLY)).unwrap();
        if open {
            let last_card = main.iter().rposition(|n| n.styles.as_slice().contains(&look::ASK)).unwrap();
            assert!(last_card + usize::try_from(main[last_card].size).unwrap() <= before, "{name}");
        } else {
            assert!(f::find(tree, Role::Heading { level: 1 }, b"Nothing needs you.").is_some());
            assert!(
                main.iter().position(|n| n.role == Role::Link && n.name.as_deref() == Some(b"Home")).unwrap() < before
            );
        }
        assert!(region < tree.nodes().iter().position(|n| n.styles.as_slice().contains(&look::DECIDED_HEAD)).unwrap());
        let links: Vec<_> = nodes.iter().filter(|n| n.role == Role::Link).map(|n| n.id).collect();
        assert_eq!(links.len(), questions.len());
        for link in links {
            assert_eq!(
                crate::act(&mut view, link, Act::Press),
                Some(d::Action::Pages(pages::Act::Go { address: Address::Accounts }))
            );
        }
        let terminal = f::page_view(&root);
        assert!(!terminal.tree().nodes().iter().any(|n| n.styles.as_slice().contains(&look::ONLY)));
    }
}
