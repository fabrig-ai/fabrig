//! Connector loading and the durable output gate (jig/docs/design/domain/engine/restart.md).
use crate::tests::new_core;
use crate::{
    CoreRecord, Event, Held, Restored,
    effects::{connector, resources},
    people, tasks,
    tests::start::{Action, Core, Env, Phase, Queue, Request, SweepInput, Time, Wall, config, limits, step_phase},
};
use jig_domain_contract as contract;
use skein_lib::{ReplyTo, Token};

fn fresh() -> (Core, Env<crate::Limits>) {
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits() };
    let mut configured = config(41);
    configured.connectors = Box::new([1, 2]);
    configured.grants = Box::new([]);
    let mut core = new_core(configured, &env.limits).expect("configured connectors");
    {
        step_phase(core.start_begin(false, env.wall), Phase::LoadCore);
    }
    let mut header = core.counters.deployment();
    header.commits = 1;
    header.projects = 1;
    assert_eq!(core.restore_core(CoreRecord::Deployment(header), &env.limits), Restored::Deployment { commits: 1 });
    let project = &config(41).seed_projects[0];
    let value = crate::admission::policy::snapshot(&project.policy.policy, &[], &[]).expect("fixture policy");
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
        while let Some(request) = out.pop() {
            crate::tests::start::worker_table(&request);
        }
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
        to_connectors(&mut core, &env);
        for connector in [1, 2] {
            let pool = tasks::PoolSlots { number: u64::from(connector), pool: name(connector), slots: 3 };
            // Slots are durable task rows; the connector's loading report must not replace them.
            let mut child = Queue::uncapped();
            tasks::step(
                &mut core.tasks,
                &env.with(env.limits.tasks),
                tasks::Event::Restore { record: tasks::Stored::Pool(pool) },
                &mut child,
            );
            assert!(child.is_empty());
            let out = crate::tests::connector_step(
                &mut core,
                &env,
                name(connector).connector,
                contract::Up::Resources(contract::resources::Up::Resource {
                    project: 1,
                    name: name(connector),
                    role: connector::ResourceRole::Context,
                    hold: connector::HoldKind::Pooled { slots: 5, wait: true },
                }),
            );
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
    core.start.held.push(Request::Ask {
        connector: 2,
        ask: crate::Ask::Connector(contract::Ask::Lifecycle(contract::lifecycle::Ask::CatchUp)),
    });
    assert_eq!(core.start.held.len(), 1);
    let Some(Request::Ask {
        connector,
        ask: crate::Ask::Connector(contract::Ask::Lifecycle(contract::lifecycle::Ask::CatchUp)),
    }) = core.start.held.pop()
    else {
        panic!("typed held connector ask")
    };
    assert_eq!(connector, 2);
    core.start.held.push(Request::Ask {
        connector,
        ask: crate::Ask::Connector(contract::Ask::Lifecycle(contract::lifecycle::Ask::CatchUp)),
    });
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
    let Some(Request::Ask {
        connector: 2,
        ask: crate::Ask::Connector(contract::Ask::Lifecycle(contract::lifecycle::Ask::CatchUp)),
    }) = out.pop()
    else {
        panic!("connector ask releases after Wires")
    };
    for _ in 0..out.len() {
        match out.pop().expect("released startup output") {
            Request::Now(now) => {
                let crate::Now::Account(_) = *now else { panic!("account startup output") };
            }

            Request::Ask { .. } | Request::Held(_) | Request::Write(_) | Request::ProjectMade { .. } => {
                panic!("only the held connector ask and account startup leave")
            }
        }
    }
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

#[test]
fn opening_asks_each_connector_to_catch_up_without_waiting_for_reports() {
    let (mut core, env) = fresh();
    to_connectors(&mut core, &env);
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
    step_phase(core.start_resume(&env, wires, &mut out), Phase::AdoptRuns);
    step_phase(core.start_done(&env, Phase::AdoptRuns, &mut out), Phase::Open);
    while out.pop().is_some() {}
    assert!(!core.start_opened());
    let Action::Idle = core.start_done(&env, Phase::Open, &mut out) else { panic!("Open ends the script") };
    assert!(core.start_opened() && core.ready());
    assert_eq!(core.start_phase(), None);
    for expected in [1, 2] {
        assert!(!core.caught_up(expected));
        let Some(Request::Ask {
            connector,
            ask: crate::Ask::Connector(contract::Ask::Lifecycle(contract::lifecycle::Ask::CatchUp)),
        }) = out.pop()
        else {
            panic!("one catching-up ask per connector")
        };
        assert_eq!(connector, expected);
    }
    assert!(out.is_empty());

    let mut signed = crate::step(
        &mut core,
        &env,
        Event::SignIn {
            until: None,
            held: None,
            reply_to: ReplyTo::new(Token::new(81)),
            identity: people::Identity {
                key: people::IdentityKey { provider: 0, subject: Box::new([9]) },
                login: Box::from(b"person".as_slice()),
                name: Box::from(b"Person".as_slice()),
            },
        },
    );
    let mut session = None;
    while let Some(request) = signed.pop() {
        if let Request::Held(held) = request
            && let Held::PeopleReply { reply: people::Reply::SignedIn { sign_in, .. }, .. } = *held
        {
            session = Some(sign_in);
        }
    }
    let mut answered = crate::step(
        &mut core,
        &env,
        Event::People(people::Event::Ask {
            models: Box::new([]),
            reply_to: ReplyTo::new(Token::new(82)),
            sign_in: session.expect("sign-in answered before reports"),
            key: [82; 16],
            ask: people::Ask::Conclude { project: 1, task: 999 },
        }),
    );
    let mut reply = None;
    while let Some(request) = answered.pop() {
        if let Request::Held(held) = request
            && let Held::PeopleReply { reply: answer, .. } = *held
        {
            reply = Some(answer);
        }
    }
    assert!(reply.is_some(), "the people ask answers before reports");
    assert_ne!(reply, Some(people::Reply::Again(people::Again::NotReady)));

    core.catch_up_done(1, connector::RestartStage::ReadAfresh);
    core.catch_up_done(2, connector::RestartStage::Settled);
    assert!(!core.caught_up(1) && !core.caught_up(2));
    core.catch_up_done(1, connector::RestartStage::Settled);
    core.catch_up_done(2, connector::RestartStage::ReadAfresh);
    assert!(core.caught_up(1) && core.caught_up(2));
}
