//! Work's three read lists, five filters and both shells (client/pages/README.md).
use super::{chats, fixtures as f, goals};

use crate::{
    frame::Bindings,
    look,
    pages::work,
    words::{Times, Words},
};

use jig_client::{Down, Readable, Readout, Up, Watchable, contract as c};

use skein_lib::{List, Queue, Time, Token, Wall};

use skein_ui::{Builder, Role, Tree};

use temper_client_domain::{
    Address, Domain, Event, GoalsFilter, Place, Request, WorkFilter,
    forge::ways::Ways,
    pages::work::{Card, Item, Landed, LandedDraft, Work},
};

use temper_client_protocol_wire::{self as wire, v1 as w};
use crate::work_tests::work::{CHAT, DRAFT, Fixture, GOAL, INDEPENDENT, LANDED, NOW, QUEUED_CHAT, QUEUED_GOAL, RECENT, has, limits, line, list, note, notes, styled, text};

#[test]
fn sections_wait_for_answers_omit_empty_and_draw_the_whole_small_project() {
    for words in [f::browser_words(), f::words()] {
        let mut page = Fixture::new(None, false);
        let (unread, _) = page.tree(&words);
        assert!(styled(&unread, look::WORK_HEADLINE).is_empty());
        assert!(styled(&unread, look::WORK_SECTION).is_empty());
        assert!(!has(&unread, "Nothing in flight yet."));
        page.populate(false);
        let (empty, _) = page.tree(&words);
        assert!(has(&empty, "Nothing in flight yet."));
        assert!(styled(&empty, look::WORK_SECTION).is_empty());
        let mut page = Fixture::new(None, false);
        page.populate(true);
        let (tree, bindings) = page.tree(&words);
        assert_eq!(skein_ui::outline::check(&tree), None);
        for value in [
            "Shop · Work",
            "Three in flight, one draft, one landed this week.",
            "3 queued",
            "main red",
            "In flight · 3",
            "newest activity first",
            "goal",
            "membership-handoff",
            "Task by task",
            "31 of 50 spent",
            "Six of nine tasks landed.",
            "6 / 9",
            "Written by the goal at 07:54",
            "Witness retention",
            "checking",
            "Handoff states",
            "landed 07:54",
            "change",
            "Witness rejoin test",
            "queued 3rd",
            "chat",
            "with you",
            "The comment is stale",
            "Comment rewritten",
            "queued 2nd",
            "Drafts · 1",
            "a goal starts from each",
            "draft",
            "read-barriers.md",
            "a chat with you",
            "Read barriers",
            "Open the draft",
            "Landed this week · 1",
        ] {
            assert!(has(&tree, value), "missing {value}");
        }
        assert_eq!(styled(&tree, look::WORK_SECTION).len(), 3);
        let mut addresses = List::uncapped();
        for node in tree.nodes() {
            if let Some(binding) = node.link {
                addresses.push(bindings.decode(binding.token, skein_ui::Act::Press)).unwrap();
            }
        }
        for address in [
            Address::Task { task: GOAL },
            Address::Task { task: INDEPENDENT },
            Address::Chat { task: CHAT },
            Address::Chat { task: DRAFT },
            Address::Task { task: QUEUED_GOAL },
            Address::Task { task: QUEUED_CHAT },
            Address::Task { task: LANDED },
        ] {
            assert!(addresses.as_slice().contains(&Some(temper_client_domain::Action::Pages(
                temper_client_domain::pages::Act::Go { address }
            ))));
        }
        let view_limits = crate::Limits::of(&limits(), &words).unwrap();
        let mut full_view = crate::View::new(&view_limits, &limits(), words.clone(), f::FORGE);
        crate::build(&mut full_view, &page.root, NOW, &mut Queue::uncapped());
        assert!(has(full_view.tree(), "Three in flight, one draft, one landed this week."), "Work's root dispatch");
        assert_eq!(skein_ui::outline::check(full_view.tree()), None);
        page.go(Address::Work { project: f::PROJECT, filter: WorkFilter::Drafts });
        let (tree, _) = page.tree(&words);
        assert_eq!(styled(&tree, look::WORK_SECTION).len(), 1);
        assert_eq!(styled(&tree, look::WORK_CARD).len(), 1);
        assert!(has(&tree, "Open the draft"));
        assert!(!has(&tree, "In flight · 3"));
    }
}
