use super::fixtures;
pub(super) use super::fixtures::cards::batch_proposal;
use super::fixtures::cards::{FUNDS, LEFT};
use jig_client::contract as c;
use skein_lib::Queue;
use skein_lib::Writer;
use skein_ui::Builder;
use temper_client_protocol_wire as wire;

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

pub(crate) fn document_proposal() -> c::ReadValue {
    let key = fixtures::chat::PROPOSAL;
    c::ReadValue::Proposal(Box::new({
        let mut funding = List::with_capacity(1);
        funding
            .push(c::FundingOffer {
                source: c::Funder::Pool { project: fixtures::PROJECT, person: 73, period: 1 },
                needed: FUNDS,
                left: Some(LEFT),
                after: Some((LEFT).saturating_sub(FUNDS)),
            })
            .expect("one funding source");
        c::Proposal {
            key,
            project: fixtures::PROJECT,
            projects: List::from_boxed(Box::new([fixtures::PROJECT])),
            reason: c::DocumentText::complete(fixtures::first_run::text(b"It touches sign-in.")),
            action: c::ProposalAction::Batch(List::with_capacity(0)),
            as_holder: true,
            waiting: c::ProposalWaiting { holder: c::HistoryHolder::Person(73), since: skein_lib::Wall::EPOCH },
            funding,
            acts: List::with_capacity(0),
        }
    }))
}

pub(crate) fn settings_page(items: &[(c::Setting, c::RevisionItemEdit)], accept: bool) -> c::ReadValue {
    let c::ReadValue::Proposal(mut p) = batch_proposal() else { panic!("proposal") };
    p.key = SETTINGS_KEY;
    p.project = 0;
    p.projects = List::with_capacity(1);
    p.projects.push(fixtures::PROJECT).unwrap();
    p.reason = c::DocumentText::complete(c::Text::new(SETTINGS_REASON.into(), 128).unwrap());
    p.acts = List::with_capacity(1);
    if accept {
        p.acts.push(c::Offered { act: c::ActKind::Accept, admission: c::Admission::Allowed, effect: None }).unwrap();
    }
    let mut heads = List::with_capacity(u32::try_from(items.len()).unwrap());
    let mut review = List::with_capacity(u32::try_from(items.len()).unwrap());
    for (setting, edit) in items {
        match setting {
            c::Setting::Application { .. } | c::Setting::ProjectPolicy { .. } => {}
            c::Setting::Charter(_)
            | c::Setting::Template(_)
            | c::Setting::Policy(_)
            | c::Setting::Model(_)
            | c::Setting::Rules => {
                heads.push(c::RevisionItemHead { setting: *setting, edit: edit.clone() }).unwrap();
            }
        }
        review
            .push(c::ReviewItem {
                setting: *setting,
                edit: edit.clone(),
                now: c::ReviewValue::None,
                origin: c::SettingOrigin::NotOffered,
                would_be: c::ReviewValue::None,
                application: None,
            })
            .unwrap();
    }
    p.action = c::ProposalAction::Settings(Box::new(c::ProposalActionSettings {
        against: 0,
        items: heads,
        review: Some(c::SettingsReview { key: Some(9001), newest: 0, items: review, cut: false }),
        made: None,
    }));
    c::ReadValue::Proposal(p)
}

pub(crate) fn settings_drive(root: &mut d::Domain, event: Event) -> Queue<d::Request> {
    let mut out = Queue::uncapped();
    d::step(
        root,
        &skein_lib::Env { now: skein_lib::Time::ZERO, wall: fixtures::NOW, limits: fixtures::domain_limits() },
        event,
        &mut out,
    );
    out
}

fn settings_up(root: &mut d::Domain, up: jig_client::Up) -> Queue<d::Request> {
    settings_drive(root, Event::Client { event: jig_client::Event::Up { up } })
}

pub(crate) fn settings_root(page: c::ReadValue) -> (d::Domain, Queue<d::Request>) {
    settings_root_in(page, false)
}

pub(crate) fn settings_root_in(page: c::ReadValue, asking: bool) -> (d::Domain, Queue<d::Request>) {
    let (mut root, me_watch, watch) =
        fixtures::root_at(d::Address::Chat { task: fixtures::chat::TASK }, &fixtures::domain_limits());
    let source = fixtures::chat::messaging_root();
    let mut task = source.pages().chat().unwrap().task(source.client()).unwrap().clone();
    task.chip.task = fixtures::chat::TASK;
    task.project = fixtures::PROJECT;
    let mut me = source.pages().me(source.client()).unwrap().clone();
    if asking {
        me = fixtures::asking_me(me);
        task.chip.executor = c::ChipExecutor::Agent { charter: 37 };
    }
    // Number order, not delivery order, chooses the default way of zero.
    let mut templates = List::with_capacity(5);
    for template in &me.projects.get(0).unwrap().templates {
        templates.push(template.clone()).unwrap();
    }
    for (number, name) in [(8, b"goal for a draft".as_slice()), (4, b"goal".as_slice())] {
        let mut way = templates.get(0).unwrap().clone();
        way.number = number;
        way.name = c::DocumentText::complete(c::Text::new(name.into(), 32).unwrap());
        way.tracked = Some(1);
        templates.push(way).unwrap();
    }
    me.projects.get_mut(0).unwrap().templates = templates;
    fixtures::deliver_me(&mut root, me_watch, me);
    task.proposals = List::with_capacity(1);
    task.proposals.push(SETTINGS_KEY).unwrap();
    let out = settings_up(
        &mut root,
        jig_client::Up::Watched {
            watch: watch.unwrap(),
            event: c::WatchEvent::Delivered(c::Delivery::Task(Box::new(task))),
        },
    );
    let mut reads = List::with_capacity(1);
    for request in &out {
        if let d::Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down: jig_client::Down::Read { read, what: jig_client::Readable::Jig(c::Read::Proposal { .. }) },
                },
        } = request
        {
            reads.push(*read).unwrap();
        }
    }
    assert_eq!(reads.len(), 1);
    let out = settings_up(
        &mut root,
        jig_client::Up::ReadDone {
            read: *reads.get(0).unwrap(),
            result: jig_client::Readout::Jig(c::ReadResult::Value(page)),
        },
    );
    (root, out)
}

pub(crate) fn settings_card(root: &d::Domain) -> d::pages::inbox::Card {
    root.pages().chat().unwrap().cards.get(0).unwrap().clone()
}

pub(crate) fn draw_settings(root: &d::Domain, card: &d::pages::inbox::Card, replaced: bool) -> skein_ui::Tree {
    let mut builder = Builder::new(&fixtures::tree_limits());
    let mut bindings = crate::bindings::Bindings::new(64);
    assert!(crate::cards::proposal::settings(&mut builder, root, card, &fixtures::words(), &mut bindings, replaced));
    builder.finish()
}

pub(crate) fn proposed_values(
    root: &mut d::Domain,
    out: &Queue<d::Request>,
    drafts: bool,
    route: w::QuestionsRoute,
    owned_approval: bool,
) {
    let mut read = None;
    for request in out {
        if let d::Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down: jig_client::Down::Read { read: token, what: jig_client::Readable::Family { record, .. } },
                },
        } = request
            && let w::Read::ProposalItems(_) = wire::decode_reading(&record.bytes, &w::CEILINGS).unwrap().read()
        {
            read = Some(*token);
        }
    }
    let ways =
        w::ProjectWays::new(&w::CEILINGS, w::ProjectWaysParts { drafts, goals: drafts, changes: drafts, way: 0 })
            .unwrap();
    let mut writer = Writer::new(usize::try_from(ways.measure()).unwrap());
    ways.encode(&mut writer).unwrap();
    let questions = w::ProjectQuestions::new(&w::CEILINGS, w::ProjectQuestionsParts { route }).unwrap();
    let mut q = Writer::new(usize::try_from(questions.measure()).unwrap());
    questions.encode(&mut q).unwrap();
    let mut classes = temper_protocol_reaches::Classes::stock(temper_protocol_reaches::Way::AgentsReview);
    classes.sets = Box::new([
        temper_protocol_reaches::PathSet {
            number: 1,
            name: b"vsr's protocol core".as_slice().into(),
            patterns: Box::new([Box::from(b"src/protocol/**".as_slice()), Box::from(b"include/vsr.h".as_slice())]),
        },
        temper_protocol_reaches::PathSet {
            number: 2,
            name: b"the tests".as_slice().into(),
            patterns: Box::new([Box::from(b"tests/**".as_slice())]),
        },
    ]);
    let mut class_items = List::uncapped();
    for item in &classes.items {
        class_items.push(item.clone()).unwrap();
    }
    class_items
        .push(temper_protocol_reaches::Item {
            subject: temper_protocol_reaches::Subject::Set(1),
            words: Box::default(),
            class: temper_protocol_reaches::Class::AskAway,
        })
        .unwrap();
    class_items
        .push(temper_protocol_reaches::Item {
            subject: temper_protocol_reaches::Subject::Kind(temper_protocol_reaches::Kind::Implementation),
            words: b"inside the wasm crate".as_slice().into(),
            class: temper_protocol_reaches::Class::AskAway,
        })
        .unwrap();
    classes.items = class_items.into_boxed();
    let record = temper_protocol_reaches::classes_record(&classes).unwrap();
    let mut classes_value = Writer::new(usize::try_from(record.measure()).unwrap());
    record.encode(&mut classes_value).unwrap();
    let mut items = List::with_capacity(3);
    for (kind, value) in [(7, writer.finish()), (6, q.finish()), (8, classes_value.finish())] {
        items
            .push(
                w::ProposalItem::new(
                    &w::CEILINGS,
                    w::ProposalItemParts {
                        kind,
                        scope: fixtures::PROJECT,
                        number: 1,
                        edit: w::SettingEdit::Set(
                            w::SettingValue::new(&w::CEILINGS, w::SettingValueParts { value }).unwrap(),
                        ),
                    },
                )
                .unwrap(),
            )
            .unwrap();
    }
    let answer =
        w::ProposalItemsAnswer::new(&w::CEILINGS, w::ProposalItemsAnswerParts { items, owned_approval }).unwrap();
    settings_up(
        root,
        jig_client::Up::ReadDone {
            read: read.expect("proposal items read"),
            result: jig_client::Readout::Family {
                family: wire::FORGE,
                record: Some(wire::proposal_items_answer(&answer)),
            },
        },
    );
}
