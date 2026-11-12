use super::fixtures::{self, chat as f};
use crate::pages::{
    Request,
    task::{Shape, Task},
};
use jig_client::{Event, Readable, Told, Watchable, contract as c};
use skein_lib::{Queue, Time};
const OUTPUTS: u32 = 16;
const WINDOW: u32 = 2;
const EMPTY_OFFSET: u32 = 0;
const FIRST: u64 = f::FIRST;
const PRIORITY: u32 = 1;
fn opened() -> (Task, f::Fixture) {
    let mut fixture = f::opened();
    let mut page = Task::new(f::TASK, FIRST, &fixtures::limits(), &mut Queue::with_capacity(OUTPUTS));
    while fixture.out.pop().is_some() {}
    page.told(Time::ZERO, &fixture.client, Told::Changed { handle: page.document }, &mut fixture.out);
    (page, fixture)
}
#[test]
fn opening_a_task_watches_task_and_tree_and_reads_activity() {
    let mut out = Queue::with_capacity(OUTPUTS);
    let page = Task::new(f::TASK, FIRST, &fixtures::limits(), &mut out);
    assert_eq!(
        out.pop(),
        Some(Request::Client(Event::Watch {
            handle: page.document,
            subject: Watchable::Jig(c::Subject::Task { task: f::TASK }),
        }))
    );
    assert_eq!(
        out.pop(),
        Some(Request::Client(Event::Watch {
            handle: page.tree,
            subject: Watchable::Jig(c::Subject::Tree { task: f::TASK, window: WINDOW }),
        }))
    );
    assert_eq!(
        out.pop(),
        Some(Request::Client(Event::Read {
            handle: page.activity,
            read: Readable::Jig(c::Read::Activity { task: f::TASK, before: None }),
        }))
    );
    assert!(out.is_empty());
}
#[test]
fn a_live_run_on_a_tasks_page_is_watched() {
    let (page, mut f) = opened();
    assert_eq!(
        f.out.pop(),
        Some(Request::Client(Event::Watch {
            handle: page.run.expect("run handle"),
            subject: Watchable::Jig(c::Subject::Run { task: f::TASK, turn: None, offset: EMPTY_OFFSET }),
        }))
    );
}
#[test]
fn a_tasks_live_run_is_unwatched_when_it_ends() {
    let (mut page, mut f) = opened();
    let run = page.run.expect("live run");
    while f.out.pop().is_some() {}
    let mut task = f::task();
    f::phase(&mut task, c::ChipPhase::Active(c::RunStage::Idle));
    f::deliver(&mut f.client, f.watch, task);
    page.told(Time::ZERO, &f.client, Told::Changed { handle: page.document }, &mut f.out);
    assert_eq!(page.run, None);
    assert_eq!(f::handles(&mut f.out).as_ref(), [run]);
}
#[test]
fn leaving_a_task_unwatches_each_of_its_handles() {
    let (page, mut f) = opened();
    while f.out.pop().is_some() {}
    page.leave(&mut f.out);
    assert_eq!(f::handles(&mut f.out).as_ref(), [page.document, page.tree, page.activity, page.run.expect("run")]);
}
#[test]
fn an_agents_task_without_a_priority_has_the_agent_shape() {
    const CHARTER: u32 = 1;
    const PARTY: u64 = 73;
    const PROCEDURE: u32 = 2;
    const CASES: &[(Option<u32>, c::ChipExecutor, Shape)] = &[
        (Some(PRIORITY), c::ChipExecutor::Agent { charter: CHARTER }, Shape::Tracked),
        (None, c::ChipExecutor::Agent { charter: CHARTER }, Shape::Agent),
        (
            Some(PRIORITY),
            c::ChipExecutor::Procedure { connector: fixtures::FORGE, kind: PROCEDURE },
            Shape::HeaderAndPlan,
        ),
        (Some(PRIORITY), c::ChipExecutor::Party { party: PARTY }, Shape::HeaderAndPlan),
        (None, c::ChipExecutor::Procedure { connector: fixtures::FORGE, kind: PROCEDURE }, Shape::HeaderAndPlan),
        (None, c::ChipExecutor::Party { party: PARTY }, Shape::HeaderAndPlan),
        (None, c::ChipExecutor::Role { project: fixtures::PROJECT, role: 1 }, Shape::HeaderAndPlan),
    ];
    for (priority, executor, expected) in CASES {
        let (mut page, mut f) = opened();
        let mut task = f::task();
        task.priority = *priority;
        task.chip.executor = *executor;
        f::deliver(&mut f.client, f.watch, task);
        page.told(Time::ZERO, &f.client, Told::Changed { handle: page.document }, &mut f.out);
        assert_eq!(page.shape, *expected, "{priority:?}, {executor:?}");
    }
}
const HELD: u64 = 12;
const FIRST_CARD: u32 = 0;
#[test]
fn releasing_from_the_plan_names_the_holds_revision_the_card_showed() {
    use crate::pages::Act;
    let mut pages = fixtures::task::held_pages();
    let mut out = fixtures::acts::act(
        &mut pages,
        Act::Decide {
            object: c::ObjectKey::Task(HELD),
            revision: Some(fixtures::acts::REVISION),
            act: c::ActKind::Release,
        },
    );
    assert!(fixtures::acts::sent(&mut out).is_empty(), "release waits for confirmation");
    let mut out = fixtures::acts::act(&mut pages, Act::Confirm);
    assert_eq!(
        fixtures::acts::sent(&mut out).as_ref(),
        [c::Ask::Release { project: fixtures::PROJECT, task: HELD, revision: fixtures::acts::REVISION, again: None }]
    );
    assert!(fixtures::acts::sent(&mut fixtures::acts::act(&mut pages, Act::Confirm)).is_empty());
}
#[test]
fn a_task_held_anew_reads_its_released_card_again() {
    let (mut page, mut fixture) = opened();
    let mut task = f::task();
    let mut held = |revision| {
        f::phase(&mut task, c::ChipPhase::Held { reason: c::HoldCategory::Stopped, revision });
        task.chip.clone()
    };
    page.held(&held(fixtures::acts::REVISION.value), &mut fixture.out);
    let card = page.cards.get_mut(FIRST_CARD).expect("held card");
    let first = card.handle;
    let c::ReadValue::Escalation(read) = fixtures::acts::held(c::ActKind::Release) else { panic!("hold") };
    let mut stopped = read.task.hold.expect("owner hold");
    stopped.revision = fixtures::acts::REVISION.value;
    let mut document = f::task();
    document.hold = Some(stopped);
    card.page = Some(c::ReadValue::Task(Box::new(document)));
    card.outcome = Some(jig_client::Outcome::Unknown);
    while fixture.out.pop().is_some() {}
    page.held(&held(fixtures::acts::REVISION.value), &mut fixture.out);
    assert!(fixture.out.pop().is_none(), "the same hold keeps its card");
    page.held(&held(fixtures::acts::NEW_REVISION.value), &mut fixture.out);
    let card = page.cards.get(FIRST_CARD).expect("held card");
    assert_eq!(page.cards.len(), 1);
    assert_ne!(card.handle, first);
    assert!(card.page.is_none() && card.outcome.is_none());
    assert_eq!(fixture.out.pop(), Some(Request::Client(Event::Unwatch { handle: first })),);
}
#[test]
fn a_moved_hold_is_refused_and_the_card_asks_to_look_again() {
    use crate::pages::Act;
    let mut pages = fixtures::task::held_pages();
    fixtures::acts::act(
        &mut pages,
        Act::Decide {
            object: c::ObjectKey::Task(HELD),
            revision: Some(fixtures::acts::REVISION),
            act: c::ActKind::Release,
        },
    );
    fixtures::acts::act(&mut pages, Act::Confirm);
    let key = pages.cards().expect("cards").get(FIRST_CARD).expect("held").key.expect("sent");
    let outcome = jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Refused {
        refusal: c::Refusal::Moved { now: fixtures::acts::NEW_REVISION, what: c::Moved::Hold { reason: None } },
        at: None,
    }));
    crate::acts::told(&mut pages, &Told::Answered { key, outcome: outcome.clone() });
    assert_eq!(pages.cards().expect("cards").get(FIRST_CARD).expect("held").outcome, Some(outcome));
    assert!(fixtures::acts::sent(&mut fixtures::acts::act(&mut pages, Act::Confirm)).is_empty());
}

use crate::pages::task::{State, header_state, link_state, row_state};
use skein_lib::{List, Wall};
const DEPENDENCY: u64 = 17;
const OTHER_DEPENDENCY: u64 = 23;
#[test]
fn a_tasks_state_says_what_it_waits_for() {
    let cases: &[(&[u64], bool, bool, State)] = &[
        (&[DEPENDENCY], false, false, State::WaitingFor(DEPENDENCY)),
        (&[DEPENDENCY], false, true, State::WaitingFor(DEPENDENCY)),
        (&[DEPENDENCY, OTHER_DEPENDENCY], false, false, State::WaitingForTasks { count: 2, more: false }),
        (&[DEPENDENCY], true, true, State::WaitingForTasks { count: 1, more: true }),
        (&[], true, true, State::WaitingForTasks { count: 0, more: true }),
        (&[], false, true, State::WaitingForHoldings),
        (&[], false, false, State::Phase(c::Phase::Waiting)),
    ];
    for (numbers, more, holds, expected) in cases {
        let mut task = f::task();
        let mut dependencies = List::with_capacity(WINDOW);
        for number in *numbers {
            dependencies.push(*number).expect("waiting dependencies");
        }
        f::phase(&mut task, c::ChipPhase::Waiting { dependencies, more: *more, holds: *holds });
        assert_eq!(row_state(&task.chip), expected.clone());
    }
}
#[test]
fn an_ended_task_says_its_owners_ending() {
    for (ending, expected) in [
        (
            Some(c::ChipEnding::Done(c::ResultValue::Report(f::document_text(b"Done")))),
            State::Ended(c::TaskStatus::Done),
        ),
        (Some(c::ChipEnding::Failed(f::document_text(b"Failed"))), State::Ended(c::TaskStatus::Failed)),
        (
            Some(c::ChipEnding::Cancelled {
                reason: f::document_text(b"Cancelled"),
                result: None,
                by: c::Requester::Person(fixtures::PARTY),
                at: Wall::EPOCH,
            }),
            State::Ended(c::TaskStatus::Cancelled),
        ),
        (None, State::Phase(c::Phase::Ended)),
    ] {
        let mut task = f::task();
        f::phase(&mut task, c::ChipPhase::Ended);
        let c::ChipPhaseFacts { details, .. } = task.chip.details.as_mut();
        details.ending = ending;
        assert_eq!(row_state(&task.chip), expected);
        task.waits_on = c::WaitsOn::News;
        assert_eq!(header_state(&task, None), expected);
    }
}
#[test]
fn another_phase_says_jigs_phase() {
    let mut task = f::task();
    f::phase(&mut task, c::ChipPhase::Active(c::RunStage::Idle));
    let c::ChipPhaseFacts { details, .. } = task.chip.details.as_mut();
    details.ending = Some(c::ChipEnding::Failed(f::document_text(b"Old ending")));
    assert_eq!(row_state(&task.chip), State::Phase(task.chip.phase));
    task.chip.phase = c::Phase::Waiting;
    task.chip.details.phase = c::ChipPhase::Waiting { dependencies: List::with_capacity(0), more: false, holds: false };
    assert_eq!(row_state(&task.chip), State::Phase(c::Phase::Waiting));
}
#[test]
fn a_dependencys_state_comes_from_its_link() {
    let mut task = f::task();
    let mut dependencies = List::with_capacity(1);
    dependencies.push(DEPENDENCY).expect("dependency");
    f::phase(&mut task, c::ChipPhase::Waiting { dependencies, more: false, holds: false });
    let chip = task.chip;
    let expected = State::WaitingFor(DEPENDENCY);
    assert_eq!(link_state(&c::TaskLink::Live(chip.clone())), Some(expected.clone()));
    assert_eq!(link_state(&c::TaskLink::Retained { chip, at: Some(Wall::EPOCH) }), Some(expected.clone()));
    for status in [c::TaskStatus::Done, c::TaskStatus::Failed, c::TaskStatus::Cancelled] {
        assert_eq!(
            link_state(&c::TaskLink::Stored { number: DEPENDENCY, status: Some(status) }),
            Some(State::Ended(status))
        );
    }
    assert_eq!(link_state(&c::TaskLink::Stored { number: DEPENDENCY, status: None }), None);
}

const ATTEMPT: u64 = f::ATTEMPT;
fn observation(work: c::ChipWork) -> c::ChipObservation {
    c::ChipObservation {
        channel: skein_lib::Token::new(1),
        work,
        since: Wall::EPOCH,
        reported_at: Wall::EPOCH,
        started: None,
    }
}
#[test]
fn a_running_rows_state_says_its_observed_work() {
    for (work, expected) in [
        (None, State::Running),
        (Some(c::ChipWork::Writing), State::Writing),
        (
            Some(c::ChipWork::Tool { call: Box::from(*b"call"), tool: Box::from(*b"read_file") }),
            State::Tool(Box::from(*b"read_file")),
        ),
        (Some(c::ChipWork::Preparing), State::Running),
        (Some(c::ChipWork::Starting), State::Running),
        (Some(c::ChipWork::Active), State::Running),
        (Some(c::ChipWork::Waiting), State::Running),
        (Some(c::ChipWork::Ending), State::Running),
    ] {
        let mut task = f::task();
        f::phase(&mut task, c::ChipPhase::Active(c::RunStage::Running { attempt: ATTEMPT }));
        let c::ChipPhaseFacts { details, .. } = task.chip.details.as_mut();
        if let Some(work) = work {
            details.observation = Some(observation(work));
        } else {
            details.observation = None;
        }
        assert_eq!(row_state(&task.chip), expected);
        task.waits_on = c::WaitsOn::Nothing;
        assert_eq!(header_state(&task, None), expected);
    }
}
fn pending(numbers: &[u64]) -> List<u64> {
    let mut pending = List::with_capacity(WINDOW);
    for number in numbers {
        pending.push(*number).expect("pending task");
    }
    pending
}
#[test]
fn the_headers_dependencies_use_the_supplied_exact_count() {
    for (numbers, count, expected) in [
        (&[DEPENDENCY][..], 1, State::WaitingFor(DEPENDENCY)),
        (&[DEPENDENCY, OTHER_DEPENDENCY][..], 2, State::WaitingForTasks { count: 2, more: false }),
        (&[DEPENDENCY][..], 3, State::WaitingForTasks { count: 3, more: false }),
        (&[][..], 1, State::WaitingForTasks { count: 1, more: false }),
    ] {
        let mut task = f::task();
        task.waits_on = c::WaitsOn::Dependencies { pending: pending(numbers), count };
        assert_eq!(header_state(&task, None), expected);
    }
}
#[test]
fn the_headers_delegates_use_the_chips_live_count() {
    for count in [0, 1, 3] {
        let mut task = f::task();
        task.waits_on = c::WaitsOn::Delegates;
        let c::ChipPhaseFacts { details, .. } = task.chip.details.as_mut();
        details.live_delegates = count;
        assert_eq!(header_state(&task, None), State::WaitingForDelegates(count));
    }
}
fn party_wait(whom: c::Whom) -> c::WaitsOn {
    c::WaitsOn::Party { whom, entries: 1, task: DEPENDENCY, kind: c::WaitedFor::Question }
}
#[test]
fn the_headers_party_is_you_only_for_the_person_or_a_held_project_role() {
    let mut task = f::task();
    let mut me = fixtures::chats::me();
    task.waits_on = party_wait(c::Whom::Person(me.party.number));
    assert_eq!(header_state(&task, Some(&me)), State::WaitingForYou);
    assert_eq!(header_state(&task, None), State::WaitingForParty(c::Whom::Person(me.party.number)));
    task.waits_on = party_wait(c::Whom::Person(DEPENDENCY));
    assert_eq!(header_state(&task, Some(&me)), State::WaitingForParty(c::Whom::Person(DEPENDENCY)));
    for (held, number) in [
        (c::HeldRole::Owner, 0),
        (c::HeldRole::Maintainer, 1),
        (c::HeldRole::Member, 2),
        (c::HeldRole::Observer, 3),
        (c::HeldRole::Policy { role: 7 }, 7),
    ] {
        me.projects.get_mut(0).expect("project").role = held;
        let whom = c::Whom::Role { project: task.project, role: number };
        task.waits_on = party_wait(whom);
        assert_eq!(header_state(&task, Some(&me)), State::WaitingForYou);
        assert_eq!(header_state(&task, None), State::WaitingForParty(whom));
        let other_project = c::Whom::Role { project: task.project + 1, role: number };
        task.waits_on = party_wait(other_project);
        assert_eq!(header_state(&task, Some(&me)), State::WaitingForParty(other_project));
    }
    let whom = c::Whom::Role { project: task.project, role: 8 };
    task.waits_on = party_wait(whom);
    assert_eq!(header_state(&task, Some(&me)), State::WaitingForParty(whom));
    me.projects.clear();
    assert_eq!(header_state(&task, Some(&me)), State::WaitingForParty(whom));
}
#[test]
fn the_header_says_news_or_falls_back_to_the_row_when_nothing_is_pending() {
    let mut task = f::task();
    task.waits_on = c::WaitsOn::News;
    assert_eq!(header_state(&task, None), State::WaitingForNews);
    f::phase(&mut task, c::ChipPhase::Waiting { dependencies: pending(&[DEPENDENCY]), more: false, holds: false });
    for waits_on in [c::WaitsOn::Nothing, c::WaitsOn::Dependencies { pending: pending(&[]), count: 0 }] {
        task.waits_on = waits_on;
        assert_eq!(header_state(&task, None), State::WaitingFor(DEPENDENCY));
    }
}

fn agent_document() -> c::Task {
    let mut task = f::task();
    task.priority = None;
    task
}
fn activity_page(position: u64) -> jig_client::Readout {
    let mut rows = List::with_capacity(1);
    rows.push(crate::tests::documents::line(
        position,
        Wall::EPOCH,
        jig_client::contract::Activity::RunBegan {
            attempt: ATTEMPT,
            why: jig_client::contract::RunWhy::Made,
            resumed: false,
        },
    ))
    .expect("activity");
    jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Activity(Box::new(c::ActivityPage {
        rows,
        before: None,
        after: None,
        cut: false,
    }))))
}
fn activity_read(page: &Task) -> Request {
    Request::Client(Event::Read {
        handle: page.activity,
        read: Readable::Jig(c::Read::Activity { task: f::TASK, before: None }),
    })
}
#[test]
fn an_agents_task_reads_its_thread_as_a_chat() {
    const NEXT: u64 = f::POSITION + 1;
    for first_page_before_document in [true, false] {
        let mut f = f::opened();
        let mut page = Task::new(f::TASK, FIRST, &fixtures::limits(), &mut Queue::with_capacity(OUTPUTS));
        let mut task = agent_document();
        f::deliver(&mut f.client, f.watch, task.clone());
        while f.out.pop().is_some() {}
        if first_page_before_document {
            assert!(page.told(
                Time::ZERO,
                &f.client,
                Told::Page { handle: page.activity, page: activity_page(f::POSITION) },
                &mut f.out,
            ));
            assert!(page.latest.is_some());
        }
        page.told(Time::ZERO, &f.client, Told::Changed { handle: page.document }, &mut f.out);
        while f.out.pop().is_some() {}
        if !first_page_before_document {
            page.told(
                Time::ZERO,
                &f.client,
                Told::Page { handle: page.activity, page: activity_page(f::POSITION) },
                &mut f.out,
            );
        }
        assert!(page.latest.is_none());
        assert_eq!(page.thread.loaded, f::POSITION);
        assert_eq!(page.thread.lines.rows.len(), 1);
        assert!(page.thread.reading.is_none());
        task.activity = NEXT;
        f::deliver(&mut f.client, f.watch, task.clone());
        page.told(Time::ZERO, &f.client, Told::Changed { handle: page.document }, &mut f.out);
        assert_eq!(f.out.pop(), Some(activity_read(&page)));
        assert!(f.out.is_empty());
        page.told(
            Time::ZERO,
            &f.client,
            Told::Page { handle: page.activity, page: jig_client::Readout::Limit },
            &mut f.out,
        );
        assert!(page.thread.reading.is_some(), "transient end keeps the pending position");
        page.told(Time::ZERO, &f.client, Told::Changed { handle: page.document }, &mut f.out);
        assert!(f.out.is_empty(), "a delivery does not duplicate the waiting read");
        page.fire(Time::ZERO.saturating_add(skein_lib::Duration::from_secs(1)), &mut f.out);
        assert_eq!(f.out.pop(), Some(activity_read(&page)));
        page.thread.run(
            &c::RunState { task: f::TASK, attempt: Some(ATTEMPT), committed_turn: 0, writing: None, call: None },
            Some(&c::RunText {
                writing: None,
                attempt: ATTEMPT,
                began: None,
                turn: c::ProvisionalTurnName { attempt: ATTEMPT, number: 1, retry: 0 },
                from: 0,
                through: 5,
                text: c::Text::new(Box::from(*b"Hello"), 5).expect("provisional text"),
                cut: false,
                call: None,
            }),
        );
        f::phase(&mut task, c::ChipPhase::Active(c::RunStage::Idle));
        f::deliver(&mut f.client, f.watch, task);
        page.told(Time::ZERO, &f.client, Told::Changed { handle: page.document }, &mut f.out);
        assert!(!page.thread.live.is_empty(), "unread committed activity keeps the run's text");
        page.told(Time::ZERO, &f.client, Told::Page { handle: page.activity, page: activity_page(NEXT) }, &mut f.out);
        assert_eq!(page.thread.loaded, NEXT);
        assert!(page.thread.live.is_empty(), "no provisional text remains once its activity is loaded");
    }
}
#[test]
fn a_transient_first_page_keeps_an_unknown_tasks_thread() {
    let mut f = f::opened();
    let mut page = Task::new(f::TASK, FIRST, &fixtures::limits(), &mut Queue::with_capacity(OUTPUTS));
    page.told(
        Time::ZERO,
        &f.client,
        Told::Page { handle: page.activity, page: jig_client::Readout::Limit },
        &mut f.out,
    );
    assert_eq!(page.shape, Shape::Unknown);
    assert!(page.thread.reading.is_some());
    f::deliver(&mut f.client, f.watch, agent_document());
    while f.out.pop().is_some() {}
    page.told(Time::ZERO, &f.client, Told::Changed { handle: page.document }, &mut f.out);
    while let Some(request) = f.out.pop() {
        if let Request::Client(Event::Read { .. }) = request {
            panic!("the document does not duplicate a waiting read")
        }
    }
    page.fire(Time::ZERO.saturating_add(skein_lib::Duration::from_secs(1)), &mut f.out);
    assert_eq!(f.out.pop(), Some(activity_read(&page)));
}
fn open_task_run(page: &Task, fixture: &mut f::Fixture) -> skein_lib::Token {
    let mut emitted = Queue::with_capacity(OUTPUTS);
    fixture.client.step_unlinked(
        Time::ZERO,
        Wall::EPOCH,
        Event::Watch {
            handle: page.run.expect("live run"),
            subject: Watchable::Jig(c::Subject::Run { task: f::TASK, turn: None, offset: EMPTY_OFFSET }),
        },
        &mut emitted,
    );
    while let Some(request) = emitted.pop() {
        if let jig_client::UnlinkedRequest::Down {
            down: jig_client::Down::Open { watch, subject: Watchable::Jig(c::Subject::Run { .. }) },
        } = request
        {
            return watch;
        }
    }
    panic!("run opened");
}
fn current_call() -> c::CurrentCall {
    c::CurrentCall {
        identity: c::CallIdentity {
            owner: skein_lib::Token::new(1),
            sequence: 1,
            retry: 0,
            id: fixtures::acts::text(b"call-1"),
        },
        tool: fixtures::acts::text(b"read_docs"),
        input: fixtures::acts::text(br#"{"page":"index"}"#),
        cut: false,
        began: Some(Wall::EPOCH),
    }
}
fn deliver_task_call(
    page: &mut Task,
    fixture: &mut f::Fixture,
    watch: skein_lib::Token,
    call: Option<c::CurrentCall>,
    text: bool,
) {
    fixture.client.step_unlinked(
        Time::ZERO,
        Wall::EPOCH,
        Event::Up {
            up: jig_client::Up::Watched {
                watch,
                event: c::WatchEvent::Delivered(c::Delivery::RunState(Box::new(c::RunState {
                    task: f::TASK,
                    attempt: Some(ATTEMPT),
                    committed_turn: 0,
                    writing: None,
                    call,
                }))),
            },
        },
        &mut Queue::with_capacity(OUTPUTS),
    );
    let handle = page.run.expect("live run");
    let told = if text { Told::Text { handle } } else { Told::Changed { handle } };
    page.told(Time::ZERO, &fixture.client, told, &mut fixture.out);
}
fn agent_page() -> (Task, f::Fixture) {
    let (mut page, mut fixture) = opened();
    f::deliver(&mut fixture.client, fixture.watch, agent_document());
    page.told(Time::ZERO, &fixture.client, Told::Changed { handle: page.document }, &mut fixture.out);
    (page, fixture)
}
#[test]
fn an_agents_page_keeps_the_call_under_way() {
    let (mut page, mut fixture) = agent_page();
    let watch = open_task_run(&page, &mut fixture);
    assert_eq!(page.current_call(), None);
    let mut call = current_call();
    for text in [false, true] {
        deliver_task_call(&mut page, &mut fixture, watch, Some(call.clone()), text);
        assert_eq!(page.current_call(), Some(&call));
        call.tool = fixtures::acts::text(b"settings");
    }
}
fn agent_with_call() -> (Task, f::Fixture, skein_lib::Token) {
    let (mut page, mut fixture) = agent_page();
    let watch = open_task_run(&page, &mut fixture);
    deliver_task_call(&mut page, &mut fixture, watch, Some(current_call()), false);
    (page, fixture, watch)
}
#[test]
fn the_call_ends_with_the_run() {
    let (mut page, mut fixture, _watch) = agent_with_call();
    let mut task = agent_document();
    f::phase(&mut task, c::ChipPhase::Active(c::RunStage::Idle));
    f::deliver(&mut fixture.client, fixture.watch, task);
    page.told(Time::ZERO, &fixture.client, Told::Changed { handle: page.document }, &mut fixture.out);
    assert_eq!(page.current_call(), None);
}
#[test]
fn a_delivery_without_a_call_clears_it() {
    let (mut page, mut fixture, watch) = agent_with_call();
    deliver_task_call(&mut page, &mut fixture, watch, None, false);
    assert_eq!(page.current_call(), None);
}
fn assert_no_call((mut page, mut fixture): (Task, f::Fixture)) {
    let watch = open_task_run(&page, &mut fixture);
    deliver_task_call(&mut page, &mut fixture, watch, Some(current_call()), false);
    assert_eq!(page.current_call(), None, "{:?}", page.shape);
}
#[test]
fn another_shape_keeps_no_call() {
    assert_no_call(goal());
    assert_no_call(procedure());
}
fn newer_cursor() -> c::Cursor {
    c::Cursor::new(b"newer".as_slice().into(), 5).unwrap()
}
#[test]
fn latest_on_an_agents_page_reads_newer() {
    let (mut pages, client) = task_pages(agent_document());
    let page = pages.task_mut().unwrap();
    page.thread.reading = Some(jig_client::thread::Reading::Latest { known: 0 });
    let jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Activity(rows))) = activity_page(f::POSITION)
    else {
        panic!("activity fixture")
    };
    let _read = page.thread.page(*rows);
    page.thread.lines.edges.before = jig_client::thread::window::Edge::More(newer_cursor());
    let handle = page.activity;
    let mut out = page_act(&mut pages, &client, Act::Latest);
    assert_eq!(
        out.pop(),
        Some(Request::Client(Event::Read {
            handle,
            read: Readable::Jig(c::Read::ActivityBeside { task: f::TASK, position: f::POSITION, side: c::Side::Later }),
        }))
    );
    assert!(out.is_empty());
    assert!(page_act(&mut pages, &client, Act::Latest).is_empty(), "one outstanding read");
    pages::step(
        &mut pages,
        &fixtures::env(&fixtures::limits()),
        &client,
        pages::Event::Told { told: Told::Page { handle, page: jig_client::Readout::Limit } },
        &mut out,
    );
    let due = Time::ZERO.saturating_add(skein_lib::Duration::from_secs(1));
    pages.task_mut().unwrap().fire(due, &mut out);
    assert_eq!(
        out.pop(),
        Some(Request::Client(Event::Read {
            handle,
            read: Readable::Jig(c::Read::ActivityBeside { task: f::TASK, position: f::POSITION, side: c::Side::Later }),
        }))
    );
    assert!(out.is_empty(), "the newer read is retried through asking");
}
#[test]
fn latest_with_nothing_newer_asks_nothing() {
    let (mut pages, client) = task_pages(agent_document());
    let page = pages.task_mut().unwrap();
    page.thread.reading = None;
    page.thread.loaded = page.thread.committed;
    assert!(page_act(&mut pages, &client, Act::Latest).is_empty());
}
#[test]
fn latest_on_another_shape_asks_nothing() {
    for shape in [Shape::Tracked, Shape::HeaderAndPlan] {
        let (mut pages, client) = task_pages(agent_document());
        let task = pages.task_mut().unwrap();
        task.shape = shape;
        task.thread.reading = None;
        task.thread.lines.edges.before = jig_client::thread::window::Edge::More(newer_cursor());
        assert!(page_act(&mut pages, &client, Act::Latest).is_empty(), "{shape:?}");
    }
}
#[test]
fn an_agents_live_run_text_reaches_its_thread() {
    use skein_lib::Time;
    let (mut page, mut f) = opened();
    f::deliver(&mut f.client, f.watch, agent_document());
    page.told(Time::ZERO, &f.client, Told::Changed { handle: page.document }, &mut f.out);
    let handle = page.run.expect("live run");
    let mut emitted = Queue::with_capacity(OUTPUTS);
    f.client.step_unlinked(
        Time::ZERO,
        Wall::EPOCH,
        Event::Watch { handle, subject: Watchable::Jig(c::Subject::Run { task: f::TASK, turn: None, offset: 0 }) },
        &mut emitted,
    );
    let mut watch = None;
    while let Some(request) = emitted.pop() {
        if let jig_client::UnlinkedRequest::Down {
            down: jig_client::Down::Open { watch: token, subject: Watchable::Jig(c::Subject::Run { .. }) },
        } = request
        {
            watch = Some(token);
        }
    }
    let watch = watch.expect("run opened");
    let mut run = c::RunState { task: f::TASK, attempt: Some(ATTEMPT), committed_turn: 0, writing: None, call: None };
    run.writing = Some(c::RunText {
        writing: None,
        attempt: ATTEMPT,
        began: Some(Wall::EPOCH),
        turn: c::ProvisionalTurnName { attempt: ATTEMPT, number: 1, retry: 0 },
        from: 0,
        through: 5,
        text: c::Text::new(Box::from(*b"Hello"), 5).expect("run text"),
        cut: false,
        call: None,
    });
    f.client.step_unlinked(
        Time::ZERO,
        Wall::EPOCH,
        Event::Up {
            up: jig_client::Up::Watched {
                watch,
                event: c::WatchEvent::Delivered(c::Delivery::RunState(Box::new(run))),
            },
        },
        &mut emitted,
    );
    for told in [Told::Changed { handle }, Told::Text { handle }] {
        assert!(page.told(Time::ZERO, &f.client, told, &mut f.out));
        assert_eq!(page.thread.live.get(0).expect("provisional turn").text.bytes(), b"Hello");
    }
    f.client.step_unlinked(
        Time::ZERO,
        Wall::EPOCH,
        Event::Up {
            up: jig_client::Up::Watched {
                watch,
                event: c::WatchEvent::Delivered(c::Delivery::RunState(Box::new(c::RunState {
                    task: f::TASK,
                    attempt: Some(ATTEMPT),
                    committed_turn: 0,
                    writing: Some(c::RunText {
                        writing: None,
                        attempt: ATTEMPT,
                        began: Some(Wall::EPOCH),
                        turn: c::ProvisionalTurnName { attempt: ATTEMPT, number: 1, retry: 0 },
                        from: 5,
                        through: 11,
                        text: c::Text::new(Box::from(*b" again"), 6).expect("next run text"),
                        cut: false,
                        call: None,
                    }),
                    call: None,
                }))),
            },
        },
        &mut emitted,
    );
    assert!(page.told(Time::ZERO, &f.client, Told::Text { handle }, &mut f.out));
    assert_eq!(page.thread.live.get(0).expect("provisional turn").text.bytes(), b"Hello again");
}
#[test]
fn a_runs_heading_follows_its_chips_phase() {
    use crate::pages::task::{Run, run};
    const LAST: u64 = ATTEMPT - 1;
    let since = Some(Wall::EPOCH);
    for (phase, attempt, since, expected) in [
        (
            c::ChipPhase::Active(c::RunStage::Claimed { attempt: ATTEMPT }),
            Some(LAST),
            since,
            Some(Run::Live { attempt: ATTEMPT, since }),
        ),
        (
            c::ChipPhase::Active(c::RunStage::Running { attempt: ATTEMPT }),
            Some(LAST),
            since,
            Some(Run::Live { attempt: ATTEMPT, since }),
        ),
        (
            c::ChipPhase::Closing(c::ClosingStage::Run { attempt: ATTEMPT }),
            Some(LAST),
            since,
            Some(Run::Live { attempt: ATTEMPT, since }),
        ),
        (
            c::ChipPhase::Active(c::RunStage::Running { attempt: ATTEMPT }),
            None,
            None,
            Some(Run::Live { attempt: ATTEMPT, since: None }),
        ),
        (c::ChipPhase::Active(c::RunStage::Idle), Some(LAST), since, Some(Run::Waiting { attempt: LAST, since })),
        (c::ChipPhase::Active(c::RunStage::Idle), Some(LAST), None, Some(Run::Waiting { attempt: LAST, since: None })),
        (c::ChipPhase::Active(c::RunStage::Idle), None, since, Some(Run::NoRun)),
        (c::ChipPhase::Ended, Some(LAST), since, Some(Run::Ended { attempt: LAST })),
        (c::ChipPhase::Ended, None, since, None),
    ] {
        let mut task = agent_document();
        f::phase(&mut task, phase.clone());
        let c::ChipPhaseFacts { details, .. } = task.chip.details.as_mut();
        details.attempt = attempt;
        details.run_since = since;
        assert_eq!(run(&task), expected, "{phase:?}, {attempt:?}, {since:?}");
    }
    let mut task = agent_document();
    task.chip.details.phase = c::ChipPhase::Active(c::RunStage::Preparing);
    assert_eq!(run(&task), Some(Run::NoRun));
}
#[test]
fn an_ended_tasks_ended_document_keeps_its_result() {
    let (mut page, mut f) = opened();
    let mut task = agent_document();
    f::phase(&mut task, c::ChipPhase::Ended);
    task.result = None;
    let result = f::result(&task, c::Ending::Done, Some(c::ResultValue::Report(f::document_text(b"Finished"))));
    page.told(
        Time::ZERO,
        &f.client,
        Told::Page {
            handle: page.document,
            page: jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Ended(Box::new(c::Ended {
                key: c::ObjectKey::Task(f::TASK),
                value: c::EndedValue::Task { document: Box::new(task), result: Box::new(result.clone()) },
                at: None,
                by: None,
            })))),
        },
        &mut f.out,
    );
    assert_eq!(page.document(&f.client).expect("final task").result, Some(c::TaskResult::Document(Box::new(result))));
}

#[test]
fn an_agents_thread_reads_on_until_the_announced_tail() {
    let (mut page, mut f) = opened();
    f::deliver(&mut f.client, f.watch, agent_document());
    page.told(Time::ZERO, &f.client, Told::Changed { handle: page.document }, &mut f.out);
    while f.out.pop().is_some() {}
    page.told(
        Time::ZERO,
        &f.client,
        Told::Page { handle: page.activity, page: activity_page(f::POSITION - 1) },
        &mut f.out,
    );
    assert_eq!(f.out.pop(), Some(activity_read(&page)));
    assert!(f.out.is_empty());
    page.told(
        Time::ZERO,
        &f.client,
        Told::Page { handle: page.activity, page: activity_page(f::POSITION) },
        &mut f.out,
    );
    assert!(page.thread.reading.is_none());
    assert_eq!(page.thread.loaded, f::POSITION);
    assert!(f.out.is_empty());
}

use crate::pages::{self, Act, Pages, task::Sending};
const WORDS: &[u8] = b"Keep the public interface";
fn task_pages(task: c::Task) -> (Pages, jig_client::Client) {
    task_pages_with(task, fixtures::PARTY, &fixtures::limits())
}
fn task_pages_with(mut task: c::Task, reader: u64, limits: &crate::Limits) -> (Pages, jig_client::Client) {
    task.label = 1234; // A task page, distinct from the chat template.
    let mut fixture = f::with_delegate();
    let mut me = fixtures::chats::me();
    me.party.number = reader;
    me.words_bytes = super::composer::BOUND;
    fixture.client.step_unlinked(Time::ZERO, Wall::EPOCH,
        Event::Up { up: jig_client::Up::Watched { watch: fixture.me,
            event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me))) } }, &mut Queue::uncapped());
    let number = task.chip.task;
    let mut pages = Pages::new();
    let env = fixtures::env(limits);
    let mut out = Queue::with_capacity(OUTPUTS);
    pages::step(
        &mut pages,
        &env,
        &fixture.client,
        pages::Event::Went { address: crate::Address::Task { task: number } },
        &mut out,
    );
    let handle = pages.task().expect("task page").document;
    pages::step(
        &mut pages,
        &env,
        &fixture.client,
        pages::Event::Told {
            told: Told::Page {
                handle,
                page: jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Task(Box::new(task)))),
            },
        },
        &mut out,
    );
    (pages, fixture.client)
}
fn page_act(pages: &mut Pages, client: &jig_client::Client, act: Act) -> Queue<Request> {
    let mut out = Queue::with_capacity(OUTPUTS);
    pages::step(pages, &fixtures::env(&fixtures::limits()), client, pages::Event::Act { act }, &mut out);
    out
}
fn offered_task(kind: c::ActKind, admission: Option<c::Admission>) -> c::Task {
    let mut task = f::task();
    task.priority = None;
    task.chip.executor = c::ChipExecutor::Agent { charter: 1 };
    task.acts = match admission {
        Some(admission) => fixtures::acts::offered(kind, admission),
        None => List::with_capacity(0),
    };
    task
}
#[test]
fn a_send_asks_for_a_task_s_composer() {
    for priority in [None, Some(PRIORITY)] {
        let mut task = offered_task(c::ActKind::Message, Some(c::Admission::Allowed));
        task.priority = priority;
        let (mut pages, client) = task_pages(task);
        assert_eq!(pages.task().expect("task").composer_focus, 0);
        page_act(&mut pages, &client, Act::MessageWords { words: Box::from(WORDS) });
        assert_eq!(pages.task().expect("task").composer_focus, 0);
        assert_eq!(fixtures::acts::sent(&mut page_act(&mut pages, &client, Act::Send)).len(), 1);
        assert_eq!(pages.task().expect("task").composer_focus, 1);
        assert!(pages.task().expect("task").words.is_empty());
        assert!(fixtures::acts::sent(&mut page_act(&mut pages, &client, Act::Send)).is_empty());
        assert_eq!(pages.task().expect("task").composer_focus, 2);
        for act in [Act::Stop, Act::AddDraft, Act::MessageWords { words: Box::from(WORDS) }] {
            page_act(&mut pages, &client, act);
            assert_eq!(pages.task().expect("task").composer_focus, 2);
        }
    }
}

#[test]
fn an_agents_send_is_one_message_only_when_allowed() {
    for (admission, sent) in [
        (Some(c::Admission::Allowed), true),
        (Some(c::Admission::Refused(c::Refusal::NotOffered)), false),
        (Some(c::Admission::Proposed { to: c::Recipient::Party(fixtures::PARTY) }), false),
        (None, false),
    ] {
        let (mut pages, client) = task_pages(offered_task(c::ActKind::Message, admission));
        page_act(&mut pages, &client, Act::MessageWords { words: Box::from(WORDS) });
        let mut out = page_act(&mut pages, &client, Act::Send);
        let asks = fixtures::acts::sent(&mut out);
        if sent {
            assert_eq!(
                asks.as_ref(),
                [c::Ask::Message { project: fixtures::PROJECT, task: f::TASK, words: fixtures::acts::text(WORDS) }]
            );
            assert!(pages.task().expect("page").words.is_empty());
            // A second nonempty message joins the same in-flight kind.
            page_act(&mut pages, &client, Act::MessageWords { words: Box::from(*b"Again") });
            assert!(fixtures::acts::sent(&mut page_act(&mut pages, &client, Act::Send)).is_empty());
        } else {
            assert!(asks.is_empty());
            assert_eq!(pages.task().expect("page").words.as_ref(), WORDS);
        }
    }
}
#[test]
fn a_goals_page_has_no_older_read() {
    let mut task = offered_task(c::ActKind::Message, Some(c::Admission::Allowed));
    task.priority = Some(PRIORITY);
    let (mut pages, client) = task_pages(task);
    assert!(page_act(&mut pages, &client, Act::Older).is_empty());
}
fn paused_agent() -> c::Task {
    let mut task = super::goal::paused_task();
    task.priority = None;
    task
}
#[test]
fn an_agent_s_pause_and_resume_are_sent_at_once() {
    for (task, act, expected, kind) in [
        (
            offered_task(c::ActKind::Stop, Some(c::Admission::Allowed)),
            Act::Pause,
            c::Ask::Stop { project: fixtures::PROJECT, task: f::TASK },
            Sending::Stop,
        ),
        (
            paused_agent(),
            Act::Resume,
            c::Ask::Release {
                project: fixtures::PROJECT,
                task: f::TASK,
                revision: c::Revision { value: 17 },
                again: None,
            },
            Sending::Resume,
        ),
    ] {
        let (mut pages, client) = task_pages(task);
        assert_eq!(fixtures::acts::sent(&mut page_act(&mut pages, &client, act.clone())).as_ref(), [expected]);
        assert!(pages.confirmation().is_none());
        assert!(fixtures::acts::sent(&mut page_act(&mut pages, &client, act)).is_empty());
        assert_eq!(pages.task().expect("page").pending.as_slice()[0].1, kind);
    }
}
#[test]
fn an_agent_s_resume_is_its_person_s_own_pause_alone() {
    enum Change {
        OtherPerson,
        OtherTask,
        Lifted,
        Unplaced,
        Active,
    }
    for changed in [Change::OtherPerson, Change::OtherTask, Change::Lifted, Change::Unplaced, Change::Active] {
        let mut task = paused_agent();
        let hold = task.chip.details.details.hold.as_mut().expect("hold");
        let placed = hold.placed.as_mut().expect("placed");
        match changed {
            Change::OtherPerson => placed.owner = c::HoldOwner::Party { person: fixtures::PARTY + 1 },
            Change::OtherTask => placed.scope = c::HoldScope::Subtree { task: f::TASK + 1 },
            Change::Lifted => placed.lifted = true,
            Change::Unplaced => hold.placed = None,
            Change::Active => f::phase(&mut task, c::ChipPhase::Active(c::RunStage::Idle)),
        }
        let (mut pages, client) = task_pages(task);
        assert!(fixtures::acts::sent(&mut page_act(&mut pages, &client, Act::Resume)).is_empty());
    }
}
#[test]
fn an_unavailable_task_act_opens_no_confirmation() {
    for admission in [
        None,
        Some(c::Admission::Refused(c::Refusal::NotOffered)),
        Some(c::Admission::Proposed { to: c::Recipient::Party(fixtures::PARTY) }),
    ] {
        for (kind, act) in [(c::ActKind::Stop, Act::Pause), (c::ActKind::Cancel, Act::Close)] {
            let (mut pages, client) = task_pages(offered_task(kind, admission.clone()));
            assert!(fixtures::acts::sent(&mut page_act(&mut pages, &client, act)).is_empty());
            assert!(pages.confirmation().is_none());
        }
    }
}
#[test]
fn an_agent_s_close_asks_first_then_cancels() {
    let (mut pages, client) = task_pages(offered_task(c::ActKind::Cancel, Some(c::Admission::Allowed)));
    assert!(fixtures::acts::sent(&mut page_act(&mut pages, &client, Act::Close)).is_empty());
    assert_eq!(pages.confirmation().expect("confirmation").act, c::ActKind::Cancel);
    assert_eq!(
        fixtures::acts::sent(&mut page_act(&mut pages, &client, Act::Confirm)).as_ref(),
        [c::Ask::Cancel {
            project: fixtures::PROJECT,
            task: f::TASK,
            reason: fixtures::acts::text(b"Closed from its page"),
            grace: None,
        }]
    );
}
#[test]
fn a_task_answer_sets_the_outcome_and_clears_pending() {
    let (mut pages, client) = task_pages(offered_task(c::ActKind::Stop, Some(c::Admission::Allowed)));
    page_act(&mut pages, &client, Act::Pause);
    let key = pages.task().expect("page").pending.as_slice()[0].0;
    let mut out = Queue::with_capacity(OUTPUTS);
    pages::step(
        &mut pages,
        &fixtures::env(&fixtures::limits()),
        &client,
        pages::Event::Told { told: Told::Pending { key } },
        &mut out,
    );
    assert!(!pages.task().expect("page").pending.is_empty());
    pages::step(
        &mut pages,
        &fixtures::env(&fixtures::limits()),
        &client,
        pages::Event::Told { told: Told::Answered { key, outcome: jig_client::Outcome::Unknown } },
        &mut out,
    );
    let page = pages.task().expect("page");
    assert!(page.pending.is_empty());
    assert_eq!(page.outcome, Some(jig_client::Outcome::Unknown));
}
#[test]
fn older_on_an_agents_page_reads_the_cursor_before_its_window() {
    use jig_client::thread::window::Edge;
    let (mut page, mut fixture) = opened();
    page.shape = Shape::Agent;
    page.thread.reading = None;
    let cursor = c::Cursor::new(Box::from(*b"older"), 32).expect("cursor");
    page.thread.lines.edges.after = Edge::More(cursor.clone());
    while fixture.out.pop().is_some() {}
    page.act(&fixtures::env(&fixtures::limits()), &fixture.client, Act::Older, &mut 0, &mut fixture.out);
    assert_eq!(
        fixture.out.pop(),
        Some(Request::Client(Event::Read {
            handle: page.activity,
            read: Readable::Jig(c::Read::Activity { task: f::TASK, before: Some(cursor) }),
        }))
    );
}

#[test]
fn a_message_answer_reconciles_its_conversation_words() {
    for outcome in [
        jig_client::Outcome::Unknown,
        jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Refused {
            refusal: c::Refusal::NotWaiting,
            at: None,
        })),
    ] {
        let (mut pages, client) = task_pages(offered_task(c::ActKind::Message, Some(c::Admission::Allowed)));
        page_act(&mut pages, &client, Act::MessageWords { words: Box::from(WORDS) });
        page_act(&mut pages, &client, Act::Send);
        let page = pages.task().expect("page");
        let key = page.pending.as_slice()[0].0;
        assert!(!page.thread.words.rows.is_empty());
        let mut out = Queue::with_capacity(OUTPUTS);
        pages::step(
            &mut pages,
            &fixtures::env(&fixtures::limits()),
            &client,
            pages::Event::Told { told: Told::Answered { key, outcome: outcome.clone() } },
            &mut out,
        );
        let page = pages.task().expect("page");
        assert!(page.pending.is_empty());
        assert_eq!(page.outcome, Some(outcome));
        assert!(page.thread.words.rows.is_empty());
    }
}
#[test]
fn a_delivered_message_is_said_by_its_words_not_an_outcome() {
    let (mut pages, client) = task_pages(offered_task(c::ActKind::Message, Some(c::Admission::Allowed)));
    page_act(&mut pages, &client, Act::MessageWords { words: Box::from(WORDS) });
    page_act(&mut pages, &client, Act::Send);
    let key = pages.task().expect("page").pending.as_slice()[0].0;
    let mut out = Queue::with_capacity(OUTPUTS);
    let done = jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::Document(
        c::DocumentOutcome::Amended { task: 9 },
    ))));
    pages::step(
        &mut pages,
        &fixtures::env(&fixtures::limits()),
        &client,
        pages::Event::Told { told: Told::Answered { key, outcome: done } },
        &mut out,
    );
    let page = pages.task().expect("page");
    assert!(page.pending.is_empty());
    assert_eq!(page.outcome, None);
}

fn amend_pages(goal: bool) -> (Pages, jig_client::Client) {
    let mut task = offered_task(c::ActKind::Amend, Some(c::Admission::Allowed));
    if goal {
        task.priority = Some(PRIORITY);
    }
    task_pages(task)
}
#[test]
fn amend_opens_only_when_the_task_offers_it_allowed() {
    for (admission, opens) in [
        (Some(c::Admission::Allowed), true),
        (Some(c::Admission::Refused(c::Refusal::NotOffered)), false),
        (Some(c::Admission::Proposed { to: c::Recipient::Party(fixtures::PARTY) }), false),
        (None, false),
    ] {
        let (mut pages, client) = task_pages(offered_task(c::ActKind::Amend, admission));
        assert!(page_act(&mut pages, &client, Act::Amend).is_empty());
        assert_eq!(pages.task().unwrap().amending.is_some(), opens);
    }
}
#[test]
fn edited_words_send_one_amendment_with_the_base_revision_and_pages_reason() {
    let (mut pages, client) = amend_pages(false);
    let base = pages.task().unwrap().document(&client).unwrap().spec_revision;
    page_act(&mut pages, &client, Act::Amend);
    page_act(&mut pages, &client, Act::AmendWords { words: Box::from(WORDS) });
    let asks = fixtures::acts::sent(&mut page_act(&mut pages, &client, Act::SendAmendment));
    let [c::Ask::Amend { project, task, revision, change, reason }] = asks.as_ref() else { panic!("one amendment") };
    assert_eq!((*project, *task, *revision), (fixtures::PROJECT, f::TASK, Some(base)));
    assert_eq!(reason.bytes(), b"Amended from its page");
    let mut spec = pages.task().unwrap().document(&client).unwrap().spec.whole().unwrap();
    spec.words = fixtures::acts::text(WORDS);
    assert_eq!(change.spec, Some(spec));
    assert!(fixtures::acts::sent(&mut page_act(&mut pages, &client, Act::SendAmendment)).is_empty());
}
#[test]
fn a_goals_priority_alone_sends_no_spec_revision() {
    let (mut pages, client) = amend_pages(true);
    page_act(&mut pages, &client, Act::Amend);
    page_act(&mut pages, &client, Act::AmendPriority { priority: PRIORITY + 1 });
    let asks = fixtures::acts::sent(&mut page_act(&mut pages, &client, Act::SendAmendment));
    let [c::Ask::Amend { revision, change, .. }] = asks.as_ref() else { panic!("priority amendment") };
    assert_eq!(*revision, None);
    assert_eq!(change.tracked, Some(c::Tracking::Tracked { priority: PRIORITY + 1 }));
}
#[test]
fn sending_no_change_says_so_until_an_edit_and_discard_drops_the_editor() {
    let (mut pages, client) = amend_pages(true);
    page_act(&mut pages, &client, Act::Amend);
    assert!(page_act(&mut pages, &client, Act::SendAmendment).is_empty());
    assert!(pages.task().unwrap().amending.as_ref().unwrap().no_change);
    page_act(&mut pages, &client, Act::AmendTracked);
    assert!(!pages.task().unwrap().amending.as_ref().unwrap().no_change);
    assert_eq!(pages.task().unwrap().amending.as_ref().unwrap().tracking, Some(c::Tracking::Untracked));
    page_act(&mut pages, &client, Act::DiscardAmendment);
    assert!(pages.task().unwrap().amending.is_none());
}
#[test]
fn a_cut_specification_opens_read_only_and_ignores_words() {
    let mut task = offered_task(c::ActKind::Amend, Some(c::Admission::Allowed));
    task.spec.specification.words.cut = true;
    let (mut pages, client) = task_pages(task);
    page_act(&mut pages, &client, Act::Amend);
    page_act(&mut pages, &client, Act::AmendWords { words: Box::from(WORDS) });
    let open = pages.task().unwrap().amending.as_ref().unwrap();
    assert!(open.cut);
    assert!(open.words.is_empty());
    assert!(open.draft.amendment().is_none());
}
fn amend_answer(pages: &mut Pages, client: &jig_client::Client, answer: c::Answer) {
    let key = pages.task().unwrap().pending.as_slice()[0].0;
    pages::step(
        pages,
        &fixtures::env(&fixtures::limits()),
        client,
        pages::Event::Told {
            told: Told::Answered { key, outcome: jig_client::Outcome::Answered(jig_client::Answered::Jig(answer)) },
        },
        &mut Queue::with_capacity(OUTPUTS),
    );
}
#[test]
fn a_done_amendment_stays_as_its_outcome_until_amend_again() {
    let (mut pages, client) = amend_pages(false);
    page_act(&mut pages, &client, Act::Amend);
    page_act(&mut pages, &client, Act::AmendWords { words: Box::from(WORDS) });
    page_act(&mut pages, &client, Act::SendAmendment);
    amend_answer(&mut pages, &client, c::Answer::Done(c::Outcome::Document(c::DocumentOutcome::Amended { task: 9 })));
    let open = pages.task().unwrap().amending.as_ref().unwrap();
    assert!(open.done);
    assert_eq!(
        open.outcome,
        Some(jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::Document(
            c::DocumentOutcome::Amended { task: 9 }
        )))))
    );
    assert!(pages.task().unwrap().pending.is_empty());
    assert!(page_act(&mut pages, &client, Act::SendAmendment).is_empty());
    page_act(&mut pages, &client, Act::DiscardAmendment);
    assert!(pages.task().unwrap().amending.as_ref().unwrap().done);
    page_act(&mut pages, &client, Act::AmendWords { words: Box::from(*b"Ignored") });
    assert_eq!(pages.task().unwrap().amending.as_ref().unwrap().words.as_ref(), WORDS);
    page_act(&mut pages, &client, Act::Amend);
    let open = pages.task().unwrap().amending.as_ref().unwrap();
    assert!(!open.done);
    assert!(open.outcome.is_none());
    assert!(open.draft.amendment().is_none());
}
#[test]
fn a_moved_refusal_keeps_the_editor_and_look_again_redrafts_the_current_document() {
    let (mut pages, client) = amend_pages(false);
    page_act(&mut pages, &client, Act::Amend);
    page_act(&mut pages, &client, Act::AmendWords { words: Box::from(WORDS) });
    page_act(&mut pages, &client, Act::SendAmendment);
    let mut task = pages.task().unwrap().document(&client).unwrap().clone();
    task.spec_revision.value += 1;
    let revision = task.spec_revision;
    let handle = pages.task().unwrap().document;
    pages::step(
        &mut pages,
        &fixtures::env(&fixtures::limits()),
        &client,
        pages::Event::Told {
            told: Told::Page {
                handle,
                page: jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Task(Box::new(task)))),
            },
        },
        &mut Queue::with_capacity(OUTPUTS),
    );
    amend_answer(
        &mut pages,
        &client,
        c::Answer::Refused { refusal: c::Refusal::Moved { now: revision, what: c::Moved::Spec }, at: None },
    );
    assert!(
        pages.task().unwrap().amending.as_ref().unwrap().draft.moved(pages.task().unwrap().document(&client).unwrap())
    );
    assert!(page_act(&mut pages, &client, Act::SendAmendment).is_empty());
    page_act(&mut pages, &client, Act::AmendAgain);
    let open = pages.task().unwrap().amending.as_ref().unwrap();
    assert_eq!(open.draft.revision(), revision);
    assert!(open.draft.amendment().is_none());
    assert!(open.outcome.is_none());
}
#[test]
fn amendment_words_are_bounded_and_must_be_utf8() {
    let (mut pages, client) = amend_pages(false);
    page_act(&mut pages, &client, Act::Amend);
    let original = pages.task().unwrap().amending.as_ref().unwrap().words.clone();
    let bound = usize::try_from(fixtures::limits().client.domain.text).unwrap();
    for words in [skein_lib::bytes::zeroed(bound + 1), Box::from([0xff])] {
        page_act(&mut pages, &client, Act::AmendWords { words });
        assert_eq!(pages.task().unwrap().amending.as_ref().unwrap().words, original);
    }
    page_act(&mut pages, &client, Act::AmendWords { words: skein_lib::bytes::zeroed(bound) });
    assert_eq!(pages.task().unwrap().amending.as_ref().unwrap().words.len(), bound);
}

fn procedure() -> (Task, f::Fixture) {
    let (mut page, mut fixture) = opened();
    while fixture.out.pop().is_some() {}
    let mut task = f::task();
    task.chip.executor = c::ChipExecutor::Procedure { connector: fixtures::FORGE, kind: 2 };
    f::deliver(&mut fixture.client, fixture.watch, task);
    page.told(Time::ZERO, &fixture.client, Told::Changed { handle: page.document }, &mut fixture.out);
    (page, fixture)
}
fn ended_read(page: &Task) -> Request {
    Request::Client(Event::Read {
        handle: page.ended.as_ref().expect("procedure's ended children").handle,
        read: Readable::Jig(c::Read::Tasks {
            query: c::TaskQuery {
                project: None,
                requester: None,
                root: None,
                parent: Some(f::TASK),
                label: None,
                phase: Some(c::PhaseClass::Ended),
                executor: None,
                tracked: None,
                order: c::QueryOrder::Ended,
            },
            live: None,
            ended: None,
        }),
    })
}
fn ended_page(cut: bool) -> c::TaskPage {
    c::TaskPage {
        live: List::with_capacity(0),
        live_after: None,
        ended: c::EndedGroup::Rows { rows: List::with_capacity(0), after: None, cut },
    }
}
fn answer_ended(page: &mut Task, fixture: &mut f::Fixture, reply: c::TaskPage) {
    let handle = page.ended.as_ref().expect("ended children").handle;
    assert!(page.told(
        Time::ZERO,
        &fixture.client,
        Told::Page {
            handle,
            page: jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Tasks(Box::new(reply)))),
        },
        &mut fixture.out,
    ));
}
#[test]
fn every_procedure_reads_its_ended_children_on_its_own_handle() {
    for connector in [fixtures::FORGE, fixtures::FORGE + 1] {
        let (mut page, mut fixture) = opened();
        while fixture.out.pop().is_some() {}
        let next = page.next;
        let mut task = f::task();
        task.chip.executor = c::ChipExecutor::Procedure { connector, kind: 2 };
        f::deliver(&mut fixture.client, fixture.watch, task);
        page.told(Time::ZERO, &fixture.client, Told::Changed { handle: page.document }, &mut fixture.out);
        assert_eq!(page.ended.as_ref().unwrap().handle.number, next);
        assert_eq!(page.next, next + 1);
        assert_eq!(fixture.out.pop(), Some(ended_read(&page)));
        assert!(fixture.out.is_empty());
        assert!(page.ended.as_ref().unwrap().reading);
        assert!(!page.ended.as_ref().unwrap().stale);
    }
}
#[test]
fn agent_and_person_tasks_do_not_read_ended_children() {
    for executor in [
        c::ChipExecutor::Agent { charter: 1 },
        c::ChipExecutor::Party { party: 73 },
        c::ChipExecutor::Role { project: fixtures::PROJECT, role: 1 },
    ] {
        let (mut page, mut fixture) = opened();
        while fixture.out.pop().is_some() {}
        let mut task = f::task();
        task.chip.executor = executor;
        task.priority = None;
        f::deliver(&mut fixture.client, fixture.watch, task);
        for handle in [page.document, page.tree] {
            page.told(Time::ZERO, &fixture.client, Told::Changed { handle }, &mut fixture.out);
        }
        assert!(page.ended.is_none());
        assert!(fixture.out.is_empty());
    }
}
#[test]
fn tree_and_document_changes_read_ended_children_again() {
    let (mut page, mut fixture) = procedure();
    assert_eq!(fixture.out.pop(), Some(ended_read(&page)));
    for handle in [page.tree, page.document] {
        answer_ended(&mut page, &mut fixture, ended_page(false));
        assert!(fixture.out.is_empty());
        page.told(Time::ZERO, &fixture.client, Told::Changed { handle }, &mut fixture.out);
        assert_eq!(fixture.out.pop(), Some(ended_read(&page)));
        assert!(fixture.out.is_empty());
    }
}
#[test]
fn changes_during_an_ended_read_coalesce_into_one_more_read() {
    let (mut page, mut fixture) = procedure();
    assert_eq!(fixture.out.pop(), Some(ended_read(&page)));
    for handle in [page.tree, page.document, page.tree] {
        page.told(Time::ZERO, &fixture.client, Told::Changed { handle }, &mut fixture.out);
        assert!(fixture.out.is_empty());
    }
    assert!(page.ended.as_ref().unwrap().stale);
    answer_ended(&mut page, &mut fixture, ended_page(false));
    assert_eq!(fixture.out.pop(), Some(ended_read(&page)));
    assert!(fixture.out.is_empty());
    assert!(page.ended.as_ref().unwrap().reading);
    assert!(!page.ended.as_ref().unwrap().stale);
    answer_ended(&mut page, &mut fixture, ended_page(true));
    assert!(fixture.out.is_empty());
    assert!(!page.ended.as_ref().unwrap().reading);
}
#[test]
fn an_ended_reply_replaces_the_kept_page() {
    let (mut page, mut fixture) = procedure();
    while fixture.out.pop().is_some() {}
    for cut in [false, true] {
        if cut {
            page.told(Time::ZERO, &fixture.client, Told::Changed { handle: page.tree }, &mut fixture.out);
            assert_eq!(fixture.out.pop(), Some(ended_read(&page)));
        }
        let reply = ended_page(cut);
        answer_ended(&mut page, &mut fixture, reply.clone());
        assert_eq!(page.ended.as_ref().unwrap().page.as_deref(), Some(&reply));
        assert!(!page.ended.as_ref().unwrap().reading);
        assert!(fixture.out.is_empty());
    }
}
#[test]
fn another_reply_keeps_the_ended_page_and_frees_the_next_read() {
    let (mut page, mut fixture) = procedure();
    while fixture.out.pop().is_some() {}
    let kept = ended_page(false);
    answer_ended(&mut page, &mut fixture, kept.clone());
    let handle = page.ended.as_ref().unwrap().handle;
    for stale in [false, true] {
        page.told(Time::ZERO, &fixture.client, Told::Changed { handle: page.tree }, &mut fixture.out);
        assert_eq!(fixture.out.pop(), Some(ended_read(&page)));
        if stale {
            page.told(Time::ZERO, &fixture.client, Told::Changed { handle: page.document }, &mut fixture.out);
        }
        page.told(
            Time::ZERO,
            &fixture.client,
            Told::Page {
                handle,
                page: jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Task(Box::new(f::task())))),
            },
            &mut fixture.out,
        );
        assert_eq!(page.ended.as_ref().unwrap().page.as_deref(), Some(&kept));
        assert_eq!(page.ended.as_ref().unwrap().reading, stale);
        if stale {
            assert_eq!(fixture.out.pop(), Some(ended_read(&page)));
        }
        assert!(fixture.out.is_empty());
    }
}
#[test]
fn leaving_a_procedure_unwatches_its_ended_read_handle() {
    let (page, mut fixture) = procedure();
    while fixture.out.pop().is_some() {}
    let next = page.leave(&mut fixture.out);
    assert_eq!(next, page.next);
    assert_eq!(
        f::handles(&mut fixture.out).as_ref(),
        [page.document, page.tree, page.activity, page.run.unwrap(), page.ended.as_ref().unwrap().handle],
    );
}

fn goal() -> (Task, f::Fixture) {
    let (mut page, mut fixture) = opened();
    while fixture.out.pop().is_some() {}
    let mut task = f::task();
    task.chip.executor = c::ChipExecutor::Agent { charter: 1 };
    task.priority = Some(PRIORITY);
    f::deliver(&mut fixture.client, fixture.watch, task);
    page.told(Time::ZERO, &fixture.client, Told::Changed { handle: page.document }, &mut fixture.out);
    (page, fixture)
}
fn opening_goal_reads(out: &mut Queue<Request>) {
    let Some(Request::Client(Event::Read { read: Readable::Jig(c::Read::Notes { .. }), .. })) = out.pop() else {
        panic!("the goal's report read");
    };
    let Some(Request::Client(Event::Read { read: Readable::Jig(c::Read::Tasks { .. }), .. })) = out.pop() else {
        panic!("the goal's own line read");
    };
}
#[test]
fn a_goal_reads_its_ended_delegates_on_its_own_handle() {
    let (page, mut fixture) = goal();
    let ended = page.ended.as_ref().unwrap();
    assert_eq!(page.shape, Shape::Tracked);
    // The ended page is followed by the report and own-line handles.
    assert_eq!(page.next, ended.handle.number + 3);
    assert_ne!(ended.handle, page.document);
    assert_ne!(ended.handle, page.tree);
    assert_ne!(ended.handle, page.activity);
    assert_ne!(Some(ended.handle), page.run);
    assert!(ended.reading);
    assert!(!ended.stale);
    assert_eq!(fixture.out.pop(), Some(ended_read(&page)));
    opening_goal_reads(&mut fixture.out);
    assert!(fixture.out.is_empty());
}
#[test]
fn a_goals_tree_and_document_changes_reread_one_page_at_a_time() {
    let (mut page, mut fixture) = goal();
    assert_eq!(fixture.out.pop(), Some(ended_read(&page)));
    opening_goal_reads(&mut fixture.out);
    for handle in [page.tree, page.document] {
        let kept = ended_page(false);
        answer_ended(&mut page, &mut fixture, kept.clone());
        assert!(fixture.out.is_empty());
        page.told(Time::ZERO, &fixture.client, Told::Changed { handle }, &mut fixture.out);
        assert_eq!(fixture.out.pop(), Some(ended_read(&page)));
        for handle in [page.document, page.tree, handle] {
            page.told(Time::ZERO, &fixture.client, Told::Changed { handle }, &mut fixture.out);
            assert!(fixture.out.is_empty());
        }
        let ended = page.ended.as_ref().unwrap();
        assert!(ended.reading && ended.stale);
        assert_eq!(ended.page.as_deref(), Some(&kept));
        answer_ended(&mut page, &mut fixture, ended_page(true));
        assert_eq!(fixture.out.pop(), Some(ended_read(&page)));
        assert!(fixture.out.is_empty());
        let ended = page.ended.as_ref().unwrap();
        assert!(ended.reading && !ended.stale);
    }
    answer_ended(&mut page, &mut fixture, ended_page(false));
    assert!(!page.ended.as_ref().unwrap().reading);
    assert!(fixture.out.is_empty());
}
#[test]
fn an_agents_task_transient_activity_read_is_asked_again() {
    for why in [c::ReadNone::Busy, c::ReadNone::Unreachable] {
        // The task announces the fourth activity before its first activity page.
        const COMMITTED: u64 = 4;
        let mut f = f::opened();
        let mut out = Queue::with_capacity(OUTPUTS);
        let mut page = Task::new(f::TASK, FIRST, &fixtures::limits(), &mut out);
        while out.pop().is_some() {}
        let mut task = agent_document();
        task.activity = COMMITTED;
        f::deliver(&mut f.client, f.watch, task);
        page.told(Time::ZERO, &f.client, Told::Changed { handle: page.document }, &mut out);
        while out.pop().is_some() {}
        page.told(
            Time::ZERO,
            &f.client,
            Told::Page { handle: page.activity, page: jig_client::Readout::Jig(c::ReadResult::None(why)) },
            &mut out,
        );
        let due = Time::ZERO.saturating_add(skein_lib::Duration::from_secs(1));
        assert_eq!(page.next_deadline(), Some(due));
        page.told(Time::ZERO, &f.client, Told::Changed { handle: page.document }, &mut out);
        assert!(out.is_empty(), "a delivery does not duplicate the waiting read");
        page.fire(due, &mut out);
        assert_eq!(out.pop(), Some(activity_read(&page)));
        let mut blocks = List::with_capacity(1);
        blocks.push(c::Block::Text(fixtures::acts::text(b"Committed reply"))).unwrap();
        page.told(
            due,
            &f.client,
            Told::Page {
                handle: page.activity,
                page: f::activity_reply(c::Speaker::Assistant, blocks.clone(), COMMITTED),
            },
            &mut out,
        );
        assert_eq!(page.thread.loaded, COMMITTED);
        let lines = page.thread.lines();
        let [jig_client::thread::Line::Turn { turn, .. }] = lines.as_slice() else { panic!("committed reply") };
        assert_eq!(turn.messages.get(0).unwrap().blocks, blocks);
        assert_eq!(page.next_deadline(), None);
    }
}

#[test]
fn newer_reads_only_on_an_agents_page() {
    use jig_client::thread::{Reading, window::Edge};
    for (shape, expected) in
        [(Shape::Agent, true), (Shape::Tracked, false), (Shape::HeaderAndPlan, false), (Shape::Unknown, false)]
    {
        let (mut page, mut fixture) = opened();
        page.shape = shape;
        page.thread.reading = Some(Reading::Latest { known: 0 });
        let jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Activity(rows))) = activity_page(f::POSITION)
        else {
            panic!("activity fixture")
        };
        let _read = page.thread.page(*rows);
        page.thread.lines.edges.before = Edge::Dropped;
        while fixture.out.pop().is_some() {}
        page.act(&fixtures::env(&fixtures::limits()), &fixture.client, Act::Newer, &mut 0, &mut fixture.out);
        let read = Request::Client(Event::Read {
            handle: page.activity,
            read: Readable::Jig(c::Read::ActivityBeside { task: f::TASK, position: f::POSITION, side: c::Side::Later }),
        });
        assert_eq!(fixture.out.pop(), expected.then_some(read), "{shape:?}");
        assert!(fixture.out.is_empty());
    }
}
#[test]
fn an_empty_agents_page_with_more_reads_on() {
    let mut fixture = f::opened();
    let mut page = Task::new(f::TASK, FIRST, &fixtures::limits(), &mut Queue::with_capacity(OUTPUTS));
    page.shape = Shape::Agent;
    let cursor = c::Cursor::new(Box::from(*b"older"), 5).unwrap();
    while fixture.out.pop().is_some() {}
    page.told(
        Time::ZERO,
        &fixture.client,
        Told::Page {
            handle: page.activity,
            page: jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Activity(Box::new(c::ActivityPage {
                rows: List::with_capacity(0),
                before: Some(cursor.clone()),
                after: None,
                cut: false,
            })))),
        },
        &mut fixture.out,
    );
    assert_eq!(
        fixture.out.pop(),
        Some(Request::Client(Event::Read {
            handle: page.activity,
            read: Readable::Jig(c::Read::Activity { task: f::TASK, before: Some(cursor) }),
        }))
    );
    assert!(fixture.out.is_empty());
}

#[test]
fn a_lasting_activity_end_is_not_asked_again_on_an_agents_page() {
    use jig_client::thread::{Reading, window::End};
    let (mut page, mut fixture) = opened();
    page.shape = Shape::Agent;
    while fixture.out.pop().is_some() {}
    page.thread.reading = Some(Reading::Latest { known: 0 });
    page.told(
        Time::ZERO,
        &fixture.client,
        Told::Page { handle: page.activity, page: jig_client::Readout::Jig(c::ReadResult::None(c::ReadNone::Busy)) },
        &mut fixture.out,
    );
    let due = page.next_deadline().expect("a busy answer retries");
    page.fire(due, &mut fixture.out);
    assert_eq!(fixture.out.pop(), Some(activity_read(&page)));
    page.told(
        due,
        &fixture.client,
        Told::Page {
            handle: page.activity,
            page: jig_client::Readout::Jig(c::ReadResult::None(c::ReadNone::Malformed)),
        },
        &mut fixture.out,
    );
    assert_eq!(page.next_deadline(), None, "a lasting end clears the retrying read");
    assert_eq!(page.thread.read_failed, Some(End::Before));
    assert!(page.thread.reading.is_none());
    page.fire(due.saturating_add(skein_lib::Duration::from_secs(120)), &mut fixture.out);
    assert!(fixture.out.is_empty(), "time alone does not repeat a lasting end");
}

fn composer_task(priority: Option<u32>, limits: &crate::Limits) -> (Task, f::Fixture) {
    let mut fixture = f::with_delegate();
    super::composer::deliver_me(&mut fixture);
    let mut task = offered_task(c::ActKind::Message, Some(c::Admission::Allowed));
    task.priority = priority;
    f::deliver(&mut fixture.client, fixture.watch, task);
    let mut page = Task::new(f::TASK, FIRST, limits, &mut Queue::uncapped());
    page.told(Time::ZERO, &fixture.client, Told::Changed { handle: page.document }, &mut Queue::uncapped());
    (page, fixture)
}
fn composer_act(page: &mut Task, fixture: &mut f::Fixture, limits: &crate::Limits, act: Act) {
    page.act(&fixtures::env(limits), &fixture.client, act, &mut 0, &mut fixture.out);
}
#[test]
fn a_kilobyte_pasted_is_sent_whole() {
    let limits = super::composer::limits();
    for priority in [None, Some(PRIORITY)] {
        let (mut page, mut f) = composer_task(priority, &limits);
        let words = super::composer::words(super::composer::BOUND);
        composer_act(&mut page, &mut f, &limits, Act::MessageWords { words: words.clone() });
        assert_eq!(page.words, words, "{priority:?}");
        composer_act(&mut page, &mut f, &limits, Act::Send);
        assert_eq!(
            fixtures::acts::sent(&mut f.out).as_ref(),
            [c::Ask::Message {
                project: fixtures::PROJECT,
                task: f::TASK,
                words: c::Text::new(words, super::composer::HOLD).unwrap()
            }],
            "{priority:?}"
        );
        assert!(page.words.is_empty());
    }
}
#[test]
fn a_paste_past_the_bound_stays_and_sends_nothing() {
    let limits = super::composer::limits();
    for priority in [None, Some(PRIORITY)] {
        for &(typed, kept, beyond, excess) in super::composer::PAST {
            let (mut page, mut f) = composer_task(priority, &limits);
            composer_act(&mut page, &mut f, &limits, Act::MessageWords { words: super::composer::words(typed) });
            let words = super::composer::words(kept);
            assert_eq!(page.words, words, "{priority:?}");
            assert_eq!(page.beyond, beyond);
            composer_act(&mut page, &mut f, &limits, Act::Send);
            assert!(fixtures::acts::sent(&mut f.out).is_empty());
            assert!(page.pending.is_empty());
            assert!(page.thread.words.rows.is_empty());
            assert_eq!(page.words, words);
            assert_eq!(pages::composer::over(&page.words, page.beyond, super::composer::BOUND), excess);
            composer_act(&mut page, &mut f, &limits, Act::MessageWords { words: Box::from(*b"short") });
            assert_eq!(page.beyond, 0);
            composer_act(&mut page, &mut f, &limits, Act::Send);
            assert_eq!(fixtures::acts::sent(&mut f.out).len(), 1);
        }
    }
}
#[test]
fn a_send_the_thread_refuses_keeps_the_words_and_says_so() {
    let limits = super::composer::limits();
    for priority in [None, Some(PRIORITY)] {
        let (mut page, mut f) = composer_task(priority, &limits);
        page.thread = jig_client::thread::Thread::new(
            f::TASK,
            &jig_client::thread::Limits {
                lines: 1,
                bytes: 4,
                text: super::composer::HOLD,
                live: super::composer::HOLD,
            },
        );
        composer_act(&mut page, &mut f, &limits, Act::MessageWords { words: Box::from(*b"Hello") });
        composer_act(&mut page, &mut f, &limits, Act::Send);
        assert_eq!(page.words.as_ref(), b"Hello", "{priority:?}");
        assert!(fixtures::acts::sent(&mut f.out).is_empty());
        assert!(page.pending.is_empty());
        assert!(page.thread.words.rows.is_empty());
        assert_eq!(page.outcome, Some(jig_client::Outcome::Limit));
    }
}

// client/pages/task.md, 4: Talk uses the composer's message lifecycle, not steering.
const ROLE: u64 = 52;
const READER: u64 = 3;
const TALK_WORDS: &[u8] = b"From now on, reproduce a flake twice before routing it.";
fn talk_pages(task: c::Task) -> (Pages, jig_client::Client) {
    task_pages_with(task, READER, &super::composer::limits())
}
fn talk_act(pages: &mut Pages, client: &jig_client::Client, act: Act) -> Queue<Request> {
    let mut out = Queue::uncapped();
    pages::step(pages, &fixtures::env(&super::composer::limits()), client, pages::Event::Act { act }, &mut out);
    out
}
fn talk_text() -> c::Text { c::Text::new(Box::from(TALK_WORDS), super::composer::HOLD).unwrap() }
fn standing_task() -> c::Task {
    let mut task = offered_task(c::ActKind::Message, Some(c::Admission::Allowed));
    task.chip.task = ROLE;
    task.flags.ended_by = c::EndedBy::Standing;
    task.acts = List::from_boxed(Box::new([
        c::Offered { act: c::ActKind::Talk, admission: c::Admission::Allowed, effect: None },
        c::Offered { act: c::ActKind::Message, admission: c::Admission::Allowed, effect: None },
        c::Offered { act: c::ActKind::Leave, admission: c::Admission::Allowed, effect: None },
    ]));
    task
}
fn task_answer(pages: &mut Pages, client: &jig_client::Client, answer: c::Answer) {
    let key = pages.task().unwrap().pending.as_slice()[0].0;
    pages::step(pages, &fixtures::env(&fixtures::limits()), client,
        pages::Event::Told { told: Told::Answered { key,
            outcome: jig_client::Outcome::Answered(jig_client::Answered::Jig(answer)) } },
        &mut Queue::uncapped());
}
#[test]
fn a_standing_roles_send_talks_and_keeps_pending_words_until_sent() {
    let (mut pages, client) = talk_pages(standing_task());
    talk_act(&mut pages, &client, Act::MessageWords { words: Box::from(TALK_WORDS) });
    assert_eq!(fixtures::acts::sent(&mut talk_act(&mut pages, &client, Act::Send)).as_ref(),
        [c::Ask::Talk { project: 1, task: ROLE, words: talk_text() }]);
    let page = pages.task().unwrap();
    assert!(page.words.is_empty());
    let pending = page.thread.words.rows.as_slice().first().unwrap();
    assert_eq!(pending.by.as_ref().unwrap().number, READER);
    assert_eq!(pending.words.bytes(), TALK_WORDS);
    assert_eq!(pending.state, jig_client::thread::words::Words::BeingSent);
    talk_act(&mut pages, &client, Act::MessageWords { words: Box::from(TALK_WORDS) });
    assert!(fixtures::acts::sent(&mut talk_act(&mut pages, &client, Act::Send)).is_empty());
    task_answer(&mut pages, &client, c::Answer::Done(c::Outcome::Document(c::DocumentOutcome::Said { task: ROLE, message: 7 })));
    let page = pages.task().unwrap();
    assert!(page.pending.is_empty());
    assert!(page.outcome.is_none());
    assert_eq!(page.thread.words.rows.as_slice()[0].state, jig_client::thread::words::Words::Sent { message: 7 });
}
#[test]
fn talk_sends_nothing_for_empty_or_over_bound_words_and_other_agents_still_message() {
    for words in [Box::from(b"".as_slice()), super::composer::words(1025)] {
        let (mut pages, client) = talk_pages(standing_task());
        talk_act(&mut pages, &client, Act::MessageWords { words: words.clone() });
        assert!(fixtures::acts::sent(&mut talk_act(&mut pages, &client, Act::Send)).is_empty());
        assert!(pages.task().unwrap().pending.is_empty());
        assert_eq!(pages.task().unwrap().words, words);
    }
    let mut task = standing_task();
    task.flags.ended_by = c::EndedBy::Result;
    let (mut pages, client) = talk_pages(task);
    talk_act(&mut pages, &client, Act::MessageWords { words: Box::from(TALK_WORDS) });
    assert_eq!(fixtures::acts::sent(&mut talk_act(&mut pages, &client, Act::Send)).as_ref(),
        [c::Ask::Message { project: 1, task: ROLE, words: talk_text() }]);
}
#[test]
fn leave_sends_at_once_only_for_the_readers_admitted_open_talk() {
    for (party, offered, expected) in [(Some(READER), true, true), (Some(4), true, false), (None, true, false), (Some(READER), false, false)] {
        let mut task = standing_task();
        task.talk = match party { Some(party) => Some(c::Talk { party, opened: Wall::EPOCH, last: Wall::EPOCH }), None => None };
        if !offered { task.acts.truncate(2); }
        let (mut pages, client) = talk_pages(task);
        let sent = fixtures::acts::sent(&mut talk_act(&mut pages, &client, Act::LeaveTalk));
        assert!(pages.confirmation().is_none());
        if expected {
            assert_eq!(sent.as_ref(), [c::Ask::Leave { project: 1, task: ROLE }]);
            task_answer(&mut pages, &client, c::Answer::Done(c::Outcome::Document(c::DocumentOutcome::Left { task: ROLE })));
            assert_eq!(pages.needs().toast(), Some(&pages::needs::Toasted::LeftTalk));
            assert!(pages.task().unwrap().outcome.is_none());
        } else { assert!(sent.is_empty()); }
    }
}
#[test]
fn talk_and_leave_keep_jigs_refusal_and_never_fall_back_to_message() {
    for act in [Act::Send, Act::LeaveTalk] {
        let mut task = standing_task();
        task.talk = Some(c::Talk { party: READER, opened: Wall::EPOCH, last: Wall::EPOCH });
        let (mut pages, client) = talk_pages(task);
        talk_act(&mut pages, &client, Act::MessageWords { words: Box::from(TALK_WORDS) });
        assert_eq!(fixtures::acts::sent(&mut talk_act(&mut pages, &client, act)).len(), 1);
        let answer = c::Answer::Refused { refusal: c::Refusal::NotOffered, at: None };
        task_answer(&mut pages, &client, answer.clone());
        assert_eq!(pages.task().unwrap().outcome,
            Some(jig_client::Outcome::Answered(jig_client::Answered::Jig(answer))));
        assert!(pages.task().unwrap().thread.words.rows.is_empty());
    }
    let mut task = standing_task();
    task.acts.get_mut(0).unwrap().admission = c::Admission::Refused(c::Refusal::NotOffered);
    let (mut pages, client) = talk_pages(task);
    talk_act(&mut pages, &client, Act::MessageWords { words: Box::from(TALK_WORDS) });
    assert!(fixtures::acts::sent(&mut talk_act(&mut pages, &client, Act::Send)).is_empty());
}
