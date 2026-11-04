//! Store fill and admission at the high-water mark (protocol/store.md, 11.2).
use super::*;

pub(super) fn filled_core(bytes: u64) -> (Core, Env<Limits>) {
    let (mut core, limits) = core();
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    drop(step(&mut core, &env, Event::Fill { bytes }));
    (core, env)
}

pub(super) fn signed_in(core: &mut Core, env: &Env<Limits>) {
    drop(step(
        core,
        env,
        Event::People(people::Event::SignedIn {
            held: None,
            reply_to: ReplyTo::new(Token::new(7200)),
            person: 1,
            sign_in: 10,
            identity: people::Identity {
                key: people::IdentityKey { provider: 0, subject: Box::new([1]) },
                login: b"owner".as_slice().into(),
                name: b"A Person".as_slice().into(),
            },
            kind: people::Kind::Person,
        }),
    ));
    drop(step(core, env, Event::Tasks(tasks::Event::Check)));
    drop(step(core, env, Event::Tasks(tasks::Event::Restored)));
    drop(step(core, env, Event::Period { project: 1, period: 1, budget: 1000, next: None }));
}

pub(super) fn ask(core: &mut Core, env: &Env<Limits>, ask: people::Ask) -> Queue<Request> {
    step(
        core,
        env,
        Event::People(people::Event::Ask {
            models: Box::new([]),
            reply_to: ReplyTo::new(Token::new(7202)),
            sign_in: 10,
            key: [72; 16],
            ask,
        }),
    )
}

pub(super) fn goal() -> people::Ask {
    let mut member = fixtures::member(b"new goal", 100);
    member.tracked = Some(1);
    people::Ask::Create { project: 1, batch: Box::new([member]) }
}

fn busy(requests: Queue<Request>) {
    let mut out = requests;
    let Some(Request::Held(held)) = out.pop() else { panic!("one busy answer") };
    let Held::PeopleReply { to, reply } = *held else { panic!("the party's answer") };
    assert_eq!(to.into_token(), Token::new(7202));
    assert_eq!(reply, people::Reply::Again(people::Again::Busy));
    assert!(out.is_empty(), "the refusal saves nothing");
}

#[test]
fn a_sign_in_at_the_fill_mark_is_refused() {
    let (mut core, env) = filled_core(limits().fill_mark);
    let identity = people::Identity {
        key: people::IdentityKey { provider: 0, subject: Box::new([2]) },
        login: b"second".as_slice().into(),
        name: b"B Person".as_slice().into(),
    };
    let mut out =
        step(&mut core, &env, Event::SignIn { held: None, reply_to: ReplyTo::new(Token::new(7201)), identity });
    let Some(Request::Now(now)) = out.pop() else { panic!("one refusal, first") };
    let Now::SignInRefused { to } = *now else { panic!("the sign-in's refusal") };
    assert_eq!(to.into_token(), Token::new(7201));
    assert!(out.is_empty(), "nothing is written");
}

#[test]
fn a_chat_at_the_fill_mark_is_answered_busy() {
    let (mut core, env) = filled_core(limits().fill_mark);
    signed_in(&mut core, &env);
    busy(ask(
        &mut core,
        &env,
        people::Ask::Create { project: 1, batch: Box::new([fixtures::member(b"new chat", 100)]) },
    ));
}

#[test]
fn a_goal_below_the_fill_mark_is_made() {
    let (mut core, env) = filled_core(limits().fill_mark - 1);
    signed_in(&mut core, &env);
    let mut out = ask(&mut core, &env, goal());
    let mut request = None;
    while let Some(next) = out.pop() {
        if let Request::Ask { ask: Ask::TaskHoldings { request: token, tracked, symbols, .. }, .. } = next {
            assert!(tracked);
            assert!(symbols.is_empty());
            request = Some(token);
        }
    }
    let request = request.expect("goal asks for connector holdings");
    for connector in [1, 2] {
        drop(step(
            &mut core,
            &env,
            Event::Holdings { refused: None, request, connector, holdings: Some(Box::new([])) },
        ));
    }
    assert!(core.tasks.task(1).is_some(), "the goal was made below the mark");
}

#[test]
fn a_goal_at_the_fill_mark_is_answered_busy() {
    let (mut core, env) = filled_core(limits().fill_mark);
    signed_in(&mut core, &env);
    busy(ask(&mut core, &env, goal()));
}

#[test]
fn a_full_sign_in_pool_holds_its_answer_and_advances_the_deployment_numbers() {
    let (mut core, limits) = core();
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let identity = people::Identity {
        key: people::IdentityKey { provider: 0, subject: Box::new([1]) },
        login: b"owner".as_slice().into(),
        name: b"A Person".as_slice().into(),
    };
    let token = Token::new(7210);
    for _ in 0..limits.people.sign_ins {
        drop(step(
            &mut core,
            &env,
            Event::SignIn { held: None, reply_to: ReplyTo::new(token), identity: identity.clone() },
        ));
    }
    let before = core.counters.next_commit();
    assert!(!core.counters.dirty());
    let mut out = step(&mut core, &env, Event::SignIn { held: None, reply_to: ReplyTo::new(token), identity });
    let Some(Request::Held(held)) = out.pop() else { panic!("Full waits for durability") };
    let Held::PeopleReply { to, reply } = *held else { panic!("people reply") };
    assert_eq!(to.into_token(), token);
    assert_eq!(
        reply,
        people::Reply::Full {
            sign_ins: limits.people.sign_ins,
            earliest: Wall::from_nanos(limits.people.sign_in_lifetime.as_nanos()),
        }
    );
    assert!(out.is_empty(), "Full writes no people record");
    assert!(core.counters.dirty());
    let after = core.counters.next_commit();
    assert_eq!(after.people, before.people + 1);
    assert_eq!(after.sign_ins, before.sign_ins + 1);
    assert_eq!(after.commits, before.commits + 1);
}

fn take_write_holdings(core: &mut Core, env: &Env<Limits>) -> Queue<Request> {
    let hold = tasks::HoldKind::Exclusive { taken: tasks::Taken::Waits };
    send_task(
        core,
        &env.limits,
        tasks::Event::Kinds { connector: 2, kinds: Box::new([tasks::Kind { connector: 2, kind: 1, hold }]) },
    );
    let mut member = new_row(core, 1, tasks::Funder::Period { project: 1, period: 1 }, None);
    let mut holdings = List::with_capacity(2);
    for path in [b"first".as_slice(), b"second".as_slice()] {
        let resource = tasks::Name { connector: 2, path: Box::new([Box::from(path)]) };
        send_task(core, &env.limits, tasks::Event::Resource { name: resource.clone(), hold });
        send_task(
            core,
            &env.limits,
            tasks::Event::ResourceAccess {
                project: 1,
                name: resource.clone(),
                access: tasks::ResourceAccess::Writable,
            },
        );
        holdings.push(tasks::Holding::Write { resource, kind: 1 }).unwrap();
    }
    member.holdings = holdings.into_boxed();
    step(
        core,
        env,
        Event::Tasks(tasks::Event::Make {
            reply_to: ReplyTo::new(Token::new(7203)),
            creator: tasks::Party::Person(1),
            batch: Box::new([member]),
        }),
    )
}

#[test]
fn a_taking_tells_each_write_holds_connector_with_a_fresh_entry() {
    let (mut core, limits) = tree_core();
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let mut out = take_write_holdings(&mut core, &env);
    let mut entries = List::with_capacity(2);
    let mut resources = List::with_capacity(2);
    while let Some(request) = out.pop() {
        if let Request::Ask { connector, ask: Ask::Hold { task, resource, entry } } = request {
            assert_eq!(task, 1);
            assert_eq!(connector, 2);
            assert_eq!(resource.connector, connector);
            assert_ne!(entry, 0);
            entries.push(entry).unwrap();
            resources.push(resource.path).unwrap();
        }
    }
    assert_eq!(entries.len(), 2);
    assert_ne!(entries.as_slice()[0], entries.as_slice()[1]);
    assert_eq!(
        resources.as_slice(),
        &[Box::from([Box::from(b"first".as_slice())]), Box::from([Box::from(b"second".as_slice())])]
    );
    drop(step(
        &mut core,
        &env,
        Event::Tasks(tasks::Event::Control {
            reply_to: ReplyTo::new(Token::new(7204)),
            by: tasks::Party::Person(1),
            task: 1,
            control: tasks::Control::Cancel { reason: Box::new([]), grace: None, message: None },
        }),
    ));
    let mut releases = 0_u32;
    for connector in [1, 2] {
        let mut out = step(&mut core, &env, Event::EffectConnector(connector::Event::Closed { task: 1, connector }));
        while let Some(request) = out.pop() {
            if let Request::Ask { ask: Ask::Release { task, entry, .. }, .. } = request {
                assert_eq!(task, 1);
                assert_ne!(entry, 0);
                assert!(!entries.as_slice().contains(&entry));
                releases += 1;
            }
        }
    }
    assert_eq!(releases, 2);
}

#[test]
fn an_exhausted_taking_holds_the_task_without_telling_any_connector() {
    for remaining in [0, 1] {
        let (mut core, limits) = tree_core();
        core.counters.deployment.connector_rows = u64::MAX - remaining;
        let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
        let mut out = take_write_holdings(&mut core, &env);
        while let Some(request) = out.pop() {
            if let Request::Ask { ask: Ask::Hold { .. }, .. } = request {
                panic!("an exhausted taking sends no hold asks");
            }
        }
        let tasks::Phase::Held { why, .. } = core.tasks.task(1).unwrap().phase else { panic!("budget hold") };
        assert_eq!(why, tasks::Hold::Budget);
    }
}
