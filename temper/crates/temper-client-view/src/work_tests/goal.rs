use super::{
    chats,
    fixtures::{self, task as f},
};

use crate::{
    bindings::Bindings,
    look,
    pages::goal,
    words::{Times, Words},
};

use jig_client::{Down, Event as ClientEvent, Readable, Readout, UnlinkedRequest, Up, contract as c};

use skein_lib::{Env, List, Queue, Time, Wall};

use skein_ui::{Builder, Role, Run, Style, Tree};

use temper_client_domain::{Address, Domain, Event, Request, pages::Act};


pub(super) use super::fixtures::goal::{
    DONE, DRAFT_WORDS, MADE, MINUTE, NOW, Side, TASK, TOTAL, document, offers, paused_document, press, question, root,
    root_with_side, sent_attempt, text,
};


const UPDATED: Wall = Wall::from_nanos(380 * MINUTE);


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


fn draw(root: &Domain, words: &Words) -> (Tree, Bindings, Times) {
    let mut b = Builder::new(&fixtures::tree_limits());
    let mut bindings = Bindings::new(fixtures::tree_limits().bindings);
    let mut times = Times::new(NOW);
    goal::main(&mut b, root, words, &mut bindings, 8, &mut times, &crate::forge_sign_in::Forges::new());
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
    let task = paused_document();
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
fn closed_word_and_token() {
    let mut task = document();
    let by = task.requester;
    end(&mut task, c::ChipEnding::Cancelled { reason: text(b"Close here."), result: None, by, at: UPDATED });
    assert_state(task, TOTAL, false, b"closed", look::GOALS_STATE_CLOSED);
}

#[test]
fn failed_word_and_token() {
    let mut task = document();
    end(&mut task, c::ChipEnding::Failed(text(b"Checks failed.")));
    assert_state(task, TOTAL, false, b"failed", look::GOALS_STATE_CLOSED);
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
fn a_closed_goals_heading() {
    let mut task = document();
    let by = task.requester;
    end(&mut task, c::ChipEnding::Cancelled { reason: text(b"Close here."), result: None, by, at: UPDATED });
    assert_heading(task, DONE, TOTAL, b"Closed by you. What landed on the goal branch stays there.");
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
    assert_heading(document(), DONE, TOTAL, b"6 of 9 tasks done.");
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

#[test]
fn a_cut_goal_keeps_its_draft_return_link() {
    let mut task = document();
    task.spec.specification.words = text(b"A long draft's preview");
    task.spec.specification.words.cut = true;
    let mut side = Side::empty();
    side.specification = Some(b"Draft: away.md\nChat: T4".as_slice().into());
    let root = root_with_side(task, DONE, TOTAL, None, false, side);
    for words in [fixtures::browser_words(), fixtures::words()] {
        let (tree, bindings, _) = draw(&root, &words);
        let mut draft = None;
        for node in tree.nodes() {
            if node.role == Role::Link && node.name.as_deref() == Some(b"Draft") {
                draft = Some(node);
                break;
            }
        }
        let draft = draft.expect("the full specification supplies the Draft return link");
        assert_eq!(
            bindings.decode(draft.binding.unwrap().token, skein_ui::Act::Press),
            Some(temper_client_domain::Action::Pages(Act::Go { address: Address::Chat { task: 4 } }))
        );
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
    let (root, _, _) = fixtures::root_at(Address::Task { task: TASK }, &fixtures::domain_limits());
    for words in [fixtures::browser_words(), fixtures::words()] {
        let tree = draw(&root, &words).0;
        assert_eq!(fixtures::text(&tree).as_ref(), b"Reading task 9\xe2\x80\xa6");
        assert_eq!(tree.nodes()[1].role, Role::Status);
    }
}


fn delegate(number: u64, title: &[u8], phase: c::ChipPhase) -> c::TreeRow {
    let mut task = document();
    task.chip.task = number;
    task.chip.title = text(title);
    f::phase(&mut task, phase);
    c::TreeRow {
        chip: task.chip,
        parent: TASK,
        depth: 1,
        proposals: List::uncapped(),
        spent: 0,
        dependencies: List::uncapped(),
        cut: false,
    }
}

fn landed(number: u64, title: &[u8]) -> c::TreeRow {
    let mut row = delegate(number, title, c::ChipPhase::Ended);
    row.chip.details.details.ending = Some(c::ChipEnding::Done(c::ResultValue::Change {
        kind: 1,
        part: c::Part { connector: fixtures::FORGE, kind: c::PartKind::Result, key: c::PartKey::Number(number) },
        words: text(b"Done."),
    }));
    row
}

fn ended_line(row: c::TreeRow) -> c::TaskLine {
    let mut line = chats::line();
    line.chip = row.chip;
    line.ended_at = Some(UPDATED);
    line
}

fn with_rows(rows: &[c::TreeRow], ended: &[c::TaskLine]) -> Side {
    let mut side = Side::empty();
    for row in rows {
        side.rows.push(row.clone()).unwrap();
    }
    if let c::EndedGroup::Rows { rows, .. } = &mut side.ended.ended {
        for row in ended {
            rows.push(row.clone()).unwrap();
        }
    }
    side
}

fn side_root(side: Side) -> Domain {
    let needs = side.need.is_some();
    root_with_side(document(), 0, 3, None, needs, side)
}

fn node_end(tree: &Tree, at: usize) -> usize {
    at.checked_add(usize::try_from(tree.nodes()[at].size).unwrap()).unwrap()
}

type RowWords = Box<[Box<[u8]>]>;

fn row_words(parts: &[&[u8]]) -> RowWords {
    let mut result = List::uncapped();
    for part in parts {
        result.push(Box::from(*part)).unwrap();
    }
    result.into_boxed()
}

fn lines(tree: &Tree) -> Box<[RowWords]> {
    let mut lines = List::uncapped();
    for (index, node) in tree.nodes().iter().enumerate() {
        if node.styles.as_slice().contains(&look::GOAL_LINE) {
            let mut parts = List::uncapped();
            let end = node_end(tree, index);
            let mut at = index.checked_add(1).unwrap();
            let start = at;
            for _ in start..end {
                if at == end {
                    break;
                }
                parts.push(fixtures::node_text(tree, at)).unwrap();
                at = node_end(tree, at);
            }
            lines.push(parts.into_boxed()).unwrap();
        }
    }
    lines.into_boxed()
}

fn task_links(tree: &Tree) -> Box<[Box<[u8]>]> {
    let side = styled(tree, look::GOAL_SIDE);
    let end = node_end(tree, side);
    let mut links = List::uncapped();
    for node in &tree.nodes()[side..end] {
        if node.role == Role::Link {
            links.push(node.name.clone().unwrap()).unwrap();
        }
    }
    links.into_boxed()
}

#[test]
fn tasks_merge_the_trees_rows_and_the_ended_page_by_number() {
    let root = side_root(with_rows(
        &[
            delegate(15, b"Fifteen", c::ChipPhase::Active(c::RunStage::Idle)),
            delegate(11, b"Tree eleven", c::ChipPhase::Active(c::RunStage::Idle)),
        ],
        &[ended_line(landed(10, b"Ten")), ended_line(landed(11, b"Stale eleven")), ended_line(landed(13, b"Thirteen"))],
    ));
    for words in [fixtures::browser_words(), fixtures::words()] {
        let (tree, bindings, _) = draw(&root, &words);
        assert_eq!(task_links(&tree), row_words(&[b"Tree eleven", b"Fifteen", b"Ten", b"Thirteen"]));
        let side = styled(&tree, look::GOAL_SIDE);
        let end = node_end(&tree, side);
        let mut numbers = [11, 15, 10, 13].into_iter();
        for node in &tree.nodes()[side..end] {
            if node.role == Role::Link {
                let task = numbers.next().unwrap();
                assert_eq!(
                    bindings.decode(node.binding.unwrap().token, skein_ui::Act::Press),
                    Some(temper_client_domain::Action::Pages(Act::Go { address: Address::Task { task } }))
                );
            }
        }
        assert!(numbers.next().is_none());
        let bar = styled(&tree, look::GOAL_BAR);
        let start = bar.checked_add(1).unwrap();
        let children = &tree.nodes()[start..node_end(&tree, bar)];
        let expected = [true, false, true, false];
        assert_eq!(children.len(), expected.len());
        for (node, landed) in children.iter().zip(expected) {
            assert_eq!(node.styles.as_slice().contains(&look::GOAL_BAR_LANDED), landed);
        }
    }
}

#[test]
fn a_delegates_own_delegates_and_cancelled_ones_are_left_out() {
    let mut child = delegate(20, b"Grandchild", c::ChipPhase::Active(c::RunStage::Idle));
    child.parent = 11;
    child.depth = 2;
    let mut cancelled = landed(12, b"Cancelled");
    cancelled.chip.details.details.ending = Some(c::ChipEnding::Cancelled {
        reason: text(b"Close"),
        result: None,
        by: c::Requester::Person(73),
        at: UPDATED,
    });
    let root = side_root(with_rows(
        &[child, cancelled.clone(), delegate(11, b"Direct", c::ChipPhase::Active(c::RunStage::Idle))],
        &[ended_line(cancelled), ended_line(landed(14, b"Done"))],
    ));
    for words in [fixtures::browser_words(), fixtures::words()] {
        let tree = draw(&root, &words).0;
        assert_eq!(task_links(&tree), row_words(&[b"Direct", b"Done"]));
        assert_eq!(tree.nodes()[styled(&tree, look::GOAL_BAR)].name.as_deref(), Some(b"1 landed of 2".as_slice()));
    }
}

#[test]
fn the_bar_is_named_and_classed() {
    let root = side_root(with_rows(
        &[
            landed(10, b"Done"),
            delegate(11, b"Running", c::ChipPhase::Active(c::RunStage::Running { attempt: 1 })),
            delegate(12, b"Waiting", c::ChipPhase::Active(c::RunStage::Idle)),
        ],
        &[],
    ));
    for (words, role) in [(fixtures::browser_words(), Role::Figure), (fixtures::words(), Role::Image)] {
        let tree = draw(&root, &words).0;
        let at = styled(&tree, look::GOAL_BAR);
        assert_eq!(tree.nodes()[at].role, role);
        assert_eq!(tree.nodes()[at].name.as_deref(), Some(b"1 landed of 3".as_slice()));
        assert_eq!(tree.nodes()[at].size, 4);
        assert_eq!(tree.nodes()[at.checked_add(1).unwrap()].styles.as_slice(), [look::GOAL_BAR_LANDED]);
        assert_eq!(tree.nodes()[at.checked_add(2).unwrap()].styles.as_slice(), [look::GOAL_BAR_RUNNING]);
        assert!(tree.nodes()[at.checked_add(3).unwrap()].styles.is_empty());
    }
}

fn assert_line(mut row: c::TreeRow, expected: &[u8], need: bool, style: Option<Style>) {
    row.chip.task = 11;
    row.chip.title = text(b"The task");
    let mut side = with_rows(&[row, delegate(12, b"Dependency", c::ChipPhase::Active(c::RunStage::Idle))], &[]);
    side.need = if need { Some(11) } else { None };
    let root = side_root(side);
    for words in [fixtures::browser_words(), fixtures::words()] {
        let tree = draw(&root, &words).0;
        let at = styled(&tree, look::GOAL_LINE);
        let subtree = &tree.nodes()[at..node_end(&tree, at)];
        assert_eq!(subtree[1].role, Role::Link);
        assert_eq!(subtree[1].name.as_deref(), Some(b"The task".as_slice()));
        let last = subtree.last().unwrap();
        assert_eq!(last.role, Role::Text);
        assert_eq!(last.text.as_slice()[0].text.as_ref(), expected);
        if let Some(style) = style {
            let mut found = false;
            for node in subtree {
                found |= node.styles.as_slice().contains(&style);
            }
            assert!(found);
        }
    }
}

fn waiting(deps: &[u64], holds: bool) -> c::TreeRow {
    let mut dependencies = List::uncapped();
    for dep in deps {
        dependencies.push(*dep).unwrap();
    }
    delegate(11, b"The task", c::ChipPhase::Waiting { dependencies, more: false, holds })
}

#[test]
fn a_running_line_says_running() {
    assert_line(
        delegate(11, b"Task", c::ChipPhase::Active(c::RunStage::Running { attempt: 1 })),
        b"running",
        false,
        Some(look::GOAL_LINE_RUNNING),
    );
}

#[test]
fn a_waits_on_line_names_its_dependency() {
    assert_line(waiting(&[12], false), b"waits on Dependency", false, None);
}

#[test]
fn a_waits_on_line_names_an_unshown_task() {
    assert_line(waiting(&[99], false), b"waits on task 99", false, None);
}

#[test]
fn a_waits_on_tasks_line_says_its_count() {
    assert_line(waiting(&[12, 13], false), b"waits on 2 tasks", false, None);
}

#[test]
fn a_waits_on_you_line_says_you() {
    assert_line(waiting(&[], false), b"waits on you", true, None);
}

#[test]
fn a_waits_on_resources_line_says_resources() {
    assert_line(waiting(&[], true), b"waits on its resources", false, None);
}

#[test]
fn a_held_line_says_held() {
    assert_line(
        delegate(
            11,
            b"Task",
            c::ChipPhase::Held { reason: c::HoldCategory::Failures { class: c::HoldFailure::Run }, revision: 1 },
        ),
        b"held",
        false,
        Some(look::GOAL_LINE_NEEDS),
    );
}

#[test]
fn a_failed_line_says_failed() {
    let mut row = landed(11, b"Task");
    row.chip.details.details.ending = Some(c::ChipEnding::Failed(text(b"Failed")));
    assert_line(row, b"failed", false, None);
}

#[test]
fn a_waits_line_says_waits() {
    assert_line(waiting(&[], false), b"waits", false, None);
}

#[test]
fn landed_rows_fold_under_n_landed_with_their_end() {
    let mut person = landed(11, b"Person");
    person.chip.details.details.ending = Some(c::ChipEnding::Done(c::ResultValue::Words(text(b"Blue"))));
    let mut advisor = landed(13, b"Advisor");
    advisor.chip.details.details.ending = Some(c::ChipEnding::Done(c::ResultValue::Report(text(b"Advice"))));
    let root = side_root(with_rows(&[landed(14, b"No time")], &[ended_line(person), ended_line(advisor)]));
    for words in [fixtures::browser_words(), fixtures::words()] {
        let tree = draw(&root, &words).0;
        let at = styled(&tree, look::GOAL_LANDED_FOLD);
        assert_eq!(tree.nodes()[at].role, Role::Disclosure);
        assert_eq!(tree.nodes()[at].name.as_deref(), Some(b"1 landed".as_slice()));
        assert!(!tree.nodes()[at].states.as_slice().contains(&skein_ui::State::Expanded));
        let bar = styled(&tree, look::GOAL_BAR);
        assert_eq!(tree.nodes()[bar].name.as_deref(), Some(b"1 landed of 3".as_slice()));
        let cells = &tree.nodes()[bar + 1..node_end(&tree, bar)];
        assert!(cells[0].styles.is_empty());
        assert!(cells[1].styles.is_empty());
        assert!(cells[2].styles.as_slice().contains(&look::GOAL_BAR_LANDED));
        for (index, node) in tree.nodes().iter().enumerate() {
            if node.role == Role::Link
                && [b"Person".as_slice(), b"Advisor"].contains(&node.name.as_deref().unwrap_or_default())
            {
                assert!(index < at, "done rows stay outside the fold");
            }
        }
        let expected = if words.shell == crate::words::Shell::Terminal {
            [
                row_words(&[b"Person", " · ".as_bytes(), b"done 06:20"]),
                row_words(&[b"Advisor", " · ".as_bytes(), b"done 06:20"]),
                row_words(&[b"No time"]),
            ]
        } else {
            [row_words(&[b"Person", b"done 06:20"]), row_words(&[b"Advisor", b"done 06:20"]), row_words(&[b"No time"])]
        };
        assert_eq!(lines(&tree).as_ref(), expected);
    }
}


fn section_text(tree: &Tree, name: &[u8]) -> Box<[u8]> {
    for (at, node) in tree.nodes().iter().enumerate() {
        if node.role == Role::Region && node.name.as_deref() == Some(name) {
            let end = node_end(tree, at);
            for index in at..end {
                if tree.nodes()[index].styles.as_slice().contains(&look::GOAL_FACTS) {
                    return fixtures::node_text(tree, index);
                }
            }
        }
    }
    panic!("section facts: {name:?}")
}

fn contains_run(tree: &Tree, value: &[u8]) -> bool {
    for node in tree.nodes() {
        for run in &node.text {
            if run.text.as_ref() == value {
                return true;
            }
        }
    }
    false
}

#[test]
fn no_tasks_while_planning_and_after() {
    for (total, expected) in [(0, "Proposing tasks from the draft…"), (3, "No tasks.")] {
        let root = root(document(), 0, total, None, false);
        for words in [fixtures::browser_words(), fixtures::words()] {
            assert_eq!(section_text(&draw(&root, &words).0, b"Tasks").as_ref(), expected.as_bytes());
        }
    }
}

#[test]
fn more_of_its_tasks_past_a_page() {
    for (after, cut, expected) in [(false, false, false), (true, false, true), (false, true, true)] {
        let mut side = Side::empty();
        if let c::EndedGroup::Rows { after: cursor, cut: capped, .. } = &mut side.ended.ended {
            *cursor = if after { Some(c::Cursor::new(Box::from(b"next".as_slice()), 32).unwrap()) } else { None };
            *capped = cut;
        }
        let root = side_root(side);
        for words in [fixtures::browser_words(), fixtures::words()] {
            let tree = draw(&root, &words).0;
            assert_eq!(contains_run(&tree, b"More of its tasks are not shown."), expected);
        }
    }
}

fn branch(task: &mut c::Task) {
    let mut path = List::uncapped();
    for part in [b"temper".as_slice(), b"9", b"g9"] {
        path.push(Box::from(part)).unwrap();
    }
    task.resources
        .push(c::ResourceLine {
            part: c::Part { connector: 1, kind: c::PartKind::Resource, key: c::PartKey::Path(path) },
            kind: 1,
            taken: false,
            holder: None,
            writer: None,
            writer_lost: false,
        })
        .unwrap();
}

#[test]
fn landing_says() {
    for (ending, at, has_branch, total, expected) in [
        (
            Some(c::ChipEnding::Done(c::ResultValue::Report(text(b"Done")))),
            Some(UPDATED),
            false,
            3,
            b"Landed at 06:20.".as_slice(),
        ),
        (Some(c::ChipEnding::Done(c::ResultValue::Report(text(b"Done")))), None, false, 3, b"Landed."),
        (Some(c::ChipEnding::Failed(text(b"Failure"))), None, true, 3, b"Failed before it landed."),
        (
            Some(c::ChipEnding::Cancelled {
                reason: text(b"Close"),
                result: None,
                by: c::Requester::Person(73),
                at: UPDATED,
            }),
            None,
            true,
            3,
            b"Closed before it landed.",
        ),
        (None, None, true, 0, b"Lands whole once all its tasks are checked."),
        (None, None, false, 3, b"Each task lands on its own once checked."),
    ] {
        let mut task = document();
        if let Some(ending) = ending {
            end(&mut task, ending);
        }
        task.chip.details.details.ended_at = at;
        if has_branch {
            branch(&mut task);
        }
        let root = root(task, 0, total, None, false);
        for words in [fixtures::browser_words(), fixtures::words()] {
            assert_eq!(section_text(&draw(&root, &words).0, b"Landing").as_ref(), expected);
        }
    }
}

#[test]
fn lands_whole_through_nine_and_past() {
    for (total, expected) in [
        (1, b"Lands whole once all one are checked.".as_slice()),
        (2, b"Lands whole once all two are checked."),
        (3, b"Lands whole once all three are checked."),
        (4, b"Lands whole once all four are checked."),
        (5, b"Lands whole once all five are checked."),
        (6, b"Lands whole once all six are checked."),
        (7, b"Lands whole once all seven are checked."),
        (8, b"Lands whole once all eight are checked."),
        (9, b"Lands whole once all nine are checked."),
        (10, b"Lands whole once all 10 are checked."),
    ] {
        let mut task = document();
        branch(&mut task);
        let root = root(task, 0, total, None, false);
        for words in [fixtures::browser_words(), fixtures::words()] {
            assert_eq!(section_text(&draw(&root, &words).0, b"Landing").as_ref(), expected);
        }
    }
}

#[test]
fn how_it_runs_says_its_way_by_its_branch() {
    for has_branch in [false, true] {
        let mut task = document();
        if has_branch {
            branch(&mut task);
        }
        let root = root(task, 0, 3, None, false);
        for words in [fixtures::browser_words(), fixtures::words()] {
            assert_eq!(
                section_text(&draw(&root, &words).0, b"How it runs").as_ref(),
                if has_branch {
                    b"Plan, tasks, land whole \xc2\xb7 asks as the project says".as_slice()
                } else {
                    b"Task by task \xc2\xb7 asks as the project says"
                }
            );
        }
    }
}

#[test]
fn decided_only_with_a_decided_line() {
    for body in [None, Some(b"No decisions.".as_slice()), Some(b"Report.\nDecided: one at 06:00 \xc2\xb7 two at 06:10")]
    {
        let report = match body {
            Some(body) => Some(note(body)),
            None => None,
        };
        let root = root(document(), 0, 3, report, false);
        for words in [fixtures::browser_words(), fixtures::words()] {
            let tree = draw(&root, &words).0;
            let mut has = false;
            for node in tree.nodes() {
                has |= node.role == Role::Region && node.name.as_deref() == Some(b"Decided by the goal");
            }
            let expected = match body {
                Some(body) => body.starts_with(b"Report."),
                None => false,
            };
            assert_eq!(has, expected);
            if has {
                assert_eq!(section_text(&tree, b"Decided by the goal").as_ref(), b"one at 06:00 \xc2\xb7 two at 06:10");
            }
            assert_eq!(tree.nodes()[styled(&tree, look::GOAL_SIDE)].role, Role::Complementary);
        }
    }
}


// The ordering fixture has two questions; choosing one records one answer.
const OPEN_QUESTIONS: usize = 2;

const CHOSEN_ANSWERS: usize = 1;

const FIRST_MESSAGE: u64 = 7;

const SECOND_MESSAGE: u64 = 3;

const CHOICES: &[u8] = b"Where?\nChoice: First\nChoice: Second (recommended)";


fn question_row(message: u64, words: &[u8]) -> (c::ObjectKey, c::Read, c::ReadValue) {
    (
        c::ObjectKey::Question { task: TASK, message },
        c::Read::Question { task: TASK, message },
        question(TASK, message, words),
    )
}

fn funds_row(kinds: &[c::ActKind]) -> (c::ObjectKey, c::Read, c::ReadValue) {
    let source = f::held_root();
    let mut task = source.pages().task().unwrap().document(source.client()).unwrap().clone();
    task.chip.task = TASK;
    task.chip.title = text(b"Funds task");
    task.hold.as_mut().unwrap().reason = c::HoldCategory::Budget;
    task.budget.numbers.spent = 31;
    task.budget.numbers.budget = 40;
    (
        c::ObjectKey::Escalation(TASK),
        c::Read::Escalation { task: TASK },
        c::ReadValue::Escalation(Box::new(c::Escalation { task: Box::new(task), acts: offers(kinds) })),
    )
}

fn bands_root(cards: &[(c::ObjectKey, c::Read, c::ReadValue)]) -> Domain {
    bands_root_read(cards, None)
}

fn bands_root_read(cards: &[(c::ObjectKey, c::Read, c::ReadValue)], whole: Option<Box<[u8]>>) -> Domain {
    let mut side = Side::empty();
    side.specification = whole;
    for card in cards {
        side.cards.push(card.clone()).unwrap();
    }
    root_with_side(document(), DONE, TOTAL, None, !cards.is_empty(), side)
}

fn bands(tree: &Tree) -> Box<[usize]> {
    let mut bands = List::uncapped();
    for (at, node) in tree.nodes().iter().enumerate() {
        if node.styles.as_slice().contains(&look::GOAL_BAND) {
            bands.push(at).unwrap();
        }
    }
    bands.into_boxed()
}

fn control_names(tree: &Tree, at: usize) -> RowWords {
    let mut names = List::uncapped();
    for node in band_nodes(tree, at) {
        if node.role == Role::Button || node.role == Role::Link {
            names.push(node.name.clone().unwrap()).unwrap();
        }
    }
    names.into_boxed()
}

fn band_nodes(tree: &Tree, at: usize) -> &[skein_ui::Node] {
    &tree.nodes()[at..node_end(tree, at)]
}

fn band_press(tree: &Tree, bindings: &Bindings, name: &[u8]) -> temper_client_domain::Action {
    for at in bands(tree).iter().copied() {
        for node in band_nodes(tree, at) {
            if node.name.as_deref() == Some(name) && (node.role == Role::Button || node.role == Role::Link) {
                return bindings.decode(node.binding.unwrap().token, skein_ui::Act::Press).unwrap();
            }
        }
    }
    panic!("band control {name:?}")
}

#[test]
fn one_band_per_open_need_in_the_inboxs_order() {
    let first = b"First question?\n\nContext.\nSaw: Evidence.\nChoice: First";
    let second = b"Second question?\n\nOther context.\nChoice: Second";
    let mut person = super::needs::person();
    person.task.task = TASK + 1;
    person.contract = c::TaskContractDocument::Report { words: 4096 };
    person.requester = c::Requester::Task(TASK);
    person.spec.specification.words = text(b"First ques");
    person.spec.specification.words.cut = true;
    let person_object = c::ObjectKey::PersonTask(person.task.task);
    let person_read = c::Read::PersonTask { task: person.task.task };
    let person = c::ReadValue::PersonTask(Box::new(person));
    for (mut root, object) in [
        (
            Box::new(bands_root(&[question_row(FIRST_MESSAGE, first), question_row(SECOND_MESSAGE, second)])),
            question_row(FIRST_MESSAGE, b"").0,
        ),
        (
            Box::new(bands_root_read(
                &[(person_object, person_read, person), question_row(SECOND_MESSAGE, second)],
                Some(Box::from(first.as_slice())),
            )),
            person_object,
        ),
    ] {
        for words in [fixtures::browser_words(), fixtures::words()] {
            let tree = draw(&root, &words).0;
            let bands = bands(&tree);
            assert_eq!(bands.len(), OPEN_QUESTIONS);
            assert!(skein_lib::bytes::find(&fixtures::node_text(&tree, bands[0]), b"Context.").is_some());
            assert!(skein_lib::bytes::find(&fixtures::node_text(&tree, bands[0]), b"Evidence.").is_some());
            assert_eq!(control_names(&tree, bands[1]), row_words(&[b"Answer"]));
            assert!(skein_lib::bytes::find(&fixtures::node_text(&tree, bands[1]), b"Other context.").is_none());
            assert_eq!(bands[0], node_end(&tree, styled(&tree, look::GOAL_HEAD)));
            assert_eq!(node_end(&tree, bands[1]), styled(&tree, look::GOAL_TWO));
            for (at, expected) in bands.iter().zip([b"First question?".as_slice(), b"Second question?"]) {
                assert_eq!(tree.nodes()[*at].role, Role::Region);
                assert_eq!(tree.nodes()[*at].name.as_deref(), Some(b"Needs you".as_slice()));
                assert_eq!(tree.nodes()[at + 1].role, Role::Paragraph);
                assert_eq!(
                    styled_text_in_band(&tree, *at, look::GOAL_BAND_TAG).as_ref(),
                    b"Needs you \xc2\xb7 a question"
                );
                assert!(skein_lib::bytes::find(&fixtures::node_text(&tree, *at), expected).is_some());
            }
        }
        fixtures::root_event(
            &mut root,
            Event::Act {
                action: temper_client_domain::Action::Pages(Act::Needs(
                    temper_client_domain::pages::needs::Act::Later { object },
                )),
            },
        );
        let tree = draw(&root, &fixtures::browser_words()).0;
        assert!(skein_lib::bytes::find(&fixtures::node_text(&tree, bands(&tree)[0]), b"Second question?").is_some());
    }
    for yours in [
        temper_client_domain::forge::approvals::Yours::Counts,
        temper_client_domain::forge::approvals::Yours::NoAccount,
    ] {
        let mut root = bands_root(&[question_row(FIRST_MESSAGE, first)]);
        let row =
            temper_client_domain::forge::approvals::Row { goal: Some(TASK), yours, ..super::needs::approval_row() };
        super::needs::approvals(&mut root, core::slice::from_ref(&row), 0);
        let (tree, bindings, _) = draw(&root, &fixtures::browser_words());
        let open = bands(&tree);
        assert_eq!(open.len(), 2);
        let approval = open[1];
        assert_eq!(
            styled_text_in_band(&tree, approval, look::GOAL_BAND_TAG).as_ref(),
            "Needs you · an approval".as_bytes()
        );
        assert!(
            skein_lib::bytes::find(&fixtures::node_text(&tree, approval), b"Approve A change on Forgejo").is_some()
        );
        if yours == temper_client_domain::forge::approvals::Yours::Counts {
            assert_eq!(control_names(&tree, approval), row_words(&["Review on Forgejo ↗".as_bytes()]));
            let mut address = None;
            for node in band_nodes(&tree, approval) {
                if node.role == Role::Link {
                    address = node.external.as_deref();
                    assert!(node.styles.as_slice().contains(&look::GOAL_PRIMARY));
                }
            }
            assert_eq!(address, Some(row.address.as_ref()));
        } else {
            assert_eq!(control_names(&tree, approval), row_words(&[b"Open"]));
            assert_eq!(
                band_press(&tree, &bindings, b"Open"),
                temper_client_domain::Action::Pages(Act::Go { address: Address::Task { task: row.task } })
            );
        }
        assert_eq!(bands(&draw(&root, &fixtures::words()).0).len(), 1);
    }
}

fn styled_text_in_band(tree: &Tree, at: usize, style: Style) -> Box<[u8]> {
    for index in at..node_end(tree, at) {
        if tree.nodes()[index].styles.as_slice().contains(&style) {
            return fixtures::node_text(tree, index);
        }
    }
    panic!("band style {style:?}")
}

#[test]
fn a_questions_choices_answer_as_needs_you_does() {
    use temper_client_domain::pages::needs;
    for words in [fixtures::browser_words(), fixtures::words()] {
        let mut root = bands_root(&[question_row(FIRST_MESSAGE, CHOICES)]);
        let (tree, bindings, _) = draw(&root, &words);
        let choices = styled(&tree, look::GOAL_CHOICES);
        assert_eq!(control_names(&tree, choices), row_words(&[b"First", b"Second"]));
        assert_eq!(tree.nodes()[styled(&tree, look::GOAL_PRIMARY)].name.as_deref(), Some(b"Second".as_slice()));
        let mut recommended = false;
        for node in tree.nodes() {
            if node.role == Role::Text && node.name.as_deref() == Some(b"recommended".as_slice()) {
                assert!(node.styles.as_slice().contains(&look::GOAL_PRIMARY));
                recommended = true;
            }
        }
        assert!(recommended);
        assert_eq!(
            fixtures::node_text(&tree, styled(&tree, look::GOAL_PRIMARY)).as_ref(),
            if words.shell == crate::words::Shell::Terminal {
                b"Second \xc2\xb7 recommended".as_slice()
            } else {
                b"recommended"
            }
        );
        let action = band_press(&tree, &bindings, b"Second");
        let object = c::ObjectKey::Question { task: TASK, message: FIRST_MESSAGE };
        assert_eq!(
            action,
            temper_client_domain::Action::Pages(Act::Needs(needs::Act::Choose {
                object,
                words: Box::from(b"Second".as_slice())
            }))
        );
        fixtures::root_event(&mut root, Event::Act { action });
        let lines = root.pages().needs().answered(fixtures::NOW, needs::ANSWERED_ON_NEEDS, Some(fixtures::PROJECT));
        assert_eq!(lines.len(), CHOSEN_ANSWERS);
        assert_eq!(lines[0].object, object);
        assert_eq!(lines[0].label, needs::Label::Choice(Box::from(b"Second".as_slice())));
        let answered_tree = draw(&root, &words).0;
        assert!(bands(&answered_tree).is_empty());
        assert!(skein_lib::bytes::find(&styled_text(&answered_tree, look::GOAL_META), b"running").is_some());
        assert_eq!(
            root.pages().needs().toast(),
            Some(&needs::Toasted::Answered { object, label: needs::Label::Choice(Box::from(b"Second".as_slice())) })
        );
    }
}

#[test]
fn no_reply_and_no_later_on_a_band() {
    let root = bands_root(&[question_row(FIRST_MESSAGE, CHOICES)]);
    for words in [fixtures::browser_words(), fixtures::words()] {
        let (tree, bindings, _) = draw(&root, &words);
        let mut controls = List::uncapped();
        for at in bands(&tree).iter().copied() {
            for node in band_nodes(&tree, at) {
                if let Some(binding) = node.binding {
                    controls.push(bindings.decode(binding.token, skein_ui::Act::Press).unwrap()).unwrap();
                }
            }
        }
        assert_eq!(
            controls.as_slice(),
            [b"First".as_slice(), b"Second"].map(|words| temper_client_domain::Action::Pages(Act::Needs(
                temper_client_domain::pages::needs::Act::Choose {
                    object: c::ObjectKey::Question { task: TASK, message: FIRST_MESSAGE },
                    words: Box::from(words)
                }
            )))
        );
    }
}

#[test]
fn no_choices_says_answer_or_open() {
    let mut held = funds_row(&[]);
    if let c::ReadValue::Escalation(h) = &mut held.2 {
        h.task.hold.as_mut().unwrap().reason = c::HoldCategory::Failures { class: c::HoldFailure::Run };
    }
    let mut person = super::needs::person();
    person.task.task = TASK;
    let person = (
        c::ObjectKey::PersonTask(TASK),
        c::Read::PersonTask { task: TASK },
        c::ReadValue::PersonTask(Box::new(person)),
    );
    let proposal = super::needs::proposal(c::Proposer::Task(TASK));
    let proposal = (
        c::ObjectKey::Proposal(proposal.key),
        c::Read::Proposal { proposal: proposal.key },
        c::ReadValue::Proposal(Box::new(proposal)),
    );
    let mut cut = question_row(FIRST_MESSAGE, CHOICES);
    if let c::ReadValue::Question(question) = &mut cut.2 {
        question.words.cut = true;
    }
    for (card, name, cut) in [
        (question_row(FIRST_MESSAGE, b"Where?"), b"Answer".as_slice(), false),
        (cut, b"Answer", true),
        (person, b"Answer", false),
        (held, b"Open", false),
        (proposal, b"Open", false),
    ] {
        let root = bands_root(&[card]);
        for words in [fixtures::browser_words(), fixtures::words()] {
            let (tree, bindings, _) = draw(&root, &words);
            let band = bands(&tree)[0];
            assert_eq!(
                skein_lib::bytes::find(&fixtures::node_text(&tree, band), b"This question is cut here.").is_some(),
                cut
            );
            if cut {
                assert_eq!(
                    band_press(&tree, &bindings, b"Read the rest"),
                    temper_client_domain::Action::Pages(Act::Go { address: Address::Task { task: TASK } })
                );
            }
            let at = styled(&tree, look::GOAL_PRIMARY);
            assert_eq!(tree.nodes()[at].role, Role::Link);
            assert!(tree.nodes()[at].styles.as_slice().contains(&look::GOAL_BTN));
            assert_eq!(
                band_press(&tree, &bindings, name),
                temper_client_domain::Action::Pages(Act::Go {
                    address: Address::Inbox {
                        filter: temper_client_domain::address::Filter {
                            project: Some(fixtures::PROJECT),
                            goal: Some(TASK),
                            kind: None
                        }
                    }
                })
            );
        }
    }
}

#[test]
fn a_funds_band_offers_release_and_leave_held() {
    for kinds in
        [&[c::ActKind::Release, c::ActKind::LeaveHeld][..], &[c::ActKind::Release], &[c::ActKind::LeaveHeld], &[]]
    {
        let root = bands_root(&[funds_row(kinds)]);
        for words in [fixtures::browser_words(), fixtures::words()] {
            let (tree, bindings, _) = draw(&root, &words);
            let at = bands(&tree)[0];
            assert_eq!(styled_text_in_band(&tree, at, look::GOAL_BAND_TAG).as_ref(), b"Needs you \xc2\xb7 funds");
            assert!(skein_lib::bytes::find(&fixtures::node_text(&tree, at), b"31 of 40 used.").is_some());
            let names = control_names(&tree, at);
            if kinds.is_empty() {
                assert_eq!(names, row_words(&[b"Open"]));
            } else {
                assert_eq!(names.len(), kinds.len());
                for (kind, name) in
                    [(c::ActKind::Release, b"Release".as_slice()), (c::ActKind::LeaveHeld, b"Leave held")]
                {
                    if kinds.contains(&kind) {
                        assert_eq!(
                            band_press(&tree, &bindings, name),
                            temper_client_domain::Action::Pages(Act::Decide {
                                object: c::ObjectKey::Escalation(TASK),
                                revision: Some(f::HOLD_REVISION),
                                act: kind
                            })
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn a_need_makes_the_state_needs() {
    for card in [question_row(FIRST_MESSAGE, CHOICES), funds_row(&[])] {
        let root = bands_root(&[card]);
        let tree = draw(&root, &fixtures::browser_words()).0;
        assert_eq!(styled_text(&tree, look::GOALS_STATE_NEEDS).as_ref(), b"needs you");
    }
    let mut task = document();
    f::phase(
        &mut task,
        c::ChipPhase::Held {
            reason: c::HoldCategory::Exceeded(c::Exceeded { bound: c::Bound::Answer, value: 1 }),
            revision: 1,
        },
    );
    let mut side = Side::empty();
    side.cards.push(funds_row(&[])).unwrap();
    let root = root_with_side(task, DONE, TOTAL, None, true, side);
    assert_eq!(
        styled_text(&draw(&root, &fixtures::browser_words()).0, look::GOALS_STATE_NEEDS).as_ref(),
        b"needs funds"
    );
}


fn acts_names(tree: &Tree) -> Box<[&[u8]]> {
    let mut names = List::uncapped();
    for node in band_nodes(tree, styled(tree, look::GOAL_ACTS)) {
        if node.role == Role::Button || node.role == Role::Link {
            names.push(node.name.as_deref().unwrap()).unwrap();
        }
    }
    names.into_boxed()
}

fn named_index(tree: &Tree, role: Role, name: &[u8]) -> usize {
    for (index, node) in tree.nodes().iter().enumerate() {
        if node.role == role && node.name.as_deref() == Some(name) {
            return index;
        }
    }
    panic!("missing {role:?} {name:?}")
}


#[test]
fn the_acts_in_order_by_offer() {
    let mut task = document();
    task.requester = c::Requester::Task(2);
    task.acts = offers(&[c::ActKind::Amend, c::ActKind::Stop, c::ActKind::Cancel]);
    let root = root(task, DONE, TOTAL, None, false);
    for words in [fixtures::browser_words(), fixtures::words()] {
        let (tree, bindings, _) = draw(&root, &words);
        assert_eq!(acts_names(&tree).as_ref(), [b"Draft".as_slice(), b"Chat T2", b"Amend", b"Pause", b"Close"]);
        for (name, act) in [(b"Amend".as_slice(), Act::Amend), (b"Pause", Act::Pause), (b"Close", Act::Close)] {
            let node = tree.nodes().get(named_index(&tree, Role::Button, name)).unwrap();
            assert!(node.styles.as_slice().contains(&look::GOAL_BTN));
            assert!(node.styles.as_slice().contains(&look::GOAL_SMALL));
            assert_eq!(node.styles.as_slice().contains(&look::GOAL_QUIET), name == b"Close");
            assert_eq!(
                bindings.decode(node.binding.unwrap().token, skein_ui::Act::Press),
                Some(temper_client_domain::Action::Pages(act))
            );
        }
    }
}

#[test]
fn not_offered_not_drawn() {
    let mut task = document();
    task.acts = offers(&[]);
    let tree = draw(&root(task, DONE, TOTAL, None, false), &fixtures::browser_words()).0;
    assert_eq!(acts_names(&tree).as_ref(), [b"Draft".as_slice()]);
    for admission in
        [c::Admission::Proposed { to: c::Recipient::Party(73) }, c::Admission::Refused(c::Refusal::NotOffered)]
    {
        let mut task = document();
        task.acts = offers(&[c::ActKind::Amend, c::ActKind::Stop, c::ActKind::Cancel]);
        for index in 0..task.acts.len() {
            task.acts.get_mut(index).unwrap().admission = admission.clone();
        }
        let tree = draw(&root(task, DONE, TOTAL, None, false), &fixtures::browser_words()).0;
        assert_eq!(acts_names(&tree).as_ref(), [b"Draft".as_slice()]);
        let said = fixtures::text(&tree);
        let expected = match &admission {
            c::Admission::Proposed { .. } => b"This needs a proposal to".as_slice(),
            c::Admission::Refused(_) => b"This act is not offered",
            c::Admission::Allowed | c::Admission::Bounded { .. } => unreachable!(),
        };
        assert!(skein_lib::bytes::find(&said, expected).is_some());
    }
}

#[test]
fn ended_has_none() {
    for ending in [
        c::ChipEnding::Done(c::ResultValue::Report(text(b"Done"))),
        c::ChipEnding::Failed(text(b"Failed")),
        c::ChipEnding::Cancelled { reason: text(b"Closed"), result: None, by: c::Requester::Person(73), at: NOW },
    ] {
        let mut task = document();
        task.acts = offers(&[c::ActKind::Amend, c::ActKind::Stop, c::ActKind::Cancel]);
        end(&mut task, ending);
        let tree = draw(&root(task, DONE, TOTAL, None, false), &fixtures::browser_words()).0;
        assert_eq!(acts_names(&tree).as_ref(), [b"Draft".as_slice()]);
    }
}

#[test]
fn paused_says_resume() {
    let mut task = paused_document();
    task.acts = offers(&[c::ActKind::Amend, c::ActKind::Release, c::ActKind::Stop, c::ActKind::Cancel]);
    let tree = draw(&root(task, DONE, TOTAL, None, false), &fixtures::browser_words()).0;
    assert_eq!(acts_names(&tree).as_ref(), [b"Draft".as_slice(), b"Amend", b"Resume", b"Close"]);
}

#[test]
fn held_otherwise_has_no_resume() {
    let mut task = paused_document();
    task.chip.details.details.hold.as_mut().unwrap().placed.as_mut().unwrap().scope =
        c::HoldScope::Subtree { task: 99 };
    task.acts = offers(&[c::ActKind::Release]);
    let tree = draw(&root(task, DONE, TOTAL, None, false), &fixtures::browser_words()).0;
    assert_eq!(acts_names(&tree).as_ref(), [b"Draft".as_slice()]);
}

#[test]
fn the_close_dialogs_words() {
    let mut task = document();
    task.acts = offers(&[c::ActKind::Cancel]);
    task.acts.get_mut(0).unwrap().effect = Some(c::Effect::Cancel(c::Impact {
        tasks: c::ImpactCounts {
            waiting: 1,
            running: 2,
            held: 3,
            closing: 4,
            ended: 5,
            done: 6,
            failed: 7,
            cancelled: 8,
        },
        live_runs: 9,
        proposals: 10,
    }));
    let mut root = root(task, DONE, TOTAL, None, false);
    press(&mut root, Act::Close);
    let tree = draw(&root, &fixtures::browser_words()).0;
    let dialog = named_index(&tree, Role::Dialog, b"Close this goal?");
    assert_eq!(tree.nodes()[dialog].name.as_deref(), Some(b"Close this goal?".as_slice()));
    assert_eq!(
        fixtures::node_text(&tree, dialog).as_ref(),
        "Close this goal?You can still read it, but it can’t be reopened.What landed on its branch stays there."
            .as_bytes()
    );
    for label in [b"Keep it".as_slice(), b"Close goal"] {
        assert!(fixtures::find(&tree, Role::Button, label).is_some());
    }
    assert_eq!(acts_names(&tree).as_ref(), [b"Draft".as_slice(), b"Close"]);
}

#[test]
fn the_editor_sits_between_header_and_bands() {
    let mut task = document();
    task.acts = offers(&[c::ActKind::Amend]);
    let mut root = root(task, DONE, TOTAL, None, true);
    press(&mut root, Act::Amend);
    let tree = draw(&root, &fixtures::browser_words()).0;
    let editor = styled(&tree, look::GOAL_FIELD);
    let header = styled(&tree, look::GOAL_HEAD);
    assert!(node_end(&tree, header) <= editor);
    assert!(editor < styled(&tree, look::GOAL_BAND));
    for (role, name) in [
        (Role::TextArea, b"Specification".as_slice()),
        (Role::Check, b"Tracked"),
        (Role::TextField, b"Priority"),
        (Role::Button, b"Send amendment"),
        (Role::Button, b"Discard"),
    ] {
        let node = tree.nodes().get(named_index(&tree, role, name)).unwrap();
        if role == Role::TextArea || role == Role::TextField {
            assert!(node.styles.as_slice().contains(&look::GOAL_TEXT));
        }
        if role == Role::Button {
            assert!(node.styles.as_slice().contains(&look::GOAL_BTN));
        }
    }
    for node in tree.nodes() {
        assert!(!node.styles.as_slice().contains(&look::TASK_AMEND));
    }
}


mod composer {
    use super::*;
    use skein_ui::State;
    const PERSON: u64 = fixtures::chat::PARTY;
    const OTHER_PERSON: u64 = 89;
    const FIRST: u64 = 1;
    const SECOND: u64 = 2;
    const THIRD: u64 = 3;
    const PENDING: &[u8] = b"Keep it small.";

    fn offered() -> c::Task {
        let mut task = document();
        task.acts = offers(&[c::ActKind::Message]);
        task
    }
    fn without_composer(task: c::Task) {
        let root = root(task, DONE, TOTAL, None, false);
        for words in [fixtures::browser_words(), fixtures::words()] {
            let tree = draw(&root, &words).0;
            for node in tree.nodes() {
                assert!(!node.styles.as_slice().contains(&look::GOAL_COMPOSER_BOX));
                assert!(!node.styles.as_slice().contains(&look::GOAL_COMPOSER));
                assert!(!node.styles.as_slice().contains(&look::GOAL_HINT));
            }
        }
    }
    fn word(message: u64, by: c::Requester, words: &[u8]) -> c::DecodedActivityLine {
        c::DecodedActivityLine {
            source: Box::new(c::ActivityLine {
                task: TASK,
                position: message,
                at: MADE,
                line: c::Activity::Words { message, by, kind: c::MessageKind::Words, words: text(words).bytes },
            }),
            decoded: None,
        }
    }
    fn with_activity(rows: &[c::DecodedActivityLine]) -> Domain {
        let mut side = Side::empty();
        side.activity =
            Some(c::ActivityPage { rows: List::from_boxed(rows.into()), before: None, after: None, cut: false });
        root_with_side(offered(), DONE, TOTAL, None, false, side)
    }
    fn sent_lines(tree: &Tree) -> Box<[Box<[u8]>]> {
        let mut lines = List::uncapped();
        for (at, node) in tree.nodes().iter().enumerate() {
            if node.styles.as_slice().contains(&look::GOAL_YOU) {
                assert_eq!(node.role, Role::Paragraph);
                lines.push(fixtures::node_text(tree, at)).unwrap();
            }
        }
        lines.into_boxed()
    }
    #[test]
    fn the_composer_shows_while_message_is_offered() {
        let root = root(offered(), DONE, TOTAL, None, false);
        for words in [fixtures::browser_words(), fixtures::words()] {
            let (tree, bindings, _) = draw(&root, &words);
            let box_at = styled(&tree, look::GOAL_COMPOSER_BOX);
            let composer = styled(&tree, look::GOAL_COMPOSER);
            assert!(box_at < composer && composer < node_end(&tree, box_at));
            assert!(node_end(&tree, styled(&tree, look::GOAL_TWO)) <= box_at);
            let at = named_index(&tree, Role::TextArea, b"Tell Away mode something");
            let node = &tree.nodes()[at];
            assert!(composer < at && at < node_end(&tree, composer));
            assert!(node.states.as_slice().contains(&State::SubmitsOnEnter));
            let binding = node.binding.unwrap();
            assert_eq!(
                bindings.decode(binding.token, skein_ui::Act::Submit),
                Some(temper_client_domain::Action::Pages(Act::Send))
            );
            assert_eq!(
                bindings.decode(binding.token, skein_ui::Act::Input { value: Box::from(PENDING) }),
                Some(temper_client_domain::Action::Pages(Act::MessageWords { words: Box::from(PENDING) }))
            );
            let send = &tree.nodes()[named_index(&tree, Role::Button, b"Send")];
            assert_eq!(
                bindings.decode(send.binding.unwrap().token, skein_ui::Act::Press),
                Some(temper_client_domain::Action::Pages(Act::Send))
            );
        }
    }
    #[test]
    fn not_offered_not_drawn() {
        let mut task = offered();
        task.acts = offers(&[]);
        without_composer(task);
    }
    #[test]
    fn not_waiting_holds_nothing() {
        let mut task = offered();
        task.acts.get_mut(0).unwrap().admission = c::Admission::Refused(c::Refusal::NotWaiting);
        let root = root(task.clone(), DONE, TOTAL, None, false);
        without_composer(task);
        let tree = draw(&root, &fixtures::browser_words()).0;
        let main = styled(&tree, look::GOAL);
        assert_eq!(node_end(&tree, styled(&tree, look::GOAL_TWO)), node_end(&tree, main));
    }
    #[test]
    fn ended_has_none() {
        let mut task = offered();
        end(&mut task, c::ChipEnding::Done(c::ResultValue::Report(text(b"Done."))));
        without_composer(task);
    }
    #[test]
    fn its_placeholder_and_hint() {
        let root = root(offered(), DONE, TOTAL, None, false);
        for words in [fixtures::browser_words(), fixtures::words()] {
            let tree = draw(&root, &words).0;
            let node = &tree.nodes()[named_index(&tree, Role::TextArea, b"Tell Away mode something")];
            assert_eq!(node.placeholder.as_deref(), Some(b"Tell Away mode something".as_slice()));
            assert_eq!(
                styled_text(&tree, look::GOAL_HINT).as_ref(),
                b"Its agent reads it at once. A change of plan becomes an amendment it shows you first."
            );
        }
    }
    #[test]
    fn a_send_asks_for_the_goals_composer() {
        let mut root = root(offered(), DONE, TOTAL, None, false);
        for count in 0..=2 {
            for words in [fixtures::browser_words(), fixtures::words()] {
                let tree = draw(&root, &words).0;
                let composer = &tree.nodes()[named_index(&tree, Role::TextArea, b"Tell Away mode something")];
                assert_eq!(composer.focus, if count == 0 { None } else { Some(count) });
            }
            press(&mut root, Act::Send);
        }
    }
    #[test]
    fn send_disabled_while_empty() {
        let mut root = root(offered(), DONE, TOTAL, None, false);
        for (draft, disabled) in [(b"".as_slice(), true), (PENDING, false), (b"", true)] {
            press(&mut root, Act::MessageWords { words: Box::from(draft) });
            let tree = draw(&root, &fixtures::browser_words()).0;
            let node = &tree.nodes()[named_index(&tree, Role::Button, b"Send")];
            assert_eq!(node.states.as_slice().contains(&State::Disabled), disabled);
        }
    }
    #[test]
    fn sent_words_oldest_first_then_being_sent() {
        let mut root = with_activity(&[
            word(SECOND, c::Requester::Person(PERSON), b"Second."),
            word(FIRST, c::Requester::Person(PERSON), b"First."),
        ]);
        press(&mut root, Act::MessageWords { words: Box::from(PENDING) });
        let attempt = sent_attempt(press(&mut root, Act::Send));
        for words in [fixtures::browser_words(), fixtures::words()] {
            let tree = draw(&root, &words).0;
            assert_eq!(sent_lines(&tree).as_ref(), row_words(&[b"First.", b"Second.", PENDING]).as_ref());
            assert!(fixtures::find(&tree, Role::Status, b"Being sent").is_some());
            assert!(node_end(&tree, styled(&tree, look::GOAL_SENT)) <= styled(&tree, look::GOAL_COMPOSER_BOX));
            assert!(skein_lib::bytes::find(&fixtures::text(&tree), b"Its agent:").is_none());
        }
        assert!(root.pages().task().unwrap().words.is_empty());
        fixtures::root_event(
            &mut root,
            Event::Client {
                event: ClientEvent::Up {
                    up: Up::Answer {
                        attempt,
                        envelope: skein_lib::RequestEnvelope::Final,
                        answer: Some(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::Document(
                            c::DocumentOutcome::Said { task: TASK, message: THIRD },
                        )))),
                        why: None,
                    },
                },
            },
        );
        read_latest(
            &mut root,
            &[
                word(THIRD, c::Requester::Person(PERSON), PENDING),
                word(SECOND, c::Requester::Person(PERSON), b"Second."),
                word(FIRST, c::Requester::Person(PERSON), b"First."),
            ],
        );
        for words in [fixtures::browser_words(), fixtures::words()] {
            let tree = draw(&root, &words).0;
            assert_eq!(sent_lines(&tree).as_ref(), row_words(&[b"First.", b"Second.", PENDING]).as_ref());
            assert!(fixtures::find(&tree, Role::Status, b"Being sent").is_none());
        }
    }
    fn read_latest(root: &mut Domain, rows: &[c::DecodedActivityLine]) {
        let handle = root.pages().task().unwrap().activity;
        let mut out = Queue::uncapped();
        temper_client_domain::step(
            root,
            &Env { now: Time::ZERO, wall: NOW, limits: fixtures::domain_limits() },
            Event::Client {
                event: ClientEvent::Read {
                    handle,
                    read: Readable::Jig(c::Read::Activity { task: TASK, before: None }),
                },
            },
            &mut out,
        );
        let mut token = None;
        while let Some(request) = out.pop() {
            if let Request::Unlinked { request: UnlinkedRequest::Down { down: Down::Read { read, .. } } } = request {
                token = Some(read);
            }
        }
        fixtures::root_event(
            root,
            Event::Client {
                event: ClientEvent::Up {
                    up: Up::ReadDone {
                        read: token.unwrap(),
                        result: Readout::Jig(c::ReadResult::Value(c::ReadValue::Activity(Box::new(c::ActivityPage {
                            rows: List::from_boxed(rows.into()),
                            before: None,
                            after: None,
                            cut: false,
                        })))),
                    },
                },
            },
        );
    }
    #[test]
    fn another_partys_words_are_not_drawn() {
        let root = with_activity(&[
            word(SECOND, c::Requester::Person(OTHER_PERSON), b"Another person."),
            word(FIRST, c::Requester::Task(TASK), b"A tool or agent."),
        ]);
        let tree = draw(&root, &fixtures::browser_words()).0;
        assert!(sent_lines(&tree).is_empty());
        for node in tree.nodes() {
            assert!(!node.styles.as_slice().contains(&look::GOAL_SENT));
        }
        let said = fixtures::text(&tree);
        assert!(skein_lib::bytes::find(&said, b"Another person.").is_none());
        assert!(skein_lib::bytes::find(&said, b"A tool or agent.").is_none());
    }
}


#[test]
fn how_it_runs_reads_the_asking_line() {
    for (spec, suffix) in [
        (b"Goal\nAsks: always".as_slice(), b"always asks you".as_slice()),
        (b"Goal\nIt says Asks: always", b"asks as the project says"),
    ] {
        let mut task = document();
        task.spec.specification.words = text(spec);
        let root = root(task, 0, 0, None, false);
        for words in [fixtures::browser_words(), fixtures::words()] {
            assert!(section_text(&draw(&root, &words).0, b"How it runs").ends_with(suffix));
        }
    }
}
