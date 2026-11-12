//! A closer's funds follow its task's runs (jig/docs/design/domain/tasks/lifecycle.md, 11).
use super::revisions::{Harness, amendment, task, to};
use crate::{
    Active, AuthorityExecutor, Cause, Closer, End, EndedBy, Event, Funder, Numbers, Party, Phase, ProcedureDecision,
    ProcedureEnd, Reasons, TaskResult, Why,
};
use skein_lib::Queue;

const TASK: u64 = 1;
const CLOSER: u64 = 2;
const ENDED: u64 = 3;
const LIVE: u64 = 4;

fn start(h: &mut Harness, number: u64, budget: u64) {
    let row = h.domain.task(number).expect("fixture task");
    if row.phase == Phase::Active(Active::Idle) {
        let message = row.last_message.checked_add(1).expect("fixture message");
        h.send(Event::Message {
            reply_to: to(),
            project: 1,
            operation: crate::Message {
                task: number,
                word: crate::Word {
                    number: message,
                    from: Party::Person(9),
                    kind: crate::MessageKind::Words,
                    words: Box::from(&b"Judge again."[..]),
                    at: h.env.wall,
                    hits: 1,
                    eligible: true,
                },
            },
        });
    }
    let context = h.domain.run_context(number).expect("fixture run context");
    let attempt = context.previous_attempt.checked_add(1).expect("fixture attempt");
    h.send(Event::Prepare { reply_to: to(), task: number });
    h.send(Event::Claim {
        reply_to: to(),
        task: number,
        attempt,
        budget,
        session_end: None,
        resumed: false,
        account: None,
        writes: Box::new([]),
        reasons: context.reasons,
        offered: Reasons::EMPTY,
    });
    h.send(Event::Started { task: number, attempt });
    assert_eq!(h.domain.task(number).expect("claimed task").phase, Phase::Active(Active::Running { attempt }));
    if let Some(word) = context.inbox.last() {
        h.send(Event::Turn {
            reply_to: to(),
            task: number,
            attempt,
            turn: 1,
            read: Some(word.number),
            offered: Some(word.number),
            outstanding: Box::new([word.number]),
            refused: Box::new([]),
            cumulative: 0,
        });
        assert!(h.domain.task(number).expect("read inbox").inbox.is_empty());
    }
}

fn answer(h: &mut Harness, number: u64, spent: u64, end: End) {
    let attempt = h.domain.task(number).expect("running task").attempt;
    h.send(Event::Activation {
        reply_to: to(),
        task: number,
        attempt,
        end,
        saved: None,
        cause: Cause::Priced { cumulative: spent },
    });
}

fn wait(h: &mut Harness) {
    let step = h.domain.task(CLOSER).expect("live closer").attempt.checked_add(1).expect("fixture step");
    let read = h.domain.procedure_offered(CLOSER);
    h.send(Event::Procedure {
        reply_to: to(),
        task: CLOSER,
        step,
        read,
        decision: ProcedureDecision::new(ProcedureEnd::Wait),
    });
    assert_eq!(h.domain.task(CLOSER).expect("waiting closer").phase, Phase::Active(Active::Idle));
}

fn fixture() -> Harness {
    let mut h = Harness::new();
    let mut new = task(TASK, &[]);
    new.flags.ended_by = EndedBy::Closer(Closer { connector: 1, code: 6 });
    new.authority.delegation.kinds = Box::new([AuthorityExecutor::Charter(1), AuthorityExecutor::Procedure(6)]);
    new.authority.budget.spend = 1_000;
    new.numbers.budget = 1_000;
    h.make([new].into());
    start(&mut h, TASK, 300);
    answer(&mut h, TASK, 10, End::Parked);
    assert_eq!(h.domain.task(TASK).expect("parked task").phase, Phase::Active(Active::Idle));
    h.send(Event::CloserBatch { task: TASK, number: CLOSER });
    assert_eq!(h.domain.task(TASK).expect("closer's task").closer, Some(CLOSER));
    assert_eq!(h.domain.task(CLOSER).expect("AsAllowed closer").numbers.budget, 990);
    wait(&mut h);
    h
}

fn child(h: &mut Harness, parent: u64, number: u64, budget: u64) {
    let mut new = task(number, &[]);
    new.authority.budget.spend = budget;
    new.numbers.budget = budget;
    new.funder = Funder::Task(parent);
    h.send(Event::Make { reply_to: to(), creator: Party::Task(parent), batch: Box::new([new]) });
    assert_eq!(h.domain.task(number).expect("funded delegate").funder, Funder::Task(parent));
}

fn finish(h: &mut Harness, number: u64, spent: u64) {
    start(h, number, 100);
    answer(
        h,
        number,
        spent,
        End::Finished { result: TaskResult::Report { words: Box::new([]) }, cancel_delegates: false },
    );
    h.send(Event::EffectsSettled { task: number });
    h.send(Event::Settled { task: number });
    assert!(h.domain.task(number).is_none(), "delegate ended");
}

fn due(h: &Harness, budget: u64) {
    let row = h.domain.task(CLOSER).expect("live closer");
    assert_eq!(row.numbers.budget, budget);
    assert_eq!(row.phase, Phase::Active(Active::Due), "returned funds wake the idle closer");
    assert_eq!(row.due_reason, Why::Changed);
    assert!(h.domain.procedure_due(CLOSER).is_some(), "the closer can step again");
}

#[test]
fn a_run_borrows_only_unreserved_closer_funds_and_its_answer_wakes_the_closer() {
    let mut h = fixture();
    h.send(Event::ChargeEffect { funder: Funder::Task(CLOSER), maximum: 20 });
    child(&mut h, CLOSER, ENDED, 100);
    child(&mut h, CLOSER, LIVE, 50);
    finish(&mut h, ENDED, 30);
    assert_eq!(
        h.domain.task(CLOSER).expect("idle closer").numbers,
        Numbers { budget: 990, spent: 20, spent_below: 30, reserved: 50 }
    );
    start(&mut h, TASK, 300);
    assert_eq!(
        h.domain.task(CLOSER).expect("closer floor").numbers,
        Numbers { budget: 100, spent: 20, spent_below: 30, reserved: 50 }
    );
    answer(&mut h, TASK, 100, End::Parked);
    due(&h, 890);
}

#[test]
fn an_answer_with_nothing_left_still_wakes_the_idle_closer() {
    let mut h = fixture();
    start(&mut h, TASK, 990);
    assert_eq!(h.domain.task(CLOSER).expect("closer lent all funds").numbers.budget, 0);
    answer(&mut h, TASK, 990, End::Parked);
    due(&h, 0);
}

#[test]
fn an_idle_tasks_funded_delegate_ending_wakes_its_closer() {
    let mut h = fixture();
    start(&mut h, TASK, 300);
    child(&mut h, TASK, ENDED, 100);
    answer(&mut h, TASK, 100, End::Parked);
    wait(&mut h);
    assert_eq!(h.domain.task(TASK).expect("idle funding task").phase, Phase::Active(Active::Idle));
    assert_eq!(h.domain.task(CLOSER).expect("idle closer").numbers.budget, 790);
    finish(&mut h, ENDED, 30);
    due(&h, 860);
}

#[test]
fn an_idle_tasks_amendment_wakes_its_closer_without_more_funds() {
    let mut h = fixture();
    // The added dependency keeps the amended task idle while its closer can step.
    h.make([task(LIVE, &[])].into());
    let mut change = amendment();
    change.dependencies = Some(Box::new([LIVE]));
    h.amend(TASK, change);
    assert_eq!(h.domain.task(TASK).expect("idle amended task").phase, Phase::Active(Active::Idle));
    due(&h, 990);
}

#[test]
fn taking_funds_preserves_nonidle_closers_and_tasks_without_one() {
    for phase in [
        Phase::Active(Active::Due),
        Phase::Closing(crate::Closing {
            stage: crate::Stage::Effects,
            ending: crate::Ending::Failed { reason: Box::new([]) },
        }),
        Phase::Ended(crate::Ending::Failed { reason: Box::new([]) }),
    ] {
        let mut h = fixture();
        let row = &mut crate::domain::task_mut(&mut h.domain, CLOSER).expect("fixture closer").record;
        row.phase = phase.clone();
        row.due_reason = Why::Made;
        let before = row.clone();
        let mut out = Queue::uncapped();
        crate::closer::take_available(&mut h.domain, &h.env, TASK, &mut out);
        assert_eq!(h.domain.task(CLOSER).expect("unchanged closer"), &before);
        assert!(out.is_empty(), "no resize or activation without growth or an idle closer");
    }
    let mut h = Harness::new();
    h.make([task(TASK, &[])].into());
    let row = &mut crate::domain::task_mut(&mut h.domain, TASK).expect("fixture task").record;
    row.phase = Phase::Active(Active::Idle);
    let before = row.clone();
    let mut out = Queue::uncapped();
    crate::closer::take_available(&mut h.domain, &h.env, TASK, &mut out);
    assert_eq!(h.domain.task(TASK).expect("task without closer"), &before);
    assert!(out.is_empty());
}
