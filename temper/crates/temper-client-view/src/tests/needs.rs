//! Browser needs words, roles and acts (temper/docs/design/client/pages/README.md).
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
    limits.client.domain.text = 512;
    limits.client.domain.turn_text = 512;
    limits.client.domain.long_text = 4096;
    limits
}
fn object(message: u64) -> c::ObjectKey {
    c::ObjectKey::Question { task: f::INBOX_TASK, message }
}
fn event(root: &mut d::Domain, event: Event, out: &mut Queue<d::Request>) {
    d::step(root, &Env { now: Time::ZERO, wall: f::NOW, limits: limits() }, event, out);
}
pub(super) fn fixture(words: &str, cut: bool, count: u32, more: bool, filter: d::address::Filter) -> d::Domain {
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
    let (mut root, _, decided) = fixture_watched(words, cut, count, more, filter, title);
    deliver_asked(&mut root, decided, None);
    root
}
fn fixture_watched(
    words: &str,
    cut: bool,
    count: u32,
    more: bool,
    filter: d::address::Filter,
    title: Option<&[u8]>,
) -> (d::Domain, skein_lib::Token, skein_lib::Token) {
    fixture_classes(words, cut, count, more, filter, title, None)
}
fn fixture_classes(
    words: &str,
    cut: bool,
    count: u32,
    more: bool,
    filter: d::address::Filter,
    title: Option<&[u8]>,
    classes: Option<&temper_protocol_reaches::Classes>,
) -> (d::Domain, skein_lib::Token, skein_lib::Token) {
    fixture_classes_role(words, cut, count, more, filter, title, classes, false, 2)
}
#[expect(clippy::too_many_arguments, reason = "the card fixture carries independent document and role rows")]
#[expect(clippy::too_many_lines, reason = "the fixture delivers independent question and class documents")]
fn fixture_classes_role(
    words: &str,
    cut: bool,
    count: u32,
    more: bool,
    filter: d::address::Filter,
    title: Option<&[u8]>,
    classes: Option<&temper_protocol_reaches::Classes>,
    owner: bool,
    projects: u32,
) -> (d::Domain, skein_lib::Token, skein_lib::Token) {
    let (mut root, me_watch, _, _, _, _, _, standing, _) = f::root_at_with_classes(
        Address::Settings { project: f::PROJECT },
        &limits(),
        b"Shop",
        None,
        None,
        None,
        Some(&[]),
        Some(d::forge::ways::Ways::STOCK),
        classes.is_none(),
        true,
        classes,
    );
    if owner || projects == 1 {
        let mut me = root.pages().me(root.client()).unwrap().clone();
        me.projects.truncate(projects);
        if owner {
            me.projects.get_mut(0).unwrap().role = c::HeldRole::Owner;
        }
        event(
            &mut root,
            Event::Client {
                event: jig_client::Event::Up {
                    up: jig_client::Up::Watched {
                        watch: me_watch,
                        event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me))),
                    },
                },
            },
            &mut Queue::uncapped(),
        );
    }
    let standing = standing.unwrap();
    let watch = standing;
    let mut opening = Queue::uncapped();
    event(&mut root, Event::Went { place: d::Place::Address(Address::Inbox { filter }) }, &mut opening);
    let mut decided = None;
    let subject = d::forge::decided::watch(f::PROJECT, &limits().forge).unwrap();
    for request in &opening {
        if let d::Request::Unlinked {
            request: jig_client::UnlinkedRequest::Down { down: jig_client::Down::Open { watch, subject: found } },
        } = request
            && found == &subject
        {
            decided = Some(*watch);
        }
    }
    for request in &opening {
        if let d::Request::Unlinked {
            request: jig_client::UnlinkedRequest::Down { down: jig_client::Down::Open { watch, subject } },
        } = request
            && Some(subject) == d::forge::decided::watch(f::SECOND_PROJECT, &limits().forge).as_ref()
        {
            deliver_asked_for(&mut root, *watch, f::SECOND_PROJECT, None);
        }
    }
    let mut entries = List::with_capacity(count);
    for message in 1..=u64::from(count) {
        let project =
            if projects > 1 && filter.project.is_none() && message == 2 { f::SECOND_PROJECT } else { f::PROJECT };
        entries.push(documents::entry(object(message), project, message, Wall::EPOCH, None)).unwrap();
    }
    let mut out = Queue::uncapped();
    event(
        &mut root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch,
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
    (root, watch, decided.unwrap())
}
fn deliver_asked(root: &mut d::Domain, watch: skein_lib::Token, reach: Option<temper_protocol_reaches::Reach>) {
    deliver_asked_for(root, watch, f::PROJECT, reach);
}
fn deliver_asked_for(
    root: &mut d::Domain,
    watch: skein_lib::Token,
    project: u32,
    reach: Option<temper_protocol_reaches::Reach>,
) {
    use temper_client_protocol_wire::{self as wire, v1 as w};
    let mut asked = List::with_capacity(1);
    if let Some(reach) = reach {
        asked
            .push(
                w::Asked::new(
                    &w::CEILINGS,
                    w::AskedParts {
                        number: 1,
                        task: f::INBOX_TASK,
                        at: 0,
                        reach: temper_protocol_reaches::reach_record(&reach),
                    },
                )
                .unwrap(),
            )
            .unwrap();
    }
    let document =
        w::Decisions::new(&w::CEILINGS, w::DecisionsParts { project, rows: List::with_capacity(0), asked, cut: false })
            .unwrap();
    event(
        root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::FamilyDelivered {
                    watch,
                    record: jig_client::Family {
                        family: wire::FORGE,
                        version: wire::CLIENT_VERSION,
                        bytes: wire::decisions(&document),
                    },
                },
            },
        },
        &mut Queue::uncapped(),
    );
}
fn deliver_decided(root: &mut d::Domain, watch: skein_lib::Token) {
    use temper_client_protocol_wire::{self as wire, v1 as w};
    let reach = temper_protocol_reaches::Reach {
        kind: temper_protocol_reaches::Kind::Tests,
        class: temper_protocol_reaches::Class::Decide,
        reason: temper_protocol_reaches::Reason::Kind,
        under: 0,
        sets: 0,
        matched: 0,
        away: false,
    };
    let row = w::Decision::new(
        &w::CEILINGS,
        w::DecisionParts {
            number: 1,
            task: 99,
            at: 0,
            question: b"Which tests?".as_slice().into(),
            took: b"The focused suite".as_slice().into(),
            why: b"It covers the rule.".as_slice().into(),
            overturned: false,
            reach: temper_protocol_reaches::reach_record(&reach),
        },
    )
    .unwrap();
    let document = w::Decisions::new(
        &w::CEILINGS,
        w::DecisionsParts {
            project: f::PROJECT,
            rows: List::from_boxed(Box::new([row])),
            asked: List::with_capacity(0),
            cut: false,
        },
    )
    .unwrap();
    let mut out = Queue::uncapped();
    event(
        root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::FamilyDelivered {
                    watch,
                    record: jig_client::Family {
                        family: wire::FORGE,
                        version: wire::CLIENT_VERSION,
                        bytes: wire::decisions(&document),
                    },
                },
            },
        },
        &mut out,
    );
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
    let nodes = main(view);
    if style == look::ASK_META {
        let start = nodes.iter().position(|n| n.styles.as_slice().contains(&look::ASK)).unwrap();
        return nodes[start..].iter().find(|n| n.styles.as_slice().contains(&style)).expect("card part");
    }
    nodes.iter().find(|n| n.styles.as_slice().contains(&style)).expect("styled part")
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
    let mut builder = skein_ui::Builder::new(&f::tree_limits());
    let mut bindings = crate::frame::Bindings::new(128);
    crate::pages::home_cards::cards(
        &mut builder,
        root,
        &[needs::Need::Card(card)],
        false,
        &words,
        &mut bindings,
        &mut crate::words::Times::new(f::NOW),
    );
    let home = builder.finish();
    let text = f::text(&home);
    for words in [b"MakeProject answers".as_slice(), b"The box test reproduces"] {
        assert!(skein_lib::bytes::find(&text, words).is_none());
    }
    for name in ["In jig’s store".as_bytes(), b"In temper serve", b"Tell it something else"] {
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
            look::NEEDS_BAR,
            look::ASK_META,
            look::FLOW_Q,
            look::ASK_CONTEXT,
            look::ASK_SAW,
            look::WHY,
            look::ASK_CHOICES,
            look::NEED_TELL,
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
        for name in ["Reply…".as_bytes(), b"Later"] {
            assert!(f::find(view.tree(), Role::Button, name).is_none());
        }
        assert!(f::find(view.tree(), Role::TextArea, b"Or tell it something else").is_some());
        home_parts(&root, object);
    }
}
#[test]
fn first_choice_is_primary_when_none_recommends_and_only_recommendations_have_rec() {
    use temper_protocol_reaches::{Class, Classes, Item, Kind, PathSet, Reach, Reason, Subject};
    for (words, primary, rec) in [
        ("Where?\nChoice: First\nChoice: Second", b"First".as_slice(), false),
        ("Where?\nChoice: First\nChoice: Second (recommended)", b"Second".as_slice(), true),
    ] {
        let root = fixture(words, false, 1, false, d::address::Filter::ALL);
        let view = f::browser_page_view(&root);
        assert_eq!(styled(&view, look::RECOMMENDED).name.as_deref(), Some(primary));
        assert_eq!(main(&view).iter().any(|n| n.styles.as_slice().contains(&look::REC)), rec);
    }
    let classes = Classes {
        items: Box::new([Item {
            subject: Subject::Kind(Kind::Implementation),
            words: b"small repairs".as_slice().into(),
            class: Class::Ask,
        }]),
        sets: Box::new([PathSet {
            number: 1,
            name: b"the protocol core".as_slice().into(),
            patterns: Box::new([b"src/**".as_slice().into()]),
        }]),
    };
    let base = Reach {
        kind: Kind::Implementation,
        class: Class::Ask,
        reason: Reason::Kind,
        under: 0,
        sets: 0,
        matched: 0,
        away: false,
    };
    let mut cases = Vec::new();
    for (kind, clause) in [
        (Kind::Wording, "it is a wording choice"),
        (Kind::Retry, "it retries or splits a failing task"),
        (Kind::Implementation, "it is an implementation choice"),
        (Kind::Tests, "it changes only tests"),
        (Kind::Dependency, "it adds a dependency"),
        (Kind::Departure, "it departs from a landed design"),
        (Kind::Money, "it spends money past a budget"),
        (Kind::Irreversible, "it can’t be undone"),
        (Kind::Rules, "it changes what reaches you"),
        (Kind::Other, "no kind of decision names it"),
    ] {
        for (class, words) in [
            (Class::Ask, "asks you unless you’re away"),
            (Class::AskAway, "asks you, even when away"),
            (Class::Wait, "always waits for you"),
        ] {
            let mut expected = String::from("Reached you because ");
            expected.push_str(clause);
            expected.push_str(", which ");
            expected.push_str(words);
            expected.push('.');
            cases.push((Reach { kind, class, ..base }, true, expected));
        }
    }
    cases.extend([
        (Reach { reason: Reason::Set(1), ..base }, true, "Reached you because it changes the protocol core, which asks you unless you’re away.".into()),
        (Reach { reason: Reason::Set(1), ..base }, false, "Reached you because it changes one of Shop’s path sets, which asks you unless you’re away.".into()),
        (Reach { under: 1, ..base }, true, "Reached you because it is an implementation choice under “small repairs”, which asks you unless you’re away.".into()),
        (Reach { reason: Reason::Always, ..base }, true, "Reached you because its goal always asks you.".into()),
    ]);
    for (reach, read, expected) in cases {
        let filter = if read {
            d::address::Filter { project: Some(f::PROJECT), ..d::address::Filter::ALL }
        } else {
            d::address::Filter::ALL
        };
        let (mut root, _, watch) =
            fixture_classes(QUESTION, false, 1, false, filter, None, if read { Some(&classes) } else { None });
        deliver_asked(&mut root, watch, Some(reach));
        let view = f::browser_page_view(&root);
        assert_eq!(styled(&view, look::WHY).name.as_deref(), Some(expected.as_bytes()), "{reach:?}");
        let card = root.pages().inbox().unwrap().cards.get(0).unwrap();
        let offered = pages::stop_asking::offered(&root, needs::Need::Card(card), f::NOW).is_some();
        assert_eq!(f::find(view.tree(), Role::Button, b"Keep asking").is_some(), offered, "{reach:?}");
        if offered {
            let rule = if reach.reason == Reason::Set(1) {
                "Stop asking: let agents decide some changes to the protocol core"
            } else if reach.under == 1 {
                "Stop asking: let agents decide implementation choices under “small repairs”"
            } else {
                continue;
            };
            assert!(f::find(view.tree(), Role::Button, rule.as_bytes()).is_some());
        }
    }
}
#[test]
fn tell_field_keeps_words_and_sends_only_a_nonblank_reply() {
    for (words, send) in
        [("", false), ("  ", false), ("My reply", true), ("A reply longer than the ordinary card text bound.", true)]
    {
        let mut root = root();
        let mut view = f::browser_page_view(&root);
        let field = f::find(view.tree(), Role::TextArea, b"Or tell it something else").unwrap();
        let input = crate::act(&mut view, field, Act::Input { value: words.as_bytes().into() }).unwrap();
        assert_eq!(
            input,
            d::Action::Pages(pages::Act::CardWords { object: object(1), words: words.as_bytes().into() })
        );
        event(&mut root, Event::Act { action: input }, &mut Queue::uncapped());
        let mut view = f::browser_page_view(&root);
        assert_eq!(styled(&view, look::NEED_TELL).role, Role::Slot);
        let field = f::find(view.tree(), Role::TextArea, b"Or tell it something else").unwrap();
        let node = view.tree().find(field).unwrap();
        assert_eq!(node.placeholder.as_deref(), Some(b"Or tell it something else".as_slice()));
        assert!(node.focus.is_none());
        assert_eq!(node.value.as_ref().unwrap().text.as_ref(), words.as_bytes());
        assert_eq!(f::find(view.tree(), Role::Button, b"Send").is_some(), send);
        assert!(f::find(view.tree(), Role::Button, b"Cancel").is_none());
        if send {
            let action = press(&mut view, b"Send");
            assert_eq!(action, expected(needs::Act::SendReply { object: object(1) }));
            event(&mut root, Event::Act { action }, &mut Queue::uncapped());
            let lines = root.pages().needs().answered(f::NOW, needs::ANSWERED_ON_NEEDS, None);
            assert_eq!(lines[0].label, needs::Label::Reply(words.as_bytes().into()));
            assert_eq!(lines[0].object, object(1));
            assert_eq!(
                root.pages().needs().shown(root.pages().inbox().unwrap(), root.forge().approvals()).unwrap().object(),
                object(2)
            );
        }
    }
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
            assert!(f::find(view.tree(), Role::TextArea, b"Or tell it something else").is_some());
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
fn phone_bar_links_home_and_counts_answered_including_cut() {
    for (projects, project, address) in [
        (1_u32, None, Address::Home { project: f::PROJECT }),
        (1, Some(f::PROJECT), Address::Home { project: f::PROJECT }),
        (2, None, Address::AllHome),
        (2, Some(f::PROJECT), Address::AllHome),
    ] {
        for more in [false, true] {
            let filter = d::address::Filter { project, ..d::address::Filter::ALL };
            let (mut root, _, decided) =
                fixture_classes_role(QUESTION, false, 3, more, filter, None, None, false, projects);
            deliver_asked(&mut root, decided, None);
            let mut view = f::browser_page_view(&root);
            assert!(
                main(&view).iter().any(|node| node.styles.as_slice().contains(&look::NEEDS_BAR)),
                "bar for {projects} projects, filter {project:?}, more {more}: {:?}",
                root.pages().inbox()
            );
            let bar = styled(&view, look::NEEDS_BAR);
            assert_eq!(bar.role, Role::Slot);
            assert_eq!(main(&view)[1].id, bar.id, "the bar precedes the page");
            let nodes = main(&view);
            let start = nodes.iter().position(|node| node.id == bar.id).unwrap();
            let link = &nodes[start + 1];
            assert_eq!(link.role, Role::Link);
            assert_eq!(link.name.as_deref(), Some(b"Needs you".as_slice()));
            let place = &nodes[start + 1 + usize::try_from(link.size).unwrap()];
            assert!(place.styles.as_slice().contains(&look::NEEDS_BAR_AT), "the place follows the link");
            let link = link.id;
            assert_eq!(
                crate::act(&mut view, link, Act::Press),
                Some(d::Action::Pages(pages::Act::Go { address: address.clone() }))
            );
            assert_eq!(styled(&view, look::NEEDS_BAR_AT).role, Role::Text);
            assert_eq!(
                styled(&view, look::NEEDS_BAR_AT).name.as_deref(),
                Some(if more { b"1 of 3+".as_slice() } else { b"1 of 3" })
            );
            for name in
                [b"temper-flow-head".as_slice(), b"temper-dots", b"temper-dot", b"temper-dot-done", b"temper-next-up"]
            {
                assert!(
                    !main(&view).iter().any(|node| node
                        .styles
                        .as_slice()
                        .iter()
                        .any(|style| look::NAMES.style(*style) == Some(name)))
                );
            }
            assert!(!main(&view).iter().any(|node| node.role == Role::Figure));
            assert!(!has(&view, b"Next:"));
            assert!(!has(&view, b"The last one"));
            choose(&mut root, 1);
            let view = f::browser_page_view(&root);
            assert_eq!(
                styled(&view, look::NEEDS_BAR_AT).name.as_deref(),
                Some(if more { b"2 of 3+".as_slice() } else { b"2 of 3" })
            );
            choose(&mut root, 2);
            choose(&mut root, 3);
            let mut view = f::browser_page_view(&root);
            assert!(!main(&view).iter().any(|node| node.styles.as_slice().contains(&look::NEEDS_BAR)));
            let home = main(&view)
                .iter()
                .find(|node| node.role == Role::Link && node.name.as_deref() == Some(b"Home"))
                .unwrap()
                .id;
            assert_eq!(
                crate::act(&mut view, home, Act::Press),
                Some(d::Action::Pages(pages::Act::Go { address: address.clone() }))
            );
        }
    }
}

#[test]
fn opened_card_and_later_order() {
    let (mut root, _, decided) = fixture_watched(
        QUESTION,
        false,
        3,
        false,
        d::address::Filter { project: Some(f::PROJECT), ..d::address::Filter::ALL },
        None,
    );
    deliver_decided(&mut root, decided);
    let view = f::browser_page_view(&root);
    assert!(!has(&view, b"Next:"));
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
    assert!(!has(&view, b"The last one"));
    assert!(has(&view, b"later"));
    assert!(has(&view, "Decided for you · 1".as_bytes()));
    assert!(!has(&view, b"for 24 hours, then in each task's history"));
    assert!(f::find(view.tree(), Role::Button, b"Limits").is_some());
    let nodes = main(&view);
    let answered = nodes.iter().position(|node| node.styles.as_slice().contains(&look::ANSWERED_SECTION)).unwrap();
    let decided = nodes
        .iter()
        .position(|node| node.role == Role::Region && node.name.as_deref() == Some("Decided for you · 1".as_bytes()))
        .unwrap();
    assert!(answered < decided);
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

fn decisions_delivered(
    root: &mut d::Domain,
    project: u32,
    watch: skein_lib::Token,
    count: u32,
    out: &mut Queue<d::Request>,
) {
    use temper_client_protocol_wire::{self as wire, v1 as w};
    let mut rows = List::with_capacity(count);
    for number in 1..=count {
        if (number % 2 == 1) != (project == f::PROJECT) {
            continue;
        }
        rows.push(
            w::Decision::new(
                &w::CEILINGS,
                w::DecisionParts {
                    number,
                    task: 90 + u64::from(number),
                    at: 3_900_000_000_000,
                    question: b"Which approach?".as_slice().into(),
                    took: b"Fixed the harness.".as_slice().into(),
                    why: b"Tests are its area.".as_slice().into(),
                    overturned: false,
                    reach: w::Reach::new(
                        &w::CEILINGS,
                        w::ReachParts {
                            kind: w::ReachKind::Tests,
                            class: w::ReachClass::Decide,
                            reason: w::ReachReason::Kind,
                            under: 0,
                            sets: 0,
                            matched: 0,
                            away: false,
                        },
                    )
                    .expect("decision reach"),
                },
            )
            .unwrap(),
        )
        .unwrap();
    }
    let bytes = wire::decisions(
        &w::Decisions::new(
            &w::CEILINGS,
            w::DecisionsParts { project, rows, asked: List::with_capacity(0), cut: false },
        )
        .unwrap(),
    );
    event(
        root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::FamilyDelivered {
                    watch,
                    record: jig_client::Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes },
                },
            },
        },
        out,
    );
}
fn empty_decided_inbox(root: &mut d::Domain, watch: skein_lib::Token, out: &mut Queue<d::Request>) {
    event(
        root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch,
                    event: c::WatchEvent::Delivered(c::Delivery::Inbox(Box::new(documents::inbox(
                        List::with_capacity(0),
                        0,
                        c::Count { value: 0, limited: false },
                        0,
                        false,
                    )))),
                },
            },
        },
        out,
    );
}
pub(super) fn decided_fixture(filter: d::address::Filter, count: u32, open: bool) -> d::Domain {
    let (mut root, _, initial) = f::root_at(Address::Inbox { filter }, &limits());
    let mut out = Queue::uncapped();
    event(&mut root, Event::Went { place: d::Place::Address(Address::Accounts) }, &mut out);
    event(&mut root, Event::Went { place: d::Place::Address(Address::Inbox { filter }) }, &mut out);
    if let Some(watch) = initial {
        empty_decided_inbox(&mut root, watch, &mut out);
    }
    while let Some(request) = out.pop() {
        let d::Request::Unlinked { request: jig_client::UnlinkedRequest::Down { down } } = request else { continue };
        match down {
            jig_client::Down::Open { watch, subject: jig_client::Watchable::Jig(c::Subject::Inbox { .. }) } => {
                empty_decided_inbox(&mut root, watch, &mut out);
            }
            jig_client::Down::Open { watch, subject } => {
                for project in [f::PROJECT, f::SECOND_PROJECT] {
                    if Some(subject.clone()) != d::forge::decided::watch(project, &limits().forge) {
                        continue;
                    }
                    decisions_delivered(&mut root, project, watch, count, &mut out);
                }
            }
            jig_client::Down::Read { read, what: jig_client::Readable::Jig(c::Read::Inbox { .. }) } => {
                event(
                    &mut root,
                    Event::Client {
                        event: jig_client::Event::Up {
                            up: jig_client::Up::ReadDone {
                                read,
                                result: jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Inbox(Box::new(
                                    c::InboxPage {
                                        selection: None,
                                        before: None,
                                        rows: List::with_capacity(0),
                                        after: None,
                                        cut: false,
                                    },
                                )))),
                            },
                        },
                    },
                    &mut out,
                );
            }
            jig_client::Down::Read { read, what: jig_client::Readable::Jig(c::Read::Task { task }) } => {
                let mut document = f::task::document();
                document.chip.task = task;
                document.chip.title = f::task::document_words(b"Decided task");
                event(
                    &mut root,
                    Event::Client {
                        event: jig_client::Event::Up {
                            up: jig_client::Up::ReadDone {
                                read,
                                result: jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Task(Box::new(
                                    document,
                                )))),
                            },
                        },
                    },
                    &mut out,
                );
            }
            jig_client::Down::Save { .. }
            | jig_client::Down::Erase { .. }
            | jig_client::Down::Send { .. }
            | jig_client::Down::Continue { .. }
            | jig_client::Down::Close { .. }
            | jig_client::Down::Read { .. } => {}
        }
    }
    assert!(root.pages().inbox().unwrap().known(), "decided fixture has an answered inbox");
    if open {
        approvals(&mut root, &[approval_row()], 0);
    }
    root
}
#[test]
fn done_with_and_without_lines_and_home_project() {
    for (answered, decided_rows) in [(false, false), (true, false), (false, true), (true, true)] {
        let (mut root, _, decided) =
            fixture_watched(QUESTION, false, u32::from(answered), false, d::address::Filter::ALL, None);
        if decided_rows {
            deliver_decided(&mut root, decided);
        }
        if answered {
            choose(&mut root, 1);
        }
        let mut view = f::browser_page_view(&root);
        assert!(f::find(view.tree(), Role::Heading { level: 1 }, b"Nothing needs you.").is_some());
        assert!(has(&view, b"Every question is answered. Work carries on; Home tells you what changed."));
        assert_eq!(main(&view).iter().any(|n| n.styles.as_slice().contains(&look::ANSWERED)), answered);
        assert_eq!(has(&view, "Decided for you · 1".as_bytes()), decided_rows);
        if decided_rows {
            assert!(has(&view, "Shop · Task 99 · 00:00".as_bytes()));
            assert!(f::find(view.tree(), Role::Button, b"Limits").is_none());
            let nodes = main(&view);
            let region = nodes
                .iter()
                .position(|node| {
                    node.role == Role::Region && node.name.as_deref() == Some("Decided for you · 1".as_bytes())
                })
                .unwrap();
            let home =
                nodes.iter().position(|node| node.role == Role::Link && node.name.as_deref() == Some(b"Home")).unwrap();
            assert!(region > home);
        }
        assert!(!main(&view).iter().any(|n| n.styles.as_slice().contains(&look::NEEDS_BAR)));
        let home = main(&view).iter().find(|n| n.role == Role::Link && n.name.as_deref() == Some(b"Home")).unwrap().id;
        assert_eq!(
            crate::act(&mut view, home, Act::Press),
            Some(d::Action::Pages(pages::Act::Go { address: Address::AllHome }))
        );
    }
}
#[test]
#[expect(clippy::disallowed_macros, reason = "the table formats expected counts and project labels in both shells")]
fn decided_head_and_rows_follow_the_project_filter() {
    for (open, count, project, showing) in [
        (false, 0, None, 0),
        (true, 0, None, 0),
        (false, 4, None, 4),
        (true, 4, None, 4),
        (true, 1, None, 1),
        (true, 4, Some(f::PROJECT), 2),
        (false, 4, Some(f::SECOND_PROJECT), 2),
    ] {
        let root = decided_fixture(d::address::Filter { project, ..d::address::Filter::ALL }, count, open);
        for (mut view, browser) in [(f::browser_page_view(&root), true), (f::page_view(&root), false)] {
            let nodes = main(&view);
            assert_eq!(nodes.iter().filter(|n| n.styles.as_slice().contains(&look::DECISION)).count(), showing);
            let heading = format!("Decided for you · {showing}");
            assert_eq!(f::find(view.tree(), Role::Region, heading.as_bytes()).is_some(), showing > 0);
            if showing == 0 {
                assert!(!has(&view, b"were decided for you"));
                continue;
            }
            assert!(
                f::find(view.tree(), Role::Heading { level: 2 }, format!("Decided for you · {showing}").as_bytes())
                    .is_some()
            );
            assert_eq!(has(&view, b"for 24 hours, then in each task's history"), project.is_none());
            assert_eq!(f::find(view.tree(), Role::Button, b"Limits").is_some(), project.is_some());
            assert!(has(&view, b"Which approach?"));
            assert!(has(&view, b"Fixed the harness."));
            assert!(has(&view, b"Allowed: test-only changes are decided and listed."));
            assert!(has(&view, b"Tests are its area."));
            let name = if project == Some(f::SECOND_PROJECT) { "Second" } else { "Shop" };
            if project.is_some() {
                assert!(has(&view, "Decided task · 01:05".as_bytes()));
                assert!(!has(&view, "Shop ·".as_bytes()));
                assert!(!has(&view, "Second ·".as_bytes()));
            } else {
                assert!(has(&view, format!("{name} · Decided task · 01:05").as_bytes()));
            }
            if open && browser {
                let sentence = match showing {
                    1 => "One",
                    2 => "Two",
                    4 => "Four",
                    _ => unreachable!(),
                };
                let suffix = if showing == 1 { "was" } else { "were" };
                let heading = format!("One thing needs you. {sentence} {suffix} decided for you.");
                assert!(f::find(view.tree(), Role::Heading { level: 1 }, heading.as_bytes()).is_some());
            }
            let region = nodes
                .iter()
                .position(|n| n.role == Role::Region && n.name.as_deref() == Some(heading.as_bytes()))
                .unwrap();
            let done_or_list = nodes.iter().position(|n| n.role == Role::Heading { level: 1 }).unwrap();
            assert!(region > done_or_list);
            if let Some(project) = project {
                let limits = f::find(view.tree(), Role::Button, b"Limits").unwrap();
                let (node, action) = if browser {
                    (limits, Act::Press)
                } else {
                    f::terminal_command(view.tree(), &view.limits().tree, limits, b"Limits", heading.as_bytes())
                };
                assert_eq!(
                    crate::act(&mut view, node, action),
                    Some(d::Action::Pages(pages::Act::Away(pages::away::Act::Limits { project })))
                );
            }
            let over = f::find(view.tree(), Role::Button, b"Overturn").unwrap();
            assert_eq!(
                crate::act(&mut view, over, Act::Press),
                Some(d::Action::Pages(pages::Act::Away(pages::away::Act::Overturn {
                    project: project.unwrap_or(f::PROJECT),
                    number: if project == Some(f::SECOND_PROJECT) { 2 } else { 1 }
                })))
            );
        }
    }
}
#[test]
fn list_keeps_one_card_and_opens_rows_in_place() {
    for (filter, kicker, chips) in [
        (d::address::Filter::ALL, "Needs you · all projects", &["Shop", "Second", "Shop"][..]),
        (d::address::Filter { project: Some(f::PROJECT), ..d::address::Filter::ALL }, "Needs you · Shop", &[][..]),
    ] {
        let mut root = fixture(QUESTION, false, 3, false, filter);
        let address = root.pages().address().clone();
        let mut view = f::browser_page_view(&root);
        assert!(f::find(view.tree(), Role::Paragraph, kicker.as_bytes()).is_some());
        assert!(f::find(view.tree(), Role::Heading { level: 1 }, b"Three things need you.").is_some());
        assert!(f::find(view.tree(), Role::Heading { level: 2 }, "Needs you · 3".as_bytes()).is_some());
        assert!(f::find(view.tree(), Role::Text, b"oldest first").is_some());
        let actual: Vec<_> = main(&view)
            .iter()
            .filter(|n| n.styles.as_slice().contains(&look::CHIP))
            .map(|n| core::str::from_utf8(n.name.as_deref().unwrap()).unwrap())
            .collect();
        assert_eq!(actual, chips);
        let shapes: Vec<_> = main(&view)
            .iter()
            .filter_map(|n| {
                if n.styles.as_slice().contains(&look::ASK) {
                    Some("open")
                } else if n.styles.as_slice().contains(&look::NEED_ROW) {
                    Some("closed")
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(shapes, ["open", "closed", "closed"]);
        let closed: Vec<_> =
            main(&view).iter().filter(|n| n.styles.as_slice().contains(&look::NEED_ROW)).map(|n| n.id).collect();
        for (id, message, name, project) in [
            (closed[0], 2, "forge-cards", if chips.is_empty() { "" } else { "Second" }),
            (closed[1], 3, "third", if chips.is_empty() { "" } else { "Shop" }),
        ] {
            let index = view.tree().nodes().iter().position(|n| n.id == id).unwrap();
            let text = f::node_text(view.tree(), index);
            assert!(skein_lib::bytes::find(&text, name.as_bytes()).is_some());
            assert!(skein_lib::bytes::find(&text, project.as_bytes()).is_some());
            assert!(skein_lib::bytes::find(&text, b"a question").is_some());
            assert!(skein_lib::bytes::find(&text, QUESTION.lines().next().unwrap().as_bytes()).is_some());
            assert_eq!(
                crate::act(&mut view, id, Act::Press),
                Some(expected(needs::Act::Open { object: object(message) }))
            );
        }
        let action = crate::act(&mut view, closed[0], Act::Press).unwrap();
        let mut out = Queue::uncapped();
        event(&mut root, Event::Act { action }, &mut out);
        for request in &out {
            if let d::Request::Unlinked {
                request: jig_client::UnlinkedRequest::Down { down: jig_client::Down::Send { .. } },
            } = request
            {
                panic!("opening a card sends no operation");
            }
        }
        assert_eq!(root.pages().address(), &address);
        let view = f::browser_page_view(&root);
        assert!(
            styled(&view, look::ASK_META)
                .name
                .as_deref()
                .unwrap()
                .windows(b"forge-cards".len())
                .any(|w| w == b"forge-cards")
        );
        let shapes: Vec<_> = main(&view)
            .iter()
            .filter_map(|n| {
                if n.styles.as_slice().contains(&look::ASK) {
                    Some("open")
                } else if n.styles.as_slice().contains(&look::NEED_ROW) {
                    Some("closed")
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(shapes, ["closed", "open", "closed"]);
        assert_eq!(styled(&view, look::NEEDS_BAR_AT).name.as_deref(), Some(b"1 of 3".as_slice()));
        assert!(!has(&view, b"Next:"));
    }
}
#[test]
fn empty_column_until_delivered_and_terminal_keeps_its_list() {
    let (pending, _, _) = f::root_at(Address::Inbox { filter: d::address::Filter::ALL }, &f::domain_limits());
    let view = f::browser_page_view(&pending);
    assert_eq!(main(&view).len(), 1);
    let pending = f::inbox_root();
    let view = f::browser_page_view(&pending);
    assert_eq!(main(&view).len(), 1);
    let (mut root, _, decided) = fixture_watched(QUESTION, false, 1, false, d::address::Filter::ALL, None);
    assert_eq!(main(&f::browser_page_view(&root)).len(), 1);
    let terminal = f::page_view(&root);
    assert!(has(&terminal, b"Fix the dropped watch"));
    assert!(!has(&terminal, b"Reached you because"));
    deliver_asked(&mut root, decided, None);
    assert!(has(&f::browser_page_view(&root), b"Reached you because it was asked of you directly."));
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
pub(super) fn held(reason: Option<c::HoldCategory>) -> c::Escalation {
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
pub(super) fn kinds(values: &[(c::ObjectKey, c::Read, c::ReadValue)]) -> d::Domain {
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
#[expect(
    clippy::too_many_lines,
    reason = "the fixture delivers the inbox, decision document and each card read together"
)]
fn kinds_owned(
    values: &[(c::ObjectKey, c::Read, c::ReadValue)],
    whole: Option<c::ReadResult>,
    owned: Option<bool>,
) -> d::Domain {
    let (mut root, _, _, watch) = f::root_at_with_needs(Address::Settings { project: f::PROJECT }, &limits());
    let mut entries = List::with_capacity(u32::try_from(values.len()).unwrap());
    for (i, (object, _, _)) in values.iter().enumerate() {
        entries.push(documents::entry(*object, f::PROJECT, u64::try_from(i).unwrap(), Wall::EPOCH, None)).unwrap();
    }
    let mut out = Queue::uncapped();
    event(
        &mut root,
        Event::Went { place: d::Place::Address(Address::Inbox { filter: d::address::Filter::ALL }) },
        &mut out,
    );
    event(
        &mut root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch,
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
            request: jig_client::UnlinkedRequest::Down { down: jig_client::Down::Open { watch, subject } },
        } = &request
            && Some(subject) == d::forge::decided::watch(f::PROJECT, &limits().forge).as_ref()
        {
            deliver_asked(&mut root, *watch, None);
        }
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
    let positions: Vec<_> = [look::ASK_META, look::FLOW_Q, look::ASK_CONTEXT, look::ASK_CHOICES]
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
        assert_eq!(
            styled(&view, look::WHY).name.as_deref(),
            Some(b"Reached you because a proposal always waits for whoever may accept it.".as_slice())
        );
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
        assert!(f::find(view.tree(), Role::Button, b"Later").is_none());
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
            styled(&view, look::WHY).name.as_deref(),
            Some(if funds {
                b"Reached you because money past a budget always waits for you.".as_slice()
            } else {
                b"Reached you because a held task always waits for whoever may release it."
            })
        );
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
    assert!(f::find(view.tree(), Role::Button, b"Later").is_none());
}
#[test]
fn mixed_kinds_keep_each_short_and_later_order() {
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
    let expectations = ["Accept the plan.", "first-run-projects", "31 of 40 used", "Review the plan"];
    for (row, short) in rows.iter().zip(expectations) {
        let needs::Need::Card(card) =
            root.pages().needs().shown(root.pages().inbox().unwrap(), root.forge().approvals()).unwrap()
        else {
            panic!("inbox card")
        };
        assert_eq!(crate::pages::ask::read(card, &f::words()).short.as_ref(), short.as_bytes());
        let view = f::browser_page_view(&root);
        assert!(!has(&view, b"Next:"));
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
        let (mut root, watch, decided) = fixture_watched(QUESTION, false, 1, false, filter, None);
        deliver_asked(&mut root, decided, None);
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
        sets: Box::new([]),
        whole: true,
        yours: d::forge::approvals::Yours::Counts,
        size: None,
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
pub(super) fn size_cases() -> [(Option<d::forge::approvals::Size>, &'static str, &'static str); 6] {
    use d::forge::approvals::Size;
    [
        (Some(Size { files: 4, additions: 142, deletions: 18, cut: false }), "+142 -18 in 4 files", " in 4 files"),
        (Some(Size { files: 1, additions: 3, deletions: 0, cut: false }), "+3 -0 in 1 file", " in 1 file"),
        (
            Some(Size { files: u32::MAX, additions: 10, deletions: 2, cut: true }),
            "+10 -2 in 4294967295+ files",
            " in 4294967295+ files",
        ),
        (
            Some(Size { files: 0, additions: u32::MAX, deletions: u32::MAX, cut: true }),
            "+4294967295+ -4294967295+ in 0 files",
            " in 0 files",
        ),
        (
            Some(Size { files: u32::MAX, additions: u32::MAX, deletions: u32::MAX, cut: false }),
            "+4294967295 -4294967295 in 4294967295 files",
            " in 4294967295 files",
        ),
        (None, "", ""),
    ]
}
pub(super) fn check_size(tree: &skein_ui::Tree, words: &str, files: &str) {
    let nodes = tree.nodes();
    let rows: Vec<_> = nodes.iter().enumerate().filter(|(_, n)| n.styles.as_slice().contains(&look::CHIPS)).collect();
    assert_eq!(rows.len(), usize::from(!words.is_empty()));
    assert_eq!(look::NAMES.style(look::CHIPS), Some(b"temper-chips".as_slice()));
    assert_eq!(look::NAMES.style(look::SIZE_FILES), Some(b"temper-size-files".as_slice()));
    if let Some((start, row)) = rows.first() {
        let row = &nodes[*start..start.checked_add(usize::try_from(row.size).unwrap()).unwrap()];
        let chip = &row[1];
        assert_eq!(row.len(), usize::try_from(chip.size).unwrap().checked_add(1).unwrap(), "one chip only");
        assert_eq!(chip.styles.as_slice(), [look::CHIP]);
        assert!(row.iter().all(|n| n.binding.is_none() && n.link.is_none() && n.external.is_none()));
        let actual: Vec<_> = row.iter().flat_map(|n| n.text.iter()).flat_map(|run| run.text.iter().copied()).collect();
        assert_eq!(actual, words.as_bytes());
        let files_node =
            row.iter().find(|n| n.styles.as_slice().contains(&look::SIZE_FILES)).expect("files' own element");
        assert_eq!(files_node.name.as_deref(), Some(files.as_bytes()));
    } else {
        assert!(!nodes.iter().any(|n| n.styles.as_slice().contains(&look::SIZE_FILES)));
    }
}
#[test]
fn approval_size_is_one_plain_chip_after_context_before_acts_or_nothing() {
    for (size, words, files) in size_cases() {
        let row = d::forge::approvals::Row { size, ..approval_row() };
        let mut root = kinds(&[]);
        approvals(&mut root, core::slice::from_ref(&row), 0);
        let mut view = f::browser_page_view(&root);
        check_size(view.tree(), words, files);
        let nodes = view.tree().nodes();
        let context = nodes.iter().rposition(|n| n.styles.as_slice().contains(&look::ASK_CONTEXT)).unwrap();
        let after = context + usize::try_from(nodes[context].size).unwrap();
        let choices = nodes.iter().position(|n| n.styles.as_slice().contains(&look::ASK_CHOICES)).unwrap();
        if size.is_some() {
            assert!(nodes[after].styles.as_slice().contains(&look::CHIPS));
            assert_eq!(after + usize::try_from(nodes[after].size).unwrap(), choices);
        } else {
            assert_eq!(after, choices, "no placeholder without a size");
        }
        assert_eq!(
            nodes[context].name.as_deref(),
            Some(b"This project's landing rule 1 asks for one approval by a maintainer.".as_slice())
        );
        assert!(f::find(view.tree(), Role::Link, "Review on Forgejo ↗".as_bytes()).is_some());
        assert_eq!(
            press(&mut view, b"Open"),
            d::Action::Pages(pages::Act::Go { address: Address::Task { task: row.task } })
        );
    }
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "the approval table holds the rule, eligibility, address and ordered card parts together"
)]
fn approval_parts_say_the_rule_and_offer_the_review_where_it_counts() {
    use d::forge::{approvals::Yours, forges::Kind};
    let rule = "This project's landing rule 1 asks for one approval by a maintainer.";
    for (kind, scope, rule_number, role, required, counted, sets, whole, yours, address, expected) in [
        (
            Kind::Forgejo,
            1,
            1,
            1,
            1,
            0,
            [(1, "vsr's protocol core")].as_slice(),
            true,
            Yours::Counts,
            "https://forge.example/files",
            [rule, "Signs off changes to vsr's protocol core."].as_slice(),
        ),
        (
            Kind::GitHub,
            0,
            2,
            0,
            2,
            1,
            [].as_slice(),
            true,
            Yours::Counts,
            "https://github.com/work/hello/pull/24/files",
            &["The deployment's landing rule 2 asks for 2 approvals by owners; 1 so far."],
        ),
        (
            Kind::Forgejo,
            1,
            1,
            1,
            1,
            0,
            [(1, "A"), (3, "B")].as_slice(),
            false,
            Yours::Counts,
            "https://forge.example/files",
            &[
                rule,
                "Signs off changes to A and B.",
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
            [(1, "A")].as_slice(),
            true,
            Yours::NoAccount,
            "https://forge.example/files",
            &[
                rule,
                "Signs off changes to A.",
                "temper knows no Forgejo account of yours, so your review wouldn't count.",
            ],
        ),
        (
            Kind::GitHub,
            1,
            1,
            1,
            1,
            0,
            [].as_slice(),
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
            [].as_slice(),
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
            [].as_slice(),
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
            [].as_slice(),
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
            [].as_slice(),
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
            [].as_slice(),
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
            [].as_slice(),
            true,
            Yours::Counts,
            "https://forge.example/files",
            &["This project's landing rule 1 asks for 2 approvals by people with role 8; 1 so far."],
        ),
        (
            Kind::Forgejo,
            1,
            1,
            1,
            1,
            0,
            [(1, "A"), (2, "B"), (3, "C")].as_slice(),
            true,
            Yours::Counts,
            "https://forge.example/files",
            &[rule, "Signs off changes to A, B and C."],
        ),
        (
            Kind::Forgejo,
            1,
            1,
            1,
            1,
            0,
            [(4, "")].as_slice(),
            true,
            Yours::Counts,
            "https://forge.example/files",
            &[rule, "Signs off changes to path set 4."],
        ),
        (
            Kind::GitHub,
            1,
            1,
            1,
            1,
            0,
            [(1, "A")].as_slice(),
            true,
            Yours::Counts,
            "https://github.com/files",
            &[rule, "Signs off changes to A."],
        ),
        (
            Kind::Forgejo,
            1,
            1,
            1,
            1,
            0,
            [
                (1, "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"),
                (2, "BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB"),
                (3, "CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC"),
                (4, "DDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDD"),
                (5, "EEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEE"),
                (6, "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"),
            ]
            .as_slice(),
            true,
            Yours::Counts,
            "https://forge.example/files",
            &[
                rule,
                "Signs off changes to AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA, BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB, CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC, DDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDD, EEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEE and FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF.",
            ],
        ),
    ] {
        let row = d::forge::approvals::Row {
            kind,
            scope,
            rule: rule_number,
            role,
            required,
            counted,
            sets: sets
                .iter()
                .map(|(number, name)| d::forge::approvals::ApprovalSet {
                    number: *number,
                    name: name.as_bytes().into(),
                })
                .collect(),
            whole,
            yours,
            address: address.as_bytes().into(),
            ..approval_row()
        };
        let mut root = kinds(&[]);
        approvals(&mut root, core::slice::from_ref(&row), 0);
        let mut view = f::browser_page_view(&root);
        let (question, review) = match (kind, sets.is_empty()) {
            (Kind::Forgejo, true) => ("Approve A change on Forgejo", "Review on Forgejo ↗"),
            (Kind::Forgejo, false) => ("Approve A change on Forgejo", "Sign off on Forgejo ↗"),
            (Kind::GitHub, true) => ("Approve A change on GitHub", "Review on GitHub ↗"),
            (Kind::GitHub, false) => ("Approve A change on GitHub", "Sign off on GitHub ↗"),
        };
        let heading = f::find(view.tree(), Role::Heading { level: 1 }, question.as_bytes()).unwrap();
        assert!(view.tree().find(heading).unwrap().styles.as_slice().contains(&look::FLOW_Q));
        let context: Vec<_> = main(&view)
            .iter()
            .filter(|node| node.role == Role::Paragraph && node.styles.as_slice().contains(&look::ASK_CONTEXT))
            .map(|node| node.name.as_deref().unwrap())
            .collect();
        assert!(context.iter().all(|line| !String::from_utf8_lossy(line).contains("wait behind")));
        assert_eq!(context, expected.iter().map(|line| line.as_bytes()).collect::<Vec<_>>());
        let primary = f::find(view.tree(), Role::Link, review.as_bytes());
        if yours == Yours::Counts && skein_ui::markdown::safe_url(address.as_bytes()) {
            let node = view.tree().find(primary.expect("eligible safe review link")).unwrap();
            assert!(node.styles.as_slice().contains(&look::RECOMMENDED));
            assert_eq!(node.external.as_deref(), Some(address.as_bytes()));
        } else {
            assert!(primary.is_none(), "{yours:?}: no review that cannot count");
        }
        let prompts = crate::shared::need_prompts(&root, &f::browser_words());
        assert_eq!(prompts.len(), 1);
        match &prompts[0].act {
            crate::shared::NeedAct::Review { words, address: target } => {
                assert!(primary.is_some());
                assert_eq!(words.as_ref(), review.as_bytes());
                assert_eq!(target.as_ref(), address.as_bytes());
            }
            crate::shared::NeedAct::Web => assert!(primary.is_none()),
            crate::shared::NeedAct::Answer(_) => panic!("an approval is answered on the forge"),
        }
        assert_eq!(
            press(&mut view, b"Open"),
            d::Action::Pages(pages::Act::Go { address: Address::Task { task: row.task } })
        );
        assert!(f::find(view.tree(), Role::Button, b"Later").is_none());
        if let Some(primary) = primary {
            let open = f::find(view.tree(), Role::Button, b"Open").unwrap();
            let nodes = main(&view);
            assert!(
                nodes.iter().position(|n| n.id == primary).unwrap() < nodes.iter().position(|n| n.id == open).unwrap()
            );
        }
        assert_eq!(styled(&view, look::NEEDS_BAR).role, Role::Slot);
        assert!(has(&view, b"1 of 1"));
        assert!(!has(&view, b"The last one"));
    }
    let mut root = fixture(QUESTION, false, 1, false, d::address::Filter::ALL);
    let row = approval_row();
    approvals(&mut root, core::slice::from_ref(&row), 1);
    let mut view = f::browser_page_view(&root);
    assert!(has(&view, b"1 of 2+"));
    assert!(!has(&view, b"Next:"));
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

pub(super) fn stop_root(owner: bool, set: bool) -> d::Domain {
    use temper_protocol_reaches::{Class, Classes, Item, Kind, PathSet, Reach, Reason, Subject, Way};
    let classes = if set {
        Classes {
            items: Box::new([Item { subject: Subject::Set(1), words: Box::default(), class: Class::Ask }]),
            sets: Box::new([PathSet {
                number: 1,
                name: b"protocol core".as_slice().into(),
                patterns: Box::new([b"src/**".as_slice().into(), b"wire/**".as_slice().into()]),
            }]),
        }
    } else {
        Classes::stock(Way::AgentsReview)
    };
    let (mut root, _, watch) = fixture_classes_role(
        QUESTION,
        false,
        1,
        false,
        d::address::Filter { project: Some(f::PROJECT), ..d::address::Filter::ALL },
        None,
        Some(&classes),
        owner,
        2,
    );
    deliver_asked(
        &mut root,
        watch,
        Some(Reach {
            kind: Kind::Implementation,
            class: Class::Ask,
            reason: if set { Reason::Set(1) } else { Reason::Kind },
            under: 0,
            sets: u8::from(set),
            matched: u8::from(set),
            away: false,
        }),
    );
    root
}
fn rule_in_terminal(root: &d::Domain, rule: &[u8]) {
    use crate::frame::Bindings;
    use crate::pages::needs::{WhyForm, why_line};
    let tree_limits = f::tree_limits();
    let mut visible = skein_ui::View::new(&tree_limits);
    let mut b = visible.builder();
    let mut bindings = Bindings::new(128);
    let card = root.pages().inbox().unwrap().cards.get(0).unwrap();
    b.open(Role::Main);
    why_line(&mut b, root, &f::words(), &mut bindings, needs::Need::Card(card), WhyForm::Rule);
    b.close();
    visible.finish(b, 1, &mut Queue::uncapped());
    let tree = visible.tree();
    for (name, action) in [
        (b"Keep asking".as_slice(), pages::stop_asking::Act::Keep { object: card.object }),
        (rule, pages::stop_asking::Act::Open { object: card.object }),
    ] {
        let control = f::find(tree, Role::Button, name).unwrap();
        let (node, first) = f::terminal_command(tree, &tree_limits, control, name, b"Reached you because");
        assert_eq!(
            bindings.decode(tree.bound(node).unwrap().token, first),
            Some(d::Action::Pages(pages::Act::StopAsking(action)))
        );
    }
    assert!(f::find(tree, Role::Button, b"Stop asking").is_none());
}
#[test]
fn reason_line_rules_and_open_offers_belong_to_the_card() {
    use d::pages::stop_asking::Act as Stop;
    use temper_protocol_reaches::{Kind, Offer, Subject};
    for owner in [true, false] {
        for set in [false, true] {
            let mut root = stop_root(owner, set);
            let mut view = f::browser_page_view(&root);
            let rule = if set {
                b"Stop asking: let agents decide some changes to protocol core".as_slice()
            } else {
                b"Stop asking: let agents decide implementation choices".as_slice()
            };
            rule_in_terminal(&root, rule);
            let mut kept = stop_root(owner, set);
            page_act(&mut kept, pages::Act::StopAsking(Stop::Keep { object: object(1) }), &mut Queue::uncapped());
            let kept_view = f::browser_page_view(&kept);
            assert!(f::find(kept_view.tree(), Role::Button, b"Keep asking").is_none());
            assert!(f::find(kept_view.tree(), Role::Button, rule).is_none());
            assert!(has(&kept_view, b"Reached you because"));
            assert!(f::find(view.tree(), Role::Button, b"Stop asking").is_none());
            let line = super::inbox::subtree(view.tree(), super::inbox::styled(view.tree(), look::WHY));
            let buttons: Vec<_> =
                line.iter().filter(|n| n.role == Role::Button).map(|n| n.name.as_deref().unwrap()).collect();
            assert_eq!(buttons, [b"Keep asking".as_slice(), rule]);
            assert!(line.iter().any(|n| n.text.iter().any(|run| run.text.as_ref() == " · ".as_bytes())));
            let nodes = main(&view);
            let position = |style| nodes.iter().position(|n| n.styles.as_slice().contains(&style)).unwrap();
            assert!(position(look::ASK_CONTEXT) < position(look::WHY));
            assert!(position(look::ASK_SAW) < position(look::WHY));
            assert!(position(look::WHY) < position(look::ASK_CHOICES));
            assert_eq!(
                press(&mut view, b"Keep asking"),
                d::Action::Pages(pages::Act::StopAsking(Stop::Keep { object: object(1) }))
            );
            assert_eq!(
                press(&mut view, rule),
                d::Action::Pages(pages::Act::StopAsking(Stop::Open { object: object(1) }))
            );
            page_act(&mut root, pages::Act::StopAsking(Stop::Open { object: object(1) }), &mut Queue::uncapped());
            let mut view = f::browser_page_view(&root);
            let toggle = f::find(view.tree(), Role::Button, rule).unwrap();
            assert!(view.tree().find(toggle).unwrap().states.as_slice().contains(&skein_ui::State::Expanded));
            let offers = super::inbox::subtree(view.tree(), super::inbox::styled(view.tree(), look::WHY_OFFERS));
            let controls: Vec<_> = offers.iter().filter(|n| n.role == Role::Button).map(|n| n.id).collect();
            let expected = if set {
                Vec::from([
                    (Offer::Words(1), "Let agents decide changes to protocol core that are:", true),
                    (
                        Offer::Split(1),
                        "Let agents decide changes to src/**, apart from the rest of protocol core.",
                        false,
                    ),
                ])
            } else {
                Vec::from([(
                    Offer::Decide(Subject::Kind(Kind::Implementation)),
                    "Let agents decide implementation choices, then tell you.",
                    false,
                )])
            };
            assert_eq!(controls.len(), expected.len());
            assert!(!offers.iter().any(|n| n.name.as_deref() == Some(b"Keep asking".as_slice())));
            for (id, (offer, words, disabled)) in controls.iter().zip(expected) {
                assert!(has(&view, words.as_bytes()));
                let node = view.tree().find(*id).unwrap();
                assert_eq!(node.name.as_deref(), Some(if owner { b"Apply".as_slice() } else { b"Propose" }));
                assert_eq!(node.states.as_slice().contains(&skein_ui::State::Disabled), disabled);
                if !disabled {
                    assert_eq!(
                        crate::act(&mut view, *id, Act::Press),
                        Some(d::Action::Pages(pages::Act::StopAsking(if owner {
                            Stop::Apply { offer }
                        } else {
                            Stop::Propose { offer }
                        })))
                    );
                }
            }
            assert_eq!(
                press(&mut view, b"Keep asking"),
                d::Action::Pages(pages::Act::StopAsking(Stop::Keep { object: object(1) }))
            );
            if set {
                let field = f::find(view.tree(), Role::TextField, b"Which changes").unwrap();
                let offers = super::inbox::subtree(view.tree(), super::inbox::styled(view.tree(), look::WHY_OFFERS));
                let sentence = offers
                    .iter()
                    .position(|n| {
                        n.name.as_deref() == Some(b"Let agents decide changes to protocol core that are:".as_slice())
                    })
                    .unwrap();
                let field_at = offers.iter().position(|n| n.id == field).unwrap();
                let act_at = offers.iter().position(|n| n.role == Role::Button).unwrap();
                assert!(sentence < field_at && field_at < act_at);
                assert_eq!(
                    view.tree().find(field).unwrap().placeholder.as_deref(),
                    Some(b"fixes that keep the wire format".as_slice())
                );
                for words in [b"fixes".as_slice(), &[b'x'; 120], &[b'x'; 121], b""] {
                    let action = crate::act(&mut view, field, Act::Input { value: words.into() }).unwrap();
                    assert_eq!(action, d::Action::Pages(pages::Act::StopAsking(Stop::Words { words: words.into() })));
                    event(&mut root, Event::Act { action }, &mut Queue::uncapped());
                    f::build(&mut view, &root);
                    let offers =
                        super::inbox::subtree(view.tree(), super::inbox::styled(view.tree(), look::WHY_OFFERS));
                    let words_act = offers.iter().find(|n| n.role == Role::Button).unwrap();
                    assert_eq!(
                        words_act.states.as_slice().contains(&skein_ui::State::Disabled),
                        words.is_empty() || words.len() > 120
                    );
                }
            }
            if set {
                page_act(
                    &mut root,
                    pages::Act::StopAsking(if owner {
                        Stop::Apply { offer: Offer::Words(1) }
                    } else {
                        Stop::Propose { offer: Offer::Words(1) }
                    }),
                    &mut Queue::uncapped(),
                );
                let view = f::browser_page_view(&root);
                let offers = super::inbox::subtree(view.tree(), super::inbox::styled(view.tree(), look::WHY_OFFERS));
                assert!(offers.iter().any(|n| n.role == Role::Alert), "a narrowed refusal stays under the offers");
            }
            let offer = if set { Offer::Split(1) } else { Offer::Decide(Subject::Kind(Kind::Implementation)) };
            page_act(
                &mut root,
                pages::Act::StopAsking(if owner { Stop::Apply { offer } } else { Stop::Propose { offer } }),
                &mut Queue::uncapped(),
            );
            let view = f::browser_page_view(&root);
            let offers = super::inbox::subtree(view.tree(), super::inbox::styled(view.tree(), look::WHY_OFFERS));
            let line = super::inbox::subtree(view.tree(), super::inbox::styled(view.tree(), look::WHY));
            for control in
                line.iter().chain(offers.iter()).filter(|n| n.role == Role::Button || n.role == Role::TextField)
            {
                assert!(control.states.as_slice().contains(&skein_ui::State::Disabled));
            }
        }
    }
}
