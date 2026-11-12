use super::fixtures;
use jig_client::{
    contract as c,
    words::{DecisionWord, Word},
};
use skein_ui::{Builder, phrases::Hole};
const FUNDS: u64 = 40;
const LEFT: u64 = 120;
const AFTER: u64 = 80;
#[test]
fn proposal_funding_uses_the_mockups_pool_sentence() {
    let words = fixtures::words();
    assert_eq!(
        words
            .say_word(
                Word::Decision(DecisionWord::Funds),
                &[Hole::Amount(FUNDS), Hole::Amount(LEFT), Hole::Amount(AFTER)]
            )
            .as_ref(),
        b"Funds 40 from your pool \xc2\xb7 120 left, 80 after",
    );
}
#[test]
fn a_result_shows_its_committed_words_in_temper_nodes() {
    let mut builder = Builder::new(&fixtures::tree_limits());
    let result = c::ResultValue::Report(c::DocumentText::complete(
        c::Text::new(Box::from(*b"Ready to review"), fixtures::domain_limits().client.domain.text)
            .expect("report fits"),
    ));
    crate::cards::value(&mut builder, &result, &fixtures::words());
    assert_eq!(fixtures::text(&builder.finish()).as_ref(), b"Ready to review");
}
const OTHER_TASK: u64 = 12;
const ENTRY: u64 = 11;
const POOL: u64 = 7;
const HOLDS: [(c::HoldCategory, &[u8]); 7] = [
    (
        c::HoldCategory::Exceeded(c::Exceeded { bound: c::Bound::Reasoning, value: 2048 }),
        b"Held: the model's reasoning went past its configured allowance of 2048 bytes",
    ),
    (
        c::HoldCategory::Exceeded(c::Exceeded { bound: c::Bound::History, value: 2048 }),
        b"Held: the run's history no longer fits its configured allowance, even compacted",
    ),
    (c::HoldCategory::Circle, b"Held for a circle of waits"),
    (c::HoldCategory::Placed { hold: u64::MAX }, b"Held by a placed hold"),
    (c::HoldCategory::Dependency { task: OTHER_TASK }, b"Dependency task 12"),
    (c::HoldCategory::Uncertain { entry: ENTRY }, b"Uncertain outbox entry 11"),
    (c::HoldCategory::PoolLost { pool: POOL }, b"Allocation lost from pool 7"),
];
#[test]
fn numbered_hold_reasons_keep_the_documents_supplied_identity() {
    for (reason, expected) in HOLDS {
        let mut builder = Builder::new(&fixtures::tree_limits());
        crate::cards::hold(&mut builder, OTHER_TASK, &reason, &fixtures::words());
        assert_eq!(fixtures::text(&builder.finish()).as_ref(), expected);
    }
}

#[test]
fn dependency_refusals_use_jigs_words() {
    for (refusal, expected) in [
        (
            c::Refusal::Circle { tasks: List::with_capacity(0), cut: false },
            b"These tasks would wait for one another".as_slice(),
        ),
        (
            c::Refusal::Dependency { dependency: OTHER_TASK, how: c::TaskStatus::Failed },
            b"The added dependency has ended",
        ),
        (c::Refusal::Kept { pending: List::with_capacity(0) }, b"This answer waits for pending dependencies"),
    ] {
        let mut builder = Builder::new(&fixtures::tree_limits());
        crate::cards::refusal(&mut builder, &fixtures::words(), &refusal);
        assert_eq!(fixtures::text(&builder.finish()).as_ref(), expected);
    }
}

#[test]
fn live_project_refusals_name_tasks_in_cards_and_send_feedback() {
    for (more, expected) in [
        (0, b"This project has live tasks; end them first: T12, T15".as_slice()),
        (2, b"This project has live tasks; end them first: T12, T15, and 2 more".as_slice()),
    ] {
        let mut tasks = List::with_capacity(2);
        tasks.push(12).unwrap();
        tasks.push(15).unwrap();
        let refusal = c::Refusal::LiveTasks { tasks, more };
        let words = fixtures::words();
        assert_eq!(
            words.say_word(Word::refusal(&refusal), &[]).as_ref(),
            b"This project has live tasks; end them first:"
        );
        let mut builder = Builder::new(&fixtures::tree_limits());
        crate::cards::refusal(&mut builder, &words, &refusal);
        assert_eq!(fixtures::text(&builder.finish()).as_ref(), expected);
        let outcome =
            jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Refused { refusal, at: None }));
        let feedback = crate::shared::outcome_feedback(&outcome, &words, false).unwrap();
        assert_eq!(feedback.head.as_ref(), expected);
        assert!(feedback.refused);
        assert!(feedback.after.is_none());
    }
}

pub(super) fn batch_proposal() -> c::ReadValue {
    let mut task = fixtures::task::document();
    task.authority.authority.spend = 6;
    task.authority.authority.delegation.tasks = 1;
    task.authority.authority.delegation.depth = 1;
    task.spec.specification.words =
        c::DocumentText::complete(c::Text::new(Box::from(*b"Ship update\nDetails"), 32).unwrap());
    let mut resources = List::with_capacity(1);
    resources
        .push(c::Part {
            connector: fixtures::FORGE,
            kind: c::PartKind::Resource,
            key: c::PartKey::Path(fixtures::path(&[b"forge", b"github.com", b"robin", b"tally", b"branch", b"main"])),
        })
        .unwrap();
    let mut tasks = List::with_capacity(1);
    tasks
        .push(c::ProposedTask {
            number: 1,
            spec: task.spec,
            executor: c::Executor::Agent { charter: 2 },
            contract: task.result_contract,
            authority: task.authority,
            deadline: c::NewDeadline::Named,
            budget: 6,
            funder: task.budget.funder,
            dependencies: List::with_capacity(0),
            resources,
            wake: task.wake,
            priority: Some(2),
            cut: false,
        })
        .unwrap();
    let root = fixtures::chat::proposal_root();
    let mut page = None;
    for card in root.pages().cards().unwrap() {
        if let Some(value) = &card.page {
            page = Some(value.clone());
            break;
        }
    }
    let mut page = page.unwrap();
    let c::ReadValue::Proposal(proposal) = &mut page else { panic!("proposal") };
    proposal.action = c::ProposalAction::Batch(tasks);
    proposal.funding = List::with_capacity(1);
    proposal
        .funding
        .push(c::FundingOffer {
            source: c::Funder::Pool { project: fixtures::PROJECT, person: 73, period: 1 },
            needed: FUNDS,
            left: Some(LEFT),
            after: Some(AFTER),
        })
        .unwrap();
    page
}

#[test]
fn a_proposals_title_and_labelled_facts_follow_the_mockup() {
    use skein_ui::Role;
    let mut builder = Builder::new(&fixtures::tree_limits());
    crate::cards::proposal(&mut builder, &batch_proposal(), &fixtures::words(), 4);
    let tree = builder.finish();
    assert!(fixtures::find(&tree, Role::Heading { level: 2 }, b"Ship update").is_some());
    assert!(fixtures::find(&tree, Role::Heading { level: 3 }, b"Proposed tasks").is_none());
    let mut labels = List::with_capacity(4);
    for node in tree.nodes() {
        if node.role == Role::Term {
            labels.push(node.name.as_deref().unwrap()).unwrap();
        }
    }
    assert_eq!(labels.as_slice(), [b"Why".as_slice(), b"Goal", b"Landing", b"Budget"]);
    for (label, expected) in [
        (b"Why".as_slice(), b"Make a goal".as_slice()),
        (b"Goal", b"Agent charter 2"),
        (b"Landing", b"branch main of robin/tally on github.com"),
        (b"Budget", b"Funding needed: 40"),
    ] {
        let at = super::chat::named_position(tree.nodes(), Role::Term, Some(label)).unwrap();
        let description = at + usize::try_from(tree.nodes()[at].size).unwrap();
        assert_eq!(tree.nodes()[description].role, Role::Description);
        assert!(skein_lib::bytes::find(&fixtures::node_text(&tree, description), expected).is_some());
    }
    assert!(fixtures::find(&tree, Role::Paragraph, b"Priority 2").is_none());
}

fn document_proposal() -> c::ReadValue {
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

#[test]
fn proposal_facts_are_labelled_joined_and_contained_in_their_rows() {
    use skein_ui::Role;
    let owner = batch_proposal();
    let mut document = document_proposal();
    let c::ReadValue::Proposal(source) = &owner else { panic!("owner proposal") };
    let c::ProposalAction::Batch(tasks) = &source.action else { panic!("batch") };
    let c::ReadValue::Proposal(target) = &mut document else { panic!("document proposal") };
    let mut tasks = tasks.clone();
    let task = tasks.get_mut(0).unwrap();
    task.authority.authority.delegation.tasks = 0;
    task.authority.authority.delegation.depth = 0;
    target.action = c::ProposalAction::Batch(tasks);
    for (page, _delegation, budget) in [
        (
            owner,
            "1 task, depth 1",
            "Funded by pool in project 1, person 73, period 1 · Funding needed: 40 · Funding left: 120 · Funding after: 80",
        ),
        (
            document,
            "0 tasks, depth 0",
            "Funded by pool in project 1, person 73, period 1 · Funding needed: 40 · Funding left: 120 · Funding after: 80",
        ),
    ] {
        let mut builder = Builder::new(&fixtures::tree_limits());
        crate::cards::proposal(&mut builder, &page, &fixtures::words(), 4);
        let tree = builder.finish();
        let mut rows = None;
        for (at, node) in tree.nodes().iter().enumerate() {
            if node.styles.as_slice().contains(&crate::look::PROPOSAL_FACTS) {
                rows = Some(at);
                break;
            }
        }
        let rows = rows.unwrap();
        let goal = alloc::fmt::format(format_args!("yours · Agent charter 2"));
        for (label, expected) in [
            (b"Goal".as_slice(), goal.as_str()),
            (b"Landing", "branch main of robin/tally on github.com · priority 2"),
            (b"Budget", budget),
        ] {
            let at = super::chat::named_position(tree.nodes(), Role::Term, Some(label)).unwrap();
            let description = at + usize::try_from(tree.nodes()[at].size).unwrap();
            assert_eq!(tree.nodes()[description].role, Role::Description);
            assert_eq!(fixtures::node_text(&tree, description).as_ref(), expected.as_bytes(), "row {label:?}");
        }
        assert_eq!(
            rows + usize::try_from(tree.nodes()[rows].size).unwrap(),
            tree.nodes().len(),
            "no loose fact after the rows"
        );
    }
}

#[test]
fn a_non_batch_proposal_keeps_its_deciding_facts() {
    let mut page = batch_proposal();
    let c::ReadValue::Proposal(proposal) = &mut page else { panic!("owner proposal") };
    proposal.action = c::ProposalAction::Release { task: c::TaskLink::Stored { number: 9, status: None }, hold: None };
    let mut builder = Builder::new(&fixtures::tree_limits());
    crate::cards::proposal(&mut builder, &page, &fixtures::words(), 4);
    let tree = builder.finish();
    assert!(skein_lib::bytes::find(&fixtures::text(&tree), b"Make a goal").is_some());
    assert!(skein_lib::bytes::find(&fixtures::text(&tree), b"Make task 9 yours").is_some());
    assert!(fixtures::find(&tree, Role::Term, b"Landing").is_none());
}

#[test]
fn a_budget_held_card_names_the_task_to_amend_and_explains_release() {
    let mut task = fixtures::task::document();
    task.chip.task = OTHER_TASK;
    task.hold = Some(Box::new(c::ChipHold {
        placed: None,
        words: None,
        circle: None,
        account: None,
        source: None,
        reason: c::HoldCategory::Budget,
        revision: 1,
        before: c::PriorPhase::Active(c::RunStage::Idle),
        escalation: None,
        tries: c::FailureTries { transient: 0, permanent: 0, run: 0, agent: 0, lost: 0, invalid: 0, infrastructure: 0 },
        hold_wait_since: None,
    }));
    let mut builder = Builder::new(&fixtures::tree_limits());
    crate::cards::proposal(&mut builder, &c::ReadValue::Task(Box::new(task)), &fixtures::words(), 4);
    let text = fixtures::text(&builder.finish());
    assert!(skein_lib::bytes::find(&text,
        b"Insufficient activation budget. A release alone holds it again: if a proposal to give task 12 more budget waits for you, accept it, then release it; otherwise close it, and start the work again once your pool has room: closing an open chat makes some.").is_some(), "{text:?}");
}

use skein_lib::{List, Queue, Writer};
use skein_ui::Role;
use temper_client_domain::{self as d, Action, Event, pages::Act};
use temper_client_protocol_wire::{self as wire, v1 as w};
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
fn settings_page(items: &[(c::Setting, c::RevisionItemEdit)], accept: bool) -> c::ReadValue {
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
fn settings_drive(root: &mut d::Domain, event: Event) -> Queue<d::Request> {
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
fn settings_root(page: c::ReadValue) -> (d::Domain, Queue<d::Request>) {
    settings_root_in(page, false)
}
fn settings_root_in(page: c::ReadValue, asking: bool) -> (d::Domain, Queue<d::Request>) {
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
fn settings_card(root: &d::Domain) -> d::pages::inbox::Card {
    root.pages().chat().unwrap().cards.get(0).unwrap().clone()
}
fn draw_settings(root: &d::Domain, card: &d::pages::inbox::Card, replaced: bool) -> skein_ui::Tree {
    let mut builder = Builder::new(&fixtures::tree_limits());
    let mut bindings = crate::frame::Bindings::new(64);
    assert!(crate::cards::proposal::settings(&mut builder, root, card, &fixtures::words(), &mut bindings, replaced));
    builder.finish()
}
fn proposed_values(
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
#[expect(clippy::disallowed_methods, reason = "ordinary tests inspect semantic nodes and their item subtrees")]
fn a_settings_proposal_s_reason_draws_its_markdown() {
    let items = [(c::Setting::Charter(c::SettingNumber { number: 2 }), c::RevisionItemEdit::Set)];
    let mut page = settings_page(&items, true);
    let c::ReadValue::Proposal(proposal) = &mut page else { panic!("settings proposal") };
    proposal.reason = c::DocumentText::complete(
        c::Text::new(Box::from(b"## temper enforces\nNo direct pushes.\n\n- one\n- two".as_slice()), 128).unwrap(),
    );
    let (root, _) = settings_root(page);
    let tree = draw_settings(&root, &settings_card(&root), false);
    assert!(fixtures::person_find(&tree, Role::Heading { level: 5 }, b"temper enforces").is_some());
    assert!(fixtures::person_find(&tree, Role::Paragraph, b"No direct pushes.").is_some());
    let reason =
        tree.nodes().iter().find(|n| n.styles.as_slice().contains(&crate::look::SETTINGS_PROPOSAL_REASON)).unwrap();
    assert_eq!(reason.role, Role::Slot);
    let mut item_text = tree.nodes().iter().enumerate().filter(|(_, node)| node.role == Role::Item);
    for expected in [b"one".as_slice(), b"two", b"charter 2"] {
        let (index, _) = item_text.next().expect("reason bullets then setting item");
        assert_eq!(fixtures::node_text(&tree, index).as_ref(), expected);
    }
    assert!(item_text.next().is_none());
    let said = fixtures::text(&tree);
    assert!(skein_lib::bytes::find(&said, b"##").is_none());
    let detail = tree.nodes().iter().find(|n| n.styles.as_slice().contains(&crate::look::SETTINGS_DETAIL)).unwrap();
    assert_eq!(detail.text.get(0).unwrap().text.as_ref(), b"charter 2");
}
#[test]
fn kinds_six_seven_and_eight_are_named_until_their_values_are_read() {
    let items = [
        (c::Setting::Application { kind: 7, scope: fixtures::PROJECT, number: 1 }, c::RevisionItemEdit::Set),
        (c::Setting::Application { kind: 6, scope: fixtures::PROJECT, number: 1 }, c::RevisionItemEdit::Set),
        (c::Setting::Application { kind: 8, scope: fixtures::PROJECT, number: 1 }, c::RevisionItemEdit::Set),
    ];
    for (on, route, expected) in [
        (true, w::QuestionsRoute::Up, b"Questions: asked of me too".as_slice()),
        (false, w::QuestionsRoute::Up, b"Questions: asked of me too"),
        (false, w::QuestionsRoute::None, b"Questions: not asked"),
    ] {
        let (mut root, out) = settings_root(settings_page(&items, true));
        let before = fixtures::text(&draw_settings(&root, &settings_card(&root), false));
        assert!(skein_lib::bytes::find(&before, b"project-ways 1").is_some());
        assert!(skein_lib::bytes::find(&before, b"project-questions 1").is_some());
        assert!(skein_lib::bytes::find(&before, b"project-classes 1").is_some());
        proposed_values(&mut root, &out, on, route, false);
        let tree = draw_settings(&root, &settings_card(&root), false);
        let said = fixtures::text(&tree);
        for expected in [
            expected,
            if on { b"Drafts: on" } else { b"Drafts: off" },
            if on { b"Goals: shown" } else { b"Goals: hidden" },
            if on { b"Changes page: shown" } else { b"Changes page: hidden" },
            b"Way: Task by task",
            b"What reaches you:",
            b"Agents decide, then tell you: wording choices, retries and splits, test-only changes",
            b"Asks you; Away may decide: implementation choices, new dependencies, departures from a landed design",
            "Asks you, even when away: changes to vsr's protocol core, implementation choices under “inside the wasm crate”, changes to the tests".as_bytes(),
            b"vsr's protocol core: src/protocol/**, include/vsr.h",
            b"the tests: tests/**",
        ] {
            assert!(skein_lib::bytes::find(&said, expected).is_some(), "{expected:?}");
        }
        assert!(skein_lib::bytes::find(&said, b"project-ways").is_none());
        assert!(skein_lib::bytes::find(&said, b"project-classes").is_none());
        let mut card = settings_card(&root);
        card.page = Some(c::ReadValue::Ended(Box::new(c::Ended {
            key: card.object,
            at: None,
            by: None,
            value: c::EndedValue::Proposal(c::ProposalHistory {
                number: SETTINGS_KEY.number,
                proposer: fixtures::chat::DELEGATE,
                as_holder: true,
                reason: c::DocumentText::complete(c::Text::new(SETTINGS_REASON.into(), 128).unwrap()),
                state: c::ProposalHistoryState::Accepted { by: c::Requester::Person(73) },
                action: c::RetainedAction::Settings(c::RetainedActionSettings {
                    projects: List::with_capacity(0),
                    against: 0,
                    items: List::with_capacity(0),
                    made: Some(1),
                }),
            }),
        })));
        let said = fixtures::text(&draw_settings(&root, &card, false));
        assert!(
            skein_lib::bytes::find(&said, if on { b"Goals: shown" } else { b"Goals: hidden" }).is_some(),
            "decided cards retain the application values already read"
        );
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
#[test]
fn a_person_who_may_not_accept_sees_who_decides() {
    let (root, _) = settings_root(settings_page(&[], false));
    let tree = draw_settings(&root, &settings_card(&root), false);
    assert!(fixtures::find(&tree, Role::Button, b"Accept").is_none());
    assert!(fixtures::find(&tree, Role::Button, b"Change something").is_none());
    assert!(fixtures::find(&tree, Role::Paragraph, b"Waiting for an owner").is_some());
    assert!(skein_lib::bytes::find(&fixtures::text(&tree), b"Pat").is_some());
}
#[test]
fn an_unread_card_says_no_waiting_before_its_read() {
    let (root, _) = settings_root(settings_page(&[], true));
    let mut card = settings_card(&root);
    card.page = None;
    let mut builder = Builder::new(&fixtures::tree_limits());
    assert!(!crate::cards::proposal::settings(
        &mut builder,
        &root,
        &card,
        &fixtures::words(),
        &mut crate::frame::Bindings::new(64),
        false
    ));
    let tree = builder.finish();
    assert!(fixtures::find(&tree, Role::Paragraph, b"Waiting for an owner").is_none());
}
#[expect(clippy::disallowed_methods, reason = "ordinary tests inspect semantic state lines and styles")]
fn decided_settings(state: c::ProposalHistoryState, choice: c::ProposalChoice, replaced: bool, expected: &str) {
    let (root, _) = settings_root(settings_page(&[], true));
    for ended in [true, false] {
        let mut card = settings_card(&root);
        if ended {
            card.page = Some(c::ReadValue::Ended(Box::new(c::Ended {
                key: card.object,
                value: c::EndedValue::Proposal(c::ProposalHistory {
                    reason: c::DocumentText::complete(
                        c::Text::new(b"review".as_slice().into(), u32::MAX).expect("fixture reason"),
                    ),
                    number: SETTINGS_KEY.number,
                    proposer: fixtures::chat::DELEGATE,
                    as_holder: true,
                    state: state.clone(),
                    action: c::RetainedAction::Settings(c::RetainedActionSettings {
                        projects: List::with_capacity(0),
                        against: 0,
                        items: List::with_capacity(0),
                        made: None,
                    }),
                }),
                at: None,
                by: None,
            })));
        } else {
            card.outcome = Some(jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Done(
                c::Outcome::Document(c::DocumentOutcome::ProposalDecided {
                    proposal: SETTINGS_KEY,
                    by: 73,
                    at: fixtures::NOW,
                    choice,
                    repeated: false,
                }),
            ))));
        }
        let tree = draw_settings(&root, &card, replaced);
        let node = tree
            .nodes()
            .iter()
            .find(|node| node.role == Role::Paragraph && node.name.as_deref() == Some(expected.as_bytes()))
            .expect(expected);
        assert!(node.styles.as_slice().contains(&crate::look::SETTINGS_PROPOSAL_STATE));
        assert!(!node.styles.as_slice().contains(&crate::look::SETTINGS_PROPOSAL_OK));
        assert_eq!(
            tree.nodes().iter().any(|node| node.styles.as_slice().contains(&crate::look::SETTINGS_PROPOSAL_SUPERSEDED)),
            replaced && choice == c::ProposalChoice::Withdrawn
        );
        assert!(fixtures::find(&tree, Role::Paragraph, b"Waiting for an owner").is_none());
        assert!(tree.nodes().iter().all(|node| node.role != Role::Button));
    }
}
#[test]
fn a_rejected_card_says_rejected_and_nothing_changes() {
    decided_settings(
        c::ProposalHistoryState::Rejected {
            by: c::Requester::Person(73),
            reason: c::DocumentText::complete(c::Text::new(b"Keep it".as_slice().into(), 32).unwrap()),
        },
        c::ProposalChoice::Rejected,
        false,
        "Rejected · nothing changes",
    );
}
#[test]
fn a_withdrawn_card_says_withdrawn_and_nothing_changes() {
    decided_settings(
        c::ProposalHistoryState::Withdrawn,
        c::ProposalChoice::Withdrawn,
        false,
        "Withdrawn · nothing changes",
    );
}
#[test]
fn a_withdrawn_card_with_a_later_one_is_replaced() {
    decided_settings(
        c::ProposalHistoryState::Withdrawn,
        c::ProposalChoice::Withdrawn,
        true,
        "Replaced by the proposal below",
    );
}
#[test]
fn a_rejected_card_with_a_later_one_stays_rejected() {
    decided_settings(
        c::ProposalHistoryState::Rejected {
            by: c::Requester::Person(73),
            reason: c::DocumentText::complete(c::Text::new(b"Keep it".as_slice().into(), 32).unwrap()),
        },
        c::ProposalChoice::Rejected,
        true,
        "Rejected · nothing changes",
    );
}
#[test]
fn an_accepted_card_keeps_its_state_line() {
    let (root, _) = settings_root(settings_page(&[], true));
    let mut card = settings_card(&root);
    card.page = Some(c::ReadValue::Ended(Box::new(c::Ended {
        key: card.object,
        value: c::EndedValue::Proposal(c::ProposalHistory {
            reason: c::DocumentText::complete(
                c::Text::new(b"review".as_slice().into(), u32::MAX).expect("fixture reason"),
            ),
            number: SETTINGS_KEY.number,
            proposer: fixtures::chat::DELEGATE,
            as_holder: true,
            state: c::ProposalHistoryState::Accepted { by: c::Requester::Person(73) },
            action: c::RetainedAction::Settings(c::RetainedActionSettings {
                projects: List::with_capacity(0),
                against: 0,
                items: List::with_capacity(0),
                made: None,
            }),
        }),
        at: None,
        by: None,
    })));
    let tree = draw_settings(&root, &card, false);
    assert!(fixtures::find(&tree, Role::Paragraph, "✓ Accepted · new work follows it".as_bytes()).is_some());
    let mut styled = false;
    for node in tree.nodes() {
        styled |= node.styles.as_slice().contains(&crate::look::SETTINGS_PROPOSAL_OK);
    }
    assert!(styled);
    assert!(fixtures::find(&tree, Role::Button, b"Accept").is_none());
    let mut card = settings_card(&root);
    card.outcome = Some(jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Done(
        c::Outcome::SettingsChanged(c::OutcomeSettingsChanged { revision: 1 }),
    ))));
    let tree = draw_settings(&root, &card, false);
    assert!(fixtures::find(&tree, Role::Paragraph, "✓ Accepted · new work follows it".as_bytes()).is_some());
    assert!(fixtures::find(&tree, Role::Button, b"Accept").is_none());
}

#[test]
fn settings_titles_name_the_deployment_or_each_touched_project() {
    let (root, _) = settings_root(settings_page(&[], true));
    let cases: [(&[u32], &[u8]); 4] = [
        (&[], b"How temper will run your projects"),
        (&[fixtures::PROJECT], b"How temper will run Shop"),
        (&[fixtures::PROJECT, 2], b"How temper will run Shop and Second"),
        (&[fixtures::PROJECT, 2, 3], b"How temper will run Shop, Second and 3"),
    ];
    for (projects, title) in cases {
        let mut card = settings_card(&root);
        let c::ReadValue::Proposal(mut proposal) = settings_page(&[], true) else { panic!("proposal") };
        proposal.projects = List::with_capacity(u32::try_from(projects.len()).unwrap());
        for project in projects {
            proposal.projects.push(*project).unwrap();
        }
        card.page = Some(c::ReadValue::Proposal(proposal));
        let tree = draw_settings(&root, &card, false);
        assert!(fixtures::find(&tree, Role::Paragraph, title).is_some(), "{:?}", fixtures::text(&tree));
    }
}
#[test]
fn a_settings_proposal_waiting_on_policy_zero_names_deployment_owners() {
    let c::ReadValue::Proposal(mut proposal) = settings_page(&[], false) else { panic!("proposal") };
    proposal.waiting.holder = c::HistoryHolder::Policy { project: 0, kind: c::MessageProposalKind::Settings };
    let (root, _) = settings_root(c::ReadValue::Proposal(proposal));
    let said = fixtures::text(&draw_settings(&root, &settings_card(&root), false));
    assert!(skein_lib::bytes::find(&said, b"Owners of this deployment").is_some());
    assert!(skein_lib::bytes::find(&said, b"Owners of project 0").is_none());
}

#[test]
fn a_decided_card_draws_from_what_jig_retains() {
    for (state, expected) in [
        (
            c::ProposalHistoryState::Accepted { by: c::Requester::Person(fixtures::chat::PARTY) },
            "✓ Accepted · new work follows it",
        ),
        (
            c::ProposalHistoryState::Rejected {
                by: c::Requester::Person(fixtures::chat::PARTY),
                reason: c::DocumentText::complete(c::Text::new(b"Keep it".as_slice().into(), 128).unwrap()),
            },
            "Rejected · nothing changes",
        ),
        (c::ProposalHistoryState::Withdrawn, "Withdrawn · nothing changes"),
    ] {
        let mut projects = List::with_capacity(1);
        projects.push(fixtures::PROJECT).unwrap();
        let mut items = List::with_capacity(1);
        items
            .push(c::RevisionItemHead {
                setting: c::Setting::Charter(c::SettingNumber { number: 2 }),
                edit: c::RevisionItemEdit::Set,
            })
            .unwrap();
        let (root, _) = settings_root(c::ReadValue::Ended(Box::new(c::Ended {
            key: c::ObjectKey::Proposal(SETTINGS_KEY),
            at: None,
            by: None,
            value: c::EndedValue::Proposal(c::ProposalHistory {
                number: SETTINGS_KEY.number,
                proposer: fixtures::chat::DELEGATE,
                as_holder: true,
                reason: c::DocumentText::complete(
                    c::Text::new(b"## temper enforces\nKeep the settings".as_slice().into(), 128).unwrap(),
                ),
                state,
                action: c::RetainedAction::Settings(c::RetainedActionSettings {
                    projects,
                    against: 1,
                    items,
                    made: None,
                }),
            }),
        })));
        for words in [fixtures::words(), fixtures::browser_words()] {
            let mut builder = Builder::new(&fixtures::tree_limits());
            assert!(crate::cards::proposal::settings(
                &mut builder,
                &root,
                &settings_card(&root),
                &words,
                &mut crate::frame::Bindings::new(64),
                false
            ));
            let tree = builder.finish();
            assert!(fixtures::find(&tree, Role::Paragraph, b"How temper will run Shop").is_some());
            assert!(fixtures::person_find(&tree, Role::Heading { level: 5 }, b"temper enforces").is_some());
            let said = fixtures::text(&tree);
            for expected in [expected.as_bytes(), b"charter 2", b"Keep the settings"] {
                assert!(skein_lib::bytes::find(&said, expected).is_some());
            }
            assert!(fixtures::find(&tree, Role::Paragraph, b"Waiting for an owner").is_none());
            assert!(fixtures::find(&tree, Role::Button, b"Accept").is_none());
        }
    }
}

#[test]
fn an_empty_proposal_has_no_items_list() {
    let (root, _) = settings_root(settings_page(&[], true));
    let tree = draw_settings(&root, &settings_card(&root), false);
    for node in tree.nodes() {
        assert!(!node.styles.as_slice().contains(&crate::look::SETTINGS_PROPOSAL_ITEMS));
    }
}

#[test]
fn a_review_adds_application_and_policy_rows_without_repeating_jigs_heads() {
    let items = [
        (c::Setting::Application { kind: 7, scope: fixtures::PROJECT, number: 1 }, c::RevisionItemEdit::Set),
        (c::Setting::Application { kind: 5, scope: 7, number: 3 }, c::RevisionItemEdit::Removed),
        (c::Setting::ProjectPolicy { project: 2 }, c::RevisionItemEdit::Reset),
        (c::Setting::Charter(c::SettingNumber { number: 2 }), c::RevisionItemEdit::Set),
    ];
    let (mut root, out) = settings_root(settings_page(&items, true));
    proposed_values(&mut root, &out, true, w::QuestionsRoute::None, false);
    let tree = draw_settings(&root, &settings_card(&root), false);
    for expected in
        [b"Goals: shown".as_slice(), b"application kind 5, 7, 3 removed", b"project policy 2 reset", b"charter 2"]
    {
        let mut count = 0_usize;
        for node in tree.nodes() {
            for run in &node.text {
                if run.text.as_ref() == expected {
                    count = count.checked_add(1).unwrap();
                }
            }
        }
        assert_eq!(count, 1, "{expected:?}");
    }
}

#[test]
fn templates_use_the_first_known_name_even_without_tracking() {
    let (root, _) = settings_root(settings_page(&[], true));
    let number = root.pages().me(root.client()).unwrap().projects.get(0).unwrap().templates.get(0).unwrap().number;
    for (number, expected) in
        [(number, b"Chat".as_slice()), (4, b"Task by task"), (8, b"Plan, tasks, land whole"), (40, b"template 40")]
    {
        for (edit, suffix) in [
            (c::RevisionItemEdit::Set, b"".as_slice()),
            (c::RevisionItemEdit::Removed, b" removed"),
            (c::RevisionItemEdit::Reset, b" reset"),
        ] {
            let mut card = settings_card(&root);
            card.page = Some(settings_page(&[(c::Setting::Template(c::SettingNumber { number }), edit)], true));
            let tree = draw_settings(&root, &card, false);
            let mut wanted = Writer::new(expected.len().checked_add(suffix.len()).unwrap());
            wanted.put(expected).unwrap();
            wanted.put(suffix).unwrap();
            let wanted = wanted.finish();
            let mut found = false;
            for node in tree.nodes() {
                for run in &node.text {
                    if run.text.as_ref() == wanted.as_ref() {
                        found = true;
                    }
                }
            }
            assert!(found, "{wanted:?}");
        }
    }
}

#[test]
fn a_refused_accept_ends_the_withdrawn_card() {
    let (root, _) = settings_root(settings_page(&[], true));
    for (replaced, expected) in [(false, "Withdrawn · nothing changes"), (true, "Replaced by the proposal below")] {
        let mut card = settings_card(&root);
        card.outcome = Some(jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Refused {
            refusal: c::Refusal::Unknown,
            at: None,
        })));
        let tree = draw_settings(&root, &card, replaced);
        let mut found = false;
        for node in tree.nodes() {
            if node.role == Role::Paragraph && node.name.as_deref() == Some(expected.as_bytes()) {
                assert!(node.styles.as_slice().contains(&crate::look::SETTINGS_PROPOSAL_STATE));
                found = true;
            }
            assert_ne!(node.role, Role::Button);
        }
        assert!(found, "{expected}");
    }
}

fn catalogue_model() -> c::CatalogueEntry {
    c::CatalogueEntry {
        number: 4,
        model: c::SettingModel {
            provider: 2,
            dialect: 1,
            endpoint: 2,
            program: None,
            name: c::Text::new(b"gpt-6.1-terra".as_slice().into(), 128).unwrap(),
            window: 131_072,
            output: 8192,
            input_price: 3,
            cached_price: 1,
            cache_write_price: None,
            output_price: 12,
            price_unit: 1_000_000,
            efforts: List::with_capacity(0),
            effort: None,
        },
        shown: Some(c::Text::new(b"GPT-6.1 Terra".as_slice().into(), 128).unwrap()),
        description: None,
        picker: true,
        withdrawn: false,
        source: c::ModelSource::Stock,
        estimated: false,
    }
}
fn model_review(value: c::CatalogueEntry) -> c::ReviewValue {
    c::ReviewValue::Value(c::SettingSet { value: c::SettingValue::Model(Box::new(value)) })
}
#[test]
fn a_model_item_reads_as_what_it_changes() {
    for (change, expected) in [
        ("on the picker", "GPT-6.1 Terra: on the picker"),
        ("off the picker", "GPT-6.1 Terra: off the picker"),
        ("new prices", "GPT-6.1 Terra: new prices"),
        ("added", "GPT-6.1 Terra: added"),
        ("withdrawn", "GPT-6.1 Terra: withdrawn"),
        ("new settings", "GPT-6.1 Terra: new settings"),
    ] {
        let mut before = catalogue_model();
        let mut after = before.clone();
        let edit = if change == "withdrawn" { c::RevisionItemEdit::Removed } else { c::RevisionItemEdit::Set };
        match change {
            "on the picker" => before.picker = false,
            "off the picker" => after.picker = false,
            "new prices" => after.model.output_price = 24,
            "new settings" => {
                after.picker = false;
                after.model.output_price = 24;
            }
            "added" | "withdrawn" => {}
            _ => unreachable!("named model change"),
        }
        let setting = c::Setting::Model(c::SettingNumber { number: 4 });
        let mut page = settings_page(&[(setting, edit.clone())], true);
        let c::ReadValue::Proposal(proposal) = &mut page else { panic!("proposal") };
        let c::ProposalAction::Settings(settings) = &mut proposal.action else { panic!("settings") };
        let item = settings.review.as_mut().unwrap().items.get_mut(0).unwrap();
        item.now = if change == "added" { c::ReviewValue::None } else { model_review(before.clone()) };
        item.would_be = if change == "withdrawn" { c::ReviewValue::None } else { model_review(after) };
        let (root, _) = settings_root(page);
        let said = fixtures::text(&draw_settings(&root, &settings_card(&root), false));
        assert!(skein_lib::bytes::find(&said, expected.as_bytes()).is_some(), "missing model change: {change}");
        if change != "added" {
            let previous = if before.picker {
                "Previously: GPT-6.1 Terra: on the picker"
            } else {
                "Previously: GPT-6.1 Terra: off the picker"
            };
            assert!(skein_lib::bytes::find(&said, previous.as_bytes()).is_some());
            assert!(
                skein_lib::bytes::find(&said, b"prices: input 3, cached 1, output 12 per 1000000 tokens").is_some()
            );
        }
    }
}
