//! The writer's atomic decision and restart state (landing-history.md, 3.1 and 4.4).
use super::*;
use crate::history::{
    ChangeHistory, ChangeIndex, Event as HistoryEvent, HOUR_NS, InLanding, LandingIndex, Last, MARKS, Mark,
    Operation as HistoryOperation, Project, Step, What as HistoryWhat,
};

fn input() -> Step {
    Step {
        what: HistoryWhat::Ready,
        landing: None,
        head: Some([3; 32]),
        base: Some([4; 32]),
        operation: HistoryOperation::Own,
        attempt: 9,
        source_at: Some(Wall::from_nanos(17)),
    }
}
fn saved(record: Stored) -> Request {
    Request::Save { record, release: None, read_afresh: None }
}
fn rows(mut out: Queue<Request>) -> Box<[Request]> {
    let mut rows = skein_lib::List::with_capacity(out.len());
    while let Some(request) = out.pop() {
        rows.push(request).unwrap();
    }
    rows.into_boxed()
}
fn setup() -> (Domain, u64) {
    let mut d = domain();
    let row = fixtures::opening_row();
    let change = row.task;
    assert!(outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) }).is_empty());
    assert!(outputs(&mut d, Event::Restore { record: Stored::Change(Box::new(row)) }).is_empty());
    (d, change)
}
#[test]
fn events_save_all_indexes_and_move_last_in_the_callers_decision() {
    let (mut d, change) = setup();
    let wall = Wall::from_nanos(HOUR_NS + 25);
    let mut out = Queue::uncapped();
    assert_eq!(d.write_event(wall, change, input(), &mut out), Some(1));
    let first = rows(out);
    let event = HistoryEvent {
        project: 5,
        number: 1,
        at: wall,
        source_at: Some(Wall::from_nanos(17)),
        change,
        repository: REPO,
        landing: None,
        head: Some([3; 32]),
        base: Some([4; 32]),
        operation: 1,
        attempt: 0,
        what: HistoryWhat::Ready,
    };
    let project = Project {
        project: 5,
        next: 2,
        kept: 1,
        whole_since: None,
        marks: Box::new([Mark { hour: Wall::from_nanos(HOUR_NS), first: 1 }]),
        sweeping: None,
        removing: false,
    };
    let mut row = fixtures::opening_row();
    row.history =
        Some(ChangeHistory { last: 1, ready: None, queued: None, paused: None, held: None, landing_held: None });
    let mut saves = skein_lib::List::with_capacity(5);
    for request in first {
        if matches!(request, Request::Save { .. } | Request::Erase { .. }) {
            saves.push(request).unwrap();
        }
    }
    let saves = saves.into_boxed();
    assert_eq!(
        saves.as_ref(),
        [
            saved(Stored::HistoryEvent(event.clone())),
            saved(Stored::HistoryChange(ChangeIndex { project: 5, change, number: 1, landing: None })),
            saved(Stored::HistoryLast(Last { project: 5, number: 1, change, at: wall })),
            saved(Stored::HistoryProject(project.clone())),
            saved(Stored::Change(Box::new(row.clone()))),
        ]
    );
    assert_eq!(d.changes.get(&change), Some(&row));
    assert_eq!(d.history_projects.get(&5), Some(&project));
    let mut second = input();
    second.landing = Some(InLanding { task: 91, round: 2, unit: 3 });
    second.operation = HistoryOperation::Of(91);
    second.attempt = 2;
    let mut out = Queue::uncapped();
    assert_eq!(d.write_event(wall, change, second.clone(), &mut out), Some(2));
    let second_rows = rows(out);
    assert!(second_rows.contains(&Request::Erase { key: Key::HistoryLast { project: 5, number: 1, change } }));
    assert!(second_rows.contains(&saved(Stored::HistoryLast(Last { project: 5, number: 2, change, at: wall }))));
    assert!(second_rows.contains(&saved(Stored::HistoryLanding(LandingIndex {
        project: 5,
        landing: 91,
        number: 2,
        change
    }))));
    assert!(second_rows.contains(&saved(Stored::HistoryChange(ChangeIndex {
        project: 5,
        change,
        number: 2,
        landing: Some(91)
    }))));
    assert!(second_rows.contains(&saved(Stored::HistoryEvent(HistoryEvent {
        number: 2,
        landing: second.landing,
        operation: 91,
        attempt: 2,
        ..event
    }))));
    assert_eq!(d.history_projects.get(&5).unwrap().marks.len(), 1, "same hour keeps its first event");
    assert_eq!(d.history_projects.get(&5).unwrap().kept, 2);
    let later = Wall::from_nanos(2 * HOUR_NS + 1);
    let mut out = Queue::uncapped();
    assert_eq!(d.write_event(later, change, input(), &mut out), Some(3));
    assert_eq!(
        d.history_projects.get(&5).unwrap().marks.as_ref(),
        [Mark { hour: Wall::from_nanos(HOUR_NS), first: 1 }, Mark { hour: Wall::from_nanos(2 * HOUR_NS), first: 3 }]
    );
    let mut empty = domain();
    let mut out = Queue::uncapped();
    assert_eq!(empty.write_event(wall, change, input(), &mut out), None);
    assert!(out.is_empty());
    assert!(empty.history_projects.is_empty());
}

#[test]
fn restored_history_keeps_intervals_numbers_and_only_the_newest_hour_marks() {
    let (mut d, change) = setup();
    let intervals = ChangeHistory {
        last: 40,
        ready: Some(2),
        queued: Some(3),
        paused: Some(4),
        held: Some(5),
        landing_held: Some(6),
    };
    let mut change_row = fixtures::opening_row();
    change_row.history = Some(intervals);
    let project = Project {
        project: 5,
        next: 41,
        kept: 12,
        whole_since: Some(Wall::from_nanos(8)),
        marks: marks(),
        sweeping: Some(72),
        removing: true,
    };
    assert!(outputs(&mut d, Event::Restore { record: Stored::HistoryProject(project.clone()) }).is_empty());
    assert!(outputs(&mut d, Event::Restore { record: Stored::Change(Box::new(change_row.clone())) }).is_empty());
    assert_eq!(d.history_projects.get(&5), Some(&project));
    assert_eq!(d.changes.get(&change), Some(&change_row));
    let wall = Wall::from_nanos(u64::from(MARKS) * HOUR_NS + 7);
    let mut out = Queue::uncapped();
    assert_eq!(d.write_event(wall, change, input(), &mut out), Some(41));
    let output = rows(out);
    let mut after = domain();
    for request in &output {
        if let Request::Save { record, .. } = request {
            assert!(outputs(&mut after, Event::Restore { record: record.clone() }).is_empty());
        }
    }
    let saved_project = d.history_projects.get(&5).unwrap();
    assert_eq!(saved_project.next, 42);
    assert_eq!(saved_project.kept, 13);
    assert_eq!(saved_project.whole_since, project.whole_since);
    assert_eq!(saved_project.sweeping, Some(72));
    assert!(saved_project.removing);
    assert_eq!(saved_project.marks.len(), usize::try_from(MARKS).unwrap());
    assert_eq!(saved_project.marks.first(), Some(&Mark { hour: Wall::from_nanos(HOUR_NS), first: 2 }));
    assert_eq!(
        saved_project.marks.last(),
        Some(&Mark { hour: Wall::from_nanos(u64::from(MARKS) * HOUR_NS), first: 41 })
    );
    assert_eq!(after.history_projects.get(&5), Some(saved_project));
    assert_eq!(after.changes.get(&change).unwrap().history, Some(ChangeHistory { last: 41, ..intervals }));
    let mut saved = Queue::uncapped();
    domain::save_change(&mut after, &mut saved, d.changes.get(&change).unwrap().clone());
    assert!(rows(saved).contains(&saved_row(&d, change)));
    assert_eq!(after.history_projects.len(), 1, "event and index restores are dropped");
}
fn saved_row(d: &Domain, change: u64) -> Request {
    saved(Stored::Change(Box::new(d.changes.get(&change).unwrap().clone())))
}

#[test]
fn a_restored_history_deadline_advances_when_the_fresh_read_stage_can_fire() {
    let mut d = domain();
    let env = Env { now: Time::from_nanos(10_000_000_000), wall: Wall::from_nanos(10_000_000_000), ..env() };
    d.restore_begin(env.wall);
    let mut restored = Queue::uncapped();
    step(
        &mut d,
        &env,
        Event::Restore {
            record: Stored::HistoryProject(Project {
                project: 5,
                next: 2,
                kept: 1,
                whole_since: None,
                marks: Box::new([]),
                sweeping: None,
                removing: false,
            }),
        },
        &mut restored,
    );
    assert!(restored.is_empty());
    assert_eq!(crate::history::sweep::deadline(&d), Some(Wall::EPOCH), "the sweep has never fired");
    assert_eq!(d.next_wall_deadline(), None, "restoration cannot fire policy timers");
    let mut out = Queue::uncapped();
    fire(&mut d, &env, &mut out);
    assert!(out.is_empty());
    step(&mut d, &env, Event::Restored { clock: skein_forge_client::RecoveryClock::Wall }, &mut restored);
    step(&mut d, &env, Event::ReadAfresh, &mut restored);
    assert_eq!(d.restore_fit_failure(), None);
    assert_eq!(d.next_wall_deadline(), Some(Wall::EPOCH));
    fire(&mut d, &env, &mut out);
    if let Some(at) = d.next_wall_deadline() {
        assert!(at > env.wall, "a fired sweep cannot stay due");
    }
    let mut asked = false;
    for request in rows(out) {
        if let Request::LoadHistory { range: HistoryRange::Last { project: 5 }, most: 1, .. } = request {
            asked = true;
        }
    }
    assert!(asked, "the restored project begins its sweep");
}

fn marks() -> Box<[Mark]> {
    let mut marks = skein_lib::List::with_capacity(MARKS);
    for hour in 0..MARKS {
        marks
            .push(Mark {
                hour: Wall::from_nanos(u64::from(hour).checked_mul(HOUR_NS).unwrap()),
                first: u64::from(hour).checked_add(1).unwrap(),
            })
            .unwrap();
    }
    marks.into_boxed()
}
