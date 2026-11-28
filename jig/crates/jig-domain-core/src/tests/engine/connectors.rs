//! Connector port decisions and the bridge (jig/docs/design/domain/engine/contracts.md, 1 and 3).

use super::*;
use crate::{Ask, Held, effects::connector, engine::Peer};
use jig_domain_contract::{
    self as contract, Connector, Declaration, Released, Up, effects, lifecycle, parts, procedures, projections,
    requests, resources, settings, topics,
};
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_domain_views as views;
use skein_lib::{Journal, Released as JournalRelease};

const DECLARER: Connector = Connector(1);
const EMPTY: Connector = Connector(2);
const OWNER: Token = Token::new(31);
const TASK: u64 = 1;
const ENTRY: u64 = 7;
const KEY: [u8; 16] = [0xf5; 16];
const CALL: crate::CallKey = crate::CallKey { task: TASK, attempt: 1, completion: 1, position: 0 };

fn declarations() -> [Declaration; 2] {
    [
        Declaration::empty(EMPTY),
        Declaration {
            connector: DECLARER,
            resources: Box::new([resources::ResourceKind { kind: 1, hold: resources::Rule::Shared }]),
            effects: Box::new([effects::EffectKind {
                kind: 1,
                form: effects::EffectForm::Set,
                recovery: effects::Recovery::Keyed,
                waits_for_dependencies: false,
            }]),
            projects_goals: true,
        },
    ]
}

fn port_twins() -> (Core, Engine, Env<Limits>) {
    let env = environment();
    let config = config(SEED);
    let mut core = Core::new(config.clone(), &declarations(), &env.limits).expect("two declared connectors");
    let mut engine = Engine::new(config, &declarations(), &env.limits).expect("two declared connectors");
    fixtures::open_core(&mut core, &env);
    fixtures::open_core(engine.core_mut(), &env);
    (core, engine, env)
}

fn compare(core: &Core, engine: &Engine, mut expected: Queue<Request>, mut actual: Queue<Out>) {
    let hash = BuildHasherDefault::<DefaultHasher>::default();
    assert_eq!(expected.len(), actual.len(), "{actual:?}");
    while let Some(request) = expected.pop() {
        let output = actual.pop().expect("parallel decision output");
        let returned = match output {
            Out::Core(request) => request,
            Out::Ask(connector, ask) => engine::bridge_ask(connector, ask),
            Out::Held(Peer::Connector(connector, released)) => engine::bridge_released(connector, released),
            Out::Held(Peer::Store(load)) => Request::Held(Box::new(Held::Store(load))),
        };
        assert_eq!(hash.hash_one(&returned), hash.hash_one(&request), "{returned:?} at the place of {request:?}");
    }
    assert_eq!(hash.hash_one(core), hash.hash_one(engine.core()), "same core state after the decision");
}

fn answer(up: Up, event: Event) {
    let (mut core, mut engine, env) = port_twins();
    let input = In::Connector(DECLARER, up);
    assert_eq!(input.new_work(), event.new_work(), "the connector keeps today's admission class");
    let expected = crate::step(&mut core, &env, event);
    let mut actual = Queue::uncapped();
    engine::step(&mut engine, &env, input, &mut actual);
    compare(&core, &engine, expected, actual);
}

fn ask(old: Ask, expected: contract::Ask) {
    let mut requests = Queue::uncapped();
    requests.push(Request::ProjectMade { project: 1 });
    requests.push(Request::Ask { connector: DECLARER.0, ask: old });
    requests.push(Request::Held(Box::new(Held::Drained)));
    let hash = BuildHasherDefault::<DefaultHasher>::default();
    let old = hash.hash_one(requests.iter().nth(1).expect("ask"));
    let mut out = Queue::uncapped();
    engine::append(requests, &mut out);
    let Some(Out::Core(Request::ProjectMade { project: 1 })) = out.pop() else { panic!("core output first") };
    let Some(Out::Ask(number, actual)) = out.pop() else { panic!("ask crosses by its port") };
    assert_eq!(number, DECLARER);
    assert_eq!(hash.hash_one(&actual), hash.hash_one(&expected), "{actual:?} is {expected:?}");
    assert_eq!(hash.hash_one(engine::bridge_ask(number, actual)), old);
    let Some(Out::Core(Request::Held(held))) = out.pop() else { panic!("held after the ask") };
    let Held::Drained = *held else { panic!("unchanged held") };
    assert!(out.is_empty());
}

#[test]
fn lifecycle_port() {
    let (mut core, mut engine, env) = port_twins();
    // Drive a real start on fresh twins: catch-up still asks both connectors, in order.
    let mut core_start = Core::new(config(SEED), &declarations(), &env.limits).unwrap();
    let mut startup = Queue::uncapped();
    engine::append(crate::tests::finish_start(&mut core_start, &env), &mut startup);
    let mut catch_up = List::with_capacity(2);
    while let Some(out) = startup.pop() {
        if let Out::Ask(number, contract::Ask::Lifecycle(lifecycle::Ask::CatchUp)) = out {
            catch_up.push(number).expect("two configured connectors");
        }
    }
    assert_eq!(catch_up.as_slice(), [DECLARER, EMPTY]);
    ask(Ask::Effect(connector::Ask::CatchUp), contract::Ask::Lifecycle(lifecycle::Ask::CatchUp));
    for stage in [lifecycle::RestartStage::ReadAfresh, lifecycle::RestartStage::Settled] {
        core.catch_up_done(EMPTY.0, stage);
        let input = In::Connector(EMPTY, Up::Lifecycle(lifecycle::Up::Done { stage }));
        assert!(!input.new_work());
        let mut out = Queue::uncapped();
        engine::step(&mut engine, &env, input, &mut out);
        compare(&core, &engine, Queue::uncapped(), out);
    }
    assert!(engine.core().caught_up(EMPTY.0));
    assert!(!engine.core().caught_up(DECLARER.0));
}

#[test]
fn resources_port() {
    ask(
        Ask::Lost { task: TASK, attempt: 1 },
        contract::Ask::Resources(resources::Ask::Lost { task: TASK, attempt: 1 }),
    );
    let resource = tasks::Name { connector: DECLARER.0, path: Box::new([Box::from(b"live".as_slice())]) };
    answer(
        Up::Resources(resources::Up::ReadAfresh { resource: resource.clone() }),
        Event::Tasks(tasks::Event::ReadAfresh { resource: resource.clone() }),
    );
    answer(
        Up::Resources(resources::Up::PoolSlots { name: resource.clone(), slots: 2 }),
        Event::EffectConnector(connector::Event::PoolSlots { name: resource, slots: 2 }),
    );
}

#[test]
fn effects_port() {
    ask(
        Ask::Effect(connector::Ask::Describe { owner: OWNER }),
        contract::Ask::Effects(effects::Ask::Describe { owner: OWNER }),
    );
    answer(Up::Effects(effects::Up::Released { task: TASK }), Event::Tasks(tasks::Event::Settled { task: TASK }));
    let (mut core, mut engine, env) = port_twins();
    let owner = core.effect_owner();
    assert_eq!(engine.core_mut().effect_owner(), owner);
    let origin = || effects::EffectOrigin::Projection { goal: TASK, entry: None };
    let event = Event::EffectStart { owner, connector: DECLARER.0, origin: origin() };
    let expected = crate::step(&mut core, &env, event);
    let mut out = Queue::uncapped();
    let input = In::Connector(DECLARER, Up::Effects(effects::Up::Start { owner, origin: origin() }));
    assert!(input.new_work());
    engine::step(&mut engine, &env, input, &mut out);
    let Some(Out::Ask(number, _)) = out.iter().next() else { panic!("effect owner asked") };
    assert_eq!(*number, DECLARER);
    compare(&core, &engine, expected, out);
}

#[test]
fn topics_port() {
    ask(
        Ask::SubscriptionDone { request: OWNER, key: CALL, subscription: 2 },
        contract::Ask::Topics(topics::Ask::Subscribed { request: OWNER, key: CALL, subscription: 2 }),
    );
    answer(
        Up::Topics(topics::Up::News {
            task: TASK,
            subscription: 2,
            topic: 3,
            class: tasks::NewsClass::Wakes,
            words: Box::new([]),
        }),
        Event::EffectConnector(connector::Event::News {
            task: TASK,
            subscription: 2,
            topic: 3,
            class: tasks::NewsClass::Wakes,
            words: Box::new([]),
        }),
    );
}

#[test]
fn parts_port() {
    ask(Ask::DropDocument { subject: OWNER }, contract::Ask::Parts(parts::Ask::DropDocument { subject: OWNER }));
    answer(
        Up::Parts(parts::Up::Made { watcher: WATCHER, outcome: views::Made::Ready }),
        Event::View(views::Event::Made { watcher: WATCHER, outcome: views::Made::Ready }),
    );
}

#[test]
fn procedures_port() {
    ask(
        Ask::StepMade { task: TASK, step: 5, first: Some(2), revisions: Box::new([(2, 3)]) },
        contract::Ask::Procedures(procedures::Ask::StepMade {
            task: TASK,
            step: 5,
            first: Some(2),
            revisions: Box::new([(2, 3)]),
        }),
    );
    answer(
        Up::Procedures(procedures::Up::Wake { task: TASK }),
        Event::Tasks(tasks::Event::WakeProcedure { task: TASK }),
    );
}

#[test]
fn projections_port() {
    ask(Ask::ForgetProjection { goal: TASK }, contract::Ask::Projections(projections::Ask::Forget { goal: TASK }));
    answer(
        Up::Projections(projections::Up::Settled { goal: TASK }),
        Event::ProjectionSettled { goal: TASK, connector: DECLARER.0 },
    );
}

#[test]
fn settings_port() {
    ask(
        Ask::Effect(connector::Ask::SaveSettingsDefaults { revision: Some(2) }),
        contract::Ask::Settings(settings::Ask::SaveDefaults { revision: Some(2) }),
    );
    let env = environment();
    let mut config = config(SEED);
    config.kinds = Box::new([crate::settings::Kind {
        number: 1,
        scope: crate::settings::KindScope::Deployment,
        connector: DECLARER.0,
        value_bytes: 64,
        per_scope: 1,
        charters: 1,
    }]);
    let mut core = Core::new(config.clone(), &declarations(), &env.limits).unwrap();
    let mut engine = Engine::new(config, &declarations(), &env.limits).unwrap();
    drop(core.start_begin(false, env.wall));
    drop(engine.core_mut().start_begin(false, env.wall));
    let expected = crate::step(
        &mut core,
        &env,
        Event::EffectConnector(connector::Event::SettingsDefaults {
            connector: DECLARER.0,
            defaults: Box::new([]),
            dropped: Box::new([]),
        }),
    );
    let input =
        In::Connector(DECLARER, Up::Settings(settings::Up::Defaults { defaults: Box::new([]), dropped: Box::new([]) }));
    assert!(!input.new_work());
    let mut actual = Queue::uncapped();
    engine::step(&mut engine, &env, input, &mut actual);
    compare(&core, &engine, expected, actual);
}

#[test]
fn requests_port() {
    ask(Ask::DropApplication { owner: OWNER }, contract::Ask::Requests(requests::Ask::Drop { owner: OWNER }));
    answer(
        Up::Requests(requests::Up::Decided { request: OWNER, outcome: Ok(OWNER) }),
        Event::ApplicationFinished { request: OWNER, outcome: Ok(OWNER) },
    );
}

#[test]
fn released_port_round_trips_both_held_requests() {
    let outcome = people::Outcome::Again(people::Again::NotReady);
    let mut requests = Queue::uncapped();
    requests.push(Request::Held(Box::new(Held::MakeEffect { connector: DECLARER.0, entry: ENTRY })));
    requests.push(Request::Held(Box::new(Held::ConnectorActed {
        connector: EMPTY.0,
        key: KEY,
        outcome: outcome.clone(),
    })));
    let mut out = Queue::uncapped();
    engine::append(requests, &mut out);
    let Some(Out::Held(Peer::Connector(number, make))) = out.pop() else { panic!("held make") };
    assert_eq!((number, &make), (DECLARER, &Released::Make { entry: ENTRY }));
    let Request::Held(make) = engine::bridge_released(number, make) else { panic!("held request") };
    let Held::MakeEffect { connector: 1, entry: ENTRY } = *make else { panic!("old make") };
    let Some(Out::Held(Peer::Connector(number, reply))) = out.pop() else { panic!("held act reply") };
    assert_eq!((number, &reply), (EMPTY, &Released::Answered { key: KEY, outcome: outcome.clone() }));
    let Request::Held(reply) = engine::bridge_released(number, reply) else { panic!("held request") };
    let Held::ConnectorActed { connector: 2, key: KEY, outcome: received } = *reply else { panic!("connector reply") };
    assert_eq!(received, outcome);
    assert!(out.is_empty());
}

#[test]
fn a_connectors_close_answer_waits_for_the_commit_and_keeps_the_key() {
    let env = environment();
    let mut cfg = config(SEED);
    cfg.grants = Box::new([people::ActGrant { connector: DECLARER.0, may: Box::new([people::ActKind::Close]) }]);
    let setup = || {
        let mut core = crate::tests::core_with_config(cfg.clone(), &env.limits);
        crate::tests::make_task(&mut core, &env.limits, TASK);
        core
    };
    let mut core = setup();
    let mut engine = Engine::from_core(setup());
    let act =
        || Box::new(people::Act::Close { project: 1, task: TASK, grace: None, reason: Box::from(b"close".as_slice()) });
    let old = crate::step(
        &mut core,
        &env,
        Event::Act { reply_to: ReplyTo::new(OWNER), connector: DECLARER.0, key: KEY, act: act(), carried: None },
    );
    let mut expected = None;
    for request in &old {
        if let Request::Held(held) = request
            && let Held::Acted { outcome, .. } = held.as_ref()
        {
            assert!(expected.replace(outcome.clone()).is_none(), "one act answer");
        }
    }
    let expected = expected.expect("today's act answer");
    if let people::Outcome::Again(_) | people::Outcome::Refused(_) = expected {
        panic!("act was refused: {expected:?}");
    }
    let input = In::Connector(DECLARER, Up::Act { key: KEY, act: act(), carried: None });
    assert!(input.new_work());
    let mut out = Queue::uncapped();
    engine::step(&mut engine, &env, input, &mut out);
    let mut journal = Journal::uncapped(8, 0);
    let mut decision = journal.decide();
    while let Some(output) = out.pop() {
        match output {
            Out::Core(Request::Write(write)) => decision.push_write(write),
            Out::Held(Peer::Connector(number, released)) => decision.push_held((number, released)),
            Out::Core(Request::Held(held)) => {
                if let Held::Acted { .. } = *held {
                    panic!("reply must go by the connector port");
                }
            }
            Out::Core(Request::Ask { .. } | Request::Now(_) | Request::ProjectMade { .. })
            | Out::Ask(..)
            | Out::Held(Peer::Store(_)) => {}
        }
    }
    journal.accept(decision);
    let commit = journal.commit().expect("closing the conversation writes");
    let mut released = Queue::uncapped();
    assert_eq!(journal.release(&mut released), JournalRelease::None);
    journal.committed(commit.number);
    assert_eq!(journal.release(&mut released), JournalRelease::Some);
    assert_eq!(released.pop(), Some((DECLARER, Released::Answered { key: KEY, outcome: expected })));
    assert!(released.is_empty());
}

#[test]
fn uncased_asks_keep_their_core_request_mark() {
    use jig_domain_fleet::{Kinds, Version};
    let hash = BuildHasherDefault::<DefaultHasher>::default();
    let mut requests = Queue::uncapped();
    for ask in [
        Ask::Workers(Box::new([])),
        Ask::Providers(Box::new([])),
        Ask::Catalogue { models: Box::new([]) },
        Ask::Gather { section: OWNER, budget: 3 },
        Ask::CutTo { section: OWNER, size: 2 },
        Ask::Drop { section: OWNER },
        Ask::ClaimDone { task: TASK, attempt: 1, kinds: Kinds::Both, version: Version { charter: 1, revision: 2 } },
        Ask::JobClaimed {
            task: TASK,
            attempt: 1,
            kind: jig_domain_link::JobKind(1),
            spec: Box::new([]),
            deadline: skein_lib::Duration::ZERO,
            result: 0,
            grants: Box::new([]),
            workspace: None,
        },
    ] {
        requests.push(Request::Ask { connector: EMPTY.0, ask });
    }
    let mut expected = Queue::uncapped();
    for request in &requests {
        expected.push(hash.hash_one(request));
    }
    let mut out = Queue::uncapped();
    engine::append(requests, &mut out);
    while let Some(request) = expected.pop() {
        let Some(Out::Core(wrapped)) = out.pop() else { panic!("uncased ask stays on the core port") };
        assert_eq!(hash.hash_one(wrapped), request);
    }
    assert!(out.is_empty());
}

#[test]
fn terminal_notifications_only_reach_their_declarer() {
    let (_, mut engine, env) = port_twins();
    let token = engine.core_mut().internal(crate::routing::flight::Internal::Acknowledge);
    let mut tasks = Queue::uncapped();
    tasks.push(tasks::Request::Acknowledged {
        reply_to: ReplyTo::new(token),
        task: TASK,
        attempt: 1,
        accepted: tasks::Accepted::New,
        parked: false,
    });
    tasks.push(tasks::Request::ResolveUncertain { task: TASK, entry: ENTRY, again: true });
    let mut requests = Queue::uncapped();
    crate::routing::start_task_outputs(engine.core_mut(), &env, tasks, &mut requests);
    let mut out = Queue::uncapped();
    engine::append(requests, &mut out);
    let Some(Out::Core(Request::Held(held))) = out.pop() else { panic!("terminal acknowledgement") };
    let Held::TaskTerminalAcknowledged { task: TASK, attempt: 1 } = *held else { panic!("fenced attempt") };
    let Some(Out::Ask(DECLARER, contract::Ask::Resources(resources::Ask::Lost { task: TASK, attempt: 1 }))) = out.pop()
    else {
        panic!("loss belongs to resource declarers")
    };
    let Some(Out::Ask(DECLARER, contract::Ask::Effects(effects::Ask::Resolve { entry: ENTRY, again: true }))) =
        out.pop()
    else {
        panic!("resolution belongs to effect declarers")
    };
    assert!(out.is_empty(), "the empty declaration received no asks");
}

#[test]
fn a_topic_notifies_only_its_connector() {
    for (remove, accepted) in [(false, true), (true, true), (false, false), (true, false)] {
        let (_, mut engine, env) = port_twins();
        let core = engine.core_mut();
        assert!(core.reserve_call(CALL, crate::CallHolder::Routed));
        let route = if remove {
            crate::RoutedCall::Unsubscribe { key: CALL, connector: Some(EMPTY.0) }
        } else {
            crate::RoutedCall::Subscribe { key: CALL, subscription: 2, connector: Some(EMPTY.0) }
        };
        let reply_to = ReplyTo::new(core.fly_call(OWNER, route));
        let mut child = Queue::uncapped();
        child.push(if accepted {
            tasks::Request::Done { reply_to }
        } else {
            tasks::Request::Refused { reply_to, problem: tasks::Problem::new(Some(TASK), tasks::Refusal::Subscription) }
        });
        let mut requests = Queue::uncapped();
        crate::routing::start_task_outputs(core, &env, child, &mut requests);
        let mut out = Queue::uncapped();
        engine::append(requests, &mut out);
        let mut found = 0_u32;
        while let Some(output) = out.pop() {
            if let Out::Ask(number, contract::Ask::Topics(ask)) = output {
                assert_eq!(number, EMPTY, "ownership comes from the topic, not capability declarations");
                match ask {
                    topics::Ask::Subscribed { request: OWNER, key: CALL, subscription: 2 } => {
                        assert!(accepted && !remove);
                    }
                    topics::Ask::Unsubscribed { request: OWNER, key: CALL } => assert!(accepted && remove),
                    topics::Ask::Refused { request: OWNER } => assert!(!accepted),
                    other @ (topics::Ask::Subscribed { .. }
                    | topics::Ask::Unsubscribed { .. }
                    | topics::Ask::Refused { .. }
                    | topics::Ask::Ended { .. }) => panic!("wrong topic ask: {other:?}"),
                }
                found += 1;
            }
        }
        assert_eq!(found, 1);
    }
}
