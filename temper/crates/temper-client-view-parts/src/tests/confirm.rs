use super::fixtures;
pub(super) use super::fixtures::confirm::{PROPOSAL, root};
use jig_client::contract::{self as c, ActKind, Admission, ObjectKey};
use skein_lib::{List, Wall};
use skein_ui::Role;
use temper_client_domain::{Action, Event, pages::Act};

const PROPOSAL_INDEX: u32 = 1;

pub(crate) fn press(root: &mut temper_client_domain::Domain, act: Act) {
    fixtures::root_event(root, Event::Act { action: Action::Pages(act) });
}
pub(crate) fn open(root: &mut temper_client_domain::Domain, act: ActKind) {
    press(root, Act::Decide { object: ObjectKey::Proposal(PROPOSAL), revision: None, act });
}

pub(crate) fn contains(text: &[u8], expected: &[u8]) -> bool {
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
        let mut bindings = crate::bindings::Bindings::new(limits.bindings);
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
            Box::new(fixtures::task_helpers::held_agent_root()),
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
            &mut crate::bindings::Bindings::new(limits.bindings),
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
fn releases_card_tree_stays_byte_identical() {
    let root = fixtures::task_helpers::held_agent_root();
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
        &mut crate::bindings::Bindings::new(fixtures::tree_limits().bindings),
        2,
    );
    let tree = builder.finish();
    // Test-only diagnostic serialization freezes all of Release's node fields.
    let actual = alloc::fmt::format(format_args!("{:?}", tree.nodes()));
    if let Some(path) = temper_test_env::value(&temper_test_env::RECORD_RELEASE) {
        std::fs::write(path, &actual).unwrap();
        return;
    }
    assert_eq!(actual.as_bytes(), std::fs::read("src/parts_tests/release-card.txt").unwrap());
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

pub(crate) fn sheet(root: &temper_client_domain::Domain) -> skein_ui::Tree {
    let mut b = skein_ui::Builder::new(&fixtures::tree_limits());
    crate::confirm::main(
        &mut b,
        root,
        &fixtures::words(),
        &mut crate::bindings::Bindings::new(fixtures::tree_limits().bindings),
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
pub(crate) fn close_root(
    stage: c::RunStage,
    below: u32,
    proposals: u32,
    delegates: bool,
) -> temper_client_domain::Domain {
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
    let mut root = fixtures::task_helpers::held_agent_root();
    let object = root.pages().cards().unwrap().get(0).unwrap().object;
    press(&mut root, Act::Decide { object, revision: Some(fixtures::task::HOLD_REVISION), act: ActKind::Release });
    root
}
pub(crate) fn restart_sheet() -> temper_client_domain::Domain {
    let mut root = fixtures::document::document_root(
        Some(b"# How we work\n"),
        temper_client_domain::pages::chat::document::Mode::Chat,
        &[],
        fixtures::document::ProposalState::NotProposed,
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
        let acts = fixtures::inbox::subtree(&tree, fixtures::inbox::styled(&tree, crate::look::CONFIRM_ACTS));
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
fn uncertain_release_choices_have_their_own_styled_group() {
    let source = fixtures::task_helpers::held_agent_root();
    let mut task = source.pages().task().unwrap().document(source.client()).unwrap().clone();
    task.acts.get_mut(0).unwrap().effect = Some(c::Effect::Release(c::ReleaseEffect::Uncertain));
    let mut root = fixtures::task::supplied_root(task, true);
    let object = root.pages().cards().unwrap().get(0).unwrap().object;
    press(&mut root, Act::Decide { object, revision: Some(fixtures::task::HOLD_REVISION), act: ActKind::Release });
    let tree = sheet(&root);
    let choices = fixtures::inbox::subtree(&tree, fixtures::inbox::styled(&tree, crate::look::CONFIRM_CHOICES));
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
    let acts = fixtures::inbox::subtree(&tree, fixtures::inbox::styled(&tree, crate::look::CONFIRM_ACTS));
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
pub(crate) fn failed_sheet(root: &mut temper_client_domain::Domain) {
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
    let acts = fixtures::inbox::styled(&tree, crate::look::CONFIRM_ACTS);
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

fn has_role(tree: &skein_ui::Tree, role: Role) -> bool {
    for node in tree.nodes() {
        if node.role == role {
            return true;
        }
    }
    false
}
pub(crate) fn has_style(tree: &skein_ui::Tree, style: skein_ui::Style) -> bool {
    for node in tree.nodes() {
        if node.styles.as_slice().contains(&style) {
            return true;
        }
    }
    false
}
