//! Kept deletion advances and restarts at each stage (draft 5).
#![expect(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::wildcard_enum_match_arm,
    clippy::match_like_matches_macro,
    reason = "ordinary Rust unit scripts select the deletion outputs under assertion"
)]
use super::*;
use crate::reads::Sweep;
use crate::routing::removal;

const PROJECT: u32 = 1;
const OTHER_PROJECT: u32 = 2;
const TASK: u64 = 1;
const LAST_TASK: u64 = 2;
const CONNECTORS: [u16; 2] = [1, 2];

fn fixture(stage: RemovalStage) -> (Core, Env<Limits>) {
    let (mut core, limits) = core();
    core.removals.insert(PROJECT, Removal { project: PROJECT, stage }).unwrap();
    (core, Env { now: Time::ZERO, wall: Wall::EPOCH, limits })
}

fn saved(out: &Queue<Request>, stage: RemovalStage) -> bool {
    out.iter().any(|request| match request {
        Request::Write(Write::Save(Record::Core(CoreRecord::Removal(row)))) => {
            row == &Removal { project: PROJECT, stage }
        }
        _ => false,
    })
}

fn swept_from(out: &Queue<Request>, after: u64) -> bool {
    out.iter().any(|request| match request {
        Request::Held(held) => match held.as_ref() {
            Held::Sweep { sweep: Sweep::Ended { after: cursor }, .. } => *cursor == after,
            _ => false,
        },
        _ => false,
    })
}

fn releases(out: &Queue<Request>) -> Vec<(u16, Token)> {
    out.iter()
        .filter_map(|request| match request {
            Request::Ask {
                connector,
                ask: Ask::Effect(connector::Ask::ReleaseProject { request, project: PROJECT }),
            } => Some((*connector, *request)),
            _ => None,
        })
        .collect()
}

#[test]
fn the_last_task_ending_moves_the_row_to_erasing() {
    let (mut core, limits) = tree_core();
    core.removals.insert(PROJECT, Removal { project: PROJECT, stage: RemovalStage::Ending }).unwrap();
    let row = new_row(&core, LAST_TASK, tasks::Funder::Period { project: PROJECT, period: 1 }, None);
    fixtures::make_batch(&mut core, &limits, tasks::Party::Person(1), Box::new([row]));
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let mut work = Queue::uncapped();
    let mut out = Queue::uncapped();
    removal::task_ended(&mut core, &env, &mut work, &mut out, PROJECT);
    assert!(out.is_empty());
    removal::resume(&mut core, &env, &mut work, &mut out);
    let mut out = Queue::uncapped();
    for connector in CONNECTORS {
        let Requests::Out(mut closed) =
            step(&mut core, &env, Event::EffectConnector(connector::Event::Closed { task: LAST_TASK, connector }));
        while let Some(request) = closed.pop() {
            out.push(request);
        }
    }
    let Requests::Out(out) = step(&mut core, &env, Event::Tasks(tasks::Event::Settled { task: LAST_TASK }));
    assert!(saved(&out, RemovalStage::Erasing));
    assert_eq!(core.removals.get(&PROJECT).unwrap().stage, RemovalStage::Erasing);
    assert!(swept_from(&out, 0));
}

#[test]
fn releasing_waits_for_every_connector_then_erases_the_row() {
    let (mut core, env) = fixture(RemovalStage::Erasing);
    core.connectors = Box::new(CONNECTORS);
    let mut work = Queue::uncapped();
    let mut out = Queue::uncapped();
    removal::rows_erased(&mut core, &env, &mut work, &mut out, PROJECT);
    assert!(saved(&out, RemovalStage::Releasing));
    let asks = releases(&out);
    assert_eq!(asks.iter().map(|ask| ask.0).collect::<Vec<_>>(), CONNECTORS);
    assert_ne!(asks[0].1, asks[1].1);
    out.clear();
    removal::released(&mut core, &mut work, &mut out, asks[0].1, OTHER_PROJECT);
    removal::released(&mut core, &mut work, &mut out, Token::new(u64::MAX), PROJECT);
    removal::released(&mut core, &mut work, &mut out, asks[0].1, PROJECT);
    removal::released(&mut core, &mut work, &mut out, asks[0].1, PROJECT);
    assert!(core.removals.contains_key(&PROJECT));
    assert!(out.is_empty());
    removal::released(&mut core, &mut work, &mut out, asks[1].1, PROJECT);
    assert!(!core.removals.contains_key(&PROJECT));
    assert!(out.iter().any(|request| match request {
        Request::Write(Write::Erase(Key::Core(CoreKey::Removal(PROJECT)))) => true,
        _ => false,
    }));
    assert!(work.iter().any(|event| match event {
        Event::People(people::Event::ProjectGone { project: PROJECT }) => true,
        _ => false,
    }));
}

#[test]
fn a_persisted_releasing_row_resends_at_open() {
    let (mut core, env) = fixture(RemovalStage::Releasing);
    let row = core.removals.remove(&PROJECT).unwrap();
    assert_eq!(core.restore_core(CoreRecord::Removal(row), &env.limits), Restored::Live);
    core.start.mode = crate::start::StartMode::LoadAndLink;
    core.start.phase = Some(crate::start::Phase::Open);
    let mut out = Queue::uncapped();
    assert!(match core.start_done(&env, crate::start::Phase::Open, &mut out) {
        crate::start::Action::Idle => true,
        _ => false,
    });
    let asks = releases(&out);
    assert_eq!(asks.iter().map(|ask| ask.0).collect::<Vec<_>>(), CONNECTORS);
    let mut work = Queue::uncapped();
    for (_, token) in asks {
        removal::released(&mut core, &mut work, &mut out, token, PROJECT);
    }
    assert!(!core.removals.contains_key(&PROJECT));
}

#[test]
fn the_ended_sweep_skips_other_projects_and_erases_one_task_at_a_time() {
    let (mut core, env) = fixture(RemovalStage::Erasing);
    let mut work = Queue::uncapped();
    let mut out = Queue::uncapped();
    removal::resume(&mut core, &env, &mut work, &mut out);
    let owner = *core.removal_sweeps.iter().next().unwrap().0;
    let mut other = fixtures::ended_report(TASK, tasks::Party::Person(1));
    other.project = OTHER_PROJECT;
    let Requests::Out(out) = removal::swept(
        &mut core,
        &env,
        &mut work,
        owner,
        Box::new([Record::Tasks(tasks::Stored::Ended(Box::new(other)))]),
        true,
    );
    assert!(swept_from(&out, TASK));
    let rows = [LAST_TASK, LAST_TASK + 1].map(|task| {
        Record::Tasks(tasks::Stored::Ended(Box::new(fixtures::ended_report(task, tasks::Party::Person(1)))))
    });
    let Requests::Out(out) = removal::swept(&mut core, &env, &mut work, owner, Box::new(rows), false);
    assert_eq!(core.erasures.len(), 1);
    assert!(out.iter().any(|request| match request {
        Request::Write(Write::DropTranscript(LAST_TASK)) => true,
        _ => false,
    }));
}

fn legacy_open(stage: RemovalStage) -> (Core, Env<Limits>) {
    let limits = limits();
    let mut core = Core::new(config(41), &limits).expect("fixture core configuration");
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let mut next = core.restart_begin(false);
    let row = Removal { project: PROJECT, stage };
    assert_eq!(core.restore_core(CoreRecord::Removal(row), &limits), Restored::Live);
    for _ in 0_u32..16 {
        match next {
            RestartRequest::Step(done) => next = core.restart_done(done),
            RestartRequest::Idle => break,
            other => panic!("restart refused: {other:?}"),
        }
    }
    assert!(core.restart_ready());
    (core, env)
}

#[test]
fn a_persisted_erasing_row_resumes_from_the_first_task_after_legacy_open() {
    let (mut core, env) = legacy_open(RemovalStage::Erasing);
    assert!(core.removal_sweeps.is_empty());
    let Requests::Out(out) = resume_removals(&mut core, &env);
    assert_eq!(core.removal_sweeps.len(), 1);
    assert!(swept_from(&out, 0));
}

#[test]
fn a_cold_legacy_releasing_row_resends_every_connector_ask() {
    let (mut core, env) = legacy_open(RemovalStage::Releasing);
    let Requests::Out(out) = resume_removals(&mut core, &env);
    let asks = releases(&out);
    assert_eq!(asks.iter().map(|ask| ask.0).collect::<Vec<_>>(), CONNECTORS);
    let mut work = Queue::uncapped();
    let mut out = Queue::uncapped();
    for (_, token) in asks {
        removal::released(&mut core, &mut work, &mut out, token, PROJECT);
    }
    assert!(!core.removals.contains_key(&PROJECT));
}

#[test]
fn releasing_without_connectors_completes_and_an_empty_resume_routes_nothing() {
    let (mut core, env) = legacy_open(RemovalStage::Releasing);
    core.connectors = Box::new([]);
    let Requests::Out(out) = resume_removals(&mut core, &env);
    assert!(releases(&out).is_empty());
    assert!(!core.removals.contains_key(&PROJECT));
    assert!(out.iter().any(|request| match request {
        Request::Write(Write::Erase(Key::Core(CoreKey::Removal(PROJECT)))) => true,
        _ => false,
    }));
    let Requests::Out(out) = resume_removals(&mut core, &env);
    assert!(out.is_empty());
}
