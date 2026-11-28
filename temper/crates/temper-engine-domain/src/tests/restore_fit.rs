//! Startup fit barriers and one retained wall observation.
#![expect(clippy::disallowed_macros, reason = "a boundary fixture injects one oversized decoded field")]
use super::resume_fixture::{StartStore, Until};
use crate::{CoreRange, Domain, Event, Range, Record as RootRecord, Request, StoreInput};
use jig_domain_core::start::Phase;
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
    StartStore::new().advance(&mut d, &env, Until::Phase(Phase::RestoreConnector { connector: 0 }));
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
    let (mut d, env) = super::resume::at_phase(false, Phase::RestoreConnector { connector: 0 });
    let owner = load(&mut d, &env, Range::Forge);
    // Queue an output behind the eventual one-shot Stop to exercise the persistent release guard.
    let mut decision = d.journal.decide();
    decision.push_held(Request::Store(crate::StoreRequest::Switch));
    d.journal.accept(decision);
    let row = forge::Stored::Binding { address: vec![0; 255].into_boxed_slice(), subject: Box::new([]) };
    crate::step(
        &mut d,
        &env,
        Event::Store(StoreInput::Loaded {
            owner,
            rows: Box::new([RootRecord::Forge { row: Box::new(row) }]),
            next: None,
        }),
    );
    let failure = *d.forge_fit_failure().unwrap();
    assert_eq!(failure.set, forge::RestoreSet::BindingAddressBytes);
    assert_eq!(
        d.start_stop(),
        Some(crate::StartStop::ForgeFit { phase: Some(Phase::RestoreConnector { connector: 0 }), failure })
    );
    assert_eq!(d.engine.core().start_failure(), None);
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
    let mut limits = super::resume_fixture::limits();
    limits.forge.client.resources = limits.forge.resources_per_task - 1;
    limits.forge.holds = limits.forge.client.resources;
    let (mut d, env) = super::resume::at_phase_with_limits(false, Phase::RestoreConnector { connector: 0 }, &limits);
    let (source, _, task) = super::projection_settled::live_goal(None);
    jig_domain_tasks::step(
        &mut d.engine.core_mut().tasks,
        &env.with(env.limits.tasks),
        jig_domain_tasks::Event::Restore {
            record: jig_domain_tasks::Stored::Live(Box::new(source.tasks().task(task).unwrap().clone())),
        },
        &mut Queue::uncapped(),
    );
    let l = env.limits.forge;
    let mut resources = skein_lib::List::with_capacity(l.resources_per_task);
    for number in 0..=l.client.resources {
        resources.push(forge::Name { forge: 1, repository: 999, what: forge::What::Pull(u64::from(number)) }).unwrap();
    }
    forge_page(&mut d, &env, forge::Stored::Names { task, resources: resources.into_boxed() });
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
            phase: Some(Phase::RestoreConnector { connector: 0 }),
            failure: *d.forge_fit_failure().unwrap()
        })
    );
}

fn load(domain: &mut Domain, env: &Env<crate::Limits>, range: Range) -> skein_lib::Token {
    StartStore::new().advance(domain, env, Until::Load(range)).expect("startup load")
}

fn forge_page(domain: &mut Domain, env: &Env<crate::Limits>, row: forge::Stored) {
    let owner = load(domain, env, Range::Forge);
    crate::step(
        domain,
        env,
        Event::Store(StoreInput::Loaded {
            owner,
            rows: Box::new([RootRecord::Forge { row: Box::new(row) }]),
            next: None,
        }),
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
    let owner = load(&mut domain, &env, Range::Core(CoreRange::Deployment));
    crate::step(&mut domain, &env, Event::Store(StoreInput::Unloaded { owner }));
    let expected = Some(crate::StartStop::Load(crate::loads::RestoreFailure {
        range: crate::loads::Kind::Deployment,
        failure: crate::loads::Failure::Store,
    }));
    assert_eq!(domain.start_stop(), expected);
    assert_eq!(domain.start_phase(), Some(Phase::LoadCore));
    assert!(!domain.journal.stopped());
    assert_eq!(domain.engine.core().start_failure(), None);
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
    let owner = load(&mut domain, &env, Range::Core(CoreRange::People));
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
        Event::Store(StoreInput::Loaded {
            owner,
            rows: Box::new([crate::Record::Core(jig_domain_core::Record::People(person))]),
            next: None,
        }),
    );
    assert_eq!(
        domain.start_stop(),
        Some(crate::StartStop::Load(crate::loads::RestoreFailure {
            range: crate::loads::Kind::People,
            failure: crate::loads::Failure::Bytes,
        }))
    );
    assert_eq!(domain.engine.core().start_failure(), None);
    assert!(!domain.journal.stopped());
    assert!(!domain.ready());
}

#[test]
fn a_stored_live_set_past_its_limit_keeps_the_core_phase_and_typed_fit() {
    use jig_domain_people as people;
    let (mut domain, env) = starting();
    let mut store = StartStore::new();
    let bound = u64::from(env.limits.people.people);
    for number in 1..=bound + 1 {
        store.keep(crate::Record::Core(jig_domain_core::Record::People(people::Stored::Person {
            number,
            identity: people::Identity {
                key: people::IdentityKey { provider: 0, subject: number.to_be_bytes().into() },
                login: Box::new([]),
                name: Box::new([]),
            },
        })));
    }
    store.advance(&mut domain, &env, Until::Open);
    let expected = crate::StartStop::Core {
        phase: Some(Phase::LoadCore),
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
    let mut store = StartStore::new();
    for number in 1..=bound + 1 {
        task.number = number;
        store.keep(RootRecord::Core(Record::Tasks(tasks::Stored::Live(Box::new(task.clone())))));
    }
    store.advance(&mut domain, &env, Until::Open);
    assert_eq!(
        domain.start_stop(),
        Some(crate::StartStop::Core {
            phase: Some(Phase::LoadCore),
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
fn link_preserves_committed_waiting_escalations() {
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
        assert_eq!(restored.engine.core().start_failure(), None);
    }
}

#[test]
fn a_lone_change_part_stops_the_root_as_an_unreadable_row_before_linking() {
    let (mut d, env) = super::resume::at_phase(false, Phase::RestoreConnector { connector: 0 });
    let task = 12;
    forge_page(
        &mut d,
        &env,
        forge::Stored::ChangeCold(Box::new(forge::ChangeCold {
            source: None,
            task,
            repository: skein_forge::api::Repository { forge: 1, repository: 999 },
            branch: b"temper/12".as_slice().into(),
            base: b"main".as_slice().into(),
            milestone: None,
            person: None,
            goal_title: None,
            draft: None,
            goal: None,
            base_repair: false,
            producing_charter: 2,
            review_charter: 4,
            producing_llm: None,
            review_llm: None,
            repairs: None,
            catch_up: None,
            close: None,
            title: b"change".as_slice().into(),
            body: Box::new([]),
            follows: None,
            report: Box::new([]),
        })),
    );
    assert_eq!(d.start_stop(), Some(crate::StartStop::ForgeRow(forge::Key::ChangeCold(task))));
    assert!(!d.ready());
}
