//! Explicit connector terminals belong to core bookkeeping even for live tasks
//! (`temper/docs/design/domain/forge.md`, 12; `jig/docs/design/domain/connectors.md`, 8).
use jig_domain_core as core;
use jig_domain_tasks as tasks;
use skein_lib::{Env, Queue, ReplyTo, Token, Wall};
use temper_engine_domain_forge as forge;

fn live_goal(tracked: Option<u32>) -> (crate::Domain, Env<crate::Limits>, u64) {
    let (mut domain, env) = super::resume_fixture::root();
    let goal = crate::fresh(&mut domain.core.counters, crate::Family::Task).unwrap();
    let task_env = Env { now: env.now, wall: env.wall, limits: env.limits.tasks };
    let mut out = Queue::uncapped();
    for event in [
        tasks::Event::Restored,
        tasks::Event::OpenPeriod { reply_to: ReplyTo::new(Token::new(2)), project: 1, period: 1, budget: 100 },
        tasks::Event::Make {
            reply_to: ReplyTo::new(Token::new(3)),
            creator: tasks::Party::Deployment { project: 1 },
            batch: Box::new([tasks::New {
                number: goal,
                project: 1,
                label: 0,
                flags: tasks::Flags::PLAIN,
                executor: tasks::Executor::Agent { charter: 1 },
                llm: None,
                spec: tasks::Spec {
                    asks: false,
                    words: b"live goal".as_slice().into(),
                    parameters: Box::new([]),
                    inputs: Box::new([]),
                },
                contract: tasks::Contract::Report { words: 1 },
                deadline: tasks::Deadline::Named,
                authority: super::held_assignment().run.authority,
                numbers: tasks::Numbers { budget: 1, spent: 0, spent_below: 0, reserved: 0 },
                funder: tasks::Funder::Period { project: 1, period: 1 },
                dependencies: Box::new([]),
                holdings: Box::new([]),
                wake: tasks::WakePolicy::DEFAULT,
                recurring: None,
                tracked,
            }]),
        },
    ] {
        tasks::step(&mut domain.core.tasks, &task_env, event, &mut out);
        while out.pop().is_some() {}
    }
    assert_eq!(domain.core.tasks.task(goal).expect("live goal").tracked, tracked);
    (domain, env, goal)
}

fn terminal(domain: &mut crate::Domain, env: &Env<crate::Limits>, goal: u64) -> core::Event {
    let when = Wall::from_nanos(1);
    assert_eq!(domain.forge.next_wall_deadline(), None);
    domain.forge.projection_after(goal, when);
    assert_eq!(domain.forge.next_wall_deadline(), Some(when));
    assert!(domain.core.tasks.task(goal).is_some(), "terminal arrives while task is live");
    let mut out = Queue::uncapped();
    out.push(forge::Request::ProjectionSettled { goal });
    let mut decision = crate::Decision::decide(&mut domain.journal, &env.limits.journal);
    crate::route::forge_route::outputs(domain, env, &mut decision, &mut out);
    let Some(crate::Work::Core(event)) = domain.work.pop() else {
        panic!("explicit terminal must reach core while task is live");
    };
    let core::Event::ProjectionSettled { goal: received, connector } = &event else {
        panic!("core receives the explicit projection terminal");
    };
    assert_eq!(*received, goal);
    assert_eq!(*connector, domain.config.forge_connector);
    assert!(domain.work.is_empty());
    assert_eq!(domain.forge.next_wall_deadline(), None, "terminal cancels the forge timer");
    event
}

#[test]
fn a_live_tracked_goal_forwards_its_explicit_terminal_and_cancels_its_timer() {
    let (mut domain, env, goal) = live_goal(Some(1));
    let event = terminal(&mut domain, &env, goal);
    let core::Requests::Out(mut out) = core::step(&mut domain.core, &crate::limits::environment_core(&env), event);
    let Some(core::Request::Decided) = out.pop() else {
        panic!("core owns stale-terminal bookkeeping");
    };
    assert!(out.is_empty());
    assert!(domain.core.tasks.task(goal).is_some());
}

#[test]
fn a_live_untracked_goal_settles_its_retained_core_projection() {
    let (mut domain, env, goal) = live_goal(None);
    let projection = core::ProjectionGoal {
        number: goal,
        project: 1,
        priority: 1,
        words: b"live goal".as_slice().into(),
        phase: tasks::Phase::Waiting,
    };
    let core_limits = crate::limits::core_limits(&env.limits);
    for row in [
        core::ProjectionRecord::Goal {
            goal: projection,
            pending: Box::new([domain.config.forge_connector]),
            changed: false,
            fed_at: None,
        },
        core::ProjectionRecord::Task {
            goal,
            task: core::ProjectionTask {
                number: goal,
                requester: tasks::Party::Deployment { project: 1 },
                words: b"live goal".as_slice().into(),
                phase: tasks::Phase::Waiting,
            },
        },
    ] {
        assert_eq!(domain.core.restore_core(core::CoreRecord::Projection(row), &core_limits), core::Restored::Live);
    }
    let event = terminal(&mut domain, &env, goal);
    let core::Requests::Out(mut out) = core::step(&mut domain.core, &crate::limits::environment_core(&env), event);
    for key in [core::ProjectionKey::Task { goal, task: goal }, core::ProjectionKey::Goal(goal)] {
        let Some(core::Request::Write(core::Write::Erase(core::Key::Core(core::CoreKey::Projection(received))))) =
            out.pop()
        else {
            panic!("core erases the retained projection row");
        };
        assert_eq!(received, key);
    }
    let Some(core::Request::Ask { connector, ask: core::Ask::ForgetProjection { goal: received } }) = out.pop() else {
        panic!("core forgets the settled connector projection");
    };
    assert_eq!(received, goal);
    assert_eq!(connector, domain.config.forge_connector);
    let Some(core::Request::Decided) = out.pop() else {
        panic!("core completed the settlement decision");
    };
    assert!(out.is_empty());
    assert_eq!(domain.core.tasks.task(goal).expect("untracking keeps task live").tracked, None);
}
