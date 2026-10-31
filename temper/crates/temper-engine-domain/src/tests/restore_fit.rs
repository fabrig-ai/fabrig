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
    let (mut d, env) = super::resume::at_phase(false, jig_domain_core::start::Phase::Link);
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
    assert_eq!(
        d.start_stop(),
        Some(crate::StartStop::ForgeFit { phase: Some(jig_domain_core::start::Phase::Link), failure })
    );
    assert_eq!(d.core.start_failure(), None);
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
    let (mut d, env) = super::resume::at_phase(false, jig_domain_core::start::Phase::Link);
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
    assert_eq!(
        d.start_stop(),
        Some(crate::StartStop::ForgeFit {
            phase: Some(jig_domain_core::start::Phase::Link),
            failure: *d.forge_fit_failure().unwrap()
        })
    );
}

fn starting() -> (Domain, Env<crate::Limits>) {
    let limits = super::resume_fixture::limits();
    let mut domain = Domain::new(super::resume_fixture::config(7), &limits).unwrap();
    let env = Env { now: skein_lib::Time::ZERO, wall: Wall::EPOCH, limits };
    crate::step(&mut domain, &env, Event::Restart { pending: false });
    (domain, env)
}

#[test]
fn an_unloaded_startup_page_keeps_its_load_cause_and_never_stops_the_journal() {
    let (mut domain, env) = starting();
    let range = crate::Range::Core(crate::CoreRange::Deployment);
    let mut issued = Queue::uncapped();
    let owner =
        crate::loads::begin(&mut domain.loads, skein_lib::Token::new(u64::MAX), range, None, 1, &mut issued).unwrap();
    crate::step(&mut domain, &env, Event::Store(crate::StoreInput::Unloaded { owner }));
    let expected = Some(crate::StartStop::Load(crate::loads::RestoreFailure {
        range: crate::loads::Kind::Deployment,
        failure: crate::loads::Failure::Store,
    }));
    assert_eq!(domain.start_stop(), expected);
    assert_eq!(domain.start_phase(), Some(jig_domain_core::start::Phase::LoadCore));
    assert!(!domain.journal.stopped());
    assert_eq!(domain.core.start_failure(), None);
    let mut out = Queue::uncapped();
    crate::release(&mut domain, &env, &mut out);
    assert_eq!(out.pop(), Some(Request::Stop));
    for _ in 0_u32..3 {
        crate::step(&mut domain, &env, Event::Resume);
        crate::release(&mut domain, &env, &mut out);
    }
    assert!(out.is_empty());
    assert_eq!(domain.start_stop(), expected);
}

#[test]
fn a_cut_startup_page_stops_as_load_bytes_before_restoring_its_prefix() {
    use jig_domain_people as people;
    let (mut domain, env) = starting();
    let mut limits = env.limits.loads;
    limits.bytes = u32::try_from(size_of::<crate::Record>()).unwrap();
    domain.loads = crate::loads::Loads::new(&limits);
    let range = crate::Range::Core(crate::CoreRange::People);
    domain.startup_range = Some(range.clone());
    let mut issued = Queue::uncapped();
    let owner =
        crate::loads::begin(&mut domain.loads, skein_lib::Token::new(u64::MAX), range, None, 1, &mut issued).unwrap();
    let person = people::Stored::Person {
        number: 1,
        identity: people::Identity {
            key: people::IdentityKey { provider: 0, subject: b"owner".as_slice().into() },
            login: Box::new([]),
            name: Box::new([]),
        },
    };
    crate::step(
        &mut domain,
        &env,
        Event::Store(crate::StoreInput::Loaded {
            owner,
            rows: Box::new([crate::Record::Core(jig_domain_core::Record::People(person))]),
            next: None,
            texts: None,
        }),
    );
    assert_eq!(
        domain.start_stop(),
        Some(crate::StartStop::Load(crate::loads::RestoreFailure {
            range: crate::loads::Kind::People,
            failure: crate::loads::Failure::Bytes,
        }))
    );
    assert_eq!(domain.core.start_failure(), None);
    assert!(!domain.journal.stopped());
    assert!(!domain.ready());
}

#[test]
fn a_stored_live_set_past_its_limit_keeps_the_core_phase_and_typed_fit() {
    use jig_domain_people as people;
    let (mut domain, env) = starting();
    let mut decision = crate::Decision::decide(&mut domain.journal, &env.limits.journal);
    let bound = u64::from(env.limits.people.people);
    for number in 1..=bound + 1 {
        crate::restart::restore_page_row(
            &mut domain,
            &env,
            crate::Record::Core(jig_domain_core::Record::People(people::Stored::Person {
                number,
                identity: people::Identity {
                    key: people::IdentityKey { provider: 0, subject: number.to_be_bytes().into() },
                    login: Box::new([]),
                    name: Box::new([]),
                },
            })),
        );
        crate::route::route_into(&mut domain, &env, &mut decision);
    }
    let expected = crate::StartStop::Core {
        phase: Some(jig_domain_core::start::Phase::LoadCore),
        failure: jig_domain_core::start::Failure::LiveSet {
            set: jig_domain_core::start::LiveSet::PeopleBound(people::RestoreBound::People),
            key: jig_domain_core::Key::People(people::Key::Person(bound + 1)),
            held: bound + 1,
            limit: bound,
        },
    };
    assert_eq!(domain.start_stop(), Some(expected.clone()));
    domain.stop_start(crate::StartStop::Binding);
    assert_eq!(domain.start_stop(), Some(expected));
    assert!(!domain.journal.stopped());
    assert!(!domain.ready());
}

#[test]
fn a_task_correlation_past_its_limit_records_the_row_entrance_fit() {
    use jig_domain_core::{Key, Record};
    use jig_domain_tasks as tasks;
    let (source, _, goal) = super::projection_settled::live_goal(None);
    let mut task = source.tasks().task(goal).unwrap().clone();
    task.executor = tasks::Executor::Agent { charter: 1 };
    task.attempt = 1;
    let (mut domain, env) = starting();
    let bound = u64::from(env.limits.tasks.tasks);
    for number in 1..=bound + 1 {
        task.number = number;
        crate::restart::restore_page_row(
            &mut domain,
            &env,
            crate::Record::Core(Record::Tasks(tasks::Stored::Live(Box::new(task.clone())))),
        );
        if number <= bound {
            assert_eq!(domain.core.start_failure(), None);
        }
    }
    assert_eq!(
        domain.start_stop(),
        Some(crate::StartStop::Core {
            phase: Some(jig_domain_core::start::Phase::LoadCore),
            failure: jig_domain_core::start::Failure::LiveSet {
                set: jig_domain_core::start::LiveSet::RunProofs,
                key: Key::Tasks(tasks::Key::Live(bound + 1)),
                held: bound + 1,
                limit: bound,
            },
        })
    );
    assert!(!domain.ready());
}

#[test]
fn link_preserves_each_committed_waiting_escalation() {
    use jig_domain_core::Record;
    use jig_domain_people as people;
    use jig_domain_tasks as tasks;
    let (source, _, goal) = super::projection_settled::live_goal(None);
    let mut task = source.tasks().task(goal).unwrap().clone();
    task.requester = tasks::Party::Person(1);
    task.phase =
        tasks::Phase::Held { was: tasks::Was::Active(tasks::Active::Due), why: tasks::Hold::Budget, source: None };
    let role = tasks::EscalationHolder::Role { project: 1, role: 0 };
    for holder in [tasks::EscalationHolder::Person(1), role] {
        task.escalation = tasks::Escalation::Waiting { revision: 1, holder, entry: 1, since: Wall::EPOCH };
        let records = [
            crate::Record::Core(Record::Tasks(tasks::Stored::Live(Box::new(task.clone())))),
            crate::Record::Core(Record::Tasks(tasks::Stored::Ledger(*source.tasks().funding(task.funder).unwrap()))),
            crate::Record::Core(Record::People(people::Stored::Person {
                number: 1,
                identity: people::Identity {
                    key: people::IdentityKey { provider: 0, subject: 7_u64.to_be_bytes().into() },
                    login: Box::new([]),
                    name: Box::new([]),
                },
            })),
        ];
        let (restored, _) = super::resume_fixture::root_with_records(
            super::resume_fixture::config(7),
            &super::resume_fixture::limits(),
            &records,
        );
        assert_eq!(restored.tasks().task(goal).unwrap().escalation, task.escalation);
        assert_eq!(restored.core.start_failure(), None);
    }
}
