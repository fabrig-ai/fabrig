//! Places bound storage, not the request rate (domain/forge/adoption.md, 13).
use super::*;
use jig_domain_people as people;

fn answered(d: &mut Domain, env: &Env<Limits>, number: u64, out: &mut Queue<Request>) {
    let request = Token::new(number);
    step(
        d,
        env,
        Event::ApplicationAnswered {
            request,
            outcome: people::Outcome::Created { first: number, last: number, given: None },
        },
        out,
    );
    let mut done = false;
    while let Some(output) = out.pop() {
        if output == (Request::ApplicationDone { request, token: request }) {
            done = true;
        }
    }
    assert!(done, "open {number} answered");
    assert_eq!(d.application_outcome(request), Some(&ApplicationOutcome::Opened { task: number }));
}

#[test]
fn thirty_three_opens_are_answered_during_retention() {
    let env = Env { limits: Limits { applications: 32, adoptions: 2, ..LIMITS }, ..env() };
    let mut d = domain_with_limits(&env.limits);
    let mut out = Queue::uncapped();
    let retention = Duration::from_secs(86_400);
    for number in 1..=32 {
        let request = Token::new(number);
        assert!(d.opening(&env, request, request, retention, None, &mut out));
        answered(&mut d, &env, number, &mut out);
    }
    let request = Token::new(33);
    assert!(d.opening(&env, request, request, retention, None, &mut out), "a kept answer must make room for open 33");
    assert_eq!(out.pop(), Some(Request::Erase { key: Key::Application(1) }));
    assert!(out.is_empty());
    assert_eq!(d.application_outcome(Token::new(1)), None);
    answered(&mut d, &env, 33, &mut out);
}

fn restore_answer(d: &mut Domain, token: u64, until: Wall) {
    outputs(
        d,
        Event::Restore {
            record: Stored::Application { token, until, outcome: ApplicationOutcome::Opened { task: token } },
        },
    );
}

fn make(number: u64) -> Event {
    Event::Application {
        role: None,
        carried: None,
        request: Token::new(number),
        token: Token::new(number),
        retention: Duration::from_secs(60),
        ask: Ask::MakeRepository {
            forge: 1,
            org: b"org".as_slice().into(),
            name: b"new".as_slice().into(),
            from: MakeFrom::Empty,
            into: None,
        },
    }
}

fn drain(out: &mut Queue<Request>) -> Box<[Request]> {
    let mut requests = skein_lib::List::with_capacity(out.len());
    while let Some(request) = out.pop() {
        requests.push(request).unwrap();
    }
    requests.into_boxed()
}

fn erased_keys(requests: &[Request]) -> Box<[Key]> {
    let mut keys = skein_lib::List::with_capacity(u32::try_from(requests.len()).unwrap());
    for request in requests {
        if let Request::Erase { key } = request {
            keys.push(key.clone()).unwrap();
        }
    }
    keys.into_boxed()
}

#[test]
fn the_earliest_answer_makes_room_for_a_make_never_a_pending_open() {
    // A lower token is not older when its retention ends later; equal ends
    // choose the lower token. Both cases leave the two pending opens intact.
    for (first_until, erased) in [(300_000_000_000, 2), (200_000_000_000, 1)] {
        let mut d = domain();
        let mut out = Queue::uncapped();
        for number in [3, 4] {
            let request = Token::new(number);
            assert!(d.opening(&env(), request, request, Duration::from_secs(60), None, &mut out));
        }
        restore_answer(&mut d, 1, Wall::from_nanos(first_until));
        restore_answer(&mut d, 2, Wall::from_nanos(200_000_000_000));
        let requests = outputs(&mut d, make(5));
        assert_eq!(erased_keys(&requests).as_ref(), &[Key::Application(erased)]);
        assert_eq!(d.application_outcome(Token::new(erased)), None);
        assert_eq!(
            d.application_outcome(Token::new(3 - erased)),
            Some(&ApplicationOutcome::Opened { task: 3 - erased })
        );
        assert!(requests.contains(&Request::ApplicationDone { request: Token::new(5), token: Token::new(5) }));
        assert_eq!(d.application_outcome(Token::new(5)), Some(&ApplicationOutcome::MakeRefused(MakeRefusal::NotOwner)));
        for number in [3, 4] {
            answered(&mut d, &env(), number, &mut out);
        }
    }
}

#[test]
fn all_places_pending_answers_a_make_busy_without_erasing() {
    let env = Env { limits: Limits { applications: 2, adoptions: 2, ..LIMITS }, ..env() };
    let mut d = domain_with_limits(&env.limits);
    let mut out = Queue::uncapped();
    for number in [1, 2] {
        let request = Token::new(number);
        assert!(d.opening(&env, request, request, Duration::from_secs(60), None, &mut out));
    }
    step(&mut d, &env, make(3), &mut out);
    assert_eq!(
        drain(&mut out).as_ref(),
        &[Request::ApplicationAgain { request: Token::new(3), why: people::Again::Busy }]
    );
    for number in [1, 2] {
        answered(&mut d, &env, number, &mut out);
    }
}

#[test]
fn busy_flights_drop_no_kept_answer() {
    // Opening flights constrain both opens and adoptions; repository reads
    // from a making constrain adoptions even without keyed flights.
    for flights in 0_u8..3 {
        let mut d = domain();
        let mut out = Queue::uncapped();
        for number in [1, 2] {
            let request = Token::new(number);
            if flights == 2 {
                outputs(
                    &mut d,
                    Event::MakingAsked {
                        request,
                        adoption: fixtures::adoption(),
                        repository: skein_forge::api::Repository {
                            forge: 1,
                            repository: u32::try_from(number + 1).unwrap(),
                        },
                    },
                );
            } else {
                assert!(d.opening(&env(), request, request, Duration::from_secs(60), None, &mut out));
            }
        }
        let kept = if flights == 2 { 4 } else { 2 };
        for number in 10..10 + kept {
            // Even expired answers must stay when no place will be taken.
            restore_answer(&mut d, number, env().wall);
        }
        let request = Token::new(99);
        if flights == 0 {
            assert!(!d.opening(&env(), request, request, Duration::from_secs(60), None, &mut out));
        } else {
            step(
                &mut d,
                &env(),
                Event::Application {
                    role: None,
                    carried: None,
                    request,
                    token: request,
                    retention: Duration::from_secs(60),
                    ask: Ask::Adopt(fixtures::adoption()),
                },
                &mut out,
            );
        }
        assert_eq!(
            drain(&mut out).as_ref(),
            &[Request::ApplicationAgain { request, why: people::Again::Busy }],
            "flights {flights}"
        );
        for number in 10..10 + kept {
            assert_eq!(d.application_outcome(Token::new(number)), Some(&ApplicationOutcome::Opened { task: number }));
        }
    }
}

#[test]
fn every_expired_answer_is_erased_by_one_request() {
    let mut d = domain();
    let wall = env().wall.as_nanos();
    for (token, until) in [(1, wall - 2), (2, wall - 1), (3, wall), (4, wall + 1)] {
        restore_answer(&mut d, token, Wall::from_nanos(until));
    }
    let requests = outputs(&mut d, make(5));
    assert_eq!(erased_keys(&requests).as_ref(), &[Key::Application(1), Key::Application(2), Key::Application(3)]);
    for token in [1, 2, 3] {
        assert_eq!(d.application_outcome(Token::new(token)), None);
    }
    assert_eq!(d.application_outcome(Token::new(4)), Some(&ApplicationOutcome::Opened { task: 4 }));
    assert!(requests.contains(&Request::ApplicationDone { request: Token::new(5), token: Token::new(5) }));
}
