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
    for (request, output) in requests.iter().zip(out) {
        if let Request::Held(held) = request
            && let crate::Held::Store(load) = held.as_ref()
        {
            let Out::Held(engine::Peer::Store(wrapped)) = output else { panic!("one held store load: {output:?}") };
            assert_eq!(hash.hash_one(load), hash.hash_one(wrapped), "{wrapped:?} differs from {load:?}");
        } else {
            let Out::Core(wrapped) = output else { panic!("one core request: {output:?}") };
            assert_eq!(hash.hash_one(request), hash.hash_one(wrapped), "{wrapped:?} differs from {request:?}");
        }
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

const READ_REPLY: ReplyTo = ReplyTo::new(Token::new(901));
const RETAINED: u64 = 99;

fn retained_fixture() -> (Engine, Env<Limits>) {
    let (core, limits) = super::watch_core();
    (Engine::from_core(core), Env { now: Time::ZERO, wall: Wall::EPOCH, limits })
}

fn read_input() -> In {
    In::Core(Event::Read { reply: READ_REPLY, sign_in: 10, read: crate::reads::Read::Task { task: RETAINED } })
}

fn read_load(out: &mut Queue<Out>) -> Token {
    let Some(Out::Held(engine::Peer::Store(engine::store::Out::Load {
        load,
        range: engine::store::Range::Read(range),
    }))) = out.pop()
    else {
        panic!("one held store load: {out:?}")
    };
    assert!(out.is_empty(), "there is no duplicate core request");
    assert_eq!(range.scope, crate::reads::Scope::Task { task: RETAINED });
    load
}

fn store_answer(
    engine: &mut Engine,
    env: &Env<Limits>,
    load: Token,
    result: engine::store::Answer,
    out: &mut Queue<Out>,
) {
    engine::step(engine, env, In::Store(engine::store::In::Loaded { load, result }), out);
}

fn read_answer(out: &mut Queue<Out>) -> crate::reads::Answer {
    let Some(Out::Core(Request::Held(held))) = out.pop() else { panic!("one held read answer: {out:?}") };
    let crate::Held::Read { reply, answer } = *held else { panic!("party answer: {held:?}") };
    assert_eq!(reply, READ_REPLY);
    assert!(out.is_empty());
    answer
}

#[test]
fn the_store_port_round_trips_a_retained_task_read() {
    use jig_domain_tasks as tasks;
    let (mut engine, env) = retained_fixture();
    let mut out = Queue::uncapped();
    out.push(Out::Core(Request::ProjectMade { project: 1 }));
    engine::step(&mut engine, &env, read_input(), &mut out);
    let Some(Out::Core(Request::ProjectMade { project: 1 })) = out.pop() else { panic!("previous output stays first") };
    let load = read_load(&mut out);

    let mut task = engine.core().tasks.task(1).expect("task fixture").clone();
    task.number = RETAINED;
    task.root = RETAINED;
    task.phase = tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::Report { words: Box::new([]) }));
    let row = super::stored_row(crate::Record::Tasks(tasks::Stored::Ended(Box::new(task))), None);
    store_answer(
        &mut engine,
        &env,
        load,
        engine::store::Answer::Rows(engine::store::Rows::Read { rows: Box::new([row]), next: None }),
        &mut out,
    );
    let crate::reads::Answer::Ended(ended) = read_answer(&mut out) else { panic!("retained task document") };
    let crate::reads::EndedValue::Task { document, .. } = ended.value else { panic!("task document") };
    assert_eq!(document.chip.number, RETAINED);
}

#[test]
fn a_store_terminal_without_a_live_load_changes_nothing() {
    use engine::store::{Answer, Failure};
    let (mut engine, mut env) = retained_fixture();
    let mut out = Queue::uncapped();
    engine::step(&mut engine, &env, read_input(), &mut out);
    let old = read_load(&mut out);
    store_answer(&mut engine, &env, old, Answer::Failed(Failure::Gone), &mut out);
    assert_eq!(read_answer(&mut out), crate::reads::Answer::None(crate::reads::Why::Gone));
    engine.core_mut().reclaim();
    engine::step(&mut engine, &env, read_input(), &mut out);
    let current = read_load(&mut out);
    assert_ne!(old, current, "a load token is never reused");

    let hash = BuildHasherDefault::<DefaultHasher>::default();
    let before = hash.hash_one(engine.core());
    env.now = Time::from_nanos(1);
    for name in [old, Token::new(u64::MAX)] {
        store_answer(&mut engine, &env, name, Answer::Failed(Failure::Busy), &mut out);
        assert!(out.is_empty());
        assert_eq!(hash.hash_one(engine.core()), before, "an unowned terminal does not open a decision");
    }
    store_answer(&mut engine, &env, current, Answer::Failed(Failure::Gone), &mut out);
    assert_eq!(read_answer(&mut out), crate::reads::Answer::None(crate::reads::Why::Gone));
}

#[test]
fn a_failed_or_wrong_shaped_store_answer_ends_a_read_as_busy() {
    use engine::store::{Answer, Failure, Rows};
    let (mut engine, env) = retained_fixture();
    let mut out = Queue::uncapped();
    let turn = crate::TurnRecord {
        task: RETAINED,
        attempt: 1,
        turn: 1,
        spent: 0,
        opens_window: true,
        read: None,
        at: Wall::EPOCH,
        transcript: Box::new([]),
    };
    for result in [
        Answer::Failed(Failure::Failed),
        Answer::Rows(Rows::Records { records: Box::new([]), more: false }),
        Answer::Rows(Rows::Turns { turns: Box::new([]), more: true }),
        Answer::Cut { turn, bytes: 1, more: true },
        Answer::Failed(Failure::Unread),
    ] {
        engine::step(&mut engine, &env, read_input(), &mut out);
        let load = read_load(&mut out);
        store_answer(&mut engine, &env, load, result, &mut out);
        assert_eq!(read_answer(&mut out), crate::reads::Answer::None(crate::reads::Why::Busy));
        engine.core_mut().reclaim();
    }
}
