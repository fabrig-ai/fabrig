//! Small core steps with public bounds enabled (jig/docs/design/domain/people/public-reading.md, 6).
use super::*;

pub(super) fn fixture() -> (Core, Limits) {
    let (old, mut limits) = crate::tests::core();
    limits.public = PublicIndexLimits { goals: 4, ended: 2, lines: 8, depth: 4 };
    limits.tasks.tasks = 32;
    limits.tasks.project_tasks = 32;
    limits.tasks.project_goals = 16;
    limits.tasks.tree_tasks = 32;
    limits.tasks.depth = 6;
    limits.tasks.delegates = 8;
    limits.tasks.batch = 8;
    limits.tasks.inbox_messages = 32;
    limits.tasks.inbox_bytes = 8192;
    limits.tasks.spec_bytes = 512;
    limits.people.inbox_tasks = 64;
    limits.people.projects = 8;
    limits.authority.projects = 8;
    let mut config = crate::tests::config(41);
    config.connectors = Box::new([1, 2]);
    let mut core = Core::new(config, &limits).unwrap();
    core.authority = old.authority;
    core.people = old.people;
    core.accounts = old.accounts;
    core.projects = skein_lib::List::with_capacity(8);
    core.names = skein_lib::Map::with_capacity(8);
    for &project in &old.projects {
        core.projects.push(project).unwrap();
    }
    for (&project, name) in &old.names {
        core.names.insert(project, name.clone()).unwrap();
    }
    core.permission_roles = old.permission_roles;
    core.template_overrides = old.template_overrides;
    core.counters.deployment.projects = 1;
    send(&mut core, &limits, tasks::Event::Check);
    send(&mut core, &limits, tasks::Event::Restored);
    send(&mut core, &limits, tasks::Event::OpenPeriod { reply_to: reply(), project: 1, period: 1, budget: 1000 });
    (core, limits)
}

pub(super) fn environment(limits: &Limits) -> Env<Limits> {
    Env { now: Time::ZERO, wall: Wall::from_nanos(10_000_000_000), limits: *limits }
}

pub(super) fn reply() -> ReplyTo {
    ReplyTo::new(Token::new(900))
}

pub(super) fn send(core: &mut Core, limits: &Limits, event: tasks::Event) {
    let _out = crate::step(core, &environment(limits), Event::Tasks(event));
    let _header = core.counters.commit_header();
    settle(core, limits);
    core.reclaim();
}

pub(super) fn make(
    core: &mut Core,
    limits: &Limits,
    number: u64,
    parent: Option<u64>,
    tracked: Option<u32>,
    conversation: bool,
) {
    make_in_project(core, limits, 1, number, parent, tracked, conversation);
}

pub(super) fn make_in_project(
    core: &mut Core,
    limits: &Limits,
    project: u32,
    number: u64,
    parent: Option<u64>,
    tracked: Option<u32>,
    conversation: bool,
) {
    let mut given = core.creating_authority(1, 1, people::Role::Owner, u32::MAX);
    given.delegation.tasks = 32;
    given.delegation.depth = 6;
    let budget = match parent {
        Some(task) => core.tasks.task(task).unwrap().numbers.budget.checked_div(3).unwrap(),
        None => 100,
    };
    given.budget.spend = budget;
    let authority = crate::translate::task_authority(&given);
    let funder = match parent {
        Some(task) => tasks::Funder::Task(task),
        None => tasks::Funder::Period { project, period: 1 },
    };
    let creator = match parent {
        Some(task) => tasks::Party::Task(task),
        None => tasks::Party::Person(1),
    };
    let new = tasks::New {
        deadline: tasks::Deadline::Named,
        label: 0,
        flags: tasks::Flags { conversation, ..tasks::Flags::PLAIN },
        number,
        project,
        executor: tasks::Executor::Agent { charter: 1 },
        llm: None,
        spec: tasks::Spec {
            job: None,
            words: Box::from(b"Public title\nPrivate specification".as_slice()),
            parameters: Box::new([]),
            inputs: Box::new([]),
            asks: false,
        },
        contract: tasks::Contract::Report { words: 64 },
        authority,
        numbers: tasks::Numbers { budget, spent: 0, spent_below: 0, reserved: 0 },
        funder,
        dependencies: Box::new([]),
        holdings: Box::new([]),
        wake: tasks::WakePolicy::DEFAULT,
        recurring: None,
        tracked,
    };
    send(core, limits, tasks::Event::Make { reply_to: reply(), creator, batch: Box::new([new]) });
    assert!(core.tasks.task(number).is_some(), "made task {number}");
}

pub(super) fn visibility(core: &mut Core, limits: &Limits, public: bool) {
    let mut child = Queue::uncapped();
    child.push(people::Request::Route {
        request: Token::new(901),
        person: 1,
        project: 1,
        role: Some(people::Role::Owner),
        ask: Box::new(people::Ask::SetVisibility { project: 1, public }),
        models: Box::new([]),
    });
    let mut work = Queue::uncapped();
    let _out = crate::routing::party::people::tag_people(core, &environment(limits), child, &mut work);
    let Some(crate::routing::Work::Event(Event::People(people::Event::Decided { outcome, .. }))) = work.pop() else {
        panic!("visibility decision");
    };
    assert_eq!(outcome, people::Outcome::VisibilitySet { project: 1, public });
    let _header = core.counters.commit_header();
    settle(core, limits);
}

pub(super) fn zero() -> PublicCounts {
    PublicCounts { working: 0, waiting: 0, held: 0, closing: 0, ended: 0 }
}

pub(super) fn amend(
    core: &mut Core,
    limits: &Limits,
    task: u64,
    tracking: Option<tasks::Tracking>,
    spec: Option<tasks::Spec>,
) {
    amend_at(core, limits, task, tracking, spec, Wall::from_nanos(10_000_000_000));
}

pub(super) fn amend_at(
    core: &mut Core,
    limits: &Limits,
    task: u64,
    tracking: Option<tasks::Tracking>,
    spec: Option<tasks::Spec>,
    wall: Wall,
) {
    let message = core.tasks.task(task).unwrap().last_message.checked_add(100).unwrap();
    let event = tasks::Event::Amend {
        operation: tasks::Amend {
            task,
            message,
            stop_run: false,
            amendment: Box::new(tasks::Amendment {
                tracked: tracking,
                spec,
                addressed: None,
                instructions: None,
                release: None,
                wake: None,
                dependencies: None,
                authority: None,
                llm: None,
                reason: Box::new([]),
            }),
        },
        places: Box::new([]),
        reply_to: reply(),
        by: tasks::Party::Person(1),
        funder: None,
    };
    let mut env = environment(limits);
    env.wall = wall;
    let _out = crate::step(core, &env, Event::Tasks(event));
    let _header = core.counters.commit_header();
    settle(core, limits);
    core.reclaim();
}

pub(super) fn end(core: &mut Core, limits: &Limits, task: u64) {
    send(
        core,
        limits,
        tasks::Event::Cancel {
            reply_to: reply(),
            by: tasks::Party::Person(1),
            operation: tasks::Cancel { task, reason: Box::new([]), grace: None, message: None },
        },
    );
    send(core, limits, tasks::Event::EffectsSettled { task });
    send(core, limits, tasks::Event::Settled { task });
    assert!(core.tasks.task(task).is_none(), "ended task {task}");
}

pub(super) fn deleting(core: &mut Core, limits: &Limits) {
    crate::admission::deletion::begin(
        core,
        &environment(limits),
        &mut Queue::uncapped(),
        &mut Queue::uncapped(),
        Token::new(902),
        1,
        None,
    )
    .unwrap();
    let _header = core.counters.commit_header();
    settle(core, limits);
}

pub(super) fn another_project(core: &mut Core, limits: &Limits, project: u32, name: &str, public: bool) {
    core.projects.push(project).unwrap();
    core.names.insert(project, Box::from(name)).unwrap();
    if public {
        core.visibility.public.insert(project, ()).unwrap();
        crate::public::publish(core, limits.public, project);
    }
}

fn settle(core: &mut Core, limits: &Limits) {
    for _ in 0_u8..4 {
        let mut out = crate::finish_decision(core, &environment(limits));
        if out.is_empty() {
            if core.counters.dirty() {
                let _header = core.counters.commit_header();
            }
            return;
        }
        for _ in 0..out.len() {
            if let Some(Request::Ask { connector, ask: Ask::ProjectGoal { feed } }) = out.pop()
                && feed.closing
            {
                let _settled = crate::step(
                    core,
                    &environment(limits),
                    Event::ProjectionSettled { goal: feed.goal.number, connector },
                );
            }
        }
        let _header = core.counters.commit_header();
    }
    panic!("fixture decisions did not finish");
}
