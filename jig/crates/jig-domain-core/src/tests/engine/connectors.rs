//! Connector port decisions (jig/docs/design/domain/engine/vocabulary.md, 2.5).

use super::*;
use crate::{Ask, Held, engine::Peer};
use jig_domain_authority as authority;
use jig_domain_contract::{
    self as contract, Connector, Declaration, Released, Up, effects, lifecycle, parts, procedures, resources, topics,
};
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{Journal, Released as JournalRelease};

const DECLARER: Connector = Connector(1);
const EMPTY: Connector = Connector(2);
const OWNER: Token = Token::new(31);
const TASK: u64 = 1;
const TOPIC: u64 = 3;
const SUBSCRIPTION: u64 = 2; // One message-family number is already used by the subscribing run fixture.
const ENTRY: u64 = 7;
const PERSON: u64 = 1;
const PROJECT: u32 = 1;
const WATCHED: &[u8] = b"watched";
const EFFECT_KIND: u16 = 1;
const KEY: [u8; 16] = [0xf5; 16];
const CALL: crate::CallKey = crate::CallKey { task: TASK, attempt: 1, completion: 1, position: 0 };

fn declarations() -> [Declaration; 2] {
    [
        Declaration::empty(EMPTY),
        Declaration {
            connector: DECLARER,
            resources: Box::new([resources::ResourceKind { kind: 1, hold: resources::Rule::Shared }]),
            effects: Box::new([effects::EffectKind {
                kind: EFFECT_KIND,
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

fn compare(core: &Core, engine: &Engine, expected: Queue<Request>, actual: Queue<Out>) {
    let mut expected_out = Queue::uncapped();
    engine::append(expected, &mut expected_out);
    let hash = BuildHasherDefault::<DefaultHasher>::default();
    assert_eq!(hash.hash_one(&expected_out), hash.hash_one(&actual), "{expected_out:?} differs from {actual:?}");
    assert_eq!(hash.hash_one(core), hash.hash_one(engine.core()), "same core state after the decision");
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

fn open_period(core: &mut Core, env: &Env<Limits>, project: u32) -> tasks::Funder {
    crate::tests::send_task(
        core,
        &env.limits,
        tasks::Event::OpenPeriod { reply_to: ReplyTo::new(OWNER), project, period: 1, budget: 1000 },
    );
    tasks::Funder::Period { project, period: 1 }
}

#[test]
fn a_description_is_outstanding_until_its_owner_answers() {
    use crate::judgements::Subject;
    let (_, mut engine, env) = port_twins();
    let core = engine.core_mut();
    let funder = open_period(core, &env, PROJECT);
    let goal = crate::tests::new_row(core, TASK, funder, Some(1));
    fixtures::make_batch(core, &env.limits, tasks::Party::Person(1), Box::new([goal]));
    assert!(core.projections.contains_key(&TASK));
    drop(crate::finish_decision(core, &env));
    let owner = core.effect_owner();
    let start = Up::Effects(effects::Up::Start {
        owner,
        origin: effects::EffectOrigin::Projection { goal: TASK, entry: None },
    });
    let mut out = Queue::uncapped();
    engine::step(&mut engine, &env, In::Connector(DECLARER, start), &mut out);
    let Some(Out::Ask(DECLARER, contract::Ask::Effects(effects::Ask::Describe { owner: asked }))) = out.pop() else {
        panic!("effect owner asked")
    };
    assert_eq!(asked, owner);
    assert!(out.is_empty());

    engine::step(&mut engine, &env, In::Connector(DECLARER, Up::Effects(effects::Up::Busy { owner: OWNER })), &mut out);
    assert!(out.is_empty(), "an unasked answer follows its normal route");
    // Pin the invariant without invoking the fail-stop on an incomplete decision.
    assert_eq!(engine.core().judgements.outstanding(), Some((DECLARER.0, Subject::Describe(owner))));

    engine::step(&mut engine, &env, In::Connector(DECLARER, Up::Effects(effects::Up::Busy { owner })), &mut out);
    assert!(engine.core().judgements.outstanding().is_none());
    drop(crate::finish_decision(engine.core_mut(), &env));
}

#[test]
fn a_lines_operation_can_end_in_a_later_decision() {
    let (mut core, _, env) = port_twins();
    let mut asks = Queue::uncapped();
    asks.push(crate::vocabulary::connector_ask(
        DECLARER.0,
        contract::Ask::Parts(parts::Ask::Lines(parts::PartsAsk {
            token: OWNER,
            party: PERSON,
            kinds: List::with_capacity(0),
            rows: List::with_capacity(0),
        })),
    ));
    core.judgements.issued(&asks);
    assert_eq!(core.judgements.outstanding(), None, "a pending lines operation is not a judgement");
    drop(crate::finish_decision(&mut core, &env));
}

#[test]
fn a_procedure_start_names_only_a_direct_person_conversation() {
    const REQUESTER: u64 = 2;
    const CODE: u32 = 1;
    const FIRST_STEP: u64 = 1;
    for (parent_conversation, expected) in [(Some(true), Some(REQUESTER)), (Some(false), None), (None, None)] {
        let (_, mut engine, env) = port_twins();
        let core = engine.core_mut();
        let period = open_period(core, &env, PROJECT);
        let (creator, funder) = match parent_conversation {
            Some(conversation) => {
                let mut parent = crate::tests::new_row(core, REQUESTER, period, None);
                parent.flags.conversation = conversation;
                fixtures::make_batch(core, &env.limits, tasks::Party::Person(PERSON), Box::new([parent]));
                (tasks::Party::Task(REQUESTER), tasks::Funder::Task(REQUESTER))
            }
            None => (tasks::Party::Person(PERSON), period),
        };
        let mut procedure = crate::tests::new_row(core, TASK, funder, None);
        procedure.executor = tasks::Executor::Procedure { connector: DECLARER.0, code: CODE };
        fixtures::make_batch(core, &env.limits, creator, Box::new([procedure]));
        let context = core.tasks.run_context(TASK).expect("procedure ready to activate");
        let mut out = Queue::uncapped();
        engine::step(&mut engine, &env, In::Core(Event::Activate { context, ready: true }), &mut out);
        let mut start = None;
        while let Some(output) = out.pop() {
            if let Out::Ask(DECLARER, contract::Ask::Procedures(ask @ procedures::Ask::Start { .. })) = output {
                assert!(start.replace(ask).is_none(), "one procedure start");
            }
        }
        let start = start.expect("start of the assigned procedure");
        let procedures::Ask::Start { context, step: FIRST_STEP, chat } = &start else {
            panic!("first step of the procedure: {start:?}");
        };
        assert_eq!(context.task, TASK);
        assert_eq!(*chat, expected, "requester's conversation flag: {parent_conversation:?}");
    }
}

#[test]
fn a_kept_effect_names_its_tasks_project() {
    const OTHER_PROJECT: u32 = 3;
    const TRACKED: u32 = 1;
    let (mut engine, env) = engine_with_effect_grant(OTHER_PROJECT);
    let core = engine.core_mut();
    let funder = open_period(core, &env, OTHER_PROJECT);
    let mut goal = crate::tests::new_row(core, TASK, funder, Some(TRACKED));
    goal.project = OTHER_PROJECT;
    fixtures::make_batch(core, &env.limits, tasks::Party::Person(PERSON), Box::new([goal]));
    let owner = core.effect_owner();
    let mut out = Queue::uncapped();
    let start = effects::Up::Start { owner, origin: effects::EffectOrigin::Projection { goal: TASK, entry: None } };
    engine::step(&mut engine, &env, In::Connector(DECLARER, Up::Effects(start)), &mut out);
    let description = Box::new(effects::EffectDescription { connector: DECLARER.0, purpose: ENTRY, effect: effect() });
    out = Queue::uncapped();
    engine::step(
        &mut engine,
        &env,
        In::Connector(DECLARER, Up::Effects(effects::Up::Described { owner, description })),
        &mut out,
    );
    let mut kept = None;
    for output in &out {
        if let Out::Ask(DECLARER, contract::Ask::Effects(keep @ effects::Ask::Keep { task: TASK, .. })) = output {
            assert!(kept.replace(keep).is_none(), "one effect kept");
        }
    }
    let Some(effects::Ask::Keep { owner: kept_owner, project, .. }) = kept else {
        panic!("keep the effect: {out:?}");
    };
    assert_eq!(*kept_owner, owner);
    assert_eq!(*project, Some(OTHER_PROJECT));
    drop(crate::finish_decision(engine.core_mut(), &env));
}

fn engine_with_effect_grant(project: u32) -> (Engine, Env<Limits>) {
    const PROJECTS_WITH_EFFECT: u32 = 2; // The default project and the effect's project.
    let mut env = environment();
    let mut config = config(SEED);
    let grant = authority::Grant {
        connector: DECLARER.0,
        kind: EFFECT_KIND,
        pattern: authority::Pattern { segments: Box::new([]), last: authority::Last::Exact(WATCHED.into()) },
    };
    config.defaults.rules.ceiling.grants = Box::new([grant.clone()]);
    let seed = &mut config.seed_projects[0];
    seed.policy.policy.ceiling.grants = Box::new([grant.clone()]);
    seed.policy.policy.roles[0].authority.grants = Box::new([grant.clone()]);
    seed.policy.policy.projections = Box::new([grant]);
    if project != PROJECT {
        env.limits.authority.projects = PROJECTS_WITH_EFFECT;
        env.limits.people.projects = PROJECTS_WITH_EFFECT + 1; // A separate roster for the deployment.
        let other =
            crate::SeedProject { number: project, name: Box::from("Another project"), policy: seed.policy.clone() };
        config.seed_projects = Box::new([config.seed_projects[0].clone(), other]);
    }
    let mut engine = Engine::new(config, &declarations(), &env.limits).expect("connector with a grant");
    fixtures::open_core(engine.core_mut(), &env);
    (engine, env)
}

fn effect() -> authority::Effect {
    authority::Effect {
        connector: DECLARER.0,
        kind: EFFECT_KIND,
        name: authority::Name { segments: Box::new([WATCHED.into()]) },
        state: [0; 32],
        price: None,
        access: authority::EffectAccess::Owned,
        additional: Box::new([]),
        guards: Box::new([]),
    }
}

#[test]
fn held_connector_outputs_reach_the_port_in_order() {
    let outcome = people::Outcome::Again(people::Again::NotReady);
    let mut requests = Queue::uncapped();
    requests.push(Request::Held(Box::new(Held::Connector(DECLARER, Released::Make { entry: ENTRY }))));
    requests.push(Request::Held(Box::new(Held::Connector(
        EMPTY,
        Released::Answered { key: KEY, outcome: outcome.clone() },
    ))));
    let mut out = Queue::uncapped();
    engine::append(requests, &mut out);
    let Some(Out::Held(Peer::Connector(number, make))) = out.pop() else { panic!("held make") };
    assert_eq!((number, &make), (DECLARER, &Released::Make { entry: ENTRY }));
    let Some(Out::Held(Peer::Connector(number, reply))) = out.pop() else { panic!("held act reply") };
    assert_eq!((number, &reply), (EMPTY, &Released::Answered { key: KEY, outcome: outcome.clone() }));
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
            Out::Write(write) => decision.push_write(write),
            Out::Held(Peer::Connector(number, released)) => decision.push_held((number, released)),
            Out::Core(Request::Held(held)) => {
                if let Held::Acted { .. } = *held {
                    panic!("reply must go by the connector port");
                }
            }
            Out::Core(Request::Ask { .. } | Request::Now(_) | Request::ProjectMade { .. })
            | Out::Ask(..)
            | Out::Held(Peer::Store(_)) => {}
            Out::Core(Request::Write(_)) => panic!("writes use the store port"),
            Out::Now(_) | Out::Held(Peer::Application(_) | Peer::Resume(_)) => panic!("no start is driven"),
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
    let Some(Out::Ask(DECLARER, contract::Ask::Resources(resources::Ask::Lost { task: TASK, attempt: 1, .. }))) =
        out.pop()
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
fn a_lost_attempt_names_only_each_declarers_own_writes() {
    const ATTEMPT: u64 = 1;
    let env = environment();
    let mut declarations = declarations();
    let taken = resources::Taken::Waits;
    for declaration in &mut declarations {
        declaration.resources =
            Box::new([resources::ResourceKind { kind: 1, hold: resources::Rule::Exclusive { taken } }]);
    }
    let mut engine = Engine::new(config(SEED), &declarations, &env.limits).expect("two resource declarers");
    let core = engine.core_mut();
    fixtures::open_core(core, &env);
    let names = [DECLARER, EMPTY]
        .map(|Connector(connector)| tasks::Name { connector, path: Box::new([Box::from(b"held".as_slice())]) });
    for name in &names {
        crate::tests::send_task(
            core,
            &env.limits,
            tasks::Event::Resource { name: name.clone(), hold: tasks::HoldKind::Exclusive { taken } },
        );
    }
    let reply = || ReplyTo::new(OWNER);
    let funder = open_period(core, &env, PROJECT);
    let mut run = crate::tests::new_row(core, TASK, funder, None);
    run.holdings = names.clone().map(|resource| tasks::Holding::Write { resource, kind: 1 }).into();
    fixtures::make_batch(core, &env.limits, tasks::Party::Person(1), Box::new([run]));
    crate::tests::send_task(core, &env.limits, tasks::Event::Prepare { reply_to: reply(), task: TASK });
    crate::tests::send_task(
        core,
        &env.limits,
        tasks::Event::Claim {
            reply_to: reply(),
            fresh: None,
            session_end: None,
            offered: tasks::Reasons::ALL,
            reasons: tasks::Reasons::FIRST,
            task: TASK,
            attempt: ATTEMPT,
            account: None,
            resumed: false,
            budget: 10,
            writes: names.clone().into(),
        },
    );
    crate::tests::send_task(core, &env.limits, tasks::Event::Started { task: TASK, attempt: ATTEMPT });
    for name in &names {
        assert!(core.tasks.writer(name).is_some(), "the run took the write slot for {name:?}");
    }

    let mut out = Queue::uncapped();
    engine::step(
        &mut engine,
        &env,
        In::Core(Event::Tasks(tasks::Event::Activation {
            reply_to: reply(),
            reads_project: false,
            task: TASK,
            attempt: ATTEMPT,
            end: tasks::End::Failed(tasks::Class::Lost),
            cause: tasks::Cause::Unpriced,
            saved: None,
        })),
        &mut out,
    );
    let mut losses = List::with_capacity(2);
    while let Some(output) = out.pop() {
        if let Out::Ask(
            connector,
            contract::Ask::Resources(resources::Ask::Lost { task: TASK, attempt: ATTEMPT, resources }),
        ) = output
        {
            losses.push((connector, resources)).expect("one loss per declarer");
        }
    }
    let [first, second] = names;
    assert_eq!(
        losses.as_slice(),
        [(DECLARER, Box::<[tasks::Name]>::from([first])), (EMPTY, Box::<[tasks::Name]>::from([second]))]
    );
    drop(crate::finish_decision(engine.core_mut(), &env));
}

fn engine_with_subscribing_run() -> (Engine, Env<Limits>) {
    let (mut engine, env) = engine_with_effect_grant(PROJECT);
    let core = engine.core_mut();
    crate::tests::make_task(core, &env.limits, TASK);
    let proof = crate::RunProof::claimed(TASK, CALL.attempt, TASK, None, jig_domain_fleet::HostKind::Worker);
    core.proofs.insert(TASK, proof).expect("proof for the subscribing run");
    let _previous_message = crate::fresh(&mut core.counters, crate::Family::Message).expect("one prior message");
    (engine, env)
}

#[test]
fn the_core_settles_subscriptions_after_any_connector_notification() {
    const UNSUBSCRIBE: crate::CallKey = crate::CallKey { position: 1, ..CALL };
    let subscribe = |kind| Event::NamedAction {
        to: ReplyTo::new(OWNER),
        key: CALL,
        action: crate::NamedAction::Subscribe { kind },
    };
    for (call, subscribed, unsubscribed) in [
        (
            Event::SubscribeTopic {
                to: ReplyTo::new(OWNER),
                key: CALL,
                connector: DECLARER.0,
                topic: TOPIC,
                resources: Box::new([effect()]),
            },
            Some(topics::Ask::Subscribed { request: OWNER, key: CALL, subscription: SUBSCRIPTION, topic: TOPIC }),
            Some(topics::Ask::Unsubscribed {
                request: OWNER,
                key: UNSUBSCRIBE,
                subscription: SUBSCRIPTION,
                topic: TOPIC,
            }),
        ),
        (subscribe(tasks::SubscriptionKind::Timer { at: fixtures::TASK_ALARM_AT, period: None }), None, None),
    ] {
        let (mut engine, env) = engine_with_subscribing_run();
        let unsubscribe = Event::NamedAction {
            to: ReplyTo::new(OWNER),
            key: UNSUBSCRIBE,
            action: crate::NamedAction::Unsubscribe { subscription: SUBSCRIPTION },
        };
        for (call, key, ask, part) in [
            (call, CALL, subscribed, crate::CallPart::Subscribed { subscription: SUBSCRIPTION }),
            (unsubscribe, UNSUBSCRIBE, unsubscribed, crate::CallPart::Unsubscribed),
        ] {
            let mut out = Queue::uncapped();
            engine::step(&mut engine, &env, In::Core(call), &mut out);
            assert_subscription_answer(&out, key, ask, &part);
            assert!(!engine.core().pending_calls.contains_key(&key));
            drop(crate::finish_decision(engine.core_mut(), &env));
        }
    }
}

#[test]
fn an_accepted_named_subscribe_notifies_only_for_a_topic() {
    use crate::routing::{Work, run::calls::named_action};
    const WATCHED_TASK: u64 = 2;
    for (kind, ask) in [
        (
            tasks::SubscriptionKind::Topic { connector: DECLARER.0, topic: TOPIC },
            Some(topics::Ask::Subscribed { request: OWNER, key: CALL, subscription: SUBSCRIPTION, topic: TOPIC }),
        ),
        (tasks::SubscriptionKind::Task { target: WATCHED_TASK, held: true, result: false }, None),
    ] {
        let (mut engine, env) = engine_with_subscribing_run();
        let core = engine.core_mut();
        let mut work = Queue::uncapped();
        let requests =
            named_action(core, &env, &mut work, ReplyTo::new(OWNER), CALL, crate::NamedAction::Subscribe { kind });
        let Some(Work::Event(Event::Tasks(tasks::Event::Subscribe { reply_to, .. }))) = work.pop() else {
            panic!("a task subscription was routed: {requests:?}");
        };
        let mut out = Queue::uncapped();
        engine::append(requests, &mut out);
        // Script acceptance at the task boundary; the child reserves topic admission for SubscribeTopic.
        append_task_answer(core, &env, tasks::Request::Done { reply_to }, &mut out);
        assert_subscription_answer(&out, CALL, ask, &crate::CallPart::Subscribed { subscription: SUBSCRIPTION });
    }
}

fn assert_subscription_answer(out: &Queue<Out>, key: crate::CallKey, ask: Option<topics::Ask>, part: &crate::CallPart) {
    use crate::{CoreRecord, Record, Write};
    let mut outputs = Queue::uncapped();
    for output in out {
        if let Out::Ask(_, contract::Ask::Topics(_)) | Out::Write(Write::Save(Record::Core(CoreRecord::Call(_)))) =
            output
        {
            outputs.push(output);
            continue;
        }
        if let Out::Core(Request::Held(held)) = output
            && let Held::CallAnswer { .. } = held.as_ref()
        {
            outputs.push(output);
        }
    }
    if let Some(ask) = ask {
        let Some(Out::Ask(connector, contract::Ask::Topics(notification))) = outputs.pop() else {
            panic!("topic notification precedes decision: {out:?}");
        };
        assert_eq!((*connector, notification), (DECLARER, &ask));
    }
    let Some(Out::Write(Write::Save(Record::Core(CoreRecord::Call(row))))) = outputs.pop() else {
        panic!("{part:?} is recorded without further input: {out:?}");
    };
    assert_eq!((row.key, &row.part, &row.settled), (key, part, &None));
    let Some(Out::Core(Request::Held(held))) = outputs.pop() else { panic!("held {part:?}: {out:?}") };
    let Held::CallAnswer { to, key: answered, part: answer } = held.as_ref() else { panic!("call answer") };
    assert_eq!((to, *answered, answer), (&ReplyTo::new(OWNER), key, part));
    assert!(outputs.pop().is_none(), "one answer and no other notification");
}

fn append_task_answer(core: &mut Core, env: &Env<Limits>, answer: tasks::Request, out: &mut Queue<Out>) {
    let mut child = Queue::uncapped();
    child.push(answer);
    let mut requests = Queue::uncapped();
    crate::routing::start_task_outputs(core, env, child, &mut requests);
    engine::append(requests, out);
}

#[test]
fn a_topic_notifies_only_its_connector() {
    for (remove, accepted) in [(false, true), (true, true), (false, false), (true, false)] {
        let (_, mut engine, env) = port_twins();
        let core = engine.core_mut();
        assert!(core.reserve_call(CALL, crate::CallHolder::Routed));
        let route = if remove {
            crate::RoutedCall::Unsubscribe { key: CALL, subscription: SUBSCRIPTION, topic: Some((EMPTY.0, TOPIC)) }
        } else {
            crate::RoutedCall::Subscribe { key: CALL, subscription: SUBSCRIPTION, topic: Some((EMPTY.0, TOPIC)) }
        };
        let reply_to = ReplyTo::new(core.fly_call(OWNER, route));
        let answer = if accepted {
            tasks::Request::Done { reply_to }
        } else {
            tasks::Request::Refused { reply_to, problem: tasks::Problem::new(Some(TASK), tasks::Refusal::Subscription) }
        };
        let mut out = Queue::uncapped();
        append_task_answer(core, &env, answer, &mut out);
        let mut found = 0_u32;
        while let Some(output) = out.pop() {
            if let Out::Ask(number, contract::Ask::Topics(ask)) = output {
                assert_eq!(number, EMPTY, "ownership comes from the topic, not capability declarations");
                match ask {
                    topics::Ask::Subscribed { request: OWNER, key: CALL, subscription: SUBSCRIPTION, topic: TOPIC } => {
                        assert!(accepted && !remove);
                    }
                    topics::Ask::Unsubscribed {
                        request: OWNER,
                        key: CALL,
                        subscription: SUBSCRIPTION,
                        topic: TOPIC,
                    } => {
                        assert!(accepted && remove);
                    }
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
