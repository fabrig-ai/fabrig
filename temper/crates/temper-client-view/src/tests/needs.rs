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
    root
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
#[test]
fn example_parts_are_in_order_and_choices_keep_their_names_and_acts() {
    let mut view = f::browser_page_view(&root());
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
    assert_eq!(
        styled(&view, look::ASK_META).name.as_deref(),
        Some(b"welcome-stall \xc2\xb7 a question \xc2\xb7 under 1 min".as_slice())
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
        expected(needs::Act::Choose { object: object(1), words: "In jig’s store".as_bytes().into() })
    );
    assert_eq!(
        press(&mut view, b"In temper serve"),
        expected(needs::Act::Choose { object: object(1), words: b"In temper serve".as_slice().into() })
    );
    assert_eq!(press(&mut view, "Reply…".as_bytes()), expected(needs::Act::OpenReply { object: object(1) }));
    assert_eq!(press(&mut view, b"Later"), expected(needs::Act::Later { object: object(1) }));
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
    for (words, cut) in [("A plain question?\n\nSome context.", false), (QUESTION, true)] {
        let root = fixture(words, cut, 1, false, d::address::Filter::ALL);
        let view = f::browser_page_view(&root);
        assert!(
            !main(&view)
                .iter()
                .any(|n| n.styles.as_slice().contains(&look::ASK_CHOICES)
                    || n.styles.as_slice().contains(&look::ASK_SAW))
        );
        assert!(f::find(view.tree(), Role::Button, "Reply…".as_bytes()).is_some());
        assert!(has(&view, b"Some context.") || has(&view, b"Choice:"));
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
    let mut root = root();
    choose(&mut root, 1);
    let mut view = f::browser_page_view(&root);
    assert!(has(&view, b"Answered"));
    assert!(has(&view, b"welcome-stall:"));
    assert!(has(&view, b"In the store"));
    assert!(main(&view).iter().any(|n| n.role == Role::Button && n.name.as_deref() == Some(b"Undo")));
    assert_eq!(press(&mut view, b"Undo"), expected(needs::Act::Undo { object: object(1) }));
    d::fire(
        &mut root,
        &Env { now: Time::ZERO.saturating_add(needs::HOLD), wall: f::NOW, limits: f::domain_limits() },
        &mut Queue::uncapped(),
    );
    let view = f::browser_page_view(&root);
    assert!(has(&view, b"In the store"));
    assert!(f::find(view.tree(), Role::Button, b"Undo").is_none());
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
