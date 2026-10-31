//! Adoption fits and opening use the same state (domain/engine.md, 6, 8).
use super::{
    Action, Core, Env, Failure, Key, Phase, Queue, Request, SweepInput, Time, Wall, config, step_phase, tasks,
    through_link,
};
use crate::start::{Continuation, LiveSet};
use crate::{CoreRecord, Event, Now, RunProof, fleet, people};
use core::panic::AssertUnwindSafe;
use skein_lib::Token;
use std::panic::catch_unwind;

const WORKER: u64 = 2;
const ENGINE: u64 = 1;
const DUE: u64 = 3;
const ATTEMPT: u64 = 7;

fn loaded(in_process: bool, attempts: u32) -> (Core, Env<crate::Limits>) {
    let (mut source, mut limits) = super::super::tree_core();
    for number in [WORKER, ENGINE, DUE] {
        super::super::make_task(&mut source, &limits, number);
    }
    limits.fleet.attempts = attempts;
    let mut configured = config(41);
    configured.workers_in_process = in_process;
    let policy = crate::policy::snapshot(&configured.seed_projects[0].policy.policy, &[], &[]).expect("project policy");
    let mut core = Core::new(configured, &limits).expect("configured start");
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    step_phase(core.start_begin(false, env.wall), Phase::LoadCore);
    let mut header = core.counters.deployment();
    header.projects = 1;
    header.tasks = DUE;
    header.people = 1;
    header.runs = ATTEMPT;
    header.commits = 1;
    assert_eq!(core.restore_core(CoreRecord::Deployment(header), &limits), crate::Restored::Deployment { commits: 1 });
    assert!(core.restore_policy(&limits, 1, Box::from("Project 1"), policy));
    for record in [
        people::Stored::Person {
            number: 1,
            identity: people::Identity {
                key: people::IdentityKey { provider: 0, subject: Box::new([1]) },
                login: Box::new([]),
                name: Box::new([]),
            },
        },
        people::Stored::Roles {
            project: 1,
            holdings: Box::new([people::Holding { person: 1, role: people::Role::Owner }]),
        },
    ] {
        drop(crate::step(&mut core, &env, Event::People(people::Event::Restore { record })));
    }
    for (_, ledger) in source.tasks.funding_records() {
        restore(&mut core, &env, tasks::Stored::Ledger(*ledger));
    }
    for (number, host) in
        [(WORKER, Some(fleet::HostKind::Worker)), (ENGINE, Some(fleet::HostKind::Engine)), (DUE, None)]
    {
        let mut row = source.tasks.task(number).expect("source task").clone();
        if let Some(host) = host {
            row.phase = tasks::Phase::Active(tasks::Active::Claimed { attempt: ATTEMPT });
            row.attempt = ATTEMPT;
            row.settings.run = Some(row.settings.task);
            assert!(
                core.restore_proof(RunProof::claimed(number, ATTEMPT, ATTEMPT, None, host), limits.tasks.result_bytes)
            );
        }
        restore(&mut core, &env, tasks::Stored::Live(Box::new(row)));
    }
    super::super::restore_key_account(&mut core, &limits, 0, 1, 1);
    drop(crate::step(
        &mut core,
        &env,
        Event::Account(crate::accounts::Event::Restore { record: crate::accounts::Stored::Next { number: 2 } }),
    ));
    (core, env)
}

fn restore(core: &mut Core, env: &Env<crate::Limits>, record: tasks::Stored) {
    assert!(core.restore_task_row(&record));
    drop(crate::step(core, env, Event::Tasks(tasks::Event::Restore { record })));
}

fn wires(core: &mut Core, env: &Env<crate::Limits>, out: &mut Queue<Request>) -> Continuation {
    through_link(core, env, out);
    step_phase(core.start_sweep(env, SweepInput::Finished, out), Phase::Switch);
    step_phase(core.start_done(env, Phase::Switch, out), Phase::Wires);
    let Action::Commit(token) = core.start_done(env, Phase::Wires, out) else { panic!("durable Wires") };
    assert!(core.adopted.is_empty());
    assert!(!core.start_opened());
    token
}

fn drain(core: &mut Core, env: &Env<crate::Limits>) {
    while let Some(event) = core.adopted.pop() {
        drop(crate::step(core, env, Event::Fleet(event)));
    }
    drop(crate::step(core, env, Event::Fleet(fleet::Event::Loaded)));
}

fn open(core: &mut Core, env: &Env<crate::Limits>, out: &mut Queue<Request>) {
    let mut action = core.start_done(env, Phase::AdoptRuns, out);
    for _ in 0..8_u32 {
        action = match action {
            Action::Step(done) => core.start_done(env, done, out),
            Action::Idle => break,
            Action::Commit(_) | Action::Refused(_) => panic!("adopted start opens"),
        };
    }
    let Action::Idle = action else { panic!("opened start is idle") };
    assert!(core.start_opened());
    assert!(core.start_opened());
}

#[test]
fn restored_claims_adopt_and_engine_claims_settle_before_open() {
    for in_process in [false, true] {
        let (mut core, env) = loaded(in_process, 1);
        let mut out = Queue::uncapped();
        let token = wires(&mut core, &env, &mut out);
        step_phase(core.start_resume(&env, token, &mut out), Phase::AdoptRuns);
        assert_eq!(core.adopted.len(), 2);
        let _panic = catch_unwind(AssertUnwindSafe(|| core.start_done(&env, Phase::AdoptRuns, &mut out)))
            .expect_err("undrained claim stops adoption completion");
        assert_eq!(core.restored_worker_claims(), if in_process { None } else { Some((WORKER, 1)) });
        drain(&mut core, &env);
        assert_eq!(core.tasks.task(ENGINE).expect("engine task").refusals, 1);
        assert_eq!(core.unreported_restored.contains_key(&WORKER), !in_process);
        open(&mut core, &env, &mut out);
    }
}

#[test]
fn stored_worker_claims_past_attempts_refuse_before_any_are_delivered() {
    let (mut core, env) = loaded(false, 0);
    let mut out = Queue::uncapped();
    let token = wires(&mut core, &env, &mut out);
    let expected =
        Failure::LiveSet { set: LiveSet::WorkerClaims, key: Key::Tasks(tasks::Key::Live(WORKER)), held: 1, limit: 0 };
    let Action::Refused(found) = core.start_resume(&env, token.clone(), &mut out) else {
        panic!("worker claims refuse as a fit")
    };
    assert_eq!(found, expected);
    assert_eq!(core.start_failure(), Some(expected));
    assert_eq!(core.start_phase(), Some(Phase::AdoptRuns));
    assert_eq!(core.adopted.len(), 2);
    assert!(!core.start_opened());
    let _panic =
        catch_unwind(AssertUnwindSafe(|| core.start_resume(&env, token, &mut out))).expect_err("continuation is stale");
}

#[test]
fn a_released_continuation_is_stale_and_a_cold_start_repeats_adoption() {
    let mut out = Queue::uncapped();
    {
        let (mut core, env) = loaded(false, 1);
        let _unreleased = wires(&mut core, &env, &mut out);
    }
    let (mut core, env) = loaded(false, 1);
    let token = wires(&mut core, &env, &mut out);
    step_phase(core.start_resume(&env, token.clone(), &mut out), Phase::AdoptRuns);
    let _panic =
        catch_unwind(AssertUnwindSafe(|| core.start_resume(&env, token, &mut out))).expect_err("continuation is stale");
    assert_eq!(core.adopted.len(), 2);
    for (event, expected) in core
        .adopted
        .iter()
        .zip([(ENGINE, ATTEMPT, fleet::HostKind::Engine), (WORKER, ATTEMPT, fleet::HostKind::Worker)])
    {
        let fleet::Event::Adopt { run, attempt, kind, .. } = event else { panic!("restored adoption") };
        assert_eq!((run.raw(), attempt.raw(), *kind), expected);
    }
    drain(&mut core, &env);
    open(&mut core, &env, &mut out);
}

#[test]
fn due_agent_preparation_waits_until_open() {
    let (mut core, env) = loaded(false, 2);
    let mut out = Queue::uncapped();
    let token = wires(&mut core, &env, &mut out);
    step_phase(core.start_resume(&env, token, &mut out), Phase::AdoptRuns);
    drop(crate::step(
        &mut core,
        &env,
        Event::Account(crate::accounts::Event::Component(crate::accounts::FromComponent::Granted {
            form: crate::accounts::Form::Credential(crate::accounts::Scheme::Bearer),
            account: 1,
            generation: 1,
            valid: skein_lib::Duration::from_secs(60),
        })),
    ));
    super::super::run_choice::host(&mut core, &env.limits, Token::new(7), &[b"scripted"]);
    let context = core.tasks.run_context(DUE).expect("due agent");
    let before = crate::step(&mut core, &env, Event::Activate { context, ready: true });
    assert!(before.is_empty());

    drain(&mut core, &env);
    open(&mut core, &env, &mut out);
    let context = core.take_due().expect("next Resume takes due agent");
    assert_eq!(context.task, DUE);
    let mut after = crate::step(&mut core, &env, Event::Activate { context, ready: true });
    let Some(Request::Now(request)) = after.pop() else { panic!("agent preparation request") };
    let Now::PrepareAgent { context } = *request else { panic!("due agent begins preparation") };
    assert_eq!(context.task, DUE);
}
