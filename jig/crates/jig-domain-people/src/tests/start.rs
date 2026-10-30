//! Explicit start's fixed-time and phase controls (engine.md, 6).
use super::{LIMITS, fixtures};
use crate::*;
use skein_lib::{Duration, Env, Queue, Time, Wall};

fn env() -> Env<Limits> {
    Env { limits: LIMITS, now: Time::ZERO, wall: Wall::EPOCH }
}
fn load(domain: &mut Domain, env: &Env<Limits>, row: Stored, out: &mut Queue<Request>) {
    step(domain, env, Event::Restore { record: row }, out);
}

#[test]
fn explicit_start_keeps_fixed_expiry_through_forward_and_backward_clock_changes() {
    for later in [0, 500] {
        let mut env = env();
        let start = Wall::from_nanos(Duration::from_secs(120).as_nanos());
        let mut domain = Domain::new(&LIMITS, Box::new([]), 1, Box::new([]));
        domain.start_restore(start);
        let mut out = Queue::uncapped();
        let rows = [
            Stored::Person {
                number: 1,
                identity: Identity {
                    key: IdentityKey { provider: 0, subject: Box::new([]) },
                    login: Box::new([]),
                    name: Box::new([]),
                },
            },
            Stored::SignIn { number: 1, person: 1, expires: Wall::from_nanos(Duration::from_secs(121).as_nanos()) },
            Stored::Answer {
                key: RequestKey { person: 1, key: [1; 16] },
                ask: Box::new(Ask::Stop { project: 0, task: 0 }),
                models: Box::new([]),
                outcome: Outcome::Stopped { task: 0 },
                at: Wall::from_nanos(Duration::from_secs(1).as_nanos()),
            },
        ];
        env.wall = Wall::from_nanos(Duration::from_secs(later).as_nanos());
        for row in &rows {
            load(&mut domain, &env, row.clone(), &mut out);
        }
        assert!(out.is_empty());
        assert!(domain.identity(1).is_some());
        assert!(!domain.ready());
        env.wall = Wall::from_nanos(Duration::from_secs(if later == 0 { 500 } else { 0 }).as_nanos());
        assert!(domain.start_link(&env));
        domain.start_sweep();
        env.wall = Wall::from_nanos(Duration::from_secs(later).as_nanos());
        for row in &rows {
            domain.start_sweep_row(&env, row, &mut out);
        }
        assert!(out.is_empty());
        fire(&mut domain, &env, &mut out);
        assert!(out.is_empty());
        assert!(domain.start_complete(&env));
        assert!(domain.ready());
        assert!(out.is_empty(), "completion never erases");
        if later == 500 {
            fire(&mut domain, &env, &mut out);
            assert_eq!(out.pop(), Some(Request::Erase { key: Key::Answer(RequestKey { person: 1, key: [1; 16] }) }));
        }
    }
}

#[test]
fn explicit_start_skips_all_expired_families_before_live_fit() {
    let mut env = env();
    env.limits.sign_ins = 1;
    env.limits.requests = 1;
    let start = Wall::from_nanos(LIMITS.request_retention.as_nanos());
    let mut domain = Domain::new(&env.limits, Box::new([]), 1, Box::new([]));
    domain.start_restore(start);
    let mut out = Queue::uncapped();
    for number in 1..=10 {
        let rows = [
            Stored::SignIn { number, person: 999, expires: start },
            Stored::Answer {
                key: RequestKey { person: 999, key: [u8::try_from(number).unwrap(); 16] },
                ask: Box::new(Ask::Say { project: 0, task: 0, words: Box::new([0; 100]) }),
                models: Box::new([]),
                outcome: Outcome::SignedOut,
                at: Wall::EPOCH,
            },
            Stored::Acted {
                key: ActKey { connector: 0, key: [u8::try_from(number).unwrap(); 16] },
                act: Act::Close { project: 0, task: 0, reason: Box::new([0; 100]), grace: None },
                outcome: Outcome::SignedOut,
                at: Wall::EPOCH,
            },
        ];
        for row in &rows {
            load(&mut domain, &env, row.clone(), &mut out);
        }
        assert!(out.is_empty());
    }
    assert!(domain.start_link(&env));
    domain.start_sweep();
    let row = Stored::SignIn { number: 1, person: 999, expires: start };
    env.wall = Wall::EPOCH;
    domain.start_sweep_row(&env, &row, &mut out);
    assert_eq!(out.pop(), Some(Request::Erase { key: Key::SignIn(1) }));
    assert!(domain.start_complete(&env));
    assert_eq!(domain.sign_in_person(1), None);
}

#[test]
#[should_panic(expected = "SignIn(7) links missing person 99")]
fn explicit_start_asserts_required_links_with_keys() {
    let env = env();
    let mut domain = Domain::new(&LIMITS, Box::new([]), 1, Box::new([]));
    domain.start_restore(Wall::EPOCH);
    load(
        &mut domain,
        &env,
        Stored::SignIn { number: 7, person: 99, expires: Wall::from_nanos(1) },
        &mut Queue::uncapped(),
    );
    domain.start_link(&env);
}

fn person(number: u64) -> Stored {
    Stored::Person {
        number,
        identity: Identity {
            key: IdentityKey { provider: 0, subject: number.to_be_bytes().into() },
            login: Box::new([]),
            name: Box::new([]),
        },
    }
}
fn answer(key: u8, ask: Ask, outcome: Outcome) -> Stored {
    Stored::Answer {
        key: RequestKey { person: 1, key: [key; 16] },
        ask: Box::new(ask),
        models: Box::new([]),
        outcome,
        at: Wall::EPOCH,
    }
}
fn new_start(env: &Env<Limits>) -> (Domain, Queue<Request>) {
    let mut domain = Domain::new(&env.limits, Box::new([]), 1, Box::new([]));
    domain.start_restore(env.wall);
    (domain, Queue::uncapped())
}

#[test]
fn explicit_start_live_fit_failure_is_sticky_and_does_not_erase() {
    let mut env = env();
    env.limits.people = 1;
    let (mut domain, mut out) = new_start(&env);
    load(&mut domain, &env, person(1), &mut out);
    load(&mut domain, &env, person(2), &mut out);
    load(&mut domain, &env, person(3), &mut out);
    assert_eq!(
        out.pop(),
        Some(Request::RestoreRefused { key: Key::Person(2), refusal: Refusal::Limit { which: LimitKind::Restore } })
    );
    assert!(out.is_empty());
    assert!(!domain.start_link(&env));
    assert!(!domain.ready());
    fire(&mut domain, &env, &mut out);
    assert!(out.is_empty());
}

#[test]
fn explicit_start_phases_reject_repeated_late_and_mixed_calls() {
    for bad in 0_u8..9 {
        let env = env();
        let mut domain = Domain::new(&LIMITS, Box::new([]), 1, Box::new([]));
        let mut out = Queue::uncapped();
        let result = std::panic::catch_unwind(core::panic::AssertUnwindSafe(|| {
            if bad == 0 {
                load(&mut domain, &env, person(1), &mut out);
            }
            if bad == 1 {
                step(&mut domain, &env, Event::Restored, &mut out);
            }
            domain.start_restore(env.wall);
            match bad {
                0..=2 => domain.start_restore(env.wall),
                3 => step(&mut domain, &env, Event::Restored, &mut out),
                4 => domain.start_sweep(),
                5 => {
                    domain.start_complete(&env);
                }
                6 => {
                    assert!(domain.start_link(&env));
                    domain.start_link(&env);
                }
                7 => {
                    assert!(domain.start_link(&env));
                    load(&mut domain, &env, person(1), &mut out);
                }
                8 => {
                    domain.start_sweep_row(&env, &person(1), &mut out);
                }
                _ => unreachable!(),
            }
        }));
        assert!(result.is_err(), "bad phase {bad}");
    }
}

#[test]
fn explicit_empty_start_blocks_admission_and_roles_until_completion() {
    let env = env();
    let (mut domain, mut out) = new_start(&env);
    for phase in 0_u8..3 {
        if phase == 1 {
            assert!(domain.start_link(&env));
        }
        if phase == 2 {
            domain.start_sweep();
        }
        step(&mut domain, &env, Event::Roles { project: 1, holdings: Box::new([]) }, &mut out);
        assert!(!domain.has_project(1));
        step(
            &mut domain,
            &env,
            Event::Ask {
                reply_to: skein_lib::ReplyTo::new(skein_lib::Token::new(1)),
                sign_in: 1,
                key: [1; 16],
                ask: Ask::Stop { project: 1, task: 1 },
                models: Box::new([]),
            },
            &mut out,
        );
        assert_eq!(
            out.pop(),
            Some(Request::Reply {
                to: skein_lib::ReplyTo::new(skein_lib::Token::new(1)),
                reply: Reply::Again(Again::NotReady)
            })
        );
        assert!(out.is_empty());
        assert!(!domain.ready());
    }
    assert!(domain.start_complete(&env));
    assert!(domain.ready());
}

#[test]
fn explicit_start_fits_names_without_canonical_or_selector_judgments() {
    let env = env();
    let (mut domain, mut out) = new_start(&env);
    load(&mut domain, &env, person(1), &mut out);
    let row = Stored::Answer {
        key: RequestKey { person: 1, key: [1; 16] },
        ask: Box::new(Ask::Say { project: 0, task: 0, words: Box::new([]) }),
        models: Box::new([Box::from(b"z".as_slice()), Box::from(b"z".as_slice()), Box::new([])]),
        outcome: Outcome::Stopped { task: 99 },
        at: Wall::EPOCH,
    };
    load(&mut domain, &env, row, &mut out);
    assert!(out.is_empty());
    assert!(domain.start_link(&env));
    domain.start_sweep();
    assert!(domain.start_complete(&env));
}

#[test]
fn explicit_start_checks_nested_bytes_and_shared_request_capacity() {
    for oversized in [false, true] {
        let mut env = env();
        env.limits.requests = if oversized { 2 } else { 1 };
        let (mut domain, mut out) = new_start(&env);
        load(&mut domain, &env, person(1), &mut out);
        let words = skein_lib::bytes::zeroed(usize::try_from(env.limits.words).unwrap());
        load(
            &mut domain,
            &env,
            answer(1, Ask::Say { project: 1, task: 1, words }, Outcome::Said { task: 1, message: 1 }),
            &mut out,
        );
        assert!(out.is_empty(), "exact bytes and count fit");
        let row = if oversized {
            let words = skein_lib::bytes::zeroed(usize::try_from(env.limits.words + 1).unwrap());
            answer(2, Ask::Say { project: 1, task: 1, words }, Outcome::Said { task: 1, message: 1 })
        } else {
            Stored::Acted {
                key: ActKey { connector: 1, key: [1; 16] },
                act: Act::RemoveAccount { account: 1 },
                outcome: Outcome::AccountChanged,
                at: Wall::EPOCH,
            }
        };
        let key = row.key();
        load(&mut domain, &env, row, &mut out);
        assert_eq!(
            out.pop(),
            Some(Request::RestoreRefused { key, refusal: Refusal::Limit { which: LimitKind::Restore } })
        );
        assert!(!domain.start_link(&env));
    }
}

#[test]
fn explicit_start_asserts_duplicate_identity_and_wrong_family() {
    for collision in [false, true] {
        let env = env();
        let (mut domain, mut out) = new_start(&env);
        let result = std::panic::catch_unwind(core::panic::AssertUnwindSafe(|| {
            if collision {
                load(&mut domain, &env, person(1), &mut out);
                let Stored::Person { identity, .. } = person(1) else { unreachable!() };
                load(&mut domain, &env, Stored::Person { number: 2, identity }, &mut out);
            } else {
                load(&mut domain, &env, Stored::EndedRead { task: 1, party: 1, through: 1 }, &mut out);
            }
        }));
        assert!(result.is_err());
        assert!(out.is_empty());
    }
}

#[test]
fn explicit_start_all_families_use_equality_and_keep_identity_after_signin_expiry() {
    let mut env = env();
    env.wall = Wall::from_nanos(LIMITS.request_retention.as_nanos());
    let (mut domain, mut out) = new_start(&env);
    load(&mut domain, &env, person(1), &mut out);
    let rows = [
        Stored::SignIn { number: 1, person: 1, expires: env.wall },
        answer(1, Ask::Stop { project: 0, task: 0 }, Outcome::Stopped { task: 0 }),
        Stored::Acted {
            key: ActKey { connector: 1, key: [1; 16] },
            act: Act::RemoveAccount { account: 1 },
            outcome: Outcome::AccountChanged,
            at: Wall::EPOCH,
        },
    ];
    for row in &rows {
        load(&mut domain, &env, row.clone(), &mut out);
    }
    assert!(out.is_empty());
    env.wall = Wall::EPOCH;
    assert!(domain.start_link(&env));
    assert!(domain.identity(1).is_some());
    domain.start_sweep();
    for row in &rows {
        domain.start_sweep_row(&env, row, &mut out);
        assert_eq!(out.pop(), Some(Request::Erase { key: row.key() }));
    }
    assert!(domain.start_complete(&env));
    assert!(domain.identity(1).is_some());
    assert_eq!(domain.sign_in_person(1), None);
    assert_eq!(domain.next_deadline(), None);
}

#[test]
fn explicit_start_live_answer_and_act_expire_only_through_timers_after_open() {
    for later in [0, 500] {
        let mut env = env();
        let (mut domain, mut out) = new_start(&env);
        let rows = [
            person(1),
            Stored::SignIn { number: 1, person: 1, expires: Wall::from_nanos(1) },
            answer(1, Ask::Stop { project: 0, task: 0 }, Outcome::Stopped { task: 0 }),
            Stored::Acted {
                key: ActKey { connector: 1, key: [1; 16] },
                act: Act::RemoveAccount { account: 1 },
                outcome: Outcome::AccountChanged,
                at: Wall::EPOCH,
            },
        ];
        env.wall = Wall::from_nanos(Duration::from_secs(later).as_nanos());
        for row in &rows {
            load(&mut domain, &env, row.clone(), &mut out);
        }
        assert_eq!(domain.next_deadline(), None);
        assert!(domain.start_link(&env));
        domain.start_sweep();
        for row in &rows {
            domain.start_sweep_row(&env, row, &mut out);
        }
        assert!(out.is_empty());
        assert!(domain.start_complete(&env));
        assert!(out.is_empty());
        env.now = Time::from_nanos(Duration::from_secs(501).as_nanos());
        for key in [
            Key::Acted(ActKey { connector: 1, key: [1; 16] }),
            Key::Answer(RequestKey { person: 1, key: [1; 16] }),
            Key::SignIn(1),
        ] {
            fire(&mut domain, &env, &mut out);
            assert_eq!(out.pop(), Some(Request::Erase { key }));
        }
        assert!(domain.identity(1).is_some());
    }
}

#[test]
fn explicit_start_asserts_each_required_local_link_before_sweep() {
    let roster_ask = Ask::SetRoles { project: 9, holdings: Box::new([Holding { person: 99, role: Role::Member }]) };
    for row in [
        Stored::Roles { project: 1, holdings: Box::new([Holding { person: 99, role: Role::Member }]) },
        Stored::ReadResults { party: 99, through: 0 },
        Stored::ReadConversation { party: 99, task: 0, through: 0 },
        Stored::Answer {
            key: RequestKey { person: 99, key: [1; 16] },
            ask: Box::new(Ask::Stop { project: 1, task: 1 }),
            models: Box::new([]),
            outcome: Outcome::Stopped { task: 1 },
            at: Wall::EPOCH,
        },
        answer(
            1,
            Ask::MakeService { project: 0, name: Box::new([]), role: Role::Member },
            Outcome::ServiceMade { person: 99 },
        ),
        answer(1, roster_ask.clone(), Outcome::RolesSet { project: 9 }),
    ] {
        let env = env();
        let (mut domain, mut out) = new_start(&env);
        load(&mut domain, &env, person(1), &mut out);
        load(&mut domain, &env, row, &mut out);
        assert!(out.is_empty());
        let result = std::panic::catch_unwind(core::panic::AssertUnwindSafe(|| {
            domain.start_link(&env);
        }));
        assert!(result.is_err());
        assert!(out.is_empty(), "link panic erases no row");
    }
    // A successful historical roster retains its members without equating it with today's roster.
    let env = env();
    let (mut domain, mut out) = new_start(&env);
    load(&mut domain, &env, person(1), &mut out);
    load(&mut domain, &env, Stored::Roles { project: 9, holdings: Box::new([]) }, &mut out);
    load(&mut domain, &env, answer(1, roster_ask.clone(), Outcome::RolesSet { project: 9 }), &mut out);
    assert_eq!(domain.project_members(9), Some([].as_slice()));
    assert!(!domain.ready());
    let result = std::panic::catch_unwind(core::panic::AssertUnwindSafe(|| {
        domain.start_link(&env);
    }));
    assert!(result.is_err());
    assert!(out.is_empty());
    let (mut domain, mut out) = new_start(&env);
    load(&mut domain, &env, person(1), &mut out);
    load(&mut domain, &env, answer(1, roster_ask, Outcome::Refused(Refusal::Unknown)), &mut out);
    assert!(domain.start_link(&env), "refused targets create no restart obligations");
}

fn copies<T: Clone + core::fmt::Debug>(value: T, count: u32) -> Box<[T]> {
    let mut rows = skein_lib::List::with_capacity(count);
    for _ in 0..count {
        rows.push(value.clone()).unwrap();
    }
    rows.into_boxed()
}

#[test]
fn explicit_start_nested_representation_bounds_are_exact() {
    for field in 0_u8..7 {
        for extra in [0_u32, 1] {
            let env = env();
            let (mut domain, mut out) = new_start(&env);
            load(&mut domain, &env, person(1), &mut out);
            let mut row = answer(1, Ask::Stop { project: 0, task: 0 }, Outcome::Stopped { task: 0 });
            match field {
                0 => {
                    let Stored::Answer { models, .. } = &mut row else { unreachable!() };
                    *models = copies::<Box<[u8]>>(Box::new([]), MODELS + extra);
                }
                1 => {
                    let Stored::Answer { models, .. } = &mut row else { unreachable!() };
                    *models = Box::new([skein_lib::bytes::zeroed(usize::try_from(MODEL_NAME_BYTES + extra).unwrap())]);
                }
                2 => {
                    let mut amendment = fixtures::unchanged(b"");
                    amendment.instructions =
                        Some(skein_lib::bytes::zeroed(usize::try_from(LIMITS.amendment_bytes + extra).unwrap()));
                    row = answer(
                        1,
                        Ask::Amend { project: 0, task: 0, revision: 0, amendment, release: None },
                        Outcome::Amended { task: 0, released: false },
                    );
                }
                3 => {
                    row = answer(
                        1,
                        Ask::Create { project: 0, batch: copies(fixtures::member(b""), LIMITS.batch + extra) },
                        Outcome::Refused(Refusal::Unknown),
                    );
                }
                4 => {
                    row = answer(
                        1,
                        Ask::Stop { project: 0, task: 0 },
                        Outcome::AccountSignInStarted {
                            source: 0,
                            address: skein_lib::bytes::zeroed(usize::try_from(ADDRESS + extra).unwrap()),
                        },
                    );
                }
                5 => {
                    row = Stored::Person {
                        number: 2,
                        identity: Identity {
                            key: IdentityKey { provider: 0, subject: Box::new([2]) },
                            login: Box::new([]),
                            name: skein_lib::bytes::zeroed(usize::try_from(LIMITS.identity_bytes - 1 + extra).unwrap()),
                        },
                    };
                }
                6 => {
                    row = Stored::Roles {
                        project: 1,
                        holdings: copies(
                            Holding { person: 1, role: Role::Policy { role: 0 } },
                            LIMITS.holdings + extra,
                        ),
                    };
                }
                _ => unreachable!(),
            }
            let key = row.key();
            load(&mut domain, &env, row, &mut out);
            if extra == 0 {
                assert!(out.is_empty(), "field {field} exactly fits");
                assert!(domain.start_link(&env));
            } else {
                assert_eq!(
                    out.pop(),
                    Some(Request::RestoreRefused { key, refusal: Refusal::Limit { which: LimitKind::Restore } }),
                    "field {field}"
                );
                assert!(!domain.start_link(&env));
            }
        }
    }
}
