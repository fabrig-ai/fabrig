//! Small core steps with public bounds enabled (jig/docs/design/domain/people.md, 13.6).
use super::*;
use crate::PublicLine;

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
    let mut config = crate::tests::config(41);
    config.connectors = Box::new([]);
    let mut core = Core::new(config, &limits).unwrap();
    core.authority = old.authority;
    core.people = old.people;
    core.accounts = old.accounts;
    core.projects = old.projects;
    core.names = old.names;
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
    let mut given = core.creating_authority(1, 1, people::Role::Owner, u32::MAX);
    given.delegation.tasks = 32;
    given.delegation.depth = 6;
    let budget = if parent.is_some() { 10 } else { 100 };
    given.budget.spend = budget;
    let authority = crate::translate::task_authority(&given);
    let funder = match parent {
        Some(task) => tasks::Funder::Task(task),
        None => tasks::Funder::Period { project: 1, period: 1 },
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
        project: 1,
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
}

pub(super) fn ready(core: &Core) -> &Project {
    match core.public_index.project(1) {
        Some(Entry::Ready(project)) => project,
        Some(Entry::Busy) | None => panic!("public project built"),
    }
}

pub(super) fn zero() -> PublicCounts {
    PublicCounts { working: 0, waiting: 0, held: 0, closing: 0, ended: 0 }
}

pub(super) fn snapshot(project: &Project) -> Box<[PublicLine]> {
    let tasks = project.public_tasks();
    let mut lines = skein_lib::List::with_capacity(u32::try_from(tasks.len()).unwrap());
    for &task in &tasks {
        lines.push(project.line(task).unwrap().clone()).unwrap();
    }
    lines.into_boxed()
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
        places: Box::new([]),
        reply_to: reply(),
        by: tasks::Party::Person(1),
        task,
        message,
        stop_run: false,
        funder: None,
        amendment: tasks::Amendment {
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
        },
    };
    let mut env = environment(limits);
    env.wall = wall;
    let _out = crate::step(core, &env, Event::Tasks(event));
    core.reclaim();
}

pub(super) fn end(core: &mut Core, limits: &Limits, task: u64) {
    send(
        core,
        limits,
        tasks::Event::Control {
            reply_to: reply(),
            by: tasks::Party::Person(1),
            task,
            control: tasks::Control::Cancel { reason: Box::new([]), grace: None, message: None },
        },
    );
    send(core, limits, tasks::Event::EffectsSettled { task });
    send(core, limits, tasks::Event::Settled { task });
    assert!(core.tasks.task(task).is_none(), "ended task {task}");
}
