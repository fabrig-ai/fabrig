//! Browser needs words, roles and acts (temper/docs/design/client/README.md, 3.5).
#![expect(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "ordinary tests inspect semantic subtrees and actions"
)]
use super::{documents, fixtures as f};
use crate::look;
use jig_client::contract as c;
use skein_lib::{Env, List, Queue, Time, Wall};
use skein_ui::{Act, Role};
use temper_client_domain::{
    self as d, Address, Event,
    pages::{self, needs},
};
const QUESTION: &str = "Fix the dropped watch in jig’s store, or keep it alive in temper serve?\n\nMakeProject answers after the client’s watch drops.\n\nSaw: The box test reproduces it every time.\nChoice: In jig’s store (recommended)\nChoice: In temper serve";
fn limits() -> d::Limits {
    let mut limits = f::domain_limits();
    limits.client.domain.reads = 8;
    limits.client.domain.long_text = 4096;
    limits
}
fn object(message: u64) -> c::ObjectKey {
    c::ObjectKey::Question { task: f::INBOX_TASK, message }
}
fn event(root: &mut d::Domain, event: Event, out: &mut Queue<d::Request>) {
    d::step(root, &Env { now: Time::ZERO, wall: f::NOW, limits: limits() }, event, out);
}
fn fixture(words: &str, cut: bool, count: u32, more: bool, filter: d::address::Filter) -> d::Domain {
    fixture_title(words, cut, count, more, filter, None)
}
fn fixture_title(
    words: &str,
    cut: bool,
    count: u32,
    more: bool,
    filter: d::address::Filter,
    title: Option<&[u8]>,
) -> d::Domain {
    fixture_watched(words, cut, count, more, filter, title).0
}
fn fixture_watched(
    words: &str,
    cut: bool,
    count: u32,
    more: bool,
    filter: d::address::Filter,
    title: Option<&[u8]>,
) -> (d::Domain, skein_lib::Token) {
    let (mut root, _, watch) = f::root_at(Address::Inbox { filter }, &limits());
    let mut entries = List::with_capacity(count);
    for message in 1..=u64::from(count) {
        entries.push(documents::entry(object(message), f::PROJECT, message, Wall::EPOCH, None)).unwrap();
    }
    let mut out = Queue::uncapped();
    event(
        &mut root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: watch.unwrap(),
                    event: c::WatchEvent::Delivered(c::Delivery::Inbox(Box::new(documents::inbox(
                        entries.clone(),
                        count,
                        c::Count { value: 0, limited: false },
                        0,
                        more,
                    )))),
                },
            },
        },
        &mut out,
    );
    while let Some(request) = out.pop() {
        if let d::Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down: jig_client::Down::Read { read, what: jig_client::Readable::Jig(c::Read::Inbox { group, .. }) },
                },
        } = &request
        {
            let rows = if *group == c::InboxReadGroup::Needs { entries.clone() } else { List::with_capacity(0) };
            event(
                &mut root,
                Event::Client {
                    event: jig_client::Event::Up {
                        up: jig_client::Up::ReadDone {
                            read: *read,
                            result: jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Inbox(Box::new(
                                c::InboxPage { selection: None, before: None, rows, after: None, cut: more },
                            )))),
                        },
                    },
                },
                &mut out,
            );
        }
        if let d::Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down:
                        jig_client::Down::Read { read, what: jig_client::Readable::Jig(c::Read::Question { message, .. }) },
                },
        } = request
        {
            let mut task = f::task::document().chip;
            task.task = f::INBOX_TASK;
            task.title = f::task::document_words(match message {
                1 => b"welcome-stall",
                2 => b"forge-cards",
                _ => b"third",
            });
            if let Some(title) = title {
                task.title = f::task::document_words(title);
            }
            let q = c::Question {
                task,
                from: None,
                message,
                words: c::DocumentText { bytes: c::Text::new(words.as_bytes().into(), 4096).unwrap(), cut },
                since: Wall::EPOCH,
                acts: {
                    let mut offers = List::with_capacity(1);
                    offers
                        .push(c::Offered { act: c::ActKind::Answer, admission: c::Admission::Allowed, effect: None })
                        .unwrap();
                    offers
                },
                ending: None,
            };
            event(
                &mut root,
                Event::Client {
                    event: jig_client::Event::Up {
                        up: jig_client::Up::ReadDone {
                            read,
                            result: jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Question(Box::new(q)))),
                        },
                    },
                },
                &mut out,
            );
        }
    }
    assert!(root.pages().inbox().unwrap().delivered(root.client()));
    (root, watch.unwrap())
}
fn root() -> d::Domain {
    fixture(QUESTION, false, 3, false, d::address::Filter { project: Some(f::PROJECT), ..d::address::Filter::ALL })
}
fn act(root: &mut d::Domain, act: needs::Act) {
    f::root_event(root, Event::Act { action: d::Action::Pages(pages::Act::Needs(act)) });
}
fn choose(root: &mut d::Domain, message: u64) {
    act(root, needs::Act::Choose { object: object(message), words: b"In the store".as_slice().into() });
}
fn main(view: &crate::View) -> &[skein_ui::Node] {
    let nodes = view.tree().nodes();
    let i = nodes.iter().position(|n| n.role == Role::Main).unwrap();
    &nodes[i..i.checked_add(usize::try_from(nodes[i].size).unwrap()).unwrap()]
}
fn styled(view: &crate::View, style: skein_ui::Style) -> &skein_ui::Node {
    main(view).iter().find(|n| n.styles.as_slice().contains(&style)).expect("styled part")
}
fn press(view: &mut crate::View, name: &[u8]) -> d::Action {
    let node = f::find(view.tree(), Role::Button, name).expect("button");
    crate::act(view, node, Act::Press).unwrap()
}
fn expected(act: needs::Act) -> d::Action {
    d::Action::Pages(pages::Act::Needs(act))
}
fn has(view: &crate::View, words: &[u8]) -> bool {
    skein_lib::bytes::find(&f::main_text(view.tree()), words).is_some()
}
fn home_parts(root: &d::Domain, object: c::ObjectKey) {
    let needs::Need::Card(card) =
        root.pages().needs().shown(root.pages().inbox().unwrap(), root.forge().approvals()).unwrap()
    else {
        panic!("inbox card")
    };
    assert_eq!(crate::pages::ask::card_kind(card), crate::words::Word::HomeQuestion);
    let words = f::browser_words();
    let asked = crate::pages::ask::read(card, &words);
    let mut builder = skein_ui::Builder::new(&f::tree_limits());
    let mut bindings = crate::frame::Bindings::new(128);
    crate::pages::ask::home_body(&mut builder, root, &asked, &words, &mut bindings, 4);
    let home = builder.finish();
    let text = f::text(&home);
    for words in [b"MakeProject answers".as_slice(), b"The box test reproduces"] {
        assert!(skein_lib::bytes::find(&text, words).is_some());
    }
    for name in ["In jig’s store".as_bytes(), b"In temper serve", b"Reply", b"Later"] {
        assert!(f::find(&home, Role::Button, name).is_some());
    }
    let choice = home
        .nodes()
        .iter()
        .find(|node| node.role == Role::Button && node.name.as_deref() == Some("In jig’s store".as_bytes()))
        .unwrap();
    assert_eq!(
        bindings.decode(choice.binding.unwrap().token, Act::Press),
        Some(expected(needs::Act::Choose { object, words: "In jig’s store".as_bytes().into() }))
    );
}
#[test]
fn example_parts_are_in_order_and_choices_keep_their_names_and_acts() {
    for (root, object, title, task) in [
        (Box::new(root()), object(1), b"welcome-stall".as_slice(), f::INBOX_TASK),
        (
            Box::new(word_person(QUESTION, false, false, c::Requester::Task(42))),
            c::ObjectKey::PersonTask(f::INBOX_TASK),
            b"Task 42",
            42,
        ),
        (Box::new(finished_person(true)), c::ObjectKey::PersonTask(f::INBOX_TASK), b"Task 42", 42),
        (
            Box::new(word_person(QUESTION, false, true, c::Requester::Person(73))),
            c::ObjectKey::PersonTask(f::INBOX_TASK),
            b"Review the plan",
            f::INBOX_TASK,
        ),
    ] {
        let needs::Need::Card(card) =
            root.pages().needs().shown(root.pages().inbox().unwrap(), root.forge().approvals()).unwrap()
        else {
            panic!("inbox card")
        };
        if card.whole.is_some() {
            assert_eq!(crate::pages::ask::read(card, &f::words()).short.as_ref(), b"Fix the dropped watch");
        }
        let mut view = f::browser_page_view(&root);
        let styles = [
            look::FLOW_HEAD,
            look::ASK_META,
            look::FLOW_Q,
            look::ASK_CONTEXT,
            look::ASK_SAW,
            look::ASK_CHOICES,
            look::PAIR,
            look::NEXT_UP,
        ];
        let order: Vec<_> =
            styles.iter().map(|s| main(&view).iter().position(|n| n.styles.as_slice().contains(s)).unwrap()).collect();
        assert!(order.windows(2).all(|p| p[0] < p[1]));
        assert_eq!(styled(&view, look::FLOW_Q).role, Role::Heading { level: 1 });
        assert_eq!(
            styled(&view, look::FLOW_Q).name.as_deref(),
            Some("Fix the dropped watch in jig’s store, or keep it alive in temper serve?".as_bytes())
        );
        assert!(
            styled(&view, look::ASK_META)
                .name
                .as_ref()
                .unwrap()
                .ends_with(&skein_lib::bytes::concat(&[title, " · a question · under 1 min".as_bytes()]))
        );
        let title = f::find(view.tree(), Role::Link, title).unwrap();
        assert_eq!(
            crate::act(&mut view, title, Act::Press),
            Some(d::Action::Pages(pages::Act::Go { address: Address::Task { task } }))
        );
        assert_eq!(styled(&view, look::ASK_SAW).role, Role::Disclosure);
        assert_eq!(styled(&view, look::ASK_SAW).name.as_deref(), Some(b"What the agent saw".as_slice()));
        assert!(!styled(&view, look::ASK_SAW).states.as_slice().contains(&skein_ui::State::Expanded));
        assert_eq!(styled(&view, look::REC).role, Role::Text);
        assert_eq!(styled(&view, look::REC).name.as_deref(), Some(b"recommended".as_slice()));
        assert_eq!(styled(&view, look::RECOMMENDED).name.as_deref(), Some("In jig’s store".as_bytes()));
        let choice = styled(&view, look::RECOMMENDED);
        let start = main(&view).iter().position(|n| n.id == choice.id).unwrap();
        let end = start.checked_add(usize::try_from(choice.size).unwrap()).unwrap();
        assert!(
            !main(&view)[start..end]
                .iter()
                .any(|n| n.text.iter().any(|run| run.text.as_ref() == "In jig’s store".as_bytes()))
        );
        assert_eq!(
            press(&mut view, "In jig’s store".as_bytes()),
            expected(needs::Act::Choose { object, words: "In jig’s store".as_bytes().into() })
        );
        assert_eq!(
            press(&mut view, b"In temper serve"),
            expected(needs::Act::Choose { object, words: b"In temper serve".as_slice().into() })
        );
        assert_eq!(press(&mut view, "Reply…".as_bytes()), expected(needs::Act::OpenReply { object }));
        assert_eq!(press(&mut view, b"Later"), expected(needs::Act::Later { object }));
        home_parts(&root, object);
    }
}
#[test]
fn first_choice_is_primary_when_none_recommends_and_only_recommendations_have_rec() {
    for (words, primary, rec) in [
        ("Where?\nChoice: First\nChoice: Second", b"First".as_slice(), false),
        ("Where?\nChoice: First\nChoice: Second (recommended)", b"Second".as_slice(), true),
    ] {
        let root = fixture(words, false, 1, false, d::address::Filter::ALL);
        let view = f::browser_page_view(&root);
        assert_eq!(styled(&view, look::RECOMMENDED).name.as_deref(), Some(primary));
        assert_eq!(main(&view).iter().any(|n| n.styles.as_slice().contains(&look::REC)), rec);
    }
}
#[test]
fn reply_form_focus_placeholder_value_and_all_acts() {
    let mut root = root();
    act(&mut root, needs::Act::OpenReply { object: object(1) });
    f::root_event(
        &mut root,
        Event::Act {
            action: d::Action::Pages(pages::Act::CardWords { object: object(1), words: b"My reply".as_slice().into() }),
        },
    );
    let mut view = f::browser_page_view(&root);
    assert_eq!(styled(&view, look::ASK_REPLY).role, Role::Slot);
    let id = f::find(view.tree(), Role::TextArea, b"Your reply").unwrap();
    let node = view.tree().nodes().iter().find(|n| n.id == id).unwrap();
    assert_eq!(node.placeholder.as_deref(), Some("Your words go to welcome-stall’s agent".as_bytes()));
    assert!(node.focus.is_some());
    assert_eq!(node.value.as_ref().unwrap().text.as_ref(), b"My reply");
    assert_eq!(
        crate::act(&mut view, id, Act::Input { value: b"Changed".as_slice().into() }),
        Some(d::Action::Pages(pages::Act::CardWords { object: object(1), words: b"Changed".as_slice().into() }))
    );
    assert_eq!(press(&mut view, b"Send reply"), expected(needs::Act::SendReply { object: object(1) }));
    assert_eq!(press(&mut view, b"Cancel"), expected(needs::Act::CancelReply));
    act(&mut root, needs::Act::CancelReply);
    assert!(!main(&f::browser_page_view(&root)).iter().any(|n| n.styles.as_slice().contains(&look::ASK_REPLY)));
}
#[test]
fn plain_and_cut_words_have_no_choices_or_saw() {
    for (words, cut) in [
        ("A plain question?\n\nSome context.", false),
        ("A plain question?\nA second line.\n\nSome context.", false),
        (QUESTION, true),
    ] {
        for (root, short) in [
            (Box::new(fixture(words, cut, 1, false, d::address::Filter::ALL)), cut),
            (Box::new(word_person(words, cut, false, c::Requester::Task(42))), false),
            (
                Box::new(word_person_read(
                    words,
                    cut,
                    false,
                    c::Requester::Task(42),
                    Some(c::ReadResult::None(c::ReadNone::Busy)),
                )),
                cut,
            ),
        ] {
            let view = f::browser_page_view(&root);
            assert!(
                !main(&view).iter().any(|n| n.styles.as_slice().contains(&look::ASK_CHOICES)
                    || n.styles.as_slice().contains(&look::ASK_SAW))
            );
            assert!(f::find(view.tree(), Role::Button, "Reply…".as_bytes()).is_some());
            assert!(has(&view, b"Some context.") || has(&view, b"Choice:"));
            let card = root.pages().inbox().unwrap().cards.get(0).unwrap();
            let message = card.group == c::InboxGroup::Questions;
            assert_eq!(
                crate::pages::ask::read(card, &f::words()).short.as_ref(),
                words.lines().next().unwrap().as_bytes()
            );
            assert_eq!(has(&view, b"This question is cut here."), short);
            if short {
                let mut view = view;
                let link = f::find(view.tree(), Role::Link, b"Read the rest").unwrap();
                assert_eq!(
                    crate::act(&mut view, link, Act::Press),
                    Some(d::Action::Pages(pages::Act::Go {
                        address: Address::Task { task: if message { f::INBOX_TASK } else { 42 } }
                    }))
                );
            } else if cut {
                assert!(has(&view, "…".as_bytes()));
            }
        }
    }
}
#[test]
fn progress_counts_answered_first_including_cut_and_exposes_pips() {
    for more in [false, true] {
        let mut root = fixture(QUESTION, false, 3, more, d::address::Filter::ALL);
        let view = f::browser_page_view(&root);
        assert_eq!(styled(&view, look::DOTS).role, Role::Figure);
        assert_eq!(
            styled(&view, look::DOTS).name.as_deref(),
            Some(if more { b"1 of 3+".as_slice() } else { b"1 of 3" })
        );
        choose(&mut root, 1);
        let view = f::browser_page_view(&root);
        assert_eq!(
            styled(&view, look::DOTS).name.as_deref(),
            Some(if more { b"2 of 3+".as_slice() } else { b"2 of 3" })
        );
        let dots: Vec<_> = main(&view).iter().filter(|n| n.styles.as_slice().contains(&look::DOT)).collect();
        assert_eq!(dots.len(), 3);
        assert_eq!(dots[0].role, Role::Text);
        assert!(dots[0].styles.as_slice().contains(&look::DOT_DONE));
        assert!(!dots[1].styles.as_slice().contains(&look::DOT_DONE));
        assert!(!dots[2].styles.as_slice().contains(&look::DOT_DONE));
    }
}
#[test]
fn next_line_last_at_and_later_order() {
    let mut root = root();
    let view = f::browser_page_view(&root);
    assert!(has(&view, "Next: forge-cards · Fix the dropped watch".as_bytes()));
    act(&mut root, needs::Act::Open { object: object(3) });
    assert!(has(&f::browser_page_view(&root), b"third"));
    assert_eq!(
        styled(&f::browser_page_view(&root), look::ASK_META).name.as_deref(),
        Some(b"third \xc2\xb7 a question \xc2\xb7 under 1 min".as_slice())
    );
    act(&mut root, needs::Act::Later { object: object(3) });
    choose(&mut root, 1);
    choose(&mut root, 2);
    let view = f::browser_page_view(&root);
    assert!(has(&view, b"The last one"));
    assert!(has(&view, b"later"));
}
#[test]
fn answered_lines_have_links_labels_and_undo_only_during_hold() {
    for (reply, label) in [(false, "In the store"), (true, "Replied: “My words”")] {
        let mut root = root();
        if reply {
            act(&mut root, needs::Act::OpenReply { object: object(1) });
            f::root_event(
                &mut root,
                Event::Act {
                    action: d::Action::Pages(pages::Act::CardWords {
                        object: object(1),
                        words: b"My words".as_slice().into(),
                    }),
                },
            );
            act(&mut root, needs::Act::SendReply { object: object(1) });
        } else {
            choose(&mut root, 1);
        }
        let mut view = f::browser_page_view(&root);
        assert!(has(&view, b"Answered"));
        assert!(has(&view, b"welcome-stall:"));
        assert!(has(&view, label.as_bytes()));
        let title = f::find(view.tree(), Role::Link, b"welcome-stall").unwrap();
        assert_eq!(
            crate::act(&mut view, title, Act::Press),
            Some(d::Action::Pages(pages::Act::Go { address: Address::Task { task: f::INBOX_TASK } }))
        );
        assert!(main(&view).iter().any(|n| n.role == Role::Button && n.name.as_deref() == Some(b"Undo")));
        assert_eq!(press(&mut view, b"Undo"), expected(needs::Act::Undo { object: object(1) }));
        d::fire(
            &mut root,
            &Env { now: Time::ZERO.saturating_add(needs::HOLD), wall: f::NOW, limits: f::domain_limits() },
            &mut Queue::uncapped(),
        );
        let view = f::browser_page_view(&root);
        assert!(has(&view, label.as_bytes()));
        assert!(f::find(view.tree(), Role::Button, b"Undo").is_none());
    }
}

#[test]
fn done_with_and_without_lines_and_home_project() {
    for answered in [false, true] {
        let mut root = fixture(QUESTION, false, u32::from(answered), false, d::address::Filter::ALL);
        if answered {
            choose(&mut root, 1);
        }
        let mut view = f::browser_page_view(&root);
        assert!(f::find(view.tree(), Role::Heading { level: 1 }, b"Nothing needs you.").is_some());
        assert!(has(&view, b"Every question is answered. Work carries on; Home tells you what changed."));
        assert_eq!(main(&view).iter().any(|n| n.styles.as_slice().contains(&look::ANSWERED)), answered);
        assert!(!main(&view).iter().any(|n| n.styles.as_slice().contains(&look::FLOW_HEAD)));
        let home = main(&view).iter().find(|n| n.role == Role::Link && n.name.as_deref() == Some(b"Home")).unwrap().id;
        assert_eq!(
            crate::act(&mut view, home, Act::Press),
            Some(d::Action::Pages(pages::Act::Go { address: Address::Home { project: f::PROJECT } }))
        );
    }
}
#[test]
fn unfiltered_meta_puts_project_first() {
    let root = fixture(QUESTION, false, 1, false, d::address::Filter::ALL);
    assert_eq!(
        styled(&f::browser_page_view(&root), look::ASK_META).name.as_deref(),
        Some(b"Shop \xc2\xb7 welcome-stall \xc2\xb7 a question \xc2\xb7 under 1 min".as_slice())
    );
}
#[test]
fn empty_column_until_delivered_and_terminal_keeps_its_list() {
    let (pending, _, _) = f::root_at(Address::Inbox { filter: d::address::Filter::ALL }, &f::domain_limits());
    let view = f::browser_page_view(&pending);
    assert_eq!(main(&view).len(), 1);
    let pending = f::inbox_root();
    let view = f::browser_page_view(&pending);
    assert_eq!(main(&view).len(), 1);
    let root = root();
    let view = f::page_view(&root);
    assert!(f::find(view.tree(), Role::Heading { level: 1 }, b"Inbox").is_some());
    assert!(main(&view).iter().any(|n| n.styles.as_slice().contains(&look::FILTERS)));
}
#[test]
fn answered_lines_expire_and_schedule_the_oldest_end() {
    let mut root = root();
    choose(&mut root, 1);
    let mut view = f::browser_page_view(&root);
    let deadline = Wall::from_nanos(needs::ANSWERED_ON_NEEDS.as_nanos());
    crate::build(
        &mut view,
        &root,
        Wall::from_nanos(deadline.as_nanos().checked_sub(1).unwrap()),
        &mut Queue::uncapped(),
    );
    assert!(main(&view).iter().any(|n| n.styles.as_slice().contains(&look::ANSWERED)));
    assert_eq!(view.next_change(), Some(deadline));
    crate::build(&mut view, &root, deadline, &mut Queue::uncapped());
    assert!(!main(&view).iter().any(|n| n.styles.as_slice().contains(&look::ANSWERED)));
}

#[test]
fn refused_answers_reopen_with_the_engines_words_on_the_card() {
    let mut root = root();
    choose(&mut root, 1);
    let env = Env { now: Time::ZERO.saturating_add(needs::HOLD), wall: f::NOW, limits: limits() };
    let mut out = Queue::uncapped();
    d::fire(&mut root, &env, &mut out);
    d::step(&mut root, &env, Event::Client { event: jig_client::Event::Stored }, &mut out);
    let attempt = out
        .iter()
        .find_map(|request| {
            if let d::Request::Unlinked {
                request: jig_client::UnlinkedRequest::Down { down: jig_client::Down::Send { attempt, .. } },
            } = request
            {
                Some(*attempt)
            } else {
                None
            }
        })
        .expect("answer sent");
    d::step(
        &mut root,
        &env,
        Event::Client {
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
        &mut out,
    );
    let view = f::browser_page_view(&root);
    assert!(has(&view, b"welcome-stall"));
    assert!(has(&view, b"not offered"));
    let refused =
        main(&view).iter().position(|n| n.role == Role::Status || n.role == Role::Alert).expect("refusal on main");
    let choices = main(&view).iter().position(|n| n.styles.as_slice().contains(&look::ASK_CHOICES)).unwrap();
    assert!(refused < choices);
}

#[test]
fn unnamed_cards_and_answered_lines_use_task_number() {
    let mut root = fixture_title(QUESTION, false, 1, false, d::address::Filter::ALL, Some(b""));
    let view = f::browser_page_view(&root);
    assert!(has(&view, b"Task 9"));
    assert_eq!(
        styled(&view, look::ASK_META).name.as_deref(),
        Some(b"Shop \xc2\xb7 Task 9 \xc2\xb7 a question \xc2\xb7 under 1 min".as_slice())
    );
    choose(&mut root, 1);
    let view = f::browser_page_view(&root);
    assert!(has(&view, b"Task 9:"));
    assert!(f::find(view.tree(), Role::Link, b"Task 9").is_some());
}

fn words(value: &str) -> c::DocumentText {
    c::DocumentText::complete(c::Text::new(value.as_bytes().into(), 4096).unwrap())
}
fn offers(kinds: &[c::ActKind]) -> List<c::Offered> {
    let mut acts = List::with_capacity(u32::try_from(kinds.len()).unwrap());
    for act in kinds {
        acts.push(c::Offered { act: *act, admission: c::Admission::Allowed, effect: None }).unwrap();
    }
    acts
}
pub(super) fn proposal(proposer: c::Proposer) -> c::Proposal {
    let source = f::chat::proposal_root();
    let mut proposal = source
        .pages()
        .cards()
        .unwrap()
        .iter()
        .find_map(|card| if let Some(c::ReadValue::Proposal(p)) = &card.page { Some(p.as_ref().clone()) } else { None })
        .unwrap();
    proposal.key.proposer = proposer;
    proposal.reason = words("Accept how temper will run your work? The agent follows this plan.");
    proposal.acts = offers(&[c::ActKind::Accept, c::ActKind::Reject]);
    let task = f::task::document();
    let mut tasks = List::with_capacity(1);
    tasks
        .push(c::ProposedTask {
            number: 1,
            executor: task.chip.details.details.executor,
            spec: task.spec,
            authority: task.authority,
            contract: c::TaskContractDocument::Verdict { choices: List::uncapped(), cut: false },
            deadline: c::NewDeadline::Named,
            budget: 6,
            funder: task.budget.funder,
            wake: task.wake,
            dependencies: List::with_capacity(0),
            resources: List::with_capacity(0),
            priority: None,
            cut: false,
        })
        .unwrap();
    proposal.action = c::ProposalAction::Batch(tasks);
    proposal.funding = List::with_capacity(1);
    proposal
        .funding
        .push(c::FundingOffer {
            source: c::Funder::Pool { project: f::PROJECT, person: 73, period: 1 },
            needed: 5,
            left: Some(20),
            after: Some(15),
        })
        .unwrap();
    proposal
}
fn held(reason: Option<c::HoldCategory>) -> c::Escalation {
    let source = f::task::held_root();
    let mut task = source.pages().task().unwrap().document(source.client()).unwrap().clone();
    task.chip.title = words("first-run-projects");
    task.spec.specification.words = words("Both halves of the store task run.");
    task.budget.numbers.spent = 31;
    task.budget.numbers.budget = 40;
    match reason {
        Some(reason) => {
            let hold = task.hold.as_mut().unwrap();
            hold.reason = reason;
            hold.words = Some(words("The Ready page waits for this decision."));
        }
        None => task.hold = None,
    }
    c::Escalation { task: Box::new(task), acts: offers(&[c::ActKind::Release, c::ActKind::LeaveHeld]) }
}
pub(super) fn person() -> c::PersonTask {
    let mut task = f::task::document();
    task.chip.title = words("Review the plan");
    task.spec.specification.words = words("Read the proposed plan and record your review.");
    c::PersonTask {
        task: task.chip,
        revision: c::Revision { value: 3 },
        requester: task.requester,
        spec: task.spec,
        address: c::PersonAddress::Party(73),
        taken_by: None,
        since: None,
        stall: None,
        deadline: None,
        contract: task.result_contract,
        results: List::with_capacity(0),
        cut: false,
        acts: offers(&[c::ActKind::Take, c::ActKind::HandBack]),
    }
}
fn word_person(words: &str, words_cut: bool, document_cut: bool, requester: c::Requester) -> d::Domain {
    word_person_read(words, words_cut, document_cut, requester, None)
}
fn word_person_read(
    words: &str,
    words_cut: bool,
    document_cut: bool,
    requester: c::Requester,
    whole: Option<c::ReadResult>,
) -> d::Domain {
    let mut person = person();
    person.task.task = f::INBOX_TASK;
    person.spec.specification.words =
        c::DocumentText { bytes: c::Text::new(words.as_bytes().into(), 4096).unwrap(), cut: words_cut };
    person.spec.cut = document_cut;
    person.contract = c::TaskContractDocument::Report { words: 4096 };
    person.requester = requester;
    person.since = Some(Wall::EPOCH);
    person.acts = offers(&[c::ActKind::Answer]);
    kinds_read(
        &[(
            c::ObjectKey::PersonTask(person.task.task),
            c::Read::PersonTask { task: person.task.task },
            c::ReadValue::PersonTask(Box::new(person)),
        )],
        whole,
    )
}
fn finished_person(whole: bool) -> d::Domain {
    let result = if whole {
        c::ReadResult::Value(c::ReadValue::TaskText(Box::new(c::TaskText {
            task: f::INBOX_TASK,
            field: c::TaskTextField::Specification,
            from: 0,
            total: u64::try_from(QUESTION.len()).unwrap(),
            next: None,
            text: c::Text::new(QUESTION.as_bytes().into(), 4096).unwrap(),
        })))
    } else {
        c::ReadResult::None(c::ReadNone::Busy)
    };
    word_person_read("Fix the dropped watch", true, false, c::Requester::Task(42), Some(result))
}
fn kinds(values: &[(c::ObjectKey, c::Read, c::ReadValue)]) -> d::Domain {
    kinds_read(values, None)
}
fn kinds_read(values: &[(c::ObjectKey, c::Read, c::ReadValue)], whole: Option<c::ReadResult>) -> d::Domain {
    kinds_owned(values, whole, None)
}
pub(super) fn settings_owned(page: c::ReadValue, owned: Option<bool>) -> d::Domain {
    let c::ReadValue::Proposal(proposal) = &page else { panic!("settings proposal") };
    let key = proposal.key;
    kinds_owned(&[(c::ObjectKey::Proposal(key), c::Read::Proposal { proposal: key }, page)], None, owned)
}
fn kinds_owned(
    values: &[(c::ObjectKey, c::Read, c::ReadValue)],
    whole: Option<c::ReadResult>,
    owned: Option<bool>,
) -> d::Domain {
    let (mut root, _, watch) = f::root_at(Address::Inbox { filter: d::address::Filter::ALL }, &limits());
    let mut entries = List::with_capacity(u32::try_from(values.len()).unwrap());
    for (i, (object, _, _)) in values.iter().enumerate() {
        entries.push(documents::entry(*object, f::PROJECT, u64::try_from(i).unwrap(), Wall::EPOCH, None)).unwrap();
    }
    let mut out = Queue::uncapped();
    event(
        &mut root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: watch.unwrap(),
                    event: c::WatchEvent::Delivered(c::Delivery::Inbox(Box::new(documents::inbox(
                        entries.clone(),
                        u32::try_from(values.len()).unwrap(),
                        c::Count { value: 0, limited: false },
                        0,
                        false,
                    )))),
                },
            },
        },
        &mut out,
    );
    while let Some(request) = out.pop() {
        if let d::Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down: jig_client::Down::Read { read, what: jig_client::Readable::Jig(what) },
                },
        } = request
        {
            let page = if let c::Read::Inbox { group, .. } = &what {
                Some(c::ReadValue::Inbox(Box::new(c::InboxPage {
                    selection: None,
                    before: None,
                    rows: if *group == c::InboxReadGroup::Needs { entries.clone() } else { List::with_capacity(0) },
                    after: None,
                    cut: false,
                })))
            } else {
                values.iter().find(|(_, wanted, _)| *wanted == what).map(|(_, _, page)| page.clone())
            };
            let result =
                if let c::Read::TaskText { .. } = what { whole.clone() } else { page.map(c::ReadResult::Value) };
            if let Some(result) = result {
                event(
                    &mut root,
                    Event::Client {
                        event: jig_client::Event::Up {
                            up: jig_client::Up::ReadDone { read, result: jig_client::Readout::Jig(result) },
                        },
                    },
                    &mut out,
                );
            }
        } else if let d::Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down: jig_client::Down::Read { read, what: jig_client::Readable::Family { record, .. } },
                },
        } = request
            && let Some(owned_approval) = owned
            && let Some(reading) =
                temper_client_protocol_wire::decode_reading(&record.bytes, &temper_client_protocol_wire::v1::CEILINGS)
            && let temper_client_protocol_wire::v1::Read::ProposalItems(_) = reading.read()
        {
            use temper_client_protocol_wire::{self as wire, v1 as w};
            let answer = w::ProposalItemsAnswer::new(
                &w::CEILINGS,
                w::ProposalItemsAnswerParts { items: List::with_capacity(0), owned_approval },
            )
            .unwrap();
            event(
                &mut root,
                Event::Client {
                    event: jig_client::Event::Up {
                        up: jig_client::Up::ReadDone {
                            read,
                            result: jig_client::Readout::Family {
                                family: wire::FORGE,
                                record: Some(wire::proposal_items_answer(&answer)),
                            },
                        },
                    },
                },
                &mut out,
            );
        }
    }
    root
}
fn proposal_row(p: c::Proposal) -> (c::ObjectKey, c::Read, c::ReadValue) {
    (c::ObjectKey::Proposal(p.key), c::Read::Proposal { proposal: p.key }, c::ReadValue::Proposal(Box::new(p)))
}
fn held_row(h: c::Escalation) -> (c::ObjectKey, c::Read, c::ReadValue) {
    (
        c::ObjectKey::Escalation(h.task.chip.task),
        c::Read::Escalation { task: h.task.chip.task },
        c::ReadValue::Escalation(Box::new(h)),
    )
}
fn ordered(view: &crate::View) {
    let positions: Vec<_> =
        [look::ASK_META, look::FLOW_Q, look::ASK_CONTEXT, look::ASK_CHOICES, look::PAIR, look::NEXT_UP]
            .iter()
            .map(|style| main(view).iter().position(|node| node.styles.as_slice().contains(style)).unwrap())
            .collect();
    assert!(positions.windows(2).all(|pair| pair[0] < pair[1]));
}
#[test]
fn proposal_parts_accept_reject_and_task_open() {
    for proposer in [c::Proposer::Task(f::INBOX_TASK), c::Proposer::Party(73)] {
        let p = proposal(proposer);
        let object = c::ObjectKey::Proposal(p.key);
        let root = kinds(&[proposal_row(p)]);
        let mut view = f::browser_page_view(&root);
        ordered(&view);
        assert_eq!(
            styled(&view, look::FLOW_Q).name.as_deref(),
            Some(b"Accept how temper will run your work? The agent follows this plan.".as_slice())
        );
        assert!(has(&view, b"Funding"));
        assert_eq!(styled(&view, look::RECOMMENDED).name.as_deref(), Some(b"Accept".as_slice()));
        assert!(!main(&view).iter().any(|n| n.styles.as_slice().contains(&look::REC)));
        for (name, act) in [(b"Accept".as_slice(), c::ActKind::Accept), (b"Reject", c::ActKind::Reject)] {
            assert_eq!(press(&mut view, name), d::Action::Pages(pages::Act::Decide { object, revision: None, act }));
        }
        let reject = main(&view)
            .iter()
            .position(|n| n.role == Role::Disclosure && n.name.as_deref() == Some(b"Reject with reason"))
            .unwrap();
        assert!(
            super::inbox::subtree(view.tree(), super::inbox::styled(view.tree(), look::ASK_CHOICES))
                .iter()
                .any(|n| n.role == Role::TextArea)
        );
        assert!(main(&view)[reject..].iter().any(|n| n.role == Role::TextArea && n.name.as_deref() == Some(b"Reason")));
        assert_eq!(
            f::find(view.tree(), Role::Button, b"Open").is_some(),
            match proposer {
                c::Proposer::Task(_) => true,
                c::Proposer::Party(_) => false,
            }
        );
        assert_eq!(press(&mut view, b"Later"), expected(needs::Act::Later { object }));
    }
}
#[test]
fn proposal_short_stops_at_sentence_or_first_line_and_batch_uses_heading() {
    for (reason, expected) in
        [("First. Second.", "First."), ("First line\nSecond. More", "First line"), ("Whole reason", "Whole reason")]
    {
        let mut p = proposal(c::Proposer::Task(9));
        p.reason = words(reason);
        let root = kinds(&[proposal_row(p)]);
        let needs::Need::Card(card) =
            root.pages().needs().shown(root.pages().inbox().unwrap(), root.forge().approvals()).unwrap()
        else {
            panic!("inbox card")
        };
        assert_eq!(crate::pages::ask::read(card, &f::words()).short.as_ref(), expected.as_bytes());
    }
    let mut p = proposal(c::Proposer::Task(9));
    p.reason = words("");
    let heading = match &mut p.action {
        c::ProposalAction::Batch(tasks) => {
            tasks.get_mut(0).unwrap().spec.specification.words =
                words("Run the work. Keep its checks\nIts full specification.");
            "Run the work. Keep its checks"
        }
        c::ProposalAction::Settings(_)
        | c::ProposalAction::Effect { .. }
        | c::ProposalAction::ProcedureEffect { .. }
        | c::ProposalAction::Amend { .. }
        | c::ProposalAction::Widen { .. }
        | c::ProposalAction::Release { .. } => panic!("batch fixture"),
    };
    assert_eq!(
        styled(&f::browser_page_view(&kinds(&[proposal_row(p)])), look::FLOW_Q).name.as_deref(),
        Some(heading.as_bytes())
    );
}
#[test]
fn a_proposal_s_question_line_leaves_off_a_heading_s_marks() {
    for (reason, expected) in [
        ("## temper enforces\nNo direct pushes.", Some("temper enforces")),
        ("##\nNo direct pushes. More.", Some("No direct pushes.")),
        ("##\n###", None),
        ("  \n## temper enforces", Some("temper enforces")),
        ("#hashtag stays", Some("#hashtag stays")),
    ] {
        let mut p = proposal(c::Proposer::Task(9));
        p.reason = words(reason);
        let root = kinds(&[proposal_row(p)]);
        let needs::Need::Card(card) =
            root.pages().needs().shown(root.pages().inbox().unwrap(), root.forge().approvals()).unwrap()
        else {
            panic!("inbox card")
        };
        let asked = crate::pages::ask::read(card, &f::words());
        assert_eq!(asked.short.as_ref(), expected.map_or(asked.title.as_ref(), str::as_bytes), "{reason}");
        assert_eq!(styled(&f::browser_page_view(&root), look::FLOW_Q).name.as_deref(), Some(asked.short.as_ref()));
    }
}
#[test]
fn held_parts_release_leave_held_and_missing_hold() {
    for reason in [Some(c::HoldCategory::Stalled), None] {
        let h = held(reason);
        let object = c::ObjectKey::Escalation(h.task.chip.task);
        let revision = h.task.hold.as_ref().map(|hold| hold.revision());
        let root = kinds(&[held_row(h)]);
        let mut view = f::browser_page_view(&root);
        ordered(&view);
        assert_eq!(styled(&view, look::FLOW_Q).name.as_deref(), Some(b"first-run-projects".as_slice()));
        assert!(has(&view, b"Both halves of the store task run."));
        assert_eq!(has(&view, b"The Ready page waits"), reason.is_some());
        assert!(skein_lib::bytes::find(styled(&view, look::ASK_META).name.as_ref().unwrap(), b"held").is_some());
        assert_eq!(styled(&view, look::RECOMMENDED).name.as_deref(), Some(b"Release".as_slice()));
        assert!(f::find(view.tree(), Role::Disclosure, b"Leave held").is_some());
        for (name, act) in [(b"Release".as_slice(), c::ActKind::Release), (b"Leave held", c::ActKind::LeaveHeld)] {
            assert_eq!(press(&mut view, name), d::Action::Pages(pages::Act::Decide { object, revision, act }));
        }
        assert_eq!(
            press(&mut view, b"Open"),
            d::Action::Pages(pages::Act::Go { address: Address::Task { task: f::INBOX_TASK } })
        );
    }
}
#[test]
fn funds_use_budget_reason_plain_numbers_and_held_acts() {
    for reason in
        [c::HoldCategory::Budget, c::HoldCategory::Exceeded(c::Exceeded { bound: c::Bound::Reasoning, value: 2048 })]
    {
        let root = kinds(&[held_row(held(Some(reason)))]);
        let view = f::browser_page_view(&root);
        let funds = reason == c::HoldCategory::Budget;
        assert_eq!(
            styled(&view, look::FLOW_Q).name.as_deref(),
            Some(if funds { b"31 of 40 used.".as_slice() } else { b"first-run-projects" })
        );
        assert_eq!(
            skein_lib::bytes::find(styled(&view, look::ASK_META).name.as_ref().unwrap(), b"funds").is_some(),
            funds
        );
        assert!(f::find(view.tree(), Role::Button, b"Release").is_some());
        assert!(f::find(view.tree(), Role::Button, b"Leave held").is_some());
        assert!(!has(&view, b"$"));
        assert!(f::find(view.tree(), Role::Button, b"Add $10").is_none());
    }
}
#[test]
fn person_spec_and_plain_acts() {
    let person = person();
    let object = c::ObjectKey::PersonTask(person.task.task);
    let root =
        kinds(&[(object, c::Read::PersonTask { task: person.task.task }, c::ReadValue::PersonTask(Box::new(person)))]);
    let mut view = f::browser_page_view(&root);
    ordered(&view);
    assert_eq!(styled(&view, look::FLOW_Q).name.as_deref(), Some(b"Review the plan".as_slice()));
    assert!(has(&view, b"Read the proposed plan and record your review."));
    assert!(!main(&view).iter().any(|n| n.styles.as_slice().contains(&look::RECOMMENDED)));
    for (name, act) in [(b"Take".as_slice(), c::ActKind::Take), (b"Hand back", c::ActKind::HandBack)] {
        assert_eq!(
            press(&mut view, name),
            d::Action::Pages(pages::Act::Decide { object, revision: Some(c::Revision { value: 3 }), act })
        );
    }
    assert!(f::find(view.tree(), Role::Button, b"Open").is_some());
    assert_eq!(press(&mut view, b"Later"), expected(needs::Act::Later { object }));
}
#[test]
fn mixed_kinds_next_line_uses_each_short() {
    let p = proposal(c::Proposer::Task(9));
    let mut p = p;
    p.reason = words("Accept the plan. More context.");
    let mut h = held(Some(c::HoldCategory::Stalled));
    h.task.chip.task = 10;
    let mut funds = held(Some(c::HoldCategory::Budget));
    funds.task.chip.task = 11;
    let mut person = person();
    person.task.task = 12;
    let rows = [
        proposal_row(p),
        held_row(h),
        held_row(funds),
        (c::ObjectKey::PersonTask(12), c::Read::PersonTask { task: 12 }, c::ReadValue::PersonTask(Box::new(person))),
    ];
    let mut root = kinds(&rows);
    let expectations = [
        ("Accept the plan.", "Next: first-run-projects · first-run-projects"),
        ("first-run-projects", "Next: first-run-projects · 31 of 40 used"),
        ("31 of 40 used", "Next: Review the plan · Review the plan"),
        ("Review the plan", "Next: Task 9 · Accept the plan."),
    ];
    for (row, (short, next)) in rows.iter().zip(expectations) {
        let needs::Need::Card(card) =
            root.pages().needs().shown(root.pages().inbox().unwrap(), root.forge().approvals()).unwrap()
        else {
            panic!("inbox card")
        };
        assert_eq!(crate::pages::ask::read(card, &f::words()).short.as_ref(), short.as_bytes());
        let view = f::browser_page_view(&root);
        assert!(has(&view, next.as_bytes()));
        act(&mut root, needs::Act::Later { object: row.0 });
    }
}

fn page_act(root: &mut d::Domain, action: pages::Act, out: &mut Queue<d::Request>) {
    event(root, Event::Act { action: d::Action::Pages(action) }, out);
}
#[test]
fn deciding_reasons_keep_their_value_and_confirmation_over_the_page() {
    let p = proposal_row(proposal(c::Proposer::Task(9)));
    let h = held_row(held(Some(c::HoldCategory::Stalled)));
    for (row, name, act_kind, dialog) in [
        (&p, b"Reject".as_slice(), c::ActKind::Reject, Some(b"Reject this proposal?".as_slice())),
        (&p, b"Accept", c::ActKind::Accept, Some(b"Accept this proposal?".as_slice())),
        (&h, b"Leave held", c::ActKind::LeaveHeld, None),
        (&h, b"Release", c::ActKind::Release, Some(b"Release this task?".as_slice())),
    ] {
        let mut root = kinds(core::slice::from_ref(row));
        let mut view = f::browser_page_view(&root);
        let mut out = Queue::uncapped();
        if act_kind == c::ActKind::Reject || act_kind == c::ActKind::LeaveHeld {
            let textarea = f::find(view.tree(), Role::TextArea, b"Reason").unwrap();
            let input =
                crate::act(&mut view, textarea, Act::Input { value: b"Keep the current plan".as_slice().into() })
                    .unwrap();
            event(&mut root, Event::Act { action: input }, &mut out);
            let view = f::browser_page_view(&root);
            let reason = main(&view).iter().find(|n| n.role == Role::TextArea).unwrap();
            assert_eq!(reason.value.as_ref().unwrap().text.as_ref(), b"Keep the current plan");
        }
        let action = press(&mut view, name);
        event(&mut root, Event::Act { action }, &mut out);
        let view = f::browser_page_view(&root);
        assert!(
            f::find(view.tree(), Role::Heading { level: 1 }, styled(&view, look::FLOW_Q).name.as_ref().unwrap())
                .is_some()
        );
        if let Some(dialog) = dialog {
            assert!(f::find(view.tree(), Role::Dialog, dialog).is_some());
            assert!(root.pages().inbox().unwrap().cards.get(0).unwrap().key.is_none());
            page_act(&mut root, pages::Act::Confirm, &mut out);
        }
        let card = root.pages().inbox().unwrap().cards.get(0).unwrap();
        assert!(card.key.is_some());
        assert!(!root.pages().needs().held(row.0));
        assert!(root.pages().needs().answered(f::NOW, needs::ANSWERED_ON_NEEDS, None).is_empty());
    }
}
#[test]
fn sent_cards_replace_all_acts_with_progress_and_the_returned_outcome() {
    let p = proposal_row(proposal(c::Proposer::Task(9)));
    let mut root = kinds(core::slice::from_ref(&p));
    let mut view = f::browser_page_view(&root);
    let action = press(&mut view, b"Accept");
    let mut out = Queue::uncapped();
    event(&mut root, Event::Act { action }, &mut out);
    page_act(&mut root, pages::Act::Confirm, &mut out);
    page_act(&mut root, pages::Act::Cancel, &mut out);
    let view = f::browser_page_view(&root);
    for name in [b"Accept".as_slice(), b"Reject", b"Open", b"Later", b"Undo"] {
        assert!(f::find(view.tree(), Role::Button, name).is_none());
    }
    event(&mut root, Event::Client { event: jig_client::Event::Stored }, &mut out);
    let attempt = out
        .iter()
        .find_map(|request| {
            if let d::Request::Unlinked {
                request: jig_client::UnlinkedRequest::Down { down: jig_client::Down::Send { attempt, .. } },
            } = request
            {
                Some(*attempt)
            } else {
                None
            }
        })
        .expect("decision sends without the question hold");
    let view = f::browser_page_view(&root);
    assert!(main(&view).iter().any(|n| n.role == Role::Status));
    event(
        &mut root,
        Event::Client {
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
        &mut out,
    );
    let view = f::browser_page_view(&root);
    assert!(has(&view, b"This act is not offered"));
    for name in [b"Accept".as_slice(), b"Reject", b"Open", b"Later", b"Undo"] {
        assert!(f::find(view.tree(), Role::Button, name).is_none());
    }
    assert!(!main(&view).iter().any(|n| n.styles.as_slice().contains(&look::ANSWERED)));
    assert!(root.pages().needs().answered(f::NOW, needs::ANSWERED_ON_NEEDS, None).is_empty());
}

#[test]
fn a_refresh_of_the_same_rows_leaves_the_tree_unchanged() {
    for filter in [d::address::Filter::ALL, d::address::Filter { project: Some(f::PROJECT), ..d::address::Filter::ALL }]
    {
        let (mut root, watch) = fixture_watched(QUESTION, false, 1, false, filter, None);
        let mut view = f::browser_page_view(&root);
        let before = stable_nodes(view.tree());
        let choice = f::find(view.tree(), Role::Button, b"In temper serve").unwrap();
        let action =
            Some(expected(needs::Act::Choose { object: object(1), words: b"In temper serve".as_slice().into() }));
        let mut entries = List::with_capacity(1);
        entries.push(documents::entry(object(1), f::PROJECT, 1, Wall::EPOCH, None)).unwrap();
        event(
            &mut root,
            Event::Client {
                event: jig_client::Event::Up {
                    up: jig_client::Up::Watched {
                        watch,
                        event: c::WatchEvent::Delivered(c::Delivery::Inbox(Box::new(documents::inbox(
                            entries,
                            1,
                            c::Count { value: 0, limited: false },
                            0,
                            false,
                        )))),
                    },
                },
            },
            &mut Queue::uncapped(),
        );
        let mut patches = Queue::uncapped();
        crate::build(&mut view, &root, f::NOW, &mut patches);
        assert!(stable_nodes(view.tree()) == before, "same elements and content during refresh at {filter:?}");
        assert_eq!(f::find(view.tree(), Role::Button, b"In temper serve"), Some(choice));
        assert_eq!(crate::act(&mut view, choice, Act::Press), action);
    }
}

fn stable_nodes(tree: &skein_ui::Tree) -> Vec<skein_ui::Node> {
    let mut nodes = tree.nodes().to_vec();
    // Binding tokens and input write revisions are renewed on every build.
    for node in &mut nodes {
        for binding in [&mut node.binding, &mut node.link].into_iter().flatten() {
            binding.token = skein_lib::Token::new(0);
        }
        if let Some(value) = &mut node.value {
            value.written = 0;
        }
    }
    nodes
}

fn empty_column(root: &d::Domain) {
    let view = f::browser_page_view(root);
    assert!(f::find(view.tree(), Role::Heading { level: 1 }, b"Nothing needs you.").is_none());
    let index = super::inbox::styled(view.tree(), look::NEEDS);
    assert_eq!(view.tree().nodes()[index].size, 1, "unanswered column has no content");
}
#[test]
fn behind_with_unanswered_reads_needs_claims_no_empty() {
    let mut page = f::ReadPage::new(Address::Inbox { filter: d::address::Filter::ALL });
    page.drop_inbox();
    assert!(f::find(f::browser_page_view(&page.root).tree(), Role::Status, b"Behind").is_some());
    empty_column(&page.root);
}
#[test]
fn lost_section_reads_are_no_answer() {
    let mut page = f::ReadPage::new(Address::Inbox {
        filter: d::address::Filter { project: Some(f::PROJECT), ..d::address::Filter::ALL },
    });
    page.refresh_inbox();
    page.sections(true);
    empty_column(&page.root);
}
#[test]
fn a_known_empty_stays_through_a_refresh() {
    let mut page = f::ReadPage::new(Address::Inbox {
        filter: d::address::Filter { project: Some(f::PROJECT), ..d::address::Filter::ALL },
    });
    page.refresh_inbox();
    page.sections(false);
    page.refresh_inbox();
    assert!(!page.root.pages().inbox().unwrap().delivered(page.root.client()));
    assert!(
        f::find(f::browser_page_view(&page.root).tree(), Role::Heading { level: 1 }, b"Nothing needs you.").is_some()
    );
}

pub(super) fn approval_row() -> d::forge::approvals::Row {
    d::forge::approvals::Row {
        task: 81,
        project: f::PROJECT,
        goal: None,
        forge: 3,
        kind: d::forge::forges::Kind::Forgejo,
        owner: b"work".as_slice().into(),
        name: b"hello".as_slice().into(),
        pull: 24,
        address: b"https://forge.example/work/hello/pulls/24/files".as_slice().into(),
        title: b"A change".as_slice().into(),
        since: Wall::EPOCH,
        scope: f::PROJECT,
        rule: 1,
        role: 1,
        required: 1,
        counted: 0,
        behind: 0,
        whole: true,
        yours: d::forge::approvals::Yours::Counts,
    }
}
pub(super) fn approvals(root: &mut d::Domain, rows: &[d::forge::approvals::Row], more: u32) {
    f::root_event(
        root,
        Event::Forge(d::forge::Event::Approvals {
            event: d::forge::approvals::Event::Rows { rows: rows.into(), more },
        }),
    );
}
#[test]
#[expect(
    clippy::too_many_lines,
    reason = "the approval table holds the rule, eligibility, address and ordered card parts together"
)]
fn approval_parts_say_the_rule_and_offer_the_review_where_it_counts() {
    use d::forge::{approvals::Yours, forges::Kind};
    let rule = "This project's landing rule 1 asks for one approval by a maintainer.";
    for (kind, scope, rule_number, role, required, counted, behind, whole, yours, address, expected) in [
        (Kind::Forgejo, 1, 1, 1, 1, 0, 0, true, Yours::Counts, "https://forge.example/files", [rule].as_slice()),
        (
            Kind::GitHub,
            0,
            2,
            0,
            2,
            1,
            3,
            true,
            Yours::Counts,
            "https://github.com/work/hello/pull/24/files",
            &[
                "The deployment's landing rule 2 asks for 2 approvals by owners; 1 so far.",
                "3 more changes wait behind this one.",
            ],
        ),
        (
            Kind::Forgejo,
            1,
            1,
            1,
            1,
            0,
            1,
            false,
            Yours::Counts,
            "https://forge.example/files",
            &[
                rule,
                "1 more change waits behind this one.",
                "There are too many reviews to read them all, so one may already count.",
            ],
        ),
        (
            Kind::Forgejo,
            1,
            1,
            1,
            1,
            0,
            0,
            true,
            Yours::NoAccount,
            "https://forge.example/files",
            &[rule, "temper knows no Forgejo account of yours, so your review wouldn't count."],
        ),
        (
            Kind::GitHub,
            1,
            1,
            1,
            1,
            0,
            0,
            true,
            Yours::NoAccount,
            "https://github.com/files",
            &[rule, "temper knows no GitHub account of yours, so your review wouldn't count."],
        ),
        (
            Kind::Forgejo,
            1,
            1,
            1,
            1,
            0,
            0,
            true,
            Yours::TooFew,
            "https://forge.example/files",
            &[
                rule,
                "Fewer people can approve it than the rule asks for, so it waits until the rule or the project's people change.",
            ],
        ),
        (
            Kind::Forgejo,
            1,
            1,
            1,
            1,
            0,
            0,
            true,
            Yours::OtherRole,
            "https://forge.example/files",
            &[rule, "It waits for a review by a maintainer; yours wouldn't count."],
        ),
        (
            Kind::Forgejo,
            1,
            1,
            2,
            1,
            0,
            0,
            true,
            Yours::Counts,
            "",
            &["This project's landing rule 1 asks for one approval by a member."],
        ),
        (
            Kind::Forgejo,
            1,
            1,
            3,
            2,
            0,
            0,
            true,
            Yours::Counts,
            "javascript:alert(1)",
            &["This project's landing rule 1 asks for 2 approvals by observers; 0 so far."],
        ),
        (
            Kind::Forgejo,
            1,
            1,
            8,
            1,
            0,
            0,
            true,
            Yours::OtherRole,
            "https://forge.example/files",
            &[
                "This project's landing rule 1 asks for one approval by someone with role 8.",
                "It waits for a review by someone with role 8; yours wouldn't count.",
            ],
        ),
        (
            Kind::Forgejo,
            1,
            1,
            8,
            2,
            1,
            0,
            true,
            Yours::Counts,
            "https://forge.example/files",
            &["This project's landing rule 1 asks for 2 approvals by people with role 8; 1 so far."],
        ),
    ] {
        let row = d::forge::approvals::Row {
            kind,
            scope,
            rule: rule_number,
            role,
            required,
            counted,
            behind,
            whole,
            yours,
            address: address.as_bytes().into(),
            ..approval_row()
        };
        let mut root = kinds(&[]);
        approvals(&mut root, core::slice::from_ref(&row), 0);
        let mut view = f::browser_page_view(&root);
        let (question, review) = match kind {
            Kind::Forgejo => ("Approve A change on Forgejo", "Review on Forgejo ↗"),
            Kind::GitHub => ("Approve A change on GitHub", "Review on GitHub ↗"),
        };
        let heading = f::find(view.tree(), Role::Heading { level: 1 }, question.as_bytes()).unwrap();
        assert!(view.tree().find(heading).unwrap().styles.as_slice().contains(&look::FLOW_Q));
        let context: Vec<_> = main(&view)
            .iter()
            .filter(|node| node.role == Role::Paragraph && node.styles.as_slice().contains(&look::ASK_CONTEXT))
            .map(|node| node.name.as_deref().unwrap())
            .collect();
        assert_eq!(context, expected.iter().map(|line| line.as_bytes()).collect::<Vec<_>>());
        let primary = f::find(view.tree(), Role::Link, review.as_bytes());
        if yours == Yours::Counts && skein_ui::markdown::safe_url(address.as_bytes()) {
            let node = view.tree().find(primary.expect("eligible safe review link")).unwrap();
            assert!(node.styles.as_slice().contains(&look::RECOMMENDED));
            assert_eq!(node.external.as_deref(), Some(address.as_bytes()));
        } else {
            assert!(primary.is_none(), "{yours:?}: no review that cannot count");
        }
        assert_eq!(
            press(&mut view, b"Open"),
            d::Action::Pages(pages::Act::Go { address: Address::Task { task: row.task } })
        );
        assert_eq!(
            press(&mut view, b"Later"),
            d::Action::Pages(pages::Act::Needs(needs::Act::Later { object: c::ObjectKey::Task(row.task) }))
        );
        assert_eq!(styled(&view, look::FLOW_HEAD).role, Role::Slot);
        assert!(has(&view, b"1 of 1"));
        assert!(has(&view, b"The last one"));
    }
    let mut root = fixture(QUESTION, false, 1, false, d::address::Filter::ALL);
    let row = approval_row();
    approvals(&mut root, core::slice::from_ref(&row), 1);
    let mut view = f::browser_page_view(&root);
    assert!(has(&view, b"1 of 2+"));
    assert_eq!(
        styled(&view, look::NEXT_UP).name.as_deref(),
        Some("Next: A change · Approve A change on Forgejo".as_bytes())
    );
    act(&mut root, needs::Act::Later { object: object(1) });
    f::build(&mut view, &root);
    assert!(has(&view, b"Approve A change on Forgejo"));
    act(&mut root, needs::Act::Later { object: c::ObjectKey::Task(row.task) });
    assert_eq!(root.pages().needs().needs(root.pages().inbox().unwrap(), root.forge().approvals()).len(), 2);
    let open = root.pages().needs().needs(root.pages().inbox().unwrap(), root.forge().approvals());
    assert_eq!(open.first().unwrap().object(), object(1));
    assert_eq!(open.last().unwrap().object(), c::ObjectKey::Task(row.task));
    // A fresh question comes before the approval marked later, without changing its count.
    choose(&mut root, 1);
    f::build(&mut view, &root);
    assert!(has(&view, b"an approval"));
    assert!(has(&view, b"later"));
    let terminal = f::page_view(&root);
    assert!(!has(&terminal, b"an approval"));
    let mut sign_in = crate::forge_sign_in::Forges::new();
    sign_in.read("3 forgejo https://sign-in.example/root owned");
    let mut view = f::view_of(f::browser_words());
    view.forge_sign_in(sign_in);
    f::build(&mut view, &root);
    let id = f::find(view.tree(), Role::Link, "Review on Forgejo ↗".as_bytes()).unwrap();
    assert_eq!(
        view.tree().external(id),
        Some(
            b"https://sign-in.example/root/user/oauth2/temper?redirect_to=%2Fwork%2Fhello%2Fpulls%2F24%2Ffiles"
                .as_slice()
        )
    );
}
