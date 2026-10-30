//! Opted-in load/fit/link (jig/docs/design/domain/engine.md, 6; domain/accounts.md, 3.6).
use jig_accounts_world::{LIMITS, World, account, provider, providers, source};
use jig_domain_accounts::{
    Change, Domain, Event, Held, Holder, Key, Kind, Limits, Providers, Request, RestoreLimit, RestoreSet, State,
    Stored, Use, programs,
};
use skein_lib::{Env, Queue, Token, Wall};

const HOLDER: Holder = Holder::Party(7);
fn configured() -> Providers {
    let mut offered = provider(1, true, true);
    offered.offers.programs = Box::new([b"program".as_slice().into()]);
    providers(vec![offered])
}
fn start(rows: &[Stored]) -> World {
    World::with_start(LIMITS, configured(), rows)
}
fn issued(number: u32, name: &[u8]) -> Stored {
    Stored::Issued { number, holder: HOLDER, name: name.into(), signed_out: false }
}
fn program(account: u32, number: u32, place: u32, name: &[u8]) -> Stored {
    Stored::Source { account, number, place, kind: Kind::Program, name: name.into(), held: None }
}
const fn restore_limit(domain: &Domain) -> Option<RestoreLimit> {
    domain.restore_limit()
}
fn failed(world: &mut World, expected: RestoreLimit) {
    let later = vec![
        issued(100, &[b'a'; 33]),
        account(101, 1, HOLDER, b""),
        account(102, 1, HOLDER, b""),
        account(103, 1, HOLDER, b""),
        account(104, 1, HOLDER, b""),
    ];
    if restore_limit(&world.domain).is_some() {
        world.restore(later.clone());
        assert_eq!(restore_limit(&world.domain), Some(expected));
    }
    assert!(world.step(Event::Want).is_empty());
    assert_eq!(restore_limit(&world.domain), Some(expected));
    assert!(!world.domain.restore_finished());
    assert_eq!(world.domain.unrestored(), 1);
    world.restore(later);
    assert_eq!(restore_limit(&world.domain), Some(expected));
    world.restore(vec![Stored::Next { number: 900 }, account(800, 1, HOLDER, b"later")]);
    assert_eq!(restore_limit(&world.domain), Some(expected));
    assert!(world.step(Event::Want).is_empty());
    assert_eq!(restore_limit(&world.domain), Some(expected));
    assert!(!world.domain.restore_finished());
    assert_eq!(world.domain.unrestored(), 1);
    assert_eq!(world.domain.next_number(), 900);
    assert!(world.step(Event::Want).is_empty());
    assert_eq!(restore_limit(&world.domain), Some(expected));
    assert!(!world.domain.restore_finished());
    assert_eq!(world.domain.unrestored(), 1);
}

#[test]
fn records_are_linked_after_loading_in_any_order() {
    let mut world = start(&[
        program(20, 21, 8, b"Upper Name"),
        issued(21, b"Upper Name"),
        account(20, 1, HOLDER, b"same"),
        account(22, 1, HOLDER, b"same"),
        Stored::Next { number: 50 },
    ]);
    assert_eq!(restore_limit(&world.domain), None);
    assert!(world.domain.account(20).expect("loaded account").sources().is_empty());
    let out = world.step(Event::Want);
    assert!(world.domain.restore_finished());
    assert_eq!(world.domain.unrestored(), 0);
    assert_eq!(restore_limit(&world.domain), None);
    assert_eq!(world.domain.account(20).expect("linked").sources().get(&8), Some(&21));
    assert!(world.domain.account(22).is_some(), "source-less identity survives");
    assert_eq!(programs::issued(&world.domain, b"Upper Name"), Some((21, HOLDER)));
    assert!(!out.iter().any(|rq| matches!(rq, Request::Save { .. } | Request::Erase { .. })));
    assert!(
        out.iter().any(|rq| matches!(rq, Request::Programs { names } if names[0].name.as_bytes() == b"Upper Name"))
    );
    assert_eq!(world.domain.next_number(), 50);
}

#[test]
fn admission_rules_remain_at_live_entrances() {
    let mut row = account(20, 1, HOLDER, b"same");
    if let Stored::Account { uses, display, .. } = &mut row {
        *uses = Some(Box::new([Use::Party(7), Use::Party(7)]));
        *display = b"\0stored".as_slice().into();
    }
    let mut named_key = source(20, 21, 0, Kind::Key);
    if let Stored::Source { name, .. } = &mut named_key {
        *name = b"stored key name".as_slice().into();
    }
    let rows = [row, named_key, source(20, 23, 1, Kind::SignIn), source(20, 24, 2, Kind::SignIn)];
    let mut default_world = World::with(configured(), &rows);
    assert_eq!(restore_limit(&default_world.domain), None);
    default_world.step(Event::Want);
    assert_eq!(default_world.domain.accounts(), 0);
    assert_eq!(restore_limit(&default_world.domain), None);
    let mut world = start(&rows);
    world.step(Event::Want);
    assert_eq!(world.domain.unrestored(), 0);
    assert_eq!(world.domain.account(20).expect("loaded").uses(), Some([Use::Party(7), Use::Party(7)].as_slice()));
    let out = world.step(Event::Change {
        request: Token::new(1),
        holder: HOLDER,
        change: Change::DeclareProgram { provider: 1, join: None, name: b"Upper Name".as_slice().into() },
    });
    assert!(matches!(out.as_slice(), [Request::Answer { answer: jig_domain_accounts::Answer::Refused { .. }, .. }]));
    assert_eq!(world.domain.sign_in(&LIMITS, HOLDER, 1, b""), Err(jig_domain_accounts::Refusal::Subject));
}

#[test]
fn removed_providers_and_source_less_accounts_keep_identity() {
    let rows = [account(20, 99, HOLDER, b"old"), source(20, 21, 7, Kind::Key), account(22, 98, HOLDER, b"")];
    let mut world = World::with_start(LIMITS, Providers::none(), &rows);
    world.step(Event::Want);
    assert!(world.domain.restore_finished());
    assert_eq!(world.domain.accounts(), 2);
    assert_eq!(world.domain.account(20).expect("kept").provider(), 99);
    assert!(!world.domain.account(20).expect("kept").plan());
    assert_eq!(world.domain.account_state(22), None);
    assert!(!world.domain.usable(20));
    let mut default_world = World::with(Providers::none(), &rows);
    default_world.step(Event::Want);
    assert_eq!(default_world.domain.accounts(), 0);
    assert_eq!(default_world.domain.unrestored(), 3);
    assert_eq!(restore_limit(&default_world.domain), None);
    assert!(default_world.domain.restore_finished());
}

#[test]
fn stored_names_reconstruct_without_declaration_spelling_rules() {
    for name in [b"".as_slice(), b"Upper Name", &[255; 32]] {
        let mut world = start(&[issued(21, name), account(20, 1, HOLDER, b""), program(20, 21, 0, name)]);
        assert_eq!(restore_limit(&world.domain), None);
        world.step(Event::Want);
        assert_eq!(restore_limit(&world.domain), None);
        assert_eq!(programs::name(&world.domain, 21, HOLDER), Some(name));
        assert!(world.domain.source_grant(21, world.now).is_some());
    }
}

#[test]
fn account_and_source_places_are_preserved() {
    let mut row = account(20, 1, HOLDER, b"");
    if let Stored::Account { place, .. } = &mut row {
        *place = 91;
    }
    let mut world = start(&[row, source(20, 21, 42, Kind::Key), source(20, 22, 5, Kind::Key)]);
    let out = world.step(Event::Want);
    assert_eq!(world.domain.account(20).expect("kept").place(), 91);
    assert_eq!(
        world.domain.account(20).expect("kept").sources().iter().collect::<Vec<_>>(),
        vec![(&5, &22), (&42, &21)]
    );
    assert!(
        !out.iter().any(|rq| matches!(rq, Request::Save { record: Stored::Account { .. } | Stored::Source { .. } }))
    );
    world.step(Event::Change {
        request: Token::new(2),
        holder: HOLDER,
        change: Change::MoveSource { account: 20, source: 21, place: 0 },
    });
    assert_eq!(world.domain.account(20).expect("kept").sources().get(&0), Some(&21));
}

#[test]
fn every_set_fits_at_its_exact_capacity() {
    let limits = Limits { accounts: 1, holder_accounts: 1, sources: 1, program_names: 1, ..LIMITS };
    let mut world = World::with_start(
        limits,
        configured(),
        &[account(20, 1, HOLDER, b""), program(20, 21, 0, b"p"), issued(21, b"p")],
    );
    assert_eq!(restore_limit(&world.domain), None);
    world.step(Event::Want);
    assert!(world.domain.restore_finished());
    assert_eq!(world.domain.unrestored(), 0);
    assert_eq!(restore_limit(&world.domain), None);
}

#[test]
fn excess_accounts_refuse_stickily() {
    let mut world = World::with_start(
        Limits { accounts: 1, holder_accounts: 1, ..LIMITS },
        configured(),
        &[account(20, 1, HOLDER, b""), account(22, 1, HOLDER, b"")],
    );
    let expected = RestoreLimit { key: Key::Account { number: 22 }, set: RestoreSet::Accounts, limit: 1 };
    assert_eq!(restore_limit(&world.domain), Some(expected));
    failed(&mut world, expected);
}
#[test]
fn excess_sources_in_one_account_refuse_stickily() {
    let mut world = World::with_start(
        Limits { sources: 1, ..LIMITS },
        configured(),
        &[account(20, 1, HOLDER, b""), source(20, 21, 0, Kind::Key), source(20, 22, 1, Kind::Key)],
    );
    assert_eq!(restore_limit(&world.domain), None, "source rows fit the aggregate table before linking");
    failed(
        &mut world,
        RestoreLimit {
            key: Key::Source { account: 20, number: 22 },
            set: RestoreSet::Sources { account: 20 },
            limit: 1,
        },
    );
}
#[test]
fn excess_total_sources_refuse_stickily() {
    let limits = Limits { accounts: 1, holder_accounts: 1, sources: 1, ..LIMITS };
    let mut world =
        World::with_start(limits, configured(), &[source(20, 21, 0, Kind::Key), source(20, 22, 1, Kind::Key)]);
    let expected =
        RestoreLimit { key: Key::Source { account: 20, number: 22 }, set: RestoreSet::TotalSources, limit: 1 };
    assert_eq!(restore_limit(&world.domain), Some(expected));
    failed(&mut world, expected);
}
#[test]
fn excess_lifetime_names_refuse_stickily() {
    let mut retired = issued(21, b"p");
    if let Stored::Issued { signed_out, .. } = &mut retired {
        *signed_out = true;
    }
    let mut world =
        World::with_start(Limits { program_names: 1, ..LIMITS }, configured(), &[retired, issued(22, b"q")]);
    let expected = RestoreLimit { key: Key::Issued { number: 22 }, set: RestoreSet::ProgramNames, limit: 1 };
    assert_eq!(restore_limit(&world.domain), Some(expected));
    failed(&mut world, expected);
}
#[test]
fn names_past_the_fixed_carrier_refuse_stickily() {
    for row in [issued(21, &[b'a'; 33]), program(20, 21, 0, &[b'a'; 33])] {
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
            [account(20, 1, HOLDER, b""), account(22, 1, HOLDER, b"")],
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

#[test]
#[should_panic(expected = "Source { account: 20, number: 21 } requires account Account { number: 20 }")]
fn missing_source_owners_panic_at_link() {
    let mut world = start(&[source(20, 21, 0, Kind::Key)]);
    world.step(Event::Want);
}
#[test]
#[should_panic(expected = "Source { account: 20, number: 21 } requires reservation Issued { number: 21 }")]
fn missing_reservations_are_not_repaired() {
    let mut world = start(&[account(20, 1, HOLDER, b""), program(20, 21, 0, b"p")]);
    world.step(Event::Want);
}
#[test]
#[should_panic(expected = "Issued { number: 21 } conflicts with Issued { number: 22 }")]
fn duplicate_reservations_panic_with_both_keys() {
    let mut world = start(&[issued(21, b"p"), issued(22, b"p")]);
    world.step(Event::Want);
}
#[test]
#[should_panic(expected = "disagrees with reservation Issued { number: 21 }")]
fn reservation_names_must_link_to_the_source() {
    let mut world = start(&[account(20, 1, HOLDER, b""), program(20, 21, 0, b"p"), issued(21, b"q")]);
    world.step(Event::Want);
}
#[test]
#[should_panic(expected = "disagrees with reservation Issued { number: 21 }")]
fn reservation_owners_must_link_to_the_source() {
    let mut world = start(&[account(20, 1, Holder::Party(8), b""), program(20, 21, 0, b"p"), issued(21, b"p")]);
    world.step(Event::Want);
}
#[test]
#[should_panic(expected = "conflicts at place 0 with source Some(21)")]
fn conflicting_source_places_panic_with_keys() {
    let mut world = start(&[account(20, 1, HOLDER, b""), source(20, 21, 0, Kind::Key), source(20, 22, 0, Kind::Key)]);
    world.step(Event::Want);
}

#[test]
#[should_panic(expected = "selected once before any operational input")]
fn accepted_credential_prevents_late_start_selection() {
    let mut world = World::with(configured(), &[]);
    let valid = skein_lib::Duration::from_secs(60);
    let grant =
        jig_domain_accounts::Grant { form: jig_domain_accounts::Form::Application, account: 20, generation: 1, valid };
    assert_eq!(world.step(Event::Credential { number: 20, generation: 1, valid }), vec![Request::Granted { grant }]);
    assert_eq!(world.domain.credential_grant(20, world.now), Some(grant));
    assert_eq!(world.domain.next_deadline(), Some(world.now.saturating_add(valid)));
    assert_eq!(restore_limit(&world.domain), None);
    world.domain.start_restore();
}

fn assert_late_selection(domain: &mut Domain) {
    let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| domain.start_restore()))
        .expect_err("operational input consumes start selection");
    let message = panic.downcast_ref::<String>().map(String::as_str).or_else(|| panic.downcast_ref::<&str>().copied());
    assert!(message.is_some_and(|text| text.contains("selected once before any operational input")));
}

#[test]
fn ignored_terminals_prevent_late_start_selection() {
    for event in [
        Event::NotOurs { source: 21 },
        Event::Component(jig_domain_accounts::FromComponent::Closed),
        Event::Finishing { source: 21 },
        Event::ProgramTargets { source: 21, hosts: Box::new([]) },
    ] {
        let mut world = World::with(configured(), &[]);
        assert!(world.step(event).is_empty());
        assert_eq!(restore_limit(&world.domain), None);
        assert_late_selection(&mut world.domain);
    }
}

#[test]
#[should_panic(expected = "selected once before any operational input")]
fn refused_change_prevents_late_start_selection() {
    let mut world = World::with(configured(), &[]);
    let request = Token::new(1);
    assert_eq!(
        world.step(Event::Change { request, holder: HOLDER, change: Change::CheckProgram { source: 21 } }),
        vec![Request::Answer {
            request,
            answer: jig_domain_accounts::Answer::Refused { refusal: jig_domain_accounts::Refusal::Closing },
        }]
    );
    world.domain.start_restore();
}

#[test]
#[should_panic(expected = "selected once before any operational input")]
fn empty_fire_prevents_late_start_selection() {
    let mut world = World::with(configured(), &[]);
    assert!(world.fire_at(0).is_empty());
    assert_eq!(world.domain.next_deadline(), None);
    assert_eq!(restore_limit(&world.domain), None);
    world.domain.start_restore();
}

fn cold_program() -> (Domain, Env<Limits>, Queue<Request>) {
    let world = World::with(configured(), &[]);
    (world.domain, Env { now: world.now, wall: world.wall, limits: world.limits }, Queue::uncapped())
}

#[test]
#[should_panic(expected = "selected once before any operational input")]
fn refused_declaration_prevents_late_start_selection() {
    let (mut domain, env, mut out) = cold_program();
    assert_eq!(
        programs::declare(&mut domain, &env, HOLDER, 1, None, b"program".as_slice().into(), &mut out),
        Err(programs::DeclarationRefusal::Account(jig_domain_accounts::Refusal::Closing))
    );
    assert!(out.is_empty());
    domain.start_restore();
}

#[test]
#[should_panic(expected = "selected once before any operational input")]
fn refused_program_check_prevents_late_start_selection() {
    let (mut domain, _, mut out) = cold_program();
    assert_eq!(programs::check(&mut domain, HOLDER, 21, &mut out), Err(jig_domain_accounts::Refusal::Closing));
    assert!(out.is_empty());
    domain.start_restore();
}

#[test]
#[should_panic(expected = "selected once before any operational input")]
fn refused_program_sign_out_prevents_late_start_selection() {
    let (mut domain, env, mut out) = cold_program();
    assert_eq!(
        programs::sign_out(&mut domain, &env, Token::new(1), HOLDER, 21, &mut out),
        Err(jig_domain_accounts::Refusal::Closing)
    );
    assert!(out.is_empty());
    domain.start_restore();
}

#[test]
#[should_panic(expected = "selected once before any operational input")]
fn closing_prevents_late_start_selection() {
    let mut world = World::with(configured(), &[]);
    assert_eq!(world.step(Event::Close), vec![Request::Component(jig_domain_accounts::ToComponent::Close)]);
    assert!(world.step(Event::Close).is_empty());
    world.domain.start_restore();
}

#[test]
fn queries_and_output_drains_permit_start_selection() {
    let mut world = World::with(configured(), &[]);
    assert_eq!(world.domain.pop_fact(), None);
    assert_eq!(world.domain.pop_ended(), None);
    assert_eq!(world.domain.accounts(), 0);
    assert_eq!(world.domain.credential_grant(20, world.now), None);
    assert_eq!(world.domain.next_deadline(), None);
    assert_eq!(programs::issued(&world.domain, b"program"), None);
    assert_eq!(programs::name(&world.domain, 21, HOLDER), None);
    assert_eq!(world.domain.sign_in(&LIMITS, HOLDER, 1, b""), Err(jig_domain_accounts::Refusal::Subject));
    world.domain.start_restore();
    world.step(Event::Want);
    assert!(world.domain.restore_finished());
    assert_eq!(restore_limit(&world.domain), None);
}

#[test]
#[should_panic(expected = "selected once before any operational input")]
fn selecting_start_twice_is_a_programmer_error() {
    start(&[]).domain.start_restore();
}
#[test]
#[should_panic(expected = "selected once before any operational input")]
fn selecting_start_after_an_empty_counter_is_a_programmer_error() {
    World::with(configured(), &[Stored::Next { number: 0 }]).domain.start_restore();
}
#[test]
#[should_panic(expected = "selected once before any operational input")]
fn selecting_start_after_a_dropped_row_is_a_programmer_error() {
    World::with(configured(), &[source(20, 21, 0, Kind::Key)]).domain.start_restore();
}
#[test]
#[should_panic(expected = "selected once before any operational input")]
fn selecting_start_after_a_zero_row_want_is_a_programmer_error() {
    let mut world = World::with(configured(), &[]);
    world.step(Event::Want);
    world.domain.start_restore();
}
#[test]
fn an_empty_start_can_be_selected_and_finishes_once() {
    let cold = Domain::new(&LIMITS, configured());
    assert_eq!(restore_limit(&cold), None);
    let mut world = start(&[]);
    assert_eq!(restore_limit(&world.domain), None);
    assert!(!world.domain.restore_finished());
    world.step(Event::Want);
    assert!(world.domain.restore_finished());
    assert!(world.step(Event::Want).is_empty());
    assert_eq!(restore_limit(&world.domain), None);
}

#[test]
fn held_wall_resets_and_runtime_movements_survive_start() {
    let mut world = start(&[]);
    world.wall = Wall::from_nanos(1_000_000_000_000);
    world.restore(vec![
        account(20, 1, HOLDER, b"subject"),
        Stored::Source {
            account: 20,
            number: 21,
            place: 4,
            kind: Kind::SignIn,
            name: Box::new([]),
            held: Some(Held::Limited { reset: Some(Wall::from_nanos(1_060_000_000_000)) }),
        },
        source(20, 22, 9, Kind::Key),
        Stored::Next { number: 90 },
    ]);
    world.step(Event::Want);
    assert_eq!(world.domain.next_deadline(), Some(skein_lib::Time::from_nanos(60_000_000_000)));
    assert!(matches!(world.domain.source(21).expect("kept").state, State::Limited { until: Some(_) }));
    assert_eq!(world.domain.next_number(), 90);
    world.fire_at(60);
    assert_eq!(world.domain.source(21).expect("kept").state, State::Asking);
    assert_eq!(world.domain.next_deadline(), None);
    let out =
        world.step(Event::Change { request: Token::new(1), holder: HOLDER, change: Change::MoveOut { account: 20 } });
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
        &[account(20, 99, HOLDER, b""), program(20, 21, 0, b"p"), issued(21, b"p")],
    );
    world.step(Event::Want);
    assert!(world.domain.account(20).expect("kept").plan());
    assert!(world.domain.source_grant(21, world.now).is_some());
}

#[test]
fn passed_resets_are_lifted_and_moving_holds_refresh() {
    let mut world = start(&[]);
    world.wall = Wall::from_nanos(100);
    let mut rows = vec![account(20, 1, HOLDER, b"")];
    for (number, held) in
        [(21, Held::Limited { reset: Some(Wall::from_nanos(99)) }), (22, Held::Moving), (23, Held::Refused)]
    {
        rows.push(Stored::Source {
            account: 20,
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
        let mut world = start(&[account(20, 1, HOLDER, b""), source(20, 21, place, Kind::Key)]);
        world.step(Event::Want);
        let out = world.step(Event::Change {
            request: Token::new(1),
            holder: HOLDER,
            change: Change::DeclareProgram { provider: 1, join: Some(20), name: b"p".as_slice().into() },
        });
        assert!(out.iter().any(|rq| matches!(
            rq,
            Request::Answer { answer: jig_domain_accounts::Answer::ProgramDeclared { .. }, .. }
        )));
        let sources = world.domain.account(20).expect("kept").sources();
        assert_eq!(sources.len(), 2);
        assert_eq!(sources.iter().map(|(_, n)| *n).collect::<Vec<_>>(), vec![21, 22]);
    }
}

#[test]
fn live_keys_append_after_sparse_restored_places() {
    for place in [1, 42, u32::MAX] {
        let mut world = start(&[account(20, 1, HOLDER, b""), source(20, 21, place, Kind::Key)]);
        world.step(Event::Want);
        world.step(Event::Change {
            request: Token::new(1),
            holder: HOLDER,
            change: Change::AddKey { provider: 1, join: Some(20), carried: 1 << 63 },
        });
        world.step(Event::Component(jig_domain_accounts::FromComponent::Granted {
            account: 22,
            generation: 1,
            form: jig_domain_accounts::Form::Credential(jig_domain_accounts::Scheme::Key),
            valid: skein_lib::Duration::from_secs(60),
        }));
        let sources = world.domain.account(20).expect("kept").sources();
        assert_eq!(sources.len(), 2);
        assert_eq!(sources.iter().map(|(_, n)| *n).collect::<Vec<_>>(), vec![21, 22]);
    }
}

fn keep(store: &mut std::collections::BTreeMap<jig_domain_accounts::Key, Stored>, out: &[Request]) {
    for rq in out {
        match rq {
            Request::Save { record } => {
                store.insert(record.key(), record.clone());
            }
            Request::Erase { key } => {
                store.remove(key);
            }
            Request::Answer { .. }
            | Request::Programs { .. }
            | Request::CheckProgram { .. }
            | Request::SignOutProgram { .. }
            | Request::Component(_)
            | Request::Signing(_)
            | Request::Granted { .. }
            | Request::Closed => {}
        }
    }
}

/// Read the durable account fields through the reader's public doors.
fn account_readback(world: &World) -> Vec<Stored> {
    world
        .domain
        .accounts_by_number()
        .iter()
        .map(|(number, row)| Stored::Account {
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
    use jig_domain_accounts::{Form, FromComponent, Scheme, Signing};
    let mut rng = skein_lib::Rng::new(seed);
    let mut world = World::with(configured(), &[]);
    let mut store = std::collections::BTreeMap::new();
    keep(&mut store, &world.step(Event::Want));
    for request in 0..80_u64 {
        let numbers: Vec<_> = world.domain.accounts_by_number().iter().map(|(n, _)| *n).collect();
        let account = numbers.get(usize::try_from(rng.below(3)).expect("small index")).copied().unwrap_or(u32::MAX);
        let sources = world.domain.source_views();
        let source = sources.get(u32::try_from(rng.below(9)).expect("small index")).map_or(u32::MAX, |f| f.source);
        let change = match rng.below(9) {
            0 => Change::DeclareProgram {
                provider: 1,
                join: None,
                name: format!("program-{request}").into_bytes().into(),
            },
            1 => Change::DeclareProgram {
                provider: 1,
                join: Some(account),
                name: format!("program-{request}").into_bytes().into(),
            },
            2 => Change::RemoveKey { source },
            3 => Change::MoveAccount { account, place: u32::try_from(rng.below(3)).expect("small place") },
            4 => Change::MoveSource { account, source, place: u32::try_from(rng.below(3)).expect("small place") },
            5 => Change::SetUses { account, uses: Some(Box::new([Use::Party(request)])) },
            6 => Change::Resume { source },
            7 => Change::AddKey { provider: 1, join: Some(account), carried: 1 << 63 },
            _ => Change::AddKey { provider: 1, join: None, carried: 1 << 63 },
        };
        let out = world.step(Event::Change { request: Token::new(request), holder: HOLDER, change });
        keep(&mut store, &out);
        for rq in out {
            if let Request::Signing(Signing::Keep { source, .. }) = rq {
                let out = world.step(Event::Component(FromComponent::Granted {
                    account: source,
                    generation: 1,
                    form: Form::Credential(Scheme::Key),
                    valid: skein_lib::Duration::from_secs(100),
                }));
                keep(&mut store, &out);
            }
        }
        if rng.chance(25) {
            let out = world.step(Event::Limited { account: source, reset: None });
            keep(&mut store, &out);
        }
        let rows: Vec<_> = store.values().cloned().collect();
        let mut again = start(&rows);
        let out = again.step(Event::Want);
        assert!(again.domain.restore_finished(), "seed {seed}, step {request}");
        assert_eq!(again.domain.unrestored(), 0);
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
    assert_eq!(writer_readback(61), writer_readback(61));
}

#[test]
fn stored_number_rules_are_not_replayed_as_links() {
    let rows = [
        account(7, 1, HOLDER, b"subject"),
        source(7, 8, 1, Kind::Key),
        source(7, 7, 0, Kind::SignIn),
        account(9, 5, Holder::Party(8), b"other"),
        source(9, 10, 0, Kind::SignIn),
    ];
    let mut world = start(&rows);
    world.step(Event::Want);
    assert!(world.domain.restore_finished());
    assert_eq!(world.domain.unrestored(), 0);
    assert_eq!(world.domain.account(7).expect("account identity").sources().len(), 2);
    assert_eq!(world.domain.source(7).expect("source identity").account, 7);
    assert!(world.domain.account(9).is_some());
    assert_eq!(world.domain.next_number(), 11);
}

#[test]
fn removing_a_key_rebuilds_plans_without_a_configured_provider() {
    for remaining in [Kind::Key, Kind::Program] {
        let mut rows = vec![account(20, 99, HOLDER, b"subject"), source(20, 21, 0, Kind::Key)];
        if remaining == Kind::Program {
            rows.extend([program(20, 22, 42, b"p"), issued(22, b"p")]);
        } else {
            rows.push(source(20, 22, 42, remaining));
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
        let kept = world.domain.account(20).expect("remaining durable identity");
        assert_eq!(kept.provider(), 99);
        assert_eq!(kept.plan(), remaining == Kind::Program);
        assert_eq!(kept.sources().get(&0), Some(&22));
        assert!(out.contains(&Request::Save {
            record: if remaining == Kind::Program { program(20, 22, 0, b"p") } else { source(20, 22, 0, remaining) }
        }));
    }
}

#[test]
fn sign_in_renews_after_the_last_representable_stored_place() {
    for start_mode in [false, true] {
        let rows = [account(20, 1, HOLDER, b"subject"), source(20, 21, u32::MAX, Kind::Key)];
        let mut world = if start_mode { start(&rows) } else { World::with(configured(), &rows) };
        world.step(Event::Want);
        if start_mode {
            assert_eq!(world.domain.account(20).expect("kept").sources().get(&u32::MAX), Some(&21));
        }
        world.step(Event::SignIn { request: Token::new(7), holder: HOLDER, by: 7, provider: 1 });
        world.step(Event::Visited { source: 22, address: b"https://issuer.test/visit".as_slice().into() });
        let out = world.step(Event::SignedIn {
            source: 22,
            subject: b"subject".as_slice().into(),
            display: Box::new([]),
            declined: false,
        });
        let kept = world.domain.account(20).expect("renewed account");
        assert_eq!(kept.sources().len(), 2);
        assert_eq!(kept.sources().get(&0), Some(&21));
        assert_eq!(kept.sources().get(&1), Some(&22));
        assert!(
            world
                .ended()
                .iter()
                .any(|ended| matches!(ended.end, jig_domain_accounts::End::Renewed { account: 20, declined: false }))
        );
        assert!(out.contains(&Request::Save { record: source(20, 22, 1, Kind::SignIn) }));
        if start_mode {
            assert!(out.contains(&Request::Save { record: source(20, 21, 0, Kind::Key) }));
        }
    }
}

#[test]
fn a_moved_sign_in_renumbers_only_once_adoption_is_kept() {
    for kept in [false, true] {
        let mut world = start(&[account(20, 1, HOLDER, b"subject"), source(20, 21, u32::MAX, Kind::Key)]);
        world.step(Event::Want);
        world.step(Event::Change {
            request: Token::new(7),
            holder: HOLDER,
            change: Change::MoveIn { carried: 1 << 63 },
        });
        assert_eq!(
            world.step(Event::Opened {
                source: 22,
                provider: 1,
                subject: b"subject".as_slice().into(),
                display: Box::new([]),
            }),
            vec![Request::Signing(jig_domain_accounts::Signing::Adopt { source: 22, carried: 1 << 63 })]
        );
        assert_eq!(world.domain.account(20).expect("kept").sources().get(&u32::MAX), Some(&21));
        let out = world.step(Event::Adopted { source: 22, kept });
        let sources = world.domain.account(20).expect("kept").sources();
        if kept {
            assert_eq!(sources.get(&0), Some(&21));
            assert_eq!(sources.get(&1), Some(&22));
            assert!(out.contains(&Request::Save { record: source(20, 21, 0, Kind::Key) }));
            assert!(out.contains(&Request::Answer {
                request: Token::new(7),
                answer: jig_domain_accounts::Answer::MovedIn { account: 20, renewed: true },
            }));
            assert_eq!(world.domain.source(22).expect("adopted source").state, State::Moving);
        } else {
            assert_eq!(sources.len(), 1);
            assert_eq!(sources.get(&u32::MAX), Some(&21));
            assert!(!out.iter().any(|rq| matches!(rq, Request::Save { .. })));
        }
    }
}
