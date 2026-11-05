//! The goal page's reads and decisions (temper/docs/design/client/README.md, 5.8).
use super::{chats::chat_line, fixtures};
use crate::{
    Address, PlanFilter,
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
        h.event(Event::Went { address: Address::Task { task: f::TASK, filter: PlanFilter::All } });
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
    h.event(Event::Went { address: Address::Task { task: OTHER, filter: PlanFilter::All } });
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
    for (ending, expected) in [(done(), Some(Line::Landed)), (failed(), Some(Line::Failed)), (cancelled(), None)] {
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
    assert_eq!(goal::fallback(State::Stopped, &task, COUNTS), Fallback::Stopped);
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
        (State::Stopped, Landing::Stopped),
        (State::Failed, Landing::Failed),
    ] {
        assert_eq!(goal::landing(state, &task, Some(LATER), COUNTS), expected);
    }
    assert_eq!(goal::landing(State::Landed, &task, None, COUNTS), Landing::Landed(None));
}
