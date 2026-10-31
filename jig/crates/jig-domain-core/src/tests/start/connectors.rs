//! Connector loading and the durable output gate (jig/docs/design/domain/engine.md, 6).
use super::{Action, Core, Env, Phase, Queue, Request, SweepInput, Time, Wall, config, limits, step_phase};
use crate::{Ask, CoreRecord, Restored, connector, resources, tasks};

fn fresh() -> (Core, Env<crate::Limits>) {
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits() };
    let mut configured = config(41);
    configured.connectors = Box::new([1, 2]);
    configured.grants = Box::new([]);
    let mut core = Core::new(configured, &env.limits).expect("configured connectors");
    {
        step_phase(core.start_begin(false, env.wall), Phase::LoadCore);
    }
    let mut header = core.counters.deployment();
    header.commits = 1;
    header.projects = 1;
    assert_eq!(core.restore_core(CoreRecord::Deployment(header), &env.limits), Restored::Deployment { commits: 1 });
    let project = &config(41).seed_projects[0];
    let value = crate::policy::snapshot(&project.policy.policy, &[], &[]).expect("fixture policy");
    assert!(core.restore_policy(&env.limits, project.number, project.name.clone(), value));
    (core, env)
}

fn name(connector: u16) -> tasks::Name {
    tasks::Name { connector, path: Box::new([Box::from(b"resource".as_slice())]) }
}

fn to_connectors(core: &mut Core, env: &Env<crate::Limits>) {
    {
        let mut out = Queue::uncapped();
        step_phase(core.start_done(env, Phase::LoadCore, &mut out), Phase::LoadInbox);
        step_phase(core.start_done(env, Phase::LoadInbox, &mut out), Phase::RestoreConnector { connector: 1 });
        assert!(out.is_empty());
    }
}

#[test]
fn roles_follow_each_connector_and_keep_the_name_bound_during_start() {
    {
        let (mut core, env) = fresh();
        assert!(!core.restore_resource_role(&env, 1, name(1), connector::ResourceRole::Context));
        to_connectors(&mut core, &env);
        for connector in [1, 2] {
            let other = if connector == 1 { 2 } else { 1 };
            assert!(!core.restore_resource_role(&env, 1, name(other), connector::ResourceRole::Context));
            let invalid = tasks::Name { connector, path: Box::new([Box::from(&[b'x'; 129][..])]) };
            assert!(!core.restore_resource_role(&env, 1, invalid, connector::ResourceRole::Context));
            assert!(core.restore_resource_role(&env, 1, name(connector), connector::ResourceRole::Context));
            assert_eq!(
                core.resource_roles.get(&resources::Key { project: 1, name: name(connector) }),
                Some(&connector::ResourceRole::Context)
            );
            if connector == 1 {
                {
                    step_phase(
                        core.start_done(&env, Phase::RestoreConnector { connector }, &mut Queue::uncapped()),
                        Phase::RestoreConnector { connector: 2 },
                    );
                }
            }
        }
        assert_eq!(core.start_failure(), None);
    }
}

#[test]
fn resource_reports_load_access_slots_and_holds_before_link_without_asking() {
    {
        let (mut core, env) = fresh();
        let policy = crate::policy::snapshot(&config(41).seed_projects[0].policy.policy, &[], &[])
            .expect("stored project policy");
        assert!(core.restore_policy(&env.limits, 1, "Project 1".into(), policy));
        to_connectors(&mut core, &env);
        for connector in [1, 2] {
            let pool = tasks::PoolSlots { number: u64::from(connector), pool: name(connector), slots: 3 };
            // Slots are durable task rows; the connector's loading report must not replace them.
            let mut child = Queue::uncapped();
            tasks::step(
                &mut core.tasks,
                &Env { now: env.now, wall: env.wall, limits: env.limits.tasks },
                tasks::Event::Restore { record: tasks::Stored::Pool(pool) },
                &mut child,
            );
            assert!(child.is_empty());
            let crate::Requests::Out(mut out) = crate::step(
                &mut core,
                &env,
                crate::Event::EffectConnector(connector::Event::Resource {
                    project: 1,
                    name: name(connector),
                    role: connector::ResourceRole::Context,
                    hold: connector::HoldKind::Pooled { slots: 5, wait: true },
                }),
            );
            let Some(Request::Decided) = out.pop() else { panic!("loading decision completes") };
            assert!(out.is_empty(), "loading reports cannot emit connector asks or other outputs");
            assert_eq!(
                core.tasks.resource_hold(&name(connector)),
                Some(tasks::HoldKind::Pooled { taken: tasks::Taken::Waits })
            );
            assert_eq!(
                core.resource_roles.get(&resources::Key { project: 1, name: name(connector) }),
                Some(&connector::ResourceRole::Context)
            );
            {
                let next = if connector == 1 { Phase::RestoreConnector { connector: 2 } } else { Phase::Link };
                step_phase(core.start_done(&env, Phase::RestoreConnector { connector }, &mut Queue::uncapped()), next);
            }
        }
        let mut out = Queue::uncapped();
        {
            step_phase(core.start_done(&env, Phase::Link, &mut out), Phase::Sweep);
        }
        assert!(core.tasks.checked());
        assert_eq!(core.start_failure(), None);
        assert!(out.is_empty());
    }
}

#[test]
fn a_connector_ask_waits_for_the_wires_continuation() {
    let (mut core, env) = fresh();
    to_connectors(&mut core, &env);
    // An internal output fixture pins the same held queue used by loading decisions.
    core.start.held.push(Request::Ask { connector: 2, ask: Ask::Effect(connector::Ask::ReadAfresh) });
    assert_eq!(core.start.held.len(), 1);
    let Some(Request::Ask { connector, ask: Ask::Effect(connector::Ask::ReadAfresh) }) = core.start.held.pop() else {
        panic!("typed held connector ask")
    };
    assert_eq!(connector, 2);
    core.start.held.push(Request::Ask { connector, ask: Ask::Effect(connector::Ask::ReadAfresh) });
    let mut out = Queue::uncapped();
    step_phase(
        core.start_done(&env, Phase::RestoreConnector { connector: 1 }, &mut out),
        Phase::RestoreConnector { connector: 2 },
    );
    step_phase(core.start_done(&env, Phase::RestoreConnector { connector: 2 }, &mut out), Phase::Link);
    step_phase(core.start_done(&env, Phase::Link, &mut out), Phase::Sweep);
    step_phase(core.start_sweep(&env, SweepInput::Finished, &mut out), Phase::Switch);
    step_phase(core.start_done(&env, Phase::Switch, &mut out), Phase::Wires);
    let Action::Commit(wires) = core.start_done(&env, Phase::Wires, &mut out) else { panic!("Wires commits") };
    assert!(out.is_empty());
    assert_eq!(core.start.held.len(), 2);
    step_phase(core.start_resume(&env, wires, &mut out), Phase::AdoptRuns);
    assert!(core.start.held.is_empty());
    let Some(Request::Ask { connector: 2, ask: Ask::Effect(connector::Ask::ReadAfresh) }) = out.pop() else {
        panic!("connector ask releases after Wires")
    };
    for _ in 0..out.len() {
        match out.pop().expect("released startup output") {
            Request::Now(now) => {
                let crate::Now::Account(_) = *now else { panic!("account startup output") };
            }
            Request::Decided => {}
            Request::Ask { .. } | Request::Held(_) | Request::Write(_) | Request::ProjectMade { .. } => {
                panic!("only the held connector ask and account startup leave")
            }
        }
    }
}

#[test]
fn another_connectors_resource_report_is_not_loading_input() {
    let (mut core, env) = fresh();
    to_connectors(&mut core, &env);
    let rejected = std::panic::catch_unwind(core::panic::AssertUnwindSafe(|| {
        crate::step(
            &mut core,
            &env,
            crate::Event::EffectConnector(connector::Event::Resource {
                project: 1,
                name: name(2),
                role: connector::ResourceRole::Owned,
                hold: connector::HoldKind::Shared,
            }),
        )
    }));
    let _panic = rejected.expect_err("another connector is not in its restore phase");
}

#[test]
fn a_full_roles_cache_fails_closed_without_refusing_the_start() {
    let (mut core, env) = fresh();
    to_connectors(&mut core, &env);
    core.resource_roles = skein_lib::Map::with_capacity(1);
    assert!(core.restore_resource_role(&env, 1, name(1), connector::ResourceRole::Context));
    assert!(!core.restore_resource_role(&env, 2, name(1), connector::ResourceRole::Context));
    assert!(core.resource_roles_full);
    assert_eq!(core.start_failure(), None);
    assert!(!core.restore_resource_role(&env, 1, name(1), connector::ResourceRole::Owned));
    assert_eq!(
        core.resource_roles.get(&resources::Key { project: 1, name: name(1) }),
        Some(&connector::ResourceRole::Owned)
    );
    assert_eq!(core.start_failure(), None);
}
