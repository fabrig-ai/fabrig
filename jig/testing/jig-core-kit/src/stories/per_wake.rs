//! Drivers for activation-specific runs (`jig/docs/design/domain/tasks.md`, 8.6 and 9).
use crate::{
    fixtures,
    world::{Setup, World},
};
use jig_domain_core as core;
use jig_domain_tasks as tasks;
use jig_test_domain as root;
use skein_lib::{ReplyTo, Token};

pub const M1: tasks::ModelEffort = tasks::ModelEffort { model: 0, effort: Some(0) };
pub const M2: tasks::ModelEffort = tasks::ModelEffort { model: 1, effort: Some(1) };

#[must_use]
pub fn world(seed: u64, parts: bool) -> World {
    let (mut configuration, mut limits) = jig_test_profile::two_charters(seed);
    let (_, plan_limits) = crate::faults::plan_fixture(seed);
    limits.core.tasks = tasks::Limits { charters: 2, executor_kinds: 3, ..plan_limits.core.tasks };
    limits.core.fleet = plan_limits.core.fleet;
    limits.core.views = plan_limits.core.views;
    limits.core.call_records = plan_limits.core.call_records;
    limits.core.run_bytes = 1024;
    configuration.core.defaults.rules.maximum_run_spend = 10;
    configuration.core.defaults.rules.ceiling.delegation.tasks = 4;
    configuration.core.defaults.rules.ceiling.delegation.depth = 3;
    let policy = &mut configuration.core.seed_projects[0].policy.policy;
    policy.ceiling.delegation.tasks = 4;
    policy.ceiling.delegation.depth = 3;
    policy.roles[0].authority.delegation.tasks = 4;
    policy.roles[0].authority.delegation.depth = 3;
    let mut charter = jig_test_profile::per_wake_charter();
    if !parts {
        charter.parts = Box::new([]);
    }
    configuration.core.defaults.charters[1] = charter;
    World::new(Setup { seed, configuration: Some(configuration), limits, ..Setup::default() })
}

#[must_use]
pub fn delegate(world: &mut World, later: bool) -> u64 {
    let caller = fixtures::run(world);
    let mut member = fixtures::member(world, tasks::Executor::Agent { charter: 2 }, b"run for its inbox");
    member.authority.budget.spend = 60;
    member.llm = Some(tasks::LlmChoice {
        account: None,
        main: M1,
        subagents: Box::new([]),
        later: later.then(|| tasks::LlmLater { main: M2, subagents: Box::new([]) }),
    });
    fixtures::delegate(world, caller, 1, member)
}

/// A delegate asks its requester through the same named action as a scripted worker.
#[must_use]
pub fn question(world: &mut World, requester: (u64, u64)) -> tasks::Word {
    let child = child(world, requester);
    question_from(world, child, requester.0)
}

#[must_use]
pub fn child(world: &mut World, requester: (u64, u64)) -> (u64, u64) {
    let mut member = fixtures::member(world, tasks::Executor::Agent { charter: 1 }, b"ask its requester");
    member.authority.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
    let child = fixtures::delegate(world, requester, 1, member);
    (child, fixtures::live_row(world, child).attempt)
}

#[must_use]
pub fn question_from(world: &mut World, child: (u64, u64), requester: u64) -> tasks::Word {
    world.send(root::Event::Core(core::Event::NamedAction {
        to: ReplyTo::new(Token::new(9200)),
        key: core::CallKey { task: child.0, attempt: child.1, completion: 1, position: 0 },
        action: core::NamedAction::Message {
            target: requester,
            kind: tasks::MessageKind::Question,
            words: b"Which next?".as_slice().into(),
        },
    }));
    fixtures::live_row(world, requester)
        .inbox
        .iter()
        .find(|word| word.kind == tasks::MessageKind::Question)
        .expect("requester's question")
        .clone()
}

/// A held delegate raises a decision entry at its nearest covering requester.
#[must_use]
pub fn escalation(world: &mut World, requester: (u64, u64)) -> u64 {
    let child = child(world, requester);
    world.send(root::Event::Answer {
        channel: Token::new(7),
        task: child.0,
        attempt: child.1,
        cumulative: 0,
        end: tasks::End::Exceeded(tasks::Exceeded { bound: tasks::Bound::Reasoning, value: 2048 }),
        read: None,
        refused: Box::new([]),
    });
    child.0
}

/// Park the named attempt, independently of the last child assigned to the worker.
pub fn park(world: &mut World, run: (u64, u64), read: Option<u64>) {
    world.await_timer(run.0);
    world.send(root::Event::Answer {
        channel: Token::new(7),
        task: run.0,
        attempt: run.1,
        cumulative: 0,
        end: tasks::End::Parked,
        read,
        refused: Box::new([]),
    });
}

pub fn turn(world: &mut World, run: (u64, u64), read: Option<u64>) {
    world.send(root::Event::Turn {
        channel: Token::new(7),
        task: run.0,
        attempt: run.1,
        turn: 1,
        cumulative: 0,
        read,
        transcript: b"committed turn".as_slice().into(),
        opens_window: true,
        refused: Box::new([]),
    });
}
