use super::{
    chats, documents,
    fixtures::{self, task as f},
};
use crate::{
    frame::Bindings,
    look,
    pages::goal,
    words::{Times, Words},
};
use jig_client::{Down, Event as ClientEvent, Readable, Readout, UnlinkedRequest, Up, Watchable, contract as c};
use skein_lib::{Env, List, Queue, Time, Wall};
use skein_ui::{Builder, Role, Run, Style, Tree};
use temper_client_domain::{Address, Domain, Event, PlanFilter, Request, pages::Act};

const TASK: u64 = 9;
const DONE: u32 = 6;
const TOTAL: u32 = 9;
const MINUTE: u64 = 60_000_000_000;
const MADE: Wall = Wall::from_nanos(340 * MINUTE);
const UPDATED: Wall = Wall::from_nanos(380 * MINUTE);
const NOW: Wall = Wall::from_nanos(720 * MINUTE);
const DRAFT_WORDS: &[u8] = b"Build the draft.\nDraft: docs/design/draft/away-mode.md\nChat: T4";

fn document() -> c::Task {
    let mut task = f::document();
    task.chip.task = TASK;
    task.priority = Some(1);
    task.project = fixtures::PROJECT;
    task.chip.details.details.project = fixtures::PROJECT;
    task.chip.title = text(b"Away mode");
    task.requester = c::Requester::Person(73);
    task.made = MADE;
    task.budget.numbers.spent = 31;
    task.budget.numbers.budget = 40;
    // These supplied fields are independent of the header's spent number.
    task.budget.numbers.spent_below = 13;
    task.budget.numbers.reserved = 5;
    task.budget.left = 2;
    task.spec.specification.words = text(DRAFT_WORDS);
    task.result = None;
    task.chip.details.details.ending = None;
    task
}
fn text(value: &[u8]) -> c::DocumentText {
    c::DocumentText::complete(c::Text::new(value.into(), 1024).unwrap())
}
fn note(body: &[u8]) -> c::Note {
    c::Note {
        number: 1,
        revision: c::Revision { value: 1 },
        scope: c::NoteScope::Task { project: fixtures::PROJECT, task: TASK },
        description: text(b"report: The report's headline."),
        body: text(body),
        writer: c::NoteAuthor::Task { task: TASK, attempt: 1 },
        tasks: List::uncapped(),
        changed: Some(UPDATED),
        acts: List::uncapped(),
        cut: false,
    }
}
fn root(task: c::Task, done: u32, total: u32, note: Option<c::Note>, needs: bool) -> Domain {
    let limits = fixtures::domain_limits();
    let (mut root, _, watch, inbox_watch) =
        fixtures::root_at_with_needs(Address::Task { task: TASK, filter: PlanFilter::All }, &limits);
    let mut out = Queue::uncapped();
    let env = Env { now: Time::ZERO, wall: NOW, limits };
    temper_client_domain::step(
        &mut root,
        &env,
        Event::Client {
            event: ClientEvent::Up {
                up: Up::Watched {
                    watch: watch.unwrap(),
                    event: c::WatchEvent::Delivered(c::Delivery::Task(Box::new(task.clone()))),
                },
            },
        },
        &mut out,
    );
    temper_client_domain::step(
        &mut root,
        &env,
        Event::Client {
            event: ClientEvent::Up {
                up: Up::Watched {
                    watch: inbox_watch,
                    event: c::WatchEvent::Delivered(c::Delivery::Inbox(Box::new(documents::inbox(
                        List::uncapped(),
                        0,
                        c::Count { value: 0, limited: false },
                        0,
                        false,
                    )))),
                },
            },
        },
        &mut out,
    );
    // Drain only the finite reads and watches opened by this document and its replies.
    while let Some(request) = out.pop() {
        let up = match request {
            Request::Unlinked {
                request: UnlinkedRequest::Down { down: Down::Read { read, what: Readable::Jig(what) } },
            } => match read_value(what, &task, done, total, note.as_ref(), needs) {
                Some(value) => Some(Up::ReadDone { read, result: Readout::Jig(c::ReadResult::Value(value)) }),
                None => None,
            },
            Request::Unlinked {
                request:
                    UnlinkedRequest::Down { down: Down::Open { watch, subject: Watchable::Jig(c::Subject::Inbox { .. }) } },
            } => Some(Up::Watched {
                watch,
                event: c::WatchEvent::Delivered(c::Delivery::Inbox(Box::new(documents::inbox(
                    List::uncapped(),
                    0,
                    c::Count { value: 0, limited: false },
                    0,
                    false,
                )))),
            }),
            Request::Client { .. }
            | Request::Unlinked { .. }
            | Request::Show { .. }
            | Request::Leave { .. }
            | Request::SignOut
            | Request::Dropped { .. }
            | Request::Fact { .. } => None,
        };
        if let Some(up) = up {
            temper_client_domain::step(&mut root, &env, Event::Client { event: ClientEvent::Up { up } }, &mut out);
        }
    }
    assert_eq!(root.pages().inbox().unwrap().needs().next().is_some(), needs, "goal inbox: {:?}", root.pages().inbox());
    assert!(root.pages().task().unwrap().goal.as_ref().unwrap().line.is_some(), "own line delivered");
    root
}
fn read_value(
    what: c::Read,
    task: &c::Task,
    done: u32,
    total: u32,
    note: Option<&c::Note>,
    needs: bool,
) -> Option<c::ReadValue> {
    match what {
        c::Read::Notes { .. } => {
            let mut rows = List::uncapped();
            if let Some(note) = note {
                rows.push(note.clone()).unwrap();
            }
            Some(c::ReadValue::Notes(Box::new(c::NotePage { rows, before: None, after: None, cut: false })))
        }
        c::Read::Tasks { query, .. } if query.tracked == Some(true) => {
            let mut line = chats::line();
            line.chip = task.chip.clone();
            line.delegates_done = done;
            line.delegates_total = total;
            let mut rows = List::uncapped();
            rows.push(line).unwrap();
            Some(c::ReadValue::Tasks(Box::new(c::TaskPage {
                live: rows,
                live_after: None,
                ended: c::EndedGroup::None,
            })))
        }
        c::Read::Inbox { group: c::InboxReadGroup::Needs, goal: Some(TASK), .. } => {
            let mut rows = List::uncapped();
            if needs {
                rows.push(documents::entry(
                    c::ObjectKey::Question { task: TASK, message: 1 },
                    fixtures::PROJECT,
                    1,
                    MADE,
                    None,
                ))
                .unwrap();
            }
            Some(c::ReadValue::Inbox(Box::new(c::InboxPage {
                rows,
                before: None,
                after: None,
                selection: None,
                cut: false,
            })))
        }
        c::Read::Settings
        | c::Read::Setting(_)
        | c::Read::Revision(_)
        | c::Read::Revisions(_)
        | c::Read::Tasks { .. }
        | c::Read::Me
        | c::Read::Turn { .. }
        | c::Read::TaskText { .. }
        | c::Read::Task { .. }
        | c::Read::Proposal { .. }
        | c::Read::Escalation { .. }
        | c::Read::Question { .. }
        | c::Read::PersonTask { .. }
        | c::Read::Result { .. }
        | c::Read::Ended { .. }
        | c::Read::History { .. }
        | c::Read::Activity { .. }
        | c::Read::Inbox { .. }
        | c::Read::Spend { .. }
        | c::Read::Project { .. }
        | c::Read::Deployment => None,
    }
}
fn draw(root: &Domain, words: &Words) -> (Tree, Bindings, Times) {
    let mut b = Builder::new(&fixtures::tree_limits());
    let mut bindings = Bindings::new(fixtures::tree_limits().bindings);
    let mut times = Times::new(NOW);
    goal::main(&mut b, root, words, &mut bindings, 8, &mut times);
    (b.finish(), bindings, times)
}
fn styled(tree: &Tree, style: Style) -> usize {
    for (index, node) in tree.nodes().iter().enumerate() {
        if node.styles.as_slice().contains(&style) {
            return index;
        }
    }
    panic!("styled node {style:?}")
}
fn styled_text(tree: &Tree, style: Style) -> Box<[u8]> {
    fixtures::node_text(tree, styled(tree, style))
}
fn heading(tree: &Tree) -> Box<[u8]> {
    for (index, node) in tree.nodes().iter().enumerate() {
        if node.role == (Role::Heading { level: 1 }) {
            return fixtures::node_text(tree, index);
        }
    }
    panic!("goal heading")
}
fn assert_heading(task: c::Task, done: u32, total: u32, expected: &[u8]) {
    let root = root(task, done, total, None, false);
    for words in [fixtures::browser_words(), fixtures::words()] {
        assert_eq!(heading(&draw(&root, &words).0).as_ref(), expected);
    }
}
fn end(task: &mut c::Task, ending: c::ChipEnding) {
    f::phase(task, c::ChipPhase::Ended);
    task.chip.details.details.ending = Some(ending.clone());
    task.result = Some(c::TaskResult::Ending(Box::new(ending)));
}

#[test]
fn the_meta_says_title_state_count_spend_and_since() {
    let root = root(document(), DONE, TOTAL, Some(note(b"First paragraph.")), false);
    for words in [fixtures::browser_words(), fixtures::words()] {
        let (tree, _, times) = draw(&root, &words);
        assert_eq!(tree.nodes()[0].role, Role::Main);
        assert_eq!(tree.nodes()[0].styles.as_slice(), [look::MAIN, look::GOAL]);
        let head = styled(&tree, look::GOAL_HEAD);
        let two = styled(&tree, look::GOAL_TWO);
        let report = styled(&tree, look::GOAL_REPORT);
        assert_eq!(head, 1);
        assert_eq!(two, head + usize::try_from(tree.nodes()[head].size).unwrap());
        assert_eq!(report, two + 1);
        let meta = styled(&tree, look::GOAL_META);
        let actual = fixtures::node_text(&tree, meta);
        if words.shell == crate::words::Shell::Browser {
            assert_eq!(actual.as_ref(), "Away mode·running·6 of 9 tasks·31 of 40 spent·since 05:40".as_bytes());
            let children = &tree.nodes()[meta + 1..meta + usize::try_from(tree.nodes()[meta].size).unwrap()];
            assert_eq!(children.len(), 9);
            for node in children {
                assert_eq!(node.role, Role::Text);
            }
            assert_eq!(children[0].text.as_slice()[0].run, Run::Strong);
        } else {
            assert_eq!(actual.as_ref(), "Away mode · running · 6 of 9 tasks · 31 of 40 spent · since 05:40".as_bytes());
            let mut print = skein_ui_text::Print::new(&fixtures::tree_limits()).unwrap();
            skein_ui_text::render(&tree, b"temper", None, &mut print);
            assert!(skein_lib::bytes::find(print.text.as_slice(), &actual).is_some());
        }
        assert_eq!(times.next, Some(Wall::from_nanos(1440 * MINUTE)));
    }
}
#[test]
fn no_count_with_no_tasks() {
    let root = root(document(), 0, 0, None, false);
    for words in [fixtures::browser_words(), fixtures::words()] {
        assert!(skein_lib::bytes::find(&styled_text(&draw(&root, &words).0, look::GOAL_META), b"tasks").is_none());
    }
}
fn assert_state(task: c::Task, total: u32, needs: bool, expected: &[u8], token: Style) {
    let root = root(task, 0, total, None, needs);
    let tree = draw(&root, &fixtures::browser_words()).0;
    let index = styled(&tree, token);
    assert_eq!(tree.nodes()[index].role, Role::Text);
    assert_eq!(fixtures::node_text(&tree, index).as_ref(), expected);
    let tree = draw(&root, &fixtures::words()).0;
    assert!(skein_lib::bytes::find(&styled_text(&tree, look::GOAL_META), expected).is_some());
}
#[test]
fn needs_you_word_and_token() {
    assert_state(document(), TOTAL, true, b"needs you", look::GOALS_STATE_NEEDS);
}
#[test]
fn needs_funds_word_and_token() {
    let mut task = document();
    f::phase(
        &mut task,
        c::ChipPhase::Held {
            reason: c::HoldCategory::Exceeded(c::Exceeded { bound: c::Bound::Answer, value: 1 }),
            revision: 1,
        },
    );
    assert_state(task, TOTAL, true, b"needs funds", look::GOALS_STATE_NEEDS);
}
#[test]
fn paused_word_and_token() {
    let mut task = document();
    f::phase(&mut task, c::ChipPhase::Held { reason: c::HoldCategory::Stopped, revision: 1 });
    task.chip.details.details.hold = Some(c::ChipHold {
        placed: Some(c::PlacedHold {
            number: 1,
            owner: c::HoldOwner::Party { person: 73 },
            mode: c::HoldMode::Stop,
            scope: c::HoldScope::Subtree { task: TASK },
            reason: Box::default(),
            placed_at: MADE,
            revision: c::Revision { value: 1 },
            lifted: false,
            may_lift: true,
        }),
        words: None,
        circle: None,
        account: None,
        source: None,
        reason: c::HoldCategory::Stopped,
        revision: 1,
        before: c::PriorPhase::Active(c::RunStage::Idle),
        escalation: None,
        tries: c::FailureTries { transient: 0, permanent: 0, run: 0, agent: 0, lost: 0, invalid: 0 },
        hold_wait_since: None,
    });
    assert_state(task, TOTAL, false, b"paused by you", look::GOALS_STATE_PAUSED);
}
#[test]
fn planning_word_and_token() {
    assert_state(document(), 0, false, b"planning", look::GOALS_STATE_PLANNING);
}
#[test]
fn running_word_and_token() {
    assert_state(document(), TOTAL, false, b"running", look::GOALS_STATE_RUNNING);
}
#[test]
fn landed_word_and_token() {
    let mut task = document();
    end(&mut task, c::ChipEnding::Done(c::ResultValue::Report(text(b"All checked."))));
    assert_state(task, TOTAL, false, b"landed", look::GOALS_STATE_LANDED);
}
#[test]
fn stopped_word_and_token() {
    let mut task = document();
    let by = task.requester;
    end(&mut task, c::ChipEnding::Cancelled { reason: text(b"Stop here."), result: None, by, at: UPDATED });
    assert_state(task, TOTAL, false, b"stopped", look::GOALS_STATE_STOPPED);
}
#[test]
fn failed_word_and_token() {
    let mut task = document();
    end(&mut task, c::ChipEnding::Failed(text(b"Checks failed.")));
    assert_state(task, TOTAL, false, b"failed", look::GOALS_STATE_STOPPED);
}
#[test]
fn the_heading_is_the_reports() {
    let root = root(document(), DONE, TOTAL, Some(note(b"Paragraph.")), false);
    for words in [fixtures::browser_words(), fixtures::words()] {
        let tree = draw(&root, &words).0;
        assert_eq!(heading(&tree).as_ref(), b"The report's headline.");
        assert_eq!(styled_text(&tree, look::GOAL_BYLINE).as_ref(), b"Written by its agent at 06:20");
    }
}
#[test]
fn a_planning_goal_names_its_draft() {
    assert_heading(document(), 0, 0, b"Planning from away-mode.md: reading it and the designs it cites.");
}
#[test]
fn a_planning_goal_with_no_draft() {
    let mut task = document();
    task.spec.specification.words = text(b"Build what was asked.");
    assert_heading(task, 0, 0, b"Planning: reading what was asked.");
}
#[test]
fn a_stopped_goals_heading() {
    let mut task = document();
    let by = task.requester;
    end(&mut task, c::ChipEnding::Cancelled { reason: text(b"Stop here."), result: None, by, at: UPDATED });
    assert_heading(task, DONE, TOTAL, b"Stopped by you. What landed on the goal branch stays there.");
}
#[test]
fn a_landed_goals_heading_is_its_results_first_line() {
    let mut task = document();
    end(&mut task, c::ChipEnding::Done(c::ResultValue::Report(text(b"All checked.\nMore results."))));
    assert_heading(task, TOTAL, TOTAL, b"All checked.");
}
#[test]
fn landed_alone() {
    let mut task = document();
    end(&mut task, c::ChipEnding::Done(c::ResultValue::Approved { state: b"approved".as_slice().into(), words: None }));
    assert_heading(task, TOTAL, TOTAL, b"Landed.");
}
#[test]
fn a_failed_goals_heading_says_why() {
    let mut task = document();
    end(&mut task, c::ChipEnding::Failed(text(b"Checks failed.")));
    assert_heading(task, DONE, TOTAL, b"Failed: Checks failed.");
}
#[test]
fn otherwise_it_counts_what_landed() {
    assert_heading(document(), DONE, TOTAL, b"6 of 9 tasks landed.");
}
#[test]
fn no_byline_with_no_report() {
    let root = root(document(), DONE, TOTAL, None, false);
    for words in [fixtures::browser_words(), fixtures::words()] {
        for node in draw(&root, &words).0.nodes() {
            assert!(!node.styles.as_slice().contains(&look::GOAL_BYLINE));
        }
    }
}
#[test]
fn draft_and_chat_links() {
    for (spec, requester, expected) in [
        (DRAFT_WORDS, c::Requester::Task(2), [(b"Draft".as_slice(), 4_u64), (b"Chat T2", 2)].as_slice()),
        (b"Build it.".as_slice(), c::Requester::Task(2), [(b"Chat T2".as_slice(), 2)].as_slice()),
        (DRAFT_WORDS, c::Requester::Person(73), [(b"Draft".as_slice(), 4)].as_slice()),
        (b"Draft: draft.md".as_slice(), c::Requester::Person(73), [].as_slice()),
        (b"Chat: T4".as_slice(), c::Requester::Person(73), [].as_slice()),
        (b"Build it.".as_slice(), c::Requester::Deployment { project: 1 }, [].as_slice()),
    ] {
        let mut task = document();
        task.spec.specification.words = text(spec);
        task.requester = requester;
        let root = root(task, DONE, TOTAL, None, false);
        for words in [fixtures::browser_words(), fixtures::words()] {
            let (tree, bindings, _) = draw(&root, &words);
            let mut links = List::uncapped();
            for (index, node) in tree.nodes().iter().enumerate() {
                if node.role == Role::Link {
                    links.push(index).unwrap();
                }
            }
            assert_eq!(links.as_slice().len(), expected.len());
            for (index, (label, task)) in links.iter().zip(expected) {
                let node = &tree.nodes()[*index];
                assert_eq!(node.name.as_deref(), Some(*label));
                assert_eq!(node.styles.as_slice(), [look::GOAL_BTN, look::GOAL_SMALL]);
                assert_eq!(
                    bindings.decode(node.binding.unwrap().token, skein_ui::Act::Press),
                    Some(temper_client_domain::Action::Pages(Act::Go { address: Address::Chat { task: *task } }))
                );
            }
        }
    }
}
fn report_paragraphs(tree: &Tree) -> Box<[Box<[u8]>]> {
    let start = styled(tree, look::GOAL_REPORT);
    let end = start.checked_add(usize::try_from(tree.nodes()[start].size).unwrap()).unwrap();
    let mut paragraphs = List::uncapped();
    for index in start.checked_add(1).unwrap()..end {
        if tree.nodes()[index].role == Role::Paragraph {
            paragraphs.push(fixtures::node_text(tree, index)).unwrap();
        }
    }
    paragraphs.into_boxed()
}
#[test]
fn the_report_draws_paragraphs_left_and_risk_in_order() {
    let root = root(
        document(),
        DONE,
        TOTAL,
        Some(note(b"First paragraph.\n\nSecond paragraph.\n\nLeft: Two tasks.\nRisk: A slow check.")),
        false,
    );
    for words in [fixtures::browser_words(), fixtures::words()] {
        let tree = draw(&root, &words).0;
        assert_eq!(tree.nodes()[styled(&tree, look::GOAL_REPORT)].role, Role::Region);
        assert_eq!(tree.nodes()[styled(&tree, look::GOAL_REPORT)].name.as_deref(), Some(b"Report".as_slice()));
        assert_eq!(tree.nodes()[styled(&tree, look::GOAL_TITLE)].role, Role::Heading { level: 2 });
        let paragraphs = report_paragraphs(&tree);
        assert_eq!(
            paragraphs.as_ref(),
            [
                b"First paragraph.".as_slice(),
                b"Second paragraph.",
                b"Left: Two tasks.",
                b"Risk: A slow check.",
                b"By its agent at 06:20"
            ]
            .map(Box::from)
        );
        for label in [b"Left: ".as_slice(), b"Risk: "] {
            let mut found = false;
            for node in tree.nodes() {
                if let Some(run) = node.text.as_slice().first()
                    && run.text.as_ref() == label
                {
                    assert_eq!(
                        run.run,
                        if words.shell == crate::words::Shell::Browser { Run::Strong } else { Run::Plain }
                    );
                    found = true;
                }
            }
            assert!(found, "report label");
        }
    }
}
#[test]
fn planning_draws_dots_and_no_left_or_risk() {
    for (spec, expected) in [
        (DRAFT_WORDS, b"Reading away-mode.md and the designs it cites, to propose its tasks.".as_slice()),
        (b"Build it.", b"Reading what was asked, to propose its tasks."),
    ] {
        let mut task = document();
        task.spec.specification.words = text(spec);
        let root = root(task, 0, 0, None, false);
        for words in [fixtures::browser_words(), fixtures::words()] {
            let tree = draw(&root, &words).0;
            assert_eq!(report_paragraphs(&tree).as_ref(), [Box::from(expected)]);
            let dots = styled(&tree, look::GOAL_TYPING);
            assert_eq!(tree.nodes()[dots].role, Role::Figure);
            assert_eq!(tree.nodes()[dots].name.as_deref(), Some(b"Planning".as_slice()));
            assert_eq!(tree.nodes()[dots].size, 4);
            for node in &tree.nodes()[dots + 1..dots + 4] {
                assert_eq!(node.role, Role::Text);
            }
            assert_eq!(
                fixtures::node_text(&tree, dots).as_ref(),
                if words.shell == crate::words::Shell::Browser { b"".as_slice() } else { b"..." }
            );
        }
    }
}
#[test]
fn a_report_less_end_draws_its_result_or_reason() {
    for ending in [
        c::ChipEnding::Done(c::ResultValue::Report(text(b"First result.\n\nSecond result."))),
        c::ChipEnding::Failed(text(b"First result.\n\nSecond result.")),
        c::ChipEnding::Cancelled {
            reason: text(b"First result.\n\nSecond result."),
            result: None,
            by: c::Requester::Person(73),
            at: UPDATED,
        },
    ] {
        let mut task = document();
        end(&mut task, ending);
        let root = root(task, DONE, TOTAL, None, false);
        for words in [fixtures::browser_words(), fixtures::words()] {
            assert_eq!(
                report_paragraphs(&draw(&root, &words).0).as_ref(),
                [Box::from(b"First result.".as_slice()), Box::from(b"Second result.".as_slice())]
            );
        }
    }
}
#[test]
fn an_unread_document_draws_as_a_task_read() {
    let (root, _, _) =
        fixtures::root_at(Address::Task { task: TASK, filter: PlanFilter::All }, &fixtures::domain_limits());
    for words in [fixtures::browser_words(), fixtures::words()] {
        let tree = draw(&root, &words).0;
        assert_eq!(fixtures::text(&tree).as_ref(), b"Reading task 9\xe2\x80\xa6");
        assert_eq!(tree.nodes()[1].role, Role::Status);
    }
}
