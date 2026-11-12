//! Declarations reserve a durable name before any host operation is possible.
use jig_accounts_world::{
    HOLDER, LIMITS, PROVIDER, World, account, held, issued, program, provider, providers, source,
};
use jig_domain_accounts::{Event, Holder, Kind, Refusal, Request, Stored, programs};
use jig_domain_link::{Form, Grant};
use skein_lib::{Env, Queue};
const OTHER: Holder = Holder::Party(8);
const JOINED_ACCOUNT: u32 = 20;
fn configured() -> jig_domain_accounts::Providers {
    let mut offered = provider(PROVIDER, true, true);
    offered.offers.programs = Box::new([b"claude_code".as_slice().into()]);
    providers(vec![offered])
}
fn fresh() -> World {
    let mut world = World::with_start(jig_accounts_world::LIMITS, configured(), &[]);
    world.step(Event::Want);
    world
}
fn declare(
    world: &mut World,
    holder: Holder,
    provider: u32,
    join: Option<u32>,
    name: &[u8],
) -> (Result<u32, programs::DeclarationRefusal>, Vec<Request>) {
    let mut out = Queue::uncapped();
    let env = Env { now: world.now, wall: world.wall, limits: LIMITS };
    let result = programs::declare(&mut world.domain, &env, holder, provider, join, name.into(), &mut out);
    let mut requests = Vec::new();
    while let Some(request) = out.pop() {
        requests.push(request);
    }
    (result, requests)
}
#[test]
fn a_declaration_saves_its_reservation_atomically() {
    let mut world = fresh();
    assert_eq!(declare(&mut world, HOLDER, PROVIDER, None, b"personal"), (Ok(1), declaration_requests(0, 1, 2)));
    let account = world.domain.account(0).expect("kept account");
    assert!(account.plan());
    assert_eq!((account.subject()).len(), 0);
}
fn declaration_requests(account: u32, source: u32, next: u32) -> Vec<Request> {
    vec![
        Request::Save { record: held(account, b"") },
        Request::Save { record: issued(source, b"personal") },
        Request::Save { record: program(account, source, 0, b"personal") },
        Request::Save { record: Stored::Next { number: next } },
        Request::Programs {
            names: Box::new([jig_domain_accounts::Program {
                name: jig_domain_link::SourceName::new(b"personal").expect("admitted fixture name"),
                cleanup: false,
            }]),
        },
        Request::Granted { grant: named_grant(source, u64::from(next)) },
    ]
}
fn forget(world: &mut World) -> Vec<Request> {
    world.step(Event::Change {
        request: skein_lib::Token::new(92),
        holder: HOLDER,
        change: jig_domain_accounts::Change::Remove { account: 0 },
    })
}
#[test]
fn a_forgotten_program_name_is_declared_again_by_its_holder() {
    let mut world = fresh();
    let mut store = std::collections::BTreeMap::new();
    let (_, first) = declare(&mut world, HOLDER, PROVIDER, None, b"personal");
    jig_accounts_world::commit(&mut store, &first);
    jig_accounts_world::commit(&mut store, &forget(&mut world));
    jig_accounts_world::commit(&mut store, &targets(&mut world, &[0]));
    jig_accounts_world::commit(&mut store, &ended(&mut world, 0, jig_domain_accounts::ProgramResult::SignedOut));
    let mut expected = declaration_requests(2, 3, 4);
    expected.insert(1, Request::Erase { key: jig_domain_accounts::Key::Issued { number: 1 } });
    expected.push(Request::CheckProgram {
        name: jig_domain_link::SourceName::new(b"personal").expect("admitted fixture name"),
    });
    let (result, requests) = declare(&mut world, HOLDER, PROVIDER, None, b"personal");
    assert_eq!((result, &requests), (Ok(3), &expected));
    assert_eq!(programs::issued(&world.domain, b"personal"), Some((3, HOLDER)));
    jig_accounts_world::commit(&mut store, &requests);
    let mut again = World::with_start(LIMITS, configured(), &store.into_values().collect::<Vec<_>>());
    again.step(Event::Want);
    assert_eq!(programs::issued(&again.domain, b"personal"), Some((3, HOLDER)));
}
#[test]
fn a_program_joins_at_the_last_source_place() {
    let mut world = World::with_start(
        jig_accounts_world::LIMITS,
        configured(),
        &[held(JOINED_ACCOUNT, b""), source(JOINED_ACCOUNT, 21, 0, Kind::Key)],
    );
    world.step(Event::Want);
    assert_eq!(declare(&mut world, HOLDER, PROVIDER, Some(20), b"personal").0, Ok(22));
    assert_eq!(
        world.domain.account(20).expect("kept account").sources().iter().map(|(p, s)| (*p, *s)).collect::<Vec<_>>(),
        vec![(0, 21), (1, 22)]
    );
}
#[test]
fn a_foreign_holder_cannot_join_an_account() {
    let mut world = World::with_start(
        jig_accounts_world::LIMITS,
        configured(),
        &[account(JOINED_ACCOUNT, PROVIDER, OTHER, b""), source(JOINED_ACCOUNT, 21, 0, Kind::Key)],
    );
    world.step(Event::Want);
    assert_eq!(
        declare(&mut world, HOLDER, PROVIDER, Some(20), b"personal"),
        (Err(programs::DeclarationRefusal::Account(Refusal::Unknown)), vec![])
    );
    assert_eq!(world.domain.next_number(), 22);
}
#[test]
fn a_provider_must_offer_a_program() {
    let mut world = World::with_start(jig_accounts_world::LIMITS, providers(vec![provider(PROVIDER, true, true)]), &[]);
    world.step(Event::Want);
    assert_eq!(
        declare(&mut world, HOLDER, PROVIDER, None, b"personal"),
        (Err(programs::DeclarationRefusal::Account(Refusal::Provider { provider: PROVIDER })), vec![])
    );
}
#[test]
fn invalid_names_allocate_nothing() {
    for name in [b"".as_slice(), b"Upper", b"a_b", &[b'a'; 33]] {
        let mut world = fresh();
        assert_eq!(
            declare(&mut world, HOLDER, PROVIDER, None, name),
            (Err(programs::DeclarationRefusal::Name), vec![])
        );
        assert_eq!(world.domain.next_number(), 0);
    }
}
#[test]
fn names_accept_the_exact_length_bound() {
    let mut world = fresh();
    assert_eq!(declare(&mut world, HOLDER, PROVIDER, None, &[b'a'; 32]).0, Ok(1));
}
#[test]
fn reservations_admit_only_their_retired_names_holder() {
    use programs::DeclarationRefusal::Taken;
    let live = vec![held(0, b""), program(0, 1, 0, b"personal"), issued(1, b"personal")];
    let retired = vec![issued(50_000, b"retired")];
    for (label, records, holder, name, expected, next, reservation) in [
        ("live holder", live.clone(), HOLDER, b"personal".as_slice(), Err(Taken), 2, Some((1, HOLDER))),
        ("live other", live, OTHER, b"personal", Err(Taken), 2, Some((1, HOLDER))),
        ("retired other", retired.clone(), OTHER, b"retired", Err(Taken), 50_001, Some((50_000, HOLDER))),
        ("retired holder", retired.clone(), HOLDER, b"retired", Ok(50_002), 50_003, Some((50_002, HOLDER))),
        ("fresh other", retired, OTHER, b"fresh", Ok(50_002), 50_003, Some((50_002, OTHER))),
    ] {
        let mut world = World::with_start(LIMITS, configured(), &records);
        world.step(Event::Want);
        let (result, requests) = declare(&mut world, holder, PROVIDER, None, name);
        assert_eq!(result, expected, "{label}");
        if expected.is_err() {
            assert!(requests.is_empty(), "{label}: refused without outputs");
        }
        assert_eq!(world.domain.next_number(), next, "{label}");
        assert_eq!(programs::issued(&world.domain, name), reservation, "{label}");
    }
}
#[test]
fn the_lifetime_name_bound_refuses_before_allocating() {
    let rows = (0..LIMITS.program_names).map(|n| issued(n, format!("old-{n}").as_bytes())).collect::<Vec<_>>();
    let mut world = World::with_start(LIMITS, configured(), &rows);
    world.step(Event::Want);
    let next = LIMITS.program_names;
    for (name, expected, accounts, counter) in [
        (b"fresh".as_slice(), Err(programs::DeclarationRefusal::Names), 0, next),
        (b"old-0", Ok(next + 1), 1, next + 2),
        (b"fresh", Err(programs::DeclarationRefusal::Names), 1, next + 2),
    ] {
        let (result, requests) = declare(&mut world, HOLDER, PROVIDER, None, name);
        assert_eq!(result, expected, "{name:?}");
        if expected.is_err() {
            assert!(requests.is_empty(), "{name:?}: refused without outputs");
        }
        assert_eq!(world.domain.accounts(), accounts, "{name:?}");
        assert_eq!(world.domain.next_number(), counter, "{name:?}");
    }
    assert_eq!(programs::issued(&world.domain, b"old-0"), Some((next + 1, HOLDER)));
}

fn sign_out(world: &mut World, holder: Holder) -> Vec<Request> {
    world.step(Event::SignOut { request: skein_lib::Token::new(91), holder, source: 1 })
}

fn targets(world: &mut World, workers: &[u32]) -> Vec<Request> {
    world.step(Event::ProgramTargets {
        source: 1,
        hosts: workers.iter().map(|number| jig_accounts_world::worker(*number)).collect(),
    })
}

fn ended(world: &mut World, worker: u32, result: jig_domain_accounts::ProgramResult) -> Vec<Request> {
    world.step(Event::ProgramSignedOut { source: 1, host: jig_accounts_world::worker(worker), result })
}

#[test]
fn signing_out_a_program_retains_its_declaration() {
    let mut world = fresh();
    assert_eq!(declare(&mut world, HOLDER, PROVIDER, None, b"personal").0, Ok(1));
    sign_out(&mut world, HOLDER);
    assert_eq!(
        targets(&mut world, &[]),
        vec![Request::Answer {
            request: skein_lib::Token::new(91),
            answer: jig_domain_accounts::Answer::SignedOut { confirmed: true, failed_hosts: Box::new([]) },
        }]
    );
    assert!(matches!(world.domain.account(0), Some(account) if account.sources().len() == 1));
}
#[test]
fn a_foreign_program_sign_out_emits_no_host_command() {
    let mut world = fresh();
    assert_eq!(declare(&mut world, HOLDER, PROVIDER, None, b"personal").0, Ok(1));
    assert_eq!(
        sign_out(&mut world, OTHER),
        vec![Request::Answer {
            request: skein_lib::Token::new(91),
            answer: jig_domain_accounts::Answer::Refused { refusal: Refusal::Unknown }
        }]
    );
}
#[test]
fn a_program_answer_names_every_unconfirmed_configured_position() {
    use jig_domain_accounts::{ProgramHost, ProgramResult};
    let mut world = fresh();
    assert_eq!(declare(&mut world, HOLDER, PROVIDER, None, b"personal").0, Ok(1));
    sign_out(&mut world, HOLDER);
    assert_eq!(targets(&mut world, &[0, 1, 2]).len(), 0);
    assert_eq!(ended(&mut world, 1, ProgramResult::SignedOut).len(), 0);
    assert_eq!(ended(&mut world, 0, ProgramResult::NotInContact).len(), 0);
    assert_eq!(
        ended(&mut world, 2, ProgramResult::Lost),
        vec![Request::Answer {
            request: skein_lib::Token::new(91),
            answer: jig_domain_accounts::Answer::SignedOut {
                confirmed: false,
                failed_hosts: Box::new([
                    ProgramHost {
                        worker: jig_accounts_world::worker(0),
                        result: jig_domain_accounts::ProgramFailure::NotInContact
                    },
                    ProgramHost {
                        worker: jig_accounts_world::worker(2),
                        result: jig_domain_accounts::ProgramFailure::Lost
                    },
                ]),
            },
        }]
    );
}
#[test]
fn a_late_program_terminal_cannot_answer_twice() {
    let mut world = fresh();
    assert_eq!(declare(&mut world, HOLDER, PROVIDER, None, b"personal").0, Ok(1));
    sign_out(&mut world, HOLDER);
    targets(&mut world, &[0]);
    ended(&mut world, 0, jig_domain_accounts::ProgramResult::Lost);
    assert_eq!(ended(&mut world, 0, jig_domain_accounts::ProgramResult::SignedOut).len(), 0);
}
#[test]
fn restart_preserves_desired_program_absence() {
    let mut world = fresh();
    let (_, declared) = declare(&mut world, HOLDER, PROVIDER, None, b"personal");
    let mut records: Vec<_> = declared
        .into_iter()
        .filter_map(|request| match request {
            Request::Save { record } => Some(record),
            Request::Programs { .. }
            | Request::ReadList { .. }
            | Request::CheckProgram { .. }
            | Request::SignOutProgram { .. }
            | Request::Component(_)
            | Request::Signing(_)
            | Request::Answer { .. }
            | Request::Granted { .. }
            | Request::Erase { .. }
            | Request::Closed => None,
        })
        .collect();
    for request in sign_out(&mut world, HOLDER) {
        if let Request::Save { record } = request {
            records.retain(|old| old.key() != record.key());
            records.push(record);
        }
    }
    records.sort_by_key(|row| match row {
        Stored::Issued { .. } => 0,
        Stored::Account { .. } => 1,
        Stored::Source { .. } => 2,
        Stored::Next { .. } => 3,
        Stored::Facts { .. } => 4,
    });
    let mut again = World::with_start(jig_accounts_world::LIMITS, configured(), &records);
    assert!(again.step(Event::Want).iter().any(|request| matches!(request,
        Request::Programs { names } if names.len() == 1 && names[0].cleanup)));
}
#[test]
fn an_explicit_holder_check_permits_signing_in_again() {
    let mut world = fresh();
    assert_eq!(declare(&mut world, HOLDER, PROVIDER, None, b"personal").0, Ok(1));
    sign_out(&mut world, HOLDER);
    targets(&mut world, &[]);
    let mut out = Queue::uncapped();
    assert_eq!(programs::check(&mut world.domain, HOLDER, 1, &mut out), Ok(()));
    let mut requests = Vec::new();
    while let Some(request) = out.pop() {
        requests.push(request);
    }
    assert!(
        requests
            .iter()
            .any(|request| matches!(request, Request::Save { record: Stored::Issued { signed_out: false, .. } }))
    );
}
#[test]
fn removing_a_program_account_retains_cleanup_evidence() {
    let mut world = fresh();
    assert_eq!(declare(&mut world, HOLDER, PROVIDER, None, b"personal").0, Ok(1));
    let requests = world.step(Event::Change {
        request: skein_lib::Token::new(92),
        holder: HOLDER,
        change: jig_domain_accounts::Change::Remove { account: 0 },
    });
    assert!(world.domain.account(0).is_none());
    assert!(requests.iter().any(|request| matches!(request, Request::Programs { names } if names[0].cleanup)));
}

#[test]
fn a_change_answers_a_program_declaration() {
    let mut world = fresh();
    let request = skein_lib::Token::new(41);
    for answer in [
        jig_domain_accounts::Answer::ProgramDeclared { source: 1, name: b"personal".as_slice().into() },
        jig_domain_accounts::Answer::Refused { refusal: Refusal::NameTaken },
    ] {
        let rows = world.step(Event::Change {
            request,
            holder: HOLDER,
            change: jig_domain_accounts::Change::DeclareProgram {
                provider: PROVIDER,
                join: None,
                name: b"personal".as_slice().into(),
            },
        });
        assert_eq!(rows.last(), Some(&Request::Answer { request, answer }));
        assert_eq!(world.domain.next_number(), 2);
    }
}
#[test]
fn a_pending_removal_blocks_reviving_its_program_name() {
    let mut world = fresh();
    declare(&mut world, HOLDER, PROVIDER, None, b"personal").0.expect("first declaration");
    forget(&mut world);
    targets(&mut world, &[0]);
    assert_eq!(
        declare(&mut world, HOLDER, PROVIDER, None, b"personal"),
        (
            Err(programs::DeclarationRefusal::Account(Refusal::Limit { bound: jig_domain_accounts::Bound::Sources })),
            vec![]
        )
    );
    assert_eq!(world.domain.next_number(), 2);
    assert_eq!(programs::issued(&world.domain, b"personal"), Some((1, HOLDER)));
}
#[test]
fn a_foreign_check_emits_no_host_command() {
    let mut world = fresh();
    let (source, _) = declare(&mut world, HOLDER, PROVIDER, None, b"personal");
    let source = source.expect("declared source");
    let request = skein_lib::Token::new(42);
    assert_eq!(
        world.step(Event::Change {
            request,
            holder: OTHER,
            change: jig_domain_accounts::Change::CheckProgram { source }
        }),
        vec![Request::Answer { request, answer: jig_domain_accounts::Answer::Refused { refusal: Refusal::Unknown } }]
    );
}
#[test]
fn a_holder_check_is_answered_after_dispatch() {
    let mut world = fresh();
    let (source, _) = declare(&mut world, HOLDER, PROVIDER, None, b"personal");
    let source = source.expect("declared source");
    let request = skein_lib::Token::new(43);
    let rows = world.step(Event::Change {
        request,
        holder: HOLDER,
        change: jig_domain_accounts::Change::CheckProgram { source },
    });
    assert_eq!(
        rows[rows.len() - 2..],
        [
            Request::CheckProgram {
                name: jig_domain_link::SourceName::new(b"personal".as_slice()).expect("admitted fixture name")
            },
            Request::Answer { request, answer: jig_domain_accounts::Answer::Done }
        ]
    );
}

#[test]
fn an_early_terminal_waits_for_its_target_snapshot() {
    let mut world = fresh();
    declare(&mut world, HOLDER, PROVIDER, None, b"personal").0.expect("declaration");
    sign_out(&mut world, HOLDER);
    assert_eq!(ended(&mut world, 0, jig_domain_accounts::ProgramResult::SignedOut).len(), 0);
    assert_eq!(
        targets(&mut world, &[0]),
        vec![Request::Answer {
            request: skein_lib::Token::new(91),
            answer: jig_domain_accounts::Answer::SignedOut { confirmed: true, failed_hosts: Box::new([]) }
        }]
    );
}
#[test]
fn an_early_failure_survives_the_target_snapshot() {
    let mut world = fresh();
    declare(&mut world, HOLDER, PROVIDER, None, b"personal").0.expect("declaration");
    sign_out(&mut world, HOLDER);
    ended(&mut world, 0, jig_domain_accounts::ProgramResult::Failed);
    assert_eq!(
        targets(&mut world, &[0]),
        vec![Request::Answer {
            request: skein_lib::Token::new(91),
            answer: jig_domain_accounts::Answer::SignedOut {
                confirmed: false,
                failed_hosts: Box::new([jig_domain_accounts::ProgramHost {
                    worker: jig_accounts_world::worker(0),
                    result: jig_domain_accounts::ProgramFailure::Failed
                }])
            }
        }]
    );
}
#[test]
fn closing_answers_a_pending_program_sign_out() {
    let mut world = fresh();
    declare(&mut world, HOLDER, PROVIDER, None, b"personal").0.expect("declaration");
    sign_out(&mut world, HOLDER);
    let rows = world.step(Event::Close);
    let answers: Vec<_> = rows.into_iter().filter(|row| matches!(row, Request::Answer { .. })).collect();
    assert_eq!(
        answers,
        vec![Request::Answer {
            request: skein_lib::Token::new(91),
            answer: jig_domain_accounts::Answer::Refused { refusal: Refusal::Closing }
        }]
    );
}
#[test]
fn a_pending_program_sign_out_blocks_account_removal() {
    let mut world = fresh();
    declare(&mut world, HOLDER, PROVIDER, None, b"personal").0.expect("declaration");
    sign_out(&mut world, HOLDER);
    let request = skein_lib::Token::new(92);
    assert_eq!(
        world.step(Event::Change {
            request,
            holder: HOLDER,
            change: jig_domain_accounts::Change::Remove { account: 0 }
        }),
        vec![Request::Answer {
            request,
            answer: jig_domain_accounts::Answer::Refused {
                refusal: Refusal::Limit { bound: jig_domain_accounts::Bound::Sources }
            }
        }]
    );
}

#[test]
fn an_accepted_program_name_is_a_fleet_source_name() {
    let mut world = fresh();
    let name = b"a-0123456789";
    let (result, rows) = declare(&mut world, HOLDER, PROVIDER, None, name);
    assert!(result.is_ok());
    let Some(Request::Programs { names }) = rows.iter().find(|row| matches!(row, Request::Programs { .. })) else {
        panic!("published names")
    };
    assert_eq!(Some(names[0].name), jig_domain_link::SourceName::new(name));
}
#[test]
fn a_refused_program_name_is_not_a_fleet_source_name() {
    let mut world = fresh();
    let name = b"not portable";
    assert_eq!(declare(&mut world, HOLDER, PROVIDER, None, name), (Err(programs::DeclarationRefusal::Name), vec![]));
    assert_eq!(jig_domain_link::SourceName::new(name), None);
}

#[test]
fn a_pending_program_sign_out_blocks_a_holder_check() {
    let mut world = fresh();
    declare(&mut world, HOLDER, PROVIDER, None, b"personal").0.expect("declaration");
    sign_out(&mut world, HOLDER);
    let mut out = Queue::uncapped();
    assert_eq!(
        programs::check(&mut world.domain, HOLDER, 1, &mut out),
        Err(Refusal::Limit { bound: jig_domain_accounts::Bound::Sources })
    );
    assert!(out.is_empty());
}
#[test]
fn an_early_program_terminal_finishes_account_removal() {
    let mut world = fresh();
    declare(&mut world, HOLDER, PROVIDER, None, b"personal").0.expect("declaration");
    let request = skein_lib::Token::new(92);
    world.step(Event::Change { request, holder: HOLDER, change: jig_domain_accounts::Change::Remove { account: 0 } });
    ended(&mut world, 0, jig_domain_accounts::ProgramResult::SignedOut);
    assert_eq!(
        targets(&mut world, &[0]),
        vec![Request::Answer {
            request,
            answer: jig_domain_accounts::Answer::Removed { confirmed: true, failed_hosts: Box::new([]) }
        }]
    );
}

fn removal_failures(
    first: jig_domain_accounts::ProgramResult,
    last: jig_domain_accounts::ProgramResult,
) -> Vec<Request> {
    let mut world = fresh();
    declare(&mut world, HOLDER, PROVIDER, None, b"personal").0.expect("first declaration");
    declare(&mut world, HOLDER, PROVIDER, Some(0), b"work").0.expect("second declaration");
    let request = skein_lib::Token::new(92);
    world.step(Event::Change { request, holder: HOLDER, change: jig_domain_accounts::Change::Remove { account: 0 } });
    for source in [1, 2] {
        world.step(Event::ProgramTargets { source, hosts: Box::new([jig_accounts_world::worker(0)]) });
    }
    assert_eq!(
        world.step(Event::ProgramSignedOut { source: 1, host: jig_accounts_world::worker(0), result: first }).len(),
        0
    );
    world.step(Event::ProgramSignedOut { source: 2, host: jig_accounts_world::worker(0), result: last })
}
fn removal_answer(result: jig_domain_accounts::ProgramFailure) -> Vec<Request> {
    vec![Request::Answer {
        request: skein_lib::Token::new(92),
        answer: jig_domain_accounts::Answer::Removed {
            confirmed: false,
            failed_hosts: Box::new([jig_domain_accounts::ProgramHost {
                worker: jig_accounts_world::worker(0),
                result,
            }]),
        },
    }]
}
#[test]
fn account_removal_keeps_the_most_severe_worker_failure() {
    use jig_domain_accounts::{ProgramFailure as Failure, ProgramResult as Result};
    for (first, last) in [(Result::Lost, Result::Failed), (Result::Failed, Result::Lost)] {
        assert_eq!(removal_failures(first, last), removal_answer(Failure::Lost));
    }
}
#[test]
fn account_removal_prefers_failure_to_absent_contact() {
    use jig_domain_accounts::{ProgramFailure as Failure, ProgramResult as Result};
    for (first, last) in [(Result::NotInContact, Result::Failed), (Result::Failed, Result::NotInContact)] {
        assert_eq!(removal_failures(first, last), removal_answer(Failure::Failed));
    }
}

#[test]
fn an_issued_name_answers_its_source_holder() {
    for signed_out in [false, true] {
        let world = World::with_start(
            jig_accounts_world::LIMITS,
            configured(),
            &[Stored::Issued { number: 21, holder: HOLDER, name: b"personal".as_slice().into(), signed_out }],
        );
        assert_eq!(programs::issued(&world.domain, b"personal"), Some((21, HOLDER)));
    }
}

fn named_grant(source: u32, generation: u64) -> Grant {
    Grant {
        form: Form::Program(jig_domain_link::SourceName::new(b"personal").expect("source name")),
        account: source,
        generation,
        valid: jig_domain_accounts::PROGRAM_VALID,
    }
}
#[test]
fn a_usable_program_source_is_granted_by_its_name() {
    let mut world = fresh();
    let (source, requests) = declare(&mut world, HOLDER, PROVIDER, None, b"personal");
    let source = source.expect("declared source");
    assert!(requests.contains(&Request::Granted { grant: named_grant(source, 2) }));
}
#[test]
fn a_program_grant_never_lapses() {
    let mut world = fresh();
    let (source, _) = declare(&mut world, HOLDER, PROVIDER, None, b"personal");
    let source = source.expect("declared source");
    assert_eq!(world.domain.source_grant(source, skein_lib::Time::from_nanos(u64::MAX)), Some(named_grant(source, 2)));
    assert_eq!(world.domain.next_deadline(), None);
}
#[test]
fn a_new_login_raises_the_program_grant_s_generation() {
    let mut world = fresh();
    let (source, _) = declare(&mut world, HOLDER, PROVIDER, None, b"personal");
    let source = source.expect("declared source");
    sign_out(&mut world, HOLDER);
    world.step(Event::ProgramTargets { source, hosts: Box::new([]) });
    let requests = world.step(Event::Change {
        request: skein_lib::Token::new(44),
        holder: HOLDER,
        change: jig_domain_accounts::Change::CheckProgram { source },
    });
    assert!(requests.contains(&Request::Save { record: Stored::Next { number: 3 } }));
    assert!(requests.contains(&Request::Granted { grant: named_grant(source, 3) }));
}
#[test]
fn a_restored_program_grant_uses_the_durable_counter_epoch() {
    for held in [None, Some(jig_domain_accounts::Held::Refused)] {
        let mut row = program(0, 1, 0, b"personal");
        if let Stored::Source { held: kept, .. } = &mut row {
            *kept = held;
        }
        let mut world = World::with_start(
            jig_accounts_world::LIMITS,
            configured(),
            &[jig_accounts_world::held(0, b""), row, issued(1, b"personal"), Stored::Next { number: 3 }],
        );
        let requests = world.step(Event::Want);
        assert_eq!(requests.contains(&Request::Granted { grant: named_grant(1, 3) }), held.is_none());
        assert!(!requests.iter().any(|request| matches!(request, Request::Save { record: Stored::Source { .. } })));
    }
}

#[test]
fn signing_out_withdraws_the_program_grant() {
    let mut world = fresh();
    declare(&mut world, HOLDER, PROVIDER, None, b"personal").0.expect("source");
    sign_out(&mut world, HOLDER);
    assert_eq!(world.domain.source_grant(1, world.now), None);
    assert_eq!(
        world.domain.select(
            jig_domain_accounts::Use::Party(7),
            PROVIDER,
            jig_domain_accounts::Ways { program: true, direct: false },
            Some(0),
            None,
            &[jig_domain_accounts::Coverage { account: 0, serves: true }],
            &[jig_domain_accounts::HostSource { source: 1, usable: true }],
            world.now,
        ),
        jig_domain_accounts::Selection::Wait { account: 0, source: Some(1), why: jig_domain_accounts::WaitWhy::Host }
    );
}
#[test]
fn a_login_check_with_an_exhausted_epoch_changes_nothing() {
    // Restoring the counter before Want gives the greatest representable epoch.
    let rows = [held(0, b""), program(0, 1, 0, b"personal"), issued(1, b"personal"), Stored::Next { number: u32::MAX }];
    let mut world = World::with_start(jig_accounts_world::LIMITS, configured(), &rows);
    world.step(Event::Want);
    let mut out = Queue::uncapped();
    assert_eq!(programs::check(&mut world.domain, HOLDER, 1, &mut out), Err(Refusal::Numbers));
    assert!(out.is_empty());
    assert_eq!(world.domain.source_grant(1, world.now), Some(named_grant(1, u64::from(u32::MAX))));
}

/// Each accepted report replaces only what it says (accounts.md, 3.7).
#[test]
fn program_reports_follow_the_live_source() {
    use jig_domain_accounts::{AccountFact, FactValue, KIND_EMAIL, Key, ORIGIN_PROGRAM};
    use skein_lib::{Token, Wall};
    const ACCOUNT: u32 = 0;
    const SOURCE: u32 = 1;
    const RETIRED_ACCOUNT: u32 = 2; // The first declaration consumes account 0 and source 1.
    let mut world = fresh();
    assert_eq!(declare(&mut world, HOLDER, PROVIDER, None, b"personal").0, Ok(SOURCE));
    // Keep a retired reservation beside the live source.
    declare(&mut world, HOLDER, PROVIDER, None, b"retired").0.expect("second declaration");
    world.step(Event::Change {
        request: Token::new(93),
        holder: HOLDER,
        change: jig_domain_accounts::Change::Remove { account: RETIRED_ACCOUNT },
    });
    let mut held = Vec::new();
    // A lapse or a status without an email sends no report to this child.
    for (name, email, changed, seen) in [
        (Some(b"personal".as_slice()), b"first@example.com".as_slice(), true, 1),
        (Some(b"personal".as_slice()), b"first@example.com".as_slice(), false, 1),
        (None, b"".as_slice(), false, 1),
        (Some(b"personal".as_slice()), b"other-host@example.com".as_slice(), true, 4),
        (Some(b"unknown".as_slice()), b"ignored@example.com".as_slice(), false, 4),
        (Some(b"retired".as_slice()), b"ignored@example.com".as_slice(), false, 4),
    ] {
        world.wall = Wall::from_nanos(world.wall.as_nanos().saturating_add(1));
        let requests = name
            .map_or_else(Vec::new, |name| world.step(Event::ProgramSaid { name: name.into(), email: email.into() }));
        if changed {
            held = vec![AccountFact {
                kind: KIND_EMAIL,
                label: Box::new([]),
                value: FactValue::Text(email.into()),
                seen: Wall::from_nanos(seen),
                source: Some(SOURCE),
                origin: ORIGIN_PROGRAM,
            }];
        }
        assert_eq!(world.domain.account_facts(ACCOUNT), held);
        let expected = if changed {
            vec![Request::Save { record: Stored::Facts { account: ACCOUNT, facts: held.clone().into() } }]
        } else {
            vec![]
        };
        assert_eq!(requests, expected);
    }
    let removed = world.step(Event::Change {
        request: Token::new(92),
        holder: HOLDER,
        change: jig_domain_accounts::Change::Remove { account: ACCOUNT },
    });
    assert!(removed.contains(&Request::Erase { key: Key::Facts { account: ACCOUNT } }));
    assert_eq!((world.domain.account_facts(ACCOUNT)).len(), 0);
    assert_eq!(
        world
            .step(Event::ProgramSaid {
                name: b"personal".as_slice().into(),
                email: b"late@example.com".as_slice().into()
            })
            .len(),
        0
    );
    assert_eq!((world.domain.account_facts(ACCOUNT)).len(), 0);
}

fn declaration_change(world: &mut World, key: u64, name: &[u8]) -> Vec<Request> {
    world.step(Event::Change {
        request: skein_lib::Token::new(key),
        holder: HOLDER,
        change: jig_domain_accounts::Change::DeclareProgram { provider: PROVIDER, join: None, name: name.into() },
    })
}

#[test]
fn a_retired_name_declaration_refuses_another_holder() {
    let mut world = fresh();
    declaration_change(&mut world, 100, b"claude-6");
    world.step(Event::Change {
        request: skein_lib::Token::new(101),
        holder: HOLDER,
        change: jig_domain_accounts::Change::Remove { account: 0 },
    });
    targets(&mut world, &[0]);
    ended(&mut world, 0, jig_domain_accounts::ProgramResult::SignedOut);
    assert!(world.views().is_empty(), "forgotten account is absent from the list");
    let refused = world.step(Event::Change {
        request: skein_lib::Token::new(102),
        holder: OTHER,
        change: jig_domain_accounts::Change::DeclareProgram {
            provider: PROVIDER,
            join: None,
            name: b"claude-6".as_slice().into(),
        },
    });
    let [Request::Answer { request, answer: jig_domain_accounts::Answer::Refused { refusal } }] = refused.as_slice()
    else {
        panic!("a reserved name must answer without writes: {refused:?}")
    };
    assert_eq!(*request, skein_lib::Token::new(102));
    assert_eq!(*refusal, Refusal::NameTaken);
    assert_eq!(world.domain.accounts(), 0);
    assert_eq!(world.views().len(), 0);
    assert_eq!(world.domain.next_number(), 2);
    let fresh = declaration_change(&mut world, 103, b"claude-7");
    assert_eq!(
        fresh.last(),
        Some(&Request::Answer {
            request: skein_lib::Token::new(103),
            answer: jig_domain_accounts::Answer::ProgramDeclared { source: 3, name: b"claude-7".as_slice().into() },
        })
    );
}
