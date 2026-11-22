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
    let mut bindings = Bindings::new(256);
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

fn group_draw(root: &d::Domain, needs: &[Need<'_>], across: bool, w: &Words) -> (Tree, Bindings) {
    let mut b = Builder::new(&f::tree_limits());
    let mut bindings = Bindings::new(256);
    b.open(Role::Main);
    crate::pages::home_cards::cards(&mut b, root, needs, across, w, &mut bindings, &mut Times::new(f::groups::NOW));
    b.close();
    (b.finish(), bindings)
}
fn styled_nodes(tree: &Tree, style: skein_ui::Style) -> &[skein_ui::Node] {
    let at = tree.nodes().iter().position(|n| n.styles.as_slice().contains(&style)).expect("styled part");
    super::inbox::subtree(tree, at)
}
fn text(nodes: &[skein_ui::Node]) -> String {
    String::from_utf8(nodes.iter().flat_map(|n| n.text.iter().flat_map(|run| run.text.iter().copied())).collect())
        .unwrap()
}

#[test]
fn home_cards_and_shared_need_lines_cut_titles_before_their_prefixes() {
    let long = "abcd ".repeat(20).trim_end().to_owned();
    let cut = format!("{}…", vec!["abcd"; 16].join(" "));
    let mut held = f::needs::held(Some(c::HoldCategory::Stalled));
    held.task.chip.title = f::needs::words(&long);
    let mut milestone = held.task.chip.clone();
    milestone.task = 700;
    milestone.details.details.executor =
        c::Executor::Procedure { connector: jig_client::shapes::RECURRING, code: jig_client::shapes::GATHERING };
    let mut goal = held.task.chip.clone();
    goal.task = 701;
    held.task.requester = c::Requester::Task(701);
    held.task.lineage_chips = skein_lib::List::from_boxed(Box::new([milestone, goal]));
    let root = f::needs::kinds(&[(
        c::ObjectKey::Escalation(held.task.chip.task),
        c::Read::Escalation { task: held.task.chip.task },
        c::ReadValue::Escalation(Box::new(held)),
    )]);
    let card = root.pages().inbox().unwrap().cards.get(0).unwrap();
    let need = Need::Card(card);
    let mut approval = f::needs::approval_row();
    approval.title = long.as_bytes().into();
    for w in [f::browser_words(), f::words()] {
        let (tree, _) = draw(&root, &[need], false, &w);
        let from = styled_nodes(&tree, look::NEED_FROM);
        assert!(text(from).starts_with(&format!("{cut} › {cut} › {cut} · ")));
        let links: Vec<_> = from.iter().filter(|n| n.role == Role::Link).collect();
        assert_eq!(links.len(), 3);
        assert!(links.iter().all(|n| n.name.as_deref() == Some(cut.as_bytes())));
        assert_eq!(text(styled_nodes(&tree, look::ASK_QUESTION)), "The Ready page waits for this decision.");
        assert_eq!(crate::shared::need_line(&need, &w).words.as_ref(), format!("{cut}: {cut}").as_bytes());
        let open = [d::pages::findings::Need::Existing(need); 4];
        let (tree, _) = draw_all(&root, &open, &w);
        assert!(text(styled_nodes(&tree, look::HOME_MORE)).contains(&cut));
        assert!(!text(tree.nodes()).contains(&long));
        let (tree, _) = draw(&root, &[Need::Approval(&approval)], false, &w);
        assert!(text(styled_nodes(&tree, look::ASK_QUESTION)).starts_with(&format!("Approve {cut} on ")));
        assert!(
            styled_nodes(&tree, look::NEED_FROM)
                .iter()
                .any(|n| n.role == Role::Link && n.name.as_deref() == Some(cut.as_bytes()))
        );
    }
}

#[test]
fn home_held_cards_say_why_and_person_cards_name_the_title_only_in_from() {
    let mut held = f::needs::held(Some(c::HoldCategory::EffectFailed));
    held.task.chip.title = f::needs::words("Release the runner");
    held.task.hold.as_mut().unwrap().words = Some(f::needs::words(
        "forge could not release this task's resources; task release 6978 --hold 1; or task cancel 6978 REASON",
    ));
    let person = f::needs::person();
    let root = f::needs::kinds(&[
        (
            c::ObjectKey::Escalation(held.task.chip.task),
            c::Read::Escalation { task: held.task.chip.task },
            c::ReadValue::Escalation(Box::new(held)),
        ),
        (
            c::ObjectKey::PersonTask(person.task.task),
            c::Read::PersonTask { task: person.task.task },
            c::ReadValue::PersonTask(Box::new(person)),
        ),
    ]);
    let cards = root.pages().inbox().unwrap().cards.as_slice();
    let needs = [Need::Card(&cards[0]), Need::Card(&cards[1])];
    for w in [f::browser_words(), f::words()] {
        let (tree, _) = draw(&root, &needs, false, &w);
        let parts: Vec<_> = tree
            .nodes()
            .iter()
            .enumerate()
            .filter(|(_, n)| n.styles.as_slice().contains(&look::NEED_CARD))
            .map(|(at, _)| super::inbox::subtree(&tree, at))
            .collect();
        assert_eq!(parts.len(), 2);
        assert_eq!(text(styled_nodes(&tree, look::ASK_QUESTION)), "Forge could not release this task's resources.");
        assert!(!parts[1].iter().any(|n| n.styles.as_slice().contains(&look::ASK_QUESTION)));
        for (part, title) in parts.iter().zip(["Release the runner", "Review the plan"]) {
            assert_eq!(text(part).matches(title).count(), 1);
            for hidden in ["Hold revision", "task release", "task cancel", "Parameter"] {
                assert!(!text(part).contains(hidden));
            }
        }
        for (need, title) in needs.iter().zip(["Release the runner", "Review the plan"]) {
            let (more, _) = draw_all(&root, &[d::pages::findings::Need::Existing(*need); 4], &w);
            assert!(text(styled_nodes(&more, look::HOME_MORE)).contains(title));
            assert!(core::str::from_utf8(&crate::shared::need_line(need, &w).words).unwrap().contains(title));
        }
    }
}

#[test]
fn decided_rows_cut_task_and_milestone_titles_not_the_question() {
    let long = "abcd ".repeat(20).trim_end().to_owned();
    let cut = format!("{}…", vec!["abcd"; 16].join(" "));
    let row = d::pages::decided::DecisionLine {
        project: f::PROJECT,
        number: 1,
        task: 9,
        title: Some(long.as_bytes().into()),
        milestone: Some(long.as_bytes().into()),
        at: f::NOW,
        question: long.as_bytes().into(),
        took: b"Do it".as_slice().into(),
        why: Box::default(),
        overturned: false,
        state: d::pages::decided::DecisionTask::Unknown,
        reach: None,
    };
    for w in [f::browser_words(), f::words()] {
        let mut builder = Builder::new(&f::tree_limits());
        let mut bindings = Bindings::new(256);
        crate::pages::decided::decision(
            &mut builder,
            None,
            &w,
            &mut bindings,
            &row,
            &mut Times::new(f::NOW),
            None,
            false,
        );
        let tree = builder.finish();
        let drawn = text(tree.nodes());
        assert!(drawn.starts_with(&format!("{cut} › {cut} · ")));
        assert!(drawn.contains(&long));
    }
}

#[test]
fn group_card_keeps_its_findings_context_choices_and_scope() {
    let bugs = f::groups::bugs();
    let root = f::groups::fixture(&[f::groups::QUESTION], &bugs, &[], false);
    let card = root.pages().inbox().unwrap().cards.get(0).unwrap();
    for across in [false, true] {
        for w in [f::browser_words(), f::words()] {
            let (tree, bindings) = group_draw(&root, &[Need::Card(card)], across, &w);
            assert!(
                tree.nodes().iter().any(|n| n.styles.as_slice().contains(&look::GROUP_CARD)
                    && n.styles.as_slice().contains(&look::NEED_CARD))
            );
            assert_eq!(look::NAMES.style(look::GROUP_CARD), Some(b"temper-group-card".as_slice()));
            assert_eq!(look::NAMES.style(look::GROUP_CONTEXT), Some(b"temper-group-context".as_slice()));
            let from = styled_nodes(&tree, look::NEED_FROM);
            let from_text = text(from);
            assert_eq!(
                from_text,
                format!(
                    "{}{}",
                    if across { "Shop " } else { "" },
                    if w.shell == crate::words::Shell::Browser {
                        "3 bugs in temper · the oldest 1 d"
                    } else {
                        "3 bugs · in temper · the oldest 1 d"
                    }
                )
            );
            let rows = styled_nodes(&tree, look::FINDING_ROWS);
            assert_eq!(rows.iter().filter(|n| n.role == Role::Item).count(), 3);
            assert!(!rows.iter().any(|n| n.role == Role::Link || n.binding.is_some()));
            let rows_text = text(rows);
            let mut last = 0;
            for title in bugs.iter().map(|row| core::str::from_utf8(&row.title).unwrap()) {
                let at = rows_text.find(title).unwrap();
                assert!(at >= last);
                last = at;
            }
            if w.shell == crate::words::Shell::Browser {
                assert!(from.iter().any(|n| n.styles.as_slice().contains(&look::CHIP_HIGH)));
                assert_eq!(look::NAMES.style(look::CHIP_HIGH), Some(b"temper-chip-high".as_slice()));
                let chips: Vec<_> = from.iter().filter(|n| n.styles.as_slice().contains(&look::CHIP)).collect();
                assert_eq!(chips.len(), if across { 2 } else { 1 });
                if across {
                    assert_eq!(chips[0].name.as_deref(), Some(b"Shop".as_slice()));
                }
                assert_eq!(rows.iter().filter(|n| n.styles.as_slice().contains(&look::SEVERITY)).count(), 3);
                assert_eq!(
                    rows.iter()
                        .filter(|n| n.styles.as_slice().contains(&look::FINDING_WHEN))
                        .map(|n| n.name.as_deref().unwrap())
                        .collect::<Vec<_>>(),
                    ["qa · 1 d".as_bytes(), "review · 16 h".as_bytes(), "review · 15 h".as_bytes()]
                );
                assert_eq!(
                    control(&tree, &bindings, "More"),
                    d::Action::Pages(d::pages::Act::Needs(d::pages::needs::Act::Open { object: card.object }))
                );
            } else {
                assert!(rows_text.contains("The model picker keeps a later choice · high · qa · 1 d"));
                assert!(rows_text.contains("A resolution ignores later files · low · review · 16 h"));
                assert!(rows_text.contains("A hung forge read holds too long · medium · review · 15 h"));
            }
            assert!(f::find(&tree, Role::Paragraph, b"Start a goal for each?").is_some());
            let context = styled_nodes(&tree, look::GROUP_CONTEXT);
            assert_eq!(text(context), "None blocks a landing.");
            assert!(context[0].styles.as_slice().contains(&look::ASK_CONTEXT));
            let position = |style| tree.nodes().iter().position(|n| n.styles.as_slice().contains(&style)).unwrap();
            assert!(position(look::ASK_QUESTION) < position(look::FINDING_ROWS));
            assert!(position(look::FINDING_ROWS) < position(look::GROUP_CONTEXT));
            assert!(position(look::GROUP_CONTEXT) < position(look::NEED_ACTS));
            let buttons: Vec<_> = styled_nodes(&tree, look::NEED_ACTS)
                .iter()
                .filter(|n| n.role == Role::Button || n.role == Role::Link)
                .collect();
            assert_eq!(
                buttons.iter().map(|n| n.name.as_deref().unwrap()).collect::<Vec<_>>(),
                [b"A goal for each".as_slice(), b"One goal for all", b"Backlog them", b"Tell it something else"]
            );
            assert_eq!(buttons[3].role, Role::Link);
            assert!(buttons[0].styles.as_slice().contains(&look::RECOMMENDED));
            assert!(buttons[1..].iter().all(|n| n.styles.as_slice().contains(&look::ASK_QUIET)));
            assert!(f::find(&tree, Role::Text, b"recommended").is_some());
            assert_eq!(
                control(&tree, &bindings, "A goal for each"),
                d::Action::Pages(d::pages::Act::Needs(d::pages::needs::Act::Choose {
                    object: card.object,
                    words: b"A goal for each".as_slice().into()
                }))
            );
        }
    }
}

#[test]
fn group_rows_omit_a_shared_kind_and_singletons_have_no_oldest_prefix() {
    use temper_client_protocol_wire::v1 as w;
    let rows = [
        f::groups::row(2, "First flake", w::Severity::Low, w::FindingKind::Flake, 24),
        f::groups::row(8, "Second flake", w::Severity::Low, w::FindingKind::Flake, 16),
    ];
    for (question, label, ages) in [
        ("Q?\nFindings: 2/2 2/8\nChoice: A", "2 findings the oldest 1 d", vec!["1 d", "16 h"]),
        ("Q?\nFindings: 2/8\nChoice: A", "1 finding 16 h", vec!["16 h"]),
    ] {
        let root = f::groups::fixture(&[question], &rows, &[], false);
        let card = root.pages().inbox().unwrap().cards.get(0).unwrap();
        let (tree, _) = group_draw(&root, &[Need::Card(card)], false, &f::browser_words());
        assert_eq!(text(styled_nodes(&tree, look::NEED_FROM)), label);
        let rows = styled_nodes(&tree, look::FINDING_ROWS);
        assert_eq!(
            rows.iter()
                .filter(|n| n.styles.as_slice().contains(&look::FINDING_WHEN))
                .map(|n| core::str::from_utf8(n.name.as_deref().unwrap()).unwrap())
                .collect::<Vec<_>>(),
            ages
        );
    }
}

#[test]
fn fourth_group_summarizes_findings_instead_of_the_question() {
    let root = f::groups::fixture(&[f::groups::QUESTION; 4], &f::groups::bugs(), &[], false);
    let needs: Vec<_> = root.pages().inbox().unwrap().cards.iter().map(Need::Card).collect();
    for w in [f::browser_words(), f::words()] {
        let (tree, _) = group_draw(&root, &needs, false, &w);
        let more = styled_nodes(&tree, look::HOME_MORE);
        assert_eq!(
            text(more),
            if w.shell == crate::words::Shell::Browser {
                "One more 3 bugs in temper · the oldest 1 d"
            } else {
                "One more: 3 bugs · high · in temper · the oldest 1 d"
            }
        );
        if w.shell == crate::words::Shell::Browser {
            assert!(more.iter().any(|n| n.styles.as_slice().contains(&look::SEVERITY_HIGH)));
        }
    }
}

#[test]
fn three_cards_keep_order_project_scope_and_the_remaining_link() {
    let root = super::needs::fixture("A question?", false, 1, false, d::address::Filter::ALL);
    let base = question_card(&root);
    for (count, more) in [(3, None), (4, Some("One more")), (5, Some("Two more")), (13, Some("10 more"))] {
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
                if count == 4 {
                    let position =
                        tree.nodes().iter().position(|n| n.styles.as_slice().contains(&look::HOME_MORE)).unwrap();
                    let nodes = super::inbox::subtree(&tree, position);
                    let text: Vec<_> =
                        nodes.iter().flat_map(|n| n.text.iter().flat_map(|run| run.text.iter().copied())).collect();
                    let text = String::from_utf8(text).unwrap();
                    assert!(text.contains("Hold the suffix? · a question · under 1 min"), "{text}");
                }
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

fn finding_root(count: u32) -> d::Domain {
    finding_root_kind(count, temper_client_protocol_wire::v1::FindingKind::Review)
}
fn finding_root_kind(count: u32, kind: temper_client_protocol_wire::v1::FindingKind) -> d::Domain {
    finding_root_with(count, kind, false)
}
fn finding_root_with(count: u32, kind: temper_client_protocol_wire::v1::FindingKind, split_area: bool) -> d::Domain {
    use jig_client::{Down, Up, Watchable};
    use skein_lib::{List, Queue};
    use temper_client_protocol_wire::{self as wire, v1 as w};
    let limits = f::domain_limits();
    let (mut root, _, _) = f::root_at(d::Address::Settings { project: f::PROJECT }, &limits);
    let mut out = Queue::uncapped();
    d::step(
        &mut root,
        &skein_lib::Env { now: skein_lib::Time::ZERO, wall: f::NOW, limits },
        d::Event::Went { place: d::Place::Address(d::Address::AllHome) },
        &mut out,
    );
    for request in &out {
        if let d::Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down { down: Down::Open { watch, subject: Watchable::Family { record, .. } } },
        } = request
            && let Some(watching) = wire::decode_watching(&record.bytes, &w::CEILINGS)
            && let w::Watch::Findings(subject) = &watching.watch
        {
            let holder = w::Holder::Party(w::HolderParty { number: 1 });
            let rows = (1..=count)
                .map(|number| w::FindingRow {
                    number,
                    at: 0,
                    kind: kind.clone(),
                    severity: w::Severity::High,
                    area: if split_area && number == 2 { Some(b"store".as_slice().into()) } else { None },
                    state: w::FindingState::New,
                    title: b"T".as_slice().into(),
                    holder: holder.clone(),
                    why: Some(b"Your area owns the view".as_slice().into()),
                    hander: Some(w::Filer::Task(w::HolderTask { number: 55 })),
                    goal: None,
                    reason: None,
                    cleared: None,
                })
                .collect::<Vec<_>>();
            let document = w::Findings {
                project: subject.project,
                rows: List::from_boxed(rows.into_boxed_slice()),
                areas: List::with_capacity(0),
                triager: None,
                holder,
                cut: false,
            };
            f::root_event(
                &mut root,
                d::Event::Client {
                    event: jig_client::Event::Up {
                        up: Up::FamilyDelivered {
                            watch: *watch,
                            record: jig_client::Family {
                                family: wire::FORGE,
                                version: wire::CLIENT_VERSION,
                                bytes: wire::findings(&document),
                            },
                        },
                    },
                },
            );
        }
    }
    root
}
fn draw_all(root: &d::Domain, open: &[d::pages::findings::Need<'_>], w: &Words) -> (Tree, Bindings) {
    let mut b = Builder::new(&f::tree_limits());
    let mut bindings = Bindings::new(256);
    b.open(Role::Main);
    crate::pages::home_cards::all(
        &mut b,
        root,
        open,
        true,
        w,
        &mut bindings,
        &mut Times::new(skein_lib::Wall::from_nanos(86_400_000_000_000)),
    );
    b.close();
    (b.finish(), bindings)
}
#[test]
fn finding_rows_cards_and_more_share_the_finding_and_its_form() {
    use d::pages::{
        Act,
        findings::{Act as F, Mode, Need as N},
    };
    let mut root = finding_root(4);
    let cards = root.pages().needs().findings().cards();
    assert!(cards.len() >= 4);
    let id = cards[0].id;
    let open: Vec<_> = cards.iter().take(4).map(|card| N::Finding(card)).collect();
    for w in [f::browser_words(), f::words()] {
        let (tree, bindings) = draw_all(&root, &open, &w);
        let more = tree.nodes().iter().position(|n| n.styles.as_slice().contains(&look::HOME_MORE)).unwrap();
        let more = super::inbox::subtree(&tree, more);
        let text: Vec<_> = more.iter().flat_map(|n| n.text.iter().flat_map(|run| run.text.iter().copied())).collect();
        let expected = if w.shell == crate::words::Shell::Browser {
            "One more T · review · 1 d"
        } else {
            "One more: T · high · review · 1 d"
        };
        assert_eq!(text, expected.as_bytes());
        let glyphs = more.iter().filter(|n| n.styles.as_slice().contains(&look::SEVERITY)).count();
        assert_eq!(glyphs, usize::from(w.shell == crate::words::Shell::Browser));
        for (name, act) in [
            ("Hand on", F::Show { id, mode: Mode::Hand }),
            ("Route to a goal", F::Show { id, mode: Mode::Route }),
            ("Backlog", F::Backlog { id }),
            ("Drop with reason", F::Show { id, mode: Mode::Drop }),
        ] {
            assert_eq!(control(&tree, &bindings, name), d::Action::Pages(Act::Finding(act)));
        }
        if w.shell == crate::words::Shell::Browser {
            assert_eq!(control(&tree, &bindings, "T"), d::Action::Pages(Act::Finding(F::Open { id })));
            assert_eq!(control(&tree, &bindings, "More"), d::Action::Pages(Act::Finding(F::Open { id })));
            assert!(
                tree.nodes()
                    .iter()
                    .any(|n| n.styles.as_slice().contains(&look::FINDING_WHEN) && n.name.as_deref() == Some(b"1 d"))
            );
        }
        assert_eq!(tree.nodes().iter().filter(|n| n.styles.as_slice().contains(&look::FINDING_ROWS)).count(), 3);
        assert!(
            f::find(&tree, Role::Paragraph, "Handed on by task 55: “Your area owns the view”".as_bytes()).is_some()
        );
        let why = tree.nodes().iter().position(|n| n.styles.as_slice().contains(&look::WHY)).unwrap();
        assert!(why > tree.nodes().iter().position(|n| n.styles.as_slice().contains(&look::HOME_MORE)).unwrap());
        assert_eq!(tree.nodes().iter().filter(|n| n.styles.as_slice().contains(&look::WHY)).count(), 1);
        assert_eq!(
            tree.nodes()[why].name.as_deref(),
            Some("These reached you as Shop’s owner: findings that no area owns come to you.".as_bytes())
        );
        assert!(f::find(&tree, Role::Button, b"Stop asking").is_none());
    }
    drop(cards);
    f::root_event(
        &mut root,
        d::Event::Act { action: d::Action::Pages(Act::Finding(F::Show { id, mode: Mode::Drop })) },
    );
    let cards = root.pages().needs().findings().cards();
    let (tree, _) = draw_all(&root, &[N::Finding(cards[0])], &f::browser_words());
    assert!(f::find(&tree, Role::TextField, b"Reason to drop").is_some());
    assert!(
        f::find(
            &tree,
            Role::Paragraph,
            "Reached you as Shop’s owner: findings that no area owns come to you.".as_bytes()
        )
        .is_some()
    );
    drop(cards);
    f::root_event(&mut root, d::Event::Act { action: d::Action::Pages(Act::Finding(F::Open { id })) });
    let cards = root.pages().needs().findings().cards();
    let mut b = Builder::new(&f::tree_limits());
    b.open(Role::Main);
    crate::pages::findings::main(
        &mut b,
        &root,
        cards[0],
        &f::browser_words(),
        &mut Bindings::new(128),
        &mut Times::new(f::NOW),
    );
    b.close();
    let tree = b.finish();
    assert!(f::find(&tree, Role::TextField, b"Reason to drop").is_some());
    let position = |style| tree.nodes().iter().position(|n| n.styles.as_slice().contains(&style)).unwrap();
    assert!(position(look::WHY) < position(look::ASK_CHOICES));
    assert!(position(look::ASK_CONTEXT) < position(look::WHY));
}

#[test]
fn finding_row_kind_and_shell_choose_the_compact_words() {
    let root = finding_root_kind(1, temper_client_protocol_wire::v1::FindingKind::Qa);
    let cards = root.pages().needs().findings().cards();
    for kind in [true, false] {
        for w in [f::browser_words(), f::words()] {
            let mut b = Builder::new(&f::tree_limits());
            let mut bindings = Bindings::new(16);
            b.open(Role::List);
            crate::pages::findings::row(
                &mut b,
                &root,
                cards[0],
                &w,
                &mut bindings,
                &mut Times::new(skein_lib::Wall::from_nanos(86_400_000_000_000)),
                kind,
            );
            b.close();
            let tree = b.finish();
            if w.shell == crate::words::Shell::Browser {
                let when = tree.nodes().iter().find(|n| n.styles.as_slice().contains(&look::FINDING_WHEN)).unwrap();
                assert_eq!(when.name.as_deref(), Some(if kind { "qa · 1 d".as_bytes() } else { b"1 d" }));
                assert_eq!(
                    control(&tree, &bindings, "T"),
                    d::Action::Pages(d::pages::Act::Finding(d::pages::findings::Act::Open { id: cards[0].id }))
                );
            } else {
                let text: Vec<_> =
                    tree.nodes().iter().flat_map(|n| n.text.iter().flat_map(|run| run.text.iter().copied())).collect();
                assert_eq!(text, if kind { "T · high · qa · 1 d".as_bytes() } else { "T · high · 1 d".as_bytes() });
            }
        }
    }
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "shared offers and distinct projects exercise the same reason placement rule"
)]
fn shared_question_reason_uses_the_first_objects_offers_but_not_across_projects() {
    use d::pages::{self, stop_asking::Act as Stop};
    use temper_protocol_reaches::{Class, Classes, Item, Kind, Reach, Reason, Subject};
    let classes = Classes {
        items: Box::new([Item { subject: Subject::Kind(Kind::Findings), words: Box::default(), class: Class::Ask }]),
        sets: Box::default(),
    };
    let (mut root, _, watch) = f::needs::fixture_classes_role(
        "Start goals?",
        false,
        2,
        false,
        d::address::Filter { project: Some(f::PROJECT), ..d::address::Filter::ALL },
        None,
        Some(&classes),
        true,
        1,
    );
    f::needs::deliver_asked(
        &mut root,
        watch,
        Some(Reach {
            kind: Kind::Findings,
            class: Class::Ask,
            reason: Reason::Kind,
            under: 0,
            sets: 0,
            matched: 0,
            away: false,
        }),
    );
    let object = root.pages().inbox().unwrap().cards.get(0).unwrap().object;
    for opened in [false, true] {
        if opened {
            f::root_event(
                &mut root,
                d::Event::Act { action: d::Action::Pages(pages::Act::StopAsking(Stop::Open { object })) },
            );
        }
        let cards = &root.pages().inbox().unwrap().cards;
        let (tree, bindings) = draw(
            &root,
            &[Need::Card(cards.get(0).unwrap()), Need::Card(cards.get(1).unwrap())],
            false,
            &f::browser_words(),
        );
        let whys: Vec<_> = tree.nodes().iter().filter(|n| n.styles.as_slice().contains(&look::WHY)).collect();
        assert_eq!(
            whys.iter()
                .filter(|n| n.name.as_deref()
                    == Some(
                        "These reached you because they start work on findings, which asks you unless you’re away."
                            .as_bytes()
                    ))
                .count(),
            1
        );
        assert!(!tree.nodes().iter().any(|n| n.name.as_deref()
            == Some("Reached you because it starts work on findings, which asks you unless you’re away.".as_bytes())));
        assert_eq!(
            control(&tree, &bindings, "Stop asking"),
            d::Action::Pages(pages::Act::StopAsking(Stop::Open { object }))
        );
        assert_eq!(f::find(&tree, Role::Button, b"Keep asking").is_some(), opened);
    }
    let (mut cross, _, watch) = f::needs::fixture_classes_role(
        "Start goals?",
        false,
        2,
        false,
        d::address::Filter::ALL,
        None,
        Some(&classes),
        true,
        2,
    );
    let reach = Reach {
        kind: Kind::Findings,
        class: Class::Ask,
        reason: Reason::Kind,
        under: 0,
        sets: 0,
        matched: 0,
        away: false,
    };
    f::needs::deliver_asked(&mut cross, watch, Some(reach));
    let question_cards = cross.pages().inbox().unwrap().cards.as_slice().to_vec();
    f::root_event(&mut cross, d::Event::Went { place: d::Place::Address(d::Address::Home { project: f::PROJECT }) });
    let mut opening = skein_lib::Queue::uncapped();
    f::needs::event(&mut cross, d::Event::Went { place: d::Place::Address(d::Address::AllHome) }, &mut opening);
    for request in &opening {
        if let d::Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down: jig_client::Down::Open { watch, subject: jig_client::Watchable::Family { record, .. } },
                },
        } = request
            && let Some(watching) =
                temper_client_protocol_wire::decode_watching(&record.bytes, &temper_client_protocol_wire::v1::CEILINGS)
            && let temper_client_protocol_wire::v1::Watch::Decided(subject) = &watching.watch
            && subject.project == f::SECOND_PROJECT
        {
            f::needs::deliver_asked_for(&mut cross, *watch, f::SECOND_PROJECT, Some(reach));
        }
    }
    let (tree, _) = draw(&cross, &question_cards.iter().map(Need::Card).collect::<Vec<_>>(), true, &f::browser_words());
    let lines: Vec<_> = tree.nodes().iter().filter(|n| n.styles.as_slice().contains(&look::WHY)).collect();
    assert_eq!(lines.len(), 2);
    assert!(lines.iter().all(|n| n.name.as_deref()
        == Some("Reached you because it starts work on findings, which asks you unless you’re away.".as_bytes())));
    let root = finding_root(2);
    let cards = root.pages().needs().findings().cards();
    let first = cards[0];
    let other = cards.iter().find(|c| c.id.project != first.id.project).unwrap();
    let (tree, _) = draw_all(
        &root,
        &[pages::findings::Need::Finding(first), pages::findings::Need::Finding(other)],
        &f::browser_words(),
    );
    let lines: Vec<_> = tree.nodes().iter().filter(|n| n.styles.as_slice().contains(&look::WHY)).collect();
    assert_eq!(lines.len(), 2);
    assert!(lines.iter().all(|n| n.name.as_deref().unwrap().starts_with(b"Reached you as ")));
    let root = finding_root_with(2, temper_client_protocol_wire::v1::FindingKind::Review, true);
    let cards = root.pages().needs().findings().cards();
    let (tree, _) = draw_all(
        &root,
        &[pages::findings::Need::Finding(cards[0]), pages::findings::Need::Finding(cards[1])],
        &f::browser_words(),
    );
    assert_eq!(tree.nodes().iter().filter(|n| n.styles.as_slice().contains(&look::WHY)).count(), 2);
    assert!(tree.nodes().iter().any(|n| n.text.iter().any(|run| run.text.as_ref() == b"in store")));
}

#[test]
fn plural_reason_words_cover_the_domains_reasons() {
    use d::pages::needs::{Holds, Why};
    use temper_protocol_reaches::{Class, Kind, Reach, Reason};
    let classes = temper_protocol_reaches::Classes {
        items: Box::new([temper_protocol_reaches::Item {
            subject: temper_protocol_reaches::Subject::Kind(Kind::Tests),
            words: b"small repairs".as_slice().into(),
            class: Class::Ask,
        }]),
        sets: Box::new([temper_protocol_reaches::PathSet {
            number: 1,
            name: b"protocol core".as_slice().into(),
            patterns: Box::new([b"src/**".as_slice().into()]),
        }]),
    };
    let (root, _, _) = f::needs::fixture_classes_role(
        "Which tests?",
        false,
        1,
        false,
        d::address::Filter { project: Some(f::PROJECT), ..d::address::Filter::ALL },
        None,
        Some(&classes),
        true,
        1,
    );
    let base = Reach {
        kind: Kind::Tests,
        class: Class::Ask,
        reason: Reason::Kind,
        under: 0,
        sets: 0,
        matched: 0,
        away: false,
    };
    let mut cases = Vec::new();
    for (kind, clause) in [
        (Kind::Wording, "they are wording choices"),
        (Kind::Retry, "they retry or split failing tasks"),
        (Kind::Implementation, "they are implementation choices"),
        (Kind::Tests, "they change only tests"),
        (Kind::Dependency, "they add dependencies"),
        (Kind::Departure, "they depart from a landed design"),
        (Kind::Money, "they spend money past a budget"),
        (Kind::Irreversible, "they can’t be undone"),
        (Kind::Rules, "they change what reaches you"),
        (Kind::Other, "no kind of decision names them"),
    ] {
        for (class, words) in [
            (Class::Ask, "asks you unless you’re away"),
            (Class::AskAway, "asks you, even when away"),
            (Class::Wait, "always waits for you"),
        ] {
            cases.push((
                Why::Reach { project: f::PROJECT, reach: Reach { kind, class, ..base } },
                format!("These reached you because {clause}, which {words}."),
            ));
        }
    }
    cases.extend([
        (Why::Reach { project: f::PROJECT, reach: Reach { under: 1, ..base } }, "These reached you because they change only tests under “small repairs”, which asks you unless you’re away.".into()),
        (Why::Reach { project: f::PROJECT, reach: Reach { reason: Reason::Set(1), ..base } }, "These reached you because they change protocol core, which asks you unless you’re away.".into()),
        (Why::Reach { project: f::PROJECT, reach: Reach { reason: Reason::Always, ..base } }, "These reached you because their goals always ask you.".into()),
        (Why::Direct, "These reached you because they were asked of you directly.".into()),
        (Why::Proposal, "These reached you because proposals always wait for whoever may accept them.".into()),
        (Why::Held, "These reached you because held tasks always wait for whoever may release them.".into()),
        (Why::Funds, "These reached you because money past a budget always waits for you.".into()),
        (Why::Finding { project: f::PROJECT, holds: Holds::Area { name: b"store".as_slice().into() } }, "These reached you as store’s owner.".into()),
        (Why::Finding { project: f::PROJECT, holds: Holds::Triager }, "These reached you as Shop’s triager: findings that no area owns come to you.".into()),
        (Why::Finding { project: f::PROJECT, holds: Holds::Owner }, "These reached you as Shop’s owner: findings that no area owns come to you.".into()),
    ]);
    for (why, expected) in cases {
        assert_eq!(
            crate::shared::why_words(&root, &f::browser_words(), &why, true).as_deref(),
            Some(expected.as_bytes()),
            "{why:?}"
        );
    }
    for (holds, expected) in [
        (Holds::Area { name: b"store".as_slice().into() }, "Reached you as store’s owner."),
        (Holds::Triager, "Reached you as Shop’s triager: findings that no area owns come to you."),
        (Holds::Owner, "Reached you as Shop’s owner: findings that no area owns come to you."),
    ] {
        assert_eq!(
            crate::shared::why_words(&root, &f::browser_words(), &Why::Finding { project: f::PROJECT, holds }, false)
                .as_deref(),
            Some(expected.as_bytes())
        );
    }
}
