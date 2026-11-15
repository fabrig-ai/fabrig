//! Milestone folds and page boundaries (domain/tasks/lifecycle.md, 12.4).
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    clippy::arithmetic_side_effects,
    reason = "ordinary Rust codec contract tests"
)]
use crate::{JobAnswer, JobPage, JobRoom, JobRow, JobWrite, codec, jobs, keys, tasks_v13};
use jig_domain_core_vocabulary::Key;
use jig_domain_tasks::{Active, AttemptRow, Closing, Held, Hold, HoldRow, Milestone, Phase, Stage, Stored, Was};
use skein_lib::Wall;

pub(crate) fn row(task: u64, position: u64, phase: Phase, at: u64) -> JobRow {
    let stored = Stored::Milestone(Milestone { task, position, phase, at: Wall::from_nanos(at) });
    JobRow { key: key(&stored), value: codec::tasks::encode(&stored, &tasks_v13::CEILINGS, &[]).unwrap() }
}
fn key(stored: &Stored) -> Box<[u8]> {
    keys::encode(&Key::Tasks(stored.key()), 1024).unwrap()
}
fn held(was: Was) -> Phase {
    Phase::Held { was, why: Hold::Stopped, source: None }
}
fn page(rows: Vec<JobRow>) -> JobPage {
    JobPage { rows: rows.into(), end: false, layout: None, room: JobRoom { writes: u32::MAX, bytes: u64::MAX } }
}
fn fold(page: &JobPage) -> JobAnswer {
    let answer = jobs::page("milestones", page).unwrap();
    assert!(answer.writes.len() <= usize::try_from(page.room.writes).unwrap());
    assert!(answer.writes.iter().map(JobWrite::bytes).sum::<u64>() <= page.room.bytes);
    answer
}
fn puts(answer: &JobAnswer) -> Vec<Stored> {
    answer
        .writes
        .iter()
        .filter_map(|write| {
            if let JobWrite::Put { key: found, value } = write {
                let stored = codec::tasks::decode(value, &tasks_v13::CEILINGS, &[]).unwrap();
                assert_eq!(found, &key(&stored));
                Some(stored)
            } else {
                None
            }
        })
        .collect()
}
fn erases(answer: &JobAnswer) -> Vec<Box<[u8]>> {
    answer
        .writes
        .iter()
        .filter_map(|write| if let JobWrite::Erase { key } = write { Some(key.clone()) } else { None })
        .collect()
}
pub(crate) fn ended_rows() -> Vec<JobRow> {
    [
        Phase::Waiting,
        Phase::Active(Active::Idle),
        Phase::Active(Active::Due),
        Phase::Active(Active::Preparing),
        Phase::Active(Active::Claimed { attempt: 1 }),
        Phase::Active(Active::Running { attempt: 1 }),
        Phase::Active(Active::Idle),
        held(Was::Active(Active::Idle)),
    ]
    .into_iter()
    .zip(1_u64..)
    .map(|(phase, position)| row(7, position, phase, position + 9))
    .collect()
}
pub(crate) fn ended_attempt() -> Stored {
    Stored::Attempt(AttemptRow {
        task: 7,
        attempt: 1,
        claimed: Some(Wall::from_nanos(14)),
        started: Some(Wall::from_nanos(15)),
        ended: Some(Wall::from_nanos(16)),
        end: None,
        phases: Some(2),
    })
}
pub(crate) fn idle_hold() -> Stored {
    Stored::Hold(HoldRow {
        task: 7,
        position: 8,
        held: Held { was: Was::Active(Active::Idle), why: Hold::Stopped, source: None },
        at: Wall::from_nanos(17),
    })
}
#[test]
fn one_tasks_eight_milestones_fold_to_an_attempt_and_a_hold() {
    let page = page(ended_rows());
    let answer = fold(&page);
    assert_eq!(answer.through, None);
    assert_eq!(erases(&answer), page.rows.iter().map(|row| row.key.clone()).collect::<Vec<_>>());
    let found = puts(&answer);
    assert_eq!(found, [ended_attempt(), idle_hold()]);
    for (stored, tag, tail) in [(&found[0], 22, 1_u64), (&found[1], 23, 8_u64)] {
        let expected = [&[1, 1, tag], 7_u64.to_be_bytes().as_slice(), tail.to_be_bytes().as_slice()].concat();
        assert_eq!(key(stored).as_ref(), expected);
    }
}
#[test]
fn held_and_closing_phases_name_attempts_until_they_move_and_each_hold_is_kept() {
    // A closing Run still names its attempt; its outcome cannot supply the run's end kind.
    let ending = jig_domain_tasks::Ending::Failed { reason: b"run failed".as_slice().into() };
    let rows = vec![
        row(7, 3, Phase::Active(Active::Claimed { attempt: 2 }), 10),
        row(7, 4, held(Was::Active(Active::Claimed { attempt: 2 })), 11),
        row(7, 6, Phase::Active(Active::Running { attempt: 2 }), 12),
        row(7, 7, held(Was::Active(Active::Running { attempt: 2 })), 13),
        row(7, 8, held(Was::Closing(Closing { stage: Stage::Run { attempt: 2 }, ending })), 14),
        row(7, 10, held(Was::Active(Active::Idle)), 15),
        row(7, 11, Phase::Active(Active::Running { attempt: 3 }), 16),
        row(7, 12, held(Was::Active(Active::Running { attempt: 3 })), 17),
    ];
    let answer = fold(&page(rows));
    let found = puts(&answer);
    let attempts: Vec<_> =
        found.iter().filter_map(|r| if let Stored::Attempt(row) = r { Some(row.clone()) } else { None }).collect();
    assert_eq!(
        attempts,
        [
            AttemptRow {
                task: 7,
                attempt: 2,
                claimed: Some(Wall::from_nanos(10)),
                started: Some(Wall::from_nanos(12)),
                ended: Some(Wall::from_nanos(15)),
                end: None,
                phases: Some(7)
            },
            AttemptRow {
                task: 7,
                attempt: 3,
                claimed: Some(Wall::from_nanos(16)),
                started: Some(Wall::from_nanos(16)),
                ended: None,
                end: None,
                phases: None
            },
        ]
    );
    let holds: Vec<_> = found
        .iter()
        .filter_map(|r| if let Stored::Hold(row) = r { Some((row.position, row.at)) } else { None })
        .collect();
    assert_eq!(
        holds,
        [
            (4, Wall::from_nanos(11)),
            (7, Wall::from_nanos(13)),
            (8, Wall::from_nanos(14)),
            (10, Wall::from_nanos(15)),
            (12, Wall::from_nanos(17))
        ]
    );
}
#[test]
fn pages_leave_the_last_task_but_consume_a_single_partial_task() {
    let mut three_tasks = vec![row(6, 1, Phase::Waiting, 9)];
    three_tasks.extend(ended_rows());
    three_tasks.push(row(8, 1, Phase::Waiting, 20));
    let answer = fold(&page(three_tasks));
    assert_eq!(
        answer.through,
        keys::encode(&Key::Tasks(jig_domain_tasks::Key::Milestone { task: 7, position: 8 }), 1024)
    );
    assert_eq!(erases(&answer).len(), 9);
    assert_eq!(puts(&answer), [ended_attempt(), idle_hold()]);
    let mut rows = ended_rows();
    let last = rows.last().unwrap().key.clone();
    rows.push(row(8, 1, Phase::Active(Active::Running { attempt: 4 }), 20));
    let answer = fold(&page(rows));
    assert_eq!(answer.through, Some(last));
    assert_eq!(erases(&answer).len(), 8);
    assert_eq!(puts(&answer), [ended_attempt(), idle_hold()]);
    let answer = fold(&page(vec![row(8, 1, Phase::Active(Active::Claimed { attempt: 4 }), 20)]));
    assert_eq!(answer.through, None);
    assert_eq!(
        puts(&answer),
        [Stored::Attempt(AttemptRow {
            task: 8,
            attempt: 4,
            claimed: Some(Wall::from_nanos(20)),
            started: None,
            ended: None,
            end: None,
            phases: None
        })]
    );
}
#[test]
fn room_cuts_at_an_attempt_free_milestone_before_the_furthest_fitting_row() {
    let rows = vec![
        row(7, 1, Phase::Active(Active::Idle), 10),
        row(7, 2, Phase::Active(Active::Claimed { attempt: 1 }), 11),
        row(7, 3, Phase::Active(Active::Running { attempt: 1 }), 12),
        row(7, 4, Phase::Active(Active::Running { attempt: 1 }), 13),
    ];
    let full = fold(&page(rows));
    for room in [
        JobRoom { writes: 4, bytes: u64::MAX },
        JobRoom { writes: u32::MAX, bytes: full.writes.iter().map(JobWrite::bytes).sum::<u64>() - 1 },
    ] {
        let mut page = page(vec![
            row(7, 1, Phase::Active(Active::Idle), 10),
            row(7, 2, Phase::Active(Active::Claimed { attempt: 1 }), 11),
            row(7, 3, Phase::Active(Active::Running { attempt: 1 }), 12),
            row(7, 4, Phase::Active(Active::Running { attempt: 1 }), 13),
        ]);
        page.room = room;
        let answer = fold(&page);
        assert_eq!(answer.through, Some(page.rows[0].key.clone()));
        assert_eq!(erases(&answer), [page.rows[0].key.clone()]);
        assert_eq!(puts(&answer), []);
        let next = fold(&self::page(page.rows.into_vec().into_iter().skip(1).collect()));
        assert_eq!(
            puts(&next),
            [Stored::Attempt(AttemptRow {
                task: 7,
                attempt: 1,
                claimed: Some(Wall::from_nanos(11)),
                started: Some(Wall::from_nanos(12)),
                ended: None,
                end: None,
                phases: None
            })]
        );
    }
}
#[test]
fn without_an_attempt_free_cut_room_consumes_what_fits_and_reputs_the_attempt() {
    let rows = vec![
        row(7, 1, Phase::Active(Active::Claimed { attempt: 1 }), 10),
        row(7, 2, Phase::Active(Active::Running { attempt: 1 }), 11),
    ];
    let mut page = page(rows);
    page.room.writes = 2;
    let answer = fold(&page);
    assert_eq!(answer.through, Some(page.rows[0].key.clone()));
    assert_eq!(erases(&answer).len(), 1);
    let first = Stored::Attempt(AttemptRow {
        task: 7,
        attempt: 1,
        claimed: Some(Wall::from_nanos(10)),
        started: None,
        ended: None,
        end: None,
        phases: None,
    });
    assert_eq!(puts(&answer), [first]);
    let next = fold(&self::page(page.rows.into_vec().into_iter().skip(1).collect()));
    assert_eq!(
        puts(&next),
        [Stored::Attempt(AttemptRow {
            task: 7,
            attempt: 1,
            claimed: Some(Wall::from_nanos(11)),
            started: Some(Wall::from_nanos(11)),
            ended: None,
            end: None,
            phases: None
        })]
    );
}
#[test]
fn a_non_milestone_or_undecodable_row_refuses_naming_its_key_even_in_the_left_task() {
    let other = codec::tasks::encode(&idle_hold(), &tasks_v13::CEILINGS, &[]).unwrap();
    for value in [Box::from([]), Box::from([0, 13, 255]), other] {
        let mut rows = ended_rows();
        let mut invalid = row(8, 1, Phase::Waiting, 20);
        invalid.value = value;
        let key = invalid.key.clone();
        rows.push(invalid);
        let refusal = jobs::page("milestones", &page(rows)).unwrap_err();
        assert_eq!(refusal.key, Some(key));
        assert!(refusal.why.contains("milestone"));
    }
}
#[test]
fn the_empty_end_page_has_no_writes_even_without_room() {
    let page = JobPage { rows: Box::new([]), end: true, layout: None, room: JobRoom { writes: 0, bytes: 0 } };
    let answer = fold(&page);
    assert!(answer.writes.is_empty());
    assert_eq!(answer.through, None);
}
