use super::fixtures;
use jig_client::contract::{self as c, ActKind, Admission, ObjectKey, Read, ReadValue};
use skein_lib::{List, Wall};
use skein_ui::Role;
use temper_client_domain::{Action, Event, address::Filter, pages::Act};
const PARTY: u64 = 73;
const TEXT_BYTES: u32 = 32;
const SINGLE_OFFER: u32 = 1;
const FUNDING_PERIOD: u64 = 1;
const FUNDING_NEEDED: u64 = 5;
const FUNDING_LEFT: u64 = 20;
const FUNDING_AFTER: u64 = 15;
const EMPTY_BATCH: u32 = 0;
const PROPOSAL_INDEX: u32 = 1;
const PROPOSAL: c::ProposalKey =
    c::ProposalKey { proposer: c::Proposer::Task(fixtures::INBOX_TASK), number: fixtures::INBOX_QUESTION };
fn text(words: &[u8]) -> c::Text {
    c::Text::new(Box::from(words), TEXT_BYTES).expect("fixture text")
}
pub(super) fn root(act: ActKind, admission: Admission) -> temper_client_domain::Domain {
    let mut acts = List::with_capacity(SINGLE_OFFER);
    acts.push(c::Offered { act, admission, effect: None }).expect("one act");
    let mut funding = List::with_capacity(SINGLE_OFFER);
    funding
        .push(c::FundingOffer {
            source: c::Funder::Pool { project: fixtures::PROJECT, person: PARTY, period: FUNDING_PERIOD },
            needed: FUNDING_NEEDED,
            left: Some(FUNDING_LEFT),
            after: Some(FUNDING_AFTER),
        })
        .expect("one funding source");
    let page = c::Proposal {
        key: PROPOSAL,
        project: fixtures::PROJECT,
        projects: List::from_boxed(Box::new([fixtures::PROJECT])),
        reason: c::DocumentText::complete(text(b"Make a goal")),
        action: c::ProposalAction::Batch(List::with_capacity(EMPTY_BATCH)),
        as_holder: true,
        waiting: c::ProposalWaiting { holder: c::HistoryHolder::Person(PARTY), since: Wall::EPOCH },
        funding,
        acts,
    };
    fixtures::inbox_with_document(
        Filter::ALL,
        Some((Read::Proposal { proposal: PROPOSAL }, ReadValue::Proposal(Box::new(page)))),
    )
}
fn press(root: &mut temper_client_domain::Domain, act: Act) {
    fixtures::root_event(root, Event::Act { action: Action::Pages(act) });
}
fn open(root: &mut temper_client_domain::Domain, act: ActKind) {
    press(root, Act::Decide { object: ObjectKey::Proposal(PROPOSAL), revision: None, act });
}
const ACCEPT_WORDS: [&[u8]; 4] = [b"Why", b"Funding needed: 5", b"Funding left: 20", b"Funding after: 15"];
#[test]
fn the_accept_confirmation_says_what_it_grants_costs_and_leaves_in_the_pool() {
    let mut root = root(ActKind::Accept, Admission::Allowed);
    open(&mut root, ActKind::Accept);
    let mut view = fixtures::view();
    let tree = fixtures::build(&mut view, &root);
    assert!(fixtures::find(tree, Role::Dialog, b"Accept this proposal?").is_some());
    let said = fixtures::text(tree);
    for expected in ACCEPT_WORDS {
        assert!(contains(&said, expected), "{expected:?}");
    }
}
#[test]
fn rejecting_says_so_with_its_reason_before_it_is_sent() {
    let mut root = root(ActKind::Reject, Admission::Allowed);
    press(&mut root, Act::CardWords { object: ObjectKey::Proposal(PROPOSAL), words: Box::from(*b"Too costly") });
    open(&mut root, ActKind::Reject);
    let mut view = fixtures::view();
    let tree = fixtures::build(&mut view, &root);
    assert!(fixtures::find(tree, Role::Dialog, b"Reject this proposal?").is_some());
    assert!(contains(&fixtures::text(tree), b"Too costly"));
}
#[test]
fn an_act_only_proposable_says_so_and_to_whom() {
    let root = root(ActKind::Accept, Admission::Proposed { to: c::Recipient::Party(PARTY) });
    let mut view = fixtures::view();
    let tree = fixtures::build(&mut view, &root);
    assert!(fixtures::find(tree, Role::Button, b"Accept").is_none());
    assert!(contains(&fixtures::text(tree), b"This needs a proposal to"));
}

fn contains(text: &[u8], expected: &[u8]) -> bool {
    for words in text.windows(expected.len()) {
        if words == expected {
            return true;
        }
    }
    false
}
// A three-place table, with room at 01:02 after the epoch; wall words use seconds.
const FULL_HELD: u32 = 3;
const FULL_ROOM: Wall = Wall::from_nanos(3_720_000_000_000);
const FULL_WORDS: &[u8] = b"The engine holds its 3 answers of the last day; room again at 3720";
const OUTCOMES: [(jig_client::Outcome, &[u8]); 4] = [
    (jig_client::Outcome::Full { held: FULL_HELD, room_at: FULL_ROOM }, FULL_WORDS),
    (jig_client::Outcome::NotKept, b"The request was not kept."),
    (jig_client::Outcome::Unknown, b"The engine no longer knows this request."),
    (
        jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Refused {
            refusal: c::Refusal::NotOffered,
            at: None,
        })),
        b"This act is not offered",
    ),
];
#[test]
fn the_card_reports_client_outcomes_in_jigs_words() {
    let root = root(ActKind::Accept, Admission::Allowed);
    let card = root.pages().inbox().expect("inbox").cards.get(PROPOSAL_INDEX).expect("proposal");
    let limits = fixtures::tree_limits();
    let words = fixtures::words();
    for (outcome, expected) in OUTCOMES {
        let mut card = card.clone();
        card.outcome = Some(outcome);
        let mut builder = skein_ui::Builder::new(&limits);
        let mut bindings = crate::frame::Bindings::new(limits.bindings);
        crate::confirm::card(
            &mut builder,
            &root,
            &card,
            &words,
            &mut bindings,
            fixtures::domain_limits().client.domain.rows,
        );
        assert!(contains(&fixtures::text(&builder.finish()), expected), "{expected:?}");
    }
}

#[test]
fn the_inbox_keeps_proposal_and_decision_facts_before_its_controls() {
    let view = fixtures::page_view(&root(ActKind::Accept, Admission::Allowed));
    let tree = view.tree();
    let details = super::inbox::styled(tree, crate::look::PROPOSAL_FACTS);
    let facts = super::inbox::subtree(tree, details);
    for expected in ACCEPT_WORDS {
        let mut found = false;
        for node in facts {
            for run in &node.text {
                found |= contains(&run.text, expected);
            }
        }
        assert!(found, "{expected:?}");
    }
    let accept = fixtures::find(tree, Role::Button, b"Accept").expect("accept");
    let mut control = None;
    let mut reason = None;
    for (index, node) in tree.nodes().iter().enumerate() {
        if node.id == accept {
            control = Some(index);
        }
        for run in &node.text {
            if run.text.as_ref() == b"Make a goal" {
                reason = Some(index);
            }
        }
    }
    assert!(control.expect("control") >= details.checked_add(facts.len()).expect("facts end"));
    for node in facts {
        assert_ne!(node.role, Role::Button);
        assert_ne!(node.role, Role::TextArea);
    }

    assert!(reason.expect("reason") > details);
}

#[test]
fn proposal_rows_and_task_release_keep_their_own_shapes() {
    for (root, expected) in [
        (
            Box::new(fixtures::chat::proposal_root()),
            &[
                Role::Paragraph,
                Role::Text,
                Role::Slot,
                Role::Term,
                Role::Text,
                Role::Description,
                Role::Paragraph,
                Role::Text,
                Role::Text,
                Role::Text,
                Role::Term,
                Role::Text,
                Role::Description,
                Role::Paragraph,
                Role::Button,
                Role::Disclosure,
                Role::TextArea,
                Role::Button,
                Role::Link,
                Role::Text,
            ][..],
        ),
        (
            Box::new(super::task::held_agent_root()),
            &[Role::Paragraph, Role::Text, Role::Paragraph, Role::Text, Role::Button][..],
        ),
    ] {
        let mut loaded = None;
        for card in root.pages().cards().expect("cards") {
            if card.page.is_some() {
                loaded = Some(card);
                break;
            }
        }
        let card = loaded.expect("loaded card");
        let limits = fixtures::tree_limits();
        let mut builder = skein_ui::Builder::new(&limits);
        crate::confirm::card(
            &mut builder,
            &root,
            card,
            &fixtures::words(),
            &mut crate::frame::Bindings::new(limits.bindings),
            fixtures::domain_limits().client.domain.rows,
        );
        let tree = builder.finish();
        let mut roles = List::with_capacity(32);
        for node in tree.nodes() {
            roles.push(node.role).expect("flat card shape");
        }
        assert_eq!(roles.as_slice(), expected);
    }
}

#[test]
fn proposal_rejection_starts_in_a_closed_disclosure_in_both_pages() {
    for root in fixtures::pair(fixtures::chat::proposal_root(), root(ActKind::Reject, Admission::Allowed)) {
        let view = fixtures::page_view(&root);
        let nodes = view.tree().nodes();
        let at = super::chat::named_position(nodes, Role::Disclosure, Some(b"Reject with reason"))
            .expect("reject disclosure");
        assert!(!nodes[at].states.as_slice().contains(&skein_ui::State::Expanded));
        let children = super::inbox::subtree(view.tree(), at);
        assert!(super::chat::named_position(children, Role::TextArea, Some(b"Reason")).is_some());
        assert!(super::chat::named_position(children, Role::Button, Some(b"Reject")).is_some());
        assert!(super::chat::named_position(children, Role::Button, Some(b"Accept")).is_none());
    }
}
#[test]
fn a_rejecting_chat_keeps_its_reason_disclosure_open() {
    let view = fixtures::page_view(&fixtures::chat::rejecting_chat());
    let disclosure = fixtures::find(view.tree(), Role::Disclosure, b"Reject with reason").expect("reject disclosure");
    assert!(view.tree().find(disclosure).unwrap().states.as_slice().contains(&skein_ui::State::Expanded));
}
#[test]
fn releases_card_tree_stays_byte_identical() {
    let root = super::task::held_agent_root();
    let mut loaded = None;
    for card in root.pages().cards().unwrap() {
        if card.page.is_some() {
            loaded = Some(card);
            break;
        }
    }
    let card = loaded.unwrap();
    let mut builder = skein_ui::Builder::new(&fixtures::tree_limits());
    crate::confirm::card(
        &mut builder,
        &root,
        card,
        &fixtures::words(),
        &mut crate::frame::Bindings::new(fixtures::tree_limits().bindings),
        2,
    );
    let tree = builder.finish();
    // Test-only diagnostic serialization freezes all of Release's node fields.
    let actual = alloc::fmt::format(format_args!("{:?}", tree.nodes()));
    if let Some(path) = temper_test_env::value(&temper_test_env::RECORD_RELEASE) {
        std::fs::write(path, &actual).unwrap();
        return;
    }
    assert_eq!(actual.as_bytes(), std::fs::read("src/tests/release-card.txt").unwrap());
}

#[test]
fn retrying_full_answers_show_the_count_and_expiry() {
    let mut builder = skein_ui::Builder::new(&fixtures::tree_limits());
    crate::cards::progress(
        &mut builder,
        &fixtures::words(),
        jig_client::Progress::Retrying { why: jig_client::Retry::Full { held: FULL_HELD, room_at: FULL_ROOM } },
    );
    assert!(contains(&fixtures::text(&builder.finish()), FULL_WORDS));
}

fn sheet(root: &temper_client_domain::Domain) -> skein_ui::Tree {
    let mut b = skein_ui::Builder::new(&fixtures::tree_limits());
    crate::confirm::main(
        &mut b,
        root,
        &fixtures::words(),
        &mut crate::frame::Bindings::new(fixtures::tree_limits().bindings),
        fixtures::domain_limits().client.domain.rows,
    );
    b.finish()
}
fn task_sheet(act: ActKind, goal: bool) -> temper_client_domain::Domain {
    let mut task = fixtures::task::document();
    task.chip.task = fixtures::chat::TASK;
    task.priority = if goal { Some(1) } else { None };
    task.chip.executor = c::ChipExecutor::Agent { charter: 1 };
    task.acts = List::with_capacity(1);
    task.acts.push(c::Offered { act, admission: Admission::Allowed, effect: None }).unwrap();
    let mut root = fixtures::task::supplied_root(task, false);
    press(&mut root, Act::Close);
    root
}
fn close_root(stage: c::RunStage, below: u32, proposals: u32, delegates: bool) -> temper_client_domain::Domain {
    let source = fixtures::chat::chat_root();
    let mut task = source.pages().chat().unwrap().task(source.client()).unwrap().clone();
    task.chip.details.phase = c::ChipPhase::Active(stage);
    let Some(c::Effect::Cancel(impact)) = &mut task.acts.get_mut(0).unwrap().effect else { panic!("cancel facts") };
    impact.tasks = c::ImpactCounts {
        waiting: below,
        running: 0,
        held: 0,
        closing: 0,
        ended: 0,
        done: 17,
        failed: 18,
        cancelled: 19,
    };
    impact.proposals = proposals;
    task.delegates = List::with_capacity(2);
    if delegates {
        for (number, title) in [(4, b"Fix the parser".as_slice()), (5, b"Review the parser")] {
            let mut chip = task.chip.clone();
            chip.task = number;
            chip.title = c::DocumentText::complete(c::Text::new(Box::from(title), 32).unwrap());
            task.delegates.push(c::LiveDelegate { chip, takeover_budget: 0 }).unwrap();
        }
    }
    let (mut root, me_watch, watch) =
        fixtures::root_at(temper_client_domain::Address::Chat { task: task.chip.task }, &fixtures::domain_limits());
    let me = source.pages().me(source.client()).unwrap().clone();
    for (watch, delivery) in
        [(me_watch, c::Delivery::Me(Box::new(me))), (watch.unwrap(), c::Delivery::Task(Box::new(task)))]
    {
        fixtures::root_event(
            &mut root,
            Event::Client {
                event: jig_client::Event::Up {
                    up: jig_client::Up::Watched { watch, event: c::WatchEvent::Delivered(delivery) },
                },
            },
        );
    }
    press(&mut root, Act::Close);
    root
}
fn proposal_sheet(act: ActKind) -> temper_client_domain::Domain {
    let mut root = root(act, Admission::Allowed);
    open(&mut root, act);
    root
}
fn release_sheet() -> temper_client_domain::Domain {
    let mut root = super::task::held_agent_root();
    let object = root.pages().cards().unwrap().get(0).unwrap().object;
    press(&mut root, Act::Decide { object, revision: Some(fixtures::task::HOLD_REVISION), act: ActKind::Release });
    root
}
fn restart_sheet() -> temper_client_domain::Domain {
    let mut root = super::document::document_root(
        Some(b"# How we work\n"),
        temper_client_domain::pages::chat::document::Mode::Chat,
        &[],
        super::document::ProposalState::NotProposed,
    );
    press(&mut root, Act::StartOver);
    root
}
type SheetRow = (Box<temper_client_domain::Domain>, &'static [u8], &'static [u8], &'static [u8]);
// One row for each confirmation in client/README.md, 3.6.
const SHEET_ROWS: usize = 7;
const TWO_ACTS: u32 = 2;
fn sheet_rows() -> [SheetRow; SHEET_ROWS] {
    [
        (Box::new(close_root(c::RunStage::Idle, 0, 0, false)), b"Close this chat?", b"Keep it", b"Close chat"),
        (Box::new(restart_sheet()), b"Start over?", b"Keep going", b"Start over"),
        (Box::new(task_sheet(ActKind::Cancel, true)), b"Close this goal?", b"Keep it", b"Close goal"),
        (Box::new(task_sheet(ActKind::Cancel, false)), b"Close this task?", b"Keep it", b"Close task"),
        (Box::new(proposal_sheet(ActKind::Accept)), b"Accept this proposal?", b"Not yet", b"Accept"),
        (Box::new(proposal_sheet(ActKind::Reject)), b"Reject this proposal?", b"Keep it", b"Reject"),
        (Box::new(release_sheet()), b"Release this task?", b"Keep it held", b"Release"),
    ]
}
#[test]
fn each_confirmation_has_its_title_and_acts() {
    for (root, title, keep, does) in sheet_rows() {
        let tree = sheet(&root);
        assert!(fixtures::find(&tree, Role::Dialog, title).is_some(), "{title:?}");
        assert!(fixtures::find(&tree, Role::Heading { level: 2 }, title).is_some(), "{title:?}");
        let acts = super::inbox::subtree(&tree, super::inbox::styled(&tree, crate::look::CONFIRM_ACTS));
        let mut buttons = List::with_capacity(TWO_ACTS);
        for node in acts {
            if node.role == Role::Button {
                buttons.push(node).expect("two sheet acts");
            }
        }
        assert_eq!(buttons.len(), TWO_ACTS);
        assert_eq!(buttons.get(0).unwrap().name.as_deref(), Some(keep));
        assert_eq!(buttons.get(0).unwrap().shortcut, Some(skein_ui::Shortcut::ESCAPE));
        assert_eq!(buttons.get(1).unwrap().name.as_deref(), Some(does));
        assert_eq!(buttons.get(1).unwrap().shortcut, None);
        assert!(buttons.get(1).unwrap().styles.as_slice().contains(&crate::look::CONFIRM_DO));
    }
}
#[test]
fn escape_cancels_a_confirmation_without_sending_its_action() {
    let mut root = close_root(c::RunStage::Idle, 0, 0, false);
    let mut view = fixtures::browser_page_view(&root);
    let keep = fixtures::find(view.tree(), Role::Button, b"Keep it").expect("keep act");
    let mut shortcut = None;
    for node in view.tree().nodes() {
        if node.id == keep {
            shortcut = node.shortcut;
        }
    }
    assert_eq!(shortcut, Some(skein_ui::Shortcut::ESCAPE));
    let action = crate::act(&mut view, keep, skein_ui::Act::Press).expect("Escape's act");
    assert_eq!(action, Action::Pages(Act::Cancel));
    let mut out = skein_lib::Queue::uncapped();
    temper_client_domain::step(
        &mut root,
        &skein_lib::Env { now: skein_lib::Time::ZERO, wall: fixtures::NOW, limits: fixtures::domain_limits() },
        Event::Act { action },
        &mut out,
    );
    assert!(root.pages().confirmation().is_none());
    assert!(out.is_empty(), "cancelling a sheet sends no request");
    assert!(root.pages().chat().unwrap().task(root.client()).is_some());
}
#[test]
fn uncertain_release_choices_have_their_own_styled_group() {
    let source = super::task::held_agent_root();
    let mut task = source.pages().task().unwrap().document(source.client()).unwrap().clone();
    task.acts.get_mut(0).unwrap().effect = Some(c::Effect::Release(c::ReleaseEffect::Uncertain));
    let mut root = fixtures::task::supplied_root(task, true);
    let object = root.pages().cards().unwrap().get(0).unwrap().object;
    press(&mut root, Act::Decide { object, revision: Some(fixtures::task::HOLD_REVISION), act: ActKind::Release });
    let tree = sheet(&root);
    let choices = super::inbox::subtree(&tree, super::inbox::styled(&tree, crate::look::CONFIRM_CHOICES));
    let mut buttons = List::with_capacity(TWO_ACTS);
    for node in choices {
        if node.role == Role::Button {
            buttons.push(node.name.as_deref()).unwrap();
        }
    }
    assert_eq!(
        buttons.as_slice(),
        [Some(b"Repeat the effect".as_slice()), Some(b"Do not repeat the effect".as_slice())]
    );
    let acts = super::inbox::subtree(&tree, super::inbox::styled(&tree, crate::look::CONFIRM_ACTS));
    let mut actions = List::with_capacity(TWO_ACTS);
    for node in acts {
        if node.role == Role::Button {
            actions.push(node.name.as_deref()).unwrap();
        }
    }
    assert_eq!(actions.as_slice(), [Some(b"Keep it held".as_slice()), Some(b"Release".as_slice())]);
}
#[test]
fn a_chat_at_rest_says_only_it_cant_be_reopened() {
    let tree = sheet(&close_root(c::RunStage::Idle, 0, 0, false));
    assert!(contains(&fixtures::text(&tree), "You can still read it, but it can’t be reopened.".as_bytes()));
    assert!(!has_role(&tree, Role::List));
}
#[test]
fn a_chat_at_work_lists_what_closing_changes_in_order() {
    let mut root = close_root(c::RunStage::Running { attempt: 1 }, 4, 2, true);
    press(&mut root, Act::DelegateFate { task: 5, fate: jig_client::shapes::Fate::Move });
    let tree = sheet(&root);
    let text = fixtures::text(&tree);
    let mut remaining = text.as_ref();
    for line in [
        b"The run under way gets 5 seconds to finish, then stops.".as_slice(),
        b"T4 Fix the parser is cancelled.",
        b"T5 Review the parser moves to you.",
        b"2 tasks below it are cancelled.",
        b"2 proposals are withdrawn.",
        b"Changes not yet begun are withdrawn. Those under way settle.",
    ] {
        let Some(at) = skein_lib::bytes::find(remaining, line) else { panic!("missing {line:?}") };
        remaining = remaining.get(at.checked_add(line.len()).unwrap()..).unwrap();
    }
}
#[test]
fn no_count_at_zero_and_one_alone_keeps_the_list() {
    for (below, proposals, expected) in
        [(1, 0, b"1 task below it is cancelled.".as_slice()), (0, 1, b"1 proposal is withdrawn.")]
    {
        let tree = sheet(&close_root(c::RunStage::Idle, below, proposals, false));
        assert!(has_style(&tree, crate::look::CONFIRM_FATES));
        let text = fixtures::text(&tree);
        assert!(contains(&text, expected));
        assert!(!contains(&text, b"0 tasks"));
        assert!(!contains(&text, b"0 proposals"));
    }
}
#[test]
fn a_delegates_row_offers_the_other_fate() {
    let mut root = close_root(c::RunStage::Idle, 2, 0, true);
    let before = sheet(&root);
    assert!(contains(&fixtures::text(&before), b"T4 Fix the parser is cancelled."));
    assert!(fixtures::find(&before, Role::Button, b"Take it instead").is_some());
    press(&mut root, Act::DelegateFate { task: 4, fate: jig_client::shapes::Fate::Move });
    let after = sheet(&root);
    assert!(contains(&fixtures::text(&after), b"T4 Fix the parser moves to you."));
    assert!(fixtures::find(&after, Role::Button, b"Cancel it instead").is_some());
}
fn failed_sheet(root: &mut temper_client_domain::Domain) {
    let mut out = skein_lib::Queue::uncapped();
    temper_client_domain::step(
        root,
        &skein_lib::Env { now: skein_lib::Time::ZERO, wall: fixtures::NOW, limits: fixtures::domain_limits() },
        Event::Act { action: Action::Pages(Act::Confirm) },
        &mut out,
    );
    let mut sent = None;
    while let Some(request) = out.pop() {
        if let temper_client_domain::Request::Unlinked {
            request: jig_client::UnlinkedRequest::Down { down: jig_client::Down::Send { attempt, .. } },
        } = request
        {
            sent = Some(attempt);
        }
    }
    fixtures::root_event(
        root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Answer {
                    attempt: sent.expect("ask sent"),
                    envelope: skein_lib::RequestEnvelope::Final,
                    answer: Some(jig_client::Answered::Jig(c::Answer::Refused {
                        refusal: c::Refusal::NotOffered,
                        at: None,
                    })),
                    why: None,
                },
            },
        },
    );
}
#[test]
fn a_failed_confirmation_says_why_above_its_acts() {
    let mut root = proposal_sheet(ActKind::Accept);
    failed_sheet(&mut root);
    let tree = sheet(&root);
    let mut why = None;
    for (index, node) in tree.nodes().iter().enumerate() {
        for run in &node.text {
            if contains(&run.text, b"This act is not offered") {
                why = Some(index);
            }
        }
    }
    let why = why.expect("the failed ask says why");
    let acts = super::inbox::styled(&tree, crate::look::CONFIRM_ACTS);
    assert!(why < acts);
}
#[test]
fn no_sheet_says_dangerous_or_done_counts() {
    for (root, title, _, _) in sheet_rows() {
        let tree = sheet(&root);
        let text = fixtures::text(&tree);
        for forbidden in [
            b"Confirm".as_slice(),
            b"Dangerous action",
            b"Close chat now",
            b"completed",
            b"Tasks done",
            b"Tasks failed",
            b"Tasks cancelled",
        ] {
            assert!(!contains(&text, forbidden), "{title:?}: {forbidden:?}");
        }
        assert!(fixtures::find(&tree, Role::Button, b"Cancel").is_none());
    }
}
#[test]
fn a_why_arriving_keeps_the_acts_elements() {
    let mut root = restart_sheet();
    let limits = fixtures::tree_limits();
    let mut old = sheet(&root);
    let acts = super::inbox::styled(&old, crate::look::CONFIRM_ACTS);
    assert!(old.nodes()[acts].key.is_some(), "the acts group keeps its own key");
    let mut next = 0;
    let mut patches = skein_lib::Queue::with_capacity(limits.patches);
    skein_ui::diff(&skein_ui::Tree::new(limits.nodes), &mut old, &limits, &mut next, &mut patches);
    while patches.pop().is_some() {}
    failed_sheet(&mut root);
    let mut after = sheet(&root);
    skein_ui::diff(&old, &mut after, &limits, &mut next, &mut patches);
    assert_kept_buttons(&old, &after, &mut patches);
    let mut view = fixtures::page_view(&root);
    let old_buttons = action_elements(view.tree());
    assert!(!has_style(view.tree(), crate::look::DOC));
    press(&mut root, Act::Mode { mode: temper_client_domain::pages::chat::document::Mode::Split });
    crate::build(&mut view, &root, fixtures::NOW, &mut patches);
    while let Some(patch) = patches.pop() {
        if let skein_ui::Patch::Remove { node } = patch {
            assert!(!old_buttons.as_slice().contains(&node), "button ancestor removed");
        }
    }
    assert!(has_style(view.tree(), crate::look::DOC));
    let after = action_elements(view.tree());
    for id in &old_buttons {
        assert!(after.as_slice().contains(id), "button or ancestor lost");
    }
}
fn has_role(tree: &skein_ui::Tree, role: Role) -> bool {
    for node in tree.nodes() {
        if node.role == role {
            return true;
        }
    }
    false
}
fn has_style(tree: &skein_ui::Tree, style: skein_ui::Style) -> bool {
    for node in tree.nodes() {
        if node.styles.as_slice().contains(&style) {
            return true;
        }
    }
    false
}
fn action_elements(tree: &skein_ui::Tree) -> List<skein_ui::NodeId> {
    let acts = super::inbox::styled(tree, crate::look::CONFIRM_ACTS);
    let acts_end = acts.checked_add(usize::try_from(tree.nodes().get(acts).unwrap().size).unwrap()).unwrap();
    let mut elements = List::with_capacity(tree.len());
    for (index, node) in tree.nodes().iter().enumerate() {
        let end = index.checked_add(usize::try_from(node.size).unwrap()).unwrap();
        if index <= acts && end > acts || index > acts && index < acts_end {
            elements.push(node.id).expect("tree bounds button ancestors");
        }
    }
    elements
}
fn assert_kept_buttons(old: &skein_ui::Tree, new: &skein_ui::Tree, patches: &mut skein_lib::Queue<skein_ui::Patch>) {
    let kept = action_elements(old);
    while let Some(patch) = patches.pop() {
        if let skein_ui::Patch::Remove { node } = patch {
            assert!(!kept.as_slice().contains(&node), "button ancestor removed");
        }
    }
    let after = action_elements(new);
    for id in &kept {
        assert!(after.as_slice().contains(id), "button or ancestor lost");
    }
}
