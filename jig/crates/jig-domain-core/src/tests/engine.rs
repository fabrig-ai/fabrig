//! Pass-through decisions and timer admission (`jig/docs/design/domain/engine/vocabulary.md`, 2).

use crate::{
    Config, Core, Event, Limits, Problem, Request, Timer,
    engine::{self, Admit, Engine, In, Out},
    tests::{config, fixtures, limits},
};
use core::hash::{BuildHasher, BuildHasherDefault};
use skein_lib::{Env, Queue, ReplyTo, Time, Token, Wall};
use std::hash::DefaultHasher;

const SEED: u64 = 41;
const WATCHER: Token = Token::new(900);

fn twins(config: &Config, limits: &Limits) -> (Core, Engine) {
    let core = Core::new(config.clone(), limits).expect("fixture core configuration");
    let engine = Engine::new(config.clone(), limits).expect("fixture engine configuration");
    (core, engine)
}

fn environment() -> Env<Limits> {
    Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits() }
}

fn fresh_engine(limits: &Limits) -> Engine {
    Engine::new(config(SEED), limits).expect("fixture engine configuration")
}

fn started_twins() -> (Core, Engine, Env<Limits>) {
    let env = environment();
    let (mut core, mut engine) = twins(&config(SEED), &env.limits);
    fixtures::open_core(&mut core, &env);
    fixtures::open_core(engine.core_mut(), &env);
    (core, engine, env)
}

fn sign_in() -> Event {
    const REPLY: Token = Token::new(60);
    Event::SignIn { until: None, held: None, reply_to: ReplyTo::new(REPLY), identity: fixtures::newcomer() }
}

fn unwatch() -> Event {
    Event::Unwatch { watcher: WATCHER }
}

fn same_decision(core: &Core, engine: &Engine, requests: &Queue<Request>, out: &Queue<Out>) {
    let hash = BuildHasherDefault::<DefaultHasher>::default();
    assert_eq!(requests.len(), out.len(), "wrapped decision length: {out:?}");
    for (request, Out::Core(wrapped)) in requests.iter().zip(out) {
        assert_eq!(hash.hash_one(request), hash.hash_one(wrapped), "{wrapped:?} differs from {request:?}");
    }
    assert_eq!(hash.hash_one(core), hash.hash_one(engine.core()), "core state after the decision");
}

#[test]
fn twins_pass_through_each_decision_in_order() {
    let (mut core, mut engine, env) = started_twins();
    // Sign-in produces writes and a held reply in the same decision.
    for (event, input) in [(sign_in(), sign_in()), (unwatch(), unwatch())] {
        let requests = crate::step(&mut core, &env, event);
        let mut out = Queue::uncapped();
        engine::step(&mut engine, &env, In::Core(input), &mut out);
        same_decision(&core, &engine, &requests, &out);
    }
}

#[test]
fn step_keeps_the_existing_output_at_the_front() {
    const PROJECT: u32 = 1;
    let env = environment();
    let mut engine = fresh_engine(&env.limits);
    fixtures::open_core(engine.core_mut(), &env);
    let mut out = Queue::uncapped();
    out.push(Out::Core(Request::ProjectMade { project: PROJECT }));
    engine::step(&mut engine, &env, In::Core(unwatch()), &mut out);
    let Some(Out::Core(Request::ProjectMade { project })) = out.pop() else { panic!("existing output first") };
    assert_eq!(project, PROJECT);
}

fn arm_timer(core: &mut Core, env: &Env<Limits>) {
    // Put the initial retention pass in flight, so its deadline cannot hide the task alarm.
    drop(crate::fire(core, env, Timer::Retention));
    fixtures::arm_task_alarm(core, &env.limits);
    drop(crate::step(core, env, sign_in()));
}

fn timer_twins() -> (Core, Engine, Env<Limits>) {
    let (mut core, mut engine, env) = started_twins();
    arm_timer(&mut core, &env);
    arm_timer(engine.core_mut(), &env);
    (core, engine, env)
}

#[test]
fn timer_queries_keep_the_cores_admission_and_clocks() {
    let (core, engine, env) = timer_twins();
    let alarm = Time::from_nanos(fixtures::TASK_ALARM_AT.as_nanos());
    let expiry = Wall::from_nanos(env.limits.people.sign_in_lifetime.as_nanos());
    for (now, wall, selected) in [
        (env.now, env.wall, None),
        (alarm, fixtures::TASK_ALARM_AT, Some(Timer::Tasks)),
        (env.now, expiry, Some(Timer::People)),
    ] {
        let all = core.due(now, wall);
        assert_eq!(all, selected);
        assert_eq!(engine::due(&engine, now, wall, Admit::All), all);
        assert_eq!(engine::due(&engine, now, wall, Admit::Finishing), core.finishing_due(now, wall));
        assert_eq!(engine::next_deadline(&engine, now, wall, Admit::All), core.next_deadline(now, wall));
        assert_eq!(
            engine::next_deadline(&engine, now, wall, Admit::Finishing),
            core.next_finishing_deadline(now, wall)
        );
    }
}

#[test]
fn firing_a_due_timer_passes_through_one_decision() {
    let (mut core, mut engine, mut env) = timer_twins();
    env.wall = fixtures::TASK_ALARM_AT;
    env.now = Time::from_nanos(env.wall.as_nanos());
    let requests = crate::fire(&mut core, &env, Timer::Tasks);
    let mut out = Queue::uncapped();
    engine::fire(&mut engine, &env, Timer::Tasks, &mut out);
    same_decision(&core, &engine, &requests, &out);
}

#[test]
fn idle_keeps_core_work_but_excludes_the_journal() {
    let mut engine = fresh_engine(&limits());
    assert_eq!(engine::idle(&engine), engine.core().quiescent(true), "fresh engine");
    let _header = engine.core_mut().decision_header(false);
    assert!(engine::idle(&engine));
    assert!(!engine.core().quiescent(false));
    let _owner = engine.core_mut().effect_owner();
    assert_eq!(engine::idle(&engine), engine.core().quiescent(true), "effect staging is in flight");
}

#[test]
fn construction_preserves_the_core_membership_refusal() {
    let config = config(SEED);
    let mut over = limits();
    // One place fewer than the fixture's configured worker count.
    let limit = u32::try_from(config.defaults.workers.len() - 1).expect("fixture worker count");
    over.fleet.configured_workers = limit;
    let problem = Core::new(config.clone(), &over).expect_err("worker membership over its bound");
    assert_eq!(problem, Problem::Workers { limit });
    assert_eq!(Engine::new(config, &over).expect_err("same worker membership refusal"), problem);
}

#[test]
fn construction_at_the_bound_preserves_the_core_state() {
    let config = config(SEED);
    let mut at = limits();
    at.fleet.configured_workers = u32::try_from(config.defaults.workers.len()).expect("fixture worker count");
    let (core, engine) = twins(&config, &at);
    let hash = BuildHasherDefault::<DefaultHasher>::default();
    assert_eq!(hash.hash_one(engine.core()), hash.hash_one(core));
}

#[test]
fn mutable_access_updates_the_shared_core() {
    let mut engine = fresh_engine(&limits());
    let header = engine.core_mut().counters.next_commit();
    assert_eq!(engine.core().counters.deployment(), header);
}
