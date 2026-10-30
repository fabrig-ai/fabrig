//! Startup fit barriers and one retained wall observation.
#![expect(clippy::disallowed_macros, reason = "a boundary fixture injects one oversized decoded field")]
use crate::{Domain, Event, Request, Work};
use skein_lib::{Env, Queue, Wall};
use temper_engine_domain_forge as forge;

#[test]
fn restart_captures_wall_before_loading_and_repeated_restart_keeps_it() {
    let limits = super::resume_fixture::limits();
    let mut d = Domain::new(super::resume_fixture::config(7), &limits).unwrap();
    let mut env = Env { now: skein_lib::Time::ZERO, wall: Wall::from_nanos(100), limits };
    assert_eq!(d.start_wall, None);
    crate::try_step(&mut d, &env, Event::Restart { pending: false }).unwrap();
    assert_eq!(d.start_wall, Some(Wall::from_nanos(100)));
    env.wall = Wall::from_nanos(200);
    crate::try_step(&mut d, &env, Event::Restart { pending: true }).unwrap();
    assert_eq!(d.start_wall, Some(Wall::from_nanos(100)));
    let mut out = Queue::uncapped();
    forge::step(
        &mut d.forge,
        &crate::translate::environment_forge(&env),
        forge::Event::Restore {
            record: forge::Stored::Application {
                token: 1,
                until: Wall::from_nanos(101),
                outcome: forge::ApplicationOutcome::ForgeRefused(forge::AdoptionRefusal::Refused),
            },
        },
        &mut out,
    );
    assert!(
        d.forge.application_outcome(skein_lib::Token::new(1)).is_some(),
        "forge receives root's original observation"
    );
}

#[test]
fn first_fit_failure_prevents_translation_linking_and_repeated_journal_release() {
    let (mut d, env) = super::resume_fixture::root();
    // Queue an output behind the eventual one-shot Stop to exercise the persistent release guard.
    let mut decision = d.journal.decide();
    decision.push_held(Request::Store(crate::StoreRequest::Switch));
    d.journal.accept(decision);
    d.work.push(Work::Forge(forge::Event::Restore {
        record: forge::Stored::Binding { address: vec![0; 255].into_boxed_slice(), subject: Box::new([]) },
    }));
    d.work.push(Work::ForgeRestored);
    let mut decision = crate::Decision::decide(&mut d.journal, &env.limits.journal);
    crate::route::route_into(&mut d, &env, &mut decision);
    let failure = *d.forge_fit_failure().unwrap();
    assert_eq!(failure.set, forge::RestoreSet::BindingAddressBytes);
    let mut out = Queue::uncapped();
    crate::release(&mut d, &env, &mut out);
    assert_eq!(out.pop(), Some(Request::Stop));
    assert!(out.is_empty());
    for _ in 0_u32..3 {
        crate::release(&mut d, &env, &mut out);
        crate::try_step(&mut d, &env, Event::Resume).unwrap();
        crate::fire(&mut d, &env);
    }
    assert!(out.is_empty());
    assert_eq!(d.forge_fit_failure(), Some(&failure));
}

#[test]
fn derived_fit_refuses_before_missing_links_at_the_restore_terminal() {
    let (mut d, env) = super::resume_fixture::root();
    let l = env.limits.forge;
    let mut number = 0;
    let mut task = 100;
    while number <= l.client.resources {
        let mut resources = skein_lib::List::with_capacity(l.resources_per_task);
        while resources.room() > 0 && number <= l.client.resources {
            resources
                .push(forge::Name { forge: 1, repository: 999, what: forge::What::Pull(u64::from(number)) })
                .unwrap();
            number += 1;
        }
        d.work.push(Work::Forge(forge::Event::Restore {
            record: forge::Stored::Names { task, resources: resources.into_boxed() },
        }));
        task += 1;
    }
    d.work.push(Work::ForgeRestored);
    let mut decision = crate::Decision::decide(&mut d.journal, &env.limits.journal);
    crate::route::route_into(&mut d, &env, &mut decision);
    assert_eq!(
        d.forge_fit_failure(),
        Some(&forge::RestoreFit {
            family: Some(forge::RecordKind::Names),
            set: forge::RestoreSet::Watches,
            held: u64::from(l.client.resources),
            additional: 1,
            limit: u64::from(l.client.resources)
        })
    );
    assert!(!d.connectors_restored());
}
