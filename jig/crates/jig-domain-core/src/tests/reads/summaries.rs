//! Lines survive record erasure; point reads do not (protocol/store/retention.md, 7).
use crate::reads::{self, Answer, EndedGroup, Loaded, Read, Why};
use crate::tests::*;

const TASK: u64 = 90;
const QUERY: TaskQuery = TaskQuery {
    project: Some(1),
    requester: None,
    root: None,
    parent: None,
    label: None,
    phase: Some(PhaseClass::Ended),
    executor: None,
    tracked: None,
    order: QueryOrder::Ended,
};

fn summary() -> tasks::TaskSummary {
    let mut row = fixtures::ended_report(TASK, tasks::Party::Person(1));
    row.spec.words = b"Ship it\nFull specification".as_slice().into();
    row.executor = tasks::Executor::Procedure { connector: 2, code: 3 };
    row.label = 7;
    row.tracked = Some(4);
    row.flags.listed = true;
    row.changed = Wall::from_nanos(11);
    row.last_activity = Wall::from_nanos(12);
    row.ended_at = Some(Wall::from_nanos(13));
    row.numbers = tasks::Numbers { budget: 100, spent: 7, spent_below: 11, reserved: 13 };
    row.authority.budget.deadline = Some(Wall::from_nanos(50));
    row.ended_delegates = tasks::Endings { done: 4, failed: 2, cancelled: 0 };
    tasks::TaskSummary::of(&row)
}

fn summary_row(summary: &tasks::TaskSummary) -> reads::Row {
    let mut row = stored_row(
        Record::Tasks(tasks::Stored::EndedByProject(tasks::EndedByProject {
            project: summary.project,
            label: summary.label,
            ended_at: summary.ended_at,
            task: summary.number,
        })),
        None,
    );
    row.summary = Some(Box::new(summary.clone()));
    row
}

#[test]
fn a_summary_line_keeps_its_chip_budget_identity_and_ended_counts() {
    let (core, limits) = watch_core();
    let summary = summary();
    let line = task_line_from_summary(&core, &limits, 1, &summary).unwrap();
    assert_eq!(line.chip.number, TASK);
    assert_eq!(line.chip.title, Text { bytes: b"Ship it".as_slice().into(), cut: false });
    assert_eq!(line.chip.executor, ExecutorKind::Procedure);
    assert_eq!(line.chip.phase, PhaseLine::Ended);
    assert_eq!(
        line.chip.details.ending,
        Some(Ending::Done(ResultValue::Report(Text { bytes: b"done".as_slice().into(), cut: false })))
    );
    assert_eq!(line.chip.details.ended_at, Some(summary.ended_at));
    assert_eq!(line.project, summary.project);
    assert_eq!(line.requester, summary.requester);
    assert_eq!(line.label, 7);
    assert_eq!(line.flags, summary.flags);
    assert_eq!(line.tracked, Some(4));
    assert_eq!(line.changed, summary.changed);
    assert_eq!(line.last_activity, summary.last_activity);
    assert_eq!(line.ended_at, Some(summary.ended_at));
    assert_eq!(
        line.budget,
        BudgetLine { numbers: summary.numbers, left: 69, funder: summary.funder, deadline: summary.budget.deadline }
    );
    assert_eq!(line.chip.details.budget, line.budget);
    assert_eq!((line.delegates_done, line.delegates_total), (4, 6));
    assert_eq!(line.phases, PhaseCounts::ZERO);
    assert_eq!(line.held_below, 0);
    assert_eq!(line.waits_on, WaitsOn::Nothing);
    assert_eq!(line.conversation, None);
    assert_eq!(line.chip.details.live_delegates, 0);
    assert_eq!(line.chip.details.observation, None);
    assert_eq!(task_line_from_summary(&core, &limits, 999, &summary), Err(Missing::NotAllowed));
    let mut cut = summary;
    cut.title_cut = true;
    cut.result.as_mut().unwrap().words_cut = true;
    let chip = chip_from_summary(&limits, &cut);
    assert!(chip.title.cut);
    let Some(Ending::Done(ResultValue::Report(words))) = &chip.details.ending else { panic!("report") };
    assert!(words.cut);
}

#[test]
fn a_query_read_returns_the_summary_line_and_filters_its_kept_keys() {
    let (mut core, limits) = watch_core();
    let summary = summary();
    for (query, count) in [
        (QUERY, 1),
        (TaskQuery { executor: Some(ExecutorKind::Agent), ..QUERY }, 0),
        (TaskQuery { tracked: Some(false), ..QUERY }, 0),
        (TaskQuery { root: Some(999), ..QUERY }, 0),
    ] {
        core.reclaim();
        let Held::ReadLoad { load, .. } =
            read_start(&mut core, &limits, 10, Read::Tasks { query, live: None, ended: None })
        else {
            panic!("ended task query loads")
        };
        let answer = loaded_answer(
            &mut core,
            &limits,
            load,
            Loaded::Page { rows: Box::new([summary_row(&summary)]), next: None },
        )
        .unwrap();
        let Answer::Tasks(page) = answer else { panic!("task groups: {answer:?}") };
        let EndedGroup::Rows { rows, next: None, cut: false } = page.ended else { panic!("ended lines") };
        assert_eq!(rows.len(), count, "{query:?}");
        if let Some(line) = rows.get(0) {
            assert_eq!(*line, task_line_from_summary(&core, &limits, 1, &summary).unwrap());
        }
    }
}

#[test]
fn an_ended_delegate_read_draws_its_chip_from_the_summary() {
    let (mut core, limits) = watch_core();
    let mut summary = summary();
    summary.requester = tasks::Party::Task(1);
    let mut row = summary_row(&summary);
    row.value = Record::Tasks(tasks::Stored::EndedDelegate(tasks::Edge { requester: 1, task: TASK }));
    let Held::ReadLoad { load, .. } = read_start(&mut core, &limits, 10, Read::EndedDelegates { task: 1, from: None })
    else {
        panic!("ended delegates load")
    };
    let answer = loaded_answer(&mut core, &limits, load, Loaded::Page { rows: Box::new([row]), next: None }).unwrap();
    let Answer::Page(page) = answer else { panic!("delegate page") };
    let [reads::PageRow::EndedDelegate(delegate)] = page.rows.as_slice() else { panic!("one delegate: {page:?}") };
    assert_eq!(delegate.chip, chip_from_summary(&limits, &summary));
    assert_eq!(delegate.at, Some(summary.ended_at));
    assert_eq!(Some(&delegate.ended), delegate.chip.details.ending.as_ref());
}

#[test]
fn a_gone_point_load_answers_gone() {
    let (mut core, limits) = watch_core();
    for read in [
        Read::Task { task: TASK },
        Read::Result { task: TASK },
        Read::TaskText { task: TASK, field: reads::TaskTextField::Specification, from: 0 },
    ] {
        core.reclaim();
        let Held::ReadLoad { load, .. } = read_start(&mut core, &limits, 10, read) else { panic!("retained load") };
        assert_eq!(loaded_answer(&mut core, &limits, load, Loaded::Gone), Some(Answer::None(Why::Gone)));
    }
}
