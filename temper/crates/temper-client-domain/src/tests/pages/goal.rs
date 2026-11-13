//! The goal page's reads and decisions (temper/docs/design/client/pages/goal.md, 2).
use super::{chats::chat_line, fixtures};
use crate::{
    Address,
    pages::{
        self, Act, Event, Pages, Request,
        goal::{self, Counts, Fallback, Landing, Line},
        goals::State,
        task::Shape,
    },
};
use fixtures::{acts, chat as f};
use jig_client::{Handle, Readable, Readout, Told, contract as c};
use skein_lib::{Duration, List, Queue, Time, Wall};
const OTHER: u64 = 89;
const LATER: Wall = Wall::from_nanos(10);
const COUNTS: Counts = Counts { done: 6, total: 9 };

struct Harness {
    pages: Pages,
    fixture: f::Fixture,
    out: Queue<Request>,
}
impl Harness {
    fn new(task: c::Task) -> Self {
        let mut h = Self { pages: Pages::new(), fixture: f::opened(), out: Queue::uncapped() };
        h.event(Event::Went { address: Address::Task { task: f::TASK } });
        h.take();
        h.deliver(task);
        h
    }
    fn event(&mut self, event: Event) {
        pages::step(&mut self.pages, &fixtures::env(&fixtures::limits()), &self.fixture.client, event, &mut self.out);
    }
    fn deliver(&mut self, task: c::Task) {
        f::deliver(&mut self.fixture.client, self.fixture.watch, task);
        self.event(Event::Told { told: Told::Changed { handle: self.pages.task().unwrap().document } });
    }
    fn answer(&mut self, handle: Handle, value: c::ReadValue) {
        self.event(Event::Told { told: Told::Page { handle, page: Readout::Jig(c::ReadResult::Value(value)) } });
    }
    fn take(&mut self) -> Box<[Request]> {
        let mut requests = List::uncapped();
        while let Some(request) = self.out.pop() {
            requests.push(request).unwrap();
        }
        requests.into_boxed()
    }
    fn goal(&self) -> &goal::Goal {
        self.pages.task().unwrap().goal.as_ref().unwrap()
    }
}
fn tracked() -> c::Task {
    let mut task = f::task();
    task.priority = Some(1);
    task
}
fn reads(requests: &[Request]) -> Box<[(Handle, c::Read)]> {
    let mut reads = List::uncapped();
    for request in requests {
        if let Request::Client(jig_client::Event::Read { handle, read: Readable::Jig(read) }) = request {
            reads.push((*handle, read.clone())).unwrap();
        }
    }
    reads.into_boxed()
}
fn report_read(requests: &[Request]) -> Option<Handle> {
    for (handle, read) in &reads(requests) {
        if let c::Read::Notes { .. } = read {
            return Some(*handle);
        }
    }
    None
}
fn line_read(requests: &[Request]) -> Option<(Handle, c::Read)> {
    for (handle, read) in &reads(requests) {
        if let c::Read::Tasks { query, .. } = read
            && query.tracked == Some(true)
        {
            return Some((*handle, read.clone()));
        }
    }
    None
}
fn row(number: u64) -> c::TaskLine {
    let mut row = chat_line();
    row.chip.task = number;
    row
}
fn cursor() -> c::Cursor {
    c::Cursor::new(b"next".as_slice().into(), 32).unwrap()
}
fn task_page(live: &[c::TaskLine], live_more: bool, ended: &[c::TaskLine], ended_more: bool) -> c::ReadValue {
    c::ReadValue::Tasks(Box::new(c::TaskPage {
        live: List::from_boxed(live.into()),
        live_after: live_more.then_some(cursor()),
        ended: c::EndedGroup::Rows {
            rows: List::from_boxed(ended.into()),
            after: ended_more.then_some(cursor()),
            cut: false,
        },
    }))
}
fn notes(headline: &[u8]) -> c::ReadValue {
    c::ReadValue::Notes(Box::new(c::NotePage {
        rows: List::from_boxed(Box::new([c::Note {
            number: 1,
            revision: c::Revision { value: 1 },
            scope: c::NoteScope::Task { project: fixtures::PROJECT, task: f::TASK },
            description: f::document_text(headline),
            body: f::document_text(b"A report."),
            writer: c::NoteAuthor::Task { task: f::TASK, attempt: 1 },
            tasks: List::with_capacity(0),
            changed: Some(LATER),
            acts: List::with_capacity(0),
            cut: false,
        }])),
        before: None,
        after: None,
        cut: false,
    }))
}
#[test]
fn a_tracked_task_reads_its_report_and_line_and_opens_its_inbox() {
    let task = tracked();
    let mut h = Harness::new(task.clone());
    let requests = h.take();
    assert_eq!(h.pages.task().unwrap().shape, Shape::Tracked);
    assert!(h.pages.task().unwrap().goal.is_some());
    let report = report_read(&requests).expect("goal report read");
    assert!(reads(&requests).contains(&(report, pages::report::read(task.project, f::TASK))));
    let (_, read) = line_read(&requests).expect("own line read");
    assert_eq!(
        read,
        c::Read::Tasks {
            query: c::TaskQuery {
                project: Some(task.project),
                requester: Some(task.requester),
                label: Some(task.label),
                tracked: Some(true),
                order: c::QueryOrder::Number,
                root: None,
                parent: None,
                phase: None,
                executor: None,
            },
            live: None,
            ended: None
        }
    );
    let inbox = h.pages.inbox().expect("goal inbox");
    assert_eq!(inbox.filter, crate::address::Filter { project: Some(task.project), kind: None, goal: Some(f::TASK) });
    assert!(requests.contains(&Request::Client(jig_client::Event::Watch {
        handle: inbox.entries,
        subject: jig_client::Watchable::Jig(c::Subject::Inbox { window: inbox.window }),
    })));
    assert!(h.pages.task().unwrap().cards.is_empty());
    assert!(core::ptr::eq(h.pages.cards().unwrap(), &raw const inbox.cards));
    h.answer(report, notes(b"report: Ready."));
    assert_eq!(h.goal().report.as_ref().unwrap().headline.as_ref(), b"Ready.");
}
#[test]
fn an_agents_task_reads_none_of_them() {
    for executor in [
        c::ChipExecutor::Agent { charter: 1 },
        c::ChipExecutor::Party { party: fixtures::PARTY },
        c::ChipExecutor::Procedure { connector: fixtures::FORGE, kind: 1 },
    ] {
        let mut task = f::task();
        task.priority = None;
        task.chip.executor = executor;
        let mut h = Harness::new(task);
        let requests = h.take();
        assert!(h.pages.task().unwrap().goal.is_none());
        assert!(h.pages.inbox().is_none());
        assert!(report_read(&requests).is_none() && line_read(&requests).is_none());
    }
}
#[test]
fn the_report_reads_again_when_last_activity_moves_and_only_then() {
    let mut task = tracked();
    let mut h = Harness::new(task.clone());
    let report = report_read(&h.take()).unwrap();
    h.deliver(task.clone());
    assert!(report_read(&h.take()).is_none());
    task.changed = LATER;
    h.deliver(task.clone());
    assert!(report_read(&h.take()).is_none());
    task.last_activity = LATER;
    h.deliver(task.clone());
    assert!(report_read(&h.take()).is_none(), "one report out");
    h.answer(report, notes(b"report: Old."));
    assert_eq!(report_read(&h.take()), Some(report), "activity moved during the read");
    h.answer(report, notes(b"report: New."));
    assert!(report_read(&h.take()).is_none());
    h.deliver(task.clone());
    assert!(report_read(&h.take()).is_none());
    task.changed = Wall::from_nanos(20);
    h.deliver(task.clone());
    assert!(report_read(&h.take()).is_none());
    task.last_activity = Wall::from_nanos(20);
    h.deliver(task.clone());
    assert_eq!(report_read(&h.take()), Some(report));
    h.answer(report, notes(b"not a report"));
    assert!(h.goal().report.is_none());
    h.take();
    h.deliver(task);
    assert!(report_read(&h.take()).is_none(), "absence is remembered");
}
#[test]
fn the_line_is_found_past_a_page() {
    let mut h = Harness::new(tracked());
    let (handle, _) = line_read(&h.take()).unwrap();
    h.answer(handle, task_page(&[row(OTHER)], true, &[], true));
    let (_, read) = line_read(&h.take()).unwrap();
    assert_eq!(read, c::Read::Tasks { query: goal::own_line_query(&tracked()), live: Some(cursor()), ended: None });
    h.answer(handle, task_page(&[], false, &[row(OTHER)], true));
    let (_, read) = line_read(&h.take()).unwrap();
    assert_eq!(read, c::Read::Tasks { query: goal::own_line_query(&tracked()), live: None, ended: Some(cursor()) });
    // An ended cursor still returns a live group; it must not send the search back to live pages.
    h.answer(handle, task_page(&[row(OTHER)], true, &[row(OTHER)], true));
    let (_, read) = line_read(&h.take()).unwrap();
    assert_eq!(read, c::Read::Tasks { query: goal::own_line_query(&tracked()), live: None, ended: Some(cursor()) });
    h.answer(handle, task_page(&[], false, &[row(f::TASK)], false));
    assert_eq!(h.goal().line.as_ref().unwrap().chip.task, f::TASK);
    assert!(line_read(&h.take()).is_none());
}
#[test]
fn the_line_is_given_up_after_eight_pages() {
    let mut h = Harness::new(tracked());
    let (handle, _) = line_read(&h.take()).unwrap();
    for page in 1_u32..=8 {
        h.answer(handle, task_page(&[row(OTHER)], true, &[], true));
        assert_eq!(line_read(&h.take()).is_some(), page < 8, "page {page}");
    }
    assert!(h.goal().line.is_none());
    h.deliver(tracked());
    assert!(line_read(&h.take()).is_none());
}
#[test]
fn the_line_reads_again_only_when_changed_or_activity_moves() {
    let mut task = tracked();
    let mut h = Harness::new(task.clone());
    let (handle, _) = line_read(&h.take()).unwrap();
    task.changed = LATER;
    h.deliver(task.clone());
    assert!(line_read(&h.take()).is_none(), "one line out");
    h.answer(handle, task_page(&[row(OTHER)], true, &[], false));
    assert_eq!(
        line_read(&h.take()).unwrap().1,
        c::Read::Tasks { query: goal::own_line_query(&task), live: None, ended: None },
        "restart a stale search"
    );
    h.answer(handle, task_page(&[row(f::TASK)], false, &[], false));
    h.take();
    h.deliver(task.clone());
    assert!(line_read(&h.take()).is_none());
    task.last_activity = LATER;
    h.deliver(task.clone());
    assert!(line_read(&h.take()).is_some());
    h.answer(handle, task_page(&[], false, &[], false));
    h.take();
    assert!(h.goal().line.is_none());
    h.deliver(task.clone());
    assert!(line_read(&h.take()).is_none(), "absence is remembered");
    task.changed = Wall::from_nanos(20);
    h.deliver(task);
    assert!(line_read(&h.take()).is_some());
}
fn ended_row(number: u64, ending: c::ChipEnding) -> c::TaskLine {
    let mut task = f::task();
    f::phase(&mut task, c::ChipPhase::Ended);
    task.chip.details.details.ending = Some(ending);
    let mut row = row(number);
    row.chip = task.chip;
    row.chip.task = number;
    row
}
fn done() -> c::ChipEnding {
    c::ChipEnding::Done(c::ResultValue::Report(f::document_text(b"Done.\nMore.")))
}
fn failed() -> c::ChipEnding {
    c::ChipEnding::Failed(f::document_text(b"No worker."))
}
fn cancelled() -> c::ChipEnding {
    c::ChipEnding::Cancelled {
        reason: f::document_text(b"Stopped."),
        result: None,
        by: c::Requester::Person(fixtures::PARTY),
        at: LATER,
    }
}
#[test]
fn the_counts_are_the_lines_and_until_then_what_the_page_holds() {
    let mut task = tracked();
    task.chip.details.details.live_delegates = 2;
    let mut h = Harness::new(task.clone());
    let (handle, _) = line_read(&h.take()).unwrap();
    let ended = task_page(&[], false, &[ended_row(1, done()), ended_row(2, failed()), ended_row(3, cancelled())], true);
    let c::ReadValue::Tasks(ended) = ended else { panic!("ended page") };
    assert_eq!(h.goal().counts(&task, None), Counts { done: 0, total: 2 });
    assert_eq!(h.goal().counts(&task, Some(&ended)), Counts { done: 1, total: 4 });
    let mut own = row(f::TASK);
    own.delegates_done = COUNTS.done;
    own.delegates_total = COUNTS.total;
    h.answer(handle, task_page(&[own], false, &[], false));
    assert_eq!(h.goal().counts(&task, Some(&ended)), COUNTS);
    assert_eq!(h.goal().state(&task, Some(&ended), true), State::Needs { funds: false });
    assert_eq!(h.goal().state(&task, None, false), State::Running);
}
#[test]
fn a_choice_on_a_goals_band_answers_through_its_inbox_card() {
    let mut h = Harness::new(tracked());
    h.take();
    let inbox = h.pages.inbox.as_mut().unwrap();
    let entries = inbox.entries;
    h.event(Event::Told { told: Told::Changed { handle: entries } });
    let query = h.pages.inbox().unwrap().queries[0];
    assert!(reads(&h.take()).contains(&(
        query,
        c::Read::Inbox {
            goal: Some(f::TASK),
            project: Some(fixtures::PROJECT),
            kind: None,
            group: c::InboxReadGroup::Needs,
            after: None
        }
    )));
    let entry = fixtures::inbox_entry(acts::QUESTION, c::InboxGroup::Questions, fixtures::PROJECT, 1);
    h.answer(
        query,
        c::ReadValue::Inbox(Box::new(c::InboxPage {
            rows: List::from_boxed(Box::new([entry])),
            selection: None,
            before: None,
            after: None,
            cut: false,
        })),
    );
    let updates = h.pages.inbox().unwrap().queries[1];
    h.event(Event::Told {
        told: Told::Page { handle: updates, page: Readout::Jig(c::ReadResult::None(c::ReadNone::Busy)) },
    });
    let card_handle = h.pages.cards().unwrap().get(0).unwrap().handle;
    let mut task = tracked();
    f::phase(&mut task, c::ChipPhase::Active(c::RunStage::Idle));
    h.deliver(task);
    h.deliver(tracked());
    let run = h.pages.task().unwrap().run.unwrap();
    assert!(run.number > card_handle.number, "task allocations follow the inbox's growing sequence");
    assert!(reads(&h.take()).contains(&(card_handle, c::Read::Question { task: acts::TASK, message: acts::MESSAGE })));
    h.answer(card_handle, acts::question());
    assert!(h.pages.cards().unwrap().get(0).unwrap().page.is_some());
    h.event(Event::Act { act: Act::CardWords { object: acts::QUESTION, words: Box::from(b"Reply".as_slice()) } });
    assert_eq!(h.pages.inbox().unwrap().cards.get(0).unwrap().words, Some(acts::text(b"Reply")));
    let updates = h.pages.inbox().unwrap().queries[1];
    h.answer(
        updates,
        c::ReadValue::Inbox(Box::new(c::InboxPage {
            rows: List::from_boxed(Box::new([fixtures::inbox_entry(
                c::ObjectKey::Result(OTHER),
                c::InboxGroup::Results,
                fixtures::PROJECT,
                2,
            )])),
            selection: None,
            before: None,
            after: None,
            cut: false,
        })),
    );
    let card_handle = h.pages.inbox().unwrap().cards.get(0).unwrap().handle;
    let second = h.pages.inbox().unwrap().cards.get(1).unwrap().handle;
    assert!(second.number > run.number, "inbox allocations follow the task's growing sequence");
    h.take();
    assert!(h.pages.task().unwrap().cards.is_empty());
    h.event(Event::Act {
        act: Act::Needs(pages::needs::Act::Choose { object: acts::QUESTION, words: Box::from(b"First".as_slice()) }),
    });
    assert!(h.pages.needs().held(acts::QUESTION));
    assert!(h.take().is_empty(), "same delayed send as Needs you");
    let mut env = fixtures::env(&fixtures::limits());
    env.now = Time::ZERO.saturating_add(Duration::from_secs(5));
    pages::fire(&mut h.pages, &env, &h.fixture.client, &mut h.out);
    assert_eq!(
        acts::sent(&mut h.out).as_ref(),
        [c::Ask::Answer {
            project: fixtures::PROJECT,
            on: c::AnswerOn::Question { task: acts::TASK, message: acts::MESSAGE, words: acts::text(b"First") },
        }]
    );
    h.event(Event::Left);
    let left = h.take();
    for handle in [card_handle, second] {
        assert!(left.contains(&Request::Client(jig_client::Event::Unwatch { handle })));
    }
}
#[test]
fn leaving_the_page_closes_its_inbox() {
    let mut h = Harness::new(tracked());
    let opened = h.take();
    let inbox = h.pages.inbox().unwrap();
    let handles = [inbox.entries, inbox.queries[0], inbox.queries[1]];
    let mut all = List::uncapped();
    for (handle, _) in &reads(&opened) {
        all.push(*handle).unwrap();
    }
    for handle in handles {
        all.push(handle).unwrap();
    }
    h.event(Event::Left);
    let left = h.take();
    for handle in &all {
        assert!(left.contains(&Request::Client(jig_client::Event::Unwatch { handle: *handle })));
    }
    assert!(h.pages.inbox().is_none());
    h.event(Event::Went { address: Address::Task { task: OTHER } });
    for handle in &all {
        assert!(h.pages.task().unwrap().document.number > handle.number);
    }
}
#[test]
fn no_longer_tracked_closes_the_inbox_and_goal_reads() {
    let mut h = Harness::new(tracked());
    h.take();
    let entries = h.pages.inbox().unwrap().entries;
    h.deliver(f::task());
    assert!(h.pages.inbox().is_none());
    assert!(h.pages.task().unwrap().goal.is_none());
    assert!(h.take().contains(&Request::Client(jig_client::Event::Unwatch { handle: entries })));
}
#[test]
fn draft_of_reads_a_lines_start_alone() {
    let path = b"docs/design/draft/away-mode.md";
    assert_eq!(
        goal::draft_of(b"Away\nDraft: docs/design/draft/away-mode.md\nChat: T4"),
        (Some(Box::from(path.as_slice())), Some(4))
    );
    for words in [b"It says Draft: wrong\nA Chat: T4".as_slice(), b" Draft: wrong\n Chat: T4", b"Neither"] {
        assert_eq!(goal::draft_of(words), (None, None));
    }
    assert_eq!(goal::draft_of(b"Chat: T4"), (None, Some(4)));
    assert_eq!(goal::draft_of(b"Draft: away.md"), (Some(Box::from(b"away.md".as_slice())), None));
    for words in [b"Chat: T4x".as_slice(), b"Chat: T", b"Chat: T18446744073709551616"] {
        assert_eq!(goal::draft_of(words).1, None);
    }
    assert_eq!(goal::file_of(path), b"away-mode.md");
    assert_eq!(goal::file_of(b"away.md"), b"away.md");
    assert_eq!(goal::file_of(b""), b"");
}
fn branch(task: &mut c::Task, parts: &[&[u8]]) {
    let mut path = List::uncapped();
    for part in parts {
        path.push(Box::from(*part)).unwrap();
    }
    task.resources = List::from_boxed(Box::new([c::ResourceLine {
        part: c::Part { connector: fixtures::FORGE, kind: c::PartKind::Resource, key: c::PartKey::Path(path) },
        kind: 1,
        taken: true,
        holder: Some(f::TASK),
        writer: None,
        writer_lost: false,
    }]));
}
#[test]
fn holds_branch_requires_equal_goal_numbers() {
    let mut task = tracked();
    task.resources.clear();
    assert!(!goal::holds_branch(&task));
    for (root, goal, expected) in [
        (b"12".as_slice(), b"g12".as_slice(), true),
        (b"12", b"g13", false),
        (b"x", b"gx", false),
        (b"012", b"g012", false),
    ] {
        branch(&mut task, &[b"forge", b"host", b"repo", b"branch", b"temper", root, goal]);
        assert_eq!(goal::holds_branch(&task), expected);
    }
    branch(&mut task, &[b"temper", b"12", b"c12"]);
    assert!(!goal::holds_branch(&task));
}
#[test]
fn line_uses_the_notes_precedence_in_every_case() {
    let words = f::document_text(b"Done.");
    let part = c::Part { connector: 1, kind: c::PartKind::Result, key: c::PartKey::Number(1) };
    for (ending, expected) in [
        (
            c::ChipEnding::Done(c::ResultValue::Change { kind: 1, part: part.clone(), words: words.clone() }),
            Some(Line::Landed),
        ),
        (c::ChipEnding::Done(c::ResultValue::Change { kind: 2, part, words: words.clone() }), Some(Line::Done)),
        (done(), Some(Line::Done)),
        (c::ChipEnding::Done(c::ResultValue::Words(words.clone())), Some(Line::Done)),
        (c::ChipEnding::Done(c::ResultValue::Failure(words.clone())), Some(Line::Done)),
        (c::ChipEnding::Done(c::ResultValue::Approved { state: Box::default(), words: None }), Some(Line::Done)),
        (
            c::ChipEnding::Done(c::ResultValue::OwnerVerdict {
                verdict: 1,
                words: words.clone(),
                fields: List::uncapped(),
            }),
            Some(Line::Done),
        ),
        (
            c::ChipEnding::Done(c::ResultValue::Verdict {
                verdict: 1,
                words: words.clone(),
                fields: List::uncapped(),
                tasks: List::uncapped(),
                proposal: None,
            }),
            Some(Line::Done),
        ),
        (
            c::ChipEnding::Done(c::ResultValue::Connector {
                summary: words.clone(),
                part: c::ApplicationPart::Result { connector: 1, task: 1 },
            }),
            Some(Line::Done),
        ),
        (
            c::ChipEnding::Done(c::ResultValue::Choice {
                option: 1,
                by: c::Party { number: 1, kind: c::PartyKind::Person, name: acts::text(b"Person") },
                reason: words,
            }),
            Some(Line::Done),
        ),
        (failed(), Some(Line::Failed)),
        (cancelled(), None),
    ] {
        let chip = ended_row(1, ending).chip;
        assert_eq!(goal::line(&chip, true), expected);
    }
    let mut task = tracked();
    assert_eq!(goal::line(&task.chip, true), Some(Line::Running));
    f::phase(&mut task, c::ChipPhase::Held { reason: c::HoldCategory::Stopped, revision: 1 });
    assert_eq!(goal::line(&task.chip, true), Some(Line::WaitsOnYou));
    assert_eq!(goal::line(&task.chip, false), Some(Line::Held));
    for (dependencies, more, holds, expected) in [
        (&[OTHER][..], false, false, Line::WaitsOn(OTHER)),
        (&[1, 2][..], false, false, Line::WaitsOnTasks(2)),
        (&[OTHER][..], true, true, Line::WaitsOnTasks(1)),
        (&[][..], true, false, Line::WaitsOnTasks(0)),
        (&[][..], false, true, Line::WaitsOnResources),
        (&[][..], false, false, Line::Waits),
    ] {
        f::phase(&mut task, c::ChipPhase::Waiting { dependencies: List::from_boxed(dependencies.into()), more, holds });
        assert_eq!(goal::line(&task.chip, false), Some(expected));
        assert_eq!(goal::line(&task.chip, true), Some(Line::WaitsOnYou));
    }
}
#[test]
fn fallback_in_each_state() {
    let mut task = tracked();
    task.spec.specification.words = f::document_text(b"Draft: docs/draft/away.md");
    let planning = Fallback::Planning { draft: Some(Box::from(b"away.md".as_slice())) };
    for state in
        [State::Needs { funds: false }, State::Needs { funds: true }, State::Paused, State::Running, State::Planning]
    {
        assert_eq!(goal::fallback(state, &task, Counts { done: 0, total: 0 }), planning);
        assert_eq!(
            goal::fallback(state, &task, COUNTS),
            if state == State::Planning { planning.clone() } else { Fallback::Tasks(COUNTS) }
        );
    }
    assert_eq!(goal::fallback(State::Closed, &task, COUNTS), Fallback::Closed);
    task.result = Some(c::TaskResult::Ending(Box::new(done())));
    assert_eq!(
        goal::fallback(State::Landed, &task, COUNTS),
        Fallback::Landed { first_line: Some(Box::from(b"Done.".as_slice())) }
    );
    task.result = Some(c::TaskResult::Ending(Box::new(failed())));
    assert_eq!(
        goal::fallback(State::Failed, &task, COUNTS),
        Fallback::Failed { reason: Box::from(b"No worker.".as_slice()) }
    );
    task.result = None;
    task.chip.details.details.ending = None;
    assert_eq!(goal::fallback(State::Landed, &task, COUNTS), Fallback::Landed { first_line: None });
    task.spec.specification.words = f::document_text(b"Build it.");
    assert_eq!(goal::fallback(State::Planning, &task, COUNTS), Fallback::Planning { draft: None });
}
#[test]
fn landing_in_each_state() {
    let mut task = tracked();
    task.resources.clear();
    for state in
        [State::Needs { funds: false }, State::Needs { funds: true }, State::Paused, State::Planning, State::Running]
    {
        assert_eq!(goal::landing(state, &task, Some(LATER), COUNTS), Landing::Each);
        branch(&mut task, &[b"temper", b"12", b"g12"]);
        assert_eq!(goal::landing(state, &task, None, COUNTS), Landing::Whole { total: COUNTS.total });
        assert_eq!(goal::landing(state, &task, None, Counts { done: 0, total: 0 }), Landing::Whole { total: 0 });
        task.resources.clear();
    }
    for (state, expected) in [
        (State::Landed, Landing::Landed(Some(LATER))),
        (State::Closed, Landing::Closed),
        (State::Failed, Landing::Failed),
    ] {
        assert_eq!(goal::landing(state, &task, Some(LATER), COUNTS), expected);
    }
    assert_eq!(goal::landing(State::Landed, &task, None, COUNTS), Landing::Landed(None));
}

fn with_offer(kind: c::ActKind) -> c::Task {
    let mut task = tracked();
    task.acts = List::from_boxed(Box::new([c::Offered { act: kind, admission: c::Admission::Allowed, effect: None }]));
    task
}
pub(super) fn paused_task() -> c::Task {
    let mut task = with_offer(c::ActKind::Release);
    f::phase(&mut task, c::ChipPhase::Held { reason: c::HoldCategory::Stopped, revision: 17 });
    task.chip.details.details.hold = Some(c::ChipHold {
        placed: Some(c::PlacedHold {
            number: 1,
            owner: c::HoldOwner::Party { person: fixtures::PARTY },
            mode: c::HoldMode::Stop,
            scope: c::HoldScope::Subtree { task: f::TASK },
            reason: Box::default(),
            placed_at: LATER,
            revision: c::Revision { value: 11 },
            lifted: false,
            may_lift: true,
        }),
        words: None,
        circle: None,
        account: None,
        source: None,
        reason: c::HoldCategory::Stopped,
        revision: 17,
        before: c::PriorPhase::Active(c::RunStage::Idle),
        escalation: None,
        tries: c::FailureTries { transient: 0, permanent: 0, run: 0, agent: 0, lost: 0, invalid: 0, infrastructure: 0 },
        hold_wait_since: None,
    });
    task
}
fn page_act(h: &mut Harness, act: Act) -> Box<[c::Ask]> {
    h.event(Event::Act { act });
    acts::sent(&mut h.out)
}
#[test]
fn pause_sends_stop_at_once() {
    let mut h = Harness::new(with_offer(c::ActKind::Stop));
    assert_eq!(page_act(&mut h, Act::Pause).as_ref(), [c::Ask::Stop { project: fixtures::PROJECT, task: f::TASK }]);
    assert!(h.pages.confirmation().is_none());
    assert!(h.pages.needs().toast().is_none());
}
#[test]
fn resume_names_the_holds_revision() {
    let mut h = Harness::new(paused_task());
    assert_eq!(
        page_act(&mut h, Act::Resume).as_ref(),
        [c::Ask::Release {
            project: fixtures::PROJECT,
            task: f::TASK,
            revision: c::Revision { value: 17 },
            again: None,
        }]
    );
    assert!(h.pages.confirmation().is_none());
}
#[test]
fn a_double_press_joins_the_pending_request() {
    for (task, act) in [(with_offer(c::ActKind::Stop), Act::Pause), (paused_task(), Act::Resume)] {
        let mut h = Harness::new(task);
        assert_eq!(page_act(&mut h, act.clone()).len(), 1);
        let pending = h.pages.task().unwrap().pending.clone();
        assert!(page_act(&mut h, act).is_empty());
        assert_eq!(h.pages.task().unwrap().pending, pending);
    }
}
#[test]
fn a_done_pause_toasts_on_its_answer_not_before() {
    for (task, act, toasted) in [
        (with_offer(c::ActKind::Stop), Act::Pause, pages::needs::Toasted::Paused),
        (paused_task(), Act::Resume, pages::needs::Toasted::Resumed),
    ] {
        let mut h = Harness::new(task);
        page_act(&mut h, act.clone());
        let key = h.pages.task().unwrap().pending.get(0).unwrap().0;
        h.event(Event::Told { told: Told::Pending { key } });
        assert!(h.pages.needs().toast().is_none());
        h.event(Event::Told {
            told: Told::Answered {
                key,
                outcome: jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Done(
                    c::Outcome::Document(if act == Act::Pause {
                        c::DocumentOutcome::Stopped { task: f::TASK }
                    } else {
                        c::DocumentOutcome::Released { task: f::TASK }
                    }),
                ))),
            },
        });
        assert_eq!(h.pages.needs().toast(), Some(&toasted));
        assert!(h.pages.task().unwrap().outcome.is_none());
        assert!(h.pages.task().unwrap().pending.is_empty());
        let mut env = fixtures::env(&fixtures::limits());
        env.now = Time::ZERO.saturating_add(pages::needs::TOAST);
        pages::fire(&mut h.pages, &env, &h.fixture.client, &mut h.out);
        assert!(h.pages.needs().toast().is_none());
    }
}
#[test]
fn a_refused_stop_keeps_jigs_words_in_its_sheet() {
    for outcome in [
        jig_client::Outcome::Unknown,
        jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Refused {
            refusal: c::Refusal::Moved { now: c::Revision { value: 18 }, what: c::Moved::Spec },
            at: None,
        })),
    ] {
        let mut h = Harness::new(with_offer(c::ActKind::Cancel));
        page_act(&mut h, Act::Close);
        page_act(&mut h, Act::Confirm);
        let key = h.pages.task().unwrap().pending.get(0).unwrap().0;
        assert!(h.pages.needs().toast().is_none());
        h.event(Event::Told { told: Told::Answered { key, outcome: outcome.clone() } });
        assert!(h.pages.needs().toast().is_none());
        assert_eq!(h.pages.confirmation().unwrap().failed.as_ref(), Some(&outcome));
        assert!(h.pages.task().unwrap().outcome.is_none());
        page_act(&mut h, Act::Needs(pages::needs::Act::Dismiss));
        assert!(h.pages.needs().toast().is_none());
    }
}
#[test]
fn close_opens_its_confirmation_and_sends_its_reason() {
    let mut h = Harness::new(with_offer(c::ActKind::Cancel));
    assert!(page_act(&mut h, Act::Close).is_empty());
    assert_eq!(h.pages.confirmation().unwrap().act, c::ActKind::Cancel);
    assert_eq!(
        page_act(&mut h, Act::Confirm).as_ref(),
        [c::Ask::Cancel {
            project: fixtures::PROJECT,
            task: f::TASK,
            reason: acts::text(b"Closed from its page"),
            grace: None,
        }]
    );
}
#[test]
fn resume_is_refused_unless_paused() {
    let mut held = paused_task();
    held.chip.details.details.hold.as_mut().unwrap().placed.as_mut().unwrap().lifted = true;
    for task in [with_offer(c::ActKind::Release), held] {
        let mut h = Harness::new(task);
        assert!(page_act(&mut h, Act::Resume).is_empty());
        assert!(h.pages.task().unwrap().pending.is_empty());
    }
    let mut h = Harness::new(paused_task());
    h.pages
        .inbox
        .as_mut()
        .unwrap()
        .cards
        .push(pages::inbox::Card {
            whole: None,
            object: c::ObjectKey::Proposal(acts::PROPOSAL),
            group: c::InboxGroup::Proposals,
            project: Some(fixtures::PROJECT),
            position: None,
            unread: None,
            handle: Handle { asker: crate::owner::Owner::Pages.asker(), number: 700 },
            page: None,
            words: None,
            key: None,
            outcome: None,
        })
        .unwrap();
    assert!(page_act(&mut h, Act::Resume).is_empty(), "an open band makes the goal needs, not paused");
}

const MESSAGE_WORDS: &[u8] = b"Keep it small.";
// One Send admits one message snapshot.
const SENT_MESSAGES: u32 = 1;
fn messenger() -> Harness {
    let mut h = Harness::new(with_offer(c::ActKind::Message));
    h.take();
    h.fixture.client.step_unlinked(
        Time::ZERO,
        Wall::EPOCH,
        jig_client::Event::Watch { handle: pages::me_handle(), subject: jig_client::Watchable::Jig(c::Subject::Me) },
        &mut Queue::uncapped(),
    );
    h.fixture.client.step_unlinked(
        Time::ZERO,
        Wall::EPOCH,
        jig_client::Event::Up {
            up: jig_client::Up::Watched {
                watch: h.fixture.me,
                event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(fixtures::chats::me()))),
            },
        },
        &mut Queue::uncapped(),
    );
    h
}
#[test]
fn send_routes_on_a_tracked_task() {
    let mut h = messenger();
    let _draft = page_act(&mut h, Act::MessageWords { words: Box::from(MESSAGE_WORDS) });
    assert_eq!(
        page_act(&mut h, Act::Send).as_ref(),
        [c::Ask::Message { project: fixtures::PROJECT, task: f::TASK, words: acts::text(MESSAGE_WORDS) }]
    );
    let page = h.pages.task().unwrap();
    assert!(page.words.is_empty());
    assert_eq!(page.thread.words.rows.len(), SENT_MESSAGES);
    assert_eq!(page.thread.words.rows.get(0).unwrap().state, jig_client::thread::words::Words::BeingSent);
    assert!(page_act(&mut h, Act::Send).is_empty());
}
#[test]
fn tracked_activity_reads_again_when_its_position_moves() {
    let mut task = tracked();
    let mut h = Harness::new(task.clone());
    h.take();
    let activity = h.pages.task().unwrap().activity;
    h.answer(
        activity,
        c::ReadValue::Activity(Box::new(c::ActivityPage {
            rows: List::uncapped(),
            before: None,
            after: None,
            cut: false,
        })),
    );
    h.take();
    task.activity += 1;
    h.deliver(task.clone());
    assert!(reads(&h.take()).contains(&(activity, c::Read::Activity { task: f::TASK, before: None })));
    h.deliver(task);
    for (handle, _) in &reads(&h.take()) {
        assert_ne!(*handle, activity);
    }
}

#[test]
fn a_refused_goal_message_uses_the_toast() {
    let mut h = messenger();
    page_act(&mut h, Act::MessageWords { words: Box::from(MESSAGE_WORDS) });
    page_act(&mut h, Act::Send);
    let key = h.pages.task().unwrap().pending.get(0).unwrap().0;
    let outcome = jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Refused {
        refusal: c::Refusal::NotWaiting,
        at: None,
    }));
    h.event(Event::Told { told: Told::Answered { key, outcome: outcome.clone() } });
    assert_eq!(h.pages.needs().toast(), Some(&pages::needs::Toasted::Refused { outcome }));
    let page = h.pages.task().unwrap();
    assert!(page.outcome.is_none());
    assert!(page.pending.is_empty());
    assert!(page.thread.words.rows.is_empty());
}

#[test]
fn answered_goal_messages_release_the_pending_word_room() {
    let mut h = messenger();
    // One more acknowledgment than the pending-word capacity must still send.
    let attempts = h.pages.task().unwrap().thread.words.rows.capacity() + 1;
    for message in 1..=attempts {
        page_act(&mut h, Act::MessageWords { words: Box::from(MESSAGE_WORDS) });
        assert_eq!(page_act(&mut h, Act::Send).len(), usize::try_from(SENT_MESSAGES).unwrap());
        let key = h.pages.task().unwrap().pending.get(0).unwrap().0;
        h.event(Event::Told {
            told: Told::Answered {
                key,
                outcome: jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Done(
                    c::Outcome::Document(c::DocumentOutcome::Said { task: f::TASK, message: u64::from(message) }),
                ))),
            },
        });
        assert!(h.pages.task().unwrap().thread.words.rows.is_empty());
    }
}

fn cut_specification() -> c::Task {
    let mut task = tracked();
    task.spec.specification.words = f::document_text(b"A long draft");
    task.spec.specification.words.cut = true;
    task
}
fn source_read(requests: &[Request]) -> Option<(Handle, u64)> {
    for (handle, read) in &reads(requests) {
        if let c::Read::TaskText { task: f::TASK, field: c::TaskTextField::Specification, from } = read {
            return Some((*handle, *from));
        }
    }
    None
}
fn source(from: u64, total: u64, next: Option<u64>, text: &[u8]) -> c::ReadValue {
    c::ReadValue::TaskText(Box::new(c::TaskText {
        task: f::TASK,
        field: c::TaskTextField::Specification,
        from,
        total,
        next,
        text: c::Text::new(text.into(), u32::try_from(text.len()).unwrap()).unwrap(),
    }))
}
#[test]
fn a_cut_specification_reads_its_provenance() {
    const FIRST: &[u8] = b"Work\nDraft: ";
    const LAST: &[u8] = b"away.md\nChat: T4";
    let total = skein_lib::len_u64(FIRST.len() + LAST.len());
    let offset = skein_lib::len_u64(FIRST.len());
    for failure in [
        None,
        Some((0, c::ReadNone::Unreachable)),
        Some((offset, c::ReadNone::Unreachable)),
        Some((offset, c::ReadNone::Busy)),
    ] {
        let mut h = Harness::new(cut_specification());
        let (handle, from) =
            source_read(&h.take()).expect("cut goal words need their source before finding Draft: and Chat: lines");
        assert_eq!(from, 0);
        for (from, next, text) in [(0, Some(offset), FIRST), (offset, None, LAST)] {
            if let Some((failed_at, why)) = failure
                && failed_at == from
            {
                retry_source(&mut h, handle, from, why);
            }
            h.answer(handle, source(from, total, next, text));
            if let Some(next) = next {
                assert_eq!(h.goal().draft, (None, None), "a partial line is not provenance");
                assert_eq!(source_read(&h.take()), Some((handle, next)));
            }
        }
        assert_eq!(h.goal().draft, (Some(Box::from(b"away.md".as_slice())), Some(4)));
        assert_eq!(source_read(&h.take()), None);
        assert_eq!(pages::next_deadline(&h.pages), None, "a completed source stops retrying");
    }
}

fn retry_source(h: &mut Harness, handle: Handle, from: u64, why: c::ReadNone) {
    let mut env = fixtures::env(&fixtures::limits());
    for seconds in [1, 2] {
        pages::step(
            &mut h.pages,
            &env,
            &h.fixture.client,
            Event::Told { told: Told::Page { handle, page: Readout::Jig(c::ReadResult::None(why)) } },
            &mut h.out,
        );
        let due = env.now.saturating_add(Duration::from_secs(seconds));
        assert_eq!(pages::next_deadline(&h.pages), Some(due), "a transient source failure is retried with backoff");
        pages::fire(&mut h.pages, &env, &h.fixture.client, &mut h.out);
        assert_eq!(source_read(&h.take()), None, "no early retry");
        env.now = due;
        pages::fire(&mut h.pages, &env, &h.fixture.client, &mut h.out);
        assert_eq!(source_read(&h.take()), Some((handle, from)), "resume the same source chunk");
    }
}

#[test]
fn specification_sources_stop_at_the_long_text_bound() {
    // fixtures::limits sets long_text to 32 bytes.
    for (words, expected) in [
        (b"Draft: a.md\nChat: T4\n12345678901".as_slice(), (Some(Box::from(b"a.md".as_slice())), Some(4))),
        (b"Draft: a.md\nChat: T4\n123456789012".as_slice(), (None, None)),
    ] {
        let mut h = Harness::new(cut_specification());
        let (handle, _) = source_read(&h.take()).unwrap();
        h.answer(handle, source(0, skein_lib::len_u64(words.len()), None, words));
        assert_eq!(h.goal().draft, expected);
        assert_eq!(source_read(&h.take()), None);
    }
}

#[test]
fn the_line_reads_always() {
    for words in [b"Asks: always".as_slice(), b"Goal\nAsks: always\nChat: T4", b"Goal\nAsks: always"] {
        assert!(goal::always_asks(words));
    }
}
#[test]
fn no_line_follows_the_switch() {
    for words in [b"".as_slice(), b"Goal\nDraft: away.md", b"Asks: away"] {
        assert!(!goal::always_asks(words));
    }
}
#[test]
fn a_line_inside_a_sentence_doesnt_count() {
    for words in [b"It says Asks: always".as_slice(), b" Asks: always", b"Asks: always later", b"Asks: always "] {
        assert!(!goal::always_asks(words));
    }
}
