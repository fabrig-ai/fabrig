//! Explicit connector terminals belong to core bookkeeping even for live tasks
//! (`temper/docs/design/domain/forge/issues.md`; `jig/docs/design/domain/connectors.md`, 8).
use jig_domain_contract as contract;
use jig_domain_core as core;
use jig_domain_core::engine::{self, In, Out};
use jig_domain_tasks as tasks;
use skein_lib::{Env, Queue, ReplyTo, Token, Wall};
use temper_engine_domain_forge as forge;

pub(super) fn live_goal(tracked: Option<u32>) -> (crate::Domain, Env<crate::Limits>, u64) {
    live_task(tracked, tasks::Executor::Procedure { connector: 0, code: 1 })
}

pub(crate) fn live_task(tracked: Option<u32>, executor: tasks::Executor) -> (crate::Domain, Env<crate::Limits>, u64) {
    let (mut domain, env) = super::resume_fixture::root();
    let goal = make_task(&mut domain, &env, tracked, executor, tasks::Party::Deployment { project: 1 });
    (domain, env, goal)
}

pub(crate) fn make_task(
    domain: &mut crate::Domain,
    env: &Env<crate::Limits>,
    tracked: Option<u32>,
    executor: tasks::Executor,
    creator: tasks::Party,
) -> u64 {
    let goal = crate::fresh(&mut domain.engine.core_mut().counters, crate::Family::Task).unwrap();
    let task_env = env.with(env.limits.tasks);
    let mut out = Queue::uncapped();
    for event in [
        tasks::Event::Restored,
        tasks::Event::OpenPeriod { reply_to: ReplyTo::new(Token::new(2)), project: 1, period: 1, budget: 100 },
        tasks::Event::Make {
            reads_project: false,
            maker: None,
            places: Box::new([]),
            reply_to: ReplyTo::new(Token::new(3)),
            creator,
            batch: Box::new([tasks::New {
                number: goal,
                project: 1,
                label: 0,
                flags: tasks::Flags::PLAIN,
                executor,
                llm: None,
                spec: tasks::Spec {
                    job: None,
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
        tasks::step(&mut domain.engine.core_mut().tasks, &task_env, event, &mut out);
        while out.pop().is_some() {}
    }
    assert_eq!(domain.engine.core().tasks.task(goal).expect("live goal").tracked, tracked);
    goal
}

fn terminal(domain: &mut crate::Domain, env: &Env<crate::Limits>, goal: u64) -> In {
    let when = Wall::from_nanos(1);
    assert_eq!(domain.forge.next_wall_deadline(), None);
    domain.forge.projection_after(goal, when);
    assert_eq!(domain.forge.next_wall_deadline(), Some(when));
    assert!(domain.engine.core().tasks.task(goal).is_some(), "terminal arrives while task is live");
    let mut out = Queue::uncapped();
    out.push(forge::Request::Up(contract::Up::Projections(contract::projections::Up::Settled { goal })));
    let mut decision = crate::Decision::decide(&mut domain.journal, &env.limits.journal);
    crate::route::forge_route::outputs(domain, env, &mut decision, &mut out);
    let Some(crate::Work::Connector(
        connector,
        up @ contract::Up::Projections(contract::projections::Up::Settled { goal: received }),
    )) = domain.work.pop()
    else {
        panic!("explicit terminal must reach core while task is live");
    };
    assert_eq!(received, goal);
    assert_eq!(connector.0, domain.config.forge_connector);
    assert!(domain.work.is_empty());
    assert_eq!(domain.forge.next_wall_deadline(), None, "terminal cancels the forge timer");
    In::Connector(connector, up)
}

#[test]
fn a_live_tracked_goal_forwards_its_explicit_terminal_and_cancels_its_timer() {
    let (mut domain, env, goal) = live_goal(Some(1));
    let event = terminal(&mut domain, &env, goal);
    let mut out = Queue::uncapped();
    engine::step(&mut domain.engine, &crate::limits::environment_core(&env), event, &mut out);
    assert!(out.is_empty());

    assert!(domain.engine.core().tasks.task(goal).is_some());
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
    let task = domain.engine.core().tasks.task(goal).unwrap().clone();
    let funding = *domain.engine.core().tasks.funding(task.funder).unwrap();
    let records = [
        crate::Record::Core(core::Record::Core(core::CoreRecord::Projection(core::ProjectionRecord::GoalWords {
            goal,
            task: goal,
            words: projection.words.clone(),
        }))),
        crate::Record::Core(core::Record::Core(core::CoreRecord::Projection(core::ProjectionRecord::TaskWords {
            goal,
            task: goal,
            words: projection.words.clone(),
        }))),
        crate::Record::Core(core::Record::Tasks(tasks::Stored::Live(Box::new(task)))),
        crate::Record::Core(core::Record::Tasks(tasks::Stored::Ledger(funding))),
        crate::Record::Core(core::Record::Core(core::CoreRecord::Projection(core::ProjectionRecord::GoalHot {
            goal: projection.hot(),
            pending: Box::new([domain.config.forge_connector]),
            changed: false,
            fed_at: None,
        }))),
        crate::Record::Core(core::Record::Core(core::CoreRecord::Projection(core::ProjectionRecord::TaskHot {
            goal,
            task: core::ProjectionTaskHot {
                number: goal,
                requester: tasks::Party::Deployment { project: 1 },
                phase: tasks::Phase::Waiting,
            },
        }))),
    ];
    (domain, _) = super::resume_fixture::root_with_records(super::resume_fixture::config(7), &env.limits, &records);
    let event = terminal(&mut domain, &env, goal);
    let mut out = Queue::uncapped();
    engine::step(&mut domain.engine, &crate::limits::environment_core(&env), event, &mut out);
    for key in [
        core::ProjectionKey::TaskWords { goal, task: goal },
        core::ProjectionKey::Task { goal, task: goal },
        core::ProjectionKey::GoalWords(goal),
        core::ProjectionKey::Goal(goal),
    ] {
        let Some(Out::Core(core::Request::Write(core::Write::Erase(core::Key::Core(core::CoreKey::Projection(
            received,
        )))))) = out.pop()
        else {
            panic!("core erases the retained projection row");
        };
        assert_eq!(received, key);
    }
    let Some(Out::Ask(connector, contract::Ask::Projections(contract::projections::Ask::Forget { goal: received }))) =
        out.pop()
    else {
        panic!("core forgets the settled connector projection")
    };
    assert_eq!(received, goal);
    assert_eq!(connector.0, domain.config.forge_connector);
    assert!(out.is_empty());

    assert_eq!(domain.engine.core().tasks.task(goal).expect("untracking keeps task live").tracked, None);
}
