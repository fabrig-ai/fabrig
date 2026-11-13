//! The chat's document in both shells (temper/docs/design/client/pages/README.md).
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    reason = "ordinary tests inspect semantic trees and prints"
)]
use super::{documents, fixtures as f};
use crate::{look, words::Shell};
use jig_client::contract as c;
use skein_lib::{Env, List, Queue, Time, Wall};
use skein_ui::{Node, Role, Run, State, Style, Tree};
use temper_client_domain::{
    self as d,
    pages::{Act, chat::document::Mode},
};

const BODY: &[u8] =
    b"# How Shop works\n\nDraft, with the user.\n\n## Starts\n\nWrite a draft.\n\n## Lands\n\nCheck and land.\n";
const TITLE: &[u8] = b"How Shop works";
const TASK: u64 = f::chat::TASK;
const LONG: u32 = 65_536;
const FIRST: &str = "Note 1 revision 1 written · 12 lines";
const LATER: &str = "Note 1 revision 2 written · +1 −1 · line 11";

fn limits() -> d::Limits {
    let mut limits = f::domain_limits();
    limits.client.domain.long_text = LONG;
    limits.client.domain.ask_bytes = 131_072;
    limits.client.domain.requests.bytes = 524_288;
    limits.client.domain.text = 512;
    limits.client.domain.turn_text = 2048;
    limits.client.domain.rows = 4;
    limits
}
fn activity(outcomes: &[&str]) -> c::ActivityPage {
    let mut rows = List::with_capacity(1);
    if !outcomes.is_empty() {
        let mut blocks = List::with_capacity(u32::try_from(outcomes.len()).unwrap());
        for (index, outcome) in outcomes.iter().enumerate() {
            blocks
                .push(c::Block::Call(c::Call {
                    number: u64::try_from(index).unwrap().checked_add(1).unwrap(),
                    tool: c::Text::new(Box::from(b"note".as_slice()), 512).unwrap(),
                    input: c::Text::new(Box::default(), 512).unwrap(),
                    outcome: Some(c::Text::new(Box::from(outcome.as_bytes()), 512).unwrap()),
                    failed: false,
                    objects: List::with_capacity(0),
                    parts: List::with_capacity(0),
                }))
                .unwrap();
        }
        let mut messages = List::with_capacity(1);
        messages.push(c::Message { speaker: c::Speaker::Assistant, blocks }).unwrap();
        rows.push(documents::turn(
            1,
            Wall::EPOCH,
            c::TurnName { attempt: 1, number: 1 },
            c::TurnRead::Decoded(c::Turn {
                messages,
                used: c::Usage { input: 0, output: 0, cached: 0, cache_written: 0 },
                spent: 0,
                read_through: None,
                llm: None,
                cut: false,
            }),
        ))
        .unwrap();
    }
    c::ActivityPage { rows, before: None, after: None, cut: false }
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ProposalState {
    Ordinary,
    DesignSession,
    DraftOffered,
    EndedDraftOffered,
    UnsendableDraftOffered,
    NotProposed,
    Waiting,
    Accepted,
    Unknown,
}
fn root(body: Option<&[u8]>, mode: Mode, outcomes: &[&str]) -> d::Domain {
    document_root(body, mode, outcomes, ProposalState::Ordinary)
}
const PROPOSAL: c::ProposalKey = c::ProposalKey { proposer: c::Proposer::Task(TASK), number: 12 };
const ACCEPTED_AT: Wall = Wall::from_nanos(50_580_000_000_000);
pub(super) fn document_root(body: Option<&[u8]>, mode: Mode, outcomes: &[&str], state: ProposalState) -> d::Domain {
    activity_root(body, mode, activity(outcomes), state)
}
#[expect(
    clippy::too_many_lines,
    reason = "the fixture pumps the task, document and departed proposal through their public events"
)]
fn activity_root(body: Option<&[u8]>, mode: Mode, activity: c::ActivityPage, state: ProposalState) -> d::Domain {
    let limits = limits();
    let drafts = state == ProposalState::DraftOffered
        || state == ProposalState::EndedDraftOffered
        || state == ProposalState::UnsendableDraftOffered;
    let ways = d::forge::ways::Ways { drafts, ..d::forge::ways::Ways::STOCK };
    let (mut root, me_watch, watch, _, _, _, _, _, _) = f::root_at_with_values(
        d::Address::Chat { task: TASK },
        &limits,
        b"Shop",
        None,
        Some(activity),
        None,
        Some(&[]),
        Some(ways),
        false,
        true,
    );
    let source = f::chat::messaging_root();
    let mut task = source.pages().chat().unwrap().task(source.client()).unwrap().clone();
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let mut out = Queue::uncapped();
    if state != ProposalState::Ordinary {
        let mut me = asking_me(&source);
        if state == ProposalState::DesignSession {
            let template = me.projects.get_mut(0).unwrap().templates.get_mut(1).unwrap();
            template.number = 6;
            template.executor = c::TemplateExecutor::Charter(12);
        }
        if state == ProposalState::DraftOffered
            || state == ProposalState::EndedDraftOffered
            || state == ProposalState::UnsendableDraftOffered
        {
            let template = me.projects.get_mut(0).unwrap().templates.get_mut(1).unwrap();
            template.tracked = Some(1);
            template.flags.conversation = false;
        }
        d::step(
            &mut root,
            &env,
            d::Event::Client {
                event: jig_client::Event::Up {
                    up: jig_client::Up::Watched {
                        watch: me_watch,
                        event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me))),
                    },
                },
            },
            &mut out,
        );
        if state != ProposalState::DraftOffered
            && state != ProposalState::EndedDraftOffered
            && state != ProposalState::UnsendableDraftOffered
        {
            let charter = if state == ProposalState::DesignSession { 12 } else { 1 };
            task.chip.executor = c::ChipExecutor::Agent { charter };
        }
        if state == ProposalState::UnsendableDraftOffered {
            task.acts.clear();
        }
        if state == ProposalState::EndedDraftOffered {
            task.chip.phase = c::Phase::Ended;
        }
    }
    if state == ProposalState::Waiting || state == ProposalState::Accepted || state == ProposalState::Unknown {
        task.proposals = List::with_capacity(1);
        task.proposals.push(PROPOSAL).unwrap();
    }
    let mut without_proposal = task.clone();
    without_proposal.proposals.clear();
    d::step(
        &mut root,
        &env,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: watch.unwrap(),
                    event: c::WatchEvent::Delivered(c::Delivery::Task(Box::new(task))),
                },
            },
        },
        &mut out,
    );
    let mut departed = false;
    for _ in 0_u32..64 {
        let Some(request) = out.pop() else {
            if !departed && (state == ProposalState::Accepted || state == ProposalState::Unknown) {
                departed = true;
                d::step(
                    &mut root,
                    &env,
                    d::Event::Client {
                        event: jig_client::Event::Up {
                            up: jig_client::Up::Watched {
                                watch: watch.unwrap(),
                                event: c::WatchEvent::Delivered(c::Delivery::Task(Box::new(without_proposal.clone()))),
                            },
                        },
                    },
                    &mut out,
                );
                continue;
            }
            break;
        };
        if let d::Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down: jig_client::Down::Read { read, what: jig_client::Readable::Jig(c::Read::Proposal { .. }) },
                },
        } = &request
        {
            let result = proposal_answer(state, departed);
            d::step(
                &mut root,
                &env,
                d::Event::Client {
                    event: jig_client::Event::Up {
                        up: jig_client::Up::ReadDone { read: *read, result: jig_client::Readout::Jig(result) },
                    },
                },
                &mut out,
            );
        }
        if let d::Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down: jig_client::Down::Read { read, what: jig_client::Readable::Jig(c::Read::Notes { scope, .. }) },
                },
        } = request
        {
            let mut rows = List::with_capacity(1);
            if let Some(body) = body {
                rows.push(c::Note {
                    number: 1,
                    revision: c::Revision { value: 2 },
                    scope,
                    description: c::DocumentText::complete(c::Text::new(Box::from(TITLE), 512).unwrap()),
                    body: c::DocumentText::complete(c::Text::new(Box::from(body), LONG).unwrap()),
                    writer: c::NoteAuthor::Task { task: TASK, attempt: 1 },
                    tasks: List::with_capacity(0),
                    changed: Some(Wall::EPOCH),
                    acts: List::with_capacity(0),
                    cut: false,
                })
                .unwrap();
            }
            d::step(
                &mut root,
                &env,
                d::Event::Client {
                    event: jig_client::Event::Up {
                        up: jig_client::Up::ReadDone {
                            read,
                            result: jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Notes(Box::new(
                                c::NotePage { rows, before: None, after: None, cut: false },
                            )))),
                        },
                    },
                },
                &mut out,
            );
        }
    }
    assert_eq!(root.pages().chat().unwrap().draft.is_some(), body.is_some());
    d::step(&mut root, &env, d::Event::Act { action: d::Action::Pages(Act::Mode { mode }) }, &mut out);
    root
}
fn asking_me(source: &d::Domain) -> c::Me {
    let mut me = source.pages().me(source.client()).unwrap().clone();
    let member = me.projects.get_mut(0).unwrap();
    let mut template = member.templates.get(0).unwrap().clone();
    template.number = 5;
    let mut templates = List::with_capacity(2);
    templates.push(member.templates.get(0).unwrap().clone()).unwrap();
    templates.push(template).unwrap();
    member.templates = templates;

    me
}
fn proposal_answer(state: ProposalState, departed: bool) -> c::ReadResult {
    if departed {
        if state == ProposalState::Unknown {
            c::ReadResult::None(c::ReadNone::Unknown)
        } else {
            c::ReadResult::Value(c::ReadValue::Ended(Box::new(c::Ended {
                key: c::ObjectKey::Proposal(PROPOSAL),
                value: c::EndedValue::Proposal(c::ProposalHistory {
                    reason: c::DocumentText::complete(
                        c::Text::new(b"review".as_slice().into(), u32::MAX).expect("fixture reason"),
                    ),
                    action: c::RetainedAction::Batch(List::with_capacity(0)),
                    number: PROPOSAL.number,
                    proposer: TASK,
                    as_holder: true,
                    state: c::ProposalHistoryState::Accepted { by: c::Requester::Person(f::chat::PARTY) },
                }),
                at: Some(ACCEPTED_AT),
                by: None,
            })))
        }
    } else {
        c::ReadResult::Value(c::ReadValue::Proposal(Box::new(c::Proposal {
            key: PROPOSAL,
            project: 1,
            projects: List::from_boxed(Box::new([1])),
            reason: c::DocumentText::complete(c::Text::new(Box::from(b"Work this way".as_slice()), 512).unwrap()),
            action: c::ProposalAction::Batch(List::with_capacity(0)),
            as_holder: true,
            waiting: c::ProposalWaiting { holder: c::HistoryHolder::Person(f::chat::PARTY), since: Wall::EPOCH },
            funding: List::with_capacity(0),
            acts: List::with_capacity(0),
        })))
    }
}
fn view(root: &d::Domain, shell: Shell) -> crate::View {
    let words = crate::words::Words::english(f::FORGE, 512, shell).unwrap();
    let limits = limits();
    let bounds = crate::Limits::of(&limits, &words).unwrap();
    let mut view = crate::View::new(&bounds, &limits, words, f::FORGE);
    f::build(&mut view, root);
    view
}
fn styled(tree: &Tree, style: Style) -> Vec<&Node> {
    tree.nodes().iter().filter(|node| node.styles.as_slice().contains(&style)).collect()
}
fn section_words(tree: &Tree, style: Style) -> Vec<u8> {
    let node = styled(tree, style)[0];
    let at = tree.nodes().iter().position(|n| n.id == node.id).unwrap();
    f::node_text(tree, at).into_vec()
}

#[test]
fn no_document_no_group_no_pane() {
    for mode in [Mode::Chat, Mode::Split, Mode::Draft] {
        let view = view(&root(None, mode, &[]), Shell::Browser);
        assert_eq!(styled(view.tree(), look::SEG).len(), 0);
        assert_eq!(styled(view.tree(), look::DOC).len(), 0);
    }
}
#[test]
fn the_group_presses_its_mode() {
    for (mode, name, next) in [
        (Mode::Chat, b"Chat".as_slice(), b"Split".as_slice()),
        (Mode::Split, b"Split", b"Draft"),
        (Mode::Draft, b"Draft", b"Chat"),
    ] {
        let root = root(Some(BODY), mode, &[]);
        let view = view(&root, Shell::Browser);
        let group = styled(view.tree(), look::SEG)[0];
        assert_eq!(group.role, Role::Group);
        assert_eq!(group.name.as_deref(), Some(b"The draft".as_slice()));
        let buttons: Vec<_> = view
            .tree()
            .nodes()
            .iter()
            .filter(|node| {
                node.role == Role::Button
                    && [b"Chat".as_slice(), b"Split", b"Draft"].contains(&node.name.as_deref().unwrap_or_default())
            })
            .collect();
        assert_eq!(buttons.len(), 3);
        for button in buttons {
            assert_eq!(button.states.as_slice().contains(&State::Pressed), button.name.as_deref() == Some(name));
            assert_eq!(button.shortcut.is_some(), button.name.as_deref() == Some(next));
        }
        let split = f::find(view.tree(), Role::Button, b"Split").unwrap();
        let Some(d::Event::Act { action: d::Action::Pages(Act::Mode { mode }) }) =
            crate::event(&view, split, skein_ui::Act::Press)
        else {
            unreachable!("a mode button sends its page act")
        };
        assert_eq!(mode, Mode::Split);
    }
}
#[test]
fn chat_draws_no_pane() {
    let view = view(&root(Some(BODY), Mode::Chat, &[]), Shell::Browser);
    assert_eq!(styled(view.tree(), look::SEG).len(), 1);
    assert_eq!(styled(view.tree(), look::DOC).len(), 0);
    assert!(!f::text(view.tree()).windows(b"Write a draft.".len()).any(|w| w == b"Write a draft."));
}
#[test]
fn split_draws_the_pane_after_the_conversation() {
    for (state, opening, run, words) in [
        (ProposalState::Ordinary, "Draft, with the user.", Run::Plain, b"Draft, with the user.".as_slice()),
        (ProposalState::Ordinary, "_Draft, with the user._", Run::Emphasis, b"Draft, with the user."),
        (ProposalState::Ordinary, "**x**", Run::Strong, b"x"),
        (ProposalState::Ordinary, "`x`", Run::Code, b"x"),
        (ProposalState::DesignSession, "_Draft, with the user._", Run::Emphasis, b"Draft, with the user."),
    ] {
        let body = String::from_utf8_lossy(BODY).replace("Draft, with the user.", opening);
        let view = view(&document_root(Some(body.as_bytes()), Mode::Split, &[], state), Shell::Browser);
        let nodes = view.tree().nodes();
        let convo = nodes.iter().position(|n| n.styles.as_slice().contains(&look::CONVO)).unwrap();
        let pane = nodes.iter().position(|n| n.styles.as_slice().contains(&look::DOC)).unwrap();
        assert_eq!(pane, convo + usize::try_from(nodes[convo].size).unwrap());
        assert_eq!(nodes[pane].role, Role::Region);
        assert_eq!(nodes[pane].name.as_deref(), Some(TITLE));
        assert_eq!(styled(view.tree(), look::MODE_SPLIT).len(), 1);
        let split = f::find(view.tree(), Role::Button, b"Split").unwrap();
        assert!(view.tree().find(split).unwrap().states.as_slice().contains(&State::Pressed));
        let muted = styled(view.tree(), look::DOC_MUTED)[0];
        assert_eq!(muted.role, Role::Slot);
        let at = nodes.iter().position(|node| node.id == muted.id).unwrap();
        let paragraph = &nodes[at + 1];
        assert_eq!(paragraph.role, Role::Paragraph);
        assert_eq!(muted.size, paragraph.size + 1);
        let runs: Vec<_> = nodes[at + 1..at + usize::try_from(muted.size).unwrap()]
            .iter()
            .flat_map(|node| node.text.iter())
            .map(|text| (text.run, text.text.as_ref()))
            .collect();
        assert_eq!(runs, [(run, words)], "opening: {opening}");
        let path = section_words(view.tree(), look::DOC_PATH);
        assert!(path.starts_with(b"docs/design/draft/how-shop-works.md"));
        assert!(path.ends_with(b"edited just now"));
    }
}
#[test]
fn sections_are_keyed_by_heading() {
    let before = view(&root(Some(b"# Title\n\n## Same\n\na\n\n## Same\n\nb\n"), Mode::Split, &[]), Shell::Browser);
    let after = view(
        &root(Some(b"# Title\n\n## New\n\nx\n\n## Same\n\na\n\n## Same\n\nb\n"), Mode::Split, &[]),
        Shell::Browser,
    );
    let a: Vec<_> = styled(before.tree(), look::DOC_SEC).iter().map(|n| n.key.unwrap()).collect();
    let b: Vec<_> = styled(after.tree(), look::DOC_SEC).iter().map(|n| n.key.unwrap()).collect();
    assert_eq!(a[1], b[2]);
    assert_eq!(a[2], b[3]);
    assert_eq!(a[1].a, a[2].a);
    assert_eq!(a[1].b, 0);
    assert_eq!(a[2].b, 1);
    assert_ne!(a[0].a, a[1].a);
}
#[test]
fn a_change_keeps_the_other_sections_nodes() {
    let mut view = view(&root(Some(BODY), Mode::Split, &[]), Shell::Browser);
    let before: Vec<_> = styled(view.tree(), look::DOC_SEC).iter().map(|n| n.id).collect();
    let mut changed = root(Some(b"# How Shop works\n\nDraft, with the user.\n\n## Inserted\n\nNew.\n\n## Starts\n\nDifferent prose.\n\n## Lands\n\nCheck and land.\n"), Mode::Split, &[]);
    f::root_event(
        &mut changed,
        d::Event::Act { action: d::Action::Pages(Act::MessageWords { words: Box::from(b"reply".as_slice()) }) },
    );
    f::build(&mut view, &changed);
    let after = styled(view.tree(), look::DOC_SEC);
    assert_eq!(after.len(), 4);
    assert_eq!(before[0], after[0].id);
    assert_eq!(before[1], after[2].id);
    assert_eq!(before[2], after[3].id);
}
#[test]
fn the_tint_follows_the_newest_write() {
    let view = view(&root(Some(BODY), Mode::Split, &[FIRST, LATER]), Shell::Browser);
    let changed = String::from_utf8(section_words(view.tree(), look::DOC_CHANGED)).unwrap();
    assert!(changed.starts_with("Lands"));
    assert!(changed.ends_with("Changed by T9 just now"));
    assert_eq!(styled(view.tree(), look::DOC_CHANGED).len(), 1);
    assert_eq!(styled(view.tree(), look::DOC_NOTE).len(), 1);
}
#[test]
fn a_first_write_tints_the_first_second_level_section() {
    let view = view(&root(Some(BODY), Mode::Split, &[FIRST]), Shell::Browser);
    let changed = String::from_utf8(section_words(view.tree(), look::DOC_CHANGED)).unwrap();
    assert!(changed.starts_with("Starts"));
    assert!(changed.ends_with("Added by T9 just now"));
    assert_eq!(styled(view.tree(), look::DOC_CHANGED).len(), 1);
}
#[test]
fn the_terminal_prints_the_document_after_the_conversation() {
    for mode in [Mode::Split, Mode::Draft] {
        for (opening, words, line) in [
            ("Draft, with the user.", "Draft, with the user.", "Draft, with the user."),
            ("_Draft, with the user._", "Draft, with the user.", "Draft, with the user."),
            ("**x**", "x", "x"),
            ("`x`", "x", "`x`"),
        ] {
            let body = String::from_utf8_lossy(BODY).replace("Draft, with the user.", opening);
            let view = view(&root(Some(body.as_bytes()), mode, &[]), Shell::Terminal);
            let mut print = skein_ui_text::Print::new(&view.limits().tree).unwrap();
            skein_ui_text::render(view.tree(), b"temper", None, &mut print);
            let printed = String::from_utf8(print.text.as_slice().to_vec()).unwrap();
            let conversation = printed.find("Conversation").unwrap();
            let document = printed.find("How Shop works").unwrap();
            assert!(conversation < document, "{printed}");
            assert!(printed.contains("Check and land."));
            assert!(printed.contains("how-shop-works.md · edited just now"));
            assert_eq!(section_words(view.tree(), look::DOC_MUTED), words.as_bytes());
            assert!(printed.lines().any(|printed| printed.trim() == line), "{printed}");
            assert!(f::find(view.tree(), Role::Region, TITLE).is_some());
            assert_eq!(styled(view.tree(), look::RAIL_UNFOLD).len(), 0);
        }
    }
}
#[test]
fn a_full_document_fits_the_views_limits() {
    let short = b"x\n\n".repeat(21_845);
    let mut body = short;
    body.push(b'x');
    assert_eq!(body.len(), usize::try_from(LONG).unwrap());
    for body in [body, vec![b'x'; usize::try_from(LONG).unwrap()], b"## h\n".repeat(13_107)] {
        let view = view(&root(Some(&body), Mode::Split, &[]), Shell::Browser);
        assert_eq!(styled(view.tree(), look::DOC).len(), 1);
        let count: usize = view.tree().nodes().iter().flat_map(|node| node.text.iter()).map(|run| run.text.len()).sum();
        assert!(count >= body.iter().filter(|b| **b == b'x' || **b == b'h').count());
    }
}

#[test]
fn the_line_says_not_proposed_yet() {
    let root = document_root(Some(BODY), Mode::Split, &[], ProposalState::NotProposed);
    let view = view(&root, Shell::Browser);
    let line = String::from_utf8(section_words(view.tree(), look::DOC_PATH)).unwrap();
    assert_eq!(line, "T9 · not proposed yetedited just now");
}
#[test]
fn the_line_says_it_waits() {
    let root = document_root(Some(BODY), Mode::Split, &[], ProposalState::Waiting);
    let view = view(&root, Shell::Browser);
    let line = String::from_utf8(section_words(view.tree(), look::DOC_PATH)).unwrap();
    assert_eq!(line, "Proposal 12 · waits for an owneredited just now");
}
#[test]
fn the_line_says_accepted() {
    let root = document_root(Some(BODY), Mode::Split, &[], ProposalState::Accepted);
    let view = view(&root, Shell::Browser);
    let line = String::from_utf8(section_words(view.tree(), look::DOC_PATH)).unwrap();
    assert_eq!(line, "Accepted at 14:03edited just now");
}
#[test]
fn an_unknown_decision_says_decided_without_a_time() {
    let root = document_root(Some(BODY), Mode::Split, &[], ProposalState::Unknown);
    let view = view(&root, Shell::Browser);
    let line = String::from_utf8(section_words(view.tree(), look::DOC_PATH)).unwrap();
    assert_eq!(line, "Proposal 12 · decidededited just now");
}
#[test]
fn start_over_has_its_own_dialog_words() {
    for shell in [Shell::Browser, Shell::Terminal] {
        let mut root = document_root(Some(BODY), Mode::Split, &[], ProposalState::NotProposed);
        let before = view(&root, shell);
        let button = f::find(before.tree(), Role::Button, b"Start over").unwrap();
        assert_eq!(styled(before.tree(), look::START_OVER).len(), 1);
        let event = crate::event(&before, button, skein_ui::Act::Press).unwrap();
        f::root_event(&mut root, event);
        let after = view(&root, shell);
        assert!(f::find(after.tree(), Role::Dialog, b"Start over?").is_some());
        assert!(
            String::from_utf8_lossy(&f::text(after.tree()))
                .contains("A new Ask temper chat starts. This one stays in Chats.")
        );
    }
}

#[test]
fn a_first_write_says_wrote() {
    for shell in [Shell::Browser, Shell::Terminal] {
        let view = view(&root(Some(BODY), Mode::Chat, &[FIRST]), shell);
        let line = String::from_utf8(section_words(view.tree(), look::TURN_TOOL)).unwrap();
        assert_eq!(line, "Wrotehow-shop-works.md12 lines");
        assert_eq!(styled(view.tree(), look::TOOL_FILE).len(), 1);
        assert!(f::find(view.tree(), Role::Disclosure, b"note").is_none());
    }
    let view = view(&root(None, Mode::Chat, &[FIRST]), Shell::Browser);
    assert_eq!(section_words(view.tree(), look::TOOL_FILE), b"untitled.md");
}

#[test]
fn a_later_write_says_edited_and_counts() {
    for shell in [Shell::Browser, Shell::Terminal] {
        for mode in [Mode::Chat, Mode::Split, Mode::Draft] {
            let view = view(&root(Some(BODY), mode, &[LATER]), shell);
            let line = String::from_utf8(section_words(view.tree(), look::TURN_TOOL)).unwrap();
            assert_eq!(line, "Editedhow-shop-works.md+1 −1");
            let mut print = skein_ui_text::Print::new(&view.limits().tree).unwrap();
            skein_ui_text::render(view.tree(), b"temper", None, &mut print);
            let printed = String::from_utf8(print.text.as_slice().to_vec()).unwrap();
            assert!(printed.contains("Edited") && printed.contains("how-shop-works.md") && printed.contains("+1 −1"));
            let show = f::find(view.tree(), Role::Button, b"Show").unwrap();
            assert!(print.numbers.as_slice().contains(&show));
        }
    }
}

fn mixed_activity(tool: &[u8], failed: bool, outcome: &str) -> c::ActivityPage {
    let mut activity = activity(&[FIRST, outcome]);
    let c::TurnRead::Decoded(turn) = &mut activity.rows.get_mut(0).unwrap().decoded.as_mut().unwrap().turn else {
        unreachable!()
    };
    let c::Block::Call(call) = turn.messages.get_mut(0).unwrap().blocks.get_mut(1).unwrap() else { unreachable!() };
    call.tool = c::Text::new(tool.into(), 512).unwrap();
    call.failed = failed;
    activity
}

#[test]
fn a_note_call_isnt_counted() {
    let root =
        activity_root(Some(BODY), Mode::Chat, mixed_activity(b"read", false, "Read README"), ProposalState::Ordinary);
    let view = view(&root, Shell::Browser);
    let disclosure = f::find(view.tree(), Role::Disclosure, b"Read 1 file").unwrap();
    assert_eq!(view.tree().find(disclosure).unwrap().description.as_deref(), None);
    let at = view.tree().nodes().iter().position(|node| node.id == disclosure).unwrap();
    assert_eq!(f::node_text(view.tree(), at).as_ref(), b"readRead README");
    assert_eq!(styled(view.tree(), look::TOOL_FILE).len(), 1);
}

#[test]
fn a_failed_note_stays_in_the_disclosure() {
    let root = activity_root(Some(BODY), Mode::Chat, mixed_activity(b"note", true, LATER), ProposalState::Ordinary);
    let view = view(&root, Shell::Browser);
    let disclosure = f::find(view.tree(), Role::Disclosure, b"note").unwrap();
    assert_eq!(view.tree().find(disclosure).unwrap().description.as_deref(), Some(b"1 failed".as_slice()));
    assert_eq!(styled(view.tree(), look::TOOL_FILE).len(), 1);
    assert!(String::from_utf8_lossy(&f::text(view.tree())).contains(LATER));
}

#[test]
fn a_recall_stays_in_the_disclosure() {
    for (tool, outcome) in [(b"recall".as_slice(), LATER), (b"note", "Unrecognized answer")] {
        let root = activity_root(Some(BODY), Mode::Chat, mixed_activity(tool, false, outcome), ProposalState::Ordinary);
        let view = view(&root, Shell::Browser);
        let disclosure = f::find(view.tree(), Role::Disclosure, tool).unwrap();
        assert_eq!(view.tree().find(disclosure).unwrap().description.as_deref(), None);
        assert!(String::from_utf8_lossy(&f::text(view.tree())).contains(outcome));
        assert_eq!(styled(view.tree(), look::TOOL_FILE).len(), 1);
    }
}

#[test]
fn show_presses_its_line() {
    for (body, outcome, line) in [(Some(BODY), FIRST, 5), (Some(BODY), LATER, 11), (None, FIRST, 1)] {
        let mut view = view(&root(body, Mode::Chat, &[outcome]), Shell::Browser);
        let show = f::find(view.tree(), Role::Button, b"Show").unwrap();
        assert_eq!(
            crate::act(&mut view, show, skein_ui::Act::Press),
            Some(d::Action::Pages(Act::ShowSection { line }))
        );
    }
}

#[test]
fn show_flashes_and_focuses_the_section() {
    let mut root = root(Some(BODY), Mode::Chat, &[LATER]);
    let mut view = view(&root, Shell::Browser);
    assert_eq!(styled(view.tree(), look::DOC_FLASH).len(), 0);
    let show = f::find(view.tree(), Role::Button, b"Show").unwrap();
    let mut previous_flash = None;
    let mut sections = None;
    for count in 1..=2 {
        let event = crate::event(&view, show, skein_ui::Act::Press).unwrap();
        f::root_event(&mut root, event);
        assert_eq!(root.pages().chat().unwrap().mode, Mode::Split);
        f::build(&mut view, &root);
        let flashed = styled(view.tree(), look::DOC_FLASH);
        assert_eq!(flashed.len(), 1);
        assert_ne!(Some(flashed[0].id), previous_flash);
        assert_eq!(flashed[0].key.unwrap().a, count);
        previous_flash = Some(flashed[0].id);
        let focused: Vec<_> = view.tree().nodes().iter().filter(|node| node.focus.is_some()).collect();
        assert_eq!(focused.len(), 1);
        assert!(focused[0].styles.as_slice().contains(&look::DOC_SEC));
        assert_eq!(focused[0].focus, Some(count));
        assert!(section_words(view.tree(), look::DOC_FLASH).starts_with(b"Lands"));
        let current: Vec<_> = styled(view.tree(), look::DOC_SEC).iter().map(|node| node.id).collect();
        if let Some(before) = &sections {
            assert_eq!(&current, before);
        }
        sections = Some(current);
    }
}

#[test]
fn the_hint_names_the_key_only_with_a_document() {
    for shell in [Shell::Browser, Shell::Terminal] {
        for body in [None, Some(BODY)] {
            let view = view(&root(body, Mode::Chat, &[]), shell);
            let hint = section_words(view.tree(), look::SESSION_HINT);
            assert_eq!(String::from_utf8(hint).unwrap().contains(" · Ctrl+\\ switches the draft"), body.is_some());
        }
    }
}

#[test]
fn no_document_no_group_on_a_chat() {
    for shell in [Shell::Browser, Shell::Terminal] {
        let empty = view(&root(None, Mode::Chat, &[]), shell);
        assert_eq!(styled(empty.tree(), look::SEG).len(), 0);
        let view = view(&root(Some(BODY), Mode::Chat, &[]), shell);
        let nodes = view.tree().nodes();
        let byline = nodes.iter().position(|node| node.styles.as_slice().contains(&look::SESSION_RIGHT)).unwrap();
        let group = nodes.iter().position(|node| node.styles.as_slice().contains(&look::SEG)).unwrap();
        assert_eq!(group, byline + if shell == Shell::Browser { 3 } else { 1 });
        let terminal = nodes.iter().position(|node| node.styles.as_slice().contains(&look::TO_TERMINAL));
        assert_eq!(terminal.is_some(), shell == Shell::Browser);
        if let Some(terminal) = terminal {
            assert!(terminal > group);
        }
    }
}

pub(super) fn offered(body: Option<&[u8]>, ended: bool) -> d::Domain {
    document_root(
        body,
        Mode::Chat,
        &[],
        if ended { ProposalState::EndedDraftOffered } else { ProposalState::DraftOffered },
    )
}
fn plus(tree: &Tree) -> bool {
    tree.nodes().iter().any(|node| node.role == Role::Button && node.name.as_deref() == Some(b"+ Draft"))
}
#[test]
fn plus_draft_shows_when_the_setting_offers_it() {
    for (body, state, offered) in [
        (None, ProposalState::DraftOffered, true),
        (None, ProposalState::Ordinary, false),
        (None, ProposalState::NotProposed, true),
        (Some(BODY), ProposalState::NotProposed, false),
    ] {
        for shell in [Shell::Browser, Shell::Terminal] {
            let mut root = document_root(body, Mode::Chat, &[], state);
            let mut shown = view(&root, shell);
            assert_eq!(plus(shown.tree()), offered);
            if offered {
                let node = shown.tree().nodes().iter().find(|node| node.name.as_deref() == Some(b"+ Draft")).unwrap();
                assert!(node.styles.as_slice().contains(&look::ADD_DRAFT));
                let id = node.id;
                assert_eq!(crate::act(&mut shown, id, skein_ui::Act::Press), Some(d::Action::Pages(Act::AddDraft)));
            }
            f::root_event(&mut root, d::Event::Act { action: d::Action::Pages(Act::AddDraft) });
            let chat = root.pages().chat().unwrap();
            assert_eq!(chat.writing, offered);
            assert_eq!(chat.mode, if offered { Mode::Split } else { Mode::Chat });
        }
    }
}
#[test]
fn none_on_an_ended_chat() {
    for shell in [Shell::Browser, Shell::Terminal] {
        assert!(!plus(view(&offered(None, true), shell).tree()));
    }
}
#[test]
fn none_with_a_document() {
    for shell in [Shell::Browser, Shell::Terminal] {
        assert!(!plus(view(&offered(Some(BODY), false), shell).tree()));
    }
}
#[test]
fn the_empty_pane() {
    let mut root = offered(None, false);
    f::root_event(&mut root, d::Event::Act { action: d::Action::Pages(Act::AddDraft) });
    for shell in [Shell::Browser, Shell::Terminal] {
        let view = view(&root, shell);
        assert_eq!(skein_ui::outline::check(view.tree()), None);
        assert!(!plus(view.tree()), "the first press removes the action");
        assert_eq!(styled(view.tree(), look::MODE_SPLIT).len(), 1);
        assert_eq!(section_words(view.tree(), look::DOC_EMPTY), b"The draft appears here as you write it together.");
        let path = section_words(view.tree(), look::DOC_PATH);
        assert!(path.starts_with(b"untitled.md"));
        assert!(path.ends_with(b"writing"));
    }
}
#[test]
fn an_empty_thread_names_its_draft() {
    for body in [None, Some(BODY)] {
        let mut root = document_root(body, Mode::Chat, &[], ProposalState::UnsendableDraftOffered);
        if body.is_none() {
            f::root_event(&mut root, d::Event::Act { action: d::Action::Pages(Act::AddDraft) });
        }
        let view = view(&root, Shell::Browser);
        let words = section_words(view.tree(), look::META_LINE);
        assert_eq!(
            words,
            if body.is_some() {
                b"A chat that writes how-shop-works.md. Say what it should hold.".as_slice()
            } else {
                b"A chat that writes a draft. Say what it should hold."
            }
        );
    }
}

#[test]
fn a_document_call_draws_its_tool_line() {
    let outcome = "Note 1 revision 1 written · 12 lines\ndocument: \"How Shop works\" written, 12 lines.";
    for shell in [Shell::Browser, Shell::Terminal] {
        let root = activity_root(
            Some(BODY),
            Mode::Split,
            mixed_activity(b"document", false, outcome),
            ProposalState::Ordinary,
        );
        let view = view(&root, shell);
        assert!(
            String::from_utf8_lossy(&section_words(view.tree(), look::TURN_TOOL))
                .contains("Wrotehow-shop-works.md12 lines")
        );
        assert_eq!(styled(view.tree(), look::TOOL_FILE).len(), 2);
        assert!(f::find(view.tree(), Role::Disclosure, b"document").is_none());
        assert_eq!(styled(view.tree(), look::DOC_CHANGED).len(), 1);
    }
}
