//! The timer case's listing traffic follows events, not elapsed real time.
use skein_forge::api::{self, Answer, Op, Read};
use skein_forge_client::{self as client, Domain, Event, Request};
use skein_lib::{Env, Queue, Time, Token, Wall};
use temper_real_loop_world::{PATIENCE, case, fixtures};

fn limits() -> client::Limits {
    let address = ([127, 0, 0, 1], 1).into();
    let (mut agent, mut config) = fixtures::documents(std::path::Path::new("/fixture"), address, address);
    (case::fixture_check().configure)(&mut config, &mut agent, address);
    let mut limits = temper_service::limits::profile(temper_service::limits::Profile::Small);
    for (name, number) in config["application"]["limits"]["overrides"].as_object().expect("overrides") {
        temper_service::limits::apply(&mut limits, name.as_bytes(), number.as_u64().expect("limit"))
            .expect("application limit");
    }
    limits.forge.client
}

fn listing(domain: &mut Domain, env: &Env<client::Limits>, out: &mut Queue<Request>) -> Option<Token> {
    client::resume(domain, env, out);
    let mut sent = None;
    while let Some(request) = out.pop() {
        if let Request::Call { call, op, .. } = request {
            assert!(matches!(op, Op::Read(Read::Items { .. })), "only repository listings: {op:?}");
            assert!(sent.replace(call).is_none(), "one listing at a time");
        }
    }
    domain.reclaim();
    sent
}

fn answer(domain: &mut Domain, env: &Env<client::Limits>, out: &mut Queue<Request>, call: Token) {
    client::step(
        domain,
        env,
        Event::Answered {
            call,
            cost: 1,
            free: 0,
            rate: None,
            result: Ok(Answer::Items { items: Box::new([]), more: false, now: env.now }),
        },
        out,
    );
}

#[test]
fn the_timer_case_lists_on_hints_instead_of_time() {
    let mut env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits() };
    let mut domain = Domain::configured(
        &env.limits,
        fixtures::SEED,
        client::Config { namespace: b"kit".as_slice().into(), writers: Box::new([]) },
    )
    .expect("fixture client");
    let mut out = Queue::with_capacity(client::max_out(&env.limits));
    client::step(&mut domain, &env, Event::Restored { clock: client::RecoveryClock::Monotonic }, &mut out);
    let repository = api::Repository { forge: 1, repository: temper_real_loop_world::forge::ID };
    client::step(
        &mut domain,
        &env,
        Event::Keep {
            owner: Token::new(1),
            watches: Box::new([client::Watch {
                resource: client::Resource { repository, what: client::What::Repository },
                participating: false,
            }]),
        },
        &mut out,
    );
    let initial = listing(&mut domain, &env, &mut out).expect("initial repository scan");
    answer(&mut domain, &env, &mut out, initial);
    // Replay every due alarm through the story's allowance, without a real clock.
    let end = env.now.saturating_add(PATIENCE);
    let mut periodic = 0;
    while let Some(next) = domain.next_deadline() {
        if next >= end {
            break;
        }
        env.now = next;
        while domain.is_due(env.now) {
            client::fire(&mut domain, &env, &mut out);
        }
        if let Some(call) = listing(&mut domain, &env, &mut out) {
            periodic += 1;
            answer(&mut domain, &env, &mut out, call);
        }
    }
    assert_eq!(periodic, 0, "elapsed time adds no listing");
    env.now = end;
    client::step(
        &mut domain,
        &env,
        Event::Hint { hint: api::Hint { repository, change: api::Change::Item(1), key: None } },
        &mut out,
    );
    env.now = env.now.saturating_add(env.limits.hinted);
    while domain.is_due(env.now) {
        client::fire(&mut domain, &env, &mut out);
    }
    let hinted = listing(&mut domain, &env, &mut out).expect("hint still refreshes the listing");
    answer(&mut domain, &env, &mut out, hinted);
    assert_eq!(listing(&mut domain, &env, &mut out), None, "the hint makes one scan");
}
