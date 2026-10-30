//! Mode selection precedes operational input (jig/docs/design/domain/engine.md, 6).
use crate::{Domain, Event, HoldKind, Name, Party, Refusal, Request, ResourceAccess, step};
use skein_lib::{Env, Queue, Time, Wall};

use super::super::charters;
use super::{DUE, LIMITS, PERSON, UNNAMED};

const SEED: u64 = 91; // The revisions harness draw source.
const INITIAL_REVISION: u32 = 1;
const NEXT_REVISION: u32 = 2;
const CONNECTOR: u16 = 1;
const KIND: u16 = 1;
const OTHER_CONNECTOR: u16 = 2;
const PROJECT: u32 = 1;
const CHARTER: u32 = 1;

fn fresh() -> (Domain, Env<crate::Limits>, Queue<Request>) {
    (
        Domain::new(&LIMITS, SEED, INITIAL_REVISION, charters(), Box::new([0])),
        Env { now: Time::ZERO, wall: Wall::EPOCH, limits: LIMITS },
        Queue::uncapped(),
    )
}

fn resource() -> Name {
    Name { connector: CONNECTOR, path: Box::new([Box::from(b"resource".as_slice())]) }
}

#[test]
#[should_panic(expected = "start_restore must be selected once before restoration input")]
fn direct_resource_input_prevents_late_start_selection() {
    let (mut domain, env, mut out) = fresh();
    let name = resource();
    step(&mut domain, &env, Event::Resource { name: name.clone(), hold: HoldKind::Shared }, &mut out);
    assert!(out.is_empty());
    assert_eq!(domain.resource_hold(&name), Some(HoldKind::Shared));
    domain.start_restore();
}

fn rejects_selection(domain: &mut Domain) {
    assert!(std::panic::catch_unwind(core::panic::AssertUnwindSafe(|| domain.start_restore())).is_err());
    assert_eq!(domain.restore_failure(), None, "selection misuse is not a fit refusal");
}

#[test]
fn resource_updates_keep_the_latest_report_and_prevent_late_selection() {
    let (mut domain, env, mut out) = fresh();
    let name = resource();
    step(&mut domain, &env, Event::Resource { name: name.clone(), hold: HoldKind::Shared }, &mut out);
    let latest = HoldKind::Exclusive { taken: crate::Taken::Waits };
    step(&mut domain, &env, Event::Resource { name: name.clone(), hold: latest }, &mut out);
    assert_eq!(domain.resource_hold(&name), Some(latest));
    assert!(out.is_empty());
    rejects_selection(&mut domain);
}

#[test]
fn malformed_ignored_resource_reports_prevent_late_selection() {
    let (mut domain, env, mut out) = fresh();
    let bytes = usize::try_from(env.limits.hold_bytes).expect("fixture byte limit fits usize") + 1;
    let name = Name { connector: CONNECTOR, path: Box::new([vec![b'x'; bytes].into_boxed_slice()]) };
    step(&mut domain, &env, Event::Resource { name: name.clone(), hold: HoldKind::Shared }, &mut out);
    assert_eq!(domain.resource_hold(&name), None);
    assert!(out.is_empty());
    rejects_selection(&mut domain);
}

#[test]
fn resource_access_removal_prevents_late_selection_even_without_a_cached_entry() {
    let (mut domain, env, mut out) = fresh();
    let name = resource();
    step(
        &mut domain,
        &env,
        Event::ResourceAccess { project: PROJECT, name: name.clone(), access: ResourceAccess::Writable },
        &mut out,
    );
    assert_eq!(crate::holds::available_access(&domain, PROJECT, &name), ResourceAccess::Writable);
    assert!(out.is_empty());
    rejects_selection(&mut domain);
}

#[test]
fn resource_access_reports_retain_and_remove_restrictions() {
    let (mut domain, env, mut out) = fresh();
    let name = resource();
    for access in [ResourceAccess::Unavailable, ResourceAccess::Context, ResourceAccess::Writable] {
        step(&mut domain, &env, Event::ResourceAccess { project: PROJECT, name: name.clone(), access }, &mut out);
        assert_eq!(crate::holds::available_access(&domain, PROJECT, &name), access);
    }
    assert!(out.is_empty());
    rejects_selection(&mut domain);
}

#[test]
fn not_ready_refusals_prevent_late_selection() {
    let (mut domain, env, mut out) = fresh();
    step(
        &mut domain,
        &env,
        Event::Make {
            reply_to: super::super::revisions::to(),
            creator: Party::Person(PERSON),
            batch: Box::new([super::task(DUE, &[])]),
        },
        &mut out,
    );
    let Some(Request::Refused { problem, .. }) = out.pop() else { panic!("make before ready refuses") };
    assert_eq!(problem.why, Refusal::NotReady);
    assert!(out.is_empty());
    assert_eq!(domain.task(DUE), None);
    rejects_selection(&mut domain);
}

#[test]
fn ignored_notifications_prevent_late_selection() {
    let (mut domain, env, mut out) = fresh();
    step(&mut domain, &env, Event::PreparationFailed { task: UNNAMED }, &mut out);
    assert!(out.is_empty());
    assert_eq!(domain.task(UNNAMED), None);
    rejects_selection(&mut domain);
}

#[test]
fn zero_price_accounting_prevents_late_selection_without_looking_up_a_funder() {
    let (mut domain, env, mut out) = fresh();
    step(&mut domain, &env, Event::ChargeEffect { funder: crate::Funder::Task(UNNAMED), maximum: 0 }, &mut out);
    assert!(out.is_empty());
    assert_eq!(domain.task(UNNAMED), None);
    rejects_selection(&mut domain);
}

#[test]
fn fire_before_ready_prevents_late_selection_without_output() {
    let (mut domain, env, mut out) = fresh();
    crate::fire(&mut domain, &env, &mut out);
    assert!(out.is_empty());
    assert_eq!(domain.next_deadline(), None);
    rejects_selection(&mut domain);
}

#[test]
fn capacity_refusals_prevent_late_selection_with_zero_full_or_available_room() {
    let request = crate::Capacity {
        project: None,
        by: Party::Person(PERSON),
        action: crate::CapacityAction::Release,
        cause: crate::CapacityCause::Tasks { limit: 0 },
    };
    for (room, first_held, first_lost, second_held, second_lost) in [(0, 0, 1, 0, 2), (1, 1, 0, 1, 1), (2, 1, 0, 2, 0)]
    {
        let limits = crate::Limits { room_refusals: room, ..LIMITS };
        let mut domain = Domain::new(&limits, SEED, INITIAL_REVISION, charters(), Box::new([0]));
        domain.remember_capacity(Wall::EPOCH, request);
        assert_eq!(domain.capacity_mark(), 1);
        assert_eq!(domain.capacity_refusals().len(), first_held);
        assert_eq!(domain.capacity_refusals_lost(), first_lost);
        rejects_selection(&mut domain);
        domain.remember_capacity(Wall::EPOCH, request);
        assert_eq!(domain.capacity_mark(), 2);
        assert_eq!(domain.capacity_refusals().len(), second_held);
        assert_eq!(domain.capacity_refusals_lost(), second_lost);
    }
}

fn configure(domain: &mut Domain, env: &Env<crate::Limits>, out: &mut Queue<Request>) {
    step(
        domain,
        env,
        Event::Kinds {
            connector: CONNECTOR,
            kinds: Box::new([crate::Kind { connector: CONNECTOR, kind: KIND, hold: HoldKind::Shared }]),
        },
        out,
    );
    step(domain, env, Event::Charters { newest: NEXT_REVISION, charters: charters(), efforts: Box::new([0]) }, out);
    assert_eq!(domain.newest(), NEXT_REVISION);
    assert_eq!(domain.charter_models(CHARTER, NEXT_REVISION), Some([0].as_slice()));
    assert!(out.is_empty());
}

#[test]
fn kinds_and_charters_may_precede_or_follow_selection() {
    for select_first in [false, true] {
        let (mut domain, env, mut out) = fresh();
        if select_first {
            domain.start_restore();
        }
        configure(&mut domain, &env, &mut out);
        if !select_first {
            domain.start_restore();
        }
        step(&mut domain, &env, Event::Check, &mut out);
        assert!(domain.checked());
        step(&mut domain, &env, Event::Restored, &mut out);
        assert!(domain.ready());
        assert_eq!(domain.restore_failure(), None);
    }
}

#[test]
fn configuration_does_not_clear_previous_operational_input() {
    let (mut domain, env, mut out) = fresh();
    step(&mut domain, &env, Event::PreparationFailed { task: UNNAMED }, &mut out);
    configure(&mut domain, &env, &mut out);
    rejects_selection(&mut domain);
}

#[test]
fn kinds_before_selection_still_validate_connector_ownership() {
    let (mut domain, env, mut out) = fresh();
    assert!(
        std::panic::catch_unwind(core::panic::AssertUnwindSafe(|| step(
            &mut domain,
            &env,
            Event::Kinds {
                connector: CONNECTOR,
                kinds: Box::new([crate::Kind { connector: OTHER_CONNECTOR, kind: KIND, hold: HoldKind::Shared }])
            },
            &mut out
        )))
        .is_err()
    );
}

#[test]
fn charters_before_selection_still_validate_model_efforts() {
    let (mut domain, env, mut out) = fresh();
    assert!(
        std::panic::catch_unwind(core::panic::AssertUnwindSafe(|| step(
            &mut domain,
            &env,
            Event::Charters { newest: INITIAL_REVISION, charters: charters(), efforts: Box::new([]) },
            &mut out
        )))
        .is_err()
    );
}

#[test]
fn observations_and_maintenance_leave_fresh_selection_available() {
    let (mut domain, _, out) = fresh();
    assert_eq!(domain.task(DUE), None);
    assert_eq!(domain.resource_hold(&resource()), None);
    assert_eq!(domain.next_deadline(), None);
    assert_eq!(domain.restore_failure(), None);
    assert!(domain.capacity_refusals().is_empty());
    assert_eq!(domain.capacity_mark(), 0);
    assert_eq!(domain.pop_fact(), None);
    domain.reclaim();
    assert!(out.is_empty());
    domain.start_restore();
}

#[test]
fn selected_mode_accepts_operational_reports_before_loading_and_linking() {
    let source = super::one_task();
    let (mut domain, env, mut out) = fresh();
    domain.start_restore();
    let name = resource();
    step(&mut domain, &env, Event::Resource { name: name.clone(), hold: HoldKind::Shared }, &mut out);
    for record in source.stored.values().filter(|record| record.key().live()) {
        step(&mut domain, &env, Event::Restore { record: record.clone() }, &mut out);
    }
    step(&mut domain, &env, Event::Check, &mut out);
    assert!(domain.checked());
    assert_eq!(domain.resource_hold(&name), Some(HoldKind::Shared));
    assert!(domain.task(DUE).is_some());
    assert_eq!(domain.restore_failure(), None);
    step(&mut domain, &env, Event::Restored, &mut out);
    assert!(domain.ready());
}

#[test]
fn selection_is_still_single_use_before_any_operational_input() {
    let (mut domain, _, _) = fresh();
    domain.start_restore();
    rejects_selection(&mut domain);
}
