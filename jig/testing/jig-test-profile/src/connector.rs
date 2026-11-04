//! Seeded connector profile (`jig/docs/design/domain/testing.md`, 2).
use jig_test_connector::{
    Config, Form, Hold, HoldMode, KindSpec, Limits, Path, PoolSpec, ProcedureAction, ProcedureResult, ProcedureSpec,
    Recovery, RequirementSpec, ResourceSpec, TopicSpec,
};
use skein_lib::{Duration, List};
/// Bounds shared by the seeded scenarios. Every vector they own fits here.
pub const LIMITS: Limits = Limits {
    tasks: 8,
    adoptions: 8,
    subscriptions: 8,
    pools: 1,
    resources: 2,
    topics: 1,
    kinds: 4,
    requirements: 2,
    procedures: 2,
    actions_per_procedure: 5,
    facts: 2,
    judges: 4,
    values: 8,
    value_bytes: 18,
    staged: 4,
    entries: 4,
    made: 8,
    resources_per_effect: 1,
    max_attempts: 3,
    write_lifetime: Duration::from_nanos(5),
    clock_margin: Duration::from_nanos(1),
    resources_per_task: 2,
    subscribers_per_topic: 8,
    lost_per_pool: 8,
    path_segments: 2,
    segment_bytes: 16,
};

/// One seeded resource path; zero is the deployment prefix.
#[must_use]
pub fn path(seed: u8, name: u8) -> Path {
    let mut segments = List::with_capacity(2);
    segments.push(Box::from([seed])).expect("one-byte seed fits");
    if name != 0 {
        segments.push(Box::from([name])).expect("one-byte resource fits");
    }
    Path::new(segments, LIMITS.segment_bytes).expect("seeded path fits")
}

/// Number of one of four effect kinds in this deployment.
#[must_use]
pub fn kind(seed: u8, local: u16) -> u16 {
    u16::from(seed) * 4 + local
}

/// Connector configuration drawn from a seed, including its procedure and judge.
#[must_use]
pub fn config(seed: u8) -> Config {
    Config {
        settings: None,
        deployment: (u128::from(seed) + 1).to_be_bytes(),
        prefix: path(seed, 0),
        resources: Box::from([
            ResourceSpec {
                credential: None,
                path: path(seed, 1),
                hold: Hold::Exclusive { mode: HoldMode::Wait },
                writable: true,
            },
            ResourceSpec {
                credential: None,
                path: path(seed, 2),
                hold: Hold::Pooled { mode: HoldMode::Wait },
                writable: true,
            },
        ]),
        pools: Box::from([PoolSpec { path: path(seed, 2), slots: 2 }]),
        topics: Box::from([TopicSpec { topic: kind(seed, 1), offered: true, resources: Box::new([path(seed, 1)]) }]),
        kinds: Box::from([
            KindSpec {
                waits_for_dependencies: false,
                kind: kind(seed, 1),
                form: Form::Creation,
                recovery: Recovery::Keyed,
                price: Some(3),
            },
            KindSpec {
                waits_for_dependencies: false,
                kind: kind(seed, 2),
                form: Form::Transition,
                recovery: Recovery::Conditional,
                price: Some(4),
            },
            KindSpec {
                waits_for_dependencies: false,
                kind: kind(seed, 3),
                form: Form::Set,
                recovery: Recovery::Idempotent,
                price: Some(2),
            },
            KindSpec {
                waits_for_dependencies: false,
                kind: kind(seed, 4),
                form: Form::Transition,
                recovery: Recovery::Unrecoverable,
                price: Some(1),
            },
        ]),
        requirements: Box::from([
            RequirementSpec { number: kind(seed, 1), guarded: true, freshness: Duration::from_nanos(4) },
            RequirementSpec { number: kind(seed, 2), guarded: false, freshness: Duration::from_nanos(4) },
        ]),
        procedures: Box::from([ProcedureSpec {
            number: kind(seed, 1),
            actions: Box::from([
                ProcedureAction::Effect { kind: kind(seed, 1), purpose: 8, target: 5 },
                ProcedureAction::Delegate { kinds: Box::from([kind(seed, 2)]) },
                ProcedureAction::Propose { spend: 7 },
                ProcedureAction::Wait { state: 5 },
                ProcedureAction::Finish { result: ProcedureResult::Local { code: 9 } },
            ]),
            max_steps: 5,
            stall: Duration::from_nanos(9),
        }]),
    }
}
