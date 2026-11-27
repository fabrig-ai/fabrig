//! Adoption fits and opening use the same state (domain/engine/restart.md, domain/engine/fleet.md).
use crate::{
    CoreRecord, Event, RunProof, fleet, people,
    start::{Continuation, LiveSet},
    tests::start::{
        Action, Core, Env, Failure, Key, Phase, Queue, Request, SweepInput, Time, Wall, config, step_phase, tasks,
        through_link,
    },
};
use jig_domain_link::{Form, Scheme};
use skein_lib::{List, Token};

const WORKER: u64 = 2;
const ENGINE: u64 = 1;
const DUE: u64 = 3;
const ATTEMPT: u64 = 7;

pub(in crate::tests) fn loaded(in_process: bool, attempts: u32) -> (Core, Env<crate::Limits>) {
    loaded_with_writes(in_process, attempts, &[])
}

fn loaded_with_writes(in_process: bool, attempts: u32, saved: &[crate::Record]) -> (Core, Env<crate::Limits>) {
    let (mut source, mut limits) = crate::tests::tree_core();
    for number in [WORKER, ENGINE, DUE] {
        crate::tests::make_task(&mut source, &limits, number);
    }
    limits.fleet.attempts = attempts;
    let mut configured = config(41);
    configured.workers_in_process = in_process;
    let policy = crate::admission::policy::snapshot(&configured.seed_projects[0].policy.policy, &[], &[])
        .expect("project policy");
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
    let mut image = List::with_capacity(16);
    for (_, ledger) in source.tasks.funding_records() {
        image.push(crate::Record::Tasks(tasks::Stored::Ledger(*ledger))).expect("fixture ledgers");
    }
    for (number, host) in
        [(WORKER, Some(fleet::HostKind::Worker)), (ENGINE, Some(fleet::HostKind::Engine)), (DUE, None)]
    {
        let mut row = source.tasks.task(number).expect("source task").clone();
        if let Some(host) = host {
            row.phase = tasks::Phase::Active(tasks::Active::Claimed { attempt: ATTEMPT });
            row.attempt = ATTEMPT;
            row.settings.run = Some(row.settings.task);
            image
                .push(crate::Record::Core(CoreRecord::RunProof(RunProof::claimed(
                    number, ATTEMPT, ATTEMPT, None, host,
                ))))
                .expect("fixture claims");
        }
        image.push(crate::Record::Tasks(tasks::Stored::Live(Box::new(row)))).expect("fixture live tasks");
    }
    for record in saved {
        let mut replaced = false;
        for row in image.iter_mut() {
            if row.key() == record.key() {
                row.clone_from(record);
                replaced = true;
                break;
            }
        }
        if !replaced {
            image.push(record.clone()).expect("fixture committed records");
        }
    }
    for record in image.into_boxed() {
        if let crate::Record::Tasks(row) = record {
            restore(&mut core, &env, row);
        } else if let crate::Record::Core(CoreRecord::RunProof(proof)) = record {
            assert!(core.restore_proof(proof, limits.tasks.result_bytes));
        }
    }
    crate::tests::restore_key_account(&mut core, &limits, 0, 1, 1);
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

pub(in crate::tests) fn wires(core: &mut Core, env: &Env<crate::Limits>, out: &mut Queue<Request>) -> Continuation {
    through_link(core, env, out);
    step_phase(core.start_sweep(env, SweepInput::Finished, out), Phase::Switch);
    step_phase(core.start_done(env, Phase::Switch, out), Phase::Wires);
    let Action::Commit(token) = core.start_done(env, Phase::Wires, out) else { panic!("durable Wires") };
    assert!(core.adopted.is_empty());
    assert!(!core.start_opened());
    token
}

fn open(core: &mut Core, env: &Env<crate::Limits>, out: &mut Queue<Request>) {
    let mut action = Action::Step(core.start_phase().expect("start in progress"));
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
        assert_eq!(core.restored_worker_claims(), if in_process { None } else { Some((WORKER, 1)) });
        let _action = core.start_done(&env, Phase::AdoptRuns, &mut out);
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
    let Action::Refused(found) = core.start_resume(&env, token, &mut out) else {
        panic!("worker claims refuse as a fit")
    };
    assert_eq!(found, expected);
    assert_eq!(core.start_failure(), Some(expected));
    assert_eq!(core.start_phase(), Some(Phase::AdoptRuns));
    assert_eq!(core.adopted.len(), 2);
    assert!(!core.start_opened());
}

#[test]
fn a_worker_claim_commit_fits_the_next_start_before_its_fleet_handoff() {
    for attempts in [2, 3] {
        let (mut core, env) = loaded(false, attempts);
        core.proofs.get_mut(&ENGINE).expect("second worker proof").host = fleet::HostKind::Worker;
        let mut out = Queue::uncapped();
        let token = wires(&mut core, &env, &mut out);
        step_phase(core.start_resume(&env, token, &mut out), Phase::AdoptRuns);
        let _action = core.start_done(&env, Phase::AdoptRuns, &mut out);
        open(&mut core, &env, &mut out);

        // The root has prepared another claim, but has not handed it to the fleet.
        crate::tests::send_task(
            &mut core,
            &env.limits,
            tasks::Event::Prepare { reply_to: skein_lib::ReplyTo::new(Token::new(DUE)), task: DUE },
        );
        core.proofs
            .insert(DUE, RunProof::claimed(DUE, ATTEMPT, ATTEMPT, None, fleet::HostKind::Worker))
            .expect("pending proof");
        core.claiming.insert(DUE, (ATTEMPT, tasks::Reasons::ALL, None, None)).expect("pending claim");
        let mut writes = crate::step(
            &mut core,
            &env,
            Event::ClaimPrepared { task: DUE, attempt: ATTEMPT, budget: 1, writes: Box::new([]) },
        );
        assert_eq!(core.tasks.task(DUE).expect("prepared task").tries, tasks::Tries::NONE);

        // Apply the writer's records to a cold image, not its in-memory task state.
        let mut saved = List::with_capacity(writes.len());
        let mut saved_task = false;
        for _ in 0..writes.len() {
            if let Request::Write(crate::Write::Save(row)) = writes.pop().expect("writer output count") {
                if let crate::Record::Tasks(tasks::Stored::Live(live)) = &row {
                    saved_task |= live.number == DUE;
                }
                if let crate::Record::Tasks(tasks::Stored::Live(_) | tasks::Stored::Ledger(_))
                | crate::Record::Core(CoreRecord::RunProof(_)) = &row
                {
                    saved.push(row).expect("no more records than outputs");
                }
            }
        }
        assert!(saved_task, "the claim or its capacity refusal is durable");
        let (mut cold, cold_env) = loaded_with_writes(false, attempts, saved.as_slice());
        cold.proofs.get_mut(&ENGINE).expect("second worker proof").host = fleet::HostKind::Worker;
        let mut out = Queue::uncapped();
        let token = wires(&mut cold, &cold_env, &mut out);
        let resumed = cold.start_resume(&cold_env, token, &mut out);
        let Action::Step(Phase::AdoptRuns) = resumed else {
            panic!("writer's commit must reopen: {resumed:?}");
        };
        assert_eq!(cold.restored_worker_claims().expect("worker claims").1, u64::from(attempts));
        let _action = cold.start_done(&cold_env, Phase::AdoptRuns, &mut out);
        open(&mut cold, &cold_env, &mut out);
        assert_eq!(
            tasks::run_attempt(&cold.tasks.task(DUE).expect("restored task").phase),
            if attempts == 3 { Some(ATTEMPT) } else { None }
        );
    }
}

#[test]
fn a_cold_start_repeats_adoption() {
    let mut out = Queue::uncapped();
    {
        let (mut core, env) = loaded(false, 1);
        let _unreleased = wires(&mut core, &env, &mut out);
    }
    let (mut core, env) = loaded(false, 1);
    let token = wires(&mut core, &env, &mut out);
    step_phase(core.start_resume(&env, token, &mut out), Phase::AdoptRuns);
    assert_eq!(core.adopted.len(), 2);
    for (event, expected) in core
        .adopted
        .iter()
        .zip([(ENGINE, ATTEMPT, fleet::HostKind::Engine), (WORKER, ATTEMPT, fleet::HostKind::Worker)])
    {
        let fleet::Event::Adopt { run, attempt, kind, .. } = event else { panic!("restored adoption") };
        assert_eq!((run.raw(), attempt.raw(), *kind), expected);
    }
    let _action = core.start_done(&env, Phase::AdoptRuns, &mut out);
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
            form: Form::Credential(Scheme::Bearer),
            account: 1,
            generation: 1,
            valid: skein_lib::Duration::from_secs(60),
        })),
    ));
    crate::tests::runs::run_choice::host(&mut core, &env.limits, Token::new(7), &[b"scripted"]);
    let context = core.tasks.run_context(DUE).expect("due agent");
    let before = crate::step(&mut core, &env, Event::Activate { context, ready: true });
    assert!(before.is_empty());
    let _action = core.start_done(&env, Phase::AdoptRuns, &mut out);
    open(&mut core, &env, &mut out);
    let context = core.take_due().expect("next Resume takes due agent");
    assert_eq!(context.task, DUE);
    let after = crate::step(&mut core, &env, Event::Activate { context, ready: true });
    assert_eq!(core.tasks.task(DUE).unwrap().phase, tasks::Phase::Active(tasks::Active::Preparing));
    assert!(core.contexts.contains_key(&DUE));
    assert!(!after.is_empty(), "due agent begins its brief");
}
