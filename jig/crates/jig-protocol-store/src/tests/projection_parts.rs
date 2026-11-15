//! Whole projection fixtures and the split job's two-write edge (protocol/store/parts.md, 3–5).
use super::fixtures;
use crate::{JobPage, JobRoom, JobRow, JobWrite, codec, core_v6 as wire, jobs, keys};
use jig_domain_core_vocabulary as core;
use jig_domain_tasks as tasks;
use skein_lib::{Reader, Writer};

pub(crate) fn goal(number: u64) -> core::ProjectionGoal {
    core::ProjectionGoal {
        number,
        project: 2,
        priority: 7,
        words: b"goal words".as_slice().into(),
        phase: fixtures::task_phase(),
    }
}
pub(crate) fn task(number: u64) -> core::ProjectionTask {
    core::ProjectionTask {
        number: number.checked_add(1).unwrap(),
        requester: tasks::Party::Task(number),
        words: b"plan words".as_slice().into(),
        phase: fixtures::task_phase(),
    }
}
pub(crate) fn records(number: u64) -> [core::ProjectionRecord; 4] {
    let (goal, goal_words) = goal(number).split();
    let (task, task_words) = task(number).split();
    [
        core::ProjectionRecord::GoalHot { goal, pending: Box::new([2, 3]), changed: true, fed_at: Some(19) },
        core::ProjectionRecord::TaskHot { goal: number, task },
        core::ProjectionRecord::GoalWords { goal: number, task: number, words: goal_words },
        core::ProjectionRecord::TaskWords { goal: number, task: number.checked_add(1).unwrap(), words: task_words },
    ]
}
pub(crate) fn key(row: &core::ProjectionRecord) -> Box<[u8]> {
    keys::encode(&core::Key::Core(core::CoreKey::Projection(row.key())), fixtures::KEY_LIMIT).unwrap()
}
fn phase(phase: &tasks::Phase) -> wire::PhaseBytes {
    let limits = codec::limits(&fixtures::CORE_LIMITS).unwrap();
    wire::PhaseBytes::new(
        &limits.core,
        wire::PhaseBytesParts { value: codec::tasks::carried::phase(phase, &limits.tasks).unwrap() },
    )
    .unwrap()
}
pub(crate) fn encoded(value: wire::ProjectionRecord) -> Box<[u8]> {
    let value = wire::ProjectionValue::new(&wire::CEILINGS, wire::ProjectionValueParts { value }).unwrap();
    let row = wire::Row::new(&wire::CEILINGS, wire::RowParts { value: wire::Value::Projection(value) }).unwrap();
    let mut writer = Writer::new(usize::try_from(row.measure()).unwrap());
    row.encode(&mut writer).unwrap();
    writer.finish()
}
pub(crate) fn legacy(number: u64) -> [JobRow; 2] {
    let core::ProjectionGoal { number, project, priority, words, phase: goal_phase } = goal(number);
    let goal = wire::ProjectionGoal::new(
        &wire::CEILINGS,
        wire::ProjectionGoalParts { number, project, priority, words, phase: phase(&goal_phase) },
    )
    .unwrap();
    let mut pending = skein_lib::List::with_capacity(2);
    for value in [2, 3] {
        pending.push(wire::Connector::new(&wire::CEILINGS, wire::ConnectorParts { value }).unwrap()).unwrap();
    }
    let goal = wire::ProjectionRecord::Goal(
        wire::ProjectionGoalRow::new(
            &wire::CEILINGS,
            wire::ProjectionGoalRowParts { goal, pending, changed: true, fed_at: Some(19) },
        )
        .unwrap(),
    );
    let core::ProjectionTask { number: task, requester, words, phase: task_phase } = task(number);
    let tasks::Party::Task(requester) = requester else { unreachable!() };
    let requester =
        wire::Party::Task(wire::PartyTask::new(&wire::CEILINGS, wire::PartyTaskParts { value: requester }).unwrap());
    let task = wire::ProjectionTask::new(
        &wire::CEILINGS,
        wire::ProjectionTaskParts { number: task, requester, words, phase: phase(&task_phase) },
    )
    .unwrap();
    let task = wire::ProjectionRecord::Task(
        wire::ProjectionTaskRow::new(&wire::CEILINGS, wire::ProjectionTaskRowParts { goal: number, task }).unwrap(),
    );
    let records = records(number);
    [JobRow { key: key(&records[0]), value: encoded(goal) }, JobRow { key: key(&records[1]), value: encoded(task) }]
}
fn put_bytes(write: &JobWrite) -> u32 {
    let JobWrite::Put { key, value } = write else { unreachable!() };
    skein_kv::put_bytes(u32::try_from(key.len()).unwrap(), u32::try_from(value.len()).unwrap()).unwrap()
}

fn writes_bytes(writes: &[JobWrite]) -> u32 {
    let mut size = 0_u32;
    for write in writes {
        size = size.checked_add(put_bytes(write)).unwrap();
    }
    size
}

#[test]
fn projection_parts_pages_obey_both_edges_and_pass_rows_already_in_parts() {
    let mut page = JobPage {
        rows: legacy(11).into(),
        end: false,
        layout: None,
        room: JobRoom { writes: 4, bytes: u64::from(u32::MAX) },
    };
    let answer = jobs::page("projection-parts", &page).unwrap();
    assert_eq!(answer.writes.len(), 4);
    let first_bytes = writes_bytes(&answer.writes[..2]);
    let total = writes_bytes(&answer.writes);
    for edge in [
        JobRoom { writes: 2, bytes: u64::from(u32::MAX) },
        JobRoom { writes: 2, bytes: u64::from(first_bytes) },
        JobRoom { writes: 4, bytes: u64::from(total - 1) },
    ] {
        page.room = edge;
        let answer = jobs::page("projection-parts", &page).unwrap();
        assert_eq!(answer.writes.len(), 2);
        assert_eq!(answer.through, Some(page.rows[0].key.clone()));
        assert_eq!(writes_bytes(&answer.writes), first_bytes);
    }
    for edge in
        [JobRoom { writes: 1, bytes: u64::from(u32::MAX) }, JobRoom { writes: 4, bytes: u64::from(first_bytes - 1) }]
    {
        page.room = edge;
        let refusal = jobs::page("projection-parts", &page).unwrap_err();
        assert_eq!(refusal.key, Some(page.rows[0].key.clone()));
        assert!(refusal.why.contains("edge"));
    }
    let limits = codec::limits(&fixtures::CORE_LIMITS).unwrap();
    let mut rows = skein_lib::List::with_capacity(4);
    for row in records(11) {
        rows.push(JobRow {
            key: key(&row),
            value: codec::core::encode(&core::CoreRecord::Projection(row), &limits).unwrap(),
        })
        .unwrap();
    }
    page.rows = rows.into_boxed();
    page.room = JobRoom { writes: 0, bytes: 0 };
    let answer = jobs::page("projection-parts", &page).unwrap();
    assert!(answer.writes.is_empty());
    assert_eq!(answer.through, Some(page.rows[3].key.clone()));
    page.rows = Box::new([]);
    page.end = true;
    let answer = jobs::page("projection-parts", &page).unwrap();
    assert!(answer.writes.is_empty());
    assert_eq!(answer.through, None);
    page.rows = Box::new([JobRow { key: key(&records(11)[0]), value: Box::new([]) }]);
    assert_eq!(jobs::page("projection-parts", &page).unwrap_err().key, Some(page.rows[0].key.clone()));
    // Whole variants are only inputs to the job, not restored projection rows.
    let legacy = legacy(11);
    codec::core::decode(&legacy[0].value, &limits).unwrap_err();
    wire::Row::decode(&wire::CEILINGS, &mut Reader::new(&legacy[0].value)).unwrap();
}
