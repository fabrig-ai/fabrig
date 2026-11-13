//! Home's shared cards (temper/docs/design/client/pages/README.md, Home, The cards).
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "view tables inspect semantic nodes and bindings"
)]
use super::fixtures as f;

use crate::{
    bindings::Bindings,
    look,
    words::{Times, Words},
};

use jig_client::contract as c;

use skein_ui::{Builder, Role, Tree};

use temper_client_domain::{self as d, pages::needs::Need};

fn draw(root: &d::Domain, needs: &[Need<'_>], across: bool, w: &Words) -> (Tree, Bindings) {
    let mut b = Builder::new(&f::tree_limits());
    let mut bindings = Bindings::new(128);
    b.open(Role::Main);
    crate::pages::home_cards::cards(&mut b, root, needs, across, w, &mut bindings, &mut Times::new(f::NOW));
    b.close();
    (b.finish(), bindings)
}

fn control(tree: &Tree, bindings: &Bindings, name: &str) -> d::Action {
    let n = tree
        .nodes()
        .iter()
        .find(|n| n.name.as_deref() == Some(name.as_bytes()) && n.binding.is_some())
        .expect("bound control");
    bindings.decode(n.binding.unwrap().token, skein_ui::Act::Press).unwrap()
}

fn question_card(root: &d::Domain) -> d::pages::inbox::Card {
    let mut card =
        root.pages().inbox().unwrap().cards.iter().find(|card| card.group == c::InboxGroup::Questions).unwrap().clone();
    let question = c::Question {
        task: f::task::document().chip,
        from: None,
        message: f::INBOX_QUESTION,
        words: c::DocumentText::complete(
            c::Text::new(
                b"Hold the suffix?\n\nChoice: Count it\nChoice: Catch up first (recommended)".as_slice().into(),
                4096,
            )
            .unwrap(),
        ),
        since: skein_lib::Wall::EPOCH,
        acts: skein_lib::List::uncapped(),
        ending: None,
    };
    card.page = Some(c::ReadValue::Question(Box::new(question)));
    card
}

#[test]
fn three_cards_keep_order_project_scope_and_the_remaining_link() {
    let root = super::needs::fixture("A question?", false, 1, false, d::address::Filter::ALL);
    let base = question_card(&root);
    for (count, more) in [(3, None), (4, Some("One more")), (5, Some("2 more"))] {
        let cards: Vec<_> = (1..=count)
            .map(|i| {
                let mut card = base.clone();
                card.object = c::ObjectKey::Question { task: f::INBOX_TASK, message: i };
                card
            })
            .collect();
        let needs: Vec<_> = cards.iter().map(Need::Card).collect();
        for across in [false, true] {
            for w in [f::words(), f::browser_words()] {
                let (tree, bindings) = draw(&root, &needs, across, &w);
                assert_eq!(tree.nodes().iter().filter(|n| n.styles.as_slice().contains(&look::NEED_CARD)).count(), 3);
                assert_eq!(look::NAMES.style(look::NEED_CARD), Some(b"temper-need-card".as_slice()));
                assert_eq!(
                    tree.nodes().iter().filter(|n| n.styles.as_slice().contains(&look::CHIP)).count(),
                    if across { 3 } else { 0 }
                );
                let objects: Vec<_> = tree
                    .nodes()
                    .iter()
                    .filter(|n| n.name.as_deref() == Some(b"Tell it something else"))
                    .map(|n| bindings.decode(n.binding.unwrap().token, skein_ui::Act::Press).unwrap())
                    .collect();
                assert_eq!(
                    objects,
                    (1..=3)
                        .map(|message| d::Action::Pages(d::pages::Act::Needs(d::pages::needs::Act::Open {
                            object: c::ObjectKey::Question { task: f::INBOX_TASK, message }
                        })))
                        .collect::<Vec<_>>()
                );
                assert_eq!(tree.nodes().iter().any(|n|n.role==Role::Link && n.name.as_deref().is_some_and(|name|name.ends_with(b" more"))),more.is_some());
                if let Some(more) = more {
                    assert_eq!(
                        control(&tree, &bindings, more),
                        d::Action::Pages(d::pages::Act::Go {
                            address: d::Address::Inbox {
                                filter: d::address::Filter {
                                    project: if across { None } else { root.pages().project() },
                                    ..d::address::Filter::ALL
                                }
                            }
                        })
                    );
                }
            }
        }
    }
}

#[test]
fn a_question_orders_the_primary_choice_before_other_acts() {
    let root = super::needs::fixture("A question?", false, 1, false, d::address::Filter::ALL);
    let card = question_card(&root);
    for w in [f::words(), f::browser_words()] {
        let (tree, bindings) = draw(&root, &[Need::Card(&card)], true, &w);
        let buttons: Vec<_> = tree
            .nodes()
            .iter()
            .filter(|n| n.role == Role::Button && !n.styles.as_slice().contains(&look::NEED_MORE))
            .collect();
        assert_eq!(
            buttons.iter().map(|n| n.name.as_deref().unwrap()).collect::<Vec<_>>(),
            vec![b"Catch up first".as_slice(), b"Count it", b"Tell it something else"]
        );
        assert!(buttons[0].styles.as_slice().contains(&look::RECOMMENDED));
        assert_eq!(
            control(&tree, &bindings, "Catch up first"),
            d::Action::Pages(d::pages::Act::Needs(d::pages::needs::Act::Choose {
                object: card.object,
                words: b"Catch up first".as_slice().into()
            }))
        );
        assert!(f::find(&tree, Role::Paragraph, b"Hold the suffix?").is_some());
        assert_eq!(f::find(&tree, Role::Button, b"More").is_some(), w.shell == crate::words::Shell::Browser);
    }
}

#[test]
fn plain_questions_have_a_primary_way_to_tell() {
    let root = super::needs::fixture("A question?", false, 1, false, d::address::Filter::ALL);
    let mut card = question_card(&root);
    if let Some(c::ReadValue::Question(q)) = &mut card.page {
        q.words = c::DocumentText::complete(c::Text::new(b"No choices here.".as_slice().into(), 64).unwrap());
    }
    for w in [f::words(), f::browser_words()] {
        let (tree, bindings) = draw(&root, &[Need::Card(&card)], false, &w);
        let tell = tree.nodes().iter().find(|n| n.name.as_deref() == Some(b"Tell it something else")).unwrap();
        assert!(tell.styles.as_slice().contains(&look::RECOMMENDED));
        assert_eq!(
            control(&tree, &bindings, "Tell it something else"),
            d::Action::Pages(d::pages::Act::Needs(d::pages::needs::Act::Open { object: card.object }))
        );
    }
}

#[test]
fn proposals_held_tasks_and_approvals_keep_their_acts() {
    let mut proposal = super::needs::proposal(c::Proposer::Task(f::INBOX_TASK));
    proposal.key.number = f::INBOX_QUESTION;
    let proposal_root = f::inbox_with_document(
        d::address::Filter::ALL,
        Some((c::Read::Proposal { proposal: proposal.key }, c::ReadValue::Proposal(Box::new(proposal)))),
    );
    let source = f::task::held_root();
    let mut task = source.pages().task().unwrap().document(source.client()).unwrap().clone();
    task.chip.task = f::INBOX_TASK;
    let offers = [c::ActKind::Release, c::ActKind::LeaveHeld]
        .into_iter()
        .map(|act| c::Offered { act, admission: c::Admission::Allowed, effect: None })
        .collect::<Vec<_>>();
    let held = c::Escalation { task: Box::new(task), acts: skein_lib::List::from_boxed(offers.into_boxed_slice()) };
    let held_root = f::inbox_with_document(
        d::address::Filter::ALL,
        Some((c::Read::Escalation { task: f::INBOX_TASK }, c::ReadValue::Escalation(Box::new(held)))),
    );
    let mut funds = held_root
        .pages()
        .inbox()
        .unwrap()
        .cards
        .iter()
        .find(|c| c.group == c::InboxGroup::Escalations)
        .unwrap()
        .clone();
    if let Some(c::ReadValue::Escalation(held)) = &mut funds.page {
        held.task.hold.as_mut().unwrap().reason = c::HoldCategory::Budget;
        held.task.budget.numbers.spent = 31;
        held.task.budget.numbers.budget = 40;
    }
    for (root, group, expected) in [
        (&proposal_root, c::InboxGroup::Proposals, ["Accept", "Reject"]),
        (&held_root, c::InboxGroup::Escalations, ["Release", "Leave held"]),
    ] {
        let card = root.pages().inbox().unwrap().cards.iter().find(|c| c.group == group).unwrap();
        for w in [f::words(), f::browser_words()] {
            let (tree, _) = draw(root, &[Need::Card(card)], true, &w);
            for name in expected {
                assert!(tree.nodes().iter().any(|n| n.name.as_deref() == Some(name.as_bytes())), "{name}");
            }
        }
    }
    for w in [f::words(), f::browser_words()] {
        let (tree, _) = draw(&held_root, &[Need::Card(&funds)], true, &w);
        assert!(f::find(&tree, Role::Paragraph, b"31 of 40 used.").is_some());
    }
    let mut approval = super::needs::approval_row();
    approval.kind = d::forge::forges::Kind::GitHub;
    let root = super::needs::fixture("A question?", false, 1, false, d::address::Filter::ALL);
    for w in [f::words(), f::browser_words()] {
        let (tree, bindings) = draw(&root, &[Need::Approval(&approval)], true, &w);
        let review = tree.nodes().iter().find(|n| n.name.as_deref() == Some("Review on GitHub ↗".as_bytes())).unwrap();
        assert!(review.styles.as_slice().contains(&look::RECOMMENDED));
        assert_eq!(
            control(&tree, &bindings, "Tell it something else"),
            d::Action::Pages(d::pages::Act::Needs(d::pages::needs::Act::Open {
                object: c::ObjectKey::Task(approval.task)
            }))
        );
    }
}

#[test]
fn approval_size_follows_the_question_before_acts_or_draws_nothing() {
    let root = super::needs::fixture("A question?", false, 1, false, d::address::Filter::ALL);
    for (size, words, files) in super::needs::size_cases() {
        let row = d::forge::approvals::Row { size, ..super::needs::approval_row() };
        let (tree, bindings) = draw(&root, &[Need::Approval(&row)], false, &f::browser_words());
        super::needs::check_size(&tree, words, files);
        let nodes = tree.nodes();
        let question = nodes.iter().position(|n| n.styles.as_slice().contains(&look::ASK_QUESTION)).unwrap();
        let after = question + usize::try_from(nodes[question].size).unwrap();
        let acts = nodes.iter().position(|n| n.styles.as_slice().contains(&look::NEED_ACTS)).unwrap();
        let context = nodes.iter().position(|n| n.styles.as_slice().contains(&look::ASK_CONTEXT)).unwrap();
        if size.is_some() {
            assert!(nodes[after].styles.as_slice().contains(&look::CHIPS));
            assert_eq!(after + usize::try_from(nodes[after].size).unwrap(), context);
        } else {
            assert_eq!(after, context, "no placeholder without a size");
        }
        assert_eq!(context + usize::try_from(nodes[context].size).unwrap(), acts);
        assert!(f::find(&tree, Role::Link, "Review on Forgejo ↗".as_bytes()).is_some());
        assert_eq!(
            control(&tree, &bindings, "Tell it something else"),
            d::Action::Pages(d::pages::Act::Needs(d::pages::needs::Act::Open { object: c::ObjectKey::Task(row.task) }))
        );
    }
}

#[test]
fn reason_line_stays_under_the_question_and_keeps_asking_under_its_offers() {
    use d::pages::{self, stop_asking::Act as Stop};
    for owner in [true, false] {
        let mut root = super::needs::stop_root(owner, true);
        let object = c::ObjectKey::Question { task: f::INBOX_TASK, message: 1 };
        for opened in [false, true] {
            if opened {
                f::root_event(
                    &mut root,
                    d::Event::Act { action: d::Action::Pages(pages::Act::StopAsking(Stop::Open { object })) },
                );
            }
            let card = root.pages().inbox().unwrap().cards.get(0).unwrap();
            for w in [f::browser_words(), f::words()] {
                let (tree, bindings) = draw(&root, &[Need::Card(card)], false, &w);
                assert_eq!(
                    control(&tree, &bindings, "Stop asking"),
                    d::Action::Pages(pages::Act::StopAsking(Stop::Open { object }))
                );
                let position = |style| tree.nodes().iter().position(|n| n.styles.as_slice().contains(&style)).unwrap();
                assert!(position(look::ASK_QUESTION) < position(look::WHY));
                assert!(position(look::WHY) < position(look::NEED_ACTS));
                assert!(!tree.nodes().iter().any(|n| n.name.as_deref()
                    == Some(b"Stop asking: let agents decide some changes to protocol core".as_slice())));
                let keep = f::find(&tree, Role::Button, b"Keep asking");
                assert_eq!(keep.is_some(), opened);
                if opened {
                    assert_eq!(
                        control(&tree, &bindings, "Keep asking"),
                        d::Action::Pages(pages::Act::StopAsking(Stop::Keep { object }))
                    );
                    let offers = super::inbox::subtree(&tree, position(look::WHY_OFFERS));
                    let buttons: Vec<_> = offers.iter().filter(|n| n.role == Role::Button).collect();
                    assert_eq!(buttons.last().unwrap().name.as_deref(), Some(b"Keep asking".as_slice()));
                    assert!(buttons.last().unwrap().styles.as_slice().contains(&look::ASK_QUIET));
                }
            }
        }
    }
}
