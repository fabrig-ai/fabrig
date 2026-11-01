//! Project release erases the connector's rows in one decision (contracts J5).

use std::collections::BTreeSet;

use jig_test_connector::{
    Effect, EffectPhase, Event, OutboxEntry, ProcedureState, Record, RecordKey, Request, ResourceRole,
};
use jig_test_connector_world::{World, kind, path};
use skein_lib::Token;

const SEED: u8 = 45;
const KEPT_PROJECT: u32 = 1;
const DELETED_PROJECT: u32 = 2;
const KEPT_TASK: u64 = 11;
const DELETED_TASK: u64 = 22;
const KEPT_PROPOSAL: u64 = 31;
const DELETED_PROPOSAL: u64 = 32;
const RETAINED_TASKS: u32 = 1; // Only KEPT_TASK belongs to the surviving project.

fn effect() -> Effect {
    Effect {
        kind: kind(SEED, 1),
        resources: Box::new([path(SEED, 1)]),
        purpose: 1,
        condition: None,
        target: 7,
        state: 7,
    }
}

fn restore(world: &mut World, record: Record) {
    world.records.insert(record.key(), record.clone());
    world.event(Event::Restore { record });
}

fn project(world: &mut World, project: u32, task: u64, proposal: u64) {
    world.event(Event::Adopt { project, resource: path(SEED, 1), role: ResourceRole::Owned });
    world.event(Event::Adopt { project, resource: path(SEED, 2), role: ResourceRole::Context });
    world.event(Event::Names { task, project, resources: Box::new([path(SEED, 1)]) });
    world.event(Event::Subscribe { task, topic: kind(SEED, 1), wake_at: 2, keep_at: 1 });
    let token = Token::new(proposal);
    world.event(Event::Describe { token, effect: effect() });
    world.event(Event::KeepProposal { token, number: proposal, task, project });
    for record in [
        Record::Procedure(ProcedureState {
            project,
            task,
            number: kind(SEED, 1),
            resource: path(SEED, 1),
            index: 0,
            steps: 0,
            awaiting: true,
            deadline: None,
            delegate: None,
            revision: 0,
        }),
        Record::ProcedureDelegateOutcome { task, step: 1, child: None, revisions: Box::new([]) },
        Record::Result { task, code: 9 },
        Record::Outbox(OutboxEntry {
            number: task,
            task,
            key: world.key(task, 1),
            effect: effect(),
            attempt: None,
            phase: EffectPhase::Kept,
        }),
        Record::Made { key: world.key(task, 2), resources: Box::new([path(SEED, 1)]), state: 7 },
    ] {
        restore(world, record);
    }
}

#[test]
fn a_released_project_leaves_no_connector_row() {
    let mut world = World::new(SEED);
    project(&mut world, KEPT_PROJECT, KEPT_TASK, KEPT_PROPOSAL);
    restore(&mut world, Record::Pool { path: path(SEED, 2), slots: 2 });
    restore(
        &mut world,
        Record::ApplicationObject { object: 1, now: jig_test_connector::ApplicationObject { revision: 1, value: 5 } },
    );
    restore(
        &mut world,
        Record::ApplicationAnswer {
            token: Token::new(1),
            until: skein_lib::Wall::from_nanos(100),
            outcome: jig_test_connector::ApplicationOutcome::Unknown,
        },
    );
    let kept = world.records.clone();
    project(&mut world, DELETED_PROJECT, DELETED_TASK, DELETED_PROPOSAL);

    let released = world.event(Event::Release { project: DELETED_PROJECT });
    let erased: BTreeSet<_> = released
        .iter()
        .filter_map(|request| match request {
            Request::Erase { key } => Some(key.clone()),
            Request::Released { project: DELETED_PROJECT } => None,
            other => panic!("project release only erases and acknowledges: {other:?}"),
        })
        .collect();
    assert_eq!(
        erased,
        BTreeSet::from([
            RecordKey::Adoption { project: DELETED_PROJECT, path: path(SEED, 1) },
            RecordKey::Adoption { project: DELETED_PROJECT, path: path(SEED, 2) },
            RecordKey::Task(DELETED_TASK),
            RecordKey::Subscription { topic: kind(SEED, 1), task: DELETED_TASK },
            RecordKey::Proposal(DELETED_PROPOSAL),
            RecordKey::Procedure(DELETED_TASK),
            RecordKey::ProcedureDelegateOutcome(DELETED_TASK),
            RecordKey::Result(DELETED_TASK),
            RecordKey::Outbox(world.key(DELETED_TASK, 1)),
            RecordKey::Made(world.key(DELETED_TASK, 2)),
        ])
    );
    assert_eq!(released.last(), Some(&Request::Released { project: DELETED_PROJECT }));
    assert_eq!(world.records, kept);
    assert_released(&mut world);
    world.cold_restart();
    assert_eq!(world.records, kept);
    assert_released(&mut world);
}

fn assert_released(world: &mut World) {
    assert_eq!(world.domain.resource_role(DELETED_PROJECT, &path(SEED, 1)), None);
    assert_eq!(world.domain.resource_role(DELETED_PROJECT, &path(SEED, 2)), None);
    assert_eq!(world.domain.resource_role(KEPT_PROJECT, &path(SEED, 1)), Some(ResourceRole::Owned));
    assert_eq!(world.domain.live_tasks(), RETAINED_TASKS);
    assert!(world.domain.subscribed_topics(DELETED_TASK).is_empty());
    let token = Token::new(DELETED_PROPOSAL);
    assert_eq!(
        world.event(Event::DescribeProposal { token, number: DELETED_PROPOSAL }),
        [Request::EffectRefused { token }]
    );
    assert_eq!(
        world.event(Event::Names {
            task: DELETED_TASK,
            project: DELETED_PROJECT,
            resources: Box::new([path(SEED, 1)]),
        }),
        [Request::Unknown { task: DELETED_TASK, resource: path(SEED, 1) }]
    );
    assert_eq!(
        world.event(Event::Release { project: DELETED_PROJECT }),
        [Request::Released { project: DELETED_PROJECT }]
    );
}

#[test]
fn releasing_a_project_without_rows_answers_at_once() {
    let mut world = World::new(SEED);
    for _ in 0..2 {
        assert_eq!(
            world.event(Event::Release { project: DELETED_PROJECT }),
            [Request::Released { project: DELETED_PROJECT }]
        );
    }
    assert!(world.records.is_empty());
    assert_eq!(world.commits, 0);
}

#[test]
#[ignore = "pd-test-connector: task-only records need a decision on project ownership after Unnamed"]
fn a_released_project_erases_rows_of_an_unnamed_task() {
    let mut world = World::new(SEED);
    world.event(Event::Adopt { project: DELETED_PROJECT, resource: path(SEED, 1), role: ResourceRole::Owned });
    world.event(Event::Names { task: DELETED_TASK, project: DELETED_PROJECT, resources: Box::new([path(SEED, 1)]) });
    world.event(Event::Subscribe { task: DELETED_TASK, topic: kind(SEED, 1), wake_at: 2, keep_at: 1 });
    let token = Token::new(DELETED_TASK);
    world.event(Event::Describe { token, effect: effect() });
    let kept =
        world.event(Event::Keep { token, entry: DELETED_TASK, task: DELETED_TASK, key: world.key(DELETED_TASK, 1) });
    world.release(&kept, &mut std::collections::VecDeque::new());
    world.event(Event::Unnamed { task: DELETED_TASK });
    world.cold_restart();
    world.event(Event::Release { project: DELETED_PROJECT });
    assert!(world.records.is_empty(), "released task rows remain: {:?}", world.records);
}
