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
    let failure = Some(RestoreFailure {
        key: Key::Person(2),
        cause: RestoreCause::Bound { which: RestoreBound::People, held: 2, limit: 1 },
    });
    assert_eq!(domain.restore_failure(), failure);
    assert!(domain.parties().is_empty());
    assert_eq!(domain.results_seen(1), None);
    assert_eq!(domain.persons(), u32::MAX);
    assert!(!domain.start_project(&env.limits, RestoreProjection::Waiting { task: 1, entries: Box::new([]) }));
    assert_eq!(domain.restore_failure(), failure);
    assert!(!domain.start_link(&env));
    assert_eq!(domain.restore_failure(), failure);
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
                assert_eq!(domain.restore_failure(), None);
                assert!(domain.start_link(&env));
            } else {
                assert_eq!(
                    out.pop(),
                    Some(Request::RestoreRefused { key, refusal: Refusal::Limit { which: LimitKind::Restore } }),
                    "field {field}"
                );
                let (which, limit) = match field {
                    0 => (RestoreBound::ModelNames, MODELS),
                    1 => (RestoreBound::ModelNameBytes, MODEL_NAME_BYTES),
                    2 => (RestoreBound::AmendmentBytes, LIMITS.amendment_bytes),
                    3 => (RestoreBound::Batch, LIMITS.batch),
                    4 => (RestoreBound::AddressBytes, ADDRESS),
                    5 => (RestoreBound::IdentityBytes, LIMITS.identity_bytes),
                    6 => (RestoreBound::Holdings, LIMITS.holdings),
                    _ => unreachable!(),
                };
                assert_eq!(
                    domain.restore_failure(),
                    Some(RestoreFailure {
                        key,
                        cause: RestoreCause::Bound { which, held: u64::from(limit) + 1, limit: u64::from(limit) },
                    })
                );
                assert!(!domain.start_link(&env));
            }
        }
    }
}

#[test]
fn selected_ended_witness_preserves_missing_person_until_link() {
    let env = env();
    let (mut domain, mut out) = new_start(&env);
    load(&mut domain, &env, Stored::ReadConversation { party: 99, task: 7, through: 2 }, &mut out);
    assert!(domain.start_project(&env.limits, RestoreProjection::EndedConversation { task: 7 }));
    assert_eq!(domain.read_conversations().get(&(99, 7)), Some(&2));
    let result = std::panic::catch_unwind(core::panic::AssertUnwindSafe(|| domain.start_link(&env)));
    let error = result.expect_err("missing person must still assert");
    let message = error.downcast_ref::<String>().expect("formatted link panic");
    assert_eq!(message, "ReadConversation { party: 99, task: 7 } links missing person 99");
    assert!(out.is_empty());
    assert_eq!(domain.restore_failure(), None);
}

#[test]
fn selected_projection_phase_and_default_doors_assert() {
    for bad in 0_u8..9 {
        let env = env();
        let mut domain = Domain::new(&env.limits, Box::new([]), 1, Box::new([]));
        let mut out = Queue::uncapped();
        let result = std::panic::catch_unwind(core::panic::AssertUnwindSafe(|| {
            if bad != 0 {
                domain.start_restore(env.wall);
            }
            match bad {
                0 => {
                    assert!(domain.start_project(&env.limits, RestoreProjection::EndedConversation { task: 7 }));
                }
                1 => step(&mut domain, &env, Event::Waiting { task: 7, entries: Box::new([]) }, &mut out),
                2 => step(
                    &mut domain,
                    &env,
                    Event::ChatState {
                        party: 1,
                        project: 1,
                        task: 7,
                        last_reply: 1,
                        live: false,
                        seen: None,
                        replies: Box::new([]),
                    },
                    &mut out,
                ),
                3 => domain.restore_ended_conversation(7),
                4..=6 => {
                    assert!(domain.start_link(&env));
                    if bad >= 5 {
                        domain.start_sweep();
                    }
                    if bad == 6 {
                        assert!(domain.start_complete(&env));
                    }
                    assert!(domain.start_project(&env.limits, RestoreProjection::EndedConversation { task: 7 }));
                }
                7..=8 => {
                    let row = Stored::Person {
                        number: 1,
                        identity: Identity {
                            key: IdentityKey { provider: 0, subject: skein_lib::bytes::zeroed(17) },
                            login: Box::new([]),
                            name: Box::new([]),
                        },
                    };
                    load(&mut domain, &env, row, &mut out);
                    if bad == 7 {
                        domain.start_restore(env.wall);
                    } else {
                        domain.restore_ended_conversation(7);
                    }
                }
                _ => unreachable!(),
            }
        }));
        assert!(result.is_err(), "mixed/late door {bad}");
    }
}

#[test]
fn default_and_successful_starts_retain_no_fit_detail() {
    let env = env();
    for selected in [false, true] {
        let mut domain = Domain::new(&env.limits, Box::new([]), 1, Box::new([]));
        let mut out = Queue::uncapped();
        if selected {
            domain.start_restore(env.wall);
        }
        load(&mut domain, &env, person(1), &mut out);
        assert_eq!(domain.restore_failure(), None);
        if selected {
            assert!(domain.start_link(&env));
            domain.start_sweep();
            assert!(domain.start_complete(&env));
        } else {
            step(&mut domain, &env, Event::Restored, &mut out);
        }
        assert_eq!(domain.restore_failure(), None);
    }
    let mut domain = Domain::new(&env.limits, Box::new([]), 1, Box::new([]));
    let mut out = Queue::uncapped();
    load(
        &mut domain,
        &env,
        answer(1, Ask::Say { project: 1, task: 1, words: skein_lib::bytes::zeroed(9) }, Outcome::SignedOut),
        &mut out,
    );
    assert_eq!(
        out.pop(),
        Some(Request::RestoreRefused {
            key: Key::Answer(RequestKey { person: 1, key: [1; 16] }),
            refusal: Refusal::Limit { which: LimitKind::Restore },
        })
    );
    assert_eq!(domain.restore_failure(), None);
}

#[test]
fn selected_capacity_details_include_the_incoming_row() {
    for which in [
        RestoreBound::SignIns,
        RestoreBound::Roles,
        RestoreBound::ReadResults,
        RestoreBound::ReadConversations,
        RestoreBound::Requests,
    ] {
        let mut env = env();
        env.limits.sign_ins = 1;
        env.limits.projects = 1;
        env.limits.people = 1;
        env.limits.inbox_tasks = 1;
        env.limits.requests = 1;
        let (mut domain, mut out) = new_start(&env);
        load(&mut domain, &env, person(1), &mut out);
        let mut last = Key::Person(1);
        for number in 1..=2 {
            let row = match which {
                RestoreBound::SignIns => Stored::SignIn { number, person: 1, expires: Wall::from_nanos(1) },
                RestoreBound::Roles => {
                    Stored::Roles { project: u32::try_from(number).unwrap(), holdings: Box::new([]) }
                }
                RestoreBound::ReadResults => Stored::ReadResults { party: number, through: 0 },
                RestoreBound::ReadConversations => Stored::ReadConversation { party: 1, task: number, through: 0 },
                RestoreBound::Requests if number == 1 => {
                    answer(1, Ask::Stop { project: 0, task: 0 }, Outcome::SignedOut)
                }
                RestoreBound::Requests => Stored::Acted {
                    key: ActKey { connector: 2, key: [2; 16] },
                    act: Act::RemoveAccount { account: 0 },
                    outcome: Outcome::AccountChanged,
                    at: Wall::EPOCH,
                },
                RestoreBound::IdentityBytes
                | RestoreBound::People
                | RestoreBound::Holdings
                | RestoreBound::ModelNames
                | RestoreBound::ModelNameBytes
                | RestoreBound::Subagents
                | RestoreBound::Operations
                | RestoreBound::Batch
                | RestoreBound::Words
                | RestoreBound::AmendmentBytes
                | RestoreBound::NameBytes
                | RestoreBound::ProgramNameBytes
                | RestoreBound::Uses
                | RestoreBound::ProgramHosts
                | RestoreBound::ProgramHostNameBytes
                | RestoreBound::AddressBytes
                | RestoreBound::NamedTasks
                | RestoreBound::KeptPending => unreachable!("capacity cases only"),
            };
            last = row.key();
            load(&mut domain, &env, row, &mut out);
            if number == 1 {
                assert!(out.is_empty());
                assert_eq!(domain.restore_failure(), None);
            }
        }
        assert_eq!(
            domain.restore_failure(),
            Some(RestoreFailure { key: last, cause: RestoreCause::Bound { which, held: 2, limit: 1 } })
        );
        assert_eq!(out.len(), 1);
    }
}

#[test]
#[expect(clippy::too_many_lines, reason = "stored exact/one-over cases name each nested representation bound")]
fn selected_nested_causes_keep_the_outer_stored_key_and_first_fit_order() {
    for field in 0_u8..13 {
        for extra in [0, 1] {
            let mut env = env();
            let (ask, outcome, which, held, limit) = match field {
                0 => {
                    let mut amendment = fixtures::unchanged(b"");
                    amendment.llm = Some(LlmChoice {
                        account: None,
                        main: ModelEffort { model: 255, effort: Some(255) },
                        subagents: copies(ModelEffort { model: 255, effort: None }, MODELS + extra),
                    });
                    env.limits.amendment_bytes = 4096;
                    (
                        Ask::Propose {
                            project: 1,
                            action: Action::Amend { task: 1, revision: 0, amendment: Box::new(amendment) },
                            reason: Box::new([]),
                        },
                        Outcome::SignedOut,
                        RestoreBound::Subagents,
                        MODELS + extra,
                        MODELS,
                    )
                }
                1 => (
                    Ask::Compound {
                        project: 0,
                        operations: {
                            let mut ops = skein_lib::List::with_capacity(LIMITS.operations + extra);
                            for _ in 0..LIMITS.operations + extra {
                                ops.push(Operation::Stop { task: 1 }).unwrap();
                            }
                            ops
                        },
                    },
                    Outcome::SignedOut,
                    RestoreBound::Operations,
                    LIMITS.operations + extra,
                    LIMITS.operations,
                ),
                2 => (
                    Ask::Propose {
                        project: 1,
                        action: Action::Batch(Box::new([fixtures::member(&skein_lib::bytes::zeroed(
                            usize::try_from(LIMITS.words + extra).unwrap(),
                        ))])),
                        reason: Box::new([]),
                    },
                    Outcome::SignedOut,
                    RestoreBound::Words,
                    LIMITS.words + extra,
                    LIMITS.words,
                ),
                3 => (
                    Ask::Rename {
                        project: 1,
                        name: "a".repeat(NAME_BYTES + usize::try_from(extra).unwrap()).into_boxed_str(),
                    },
                    Outcome::SignedOut,
                    RestoreBound::NameBytes,
                    u32::try_from(NAME_BYTES).unwrap() + extra,
                    u32::try_from(NAME_BYTES).unwrap(),
                ),
                4 => (
                    Ask::AccountChange {
                        holder: AccountHolder::Own,
                        change: AccountChange::DeclareProgram {
                            provider: 0,
                            join: None,
                            name: skein_lib::bytes::zeroed(32 + usize::try_from(extra).unwrap()),
                        },
                    },
                    Outcome::SignedOut,
                    RestoreBound::ProgramNameBytes,
                    32 + extra,
                    32,
                ),
                5 => {
                    env.limits.words = 4096;
                    (
                        Ask::AccountChange {
                            holder: AccountHolder::Own,
                            change: AccountChange::SetUses {
                                account: 0,
                                uses: Some(copies(Use::Party(1), USES + extra)),
                            },
                        },
                        Outcome::SignedOut,
                        RestoreBound::Uses,
                        USES + extra,
                        USES,
                    )
                }
                6 => (
                    Ask::Stop { project: 0, task: 0 },
                    Outcome::AccountRemoved {
                        confirmed: false,
                        failed_hosts: copies(
                            AccountHostFailure { name: Box::new([]), why: AccountHostWhy::Lost },
                            LIMITS.program_hosts + extra,
                        ),
                    },
                    RestoreBound::ProgramHosts,
                    LIMITS.program_hosts + extra,
                    LIMITS.program_hosts,
                ),
                7 => (
                    Ask::Stop { project: 0, task: 0 },
                    Outcome::AccountSignedOut {
                        confirmed: false,
                        failed_hosts: Box::new([AccountHostFailure {
                            name: skein_lib::bytes::zeroed(
                                usize::try_from(LIMITS.program_host_name_bytes + extra).unwrap(),
                            ),
                            why: AccountHostWhy::Lost,
                        }]),
                    },
                    RestoreBound::ProgramHostNameBytes,
                    LIMITS.program_host_name_bytes + extra,
                    LIMITS.program_host_name_bytes,
                ),
                8 => (
                    Ask::Stop { project: 0, task: 0 },
                    Outcome::CompoundRefused {
                        at: 0,
                        why: Refusal::Circle { tasks: copies(7_u64, NAMED_TASKS + extra), cut: false },
                    },
                    RestoreBound::NamedTasks,
                    NAMED_TASKS + extra,
                    NAMED_TASKS,
                ),
                9 => (
                    Ask::Stop { project: 0, task: 0 },
                    Outcome::Refused(Refusal::Kept { pending: copies(7_u64, 64 + extra) }),
                    RestoreBound::KeptPending,
                    64 + extra,
                    64,
                ),
                10 => {
                    let mut amendment = fixtures::unchanged(b"");
                    amendment.instructions =
                        Some(skein_lib::bytes::zeroed(usize::try_from(LIMITS.amendment_bytes + extra).unwrap()));
                    (
                        Ask::Propose {
                            project: 1,
                            action: Action::Amend { task: 1, revision: 0, amendment: Box::new(amendment) },
                            reason: Box::new([]),
                        },
                        Outcome::SignedOut,
                        RestoreBound::AmendmentBytes,
                        LIMITS.amendment_bytes + extra,
                        LIMITS.amendment_bytes,
                    )
                }
                11 => (
                    Ask::AnswerPerson {
                        project: 1,
                        task: 1,
                        revision: 0,
                        result: PersonResult::Approved {
                            state: Box::new([0]),
                            words: Some(skein_lib::bytes::zeroed(usize::try_from(LIMITS.words - 1 + extra).unwrap())),
                        },
                    },
                    Outcome::SignedOut,
                    RestoreBound::Words,
                    LIMITS.words + extra,
                    LIMITS.words,
                ),
                12 => {
                    env.limits.words = 32;
                    (
                        Ask::MakeService {
                            project: 1,
                            name: skein_lib::bytes::zeroed(usize::try_from(LIMITS.identity_bytes - 8 + extra).unwrap()),
                            role: Role::Member,
                        },
                        Outcome::SignedOut,
                        RestoreBound::IdentityBytes,
                        LIMITS.identity_bytes + extra,
                        LIMITS.identity_bytes,
                    )
                }
                _ => unreachable!(),
            };
            let (mut domain, mut out) = new_start(&env);
            load(&mut domain, &env, person(1), &mut out);
            let row = answer(19, ask, outcome);
            let key = row.key();
            load(&mut domain, &env, row, &mut out);
            if extra == 0 {
                assert_eq!(domain.restore_failure(), None, "field {field} exactly fits");
                assert!(out.is_empty());
            } else {
                assert_eq!(
                    domain.restore_failure(),
                    Some(RestoreFailure {
                        key,
                        cause: RestoreCause::Bound { which, held: u64::from(held), limit: u64::from(limit) },
                    }),
                    "field {field}"
                );
                let first = domain.restore_failure();
                load(&mut domain, &env, person(2), &mut out);
                assert!(!domain.start_link(&env));
                assert_eq!(domain.restore_failure(), first);
                assert_eq!(out.len(), 1);
            }
        }
    }
}
