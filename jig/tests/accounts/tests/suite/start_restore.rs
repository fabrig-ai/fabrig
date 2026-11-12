//! Load/fit/link (jig/docs/design/domain/engine/restart.md; domain/accounts.md, 3.6).
use jig_accounts_world::{
    HOLDER, LIMITS, PROVIDER, World, account, commit, held, issued, program, provider, providers, source,
};
use jig_domain_accounts::{
    Change, Domain, Event, Held, Holder, Key, Kind, Limits, Providers, Request, RestoreLimit, RestoreSet, State,
    Stored, Use, programs,
};
use jig_domain_link::{Form, Scheme};
use skein_lib::{Token, Wall};

const ACCOUNT: u32 = 20;
const PROGRAM_SOURCE: u32 = 21;
const PROGRAM_PLACE: u32 = 8;
const REMOVED_PROVIDER: u32 = 99;
const OTHER_REMOVED_PROVIDER: u32 = 98;
fn configured() -> Providers {
    let mut offered = provider(PROVIDER, true, true);
    offered.offers.programs = Box::new([b"program".as_slice().into()]);
    providers(vec![offered])
}
fn start(rows: &[Stored]) -> World {
    World::with_start(LIMITS, configured(), rows)
}
const fn restore_limit(domain: &Domain) -> Option<RestoreLimit> {
    domain.restore_limit()
}
fn failed(world: &mut World, expected: RestoreLimit) {
    let later = vec![issued(100, &[b'a'; 33]), held(101, b""), held(102, b""), held(103, b""), held(104, b"")];
    if restore_limit(&world.domain).is_some() {
        world.restore(later.clone());
        assert_eq!(restore_limit(&world.domain), Some(expected));
    }
    assert_eq!(world.step(Event::Want).len(), 0);
    assert_eq!(restore_limit(&world.domain), Some(expected));
    assert!(!world.domain.restore_finished());

    world.restore(later);
    assert_eq!(restore_limit(&world.domain), Some(expected));
    world.restore(vec![Stored::Next { number: 900 }, held(800, b"later")]);
    assert_eq!(restore_limit(&world.domain), Some(expected));
    assert_eq!(world.step(Event::Want).len(), 0);
    assert_eq!(restore_limit(&world.domain), Some(expected));
    assert!(!world.domain.restore_finished());

    assert_eq!(world.domain.next_number(), 900);
    assert_eq!(world.step(Event::Want).len(), 0);
    assert_eq!(restore_limit(&world.domain), Some(expected));
    assert!(!world.domain.restore_finished());
}

#[test]
fn records_are_linked_after_loading_in_any_order() {
    let fact = jig_domain_accounts::AccountFact {
        kind: 999,
        label: Box::new([]),
        value: jig_domain_accounts::FactValue::Count(42),
        seen: Wall::EPOCH,
        source: None,
        origin: 999,
    };
    let orphan = ACCOUNT + 100;
    let mut world = start(&[
        Stored::Facts { account: ACCOUNT, facts: Box::new([fact.clone()]) },
        Stored::Facts { account: orphan, facts: Box::new([fact.clone()]) },
        program(ACCOUNT, PROGRAM_SOURCE, PROGRAM_PLACE, b"Upper Name"),
        issued(PROGRAM_SOURCE, b"Upper Name"),
        held(ACCOUNT, b"same"),
        held(22, b"same"),
        Stored::Next { number: 50 },
    ]);
    assert_eq!(restore_limit(&world.domain), None);
    assert!(world.domain.account(ACCOUNT).expect("loaded account").sources().is_empty());
    let out = world.step(Event::Want);
    assert!(world.domain.restore_finished());
    assert_eq!(world.domain.account_facts(ACCOUNT), &[fact]);
    assert_eq!(world.domain.account_facts(orphan), []);
    assert_eq!(world.domain.account_facts(22), []);

    assert_eq!(restore_limit(&world.domain), None);
    assert_eq!(world.domain.account(ACCOUNT).expect("linked").sources().get(&PROGRAM_PLACE), Some(&PROGRAM_SOURCE));
    assert!(world.domain.account(22).is_some(), "source-less identity survives");
    assert_eq!(programs::issued(&world.domain, b"Upper Name"), Some((PROGRAM_SOURCE, HOLDER)));
    assert!(!out.iter().any(|rq| matches!(rq, Request::Save { .. } | Request::Erase { .. })));
    assert!(
        out.iter().any(|rq| matches!(rq, Request::Programs { names } if names[0].name.as_bytes() == b"Upper Name"))
    );
    assert_eq!(world.domain.next_number(), 50);
}

#[test]
fn admission_rules_remain_at_live_entrances() {
    let mut row = held(ACCOUNT, b"same");
    if let Stored::Account { uses, display, .. } = &mut row {
        *uses = Some(Box::new([Use::Party(7), Use::Party(7)]));
        *display = b"\0stored".as_slice().into();
    }
    let mut named_key = source(ACCOUNT, 21, 0, Kind::Key);
    if let Stored::Source { name, .. } = &mut named_key {
        *name = b"stored key name".as_slice().into();
    }
    let rows = [row, named_key, source(ACCOUNT, 23, 1, Kind::SignIn), source(ACCOUNT, 24, 2, Kind::SignIn)];
    let mut world = start(&rows);
    world.step(Event::Want);

    assert_eq!(world.domain.account(ACCOUNT).expect("loaded").uses(), Some([Use::Party(7), Use::Party(7)].as_slice()));
    let out = world.step(Event::Change {
        request: Token::new(1),
        holder: HOLDER,
        change: Change::DeclareProgram { provider: PROVIDER, join: None, name: b"Upper Name".as_slice().into() },
    });
    assert!(matches!(out.as_slice(), [Request::Answer { answer: jig_domain_accounts::Answer::Refused { .. }, .. }]));
    assert_eq!(world.domain.sign_in(&LIMITS, HOLDER, 1, b""), Err(jig_domain_accounts::Refusal::Subject));
}

#[test]
fn removed_providers_and_source_less_accounts_keep_identity() {
    let rows = [
        account(ACCOUNT, REMOVED_PROVIDER, HOLDER, b"old"),
        source(ACCOUNT, 21, 7, Kind::Key),
        account(22, OTHER_REMOVED_PROVIDER, HOLDER, b""),
    ];
    let mut world = World::with_start(LIMITS, Providers::none(), &rows);
    world.step(Event::Want);
    assert!(world.domain.restore_finished());
    assert_eq!(world.domain.accounts(), 2);
    assert_eq!(world.domain.account(ACCOUNT).expect("kept").provider(), 99);
    assert!(!world.domain.account(ACCOUNT).expect("kept").plan());
    assert_eq!(world.domain.account_state(22), None);
    assert!(!world.domain.usable(ACCOUNT));
}

#[test]
fn stored_names_reconstruct_without_declaration_spelling_rules() {
    for name in [b"".as_slice(), b"Upper Name", &[255; 32]] {
        let mut world = start(&[issued(21, name), held(ACCOUNT, b""), program(ACCOUNT, 21, 0, name)]);
        assert_eq!(restore_limit(&world.domain), None);
        world.step(Event::Want);
        assert_eq!(restore_limit(&world.domain), None);
        assert_eq!(programs::name(&world.domain, 21, HOLDER), Some(name));
        assert!(world.domain.source_grant(21, world.now).is_some());
    }
}

#[test]
fn account_and_source_places_are_preserved() {
    let mut row = held(ACCOUNT, b"");
    if let Stored::Account { place, .. } = &mut row {
        *place = 91;
    }
    let mut world = start(&[row, source(ACCOUNT, 21, 42, Kind::Key), source(ACCOUNT, 22, 5, Kind::Key)]);
    let out = world.step(Event::Want);
    assert_eq!(world.domain.account(ACCOUNT).expect("kept").place(), 91);
    assert_eq!(
        world.domain.account(ACCOUNT).expect("kept").sources().iter().collect::<Vec<_>>(),
        vec![(&5, &22), (&42, &21)]
    );
    assert!(
        !out.iter().any(|rq| matches!(rq, Request::Save { record: Stored::Account { .. } | Stored::Source { .. } }))
    );
    world.step(Event::Change {
        request: Token::new(2),
        holder: HOLDER,
        change: Change::MoveSource { account: ACCOUNT, source: 21, place: 0 },
    });
    assert_eq!(world.domain.account(ACCOUNT).expect("kept").sources().get(&0), Some(&21));
}

#[test]
fn every_set_fits_at_its_exact_capacity() {
    let limits = Limits { accounts: 1, holder_accounts: 1, sources: 1, program_names: 1, ..LIMITS };
    let mut world =
        World::with_start(limits, configured(), &[held(ACCOUNT, b""), program(ACCOUNT, 21, 0, b"p"), issued(21, b"p")]);
    assert_eq!(restore_limit(&world.domain), None);
    world.step(Event::Want);
    assert!(world.domain.restore_finished());

    assert_eq!(restore_limit(&world.domain), None);
}

#[test]
fn excess_stored_sets_refuse_stickily() {
    let mut retired = issued(21, b"p");
    if let Stored::Issued { signed_out, .. } = &mut retired {
        *signed_out = true;
    }
    for (limits, rows, expected) in [
        (
            Limits { accounts: 1, holder_accounts: 1, ..LIMITS },
            [held(ACCOUNT, b""), held(22, b"")],
            RestoreLimit { key: Key::Account { number: 22 }, set: RestoreSet::Accounts, limit: 1 },
        ),
        (
            Limits { accounts: 1, holder_accounts: 1, sources: 1, ..LIMITS },
            [source(ACCOUNT, 21, 0, Kind::Key), source(ACCOUNT, 22, 1, Kind::Key)],
            RestoreLimit { key: Key::Source { account: ACCOUNT, number: 22 }, set: RestoreSet::TotalSources, limit: 1 },
        ),
        (
            Limits { program_names: 1, ..LIMITS },
            [retired, issued(22, b"q")],
            RestoreLimit { key: Key::Issued { number: 22 }, set: RestoreSet::ProgramNames, limit: 1 },
        ),
    ] {
        let mut world = World::with_start(limits, configured(), &rows);
        assert_eq!(restore_limit(&world.domain), Some(expected));
        failed(&mut world, expected);
    }
}
#[test]
fn excess_sources_in_one_account_refuse_stickily() {
    let mut world = World::with_start(
        Limits { sources: 1, ..LIMITS },
        configured(),
        &[held(ACCOUNT, b""), source(ACCOUNT, 21, 0, Kind::Key), source(ACCOUNT, 22, 1, Kind::Key)],
    );
    assert_eq!(restore_limit(&world.domain), None, "source rows fit the aggregate table before linking");
    failed(
        &mut world,
        RestoreLimit {
            key: Key::Source { account: ACCOUNT, number: 22 },
            set: RestoreSet::Sources { account: ACCOUNT },
            limit: 1,
        },
    );
}
#[test]
fn names_past_the_fixed_carrier_refuse_stickily() {
    for row in [issued(21, &[b'a'; 33]), program(ACCOUNT, 21, 0, &[b'a'; 33])] {
        let expected = RestoreLimit { key: row.key(), set: RestoreSet::NameBytes, limit: jig_domain_accounts::NAME };
        let mut world = start(&[row]);
        assert_eq!(restore_limit(&world.domain), Some(expected));
        failed(&mut world, expected);
    }
}

#[test]
fn retained_capacities_bound_accounts_and_lifetime_names_after_limits_grow() {
    for (rows, expected) in [
        (
            [
                Stored::Facts { account: ACCOUNT, facts: Box::new([]) },
                Stored::Facts { account: 22, facts: Box::new([]) },
            ],
            RestoreLimit { key: Key::Facts { account: 22 }, set: RestoreSet::Accounts, limit: 1 },
        ),
        (
            [held(ACCOUNT, b""), held(22, b"")],
            RestoreLimit { key: Key::Account { number: 22 }, set: RestoreSet::Accounts, limit: 1 },
        ),
        (
            [issued(21, b"p"), issued(22, b"q")],
            RestoreLimit { key: Key::Issued { number: 22 }, set: RestoreSet::ProgramNames, limit: 1 },
        ),
    ] {
        let mut world = World::with_start(
            Limits { accounts: 1, holder_accounts: 1, program_names: 1, ..LIMITS },
            configured(),
            &[],
        );
        world.limits.accounts = LIMITS.accounts;
        world.limits.program_names = LIMITS.program_names;
        world.restore(rows.into());
        assert_eq!(restore_limit(&world.domain), Some(expected));
        failed(&mut world, expected);
    }
}

fn refused_link(world: &mut World, key: Key, target: Key) {
    let expected = jig_domain_accounts::RestoreLink { key, target };
    assert_eq!(world.step(Event::Want).len(), 0);
    assert_eq!(world.domain.restore_link(), Some(expected));
    assert_eq!(world.domain.restore_limit(), None);
    assert!(!world.domain.restore_finished());
    assert_eq!(world.step(Event::Want).len(), 0);
    assert_eq!(world.domain.restore_link(), Some(expected));
}

#[test]
fn missing_source_owners_refuse_at_link() {
    let mut world = start(&[source(ACCOUNT, 21, 0, Kind::Key)]);
    refused_link(&mut world, Key::Source { account: ACCOUNT, number: 21 }, Key::Account { number: ACCOUNT });
}
#[test]
fn missing_reservations_are_not_repaired() {
    let mut world = start(&[held(ACCOUNT, b""), program(ACCOUNT, 21, 0, b"p")]);
    refused_link(&mut world, Key::Source { account: ACCOUNT, number: 21 }, Key::Issued { number: 21 });
}
#[test]
fn duplicate_reservations_refuse_with_both_keys() {
    let mut world = start(&[issued(21, b"p"), issued(22, b"p")]);
    refused_link(&mut world, Key::Issued { number: 21 }, Key::Issued { number: 22 });
}
#[test]
fn reservation_names_must_link_to_the_source() {
    let mut world = start(&[held(ACCOUNT, b""), program(ACCOUNT, 21, 0, b"p"), issued(21, b"q")]);
    refused_link(&mut world, Key::Source { account: ACCOUNT, number: 21 }, Key::Issued { number: 21 });
}
#[test]
fn reservation_owners_must_link_to_the_source() {
    let mut world =
        start(&[account(ACCOUNT, PROVIDER, Holder::Party(8), b""), program(ACCOUNT, 21, 0, b"p"), issued(21, b"p")]);
    refused_link(&mut world, Key::Source { account: ACCOUNT, number: 21 }, Key::Issued { number: 21 });
}
#[test]
fn conflicting_source_places_refuse_with_keys() {
    let mut world = start(&[held(ACCOUNT, b""), source(ACCOUNT, 21, 0, Kind::Key), source(ACCOUNT, 22, 0, Kind::Key)]);
    refused_link(
        &mut world,
        Key::Source { account: ACCOUNT, number: 22 },
        Key::Source { account: ACCOUNT, number: 21 },
    );
}

#[test]
fn an_empty_start_finishes_once() {
    let cold = Domain::new(&LIMITS, configured());
    assert_eq!(restore_limit(&cold), None);
    let mut world = start(&[]);
    assert_eq!(restore_limit(&world.domain), None);
    assert!(!world.domain.restore_finished());
    world.step(Event::Want);
    assert!(world.domain.restore_finished());
    assert_eq!(world.step(Event::Want).len(), 0);
    assert_eq!(restore_limit(&world.domain), None);
}

#[test]
fn held_wall_resets_and_runtime_movements_survive_start() {
    let mut world = start(&[]);
    world.wall = Wall::from_nanos(1_000_000_000_000);
    world.restore(vec![
        held(ACCOUNT, b"subject"),
        Stored::Source {
            account: ACCOUNT,
            number: 21,
            place: 4,
            kind: Kind::SignIn,
            name: Box::new([]),
            held: Some(Held::Limited { reset: Some(Wall::from_nanos(1_060_000_000_000)) }),
        },
        source(ACCOUNT, 22, 9, Kind::Key),
        Stored::Next { number: 90 },
    ]);
    world.step(Event::Want);
    assert_eq!(world.domain.next_deadline(), Some(skein_lib::Time::from_nanos(60_000_000_000)));
    assert!(matches!(world.domain.source(21).expect("kept").state, State::Limited { until: Some(_) }));
    assert_eq!(world.domain.next_number(), 90);
    world.fire_at(60);
    assert_eq!(world.domain.source(21).expect("kept").state, State::Asking);
    assert_eq!(world.domain.next_deadline(), None);
    let out = world.step(Event::Change {
        request: Token::new(1),
        holder: HOLDER,
        change: Change::MoveOut { account: ACCOUNT },
    });
    assert!(
        out.iter().any(|rq| matches!(rq, Request::Signing(jig_domain_accounts::Signing::Withdraw { source: 21, .. })))
    );
    assert_eq!(world.domain.next_number(), 91);
    assert_eq!(world.domain.source(21).expect("kept").state, State::Moving);
}

#[test]
fn removed_provider_programs_keep_plan_observations() {
    let mut world = World::with_start(
        LIMITS,
        Providers::none(),
        &[account(ACCOUNT, REMOVED_PROVIDER, HOLDER, b""), program(ACCOUNT, 21, 0, b"p"), issued(21, b"p")],
    );
    world.step(Event::Want);
    assert!(world.domain.account(ACCOUNT).expect("kept").plan());
    assert!(world.domain.source_grant(21, world.now).is_some());
}

#[test]
fn passed_resets_are_lifted_and_moving_holds_refresh() {
    let mut world = start(&[]);
    world.wall = Wall::from_nanos(100);
    let mut rows = vec![held(ACCOUNT, b"")];
    for (number, held) in
        [(21, Held::Limited { reset: Some(Wall::from_nanos(99)) }), (22, Held::Moving), (23, Held::Refused)]
    {
        rows.push(Stored::Source {
            account: ACCOUNT,
            number,
            place: number,
            kind: Kind::SignIn,
            name: Box::new([]),
            held: Some(held),
        });
    }
    world.restore(rows);
    let out = world.step(Event::Want);
    assert_eq!(world.domain.source(21).expect("kept").state, State::Asking);
    assert_eq!(world.domain.source(23).expect("kept").state, State::Refused);
    assert!(out.contains(&Request::Signing(jig_domain_accounts::Signing::Refresh { source: 22 })));
    assert_eq!(world.domain.next_deadline(), None);
}

#[test]
fn live_sources_append_after_sparse_restored_places() {
    for place in [1, 42, u32::MAX] {
        let mut world = start(&[held(ACCOUNT, b""), source(ACCOUNT, 21, place, Kind::Key)]);
        world.step(Event::Want);
        let out = world.step(Event::Change {
            request: Token::new(1),
            holder: HOLDER,
            change: Change::DeclareProgram { provider: PROVIDER, join: Some(ACCOUNT), name: b"p".as_slice().into() },
        });
        assert!(out.iter().any(|rq| matches!(
            rq,
            Request::Answer { answer: jig_domain_accounts::Answer::ProgramDeclared { .. }, .. }
        )));
        let sources = world.domain.account(ACCOUNT).expect("kept").sources();
        assert_eq!(sources.len(), 2);
        assert_eq!(sources.iter().map(|(_, n)| *n).collect::<Vec<_>>(), vec![21, 22]);
    }
}

#[test]
fn live_keys_append_after_sparse_restored_places() {
    for place in [1, 42, u32::MAX] {
        let mut world = start(&[held(ACCOUNT, b""), source(ACCOUNT, 21, place, Kind::Key)]);
        world.step(Event::Want);
        world.step(Event::Change {
            request: Token::new(1),
            holder: HOLDER,
            change: Change::AddKey { provider: PROVIDER, join: Some(ACCOUNT), carried: 1 << 63 },
        });
        world.step(Event::Component(jig_domain_accounts::FromComponent::Granted {
            account: 22,
            generation: 1,
            form: Form::Credential(Scheme::Key),
            valid: skein_lib::Duration::from_secs(60),
        }));
        let sources = world.domain.account(ACCOUNT).expect("kept").sources();
        assert_eq!(sources.len(), 2);
        assert_eq!(sources.iter().map(|(_, n)| *n).collect::<Vec<_>>(), vec![21, 22]);
    }
}

/// Read the durable account fields through the reader's public doors.
fn account_readback(world: &World) -> Vec<Stored> {
    world
        .domain
        .accounts_by_number()
        .iter()
        .map(|(number, row)| Stored::Account {
            list: world.domain.list(*number).copied(),
            number: *number,
            provider: row.provider(),
            holder: row.holder(),
            place: row.place(),
            uses: row.uses().map(Into::into),
            subject: row.subject().into(),
            display: row.display().into(),
        })
        .collect()
}
fn source_readback(world: &World) -> Vec<(u32, u32, Kind, Option<Held>)> {
    world
        .domain
        .source_views()
        .iter()
        .map(|fact| {
            let held = match fact.state {
                State::Limited { until } => Some(Held::Limited { reset: until.map(|d| d.wall) }),
                State::Refused => Some(Held::Refused),
                State::Moving => Some(Held::Moving),
                State::Asking
                | State::Granted
                | State::Later { .. }
                | State::NeedsSignIn
                | State::Misconfigured
                | State::Unloaded { .. }
                | State::Cancelled
                | State::Closing => None,
            };
            (fact.account, fact.source, fact.kind, held)
        })
        .collect()
}

/// Saves and erases are the writer's; every cold reader loads the resulting keyed snapshot.
pub(crate) fn writer_readback(seed: u64) -> Vec<Stored> {
    use jig_domain_accounts::{FromComponent, Signing};
    let mut rng = skein_lib::Rng::new(seed);
    let mut world = World::with_start(jig_accounts_world::LIMITS, configured(), &[]);
    let mut store = std::collections::BTreeMap::new();
    commit(&mut store, &world.step(Event::Want));
    for request in 0..80_u64 {
        let numbers: Vec<_> = world.domain.accounts_by_number().iter().map(|(n, _)| *n).collect();
        let account = numbers.get(usize::try_from(rng.below(3)).expect("small index")).copied().unwrap_or(u32::MAX);
        let sources = world.domain.source_views();
        let source = sources.get(u32::try_from(rng.below(9)).expect("small index")).map_or(u32::MAX, |f| f.source);
        let change = match rng.below(9) {
            0 => Change::DeclareProgram {
                provider: PROVIDER,
                join: None,
                name: format!("program-{request}").into_bytes().into(),
            },
            1 => Change::DeclareProgram {
                provider: PROVIDER,
                join: Some(account),
                name: format!("program-{request}").into_bytes().into(),
            },
            2 => Change::RemoveKey { source },
            3 => Change::MoveAccount { account, place: u32::try_from(rng.below(3)).expect("small place") },
            4 => Change::MoveSource { account, source, place: u32::try_from(rng.below(3)).expect("small place") },
            5 => Change::SetUses { account, uses: Some(Box::new([Use::Party(request)])) },
            6 => Change::Resume { source },
            7 => Change::AddKey { provider: PROVIDER, join: Some(account), carried: 1 << 63 },
            _ => Change::AddKey { provider: PROVIDER, join: None, carried: 1 << 63 },
        };
        let out = world.step(Event::Change { request: Token::new(request), holder: HOLDER, change });
        commit(&mut store, &out);
        for rq in out {
            if let Request::Signing(Signing::Keep { source, .. }) = rq {
                let out = world.step(Event::Component(FromComponent::Granted {
                    account: source,
                    generation: 1,
                    form: Form::Credential(Scheme::Key),
                    valid: skein_lib::Duration::from_secs(100),
                }));
                commit(&mut store, &out);
            }
        }
        if rng.chance(25) {
            let out = world.step(Event::Limited { account: source, reset: None });
            commit(&mut store, &out);
        }
        let rows: Vec<_> = store.values().cloned().collect();
        let mut again = start(&rows);
        let out = again.step(Event::Want);
        assert!(again.domain.restore_finished(), "seed {seed}, step {request}");

        assert_eq!(restore_limit(&again.domain), None);
        assert!(
            !out.iter().any(|rq| matches!(rq, Request::Save { .. } | Request::Erase { .. })),
            "seed {seed}, step {request}: {out:?}"
        );
        assert_eq!(account_readback(&again), account_readback(&world), "seed {seed}, step {request}");
        assert_eq!(source_readback(&again), source_readback(&world), "seed {seed}, step {request}");
        assert_eq!(again.domain.next_number(), world.domain.next_number());
        for (n, row) in again.domain.accounts_by_number() {
            let original = world.domain.account(*n).expect("writer account");
            assert_eq!(row.sources().iter().collect::<Vec<_>>(), original.sources().iter().collect::<Vec<_>>());
            assert_eq!(row.plan(), original.plan());
        }
        for row in &rows {
            if let Stored::Issued { number, holder, name, .. } = row {
                assert_eq!(programs::issued(&again.domain, name), Some((*number, *holder)));
            }
        }
    }
    store.into_values().collect()
}

#[test]
fn writer_produced_snapshots_restore_and_read_back() {
    use jig_domain_accounts::{ListRecord, ReadFailure};
    for list in [
        None,
        Some(ListRecord {
            names: u64::MAX,
            read: Some(Wall::EPOCH),
            tried: Some(Wall::from_nanos(u64::MAX)),
            failure: Some(ReadFailure::Refused { status: 403 }),
            left_out: u32::MAX,
        }),
    ] {
        let mut row = held(ACCOUNT, b"subject");
        let Stored::Account { list: stored_list, .. } = &mut row else { unreachable!("account fixture") };
        *stored_list = list;
        let mut world = start(&[row.clone()]);
        world.step(Event::Want);
        assert_eq!(world.domain.list(ACCOUNT), list.as_ref());
        assert_eq!(world.domain.list(u32::MAX), None);
        let out = world.step(Event::Change {
            request: Token::new(1),
            holder: HOLDER,
            change: Change::SetUses { account: ACCOUNT, uses: Some(Box::new([Use::Party(7)])) },
        });
        let mut store = std::collections::BTreeMap::from([(row.key(), row)]);
        commit(&mut store, &out);
        let rows: Vec<_> = store.into_values().collect();
        let Stored::Account { list: saved, uses, .. } = rows.first().expect("saved account") else {
            panic!("account row")
        };
        assert_eq!(*saved, list);
        assert_eq!(uses.as_deref(), Some([Use::Party(7)].as_slice()), "the writer saved the change");
        let mut again = start(&rows);
        again.step(Event::Want);
        assert_eq!(again.domain.list(ACCOUNT), list.as_ref());
    }
    assert_eq!(writer_readback(61), writer_readback(61));
}

#[test]
fn stored_number_rules_are_not_replayed_as_links() {
    let rows = [
        held(7, b"subject"),
        source(7, 8, 1, Kind::Key),
        source(7, 7, 0, Kind::SignIn),
        account(9, 5, Holder::Party(8), b"other"),
        source(9, 10, 0, Kind::SignIn),
    ];
    let mut world = start(&rows);
    world.step(Event::Want);
    assert!(world.domain.restore_finished());

    assert_eq!(world.domain.account(7).expect("account identity").sources().len(), 2);
    assert_eq!(world.domain.source(7).expect("source identity").account, 7);
    assert!(world.domain.account(9).is_some());
    assert_eq!(world.domain.next_number(), 11);
}

#[test]
fn removing_a_key_rebuilds_plans_without_a_configured_provider() {
    for remaining in [Kind::Key, Kind::Program] {
        let mut rows = vec![account(ACCOUNT, REMOVED_PROVIDER, HOLDER, b"subject"), source(ACCOUNT, 21, 0, Kind::Key)];
        if remaining == Kind::Program {
            rows.extend([program(ACCOUNT, 22, 42, b"p"), issued(22, b"p")]);
        } else {
            rows.push(source(ACCOUNT, 22, 42, remaining));
        }
        let mut world = World::with_start(LIMITS, Providers::none(), &rows);
        world.step(Event::Want);
        let out = world.step(Event::Change {
            request: Token::new(1),
            holder: HOLDER,
            change: Change::RemoveKey { source: 21 },
        });
        assert!(out.contains(&Request::Answer { request: Token::new(1), answer: jig_domain_accounts::Answer::Done }));
        assert!(world.domain.source(21).is_none());
        let kept = world.domain.account(ACCOUNT).expect("remaining durable identity");
        assert_eq!(kept.provider(), 99);
        assert_eq!(kept.plan(), remaining == Kind::Program);
        assert_eq!(kept.sources().get(&0), Some(&22));
        assert!(out.contains(&Request::Save {
            record: if remaining == Kind::Program {
                program(ACCOUNT, 22, 0, b"p")
            } else {
                source(ACCOUNT, 22, 0, remaining)
            }
        }));
    }
}

#[test]
fn sign_in_renews_after_the_last_representable_stored_place() {
    let rows = [held(ACCOUNT, b"subject"), source(ACCOUNT, 21, u32::MAX, Kind::Key)];
    let mut world = start(&rows);
    world.step(Event::Want);
    assert_eq!(world.domain.account(ACCOUNT).expect("kept").sources().get(&u32::MAX), Some(&21));
    world.step(Event::SignIn { request: Token::new(7), holder: HOLDER, by: 7, provider: PROVIDER });
    world.step(Event::Visited { source: 22, address: b"https://issuer.test/visit".as_slice().into() });
    let out = world.step(Event::SignedIn {
        source: 22,
        subject: b"subject".as_slice().into(),
        display: Box::new([]),
        declined: false,
        said: jig_domain_accounts::Said::default(),
    });
    let kept = world.domain.account(ACCOUNT).expect("renewed account");
    assert_eq!(kept.sources().len(), 2);
    assert_eq!(kept.sources().get(&0), Some(&21));
    assert_eq!(kept.sources().get(&1), Some(&22));
    assert!(
        world
            .ended()
            .iter()
            .any(|ended| matches!(ended.end, jig_domain_accounts::End::Renewed { account: ACCOUNT, declined: false }))
    );
    assert!(out.contains(&Request::Save { record: source(ACCOUNT, 22, 1, Kind::SignIn) }));
    assert!(out.contains(&Request::Save { record: source(ACCOUNT, 21, 0, Kind::Key) }));
}

#[test]
fn a_moved_sign_in_renumbers_only_once_adoption_is_kept() {
    for kept in [false, true] {
        let mut world = start(&[held(ACCOUNT, b"subject"), source(ACCOUNT, 21, u32::MAX, Kind::Key)]);
        world.step(Event::Want);
        world.step(Event::Change {
            request: Token::new(7),
            holder: HOLDER,
            change: Change::MoveIn { carried: 1 << 63 },
        });
        assert_eq!(
            world.step(Event::Opened {
                source: 22,
                provider: PROVIDER,
                subject: b"subject".as_slice().into(),
                display: Box::new([]),
            }),
            vec![Request::Signing(jig_domain_accounts::Signing::Adopt { source: 22, carried: 1 << 63 })]
        );
        assert_eq!(world.domain.account(ACCOUNT).expect("kept").sources().get(&u32::MAX), Some(&21));
        let out = world.step(Event::Adopted { source: 22, kept });
        let sources = world.domain.account(ACCOUNT).expect("kept").sources();
        if kept {
            assert_eq!(sources.get(&0), Some(&21));
            assert_eq!(sources.get(&1), Some(&22));
            assert!(out.contains(&Request::Save { record: source(ACCOUNT, 21, 0, Kind::Key) }));
            assert!(out.contains(&Request::Answer {
                request: Token::new(7),
                answer: jig_domain_accounts::Answer::MovedIn { account: ACCOUNT, renewed: true },
            }));
            assert_eq!(world.domain.source(22).expect("adopted source").state, State::Moving);
        } else {
            assert_eq!(sources.len(), 1);
            assert_eq!(sources.get(&u32::MAX), Some(&21));
            assert!(!out.iter().any(|rq| matches!(rq, Request::Save { .. })));
        }
    }
}

#[test]
fn program_and_sign_in_append_after_restoration() {
    let rows = [held(ACCOUNT, b"subject"), source(ACCOUNT, 21, 5, Kind::Key), source(ACCOUNT, 22, 42, Kind::Key)];
    let limits = Limits { sources: 4, ..LIMITS };
    let mut world = World::with_start(limits, configured(), &rows);
    let out = world.step(Event::Want);
    let mut store = rows.into_iter().map(|row| (row.key(), row)).collect();
    commit(&mut store, &out);
    let (first, last, declared, signed_in) = (5, 42, 43, 44);
    let out = world.step(Event::Change {
        request: Token::new(1),
        holder: HOLDER,
        change: Change::DeclareProgram { provider: PROVIDER, join: Some(ACCOUNT), name: b"p".as_slice().into() },
    });
    assert!(out.contains(&Request::Save { record: program(ACCOUNT, 23, declared, b"p") }));
    commit(&mut store, &out);
    let out = world.step(Event::SignIn { request: Token::new(2), holder: HOLDER, by: 7, provider: PROVIDER });
    commit(&mut store, &out);
    world.step(Event::Visited { source: 24, address: b"https://issuer.test/visit".as_slice().into() });
    let out = world.step(Event::SignedIn {
        source: 24,
        subject: b"subject".as_slice().into(),
        display: Box::new([]),
        declined: false,
        said: jig_domain_accounts::Said::default(),
    });
    assert!(out.contains(&Request::Save { record: source(ACCOUNT, 24, signed_in, Kind::SignIn) }));
    commit(&mut store, &out);
    for (number, place, kind) in [(21, first, Kind::Key), (22, last, Kind::Key), (24, signed_in, Kind::SignIn)] {
        assert_eq!(store.get(&Key::Source { account: ACCOUNT, number }), Some(&source(ACCOUNT, number, place, kind)));
        assert_eq!(world.domain.account(ACCOUNT).expect("kept account").sources().get(&place), Some(&number));
    }
    assert_eq!(store.get(&Key::Source { account: ACCOUNT, number: 23 }), Some(&program(ACCOUNT, 23, declared, b"p")));
    assert_eq!(world.domain.account(ACCOUNT).expect("kept account").sources().get(&declared), Some(&23));
}

#[test]
fn registered_facts_are_shown_to_their_audience() {
    use jig_domain_accounts::{AccountFact, Audience, FactValue, FactWindow, audience};
    let holder = Some(Audience::Holder);
    let account = Some(Audience::Account);
    for (kind, expected) in [
        (0, None),
        (1, holder),
        (2, holder),
        (3, account),
        (4, account),
        (5, holder),
        (6, holder),
        (7, account),
        (8, None),
    ] {
        assert_eq!(audience(kind), expected);
        for origin in 0..=7 {
            for state in 0..=4 {
                let registered = expected.is_some() && (1..=6).contains(&origin) && (1..=3).contains(&state);
                let fact = AccountFact {
                    kind,
                    label: Box::new([]),
                    value: FactValue::Window(FactWindow { length: None, used: None, resets: None, state }),
                    seen: Wall::EPOCH,
                    source: None,
                    origin,
                };
                assert_eq!(fact.registered(), registered, "kind {kind}, origin {origin}, state {state}");
                assert_eq!(fact.shown_to(true), registered);
                assert_eq!(fact.shown_to(false), registered && expected == account);
            }
        }
    }
}
