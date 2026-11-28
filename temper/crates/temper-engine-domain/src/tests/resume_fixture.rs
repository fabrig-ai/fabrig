//! Unit configuration uses the existing engine world's tiny limits and charter,
//! with no templates. The empty startup retention pass is settled before runtime measurements;
//! no worker or connector driver is added.
use crate as engine;
use jig_domain_core::engine::store as engine_store;
use jig_domain_fleet as fleet;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{Env, Time, Wall};

pub use super::step_start_fixture::{StartStore, Until, config, config_with_project_room, limits};
pub use temper_engine_step_kit::start::{charter, models};

// Phases and durable continuations for one connector, followed by Idle.
const RESTORE_PASSES: u32 = 20;

pub fn root() -> (engine::Domain, Env<engine::Limits>) {
    let (mut root, env) = root_with_config(config(7));
    // Settle the startup placement probe before tests measure empty runtime resumes.
    let mut out = skein_lib::Queue::uncapped();
    fleet::resume(&mut root.engine.core_mut().fleet, &env.with(env.limits.fleet), true, &mut out);
    assert!(out.is_empty(), "the fixture restores no claims or workers");
    settle_retention(&mut root, &env);
    (root, env)
}

fn settle_retention(root: &mut engine::Domain, env: &Env<engine::Limits>) {
    use core::engine::{self as api, In, Out, Peer};
    use jig_domain_core as core;
    let env = crate::limits::environment_core(env);
    let mut requests = skein_lib::Queue::uncapped();
    api::fire(&mut root.engine, &env, core::Timer::Retention, &mut requests);
    for _ in 0..RESTORE_PASSES {
        let Some(output) = requests.pop() else { break };
        let Out::Held(Peer::Store(engine_store::Out::Load { load, range: engine_store::Range::Read(range) })) = output
        else {
            panic!("retention loads its due range")
        };
        let core::reads::Scope::Retention { .. } = range.scope else { panic!("the fixture has no retention owner") };
        assert!(requests.is_empty(), "one internal read at a time");
        api::step(
            &mut root.engine,
            &env,
            In::Store(engine_store::In::Loaded {
                load,
                result: engine_store::Answer::Rows(engine_store::Rows::Read { rows: Box::new([]), next: None }),
            }),
            &mut requests,
        );
        root.engine.core_mut().reclaim();
    }
    assert!(requests.is_empty(), "empty-store retention completed without a write");
}
pub fn root_with_config(config: engine::Config) -> (engine::Domain, Env<engine::Limits>) {
    root_with_config_and_limits(config, &limits())
}

pub fn root_with_config_and_limits(
    config: engine::Config,
    limits: &engine::Limits,
) -> (engine::Domain, Env<engine::Limits>) {
    root_with_records(config, limits, &[])
}

/// Build a runtime fixture without startup IO or a Wires commit. Tests of
/// startup itself drive `StartStore` through the ports instead.
pub fn root_with_records(
    config: engine::Config,
    limits: &engine::Limits,
    records: &[engine::Record],
) -> (engine::Domain, Env<engine::Limits>) {
    // Model a later start: the store supplies the complete project rows and
    // their deployment high-water mark, rather than configuration reseeding.
    let mut projects = skein_lib::List::with_capacity(limits.authority.projects);
    let mut project_mark = 0;
    for project in &config.core.seed_projects {
        project_mark = project_mark.max(u64::from(project.number));
        let value = jig_domain_core::policy_snapshot(
            &project.policy.policy,
            &project.policy.permissions,
            &project.policy.templates,
        )
        .expect("fixture policy snapshot");
        projects
            .push(people::Stored::Policy { project: project.number, name: project.name.clone(), value })
            .expect("bounded fixture projects");
    }
    let mut root = engine::Domain::new(config, limits).unwrap();
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: *limits };
    root.start_wall = Some(env.wall);
    let mut request = root.engine.core_mut().start_begin(false, env.wall);
    let mut deployment = root.engine.core().counters.deployment();
    deployment.projects = project_mark;
    for record in records {
        match record {
            engine::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row))) => {
                deployment.tasks = deployment.tasks.max(row.number);
            }
            engine::Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Projection(
                jig_domain_core::ProjectionRecord::GoalHot { goal, .. },
            ))) => deployment.tasks = deployment.tasks.max(goal.number),
            engine::Record::Core(_) | engine::Record::Forge { .. } => {}
        }
    }
    restore_fixture_row(
        &mut root,
        &env,
        engine::Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Deployment(deployment))),
    );
    for record in projects.into_boxed() {
        restore_fixture_row(&mut root, &env, engine::Record::Core(jig_domain_core::Record::People(record)));
    }
    let mut decision = engine::Decision::decide(&mut root.journal, &limits.journal);
    for record in records {
        if let engine::Record::Forge { .. } = record {
            root.forge.restore_begin(env.wall);
        }
        restore_fixture_row(&mut root, &env, record.clone());
        engine::route::route_into(&mut root, &env, &mut decision);
    }
    // This fixture discards the restoration route rather than simulating startup IO.
    if let Some(commits) = root.restored_commits.take() {
        root.journal = engine::Journal::uncapped(1, commits);
        root.sent_commit = commits;
        root.durable_commit = commits;
    }
    for _ in 0..RESTORE_PASSES {
        let mut output = skein_lib::Queue::uncapped();
        request = match request {
            jig_domain_core::start::Action::Step(phase) => match phase {
                jig_domain_core::start::Phase::Sweep => root.engine.core_mut().start_sweep(
                    &crate::limits::environment_core(&env),
                    jig_domain_core::start::SweepInput::Finished,
                    &mut output,
                ),
                phase @ (jig_domain_core::start::Phase::LoadCore
                | jig_domain_core::start::Phase::LoadInbox
                | jig_domain_core::start::Phase::RestoreConnector { .. }
                | jig_domain_core::start::Phase::Link
                | jig_domain_core::start::Phase::Switch
                | jig_domain_core::start::Phase::Wires
                | jig_domain_core::start::Phase::AdoptRuns
                | jig_domain_core::start::Phase::Open) => {
                    root.engine.core_mut().start_done(&crate::limits::environment_core(&env), phase, &mut output)
                }
            },
            jig_domain_core::start::Action::Commit(continuation) => {
                root.engine.core_mut().start_resume(&crate::limits::environment_core(&env), continuation, &mut output)
            }
            jig_domain_core::start::Action::Idle => {
                root.opened = true;
                break;
            }
            jig_domain_core::start::Action::Refused(failure) => panic!("fixture start: {failure:?}"),
        };
    }
    assert!(root.ready(), "fixture store restored");
    (root, env)
}

fn restore_fixture_row(root: &mut engine::Domain, env: &Env<engine::Limits>, row: engine::Record) {
    use jig_domain_core::Restore;
    use temper_engine_domain_forge as forge;
    match row {
        engine::Record::Core(row) => {
            engine::store_route::observe_row(root, env, &row);
            let core_env = crate::limits::environment_core(env);
            match root.engine.core_mut().restore(&core_env, row) {
                Restore::Deployment { commits } => {
                    root.core_header_loaded = true;
                    root.restored_commits = Some(commits);
                }
                Restore::Kept | Restore::Swept => {}
                Restore::Refused => panic!("fixture could not restore: {:?}", root.engine.core().start_failure()),
            }
        }
        engine::Record::Forge { row } => {
            if let forge::Stored::Repository(repository) = row.as_ref() {
                let connector = root.config.forge_connector;
                root.work.push(engine::Work::Connector(
                    jig_domain_contract::Connector(connector),
                    forge::adoption_up(connector, repository),
                ));
            }
            if let forge::Stored::LimitHold(row) = *row {
                engine::route::forge_route::limit_holds::restore(root, row);
            } else {
                root.work.push(engine::Work::Forge(forge::Event::Restore { record: *row }));
            }
        }
    }
}
